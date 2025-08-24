use crate::components::{ErrorMessage, Header, LoadingSpinner};
use crate::Route;
use dioxus::prelude::*;
use mindreadr_core::game::{Game, GameState, PlayerName};
use serde::{Deserialize, Serialize}; // new for navigation
#[cfg(feature = "server")]
pub mod backend; // made public
mod components;
mod lobby; // new
pub use lobby::GameLobby; // re-export

#[component]
pub fn Games() -> Element {
    let mut games = use_signal(Vec::<GameSummary>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let nav = use_navigator();

    // Initial load: fetch games list
    use_effect(move || {
        spawn(async move {
            // Load games
            match get_games().await {
                Ok(live_games) => {
                    games.set(live_games);
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load games: {}", e)));
                    loading.set(false);
                }
            }
        });
    });

    let handle_create_game = move |_| {
        let nav = nav.clone();
        spawn(async move {
            match create_game().await {
                Ok(created) => {
                    // Optimistically insert into list (optional UI improvement)
                    games.write().insert(0, created.clone());
                    // Navigate to lobby where auto-join will occur
                    nav.push(Route::GameLobby {
                        game_id: created.id,
                    });
                }
                Err(e) => {
                    error.set(Some(format!("Failed to create game: {}", e)));
                }
            }
        });
    };

    rsx! {
        Header {}

        main { class: "min-h-screen bg-gray-50 py-8",
            div { class: "max-w-6xl mx-auto px-6",
                components::GamesHeader {}

                if loading() {
                    LoadingSpinner { message: "Loading games...".to_string() }
                } else if let Some(error_msg) = error() {
                    ErrorMessage { message: error_msg }
                } else if games().is_empty() {
                    components::EmptyGamesState { on_create_game: handle_create_game }
                } else {
                    components::GamesGrid { games: games(), on_create_game: handle_create_game }
                }
            }
        }
    }
}

/// Lightweight summary of a game used for UI rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameSummary {
    pub id: u32,
    pub player_count: usize,
    pub state: GameState,
    pub players: Vec<PlayerName>,
}

impl From<Game> for GameSummary {
    fn from(game: Game) -> Self {
        GameSummary {
            id: game.id,
            player_count: game.players.len(),
            state: game.state,
            players: game.players,
        }
    }
}

#[server]
async fn get_games() -> Result<Vec<GameSummary>, ServerFnError> {
    use crate::server::get_db_pool;
    let db = get_db_pool().await;
    let games = backend::get_games(GameState::WaitingForPlayers)(db).await?;
    Ok(games.into_iter().map(Game::into).collect())
}

/// Server function that creates a new empty game (no players) and returns a summary.
#[server]
async fn create_game() -> Result<GameSummary, ServerFnError> {
    use crate::server::get_db_pool;
    let db = get_db_pool().await;
    let game = backend::create_game()(db).await?;
    Ok(game.into())
}
