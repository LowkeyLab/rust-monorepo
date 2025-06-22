use crate::entities::*;
use askama::Template;
use axum::{
    Form, Router,
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::Html,
    routing::get,
};
use sea_orm::*;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CreateNameForm {
    discord_id: u64,
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct EditNameForm {
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

/// Error type for NameService operations.
#[derive(Debug, thiserror::Error)]
pub enum NameServiceError {
    /// Represents a duplicate Discord ID error.
    #[error("Discord ID {0} already exists")]
    DuplicateDiscordId(u64),
    /// Represents a database error.
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    /// Represents a name not found error.
    #[error("Name entry with ID {0} not found")]
    NameNotFound(u32),
}

pub struct NameService<'a> {
    db: &'a sea_orm::DatabaseConnection,
}

impl From<name::Model> for Name {
    fn from(model: name::Model) -> Self {
        Name::new(model.id as u32, model.discord_id as u64, model.name)
    }
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
    pub async fn create_name(
        &self,
        discord_id: u64,
        name: String,
    ) -> Result<Name, NameServiceError> {
        // Check if Discord ID already exists
        if self.discord_id_exists(discord_id).await? {
            return Err(NameServiceError::DuplicateDiscordId(discord_id));
        }

        let active_model = name::ActiveModel {
            discord_id: ActiveValue::Set(discord_id as i64),
            name: ActiveValue::Set(name.clone()),
            ..Default::default()
        };
        let created_model = active_model.insert(self.db).await?;
        Ok(Name::from(created_model))
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
    pub async fn edit_name_by_id(
        &self,
        id: u32,
        new_name: String,
    ) -> Result<Name, NameServiceError> {
        let name_to_update = name::Entity::find_by_id(id as i32)
            .one(self.db)
            .await?
            .ok_or(NameServiceError::NameNotFound(id))?;

        let mut active_model: name::ActiveModel = name_to_update.into();
        active_model.name = ActiveValue::Set(new_name.clone());
        let updated_model = active_model.update(self.db).await?;

        Ok(Name::from(updated_model))
    }

    /// Retrieves all name entries from the database.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `Name` if successful, or an error otherwise.
    #[tracing::instrument(skip(self))]
    pub async fn get_all_names(&self) -> Result<Vec<Name>, NameServiceError> {
        let names = name::Entity::find()
            .all(self.db)
            .await?
            .into_iter()
            .map(Name::from)
            .collect();
        Ok(names)
    }

    /// Deletes a name entry by their ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the name entry to delete.
    ///
    /// # Returns
    ///
    /// A `Result` containing the deleted `Name` if successful, or an error otherwise.
    #[tracing::instrument(skip(self))]
    pub async fn delete_name_by_id(&self, id: u32) -> Result<Name, NameServiceError> {
        let name_to_delete = name::Entity::find_by_id(id as i32)
            .one(self.db)
            .await?
            .ok_or(NameServiceError::NameNotFound(id))?;

        let name_copy = Name::from(name_to_delete.clone());
        name::Entity::delete_by_id(id as i32).exec(self.db).await?;
        Ok(name_copy)
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
    async fn discord_id_exists(&self, discord_id: u64) -> Result<bool, NameServiceError> {
        let existing_name = name::Entity::find()
            .filter(name::Column::DiscordId.eq(discord_id as i64))
            .one(self.db)
            .await?;
        Ok(existing_name.is_some())
    }

    /// Retrieves a name entry by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the name entry to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Name` if successful, or an error otherwise.
    #[tracing::instrument(skip(self))]
    pub async fn get_name_by_id(&self, id: u32) -> Result<Name, NameServiceError> {
        let name_model = name::Entity::find_by_id(id as i32)
            .one(self.db)
            .await?
            .ok_or(NameServiceError::NameNotFound(id))?;
        Ok(Name::from(name_model))
    }
}

/// Helper function to get all names, sort them using the provided function, and render them as a names table.
/// This reduces code duplication across handlers that need to display sorted names.
#[tracing::instrument(skip(name_service, sort_fn))]
async fn render_names_table<F>(
    name_service: &NameService<'_>,
    sort_fn: F,
) -> Result<String, NameError>
where
    F: FnOnce(&mut Vec<Name>),
{
    let mut names = name_service.get_all_names().await?;
    sort_fn(&mut names);
    let table_template = NamesTableTemplate::new(names);
    table_template.render().map_err(NameError::from)
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
    /// Represents a name service error.
    #[error("Name service error")]
    Service(#[from] NameServiceError),
    /// Represents a duplicate Discord ID error.
    #[error("A name entry already exists for this Discord ID")]
    DuplicateDiscordId,
}

impl axum::response::IntoResponse for NameError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, user_facing_error_message) = match self {
            NameError::DuplicateDiscordId => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "A name entry already exists for this Discord ID. Please use a different Discord ID.",
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred while processing your request. Please try again later.",
            ),
        };

        let error_template = ErrorMessageTemplate::new(user_facing_error_message.to_string());
        let Ok(rendered) = error_template.render() else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let mut response = (status_code, Html(rendered)).into_response();
        // Add HTMX headers to retarget the error message to the error div
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("hx-reswap"),
            HeaderValue::from_static("innerHTML"),
        );
        response.headers_mut().extend(headers);
        response
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

#[derive(Template)]
#[template(path = "names/error_message.html")]
struct ErrorMessageTemplate {
    message: String,
}

impl ErrorMessageTemplate {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

#[derive(Template)]
#[template(path = "names/edit_name_form.html")]
struct EditNameFormTemplate {
    name: Name,
}

impl EditNameFormTemplate {
    pub fn new(name: Name) -> Self {
        Self { name }
    }
}

#[derive(Template)]
#[template(path = "names/name_row.html")]
struct NameRowTemplate {
    name: Name,
}

impl NameRowTemplate {
    pub fn new(name: Name) -> Self {
        Self { name }
    }
}

/// Handler for the /names endpoint that displays all names in a table.
#[tracing::instrument(skip(state))]
async fn names_handler(State(state): State<NameState>) -> Result<Html<String>, NameError> {
    let name_service = NameService::new(&state.db);
    let mut names = name_service.get_all_names().await?;
    names.sort_by_key(|name| name.id());
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
            // Get updated names for the table and render
            let table_html = render_names_table(&name_service, |names| {
                names.sort_by_key(|name| name.id());
            })
            .await?;
            Ok(Html(table_html))
        }
        Err(NameServiceError::DuplicateDiscordId(_)) => Err(NameError::DuplicateDiscordId),
        Err(err) => Err(NameError::Service(err)),
    }
}

