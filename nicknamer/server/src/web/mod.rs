use askama::Template;
use axum::extract::{Form, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use migration::MigratorTrait;
use sea_orm::Database;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::config;
use crate::web::middleware::cors_expose_headers;

pub mod middleware;

/// Custom error type for web handler operations.
#[derive(Debug, thiserror::Error)]
enum WebError {
    /// Represents an error during template rendering.
    /// The specific `askama::Error` is captured as the source of this error.
    #[error("Template rendering failed")]
    Template(#[from] askama::Error),
}

impl axum::response::IntoResponse for WebError {
    fn into_response(self) -> axum::response::Response {
        let user_facing_error_message =
            "An unexpected error occurred while processing your request. Please try again later.";
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                "<h1>Internal Server Error</h1><p>{}</p>",
                user_facing_error_message
            )),
        )
            .into_response()
    }
}

#[derive(Clone)]
struct AppState {
    config: Arc<config::Config>,
}

/// Represents the login request payload.
#[derive(serde::Deserialize, Debug)]
struct LoginRequest {
    username: String,
    password: String,
}

#[tracing::instrument]
pub async fn start_web_server(config: config::Config) -> anyhow::Result<()> {
    use axum::Router;

    let server_address = format!("0.0.0.0:{}", &config.port);
    let listener = tokio::net::TcpListener::bind(&server_address).await?;
    tracing::info!("Web server running on http://{}", server_address);

    let db = Database::connect(&config.db_url).await?;
    migration::Migrator::up(&db, None).await?;
    tracing::info!("Database migrations applied successfully");
    let middleware = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().level(Level::INFO))
                .on_request(DefaultOnRequest::default())
                .on_response(DefaultOnResponse::default()),
        )
        .layer(axum::middleware::from_fn(cors_expose_headers));
    let app = Router::new()
        .layer(middleware)
        .route("/health", axum::routing::get(health_check_handler))
        .route("/", axum::routing::get(welcome_handler))
        .route("/login", axum::routing::post(login_handler))
        .with_state(AppState {
            config: Arc::new(config),
        });

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check_handler() -> &'static str {
    "OK"
}

/// Handles the login request.
/// Checks submitted username and password against admin credentials.
async fn login_handler(
    State(state): State<AppState>,
    Form(payload): Form<LoginRequest>,
) -> Result<Response, WebError> {
    if payload.username == state.config.admin_username
        && payload.password == state.config.admin_password
    {
        let html = LoginSuccessTemplate {
            name: &payload.username,
        }
        .render()
        .map_err(WebError::from)?;

        Ok(Html(html).into_response())
    } else {
        let error_message = LoginErrorMessageTemplate.render().map_err(WebError::from)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("hx-retarget"),
            HeaderValue::from_static("#login-message"),
        );
        headers.insert(
            HeaderName::from_static("hx-reswap"),
            HeaderValue::from_static("outerHTML"),
        );

        let mut response = Html(error_message).into_response();
        response.headers_mut().extend(headers);
        Ok(response)
    }
}

async fn welcome_handler() -> Result<Html<String>, WebError> {
    IndexTemplate.render().map(Html).map_err(WebError::from)
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "login_success.html")]
struct LoginSuccessTemplate<'a> {
    name: &'a str,
}

#[derive(Template)]
#[template(path = "login_error_message.html")]
struct LoginErrorMessageTemplate;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for `oneshot`

    async fn test_app(config: Config) -> axum::Router {
        let state = AppState {
            config: Arc::new(config),
        };
        Router::new()
            .route("/login", axum::routing::post(login_handler))
            .with_state(state)
    }

    #[tokio::test]
    async fn can_check_health() {
        let response = health_check_handler().await;
        assert_eq!(response, "OK");
    }

    #[tokio::test]
    async fn can_login_with_valid_credentials() {
        let config = Config {
            db_url: "".to_string(),
            port: 8080,
            admin_username: "admin".to_string(),
            admin_password: "password".to_string(),
        };
        let app = test_app(config).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/login")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("username=admin&password=password"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(
            body,
            LoginSuccessTemplate { name: "admin" }.render().unwrap()
        );
    }

    #[tokio::test]
    async fn can_reject_invalid_credentials() {
        let config = Config {
            db_url: "".to_string(),
            port: 8080,
            admin_username: "admin".to_string(),
            admin_password: "password".to_string(),
        };
        let app = test_app(config).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/login")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("username=wrong&password=wrong"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Check HX-Retarget header
        let hx_retarget = response.headers().get("hx-retarget");
        assert_eq!(
            hx_retarget,
            Some(&axum::http::HeaderValue::from_static("#login-message"))
        );

        // Check HX-Reswap header
        let hx_reswap = response.headers().get("hx-reswap");
        assert_eq!(
            hx_reswap,
            Some(&axum::http::HeaderValue::from_static("outerHTML"))
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let rendered_error = LoginErrorMessageTemplate.render().unwrap();
        assert_eq!(body, rendered_error);
        // Verify the error message is included in the response
        assert!(rendered_error.contains("Login failed. Please try again."));
    }

    #[tokio::test]
    async fn can_render_welcome_page_with_correct_content_type() {
        let result = welcome_handler().await;
        assert!(
            result.is_ok(),
            "welcome() returned an error: {:?}",
            result.err()
        );
        let response: axum::response::Response =
            axum::response::IntoResponse::into_response(result.unwrap());
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Welcome handler should return OK for this test"
        );
        let content_type = response.headers().get(axum::http::header::CONTENT_TYPE);
        assert_eq!(
            content_type,
            Some(&axum::http::HeaderValue::from_static(
                "text/html; charset=utf-8"
            ))
        );
    }

    #[tokio::test]
    async fn can_handle_template_error_with_internal_server_error() {
        // Simulate a template rendering error using askama::Error::Custom
        let custom_error_message = "Simulated template rendering failure".to_string();
        let template_error = askama::Error::Custom(custom_error_message.into());

        let web_error = WebError::Template(template_error);
        let response = axum::response::IntoResponse::into_response(web_error);

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let expected_error_message = "<h1>Internal Server Error</h1><p>An unexpected error occurred while processing your request. Please try again later.</p>";
        assert_eq!(std::str::from_utf8(&body).unwrap(), expected_error_message);
    }
}
