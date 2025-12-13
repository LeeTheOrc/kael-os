use dioxus::prelude::*;
use crate::auth::AuthService;
use crate::firebase::{self, ApiKey};

#[derive(Props, Clone, PartialEq)]
pub struct ApiKeyManagerProps {
    pub auth_service: Signal<AuthService>,
}

#[allow(non_snake_case)]
pub fn ApiKeyManager(props: ApiKeyManagerProps) -> Element {
    let mut key_name = use_signal(String::new);
    let mut key_value = use_signal(String::new);
    let mut error_message = use_signal(String::new);
    let api_keys = use_signal(Vec::<ApiKey>::new);

    let auth = props.auth_service.read();
    let user_signal = use_signal(|| auth.get_user());

    use_effect(move || {
        if let Some(user) = user_signal().clone() {
            let mut api_keys = api_keys;
            spawn(async move {
                match firebase::get_api_keys(&user).await {
                    Ok(keys) => api_keys.set(keys),
                    Err(e) => error_message.set(e),
                }
            });
        }
    });

    rsx! {
        div {
            class: "w-full max-w-md mt-4",
            style: "padding: 1.5rem; border-radius: 12px; border: 1px solid #3a2d56; background: #1c162b;",

            h2 { class: "text-lg font-bold text-gray-200 mb-2", "API Key Management" }

            if !error_message().is_empty() {
                p { class: "text-red-400 text-sm mb-2", "{error_message}" }
            }

            // Form to add a new key
            form {
                class: "flex flex-col gap-2",
                onsubmit: move |_ev| {
                    if let Some(user) = user_signal().clone() {
                        let mut api_keys_writer = api_keys;
                        spawn(async move {
                            match firebase::save_api_key(&user, &key_name(), &key_value()).await {
                                Ok(_) => {
                                    // Refresh the list of keys
                                    match firebase::get_api_keys(&user).await {
                                        Ok(keys) => api_keys_writer.set(keys),
                                        Err(e) => error_message.set(e),
                                    }
                                    key_name.set(String::new());
                                    key_value.set(String::new());
                                },
                                Err(e) => error_message.set(e),
                            }
                        });
                    }
                },
                input {
                    class: "w-full p-2 rounded-md border",
                    style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                    placeholder: "API Key Name (e.g., OpenAI)",
                    value: "{key_name}",
                    oninput: move |e| key_name.set(e.value()),
                }
                input {
                    class: "w-full p-2 rounded-md border",
                    style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                    placeholder: "API Key Value",
                    "type": "password",
                    value: "{key_value}",
                    oninput: move |e| key_value.set(e.value()),
                }
                button {
                    class: "w-full px-3 py-2 rounded-md font-bold text-sm",
                    style: "background: linear-gradient(135deg, #ffcc00 0%, #ffa500 100%); color: #120e1a; border: 1px solid #ffcc00; cursor: pointer;",
                    "type": "submit",
                    "Save Key"
                }
            }

            // List of existing keys
            div { class: "mt-4",
                h3 { class: "text-md font-semibold text-gray-300 mb-2", "Saved Keys" }
                if api_keys().is_empty() {
                    p { class: "text-sm text-gray-400", "No API keys saved yet." }
                } else {
                    ul { class: "space-y-2",
                        for key in api_keys() {
                            li {
                                class: "flex items-center justify-between p-2 rounded-md",
                                style: "background-color: #0f0b1a; border: 1px solid #3a2a50;",
                                span { class: "text-gray-300", "{key.name}" }
                                button {
                                    class: "px-2 py-1 text-xs rounded-md font-bold",
                                    style: "background: #e53e3e; color: white;",
                                    onclick: move |_| {
                                        if let Some(user) = user_signal().clone() {
                                            let key_id = key.id.clone();
                                            let mut api_keys_writer = api_keys;
                                            spawn(async move {
                                                match firebase::delete_api_key(&user, &key_id).await {
                                                    Ok(_) => {
                                                        // Refresh the list of keys
                                                        match firebase::get_api_keys(&user).await {
                                                            Ok(keys) => api_keys_writer.set(keys),
                                                            Err(e) => error_message.set(e),
                                                        }
                                                    },
                                                    Err(e) => error_message.set(e),
                                                }
                                            });
                                        }
                                    },
                                    "Delete"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
