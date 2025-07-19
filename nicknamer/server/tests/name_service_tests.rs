use nicknamer_server::entities::name;
use nicknamer_server::name::NameService;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
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
        .create_name(discord_id, name.clone(), "server123".to_string())
        .await
        .expect("Failed to create name");

    // Verify that the created name has the correct properties
    assert_eq!(created_name.discord_id(), discord_id);
    assert_eq!(created_name.name(), &name);
    assert!(created_name.id() > 0); // ID should be generated and positive
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
        server_id: ActiveValue::Set("server123".to_string()),
        ..Default::default()
    };
    let initial_name_entry = active_model
        .insert(&state.db)
        .await
        .expect("Failed to create name");

    let expected_initial_name = nicknamer_server::name::Name::from(initial_name_entry.clone());
    let service_initial_name = nicknamer_server::name::Name::from(initial_name_entry.clone());
    assert_eq!(service_initial_name, expected_initial_name);

    // Edit the name
    let new_name = "UpdatedName".to_string();
    let updated_name = name_service
        .edit_name_by_id(
            initial_name_entry.id as u32,
            new_name.clone(),
            "server456".to_string(),
        )
        .await
        .expect("Failed to update name");

    let expected_updated_name = {
        let mut expected_model = initial_name_entry.clone();
        expected_model.name = new_name.clone();
        expected_model.server_id = "server456".to_string();
        nicknamer_server::name::Name::from(expected_model)
    };
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
        server_id: ActiveValue::Set("server789".to_string()),
        ..Default::default()
    };
    let initial_name = active_model
        .insert(&state.db)
        .await
        .expect("Failed to create name");

    // Verify that an error is returned if the name ID does not exist
    let non_existent_id = initial_name.id + 1; // Assuming this ID won't exist
    let result = name_service
        .edit_name_by_id(
            non_existent_id as u32,
            "AnotherName".to_string(),
            "server999".to_string(),
        )
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

    let expected_name1 = nicknamer_server::name::Name::from(created_name1);
    let expected_name2 = nicknamer_server::name::Name::from(created_name2);

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

#[tokio::test]
async fn cannot_create_name_with_duplicate_discord_id_and_server_id() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);
    let discord_id = 123456789;
    let server_id = "server123";

    // First name creation should succeed
    let first_name = name_service
        .create_name(discord_id, "FirstUser".to_string(), server_id.to_string())
        .await
        .expect("Failed to create first name");

    assert_eq!(first_name.discord_id(), discord_id);
    assert_eq!(first_name.name(), "FirstUser");

    // Second name creation with same Discord ID and same Server ID should fail
    let second_creation_result = name_service
        .create_name(discord_id, "SecondUser".to_string(), server_id.to_string())
        .await;

    assert!(second_creation_result.is_err());
    if let Err(e) = second_creation_result {
        assert!(e.to_string().contains("already exists"));
    }
}

#[tokio::test]
async fn can_create_name_with_same_discord_id_but_different_server_id() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);
    let discord_id = 123456789;

    // First name creation should succeed
    let first_name = name_service
        .create_name(discord_id, "FirstUser".to_string(), "server123".to_string())
        .await
        .expect("Failed to create first name");

    assert_eq!(first_name.discord_id(), discord_id);
    assert_eq!(first_name.name(), "FirstUser");

    // Second name creation with same Discord ID but different Server ID should succeed
    let second_creation_result = name_service
        .create_name(
            discord_id,
            "SecondUser".to_string(),
            "server456".to_string(),
        )
        .await;

    assert!(second_creation_result.is_ok());
    let second_name = second_creation_result.unwrap();
    assert_eq!(second_name.discord_id(), discord_id);
    assert_eq!(second_name.name(), "SecondUser");
    assert_eq!(second_name.server_id(), "server456");
}

