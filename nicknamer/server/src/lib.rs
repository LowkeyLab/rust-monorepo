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

pub mod web;
