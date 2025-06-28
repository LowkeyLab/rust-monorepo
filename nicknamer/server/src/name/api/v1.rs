use crate::name::{Name, NameService, NameState};
use crate::web::api::v1::ServerErrorResponse;
use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// JSON representation of a Name for API responses.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NameJson {
    /// Unique identifier for the name
    id: u32,
    /// Discord user ID associated with the name
    discord_id: u64,
    /// The actual name/nickname
    name: String,
}

impl From<Name> for NameJson {
    fn from(name: Name) -> Self {
        Self {
            id: name.id(),
            discord_id: name.discord_id(),
            name: name.name().to_string(),
        }
    }
}

/// API response for listing all names.
#[derive(Debug, Serialize, ToSchema)]
pub struct NamesResponse {
    /// List of names
    names: Vec<NameJson>,
    /// Total number of names
    count: usize,
}

/// Handler for GET /api/v1/names - Returns all names in JSON format.
#[tracing::instrument(skip(state))]
#[utoipa::path(
    get,
    path = "/api/v1/names",
    responses(
        (status = 200, description = "Successfully retrieved all names", body = NamesResponse),
        (status = 500, description = "Internal server error", body = ServerErrorResponse)
    ),
    tag = "Names"
)]
pub async fn get_names_handler(
    State(state): State<Arc<NameState>>,
) -> Result<Json<NamesResponse>, (StatusCode, Json<ServerErrorResponse>)> {
    let service = NameService::new(&state.db);

    match service.get_all_names().await {
        Ok(names) => {
            let json_names: Vec<NameJson> = names.into_iter().map(NameJson::from).collect();
            let count = json_names.len();

            Ok(Json(NamesResponse {
                names: json_names,
                count,
            }))
        }
        Err(err) => {
            tracing::error!("Failed to get names: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ServerErrorResponse::new(
                    "Failed to retrieve names".to_string(),
                )),
            ))
        }
    }
}

/// Creates and returns the names API router.
pub fn create_api_router(state: Arc<NameState>) -> Router {
    Router::new()
        .route("/names", get(get_names_handler))
        .with_state(state)
}
