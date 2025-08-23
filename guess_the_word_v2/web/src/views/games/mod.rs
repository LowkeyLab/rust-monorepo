use crate::components::{ErrorMessage, Header, LoadingSpinner};
use dioxus::prelude::*;
use guess_the_word_v2_core::Player;
use guess_the_word_v2_core::{Game, GameState};
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
mod backend;
mod components;

#[component]
pub fn Games() -> Element {
    let mut games = use_signal(Vec::<GameSummary>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);

    // Load games on component mount
    use_effect(move || {
        spawn(async move {
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
        // TODO: Implement game creation logic
        println!("Creating new game...");
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
    pub players: Vec<Player>,
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
    let mut summaries = Vec::new();
    for game in games {
        summaries.push(game.into());
    }
    Ok(summaries)
}
