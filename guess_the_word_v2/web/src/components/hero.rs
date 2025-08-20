use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    rsx! {
        section { class: "bg-gradient-to-br from-purple-600 to-blue-600 text-white py-16 px-6",
            div { class: "max-w-4xl mx-auto text-center",
                h2 { class: "text-5xl font-bold mb-6", "Welcome to MindReadr" }
                p { class: "text-xl mb-8 leading-relaxed",
                    "MindReadr is an exciting game where you and your friends try to read each other's minds! "
                    "Challenge your intuition, test your connection, and see how well you can predict what others are thinking."
                }
                p { class: "text-lg opacity-90 mb-8",
                    "Can you unlock the secrets of the mind? Let the games begin!"
                }
                div { class: "flex flex-col sm:flex-row gap-4 justify-center",
                    button {
                        class: "bg-white text-purple-600 px-8 py-3 rounded-lg font-semibold text-lg hover:bg-gray-100 transition-colors",
                        "Start Playing"
                    }
                    button {
                        class: "border-2 border-white text-white px-8 py-3 rounded-lg font-semibold text-lg hover:bg-white hover:text-purple-600 transition-colors",
                        "How to Play"
                    }
                }
            }
        }
    }
}
