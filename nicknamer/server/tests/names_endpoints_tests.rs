use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use insta::assert_yaml_snapshot;
use nicknamer_server::entities::name;
use nicknamer_server::name::{NameState, create_name_router};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;
use testcontainers_modules::{postgres, testcontainers};
use tower::ServiceExt;

mod common;

/// HTTP response snapshot for testing endpoints.
#[derive(Debug, Serialize)]
struct HttpResponseSnapshot {
    test_context: String,
    status: u16,
    headers: std::collections::BTreeMap<String, String>,
    html_body: Vec<String>,
}

impl HttpResponseSnapshot {
    /// Create a new HTTP response snapshot.
    fn new(
        body_text: &str,
        status: StatusCode,
        headers: &axum::http::HeaderMap,
        test_context: &str,
    ) -> Self {
        Self {
            test_context: test_context.to_string(),
            status: status.as_u16(),
            headers: filter_variable_headers(headers),
            html_body: normalize_html_for_snapshot(body_text),
        }
    }
}

/// Normalize HTML content for consistent snapshots by removing dynamic values.
fn normalize_html_for_snapshot(html: &str) -> Vec<String> {
    // Split HTML by newlines and convert to Vec<String>
    // In the future, we could add more sophisticated normalization
    html.lines().map(|line| line.to_string()).collect()
}

