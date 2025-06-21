use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use insta::assert_yaml_snapshot;
use nicknamer_server::name::{Name, NameService, NameState, create_name_router};
use sea_orm::DatabaseConnection;
use serde_json::{Value, json};
use std::sync::Arc;
use testcontainers_modules::{postgres, testcontainers};
use tower::ServiceExt;

mod common;

/// Helper to create snapshot data from HTML response and metadata.
fn create_html_snapshot_data(
    body_text: &str,
    status: StatusCode,
    content_type: Option<&str>,
    test_context: &str,
) -> Value {
    json!({
        "test_context": test_context,
        "status": status.as_u16(),
        "content_type": content_type,
        "html_body": normalize_html_for_snapshot(body_text)
    })
}

/// Normalize HTML content for consistent snapshots by removing dynamic values.
fn normalize_html_for_snapshot(html: &str) -> String {
    // For now, return HTML as-is since we'll use deterministic test data
    // In the future, we could add more sophisticated normalization
    html.to_string()
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
    let _test_names = create_test_names(&state.db).await;

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

    let snapshot_data = create_html_snapshot_data(
        body_text,
        StatusCode::OK,
        Some("text/html; charset=utf-8"),
        "names_table_with_existing_names",
    );

    assert_yaml_snapshot!(snapshot_data);
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

    let snapshot_data = create_html_snapshot_data(
        body_text,
        StatusCode::OK,
        Some("text/html; charset=utf-8"),
        "empty_names_table",
    );

    assert_yaml_snapshot!(snapshot_data);
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data = create_html_snapshot_data(
        body_text,
        StatusCode::OK,
        Some("text/html; charset=utf-8"),
        "names_endpoint_content_type_check",
    );

    assert_yaml_snapshot!(snapshot_data);
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

    let snapshot_data = create_html_snapshot_data(
        body_text,
        StatusCode::OK,
        Some("text/html; charset=utf-8"),
        "create_name_successfully",
    );

    assert_yaml_snapshot!(snapshot_data);
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

    let snapshot_data = create_html_snapshot_data(
        body_text,
        StatusCode::OK,
        Some("text/html; charset=utf-8"),
        "create_multiple_names_update_count",
    );

    assert_yaml_snapshot!(snapshot_data);
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

    let snapshot_data = create_html_snapshot_data(
        body_text,
        StatusCode::OK,
        Some("text/html; charset=utf-8"),
        "form_with_special_characters",
    );

    assert_yaml_snapshot!(snapshot_data);
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

    let snapshot_data = create_html_snapshot_data(
        body_text,
        StatusCode::OK,
        Some("text/html; charset=utf-8"),
        "add_name_form",
    );

    assert_yaml_snapshot!(snapshot_data);
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

    let snapshot_data = create_html_snapshot_data(
        body_text,
        StatusCode::OK,
        Some("text/html; charset=utf-8"),
        "table_fragment_not_full_page",
    );

    assert_yaml_snapshot!(snapshot_data);
}
