use dioxus::prelude::*;

#[component]
pub fn NotFound(route: Vec<String>) -> Element {
    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center bg-gray-100",
            div {
                class: "text-center px-6",
                div {
                    class: "mb-8",
                    h1 {
                        class: "text-9xl font-bold text-gray-300 mb-4",
                        "404"
                    }
                    h2 {
                        class: "text-4xl font-bold text-gray-800 mb-4",
                        "Page Not Found"
                    }
                    p {
                        class: "text-xl text-gray-600 mb-8",
                        "Oops! The page you're looking for doesn't exist in the MindReadr universe."
                    }
                }
                div {
                    class: "space-y-4",
                    Link {
                        to: "/",
                        class: "inline-block bg-purple-600 hover:bg-purple-700 text-white font-bold py-3 px-6 rounded-lg transition-colors duration-200",
                        "Return to Home"
                    }
                    p {
                        class: "text-gray-500",
                        "Let's get back to reading minds!"
                    }
                }
            }
        }
    }
}
