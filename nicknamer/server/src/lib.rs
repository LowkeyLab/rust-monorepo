pub mod config {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Config {
        pub db_url: String,
        #[serde(default = "default_port")]
        pub port: u16,
        pub admin_username: String,
        pub admin_password: String,
    }

    impl Config {
        /// Loads configuration from environment variables.
        pub fn from_env() -> anyhow::Result<Self> {
            let settings = config::Config::builder()
                .add_source(config::Environment::default())
                .build()?;

            let config: Config = settings.try_deserialize()?;
            Ok(config)
        }
    }

    fn default_port() -> u16 {
        8080
    }
}
pub mod entities;
pub mod name {
    use crate::entities::*;
    use sea_orm::*;
    #[derive(Debug, PartialEq, Clone, Eq, Hash)]
    pub struct Name {
        id: u32,
        discord_id: u64,
        name: String,
    }
    impl Name {
        pub fn new(id: u32, discord_id: u64, name: String) -> Self {
            Self {
                id,
                discord_id,
                name,
            }
        }
        /// Returns the ID of the name.
        pub fn get_id(&self) -> u32 {
            self.id
        }
    }
    pub struct NameService<'a> {
        db: &'a sea_orm::DatabaseConnection,
    }

    impl NameService<'_> {
        pub fn new(db: &sea_orm::DatabaseConnection) -> NameService {
            NameService { db }
        }

        /// Creates a new name entry in the database.
        /// # Arguments
        ///
        /// * `discord_id` - The Discord ID of the user.
        /// * `name` - The name of the user.
        ///
        /// # Returns
        ///
        /// A `Result` containing the created `Name` if successful, or an error otherwise.
        #[tracing::instrument(skip(self))]
        pub async fn create_name(&self, discord_id: u64, name: String) -> anyhow::Result<Name> {
            let active_model = name::ActiveModel {
                discord_id: ActiveValue::Set(discord_id as i64),
                name: ActiveValue::Set(name.clone()),
                ..Default::default()
            };
            let created_model = active_model.insert(self.db).await?;
            Ok(Name::new(
                created_model.id as u32,
                created_model.discord_id as u64,
                created_model.name,
            ))
        }

        /// Edits a name entry by their ID.
        ///
        /// # Arguments
        ///
        /// * `id` - The ID of the name entry to edit.
        /// * `new_name` - The new name for the entry.
        ///
        /// # Returns
        ///
        /// A `Result` containing the updated `Name` if successful, or an error otherwise.
        #[tracing::instrument(skip(self))]
        pub async fn edit_name_by_id(&self, id: u32, new_name: String) -> anyhow::Result<Name> {
            let name_to_update = name::Entity::find_by_id(id as i32)
                .one(self.db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Name entry with ID {} not found", id))?;

            let mut active_model: name::ActiveModel = name_to_update.into();
            active_model.name = ActiveValue::Set(new_name.clone());
            let updated_model = active_model.update(self.db).await?;

            Ok(Name::new(
                updated_model.id as u32,
                updated_model.discord_id as u64,
                updated_model.name,
            ))
        }

        /// Retrieves all name entries from the database.
        ///
        /// # Returns
        ///
        /// A `Result` containing a vector of `Name` if successful, or an error otherwise.
        #[tracing::instrument(skip(self))]
        pub async fn get_all_names(&self) -> anyhow::Result<Vec<Name>> {
            let names = name::Entity::find()
                .all(self.db)
                .await?
                .into_iter()
                .map(|model| Name::new(model.id as u32, model.discord_id as u64, model.name))
                .collect();
            Ok(names)
        }
    }
}

pub mod web {
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
    use crate::web::middleware::CorsExposeLayer;

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
            let user_facing_error_message = "An unexpected error occurred while processing your request. Please try again later.";
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
            .layer(CorsExposeLayer::new());
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

    mod middleware {
        use axum::http::{HeaderName, HeaderValue, Request, Response};
        use pin_project_lite::pin_project;
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};
        use tower::{Layer, Service};
        /// Layer that adds CORS headers to expose HTMX headers
        #[derive(Clone, Default)]
        pub struct CorsExposeLayer;

        impl CorsExposeLayer {
            /// Creates a new CorsExposeLayer
            pub fn new() -> Self {
                Self
            }
        }

        impl<S> Layer<S> for CorsExposeLayer {
            type Service = CorsExposeService<S>;

            fn layer(&self, inner: S) -> Self::Service {
                CorsExposeService { inner }
            }
        }

        /// Service that adds Access-Control-Expose-Headers to responses
        #[derive(Clone)]
        pub struct CorsExposeService<S> {
            inner: S,
        }

        impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CorsExposeService<S>
        where
            S: Service<Request<ReqBody>, Response = Response<ResBody>>,
        {
            type Response = Response<ResBody>;
            type Error = S::Error;
            type Future = CorsExposeFuture<S::Future>;

            fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                self.inner.poll_ready(cx)
            }

            fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
                CorsExposeFuture {
                    future: self.inner.call(request),
                }
            }
        }

        pin_project! {
            /// Future that resolves to a response with CORS headers added
            pub struct CorsExposeFuture<F> {
                #[pin]
                future: F,
            }
        }

        impl<F, ResBody, E> Future for CorsExposeFuture<F>
        where
            F: Future<Output = Result<Response<ResBody>, E>>,
        {
            type Output = Result<Response<ResBody>, E>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let this = self.project();
                match this.future.poll(cx) {
                    Poll::Ready(Ok(mut response)) => {
                        // Add the Access-Control-Expose-Headers header
                        response.headers_mut().insert(
                            HeaderName::from_static("access-control-expose-headers"),
                            HeaderValue::from_static("hx-retarget,hx-reswap"),
                        );
                        Poll::Ready(Ok(response))
                    }
                    Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                    Poll::Pending => Poll::Pending,
                }
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use axum::body::Body;
            use axum::http::{Request, StatusCode};
            use axum::{Router, response::Response};
            use tower::ServiceExt;

            #[tokio::test]
            async fn can_add_cors_expose_header() {
                let app = Router::new()
                    .route("/test", axum::routing::get(|| async { "test response" }))
                    .layer(CorsExposeLayer::new());

                let response = app
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri("/test")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::OK);

                let headers = response.headers();
                let expose_headers = headers.get("access-control-expose-headers");
                assert_eq!(
                    expose_headers,
                    Some(&axum::http::HeaderValue::from_static(
                        "hx-retarget,hx-reswap"
                    ))
                );
            }

            #[tokio::test]
            async fn can_preserve_existing_headers_when_adding_cors() {
                async fn handler_with_custom_header() -> Response<String> {
                    let mut response = Response::new("test response".to_string());
                    response.headers_mut().insert(
                        "custom-header",
                        axum::http::HeaderValue::from_static("custom-value"),
                    );
                    response
                }

                let app = Router::new()
                    .route(
                        "/test-with-headers",
                        axum::routing::get(handler_with_custom_header),
                    )
                    .layer(CorsExposeLayer::new());

                let response = app
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri("/test-with-headers")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::OK);

                let headers = response.headers();

                // Check that our CORS header was added
                let expose_headers = headers.get("access-control-expose-headers");
                assert_eq!(
                    expose_headers,
                    Some(&axum::http::HeaderValue::from_static(
                        "hx-retarget,hx-reswap"
                    ))
                );

                // Check that existing headers are preserved
                let custom_header = headers.get("custom-header");
                assert_eq!(
                    custom_header,
                    Some(&axum::http::HeaderValue::from_static("custom-value"))
                );
            }
        }
    }
}
