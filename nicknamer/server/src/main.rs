use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("nicknamer_server=info".parse().unwrap())
                .add_directive("tower_http=debug".parse().unwrap()),
        )
        .init();
    let config = nicknamer_server::config::Config::from_env()?;
    nicknamer_server::web::start_web_server(config).await
}
