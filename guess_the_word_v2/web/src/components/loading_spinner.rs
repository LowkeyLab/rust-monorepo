use dioxus::prelude::*;

/// A loading spinner component with customizable message
#[component]
pub fn LoadingSpinner(message: Option<String>) -> Element {
    let message = message.unwrap_or_else(|| "Loading...".to_string());

    rsx! {
        div { class: "text-center py-12",
            div { class: "inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600" }
            p { class: "mt-4 text-gray-600", "{message}" }
        }
    }
}
