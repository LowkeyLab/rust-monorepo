use askama::Template;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::Html;
use migration::MigratorTrait;
use sea_orm::Database;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::auth::{
    AuthState, CurrentUser, auth_user_middleware, create_login_router, login_redirect_middleware,
};
use crate::config::{self, Config};
use crate::name::{NameState, create_name_router};

pub mod middleware;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<sea_orm::DatabaseConnection>,
}

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

#[tracing::instrument(skip(config))]
pub async fn start_web_server(config: config::Config) -> anyhow::Result<()> {
    use axum::Router;

    let server_address = format!("0.0.0.0:{}", &config.port);
    let listener = tokio::net::TcpListener::bind(&server_address).await?;
    tracing::info!("Web server running on http://{}", server_address);

    let db = Database::connect(&config.db_url).await?;
    migration::Migrator::up(&db, None).await?;
    tracing::info!("Database migrations applied successfully");

    // Create AuthState from config
    let auth_state = Arc::new(AuthState::from_config(&config));

    // Create the login router with AuthState
    let login_router = create_login_router(auth_state.clone());

    let name_state = NameState { db: Arc::new(db) };

    // Create name router with database connection
    let name_router = create_name_router(name_state);

    let protected_routes = Router::new().merge(name_router).layer(
        ServiceBuilder::new()
            .layer(from_fn_with_state(auth_state.clone(), auth_user_middleware))
            .layer(from_fn(login_redirect_middleware)),
    );

    let public_routes = Router::new()
        .route("/health", axum::routing::get(health_check_handler))
        .route("/", axum::routing::get(welcome_handler))
        .route(
            "/call-to-action",
            axum::routing::get(call_to_action_handler),
        )
        .merge(login_router)
        .layer(
            ServiceBuilder::new()
                .layer(from_fn_with_state(auth_state.clone(), auth_user_middleware)),
        );
    // Create main router and merge with login router and name router
    let app = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(from_fn(middleware::cors_expose_headers)),
        );

    axum::serve(listener, app).await?;
    Ok(())
}

#[tracing::instrument]
async fn health_check_handler() -> &'static str {
    "OK"
}

#[tracing::instrument]
async fn welcome_handler() -> Result<Html<String>, WebError> {
    let template = IndexTemplate::new();
    template.render().map(Html).map_err(WebError::from)
}

#[tracing::instrument]
async fn call_to_action_handler(
    current_user: Option<Extension<CurrentUser>>,
) -> Result<Html<String>, WebError> {
    let template = match current_user {
        Some(Extension(user)) => CallToActionTemplate::new(Some(user.username.clone())),
        None => CallToActionTemplate::new(None),
    };
    template.render().map(Html).map_err(WebError::from)
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

impl IndexTemplate {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Template)]
#[template(path = "partials/call_to_action.html")]
struct CallToActionTemplate {
    username: Option<String>,
}

impl CallToActionTemplate {
    pub fn new(username: Option<String>) -> Self {
        Self { username }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::CurrentUser;
    use crate::testing::HttpResponseSnapshot;
    use axum::http::StatusCode;
    use insta::assert_yaml_snapshot;

    #[tokio::test]
    async fn can_check_health() {
        let response = health_check_handler().await;
        assert_eq!(response, "OK");
    }

    #[tokio::test]
    async fn can_render_welcome_page() {
        let result = welcome_handler().await;
        assert!(
            result.is_ok(),
            "welcome() returned an error: {:?}",
            result.err()
        );

        let html = result.unwrap();
        let snapshot = HttpResponseSnapshot::from_html(&html.0, "welcome_page");
        assert_yaml_snapshot!(snapshot);
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
        let body_text = std::str::from_utf8(&body).unwrap();
        
        let snapshot = HttpResponseSnapshot::from_html(body_text, "template_error_response");
        assert_yaml_snapshot!(snapshot);
    }

    #[tokio::test]
    async fn can_render_call_to_action_for_authenticated_user() {
        let user = CurrentUser {
            username: "testuser".to_string(),
        };
        let result = call_to_action_handler(Some(Extension(user))).await;

        assert!(
            result.is_ok(),
            "call_to_action_handler returned an error: {:?}",
            result.err()
        );

        let html = result.unwrap();
        let snapshot = HttpResponseSnapshot::from_html(&html.0, "call_to_action_authenticated");
        assert_yaml_snapshot!(snapshot);
    }

    #[tokio::test]
    async fn can_render_call_to_action_for_unauthenticated_user() {
        let result = call_to_action_handler(None).await;

        assert!(
            result.is_ok(),
            "call_to_action_handler returned an error: {:?}",
            result.err()
        );

        let html = result.unwrap();
        let snapshot = HttpResponseSnapshot::from_html(&html.0, "call_to_action_unauthenticated");
        assert_yaml_snapshot!(snapshot);
    }

    #[tokio::test]
    async fn can_render_call_to_action_with_correct_content_type() {
        let result = call_to_action_handler(None).await;
        assert!(
            result.is_ok(),
            "call_to_action_handler returned an error: {:?}",
            result.err()
        );

        let response: axum::response::Response =
            axum::response::IntoResponse::into_response(result.unwrap());
        assert_eq!(response.status(), StatusCode::OK);

        let content_type = response.headers().get(axum::http::header::CONTENT_TYPE);
        assert_eq!(
            content_type,
            Some(&axum::http::HeaderValue::from_static(
                "text/html; charset=utf-8"
            ))
        );
    }
}
