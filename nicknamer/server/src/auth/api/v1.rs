/// JSON request payload for API login
#[derive(serde::Deserialize, Debug)]
pub struct JsonLoginRequest {
    pub username: String,
    pub password: String,
}

/// JSON response for successful API login
#[derive(serde::Serialize, Debug)]
pub struct LoginResponse {
    pub token: String,
}

/// JSON response for API errors
#[derive(serde::Serialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

use crate::auth::{AuthState, CurrentUser, decode_jwt, encode_jwt};
use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Creates a JSON API router for authentication endpoints.
pub fn create_api_router(state: Arc<AuthState>) -> Router<()> {
    Router::new()
        .route("/api/v1/login", axum::routing::post(json_login_handler))
        .with_state(state)
}

/// API authentication middleware that extracts the current user from Authorization Bearer header.
/// Sets the CurrentUser extension if a valid JWT token is found in the Authorization header.
pub async fn auth_user_middleware(
    State(state): State<Arc<AuthState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = decode_jwt(token, &state.jwt_secret).await {
                    let current_user = CurrentUser::new(claims.username);
                    request.extensions_mut().insert(current_user);
                }
            }
        }
    }

    next.run(request).await
}

/// Middleware that ensures the current user is authenticated.
/// Returns UNAUTHORIZED if the CurrentUser extension is not found in the request.
/// This middleware should be applied after auth_user_middleware.
pub async fn require_auth_middleware(request: Request, next: Next) -> Response {
    // Check if user is authenticated by looking for CurrentUser extension
    let is_authenticated = request.extensions().get::<CurrentUser>().is_some();

    if !is_authenticated {
        let error_response = ErrorResponse {
            error: "UNAUTHORIZED".to_string(),
            message: "Authentication required to access this resource".to_string(),
        };
        return (StatusCode::UNAUTHORIZED, Json(error_response)).into_response();
    }

    next.run(request).await
}

/// Handles JSON login requests and returns a JWT token.
/// Validates credentials and returns either a success response with token or an error.
#[tracing::instrument(skip(state, payload))]
pub async fn json_login_handler(
    State(state): State<Arc<AuthState>>,
    Json(payload): Json<JsonLoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    if payload.username == state.admin_username && payload.password == state.admin_password {
        // Generate JWT token
        let jwt_token = encode_jwt(payload.username.clone(), &state.jwt_secret)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "JWT_ERROR".to_string(),
                        message: "Failed to generate authentication token".to_string(),
                    }),
                )
            })?;

        let response = LoginResponse { token: jwt_token };

        Ok(Json(response))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "INVALID_CREDENTIALS".to_string(),
                message: "Invalid username or password".to_string(),
            }),
        ))
    }
}
