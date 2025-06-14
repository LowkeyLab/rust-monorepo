use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = nicknamer_server::config::Config::new();
    let db = sea_orm::Database::connect(&cfg.db_url).await?;
    migration::Migrator::up(&db, None).await?;
    Ok(())
}
