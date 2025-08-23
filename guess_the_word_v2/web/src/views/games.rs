use crate::components::Header;
use dioxus::prelude::*;
use guess_the_word_v2_core::GameState;
use guess_the_word_v2_core::Player;
use serde::{Deserialize, Serialize};

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

    rsx! {
        Header {}
        main { class: "min-h-screen bg-gray-50 py-8",
            div { class: "max-w-6xl mx-auto px-6",
                div { class: "text-center mb-8",
                    h1 { class: "text-4xl font-bold text-gray-900 mb-4", "Live Games" }
                    p { class: "text-lg text-gray-600",
                        "Join an existing game or create a new one to start playing!"
                    }
                }

                if loading() {
                    div { class: "text-center py-12",
                        div { class: "inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600" }
                        p { class: "mt-4 text-gray-600", "Loading games..." }
                    }
                } else if let Some(error_msg) = error() {
                    div { class: "bg-red-50 border border-red-200 rounded-lg p-6 text-center",
                        p { class: "text-red-600", "{error_msg}" }
                    }
                } else if games().is_empty() {
                    div { class: "text-center py-12",
                        div { class: "text-6xl mb-4", "🎮" }
                        h2 { class: "text-2xl font-semibold text-gray-900 mb-2", "No Live Games" }
                        p { class: "text-gray-600 mb-6", "Be the first to start a game!" }
                        button {
                            class: "bg-purple-600 text-white px-6 py-3 rounded-lg font-semibold hover:bg-purple-700 transition-colors",
                            "Create New Game"
                        }
                    }
                } else {
                    div { class: "space-y-6",
                        div { class: "flex justify-between items-center",
                            h2 { class: "text-2xl font-semibold text-gray-900",
                                "Active Games ({games().len()})"
                            }
                            button {
                                class: "bg-purple-600 text-white px-4 py-2 rounded-lg font-medium hover:bg-purple-700 transition-colors",
                                "Create New Game"
                            }
                        }

                        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                            {games().iter().map(|game| rsx! {
                                GameCard { key: "{game.id}", game: game.clone() }
                            })}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GameCard(game: GameSummary) -> Element {
    let state_color = match game.state {
        GameState::WaitingForPlayers => "bg-yellow-100 text-yellow-800",
        GameState::InProgress => "bg-green-100 text-green-800",
        GameState::Finished => "bg-gray-100 text-gray-800",
    };

    let state_text = match game.state {
        GameState::WaitingForPlayers => "Waiting for Players",
        GameState::InProgress => "In Progress",
        GameState::Finished => "Finished",
    };

    rsx! {
        div { class: "bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow",
            div { class: "flex justify-between items-start mb-4",
                h3 { class: "text-xl font-semibold text-gray-900", "Game #{game.id}" }
                span { class: "px-2 py-1 rounded-full text-xs font-medium {state_color}",
                    "{state_text}"
                }
            }

            div { class: "space-y-3",
                div { class: "flex items-center text-gray-600",
                    span { class: "text-2xl mr-2", "👥" }
                    span { "{game.player_count}/2 players" }
                }

                if !game.players.is_empty() {
                    div { class: "space-y-1",
                        h4 { class: "text-sm font-medium text-gray-700", "Players:" }
                        {game.players.iter().map(|player| rsx! {
                            div { key: "{player.id}", class: "text-sm text-gray-600 ml-4",
                                "• {player.name}"
                            }
                        })}
                    }
                }

                div { class: "pt-4",
                    if matches!(game.state, GameState::WaitingForPlayers) && game.player_count < 2 {
                        button {
                            class: "w-full bg-blue-600 text-white py-2 px-4 rounded-lg font-medium hover:bg-blue-700 transition-colors",
                            "Join Game"
                        }
                    } else if matches!(game.state, GameState::InProgress) {
                        button {
                            class: "w-full bg-green-600 text-white py-2 px-4 rounded-lg font-medium hover:bg-green-700 transition-colors",
                            "Watch Game"
                        }
                    } else {
                        button {
                            class: "w-full bg-gray-400 text-white py-2 px-4 rounded-lg font-medium cursor-not-allowed",
                            disabled: true,
                            "Game Full"
                        }
                    }
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

#[server]
async fn get_games() -> Result<Vec<GameSummary>, ServerFnError> {
    use crate::server::get_db_pool;

    let db = get_db_pool().await;
    backend::get_games_waiting_for_players()(db).await?
}

#[cfg(feature = "server")]
mod backend {
    use crate::server::entities::{games, prelude::*};
    use anyhow::Result;
    use guess_the_word_v2_core::{Game, GameState, Player};
    use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Game not found")]
        NotFound,
        #[error("Database error: {0}")]
        DatabaseError(#[from] sea_orm::DbErr),
    }

    pub async fn get_games(
        state: GameState,
    ) -> impl AsyncFn(&DatabaseConnection) -> Result<Vec<Game>, Error> {
        async move |db: &DatabaseConnection| {
            let games_waiting_for_players = Games::find()
                .filter(games::Column::State.eq("WaitingForPlayers"))
                .find_with_related(Players)
                .all(db)
                .await?;
            let mut games = Vec::new();
            for (game, players) in games_waiting_for_players {
                let state = GameState::WaitingForPlayers;
                let player_list: Vec<Player> = players
                    .into_iter()
                    .map(|p| Player {
                        id: p.id as u32,
                        name: p.name,
                    })
                    .collect();
                games.push(Game {
                    id: game.id as u32,
                    state,
                    players: player_list,
                    rounds: vec![],
                    current_round: None,
                });
            }
            Ok(games)
        }
    }
}
