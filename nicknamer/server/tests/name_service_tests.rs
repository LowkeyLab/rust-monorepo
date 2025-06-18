use nicknamer_server::name::NameService;
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
async fn can_register_name() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);
    let discord_id = 123456789;
    let name = "TestUser".to_string();
    let created_name = name_service
        .create_name(discord_id, name.clone())
        .await
        .expect("Failed to create name");

    let expected_name = nicknamer_server::name::Name::new(
        created_name.get_id(), // The ID is generated, so we use the created name's ID
        discord_id,
        name,
    );
    assert_eq!(created_name, expected_name);
}

#[tokio::test]
async fn can_update_name() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);

    // Create a name entry to edit
    let initial_discord_id = 987654321;
    let initial_name = "InitialName".to_string();
    let initial_name_entry = name_service
        .create_name(initial_discord_id, initial_name.clone())
        .await
        .expect("Failed to create name");

    let expected_initial_name = nicknamer_server::name::Name::new(
        initial_name_entry.get_id(),
        initial_discord_id,
        initial_name,
    );
    assert_eq!(initial_name_entry, expected_initial_name);

    // Edit the name
    let new_name = "UpdatedName".to_string();
    let updated_name = name_service
        .edit_name_by_id(initial_name_entry.get_id(), new_name.clone())
        .await
        .expect("Failed to update name");

    let expected_updated_name = nicknamer_server::name::Name::new(
        initial_name_entry.get_id(), // ID remains the same
        initial_discord_id,          // Discord ID remains the same
        new_name,
    );
    assert_eq!(updated_name, expected_updated_name);
}

#[tokio::test]
async fn can_handle_update_when_name_not_found() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);

    // Create a name entry to ensure there's some data, but we'll use a different ID for the failing test
    let initial_name = name_service
        .create_name(111222333, "SomeUser".to_string())
        .await
        .expect("Failed to create name");

    // Verify that an error is returned if the name ID does not exist
    let non_existent_id = initial_name.get_id() + 1; // Assuming this ID won't exist
    let result = name_service
        .edit_name_by_id(non_existent_id, "AnotherName".to_string())
        .await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            format!("Name entry with ID {} not found", non_existent_id)
        );
    }
}

#[tokio::test]
async fn can_get_all_names() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);

    // Create a couple of name entries
    let name1_discord_id = 1;
    let name1_name = "UserOne".to_string();
    let created_name1 = name_service
        .create_name(name1_discord_id, name1_name.clone())
        .await
        .expect("Failed to create name1");

    let name2_discord_id = 2;
    let name2_name = "UserTwo".to_string();
    let created_name2 = name_service
        .create_name(name2_discord_id, name2_name.clone())
        .await
        .expect("Failed to create name2");

    let names = name_service
        .get_all_names()
        .await
        .expect("Failed to get all names");

    assert_eq!(names.len(), 2);

    let expected_name1 =
        nicknamer_server::name::Name::new(created_name1.get_id(), name1_discord_id, name1_name);
    let expected_name2 =
        nicknamer_server::name::Name::new(created_name2.get_id(), name2_discord_id, name2_name);

    assert!(names.contains(&expected_name1));
    assert!(names.contains(&expected_name2));
}

#[tokio::test]
async fn can_handle_empty_names_list() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);

    let names = name_service
        .get_all_names()
        .await
        .expect("Failed to get all names");

    assert!(names.is_empty());
}
