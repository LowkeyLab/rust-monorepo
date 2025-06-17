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
        pub fn from_env() -> Self {
            envy::from_env().expect("Failed to load configuration from environment variables")
        }
    }
    fn default_port() -> u16 {
        8080
    }
}
pub mod entities;
pub mod user {
    use crate::entities::*;
    use sea_orm::*;
    #[derive(Debug, PartialEq, Clone, Eq, Hash)]
    pub struct User {
        id: u32,
        discord_id: u64,
        name: String,
    }
    impl User {
        pub fn new(id: u32, discord_id: u64, name: String) -> Self {
            Self {
                id,
                discord_id,
                name,
            }
        }
        /// Returns the ID of the user.
        pub fn get_id(&self) -> u32 {
            self.id
        }
    }
    pub struct UserService<'a> {
        db: &'a sea_orm::DatabaseConnection,
    }

    impl UserService<'_> {
        pub fn new(db: &sea_orm::DatabaseConnection) -> UserService {
            UserService { db }
        }

        /// Creates a new user in the database.
        /// # Arguments
        ///
        /// * `discord_id` - The Discord ID of the user.
        /// * `name` - The name of the user.
        ///
        /// # Returns
        ///
        /// A `Result` containing the created `User` if successful, or an error otherwise.
        #[tracing::instrument(skip(self))]
        pub async fn create_user(&self, discord_id: u64, name: String) -> anyhow::Result<User> {
            let active_model = user::ActiveModel {
                discord_id: ActiveValue::Set(discord_id as i64),
                name: ActiveValue::Set(name.clone()),
                ..Default::default()
            };
            let created_model = active_model.insert(self.db).await?;
            Ok(User::new(
                created_model.id as u32,
                created_model.discord_id as u64,
                created_model.name,
            ))
        }

        /// Edits a user's name by their ID.
        ///
        /// # Arguments
        ///
        /// * `id` - The ID of the user to edit.
        /// * `new_name` - The new name for the user.
        ///
        /// # Returns
        ///
        /// A `Result` containing the updated `User` if successful, or an error otherwise.
        #[tracing::instrument(skip(self))]
        pub async fn edit_user_name_by_id(
            &self,
            id: u32,
            new_name: String,
        ) -> anyhow::Result<User> {
            let user_to_update = user::Entity::find_by_id(id as i32)
                .one(self.db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("User with ID {} not found", id))?;

            let mut active_model: user::ActiveModel = user_to_update.into();
            active_model.name = ActiveValue::Set(new_name.clone());
            let updated_model = active_model.update(self.db).await?;

            Ok(User::new(
                updated_model.id as u32,
                updated_model.discord_id as u64,
                updated_model.name,
            ))
        }

        /// Retrieves all users from the database.
        ///
        /// # Returns
        ///
        /// A `Result` containing a vector of `User` if successful, or an error otherwise.
        #[tracing::instrument(skip(self))]
        pub async fn get_all_users(&self) -> anyhow::Result<Vec<User>> {
            let users = user::Entity::find()
                .all(self.db)
                .await?
                .into_iter()
                .map(|model| User::new(model.id as u32, model.discord_id as u64, model.name))
                .collect();
            Ok(users)
        }
    }

    pub struct UserController<'a> {
        user_service: UserService<'a>,
    }

    impl UserController<'_> {
        pub fn new(user_service: UserService) -> UserController {
            UserController { user_service }
        }

        /// Handles the creation of a new user.
        ///
        /// # Arguments
        ///
        /// * `discord_id` - The Discord ID of the user.
        /// * `name` - The name of the user.
        ///
        /// # Returns
        ///
        /// A `Result` containing the created `User` if successful, or an error otherwise.
        #[tracing::instrument(skip(self))]
        pub async fn create_user(
            &self,
            discord_id: u64,
            name: String,
        ) -> anyhow::Result<User> {
            self.user_service.create_user(discord_id, name).await
        }
    }
}

pub mod web {
    use askama::Template;
    use axum::extract::{Extension, Form};
    use axum::http::StatusCode;
    use axum::response::{Html, IntoResponse, Response};
    use migration::MigratorTrait;
    use sea_orm::Database;
    use std::sync::Arc;

