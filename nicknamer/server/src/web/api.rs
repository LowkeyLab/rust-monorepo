pub(crate) mod v1 {
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
    use utoipa::OpenApi;
    use utoipa_swagger_ui::SwaggerUi;

    /// OpenAPI documentation for the v1 API
    #[derive(OpenApi)]
    #[openapi(
        paths(
            crate::auth::api::v1::json_login_handler,
            crate::name::api::v1::get_names_handler,
        ),
        components(
            schemas(
                crate::auth::api::v1::JsonLoginRequest,
                crate::auth::api::v1::LoginResponse,
                crate::auth::api::v1::ErrorResponse,
                crate::name::api::v1::NameJson,
                crate::name::api::v1::NamesResponse,
                crate::name::api::v1::ErrorResponse,
            )
        ),
        tags(
            (name = "Authentication", description = "Authentication endpoints"),
            (name = "Names", description = "Name management endpoints")
        ),
        info(
            title = "Nicknamer API",
            version = "1.0.0",
            description = "API for managing Discord nicknames and authentication"
        )
    )]
    pub struct ApiDoc;

    /// Creates the API routes for JSON API endpoints.
    pub fn create_api_router(
        auth_state: Arc<AuthState>,
        name_state: Arc<NameState>,
    ) -> axum::Router {
        let login_router = auth::api::v1::create_api_router(auth_state.clone());
        let names_router = crate::name::api::v1::create_api_router(name_state.clone());
        let protected_routes = names_router
            .layer(ServiceBuilder::new().layer(from_fn(auth::api::v1::require_auth_middleware)));
        let public_routes = login_router;
        let api_routes = public_routes.merge(protected_routes);

        Router::new()
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .nest("/api/v1", api_routes)
            .layer(ServiceBuilder::new().layer(from_fn_with_state(
                auth_state,
                auth::api::v1::auth_user_middleware,
            )))
    }
}
