use crate::components::GameCard;
use crate::views::GameSummary;
use dioxus::prelude::*;

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
