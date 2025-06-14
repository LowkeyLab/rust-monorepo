use sea_orm_migration::prelude::*;
use tracing::info;

mod entities;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = config::Config::new();
    let db = sea_orm::Database::connect(&cfg.db_url).await?;
    migration::Migrator::up(&db, None).await?;
    Ok(())
}

mod config {
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
}

mod user {
    use crate::entities::{prelude::*, *};
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
    struct UserService<'a> {
        db: &'a sea_orm::DatabaseConnection,
    }

    impl UserService<'_> {
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use migration::MigratorTrait;
        use sea_orm::Database;
        use testcontainers_modules::testcontainers::runners::AsyncRunner;
        use testcontainers_modules::{postgres, testcontainers};
        use tracing::info;

        struct TestContext {
            container: testcontainers::ContainerAsync<postgres::Postgres>,
            db: DatabaseConnection,
        }

        async fn setup() -> anyhow::Result<TestContext> {
            tracing_subscriber::fmt().with_env_filter("debug").init();
            let container = setup_container().await?;
            let db = setup_db(&container).await?;
            Ok(TestContext { db, container })
        }

        async fn setup_container()
        -> anyhow::Result<testcontainers::ContainerAsync<postgres::Postgres>> {
            let container = postgres::Postgres::default().start().await?;
            Ok(container)
        }

        async fn setup_db(
            container: &testcontainers::ContainerAsync<postgres::Postgres>,
        ) -> anyhow::Result<DatabaseConnection> {
            let host = container.get_host().await?;
            let port = container.get_host_port_ipv4(5432).await?;
            let db_url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);
            let db = Database::connect(&db_url).await?;
            migration::Migrator::up(&db, None).await?;
            Ok(db)
        }

        #[tokio::test]
        async fn test_create_user() -> anyhow::Result<()> {
            let state = setup().await?;
            info!("Setup completed, starting test...");
            let user_service = UserService { db: &state.db };
            let user = user_service
                .create_user(123456789, "TestUser".to_string())
                .await?;
            assert_eq!(user.get_discord_id(), 123456789);
            assert_eq!(user.get_name(), "TestUser");
            Ok(())
        }
    }
}
