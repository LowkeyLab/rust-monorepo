use crate::name::{Name, NameService, NameState};
use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};
use serde::{Deserialize, Serialize};

/// JSON representation of a Name for API responses.
#[derive(Debug, Serialize, Deserialize)]
pub struct NameJson {
    id: u32,
    discord_id: u64,
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
#[derive(Debug, Serialize)]
pub struct NamesResponse {
    names: Vec<NameJson>,
    count: usize,
}

/// Error response for API errors.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    error: String,
}

/// Handler for GET /api/v1/names - Returns all names in JSON format.
#[tracing::instrument(skip(state))]
pub async fn get_names_handler(
    State(state): State<NameState>,
) -> Result<Json<NamesResponse>, (StatusCode, Json<ErrorResponse>)> {
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
                Json(ErrorResponse {
                    error: "Failed to retrieve names".to_string(),
                }),
            ))
        }
    }
}

/// Creates and returns the v1 API router.
pub fn create_v1_router(state: NameState) -> Router {
    Router::new()
        .route("/names", get(get_names_handler))
        .with_state(state)
}
