use nicknamer_server::entities::name;
use nicknamer_server::name::NameService;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use testcontainers_modules::{postgres, testcontainers};

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

    // Create a name entry directly using the entity ActiveModel
    let initial_discord_id = 987654321;
    let initial_name = "InitialName".to_string();
    let active_model = name::ActiveModel {
        discord_id: ActiveValue::Set(initial_discord_id),
        name: ActiveValue::Set(initial_name.clone()),
        ..Default::default()
    };
    let initial_name_entry = active_model
        .insert(&state.db)
        .await
        .expect("Failed to create name");

    let expected_initial_name = nicknamer_server::name::Name::new(
        initial_name_entry.id as u32,
        initial_discord_id as u64,
        initial_name,
    );
    let service_initial_name = nicknamer_server::name::Name::new(
        initial_name_entry.id as u32,
        initial_name_entry.discord_id as u64,
        initial_name_entry.name.clone(),
    );
    assert_eq!(service_initial_name, expected_initial_name);

    // Edit the name
    let new_name = "UpdatedName".to_string();
    let updated_name = name_service
        .edit_name_by_id(initial_name_entry.id as u32, new_name.clone())
        .await
        .expect("Failed to update name");

    let expected_updated_name = nicknamer_server::name::Name::new(
        initial_name_entry.id as u32, // ID remains the same
        initial_discord_id as u64,    // Discord ID remains the same
        new_name,
    );
    assert_eq!(updated_name, expected_updated_name);
}

#[tokio::test]
async fn can_handle_update_when_name_not_found() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);

    // Create a name entry directly using the entity ActiveModel to ensure there's some data
    let active_model = name::ActiveModel {
        discord_id: ActiveValue::Set(111222333),
        name: ActiveValue::Set("SomeUser".to_string()),
        ..Default::default()
    };
    let initial_name = active_model
        .insert(&state.db)
        .await
        .expect("Failed to create name");

    // Verify that an error is returned if the name ID does not exist
    let non_existent_id = initial_name.id + 1; // Assuming this ID won't exist
    let result = name_service
        .edit_name_by_id(non_existent_id as u32, "AnotherName".to_string())
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

    // Create a couple of name entries directly using the entity ActiveModel
    let name1_discord_id = 1;
    let name1_name = "UserOne".to_string();
    let active_model1 = name::ActiveModel {
        discord_id: ActiveValue::Set(name1_discord_id),
        name: ActiveValue::Set(name1_name.clone()),
        ..Default::default()
    };
    let created_name1 = active_model1
        .insert(&state.db)
        .await
        .expect("Failed to create name1");

    let name2_discord_id = 2;
    let name2_name = "UserTwo".to_string();
    let active_model2 = name::ActiveModel {
        discord_id: ActiveValue::Set(name2_discord_id),
        name: ActiveValue::Set(name2_name.clone()),
        ..Default::default()
    };
    let created_name2 = active_model2
        .insert(&state.db)
        .await
        .expect("Failed to create name2");

    let names = name_service
        .get_all_names()
        .await
        .expect("Failed to get all names");

    assert_eq!(names.len(), 2);

    let expected_name1 = nicknamer_server::name::Name::new(
        created_name1.id as u32,
        name1_discord_id as u64,
        name1_name,
    );
    let expected_name2 = nicknamer_server::name::Name::new(
        created_name2.id as u32,
        name2_discord_id as u64,
        name2_name,
    );

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
