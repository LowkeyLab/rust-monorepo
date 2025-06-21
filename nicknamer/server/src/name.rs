use crate::entities::*;
use askama::Template;
use axum::{Form, Router, extract::State, http::StatusCode, response::Html, routing::get};
use sea_orm::*;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CreateNameForm {
    discord_id: u64,
    name: String,
}

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

    /// Returns the Discord ID of the name.
    pub fn discord_id(&self) -> u64 {
        self.discord_id
    }

    /// Returns the name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the ID of the name.
    pub fn id(&self) -> u32 {
        self.id
    }
}

#[derive(Clone, Debug)]
pub struct NameState {
    pub db: Arc<sea_orm::DatabaseConnection>,
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
        // Check if Discord ID already exists
        if self.discord_id_exists(discord_id).await? {
            return Err(anyhow::anyhow!("Discord ID {} already exists", discord_id));
        }

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

    /// Checks if a name entry with the given Discord ID already exists.
    ///
    /// # Arguments
    ///
    /// * `discord_id` - The Discord ID to check for.
    ///
    /// # Returns
    ///
    /// A `Result` containing `true` if the Discord ID exists, `false` otherwise, or an error.
    #[tracing::instrument(skip(self))]
    async fn discord_id_exists(&self, discord_id: u64) -> anyhow::Result<bool> {
        let existing_name = name::Entity::find()
            .filter(name::Column::DiscordId.eq(discord_id as i64))
            .one(self.db)
            .await?;
        Ok(existing_name.is_some())
    }
}

/// Custom error type for name handler operations.
#[derive(Debug, thiserror::Error)]
enum NameError {
    /// Represents an error during template rendering.
    #[error("Template rendering failed")]
    Template(#[from] askama::Error),
    /// Represents a database error.
    #[error("Database error")]
    Database(#[from] anyhow::Error),
    /// Represents a duplicate Discord ID error.
    #[error("A name entry already exists for this Discord ID")]
    DuplicateDiscordId,
}

impl axum::response::IntoResponse for NameError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, user_facing_error_message) = match self {
            NameError::DuplicateDiscordId => (
                StatusCode::BAD_REQUEST,
                "A name entry already exists for this Discord ID. Please use a different Discord ID.",
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred while processing your request. Please try again later.",
            ),
        };

        (
            status_code,
            Html(format!(
                "<h1>Error</h1><p>{}</p>",
                user_facing_error_message
            )),
        )
            .into_response()
    }
}

#[derive(Template)]
#[template(path = "names/names.html")]
struct NamesTemplate {
    names: Vec<Name>,
}

impl NamesTemplate {
    pub fn new(names: Vec<Name>) -> Self {
        Self { names }
    }
}

#[derive(Template)]
#[template(path = "names/add_name_form.html")]
struct AddNameFormTemplate;

#[derive(Template)]
#[template(path = "names/names_table.html")]
struct NamesTableTemplate {
    names: Vec<Name>,
}

impl NamesTableTemplate {
    pub fn new(names: Vec<Name>) -> Self {
        Self { names }
    }
}

/// Handler for the /names endpoint that displays all names in a table.
#[tracing::instrument(skip(state))]
async fn names_handler(State(state): State<NameState>) -> Result<Html<String>, NameError> {
    let name_service = NameService::new(&state.db);
    let names = name_service.get_all_names().await?;
    let template = NamesTemplate::new(names);
    template.render().map(Html).map_err(NameError::from)
}

/// Handler for creating a new name via POST request.
#[tracing::instrument(skip(state))]
async fn create_name_handler(
    State(state): State<NameState>,
    Form(form): Form<CreateNameForm>,
) -> Result<Html<String>, NameError> {
    let name_service = NameService::new(&state.db);

    match name_service.create_name(form.discord_id, form.name).await {
        Ok(_) => {
            // Return the updated names table only
            let names = name_service.get_all_names().await?;
            let template = NamesTableTemplate::new(names);
            template.render().map(Html).map_err(NameError::from)
        }
        Err(err) => {
            // Check if the error is about duplicate Discord ID
            if err.to_string().contains("already exists") {
                Err(NameError::DuplicateDiscordId)
            } else {
                Err(NameError::Database(err))
            }
        }
    }
}

/// Handler for serving the add name form.
#[tracing::instrument]
async fn add_name_form_handler() -> Result<Html<String>, NameError> {
    let template = AddNameFormTemplate;
    template.render().map(Html).map_err(NameError::from)
}

/// Creates and returns the name router with all name-related routes.
pub fn create_name_router(state: NameState) -> Router {
    Router::new()
        .route("/names", get(names_handler).post(create_name_handler))
        .route("/names/form", get(add_name_form_handler))
        .with_state(state)
}
