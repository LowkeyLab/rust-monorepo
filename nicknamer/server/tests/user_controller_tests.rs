mod common;

use crate::common::setup;
use nicknamer_server::user::{User, UserController, UserService};

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
