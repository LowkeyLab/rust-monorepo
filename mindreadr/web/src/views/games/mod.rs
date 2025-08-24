use crate::components::{ErrorMessage, Header, LoadingSpinner};
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use mindreadr_core::game::PlayerId;
use mindreadr_core::{Game, GameState};
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
mod backend;
mod components;

#[component]
pub fn Games() -> Element {
    let mut games = use_signal(Vec::<GameSummary>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut user_name = use_signal(|| None::<String>);

    // Load username from storage on component mount
    use_effect(move || {
        spawn(async move {
            // Load username from storage
            if let Ok(stored_name) = LocalStorage::get::<String>("user_name") {
                if !stored_name.trim().is_empty() {
                    user_name.set(Some(stored_name));
                }
            }

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
        // TODO: Implement actual game creation logic
        println!("Creating new game for user: {:?}", user_name());
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameSummary {
    pub id: u32,
    pub player_count: usize,
    pub state: GameState,
    pub players: Vec<PlayerId>,
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