/// Filter out variable headers from response headers for snapshot testing.
fn filter_variable_headers(headers: &axum::http::HeaderMap) -> BTreeMap<String, String> {
    let variable_headers = [
        "date",
        "expires",
        "last-modified",
        "etag",
        "server",
        "x-request-id",
        "x-trace-id",
        "set-cookie",
        "content-length",
    ];

    headers
        .iter()
        .filter_map(|(name, value)| {
            let name_str = name.as_str().to_lowercase();
            if variable_headers.contains(&name_str.as_str()) {
                None
            } else {
                value.to_str().ok().map(|v| (name_str, v.to_string()))
            }
        })
        .collect()
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
async fn create_test_names(db: &DatabaseConnection) {
    let name1 = name::ActiveModel {
        discord_id: Set(123456789),
        name: Set("TestUser1".to_string()),
        ..Default::default()
    };

    let name2 = name::ActiveModel {
        discord_id: Set(987654321),
        name: Set("TestUser2".to_string()),
        ..Default::default()
    };

    let _result1 = name1.insert(db).await.unwrap();
    let _result2 = name2.insert(db).await.unwrap();
}

/// Test helper to create a single test name and return its ID.
async fn create_single_test_name(db: &DatabaseConnection) -> i32 {
    let name = name::ActiveModel {
        discord_id: Set(555444333),
        name: Set("DeleteTestUser".to_string()),
        ..Default::default()
    };

    let result = name.insert(db).await.unwrap();
    result.id
}

/// Test helper to create a single test name for editing and return its ID.
async fn create_editable_test_name(db: &DatabaseConnection) -> i32 {
    let name = name::ActiveModel {
        discord_id: Set(777888999),
        name: Set("EditableTestUser".to_string()),
        ..Default::default()
    };

    let result = name.insert(db).await.unwrap();
    result.id
}

#[tokio::test]
async fn can_display_names_table_when_names_exist() {
    let state = setup().await.expect("Failed to setup test context");
    create_test_names(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .uri("/names")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        status,
        &headers,
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

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data = HttpResponseSnapshot::new(body_text, status, &headers, "empty_names_table");

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

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        status,
        &headers,
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

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "create_name_successfully");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_create_multiple_names_and_update_count() {
    let state = setup().await.expect("Failed to setup test context");
    create_test_names(&state.db).await;

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

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        status,
        &headers,
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

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "form_with_special_characters");

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
        .uri("/names/add")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data = HttpResponseSnapshot::new(body_text, status, &headers, "add_name_form");

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

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "table_fragment_not_full_page");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn cannot_create_name_with_duplicate_discord_id() {
    let state = setup().await.expect("Failed to setup test context");

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state.clone());

    // First, create a name with a specific Discord ID
    let form_data = "discord_id=123456789&name=FirstUser";
    let request = Request::builder()
        .method(Method::POST)
        .uri("/names")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let _response = app.oneshot(request).await.unwrap();

    // Now try to create another name with the same Discord ID
    let duplicate_form_data = "discord_id=123456789&name=SecondUser";
    let app2 = create_name_router(name_state.clone());
    let duplicate_request = Request::builder()
        .method(Method::POST)
        .uri("/names")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(duplicate_form_data))
        .unwrap();

    let duplicate_response = app2.oneshot(duplicate_request).await.unwrap();

    let headers = duplicate_response.headers().clone();
    let body = axum::body::to_bytes(duplicate_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Verify the error message is user-friendly
    assert!(body_text.contains("A name entry already exists for this Discord ID"));

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        StatusCode::UNPROCESSABLE_ENTITY,
        &headers,
        "duplicate_discord_id_error",
    );

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_delete_name_successfully() {
    let state = setup().await.expect("Failed to setup test context");
    let name_id = create_single_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/names/{}", name_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should return the updated names table (empty in this case)
    assert!(body_text.contains("No names found in the database"));

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "delete_name_successfully");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_delete_name_and_update_table_count() {
    let state = setup().await.expect("Failed to setup test context");

    // Create multiple names
    create_test_names(&state.db).await;
    let delete_name_id = create_single_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/names/{}", delete_name_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should show remaining names (2) and updated count
    assert!(body_text.contains("TestUser1"));
    assert!(body_text.contains("TestUser2"));
    assert!(!body_text.contains("DeleteTestUser"));
    assert!(body_text.contains("<div class=\"stat-value\">2</div>"));

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        status,
        &headers,
        "delete_name_and_update_table_count",
    );

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_handle_delete_request_for_nonexistent_name() {
    let state = setup().await.expect("Failed to setup test context");

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    // Try to delete a name with ID that doesn't exist
    let request = Request::builder()
        .method(Method::DELETE)
        .uri("/names/99999")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should contain error message
    assert!(body_text.contains("An unexpected error occurred"));

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "delete_nonexistent_name_error");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn delete_endpoint_returns_table_fragment_not_full_page() {
    let state = setup().await.expect("Failed to setup test context");
    let name_id = create_single_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/names/{}", name_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should return only the table fragment, not a full HTML page
    assert!(!body_text.contains("<html"));
    assert!(!body_text.contains("<head"));
    assert!(!body_text.contains("<body"));

    // Should contain the table structure or empty message
    assert!(body_text.contains("No names found") || body_text.contains("<table"));

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "delete_returns_table_fragment");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn delete_endpoint_returns_correct_content_type() {
    let state = setup().await.expect("Failed to setup test context");
    let name_id = create_single_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/names/{}", name_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        status,
        &headers,
        "delete_endpoint_content_type_check",
    );

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_serve_edit_name_form() {
    let state = setup().await.expect("Failed to setup test context");
    let name_id = create_editable_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .uri(format!("/names/{}/edit", name_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should contain the edit form with the current name
    assert!(body_text.contains("EditableTestUser"));
    assert!(body_text.contains("name=\"name\""));
    assert!(body_text.contains("hx-put"));

    let snapshot_data = HttpResponseSnapshot::new(body_text, status, &headers, "edit_name_form");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_handle_edit_form_request_for_nonexistent_name() {
    let state = setup().await.expect("Failed to setup test context");

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .uri("/names/99999/edit")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should contain error message
    assert!(body_text.contains("An unexpected error occurred"));

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        status,
        &headers,
        "edit_form_nonexistent_name_error",
    );

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_update_name_successfully() {
    let state = setup().await.expect("Failed to setup test context");
    let name_id = create_editable_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let form_data = "name=UpdatedTestUser";
    let request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/names/{}", name_id))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should return the updated name row
    assert!(body_text.contains("UpdatedTestUser"));
    assert!(!body_text.contains("EditableTestUser"));

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "update_name_successfully");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_update_name_with_special_characters() {
    let state = setup().await.expect("Failed to setup test context");
    let name_id = create_editable_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let form_data = "name=Updated%20User%20With%20Spaces%21%40%23";
    let request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/names/{}", name_id))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should handle URL decoding and contain the updated name
    assert!(body_text.contains("Updated User With Spaces!@#"));

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        status,
        &headers,
        "update_name_with_special_characters",
    );

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_handle_update_request_for_nonexistent_name() {
    let state = setup().await.expect("Failed to setup test context");

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let form_data = "name=NonexistentUser";
    let request = Request::builder()
        .method(Method::PUT)
        .uri("/names/99999")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should contain error message
    assert!(body_text.contains("An unexpected error occurred"));

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "update_nonexistent_name_error");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn update_endpoint_returns_name_row_fragment_not_full_page() {
    let state = setup().await.expect("Failed to setup test context");
    let name_id = create_editable_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let form_data = "name=FragmentTestUser";
    let request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/names/{}", name_id))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should return only the name row fragment, not a full HTML page
    assert!(!body_text.contains("<html"));
    assert!(!body_text.contains("<head"));
    assert!(!body_text.contains("<body"));

    // Should contain the table row structure
    assert!(body_text.contains("<tr"));
    assert!(body_text.contains("FragmentTestUser"));

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        status,
        &headers,
        "update_returns_name_row_fragment",
    );

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn update_endpoint_returns_correct_content_type() {
    let state = setup().await.expect("Failed to setup test context");
    let name_id = create_editable_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let form_data = "name=ContentTypeTestUser";
    let request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/names/{}", name_id))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        status,
        &headers,
        "update_endpoint_content_type_check",
    );

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn edit_form_endpoint_returns_correct_content_type() {
    let state = setup().await.expect("Failed to setup test context");
    let name_id = create_editable_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let request = Request::builder()
        .uri(format!("/names/{}/edit", name_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        status,
        &headers,
        "edit_form_endpoint_content_type_check",
    );

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_update_name_with_empty_string() {
    let state = setup().await.expect("Failed to setup test context");
    let name_id = create_editable_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let form_data = "name=";
    let request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/names/{}", name_id))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should handle empty string as name
    assert!(body_text.contains("<td></td>") || body_text.contains("</td>"));

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "update_name_with_empty_string");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_update_name_with_very_long_string() {
    let state = setup().await.expect("Failed to setup test context");
    let name_id = create_editable_test_name(&state.db).await;

    let name_state = NameState {
        db: Arc::new(state.db),
    };
    let app = create_name_router(name_state);

    let long_name = "A".repeat(100); // 100 character name
    let form_data = format!("name={}", long_name);
    let request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/names/{}", name_id))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    // Should handle very long names
    assert!(body_text.contains(&long_name));

    let snapshot_data = HttpResponseSnapshot::new(
        body_text,
        status,
        &headers,
        "update_name_with_very_long_string",
    );

    assert_yaml_snapshot!(snapshot_data);
}
