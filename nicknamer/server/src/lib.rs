pub mod config {
    use dotenvy::dotenv_iter;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Config {
        pub db_url: String,
    }

    impl Config {
        pub fn new() -> Self {
            let iter = dotenv_iter()
                .expect("Failed to load .env file")
                .map(|res| res.expect("Failed to read environment variable"));
            envy::from_iter(iter).expect("Failed to parse environment variables into Config")
        }
    }

    impl Default for Config {
        fn default() -> Self {
            Self::new()
        }
    }
}
pub mod entities;
pub mod user {
    use crate::entities::*;
    use sea_orm::*;
    #[derive(Debug)]
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
        pub fn get_discord_id(&self) -> u64 {
            self.discord_id
        }
        pub fn get_name(&self) -> &str {
            &self.name
        }
    }
    pub struct UserService<'a> {
        db: &'a sea_orm::DatabaseConnection,
    }

    impl UserService<'_> {
        pub fn new(db: &sea_orm::DatabaseConnection) -> UserService {
            UserService { db }
        }

        #[tracing::instrument(skip(self))]
        pub async fn create_user(&self, discord_id: u64, name: String) -> anyhow::Result<User> {
            let active_model = user::ActiveModel {
                discord_id: ActiveValue::Set(discord_id as i64),
                name: ActiveValue::Set(name.clone()),
                ..Default::default()
            };
            active_model.insert(self.db).await?;
            Ok(User::new(0, discord_id, name))
        }
    }
}
