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
    let _ = tracing_subscriber::fmt().try_init();
    let container = setup_container().await?;
    let db = setup_db(&container).await?;
    Ok(TestContext { db, container })
}

#[tokio::test]
async fn test_user_registration() {
    let state = setup().await.expect("Failed to setup test context");
    let user_service = UserService::new(&state.db);
    let user = user_service
        .create_user(123456789, "TestUser".to_string())
        .await
        .expect("Failed to create user");
    assert_eq!(user.get_discord_id(), 123456789);
    assert_eq!(user.get_name(), "TestUser");
}

#[tokio::test]
async fn test_update_user_name_successfully() {
    let state = setup().await.expect("Failed to setup test context");
    let user_service = UserService::new(&state.db);

    // Create a user to edit
    let initial_user = user_service
        .create_user(987654321, "InitialName".to_string())
        .await
        .expect("Failed to create user");
    assert_eq!(initial_user.get_discord_id(), 987654321);
    assert_eq!(initial_user.get_name(), "InitialName");

    // Edit the user's name
    let new_name = "UpdatedName".to_string();
    let updated_user = user_service
        .edit_user_name_by_id(initial_user.get_id(), new_name.clone())
        .await
        .expect("Failed to update user name");

    assert_eq!(updated_user.get_discord_id(), initial_user.get_discord_id());
    assert_eq!(updated_user.get_name(), new_name);
    assert_eq!(updated_user.get_id(), initial_user.get_id());
}

#[tokio::test]
async fn test_update_user_name_fails_if_user_not_found() {
    let state = setup().await.expect("Failed to setup test context");
    let user_service = UserService::new(&state.db);

    // Create a user to ensure there's some data, but we'll use a different ID for the failing test
    let initial_user = user_service
        .create_user(111222333, "SomeUser".to_string())
        .await
        .expect("Failed to create user");

    // Verify that an error is returned if the user ID does not exist
    let non_existent_id = initial_user.get_id() + 1; // Assuming this ID won't exist
    let result = user_service
        .edit_user_name_by_id(non_existent_id, "AnotherName".to_string())
        .await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            format!("User with ID {} not found", non_existent_id)
        );
    }
}

#[tokio::test]
async fn can_get_all_users() {
    let state = setup().await.expect("Failed to setup test context");
    let user_service = UserService::new(&state.db);

    // Create a couple of users
    let user1 = user_service
        .create_user(1, "UserOne".to_string())
        .await
        .expect("Failed to create user1");
    let user2 = user_service
        .create_user(2, "UserTwo".to_string())
        .await
        .expect("Failed to create user2");

    let users = user_service
        .get_all_users()
        .await
        .expect("Failed to get all users");

    assert_eq!(users.len(), 2);
    assert!(
        users
            .iter()
            .any(|u| u.get_id() == user1.get_id() && u.get_name() == user1.get_name())
    );
    assert!(
        users
            .iter()
            .any(|u| u.get_id() == user2.get_id() && u.get_name() == user2.get_name())
    );
}

#[tokio::test]
async fn get_all_users_returns_empty_vec_when_no_users() {
    let state = setup().await.expect("Failed to setup test context");
    let user_service = UserService::new(&state.db);

    let users = user_service
        .get_all_users()
        .await
        .expect("Failed to get all users");

    assert!(users.is_empty());
}
