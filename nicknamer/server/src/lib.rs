pub mod config {
    use serde::Deserialize;
    #[derive(Deserialize, Debug)]
    pub struct Config {
        pub db_url: String,
        #[serde(default = "default_port")]
        pub port: u16,
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
}

pub mod web {
    use askama::Template;
    use axum::response::IntoResponse;
    use migration::MigratorTrait;
    use sea_orm::Database;

    use crate::config;

    #[tracing::instrument]
    pub async fn start_web_server(config: config::Config) -> anyhow::Result<()> {
        use axum::Router;

        let app = Router::new()
            .route("/health", axum::routing::get(health_check))
            .route("/", axum::routing::get(welcome));
        let server_address = format!("0.0.0.0:{}", config.port);
        let listener = tokio::net::TcpListener::bind(&server_address).await?;
        tracing::info!("Web server running on http://{}", server_address);

        let db = Database::connect(&config.db_url).await?;
        migration::Migrator::up(&db, None).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }

    #[tracing::instrument]
    async fn health_check() -> &'static str {
        "OK"
    }

    async fn welcome() -> impl IntoResponse {
        IndexTemplate.render().unwrap()
    }

    #[derive(Template)]
    #[template(path = "index.html")]
    struct IndexTemplate;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_health_check() {
            let response = health_check().await;
            assert_eq!(response, "OK");
        }

        async fn test_welcome() {
            
        }
    }
}