#[tokio::test]
async fn can_delete_name() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);

    // Create a name to delete using ActiveModel
    let discord_id: i64 = 123456789;
    let name = "TestUser".to_string();
    let active_model = name::ActiveModel {
        discord_id: ActiveValue::Set(discord_id),
        name: ActiveValue::Set(name.clone()),
        ..Default::default()
    };
    let created_name_model = active_model
        .insert(&state.db)
        .await
        .expect("Failed to create name");
    let created_name = nicknamer_server::name::Name::from(created_name_model);

    // Verify it was created
    let names_before = name::Entity::find()
        .all(&state.db)
        .await
        .expect("Failed to get all names from database");
    assert_eq!(names_before.len(), 1);

    // Delete the name
    let deleted_name = name_service
        .delete_name_by_id(created_name.id())
        .await
        .expect("Failed to delete name");

    // Verify the deleted name matches what was created
    assert_eq!(deleted_name.id(), created_name.id());
    assert_eq!(deleted_name.discord_id(), discord_id as u64);
    assert_eq!(deleted_name.name(), &name);

    // Verify it was deleted
    let names_after = name::Entity::find()
        .all(&state.db)
        .await
        .expect("Failed to get all names from database");
    assert!(names_after.is_empty());
}

#[tokio::test]
async fn can_handle_delete_when_name_not_found() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);

    // Try to delete a non-existent name
    let result = name_service.delete_name_by_id(999).await;

    // Should return an error
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("not found") || e.to_string().contains("NameNotFound"));
    }
}

#[tokio::test]
async fn can_get_name_by_id() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);

    // Create a name entry directly using the entity ActiveModel
    let discord_id = 555666777;
    let name = "GetTestUser".to_string();
    let active_model = name::ActiveModel {
        discord_id: ActiveValue::Set(discord_id),
        name: ActiveValue::Set(name.clone()),
        ..Default::default()
    };
    let created_name_model = active_model
        .insert(&state.db)
        .await
        .expect("Failed to create name");

    // Get the name by ID
    let retrieved_name = name_service
        .get_name_by_id(created_name_model.id as u32)
        .await
        .expect("Failed to get name by ID");

    // Construct expected result and compare
    let expected_name = nicknamer_server::name::Name::from(created_name_model);
    assert_eq!(retrieved_name, expected_name);
}

#[tokio::test]
async fn can_handle_get_name_by_nonexistent_id() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);

    // Create a name entry to ensure we have some data and know what ID won't exist
    let active_model = name::ActiveModel {
        discord_id: ActiveValue::Set(777888999),
        name: ActiveValue::Set("ExistingUser".to_string()),
        ..Default::default()
    };
    let created_name = active_model
        .insert(&state.db)
        .await
        .expect("Failed to create name");

    // Try to get a name with a non-existent ID
    let non_existent_id = created_name.id + 100; // Ensure this ID won't exist
    let result = name_service.get_name_by_id(non_existent_id as u32).await;

    // Should return an error
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            format!("Name entry with ID {} not found", non_existent_id)
        );
    }
}

#[tokio::test]
async fn can_get_multiple_names_by_different_ids() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);

    // Create multiple name entries
    let names_data = vec![
        (111222333, "FirstUser".to_string()),
        (444555666, "SecondUser".to_string()),
        (777888999, "ThirdUser".to_string()),
    ];

    let mut created_models = Vec::new();
    for (discord_id, name) in names_data {
        let active_model = name::ActiveModel {
            discord_id: ActiveValue::Set(discord_id),
            name: ActiveValue::Set(name),
            ..Default::default()
        };
        let created_model = active_model
            .insert(&state.db)
            .await
            .expect("Failed to create name");
        created_models.push(created_model);
    }

    // Retrieve each name by ID and verify
    for created_model in created_models {
        let retrieved_name = name_service
            .get_name_by_id(created_model.id as u32)
            .await
            .expect("Failed to get name by ID");

        let expected_name = nicknamer_server::name::Name::from(created_model);
        assert_eq!(retrieved_name, expected_name);
    }
}

#[tokio::test]
async fn can_handle_malformed_yaml_in_bulk_create() {
    let state = setup().await.expect("Failed to setup test context");
    let name_service = NameService::new(&state.db);

    // Test with invalid YAML content
    let invalid_yaml = "invalid yaml content: not properly formatted";
    let server_id = "test-server".to_string();

    let result = name_service
        .bulk_create_names(invalid_yaml, server_id)
        .await;

    // Should return MalformedData error
    assert!(result.is_err());
    let error = result.unwrap_err();
    match error {
        nicknamer_server::name::NameServiceError::MalformedData(msg) => {
            assert!(msg.contains("Invalid YAML format"));
        }
        _ => panic!("Expected MalformedData error, got: {:?}", error),
    }
}
