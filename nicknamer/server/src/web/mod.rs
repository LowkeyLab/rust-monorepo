use askama::Template;
use axum::extract::Extension;
use axum::http::{HeaderName, StatusCode};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::Html;
use migration::MigratorTrait;
use sea_orm::Database;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::auth::{
    AuthState, CurrentUser, auth_user_middleware, create_login_router, login_redirect_middleware,
};
use crate::config::{self, Config};
use crate::name::{NameState, create_name_router};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<sea_orm::DatabaseConnection>,
}

/// Custom error type for web handler operations.
#[derive(Debug, thiserror::Error)]
pub enum WebError {
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
                .layer(CorsLayer::new().expose_headers([
                    HeaderName::from_static("hx-retarget"),
                    HeaderName::from_static("hx-reswap"),
                ])),
        );

    axum::serve(listener, app).await?;
    Ok(())
}

#[tracing::instrument]
pub async fn health_check_handler() -> &'static str {
    "OK"
}

#[tracing::instrument]
pub async fn welcome_handler() -> Result<Html<String>, WebError> {
    let template = IndexTemplate::new();
    template.render().map(Html).map_err(WebError::from)
}

#[tracing::instrument]
pub async fn call_to_action_handler(
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
#[template(path = "welcome/call_to_action.html")]
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
    use axum::http::StatusCode;

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

        assert_eq!(
            body_text,
            "<h1>Internal Server Error</h1><p>An unexpected error occurred while processing your request. Please try again later.</p>"
        );
    }
}
