use askama::Template;
use axum::Router;
use axum::extract::{Form, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::response::{Html, IntoResponse, Response};
use jsonwebtoken::encode;
use std::sync::Arc;

use crate::config::Config;

/// Authentication state containing admin credentials and JWT secret.
#[derive(Clone)]
pub struct AuthState {
    pub admin_username: String,
    pub admin_password: String,
    pub jwt_secret: String,
}

impl AuthState {
    /// Creates a new AuthState from the application config.
    pub fn from_config(config: &Config) -> Self {
        Self {
            admin_username: config.admin_username.clone(),
            admin_password: config.admin_password.clone(),
            jwt_secret: config.jwt_secret.clone(),
        }
    }
}

/// Creates a login router with authentication routes.
pub fn create_login_router() -> Router<Arc<AuthState>> {
    Router::new().route("/login", axum::routing::post(login_handler))
}

/// Represents the login request payload.
#[derive(serde::Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Claims {
    pub exp: usize,       // Expiry time of the token
    pub iat: usize,       // Issued at time of the token
    pub username: String, // Username of the authenticated user
}
/// Custom error type for authentication operations.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// Represents an error during template rendering.
    /// The specific `askama::Error` is captured as the source of this error.
    #[error("Template rendering failed")]
    Template(#[from] askama::Error),
}

impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let user_facing_error_message =
            "An unexpected error occurred while processing your request. Please try again later.";
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                "<h1>Internal Server Error</h1><p>{}</p>",
                user_facing_error_message
            )),
        )
            .into_response()
    }
}

/// Handles the login request.
/// Checks submitted username and password against admin credentials.
pub async fn login_handler(
    State(state): State<Arc<AuthState>>,
    Form(payload): Form<LoginRequest>,
) -> Result<Response, AuthError> {
    if payload.username == state.admin_username && payload.password == state.admin_password {
        let html = LoginSuccessTemplate {
            name: &payload.username,
        }
        .render()
        .map_err(AuthError::from)?;

        Ok(Html(html).into_response())
    } else {
        let error_message = LoginErrorMessageTemplate
            .render()
            .map_err(AuthError::from)?;

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

async fn encode_jwt(username: String, jwt_secret: String) -> anyhow::Result<String> {
    let now = chrono::Utc::now();
    let expire = chrono::Duration::hours(24);
    let exp = (now + expire).timestamp() as usize;
    let iat = now.timestamp() as usize;
    let claims = Claims { exp, iat, username };
    let jwt = encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    Ok(jwt)
}

async fn decode_jwt(token: &str, jwt_secret: &str) -> anyhow::Result<Claims> {
    let token_data = jsonwebtoken::decode(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )?;
    Ok(token_data.claims)
}

#[derive(Template)]
#[template(path = "login_success.html")]
pub struct LoginSuccessTemplate<'a> {
    pub name: &'a str,
}

#[derive(Template)]
#[template(path = "login_error_message.html")]
pub struct LoginErrorMessageTemplate;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::config::Config;

    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for `oneshot`

    async fn test_app(config: Config) -> axum::Router {
        let auth_state = Arc::new(AuthState::from_config(&config));
        super::create_login_router().with_state(auth_state)
    }

    #[tokio::test]
    async fn can_login_with_valid_credentials() {
        let config = Config {
            db_url: "".to_string(),
            port: 8080,
            admin_username: "admin".to_string(),
            admin_password: "password".to_string(),
            jwt_secret: "some_secret".to_string(),
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
            jwt_secret: "some_secret".to_string(),
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
    async fn can_handle_template_error_with_internal_server_error() {
        // Simulate a template rendering error using askama::Error::Custom
        let custom_error_message = "Simulated template rendering failure".to_string();
        let template_error = askama::Error::Custom(custom_error_message.into());

        let auth_error = AuthError::Template(template_error);
        let response = axum::response::IntoResponse::into_response(auth_error);

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let expected_error_message = "<h1>Internal Server Error</h1><p>An unexpected error occurred while processing your request. Please try again later.</p>";
        assert_eq!(std::str::from_utf8(&body).unwrap(), expected_error_message);
    }
}
