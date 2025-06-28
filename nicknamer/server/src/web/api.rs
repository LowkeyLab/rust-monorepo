use std::sync::Arc;

use crate::{
    auth::{self, AuthState},
    name::NameState,
};

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
};

use tower::ServiceBuilder;
/// Creates the API routes for JSON API endpoints.
pub fn create_api_router(auth_state: Arc<AuthState>, name_state: Arc<NameState>) -> axum::Router {
    let login_router = auth::api::v1::create_api_router(auth_state.clone());
    let names_router = crate::name::api::v1::create_api_router(name_state.clone());
    let protected_routes = names_router
        .layer(ServiceBuilder::new().layer(from_fn(auth::api::v1::require_auth_middleware)));
    let public_routes = login_router;
    let api_routes = public_routes.merge(protected_routes);
    Router::new()
        .nest("/api/v1", api_routes)
        .layer(ServiceBuilder::new().layer(from_fn_with_state(
            auth_state,
            auth::api::v1::auth_user_middleware,
        )))
}
