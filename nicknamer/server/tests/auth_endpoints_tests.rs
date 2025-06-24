use axum::body::Body;
use axum::extract::Extension;
use axum::http::Request;
use axum::middleware::{from_fn, from_fn_with_state};
use insta::assert_yaml_snapshot;
use nicknamer_server::auth::{
    AuthError, AuthState, CurrentUser, create_login_router, encode_jwt, login_page_handler,
};
use nicknamer_server::config::Config;
use std::sync::Arc;
use tower::ServiceExt;

mod common;

use common::{HttpResponseSnapshot, create_stub_user_middleware};

/// Setup function for auth endpoint tests.
async fn setup_auth_state() -> Arc<AuthState> {
    let config = Config {
        db_url: "".to_string(),
        port: 8080,
        admin_username: "admin".to_string(),
        admin_password: "password".to_string(),
        jwt_secret: "some_secret".to_string(),
    };
    Arc::new(AuthState::from_config(&config))
}

/// Test helper to create test app with auth state.
async fn create_test_app() -> (axum::Router, Arc<AuthState>) {
    let auth_state = setup_auth_state().await;
    let app = create_login_router(auth_state.clone()).layer(from_fn_with_state(
        auth_state.clone(),
        nicknamer_server::auth::auth_user_middleware,
    ));
    (app, auth_state)
}

/// Test helper to create test app with a logged-in user.
async fn create_test_app_with_logged_in_user(username: String) -> (axum::Router, Arc<AuthState>) {
    let auth_state = setup_auth_state().await;
    let app = create_login_router(auth_state.clone())
        .layer(from_fn(create_stub_user_middleware(username)));
    (app, auth_state)
}

#[tokio::test]
async fn can_login_with_valid_credentials() {
    let (app, _auth_state) = create_test_app().await;

    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=admin&password=password"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "login_with_valid_credentials");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_reject_invalid_credentials() {
    let (app, _auth_state) = create_test_app().await;

    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=wrong&password=wrong"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "reject_invalid_credentials");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_handle_template_error_with_internal_server_error() {
    // Simulate a template rendering error using askama::Error::Custom
    let custom_error_message = "Simulated template rendering failure".to_string();
    let template_error = askama::Error::Custom(custom_error_message.into());

    let auth_error = AuthError::Template(template_error);
    let response = axum::response::IntoResponse::into_response(auth_error);

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "handle_template_error");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_return_success_when_already_logged_in() {
    let (app, auth_state) = create_test_app().await;

    // First, create a valid JWT token
    let jwt_token = encode_jwt("admin".to_string(), &auth_state.jwt_secret)
        .await
        .unwrap();

    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .header("cookie", format!("auth_token={}", jwt_token))
        .body(Body::from("username=admin&password=password"))
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
        "return_success_when_already_logged_in",
    );

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_display_login_page() {
    let (app, _auth_state) = create_test_app().await;

    let request = Request::builder()
        .method("GET")
        .uri("/login")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = std::str::from_utf8(&body).unwrap();

    let snapshot_data =
        HttpResponseSnapshot::new(body_text, status, &headers, "display_login_page");

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_display_login_page_with_homepage_button_when_logged_in() {
    let (app, _auth_state) = create_test_app_with_logged_in_user("admin".to_string()).await;

    let request = Request::builder()
        .method("GET")
        .uri("/login")
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
        "display_login_page_with_homepage_button_when_logged_in",
    );

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_render_login_page_form_when_user_not_logged_in() {
    let result = login_page_handler(None).await;

    assert!(result.is_ok());
    let html = result.unwrap().0;

    let snapshot_data = HttpResponseSnapshot::new(
        &html,
        axum::http::StatusCode::OK,
        &axum::http::HeaderMap::new(),
        "render_login_page_form_when_user_not_logged_in",
    );

    assert_yaml_snapshot!(snapshot_data);
}

#[tokio::test]
async fn can_render_login_page_with_homepage_button_when_user_logged_in() {
    let current_user = CurrentUser::new("testuser".to_string());
    let extension = Extension(current_user);

    let result = login_page_handler(Some(extension)).await;

    assert!(result.is_ok());
    let html = result.unwrap().0;

    let snapshot_data = HttpResponseSnapshot::new(
        &html,
        axum::http::StatusCode::OK,
        &axum::http::HeaderMap::new(),
        "render_login_page_with_homepage_button_when_user_logged_in",
    );

    assert_yaml_snapshot!(snapshot_data);
}
