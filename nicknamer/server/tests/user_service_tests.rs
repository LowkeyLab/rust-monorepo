use migration::MigratorTrait;
use nicknamer_server::user::UserService; // Use this if UserService is defined in the user module
use sea_orm::{Database, DatabaseConnection};
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::{postgres, testcontainers};

struct TestContext {
    #[allow(dead_code)] // container is kept to ensure it's not dropped
    container: testcontainers::ContainerAsync<postgres::Postgres>,
    db: DatabaseConnection,
}

async fn setup_container() -> anyhow::Result<testcontainers::ContainerAsync<postgres::Postgres>> {
    let container = postgres::Postgres::default().start().await?;
    Ok(container)
}

async fn setup_db(
    container: &testcontainers::ContainerAsync<postgres::Postgres>,
) -> anyhow::Result<DatabaseConnection> {
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(5432).await?;
    let db_url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);
    let db = Database::connect(&db_url).await?;
    migration::Migrator::up(&db, None).await?;
    Ok(db)
}

async fn setup() -> anyhow::Result<TestContext> {
    // Allow multiple calls to init for tests.
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init();
    let container = setup_container().await?;
    let db = setup_db(&container).await?;
    Ok(TestContext { db, container })
}

#[tokio::test]
async fn test_create_user() -> anyhow::Result<()> {
    let state = setup().await?;
    let user_service = UserService::new(&state.db);
    let user = user_service
        .create_user(123456789, "TestUser".to_string())
        .await?;
    assert_eq!(user.get_discord_id(), 123456789);
    assert_eq!(user.get_name(), "TestUser");
    Ok(())
}
