use dioxus::prelude::*;

/// Modal component for collecting user's name
#[component]
pub fn NameInputModal(
    show: bool,
    on_name_submit: EventHandler<String>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut name_input = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let handle_submit = move |evt: FormEvent| {
        evt.prevent_default();
        let name = name_input().trim().to_string();

        if name.is_empty() {
            error.set(Some("Please enter your name".to_string()));
            return;
        }

        if name.len() < 2 {
            error.set(Some("Name must be at least 2 characters long".to_string()));
            return;
        }

        if name.len() > 20 {
            error.set(Some(
                "Name must be no more than 20 characters long".to_string(),
            ));
            return;
        }

        error.set(None);
        on_name_submit.call(name);
    };

    let handle_input = move |evt: FormEvent| {
        name_input.set(evt.value());
        if error().is_some() {
            error.set(None);
        }
    };

    if !show {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| on_cancel.call(()),

            div {
                class: "bg-white rounded-lg shadow-xl p-6 w-full max-w-md mx-4",
                onclick: move |evt| evt.stop_propagation(),

                h2 { class: "text-2xl font-bold text-gray-900 mb-4 text-center",
                    "What's your name?"
                }

                p { class: "text-gray-600 mb-6 text-center",
                    "We'll save your name on this device so you don't have to enter it again."
                }

                form { onsubmit: handle_submit,
                    div { class: "mb-4",
                        input {
                            r#type: "text",
                            placeholder: "Enter your name",
                            value: "{name_input}",
                            oninput: handle_input,
                            class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                            autofocus: true,
                            maxlength: 20
                        }
                    }

                    if let Some(error_msg) = error() {
                        div { class: "mb-4 text-red-600 text-sm", "{error_msg}" }
                    }

                    div { class: "flex space-x-3",
                        button {
                            r#type: "submit",
                            class: "flex-1 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium",
                            "Save Name"
                        }
                        button {
                            r#type: "button",
                            onclick: move |_| on_cancel.call(()),
                            class: "flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors",
                            "Cancel"
                        }
                    }
                }
            }
        }
    }
}
