#[cfg(feature = "server")]
mod config;
#[cfg(feature = "server")]
pub mod entities;

use crate::App;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, RequestPartsExt};
use dioxus::prelude::{DioxusServerContext, FromServerContext};
use thiserror::Error;
use tracing::instrument;

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Server error with coder: {0}")]
    ServerError(StatusCode),
}

impl FromServerContext<State<AppState>> for AppState {
    type Rejection = Error;

    async fn from_request(req: &DioxusServerContext) -> Result<Self, Self::Rejection> {
        let state: State<AppState> = req
            .extract()
            .await
            .map_err(|_| Error::ServerError(StatusCode::INTERNAL_SERVER_ERROR))?;
        Ok(state)
    }
}

#[async_trait]
impl FromRequestParts<()> for AppState {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &()) -> Result<Self, Self::Rejection> {
        let state = parts
            .extract_with_state::<AppState, _>(_state)
            .await
            .map_err(|_| Error::ServerError(StatusCode::INTERNAL_SERVER_ERROR))?;
        Ok(state)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let Error::ServerError(status_code) = self;
        (status_code, self.to_string()).into_response()
    }
}

#[instrument]
pub(crate) async fn launch_server() {
    use dioxus::prelude::*;
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{Database, DatabaseConnection};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    // Load configuration from environment variables
    let config = config::ServerConfig::load().expect("Failed to load server configuration");

    let db: DatabaseConnection = Database::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    let app_state = AppState { db };

    // Get the address the server should run on. If the CLI is running, the CLI proxies fullstack into the main address,
    // and we use the generated address the CLI gives us
    let ip =
        dioxus::cli_config::server_ip().unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let port = dioxus::cli_config::server_port().unwrap_or(8080);
    let address = SocketAddr::new(ip, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let router = axum::Router::new()
        // serve_dioxus_application adds routes to server side render the application, serve static assets, and register server functions
        .serve_dioxus_application(ServeConfig::new().unwrap(), App)
        .with_state(app_state)
        .into_make_service();
    axum::serve(listener, router).await.unwrap();
}
