use crate::auth::AuthService;
use crate::firebase::{self, ApiKey};
use crate::llm::{self, LLMProvider, LLMRequest};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ApiKeyManagerProps {
    pub auth_service: Signal<AuthService>,
}

#[allow(non_snake_case)]
pub fn ApiKeyManager(props: ApiKeyManagerProps) -> Element {
    let mut key_name = use_signal(String::new);
    let mut key_value = use_signal(String::new);
    let mut error_message = use_signal(String::new);
    let mut success_message = use_signal(String::new);
    let mut test_status = use_signal(String::new);
    let api_keys = use_signal(Vec::<ApiKey>::new);

    let auth = props.auth_service.read();
    let user_signal = use_signal(|| auth.get_user());

    use_effect(move || {
        if let Some(user) = user_signal().clone() {
            let mut api_keys = api_keys;
            let mut error_msg = error_message.clone();
            spawn(async move {
                match firebase::get_api_keys(&user).await {
                    Ok(keys) => {
                        // Cache keys locally as fallback
                        if let Ok(json) = serde_json::to_string(&keys) {
                            let _ = std::fs::write("/tmp/kael_cached_api_keys.json", json);
                        }
                        api_keys.set(keys);
                    }
                    Err(e) => {
                        // Try to load from local cache if Firebase fails
                        if let Ok(cached) = std::fs::read_to_string("/tmp/kael_cached_api_keys.json") {
                            if let Ok(keys) = serde_json::from_str::<Vec<firebase::ApiKey>>(&cached) {
                                log::warn!("Firebase unavailable, using cached keys");
                                api_keys.set(keys);
                                return;
                            }
                        }
                        error_msg.set(format!("Failed to load keys: {}", e));
                    }
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
            
            if !success_message().is_empty() {
                p { class: "text-green-400 text-sm mb-2", "{success_message}" }
            }
            
            if !test_status().is_empty() {
                p { class: "text-yellow-400 text-sm mb-2", "{test_status}" }
            }

            // Form to add a new key
            form {
                class: "flex flex-col gap-2",
                onsubmit: move |_ev| {
                    if let Some(user) = user_signal().clone() {
                        let name = key_name();
                        let value = key_value();
                        let mut api_keys_writer = api_keys;
                        let mut test_signal = test_status.clone();
                        let mut error_signal = error_message.clone();
                        let mut success_signal = success_message.clone();
                        
                        test_signal.set(format!("🔐 Saving key '{}'...", name));
                        error_signal.set(String::new());
                        success_signal.set(String::new());
                        
                        spawn(async move {
                            // Save the key first
                            match firebase::save_api_key(&user, &name, &value).await {
                                Ok(_) => {
                                    test_signal.set(format!("✅ Saved! Testing '{}'...", name));
                                    
                                    // Test the key based on provider name
                                    let test_result = match name.as_str() {
                                        "Mistral AI" => {
                                            let req = LLMRequest {
                                                provider: LLMProvider::Mistral,
                                                model: String::new(),
                                                prompt: "ping".to_string(),
                                                api_key: Some(value.clone()),
                                                system: Some("Reply with 'ok'".to_string()),
                                            };
                                            llm::send_request_with_fallback(req, Some(&user), vec![]).await
                                        },
                                        "Google Gemini" => {
                                            let req = LLMRequest {
                                                provider: LLMProvider::Gemini,
                                                model: String::new(),
                                                prompt: "ping".to_string(),
                                                api_key: Some(value.clone()),
                                                system: Some("Reply with 'ok'".to_string()),
                                            };
                                            llm::send_request_with_fallback(req, Some(&user), vec![]).await
                                        },
                                        _ => {
                                            // For other providers, just mark as saved
                                            test_signal.set(format!("✅ '{}' saved (validation not implemented for this provider)", name));
                                            match firebase::get_api_keys(&user).await {
                                                Ok(keys) => api_keys_writer.set(keys),
                                                Err(e) => error_signal.set(e),
                                            }
                                            key_name.set(String::new());
                                            key_value.set(String::new());
                                            return;
                                        }
                                    };
                                    
                                    // Show test results
                                    match test_result {
                                        Ok(_) => {
                                            success_signal.set(format!("✅ '{}' is working correctly!", name));
                                            test_signal.set(String::new());
                                        },
                                        Err(e) => {
                                            error_signal.set(format!("⚠️ Key saved but test failed: {}", e));
                                            test_signal.set(String::new());
                                        }
                                    }
                                    
                                    // Refresh the list of keys
                                    match firebase::get_api_keys(&user).await {
                                        Ok(keys) => api_keys_writer.set(keys),
                                        Err(e) => error_signal.set(e),
                                    }
                                    key_name.set(String::new());
                                    key_value.set(String::new());
                                },
                                Err(e) => {
                                    error_signal.set(format!("❌ Failed to save: {}", e));
                                    test_signal.set(String::new());
                                }
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
