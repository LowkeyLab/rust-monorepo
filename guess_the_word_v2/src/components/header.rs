use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    rsx! {
        header {
            class: "text-center py-4",
            h1 {
                class: "text-3xl font-bold text-gray-800",
                "MindReadr"
            }
        }
    }
}
