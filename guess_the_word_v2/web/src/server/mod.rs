#[cfg(feature = "server")]
mod config;
#[cfg(feature = "server")]
mod entities;

use crate::App;
use axum::extract::FromRef;
use tracing::instrument;

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
}

#[cfg(feature = "server")]
impl FromRef<()> for AppState {
    fn from_ref(_: &()) -> Self {
        panic!("AppState cannot be created from ()");
    }
}

#[cfg(feature = "server")]
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

    let app_state = AppState { db: db.clone() };

    // Get the address the server should run on. If the CLI is running, the CLI proxies fullstack into the main address
    // and we use the generated address the CLI gives us
    let ip =
        dioxus::cli_config::server_ip().unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let port = dioxus::cli_config::server_port().unwrap_or(8080);
    let address = SocketAddr::new(ip, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let router = axum::Router::new()
        // serve_dioxus_application adds routes to server side render the application, serve static assets, and register server functions
        .serve_dioxus_application(ServeConfig::new().unwrap(), App)
        .into_make_service();
    axum::serve(listener, router).await.unwrap();
}
