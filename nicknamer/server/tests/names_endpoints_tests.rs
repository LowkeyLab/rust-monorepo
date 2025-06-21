use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use nicknamer_server::name::{Name, NameService, NameState, create_name_router};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use testcontainers_modules::{postgres, testcontainers};
use tower::ServiceExt;

mod common;

/// Helper to validate names page contains all expected names and structure.
fn assert_names_page_contains_names(body_text: &str, expected_names: &[Name]) {
    // Verify page structure
    assert!(body_text.contains("Names Database"), "Missing page title");
    assert!(body_text.contains("Stored Names"), "Missing section header");

    if expected_names.is_empty() {
        assert!(
            body_text.contains("No names found in the database"),
            "Missing empty state message"
        );
    } else {
        // Verify all names are present
        for name in expected_names {
            assert!(
                body_text.contains(name.name()),
                "Missing name: {}",
                name.name()
            );
            assert!(
                body_text.contains(&name.discord_id().to_string()),
                "Missing discord ID: {}",
                name.discord_id()
            );
            assert!(
                body_text.contains(&name.id().to_string()),
                "Missing name ID: {}",
                name.id()
            );
        }

        // Verify count is correct
        assert!(
            body_text.contains("Total Names"),
            "Missing total names label"
        );
        assert!(
            body_text.contains(&expected_names.len().to_string()),
            "Incorrect name count, expected: {}",
            expected_names.len()
        );
    }
}

/// Helper to validate table fragment contains expected names without full page structure.
fn assert_table_fragment_contains_names(body_text: &str, expected_names: &[Name]) {
    // Should contain table identifier
    assert!(
        body_text.contains(r#"id="names-table""#),
        "Missing table ID"
    );

    if expected_names.is_empty() {
        assert!(
            body_text.contains("No names found in the database"),
            "Missing empty state in table fragment"
        );
    } else {
        // Verify all names are present
        for name in expected_names {
            assert!(
                body_text.contains(name.name()),
                "Missing name in table fragment: {}",
                name.name()
            );
            assert!(
                body_text.contains(&name.discord_id().to_string()),
                "Missing discord ID in table fragment: {}",
                name.discord_id()
            );
        }

        // Verify count is present
        assert!(
            body_text.contains(&expected_names.len().to_string()),
            "Missing correct count in table fragment: {}",
            expected_names.len()
        );
    }

    // Should NOT contain full page elements
    assert!(
        !body_text.contains("navbar"),
        "Table fragment should not contain navbar"
    );
    assert!(
        !body_text.contains("Names Database"),
        "Table fragment should not contain page title"
    );
    assert!(
        !body_text.contains("← Back"),
        "Table fragment should not contain navigation"
    );
}

/// Helper to validate add name form contains expected form elements.
fn assert_add_name_form_is_valid(body_text: &str) {
    let expected_elements = [
        "Add New Name",
        "Discord ID",
        "Name",
        "<form",
        r#"hx-post="/names""#,
        "input",
        r#"type="submit""#,
    ];

    for element in expected_elements {
        assert!(
            body_text.contains(element),
            "Add name form missing expected element: {}",
            element
        );
    }
}

/// Test context for endpoint tests.
pub struct TestContext {
    #[allow(dead_code)] // container is kept to ensure it's not dropped
    pub container: testcontainers::ContainerAsync<postgres::Postgres>,
    pub db: DatabaseConnection,
}

/// Setup function for endpoint tests using PostgreSQL container.
async fn setup() -> anyhow::Result<TestContext> {
    // Allow multiple calls to init for tests.
    let _ = tracing_subscriber::fmt().try_init();
    let container = common::setup_container().await?;
    let db = common::setup_db(&container).await?;
    Ok(TestContext { db, container })
}

/// Test helper to create test names in the database.
async fn create_test_names(db: &DatabaseConnection) -> Vec<Name> {
    let name_service = NameService::new(db);

    let name1 = name_service
        .create_name(123456789, "TestUser1".to_string())
        .await
        .unwrap();

    let name2 = name_service
        .create_name(987654321, "TestUser2".to_string())
        .await
        .unwrap();

    vec![name1, name2]
}

#[tokio::test]
async fn can_display_names_table_when_names_exist() {
    let state = setup().await.expect("Failed to setup test context");
    let test_names = create_test_names(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .uri("/names")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    assert_names_page_contains_names(body_text, &test_names);
}

#[tokio::test]
async fn can_display_empty_names_table_when_no_names_exist() {
    let state = setup().await.expect("Failed to setup test context");

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .uri("/names")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    assert_names_page_contains_names(body_text, &[]);
}

#[tokio::test]
async fn names_endpoint_returns_correct_content_type() {
    let state = setup().await.expect("Failed to setup test context");

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .uri("/names")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/html; charset=utf-8"
    );
}

#[tokio::test]
async fn can_create_name_successfully() {
    let state = setup().await.expect("Failed to setup test context");

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state.clone());

    let form_data = "discord_id=555666777&name=NewTestUser";
    let request = Request::builder()
        .method(Method::POST)
        .uri("/names")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Verify the name was actually created in the database
    let name_service = NameService::new(&name_state.db);
    let names = name_service.get_all_names().await.unwrap();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name(), "NewTestUser");
    assert_eq!(names[0].discord_id(), 555666777);

    // Verify the response contains the table fragment with the new name
    assert_table_fragment_contains_names(body_text, &names);
}

#[tokio::test]
async fn can_create_multiple_names_and_update_count() {
    let state = setup().await.expect("Failed to setup test context");
    let _existing_names = create_test_names(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state.clone());

    let form_data = "discord_id=111222333&name=ThirdUser";
    let request = Request::builder()
        .method(Method::POST)
        .uri("/names")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Verify all names exist in the database
    let name_service = NameService::new(&name_state.db);
    let names = name_service.get_all_names().await.unwrap();
    assert_eq!(names.len(), 3);

    // Verify the response contains the table fragment with all names
    assert_table_fragment_contains_names(body_text, &names);
}

#[tokio::test]
async fn can_handle_form_with_special_characters_in_name() {
    let state = setup().await.expect("Failed to setup test context");

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state.clone());

    let form_data = "discord_id=888999000&name=User%20With%20Spaces%21";
    let request = Request::builder()
        .method(Method::POST)
        .uri("/names")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Verify the name was stored correctly in the database
    let name_service = NameService::new(&name_state.db);
    let names = name_service.get_all_names().await.unwrap();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name(), "User With Spaces!");
    assert_eq!(names[0].discord_id(), 888999000);

    // Verify the response contains the table fragment with the special characters name
    assert_table_fragment_contains_names(body_text, &names);
}

#[tokio::test]
async fn can_serve_add_name_form() {
    let state = setup().await.expect("Failed to setup test context");

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .uri("/names/form")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/html; charset=utf-8"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    assert_add_name_form_is_valid(body_text);
}

#[tokio::test]
async fn post_endpoint_returns_table_fragment_not_full_page() {
    let state = setup().await.expect("Failed to setup test context");

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state.clone());

    let form_data = "discord_id=777888999&name=FragmentTestUser";
    let request = Request::builder()
        .method(Method::POST)
        .uri("/names")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Verify the name was created in the database
    let name_service = NameService::new(&name_state.db);
    let names = name_service.get_all_names().await.unwrap();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name(), "FragmentTestUser");
    assert_eq!(names[0].discord_id(), 777888999);

    // Verify the response is a table fragment, not a full page
    assert_table_fragment_contains_names(body_text, &names);
}
