use askama::Template;
use axum::Router;
use axum::extract::{Extension, Form, Request, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::CookieJar;
use jsonwebtoken::encode;
use std::sync::Arc;

use crate::config::Config;

/// Represents the currently authenticated user.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub username: String,
}

impl CurrentUser {
    /// Creates a new CurrentUser instance.
    pub fn new(username: String) -> Self {
        Self { username }
    }
}

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

/// Creates a router with authentication middleware.
pub fn create_auth_router() -> Router<Arc<AuthState>> {
    Router::new().layer(middleware::from_fn_with_state(
        Arc::new(AuthState {
            admin_username: String::new(),
            admin_password: String::new(),
            jwt_secret: String::new(),
        }),
        auth_middleware,
    ))
}

/// Authentication middleware that checks for valid JWT tokens and sets CurrentUser extension.
pub async fn auth_middleware(
    State(state): State<Arc<AuthState>>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(token_cookie) = jar.get("auth_token") {
        if let Ok(claims) = decode_jwt(token_cookie.value(), &state.jwt_secret).await {
            let current_user = CurrentUser::new(claims.username);
            request.extensions_mut().insert(current_user);
        }
    }

    next.run(request).await
}

/// Represents the login request payload.
#[derive(serde::Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Claims {
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
    /// Represents an error during JWT operations.
    /// The specific `jsonwebtoken::errors::Error` is captured as the source of this error.
    #[error("JWT operation failed")]
    JwtError,
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
/// If a user is already logged in, returns a success message.
pub async fn login_handler(
    State(state): State<Arc<AuthState>>,
    jar: CookieJar,
    current_user: Option<Extension<CurrentUser>>,
    Form(payload): Form<LoginRequest>,
) -> Result<(CookieJar, Response), AuthError> {
    // Check if user is already logged in
    if let Some(Extension(user)) = current_user {
        let html = LoginSuccessTemplate {
            name: &user.username,
        }
        .render()
        .map_err(AuthError::from)?;

        return Ok((jar, Html(html).into_response()));
    }
    if payload.username == state.admin_username && payload.password == state.admin_password {
        // Generate JWT token
        let jwt_token = encode_jwt(payload.username.clone(), &state.jwt_secret)
            .await
            .map_err(|_| AuthError::JwtError)?;

        // Create cookie with JWT token
        let cookie = axum_extra::extract::cookie::Cookie::build(("auth_token", jwt_token))
            .http_only(true)
            .secure(false) // Set to true in production with HTTPS
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .max_age(time::Duration::hours(24))
            .path("/")
            .build();

        let updated_jar = jar.add(cookie);

        let html = LoginSuccessTemplate {
            name: &payload.username,
        }
        .render()
        .map_err(AuthError::from)?;

        Ok((updated_jar, Html(html).into_response()))
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
        Ok((jar, response))
    }
}

pub async fn encode_jwt(username: String, jwt_secret: &str) -> anyhow::Result<String> {
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

#[allow(dead_code)]
pub async fn decode_jwt(token: &str, jwt_secret: &str) -> anyhow::Result<Claims> {
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
        super::create_login_router()
            .layer(middleware::from_fn_with_state(
                auth_state.clone(),
                super::auth_middleware,
            ))
            .with_state(auth_state)
    }

    async fn test_app_with_auth(config: Config) -> axum::Router {
        let auth_state = Arc::new(AuthState::from_config(&config));
        super::create_login_router()
            .layer(middleware::from_fn_with_state(
                auth_state.clone(),
                super::auth_middleware,
            ))
            .with_state(auth_state)
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

        // Check that the auth_token cookie was set
        let set_cookie_headers: Vec<_> = response.headers().get_all("set-cookie").iter().collect();
        assert!(
            !set_cookie_headers.is_empty(),
            "Expected Set-Cookie header to be present"
        );

        let cookie_header = set_cookie_headers[0].to_str().unwrap();
        assert!(
            cookie_header.contains("auth_token="),
            "Expected auth_token cookie to be set"
        );
        assert!(
            cookie_header.contains("HttpOnly"),
            "Expected HttpOnly flag to be set"
        );
        assert!(
            cookie_header.contains("Path=/"),
            "Expected Path to be set to /"
        );

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

    #[tokio::test]
    async fn can_return_success_when_already_logged_in() {
        let config = Config {
            db_url: "".to_string(),
            port: 8080,
            admin_username: "admin".to_string(),
            admin_password: "password".to_string(),
            jwt_secret: "some_secret".to_string(),
        };

        // First, create a valid JWT token
        let jwt_token = super::encode_jwt("admin".to_string(), &config.jwt_secret)
            .await
            .unwrap();

        let app = test_app_with_auth(config).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/login")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("cookie", format!("auth_token={}", jwt_token))
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
}
