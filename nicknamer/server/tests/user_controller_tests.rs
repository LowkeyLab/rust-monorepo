mod common;

use nicknamer_server::user::{User, UserController, UserService};
use sea_orm::DatabaseConnection;
use testcontainers_modules::{postgres, testcontainers}; // Add this line

// 1. Define TestContext struct locally
pub struct TestContext {
    #[allow(dead_code)] // container is kept to ensure it's not dropped
    pub container: testcontainers::ContainerAsync<postgres::Postgres>,
    pub db: DatabaseConnection,
}

// 2. Define setup() function locally, using public functions from common module
async fn setup() -> anyhow::Result<TestContext> {
    // Allow multiple calls to init for tests.
    let _ = tracing_subscriber::fmt().try_init(); // Consider if this is still needed or should be handled differently
    let container = common::setup_container().await?;
    let db = common::setup_db(&container).await?;
    Ok(TestContext { db, container })
}

#[tokio::test]
async fn can_create_user() {
    let setup = setup().await.expect("Failed to set up test environment");
    let user_service = UserService::new(&setup.db);
    let user_controller = UserController::new(user_service);

    let discord_id = 123456789012345678;
    let name = "Test User".to_string();

    let created_user_result = user_controller.create_user(discord_id, name.clone()).await;

    assert!(
        created_user_result.is_ok(),
        "User creation should succeed. Error: {:?}",
        created_user_result.err()
    );
    let created_user = created_user_result.unwrap();

    let expected_user = User::new(created_user.get_id(), discord_id, name);

    assert_eq!(created_user, expected_user);
}
