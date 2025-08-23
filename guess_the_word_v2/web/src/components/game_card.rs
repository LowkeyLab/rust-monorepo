use crate::views::GameSummary;
use dioxus::prelude::*;
use guess_the_word_v2_core::GameState;

/// Individual game card component displaying game information and actions
#[component]
pub fn GameCard(game: GameSummary) -> Element {
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
