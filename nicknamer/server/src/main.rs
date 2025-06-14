use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    nicknamer_server::start_web_server().await
}
