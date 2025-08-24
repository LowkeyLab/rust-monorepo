use crate::components::{ErrorMessage, Header, LoadingSpinner};
use crate::state::use_game_player_map;
use dioxus::prelude::*;
use mindreadr_core::game::{GameState, PlayerName};
use serde::{Deserialize, Serialize};

/// Detailed view of a single game lobby. Always attempts to join via backend and displays any join error.
#[component]
pub fn GameLobby(game_id: u32) -> Element {
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>); // fatal load error
    let mut join_error = use_signal(|| None::<String>); // non-fatal join attempt error
    let mut game = use_signal(|| None::<GetGameDto>); // renamed
    let player_map = use_game_player_map();

    // On mount: attempt to join first; if join fails, fetch game to display current state.
    use_effect(move || {
        let player_map = player_map.clone();
        let gid = game_id;
        spawn(async move {
            match join_game(gid).await {
                Ok(resp) => {
                    // Persist player mapping
                    let mut pm = player_map;
                    pm.update(|m| m.assign(resp.game.id, resp.player_name.clone()));
                    game.set(Some(resp.game));
                    loading.set(false);
                }
                Err(join_e) => {
                    // Record join error, then fetch game state to show user.
                    join_error.set(Some(format!("Join failed: {join_e}")));
                    match get_game(gid).await {
                        Ok(g) => {
                            game.set(Some(g));
                            loading.set(false);
                        }
                        Err(fetch_e) => {
                            error.set(Some(format!(
                                "Failed to load game after join error: {fetch_e}"
                            )));
                            loading.set(false);
                        }
                    }
                }
            }
        });
    });

    rsx! {
        Header {}
        main { class: "min-h-screen bg-gray-50 py-8",
            div { class: "max-w-3xl mx-auto px-6 space-y-4",
                if loading() { LoadingSpinner { message: "Joining game...".to_string() } }
                else if let Some(err) = error() { ErrorMessage { message: err } }
                else if let Some(detail) = game() {
                    if let Some(jerr) = join_error() { ErrorMessage { message: jerr } }
                    LobbyGameDetails { detail }
                }
            }
        }
    }
}

/// Component rendering the current game lobby details.
#[component]
fn LobbyGameDetails(detail: GetGameDto) -> Element {
    // updated type
    let state_color = match detail.state {
        GameState::WaitingForPlayers => "bg-yellow-100 text-yellow-800",
        GameState::InProgress => "bg-green-100 text-green-800",
        GameState::Finished => "bg-gray-100 text-gray-800",
    };
    let state_text = match detail.state {
        GameState::WaitingForPlayers => "Waiting for Players",
        GameState::InProgress => "In Progress",
        GameState::Finished => "Finished",
    };
    let full = detail.player_count >= 2;
    rsx! {
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
            div { class: "pt-2 text-sm text-gray-600",
                if matches!(detail.state, GameState::WaitingForPlayers) && !full { span { "Waiting for players..." } }
                else if matches!(detail.state, GameState::InProgress) { span { class: "text-green-700", "Game in progress." } }
                else if full { span { "Game is full." } }
                else { span { "Game finished." } }
            }
        }
    }
}

/// DTO returned by the get_game server function.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetGameDto {
    // renamed from GameDetail
    id: u32,
    player_count: usize,
    state: GameState,
    players: Vec<PlayerName>,
}

/// Fetch a single game and map into lobby detail.
#[server]
async fn get_game(game_id: u32) -> Result<GetGameDto, ServerFnError> {
    // return type updated
    use crate::server::get_db_pool;
    let db = get_db_pool().await;
    let game = super::backend::get_game(game_id)(db).await?;
    Ok(GetGameDto {
        id: game.id,
        player_count: game.players.len(),
        state: game.state,
        players: game.players,
    })
}

/// DTO returned by the join_game server function.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JoinGameDto {
    // renamed from JoinGameResponseDetail
    game: GetGameDto,
    player_name: String,
}

/// Add the current user (anonymous) as a player to the game.
#[server]
async fn join_game(game_id: u32) -> Result<JoinGameDto, ServerFnError> {
    // return type updated
    use crate::server::get_db_pool;
    let db = get_db_pool().await;
    let player = super::backend::add_player(game_id);
    let resp = player(db).await?;
    Ok(JoinGameDto {
        game: GetGameDto {
            id: resp.game.id,
            player_count: resp.game.players.len(),
            state: resp.game.state,
            players: resp.game.players,
        },
        player_name: resp.player_id,
    })
}
