use super::*;

/// Header section for the games page with title and description
#[component]
pub fn GamesHeader() -> Element {
    rsx! {
        div { class: "text-center mb-8",
            h1 { class: "text-4xl font-bold text-gray-900 mb-4", "Live Games" }
            p { class: "text-lg text-gray-600",
                "Join an existing game or create a new one to start playing!"
            }
        }
    }
}

/// Component displayed when no games are available
#[component]
pub fn EmptyGamesState(on_create_game: EventHandler<()>) -> Element {
    rsx! {
        div { class: "text-center py-12",
            div { class: "text-6xl mb-4", "🎮" }
            h2 { class: "text-2xl font-semibold text-gray-900 mb-2", "No Live Games" }
            p { class: "text-gray-600 mb-6", "Be the first to start a game!" }
            button {
                class: "bg-purple-600 text-white px-6 py-3 rounded-lg font-semibold hover:bg-purple-700 transition-colors",
                onclick: move |_| on_create_game.call(()),
                "Create New Game"
            }
        }
    }
}

/// Grid component for displaying active games with header and create button
#[component]
pub fn GamesGrid(games: Vec<GameSummary>, on_create_game: EventHandler<()>) -> Element {
    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h2 { class: "text-2xl font-semibold text-gray-900",
                    "Active Games ({games.len()})"
                }
                button {
                    class: "bg-purple-600 text-white px-4 py-2 rounded-lg font-medium hover:bg-purple-700 transition-colors",
                    onclick: move |_| on_create_game.call(()),
                    "Create New Game"
                }
            }

            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                {games.iter().map(|game| rsx! {
                    GameCard { key: "{game.id}", game: game.clone() }
                })}
            }
        }
    }
}

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
                            div { key: "{player.as_str()}", class: "text-sm text-gray-600 ml-4",
                                "• {player.as_str()}"
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