    use crate::config;

    /// Custom error type for web handler operations.
    #[derive(Debug, thiserror::Error)]
    enum WebError {
        /// Represents an error during template rendering.
        /// The specific `askama::Error` is captured as the source of this error.
        #[error("Template rendering failed")]
        Template(#[from] askama::Error),
    }

    impl IntoResponse for WebError {
        fn into_response(self) -> Response {
            // The error itself (self) and its source (self.source()) are available
            // for any higher-level error reporting or logging mechanisms configured
            // in the application (e.g., a tracing subscriber).
            // This IntoResponse implementation focuses on providing a user-friendly
            // error page without exposing internal error details.

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

    /// Represents the login request payload.
    #[derive(serde::Deserialize, Debug)]
    struct LoginRequest {
        username: String,
        password: String,
    }

    #[tracing::instrument]
    pub async fn start_web_server(config: config::Config) -> anyhow::Result<()> {
        use axum::Router;

        let shared_config = Arc::new(config);

        let app = Router::new()
            .route("/health", axum::routing::get(health_check))
            .route("/", axum::routing::get(welcome))
            .route("/login", axum::routing::post(login_handler)) // Add login route
            .layer(Extension(shared_config.clone())); // Add config as an extension

        let server_address = format!("0.0.0.0:{}", &shared_config.port);
        let listener = tokio::net::TcpListener::bind(&server_address).await?;
        tracing::info!("Web server running on http://{}", server_address);

        let db = Database::connect(&shared_config.db_url).await?;
        migration::Migrator::up(&db, None).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }

    #[tracing::instrument]
    async fn health_check() -> &'static str {
        "OK"
    }

    /// Handles the login request.
    /// Checks submitted username and password against admin credentials.
    #[tracing::instrument(skip(config, payload))]
    async fn login_handler(
        Extension(config): Extension<Arc<config::Config>>,
        Form(payload): Form<LoginRequest>,
    ) -> Result<Html<String>, WebError> {
        if payload.username == config.admin_username && payload.password == config.admin_password {
            LoginSuccessTemplate {
                name: &payload.username,
            }
            .render()
            .map(Html)
            .map_err(WebError::from)
        } else {
            LoginFailureTemplate
                .render()
                .map(Html)
                .map_err(WebError::from)
        }
    }

    async fn welcome() -> Result<Html<String>, WebError> {
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
    #[template(path = "login_failure.html")]
    struct LoginFailureTemplate;

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::config::Config;
        use axum::Router;
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt; // for `oneshot`

        async fn test_app(config: Config) -> axum::Router {
            Router::new()
                .route("/login", axum::routing::post(login_handler))
                .layer(Extension(Arc::new(config)))
        }

        #[tokio::test]
        async fn test_health_check() {
            let response = health_check().await;
            assert_eq!(response, "OK");
        }

        #[tokio::test]
        async fn login_handler_successful_login() {
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
        async fn login_handler_invalid_credentials() {
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

            assert_eq!(response.status(), StatusCode::OK); // Axum returns OK with HTML body for Form errors by default

            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            assert_eq!(body, LoginFailureTemplate.render().unwrap());
        }

        #[tokio::test]
        async fn renders_welcome_page_with_html_content_type() {
            let result = welcome().await;
            assert!(
                result.is_ok(),
                "welcome() returned an error: {:?}",
                result.err()
            );
            let response = result.unwrap().into_response();
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
        async fn web_error_template_into_response_returns_internal_server_error() {
            // Simulate a template rendering error using askama::Error::Custom
            let custom_error_message = "Simulated template rendering failure".to_string();
            let template_error = askama::Error::Custom(custom_error_message.into());

            let web_error = WebError::Template(template_error);
            let response = web_error.into_response();

            assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let expected_error_message = "<h1>Internal Server Error</h1><p>An unexpected error occurred while processing your request. Please try again later.</p>";
            assert_eq!(std::str::from_utf8(&body).unwrap(), expected_error_message);
        }
    }
}
