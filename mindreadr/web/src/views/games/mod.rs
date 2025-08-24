use crate::components::{ErrorMessage, Header, LoadingSpinner};
use crate::state::use_mindreadr_state;
use dioxus::prelude::*;
use mindreadr_core::PlayerName;
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
    let client_state = use_mindreadr_state();

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
        let mut games_signal = games;
        let mut error_signal = error;
        spawn(async move {
            // 1. Create empty game
            let created = match create_game().await {
                Ok(g) => g,
                Err(e) => {
                    error_signal.set(Some(format!("Failed to create game: {}", e)));
                    return;
                }
            };
            let created_id = created.id;
            // Insert placeholder (empty) game at top immediately for responsiveness
            {
                let mut list = games_signal.write();
                list.insert(0, created.clone());
            }
            // 2. Join game to get assigned player id and updated state
            match join_game(created_id).await {
                Ok(resp) => {
                    // Persist mapping in client state
                    client_state.update(|st| {
                        st.game_players
                            .insert(resp.game.id, resp.player_name.clone());
                    });
                    let mut list = games_signal.write();
                    if let Some(pos) = list.iter().position(|g| g.id == resp.game.id) {
                        list.remove(pos);
                    }
                    list.insert(0, resp.game);
                }
                Err(e) => {
                    error_signal.set(Some(format!("Failed to join game {}: {}", created_id, e)));
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

/// Server function that adds a player to the specified game and returns the updated game and assigned player name.
#[server]
async fn join_game(game_id: u32) -> Result<JoinGameResponse, ServerFnError> {
    use crate::server::get_db_pool;
    let db = get_db_pool().await;
    let result = backend::add_player(game_id)(db).await?;
    Ok(JoinGameResponse {
        game: result.game.into(),
        player_name: result.player_id,
    })
}

/// Response payload when joining a game (adding a player).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JoinGameResponse {
    /// Updated game after adding the player.
    pub game: GameSummary,
    /// Player name assigned by core game logic (e.g., "Player1", "Player2").
    pub player_name: String,
}
