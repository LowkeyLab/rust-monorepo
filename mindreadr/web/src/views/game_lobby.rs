use crate::components::{ErrorMessage, Header, LoadingSpinner};
use crate::state::use_game_player_map;
use dioxus::prelude::*;
use mindreadr_core::{GameState, PlayerName};
use serde::{Deserialize, Serialize};

/// Detailed view of a single game lobby. Shows players and allows joining while waiting.
#[component]
pub fn GameLobby(game_id: u32) -> Element {
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut game = use_signal(|| None::<GameDetail>);
    let mut joining = use_signal(|| false);

    // Initial load of the game
    use_effect(move || {
        let gid = game_id;
        spawn(async move {
            match get_game(gid).await {
                Ok(g) => {
                    game.set(Some(g));
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load game: {e}")));
                    loading.set(false);
                }
            }
        });
    });

    let handle_join = move |_| {
        if joining() {
            return;
        }
        joining.set(true);
        let gid = game_id;
        spawn(async move {
            let map = use_game_player_map();
            match join_game(gid).await {
                Ok(resp) => {
                    let mut map_handle = map;
                    map_handle.update(|m| m.assign(resp.game.id, resp.player_name.clone()));
                    game.set(Some(resp.game));
                    joining.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to join game: {e}")));
                    joining.set(false);
                }
            }
        });
    };

    rsx! {
        Header {}
        main { class: "min-h-screen bg-gray-50 py-8",
            div { class: "max-w-3xl mx-auto px-6",
                if loading() { LoadingSpinner { message: "Loading game...".to_string() } }
                else if let Some(err) = error() { ErrorMessage { message: err } }
                else if let Some(detail) = game() {
                    let state_color = match detail.state { GameState::WaitingForPlayers => "bg-yellow-100 text-yellow-800", GameState::InProgress => "bg-green-100 text-green-800", GameState::Finished => "bg-gray-100 text-gray-800" };
                    let state_text = match detail.state { GameState::WaitingForPlayers => "Waiting for Players", GameState::InProgress => "In Progress", GameState::Finished => "Finished" };
                    let full = detail.player_count >= 2;
                    div { class: "bg-white rounded-lg shadow p-6 space-y-6",
                        div { class: "flex justify-between items-start",
                            h1 { class: "text-2xl font-bold", "Game #{detail.id}" }
                            span { class: "px-2 py-1 rounded-full text-xs font-medium {state_color}", "{state_text}" }
                        }
                        div { class: "space-y-2",
                            h2 { class: "text-lg font-semibold", "Players ({detail.player_count}/2)" }
                            if detail.players.is_empty() { p { class: "text-gray-500", "No players yet" } }
                            else {
                                ul { class: "list-disc list-inside text-gray-700 text-sm",
                                    {detail.players.iter().map(|p| rsx!{ li { key: "{p.as_str()}", "{p.as_str()}" } })}
                                }
                            }
                        }
                        div { class: "pt-4",
                            if matches!(detail.state, GameState::WaitingForPlayers) && !full {
                                button { class: "bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 disabled:opacity-50", disabled: joining(), onclick: handle_join,
                                    if joining() { "Joining..." } else { "Join Game" }
                                }
                            } else if matches!(detail.state, GameState::InProgress) {
                                span { class: "text-green-700", "Game in progress." }
                            } else if full {
                                span { class: "text-gray-600", "Game is full." }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Detailed game information for lobby view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct GameDetail {
    id: u32,
    player_count: usize,
    state: GameState,
    players: Vec<PlayerName>,
}

/// Fetch a single game and map into lobby detail.
#[server]
async fn get_game(game_id: u32) -> Result<GameDetail, ServerFnError> {
    use crate::server::get_db_pool;
    let db = get_db_pool().await;
    let game = crate::views::games::backend::get_game(game_id)(db).await?;
    Ok(GameDetail {
        id: game.id,
        player_count: game.players.len(),
        state: game.state,
        players: game.players,
    })
}

/// Response after successfully joining a game: updated game plus assigned player name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct JoinGameResponseDetail {
    game: GameDetail,
    player_name: String,
}

/// Add the current user (anonymous) as a player to the game.
#[server]
async fn join_game(game_id: u32) -> Result<JoinGameResponseDetail, ServerFnError> {
    use crate::server::get_db_pool;
    let db = get_db_pool().await;
    let resp = crate::views::games::backend::add_player(game_id)(db).await?;
    Ok(JoinGameResponseDetail {
        game: GameDetail {
            id: resp.game.id,
            player_count: resp.game.players.len(),
            state: resp.game.state,
            players: resp.game.players,
        },
        player_name: resp.player_id,
    })
}