/// Handler for serving the add name form.
#[tracing::instrument]
async fn add_name_form_handler() -> Result<Html<String>, NameError> {
    let template = AddNameFormTemplate;
    template.render().map(Html).map_err(NameError::from)
}

/// Handler for deleting a name via POST request.
#[tracing::instrument(skip(state))]
async fn delete_name_handler(
    State(state): State<NameState>,
    axum::extract::Path(id): axum::extract::Path<u32>,
) -> Result<Html<String>, NameError> {
    let name_service = NameService::new(&state.db);

    match name_service.delete_name_by_id(id).await {
        Ok(_) => {
            // Get updated names for the table and render
            let table_html = render_names_table(&name_service, |names| {
                names.sort_by_key(|name| name.id());
            })
            .await?;
            Ok(Html(table_html))
        }
        Err(err) => Err(NameError::Service(err)),
    }
}

/// Handler for serving the edit name form.
#[tracing::instrument(skip(state))]
async fn edit_name_handler(
    State(state): State<NameState>,
    axum::extract::Path(id): axum::extract::Path<u32>,
) -> Result<Html<String>, NameError> {
    let name_service = NameService::new(&state.db);

    match name_service.get_name_by_id(id).await {
        Ok(name) => {
            let template = EditNameFormTemplate::new(name);
            template.render().map(Html).map_err(NameError::from)
        }
        Err(err) => Err(NameError::Service(err)),
    }
}

/// Handler for updating a name via PUT request.
#[tracing::instrument(skip(state))]
async fn update_name_handler(
    State(state): State<NameState>,
    axum::extract::Path(id): axum::extract::Path<u32>,
    Form(form): Form<EditNameForm>,
) -> Result<Html<String>, NameError> {
    let name_service = NameService::new(&state.db);

    match name_service.edit_name_by_id(id, form.name).await {
        Ok(_) => {
            // Get the updated name to render just this row
            let updated_name = name_service.get_name_by_id(id).await?;

            // Render only the updated name row
            let row_template = NameRowTemplate::new(updated_name);
            let row_html = row_template.render().map_err(NameError::from)?;

            Ok(Html(row_html))
        }
        Err(err) => Err(NameError::Service(err)),
    }
}

/// Handler for GET /names/{id} that returns a single name row.
#[tracing::instrument(skip(state))]
async fn get_name_row_handler(
    State(state): State<NameState>,
    axum::extract::Path(id): axum::extract::Path<u32>,
) -> Result<Html<String>, NameError> {
    let name_service = NameService::new(&state.db);

    match name_service.get_name_by_id(id).await {
        Ok(name) => {
            let template = NameRowTemplate::new(name);
            template.render().map(Html).map_err(NameError::from)
        }
        Err(err) => Err(NameError::Service(err)),
    }
}

/// Creates and returns the name router with all name-related routes.
pub fn create_name_router(state: NameState) -> Router {
    Router::new()
        .route("/names", get(names_handler).post(create_name_handler))
        .route("/names/form", get(add_name_form_handler))
        .route(
            "/names/{id}",
            get(get_name_row_handler)
                .delete(delete_name_handler)
                .put(update_name_handler),
        )
        .route("/names/{id}/edit", get(edit_name_handler))
        .with_state(state)
}
