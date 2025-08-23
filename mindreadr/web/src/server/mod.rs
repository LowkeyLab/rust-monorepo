#[cfg(feature = "server")]
mod config;
#[cfg(feature = "server")]
pub mod entities;

use crate::App;
use dioxus::prelude::*;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::OnceCell;
use tracing::instrument;

static DB_POOL: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn get_db_pool() -> &'static DatabaseConnection {
    DB_POOL
        .get_or_init(|| async {
            let config = config::ServerConfig::load().expect("Failed to load server configuration");
            Database::connect(&config.database_url)
                .await
                .expect("Failed to connect to database")
        })
        .await
}

#[instrument]
pub async fn launch_server() {
    let db = get_db_pool().await;

    Migrator::up(db, None)
        .await
        .expect("Failed to run migrations");

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
        .into_make_service();
    axum::serve(listener, router).await.unwrap();
}

mod auth {}
