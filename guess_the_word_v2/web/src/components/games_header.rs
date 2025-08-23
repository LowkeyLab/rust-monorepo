use dioxus::prelude::*;

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
