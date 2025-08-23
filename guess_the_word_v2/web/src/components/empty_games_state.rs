use dioxus::prelude::*;

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
