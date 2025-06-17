use nicknamer_server::user::UserService;
use sea_orm::DatabaseConnection; // Added for TestContext
use testcontainers_modules::{postgres, testcontainers}; // Added for TestContext

mod common;

// 1. Define TestContext struct locally
pub struct TestContext {
    #[allow(dead_code)] // container is kept to ensure it's not dropped
    pub container: testcontainers::ContainerAsync<postgres::Postgres>,
    pub db: DatabaseConnection,
}

// 2. Define setup() function locally, using public functions from common module
async fn setup() -> anyhow::Result<TestContext> {
    // Allow multiple calls to init for tests.
    let _ = tracing_subscriber::fmt().try_init();
    let container = common::setup_container().await?;
    let db = common::setup_db(&container).await?;
    Ok(TestContext { db, container })
}

#[tokio::test]
async fn test_user_registration() {
    let state = setup().await.expect("Failed to setup test context");
    let user_service = UserService::new(&state.db);
    let discord_id = 123456789;
    let name = "TestUser".to_string();
    let created_user = user_service
        .create_user(discord_id, name.clone())
        .await
        .expect("Failed to create user");

    let expected_user = nicknamer_server::user::User::new(
        created_user.get_id(), // The ID is generated, so we use the created user's ID
        discord_id,
        name,
    );
    assert_eq!(created_user, expected_user);
}

#[tokio::test]
async fn can_update_user_name() {
    let state = setup().await.expect("Failed to setup test context");
    let user_service = UserService::new(&state.db);

    // Create a user to edit
    let initial_discord_id = 987654321;
    let initial_name = "InitialName".to_string();
    let initial_user = user_service
        .create_user(initial_discord_id, initial_name.clone())
        .await
        .expect("Failed to create user");

    let expected_initial_user =
        nicknamer_server::user::User::new(initial_user.get_id(), initial_discord_id, initial_name);
    assert_eq!(initial_user, expected_initial_user);

    // Edit the user's name
    let new_name = "UpdatedName".to_string();
    let updated_user = user_service
        .edit_user_name_by_id(initial_user.get_id(), new_name.clone())
        .await
        .expect("Failed to update user name");

    let expected_updated_user = nicknamer_server::user::User::new(
        initial_user.get_id(), // ID remains the same
        initial_discord_id,    // Discord ID remains the same
        new_name,
    );
    assert_eq!(updated_user, expected_updated_user);
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
    let user1_discord_id = 1;
    let user1_name = "UserOne".to_string();
    let created_user1 = user_service
        .create_user(user1_discord_id, user1_name.clone())
        .await
        .expect("Failed to create user1");

    let user2_discord_id = 2;
    let user2_name = "UserTwo".to_string();
    let created_user2 = user_service
        .create_user(user2_discord_id, user2_name.clone())
        .await
        .expect("Failed to create user2");

    let users = user_service
        .get_all_users()
        .await
        .expect("Failed to get all users");

    assert_eq!(users.len(), 2);

    let expected_user1 =
        nicknamer_server::user::User::new(created_user1.get_id(), user1_discord_id, user1_name);
    let expected_user2 =
        nicknamer_server::user::User::new(created_user2.get_id(), user2_discord_id, user2_name);

    assert!(users.contains(&expected_user1));
    assert!(users.contains(&expected_user2));
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
