#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let config = nicknamer_server::config::Config::from_env();
    nicknamer_server::web::start_web_server(config).await
}
