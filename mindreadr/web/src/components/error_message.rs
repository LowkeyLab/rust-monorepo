use dioxus::prelude::*;

/// An error message component for displaying error states
#[component]
pub fn ErrorMessage(message: String) -> Element {
    rsx! {
        div { class: "bg-red-50 border border-red-200 rounded-lg p-6 text-center",
            p { class: "text-red-600", "{message}" }
        }
    }
}
