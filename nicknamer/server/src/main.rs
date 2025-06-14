use sea_orm_migration::prelude::*;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = config::Config::new();
    let db = sea_orm::Database::connect(&cfg.db_url).await?;
    let schema_manager = sea_orm_migration::SchemaManager::new(&db);
    migration::Migrator::refresh(&db).await?;
    Ok(())
}

mod config {
    use dotenvy::dotenv_iter;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Config {
        pub db_url: String,
    }

    impl Config {
        pub fn new() -> Self {
            let iter = dotenv_iter()
                .expect("Failed to load .env file")
                .map(|res| res.expect("Failed to read environment variable"));
            envy::from_iter(iter).expect("Failed to parse environment variables into Config")
        }
    }
}
