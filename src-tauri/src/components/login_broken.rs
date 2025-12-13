use dioxus::prelude::*;
use dioxus_desktop::use_window;
use crate::auth::{AuthService, User};
use crate::components::icons::SparkIcon;

#[derive(Props, Clone, PartialEq)]
pub struct LoginProps {
    pub auth_service: Signal<AuthService>,
}

#[allow(non_snake_case)]
pub fn LoginPanel(mut props: LoginProps) -> Element {
    let mut email_input = use_signal(String::new);
    let mut token_input = use_signal(String::new);

    let auth = props.auth_service.read();
    let mut show_oauth = use_signal(|| None as Option<String>);

    rsx! {
        div {
            class: "flex-1 flex flex-col p-4",
            style: "display: flex; flex-direction: column; overflow: hidden; justify-content: center; align-items: center;",

            if auth.is_authenticated() {
                if let Some(user) = auth.get_user() {
                    // User logged in
                    div {
                        class: "w-full max-w-md",
                        style: "background: linear-gradient(150deg, #1c162b 0%, #120e1a 55%, #0f0c1a 100%); padding: 2rem; border-radius: 16px; border: 1px solid #3a2d56; box-shadow: 0 14px 34px #00000066;",

                        div { class: "flex items-center gap-3 mb-4",
                            if let Some(photo) = &user.photo_url {
                                img { src: "{photo}", style: "width: 48px; height: 48px; border-radius: 50%; border: 2px solid #ffcc00;" }
                            } else {
                                div { style: "width: 48px; height: 48px; border-radius: 50%; background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); display: flex; align-items: center; justify-content: center;",
                                    span { style: "font-weight: bold; color: #120e1a; font-size: 20px;", "{user.name.chars().next().unwrap_or('U')}" }
                                }
                            }
                            div {
                                div { style: "color: #f7f2ff; font-weight: bold; font-size: 16px;", "{user.name}" }
                                div { style: "color: #a99ec3; font-size: 13px;", "{user.email}" }
                            }
                        }

                        div { class: "mb-4",
                            div { style: "color: #7aebbe; font-size: 14px; font-weight: bold; margin-bottom: 8px;", "✓ Authenticated" }
                            p { style: "color: #cbd5ff; font-size: 13px; margin: 0;", "Your API keys are encrypted in Firebase." }
                        }

                        button {
                            class: "w-full px-4 py-2 rounded-lg font-bold",
                            style: "background: linear-gradient(135deg, #e040fb 0%, #ffcc00 45%, #7aebbe 100%); color: #120e1a; border: 1px solid #ffcc00; cursor: pointer;",
                            onclick: move |_| {
                                props.auth_service.write().logout();
                                email_input.set(String::new());
                                token_input.set(String::new());
                            },
                            "Logout"
                        }
                    }

                    // Manual token input for testing
                    div { class: "mt-4 pt-4 border-t", style: "border-color: #3a2d56;",
                        div { style: "color: #a99ec3; font-size: 12px; margin-bottom: 8px;", "Or paste your auth token:" }
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Firebase ID Token",
                            value: "{token_input}",
                            oninput: move |e| token_input.set(e.value()),
                        }
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Email",
                            value: "{email_input}",
                            oninput: move |e| email_input.set(e.value()),
                        }
                        button {
                            class: "w-full px-3 py-2 rounded-md font-bold text-sm",
                            style: "background: linear-gradient(135deg, #ffcc00 0%, #ffa500 100%); color: #120e1a; border: 1px solid #ffcc00; cursor: pointer;",
                            onclick: move |_| {
                                if !email_input().is_empty() && !token_input().is_empty() {
                                    let user = User {
                                        uid: uuid::Uuid::new_v4().to_string(),
                                        email: email_input(),
                                        name: email_input().split('@').next().unwrap_or("User").to_string(),
                                        photo_url: None,
                                        id_token: token_input(),
                                    };
                                    props.auth_service.write().set_user(user);
                                }
                            },
                            "Login"
                        }
                    }
                }
            } else {
                // Not authenticated: show OAuth + manual options
                div {
                    class: "w-full max-w-md",
                    style: "background: linear-gradient(150deg, #1c162b 0%, #120e1a 55%, #0f0c1a 100%); padding: 2rem; border-radius: 16px; border: 1px solid #3a2d56; box-shadow: 0 14px 34px #00000066;",

                    div { class: "flex items-center gap-2 mb-4",
                        SparkIcon {},
                        div { style: "color: #f7f2ff; font-weight: bold; font-size: 16px;", "Sign in to sync settings (optional)" }
                    }

                    div { class: "space-y-2 mb-4",
                        button {
                            class: "w-full px-4 py-3 rounded-lg font-bold flex items-center justify-center gap-2",
                            style: "background: linear-gradient(135deg, #4285f4 0%, #357ae8 100%); color: white; border: 1px solid #357ae8; cursor: pointer;",
                            onclick: move |_| {
                                let api_key = std::env::var("VITE_FIREBASE_API_KEY").unwrap_or_default();
                                let auth_domain = std::env::var("VITE_FIREBASE_AUTH_DOMAIN").unwrap_or_default();
                                let project_id = std::env::var("VITE_FIREBASE_PROJECT_ID").unwrap_or_default();
                                if api_key.is_empty() || auth_domain.is_empty() || project_id.is_empty() {
                                    log::error!("Missing Firebase env (VITE_FIREBASE_API_KEY/AUTH_DOMAIN/PROJECT_ID)");
                                    return;
                                }
                                let url = format!("/oauth.html?apiKey={}&authDomain={}&projectId={}&provider=google", api_key, auth_domain, project_id);
                                show_oauth.set(Some(url));
                            },
                            span { "🔵 Sign in with Google" }
                        }
                        button {
                            class: "w-full px-4 py-3 rounded-lg font-bold flex items-center justify-center gap-2",
                            style: "background: linear-gradient(135deg, #24292e 0%, #1a1e22 100%); color: white; border: 1px solid #444; cursor: pointer;",
                            onclick: move |_| {
                                let api_key = std::env::var("VITE_FIREBASE_API_KEY").unwrap_or_default();
                                let auth_domain = std::env::var("VITE_FIREBASE_AUTH_DOMAIN").unwrap_or_default();
                                let project_id = std::env::var("VITE_FIREBASE_PROJECT_ID").unwrap_or_default();
                                if api_key.is_empty() || auth_domain.is_empty() || project_id.is_empty() {
                                    log::error!("Missing Firebase env (VITE_FIREBASE_API_KEY/AUTH_DOMAIN/PROJECT_ID)");
                                    return;
                                }
                                let url = format!("/oauth.html?apiKey={}&authDomain={}&projectId={}&provider=github", api_key, auth_domain, project_id);
                                show_oauth.set(Some(url));
                            },
                            span { "⚫ Sign in with GitHub" }
                        }
                    }

                    // Manual token input
                    div { class: "mt-2",
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Firebase ID Token",
                            value: "{token_input}",
                            oninput: move |e| token_input.set(e.value()),
                        }
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Email",
                            value: "{email_input}",
                            oninput: move |e| email_input.set(e.value()),
                        }
                        button {
                            class: "w-full px-3 py-2 rounded-md font-bold text-sm",
                            style: "background: linear-gradient(135deg, #ffcc00 0%, #ffa500 100%); color: #120e1a; border: 1px solid #ffcc00; cursor: pointer;",
                            onclick: move |_| {
                                if !email_input().is_empty() && !token_input().is_empty() {
                                    let user = User {
                                        uid: uuid::Uuid::new_v4().to_string(),
                                        email: email_input(),
                                        name: email_input().split('@').next().unwrap_or("User").to_string(),
                                        photo_url: None,
                                        id_token: token_input(),
                                    };
                                    props.auth_service.write().set_user(user);
                                }
                            },
                            "Login"
                        }
                    }

                    // OAuth modal iframe
                    if let Some(url) = show_oauth() {
                        div {
                            class: "fixed inset-0 bg-black/60 flex items-center justify-center",
                            div {
                                class: "bg-[#0f0b1a] border border-[#3a2d56] rounded-xl shadow-xl w-[720px] h-[560px] overflow-hidden",
                                div { class: "flex items-center justify-between p-2 border-b", style: "border-color: #3a2d56;",
                                    div { style: "color: #a99ec3; font-size: 12px;", "Firebase Login" }
                                    button {
                                        class: "px-2 py-1 text-sm rounded-md",
                                        style: "background: #1a1426; color: #cbd5ff; border: 1px solid #3a2d56;",
                                        onclick: move |_| show_oauth.set(None),
                                        "Close"
                                    }
                                }
                                iframe { src: "{url}", style: "width: 100%; height: calc(100% - 36px); border: none; background: #0f0b1a;" }
                            }
                        }
                    }
                }
            }
        }
    }
}

    rsx! {
        div {
            class: "flex-1 flex flex-col p-4",
            style: "display: flex; flex-direction: column; overflow: hidden; justify-content: center; align-items: center;",

            if auth.is_authenticated() {
                if let Some(user) = auth.get_user() {
                    // User logged in
                    div {
                        class: "w-full max-w-md",
                        style: "background: linear-gradient(150deg, #1c162b 0%, #120e1a 55%, #0f0c1a 100%); padding: 2rem; border-radius: 16px; border: 1px solid #3a2d56; box-shadow: 0 14px 34px #00000066;",

                        div { class: "flex items-center gap-3 mb-4",
                            if let Some(photo) = &user.photo_url {
                                img { src: "{photo}", style: "width: 48px; height: 48px; border-radius: 50%; border: 2px solid #ffcc00;" }
                            } else {
                                div { style: "width: 48px; height: 48px; border-radius: 50%; background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); display: flex; align-items: center; justify-content: center;",
                                    span { style: "font-weight: bold; color: #120e1a; font-size: 20px;", "{user.name.chars().next().unwrap_or('U')}" }
                                }
                            }
                            div {
                                div { style: "color: #f7f2ff; font-weight: bold; font-size: 16px;", "{user.name}" }
                                div { style: "color: #a99ec3; font-size: 13px;", "{user.email}" }
                            }
                        }

                        div { class: "mb-4",
                            div { style: "color: #7aebbe; font-size: 14px; font-weight: bold; margin-bottom: 8px;", "✓ Authenticated" }
                            p { style: "color: #cbd5ff; font-size: 13px; margin: 0;", "Your API keys are encrypted in Firebase." }
                        }

                        button {
                            class: "w-full px-4 py-2 rounded-lg font-bold",
                            style: "background: linear-gradient(135deg, #e040fb 0%, #ffcc00 45%, #7aebbe 100%); color: #120e1a; border: 1px solid #ffcc00; cursor: pointer;",
                            onclick: move |_| {
                                props.auth_service.write().logout();
                                email_input.set(String::new());
                                token_input.set(String::new());
                            },
                            "Logout"
                        }
                    }

                    // Manual token input for testing
                    div { class: "mt-4 pt-4 border-t", style: "border-color: #3a2d56;",
                        div { style: "color: #a99ec3; font-size: 12px; margin-bottom: 8px;", "Or paste your auth token:" }
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Firebase ID Token",
                            value: "{token_input}",
                            oninput: move |e| token_input.set(e.value()),
                        }
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Email",
                            value: "{email_input}",
                            oninput: move |e| email_input.set(e.value()),
                        }
                        button {
                            class: "w-full px-3 py-2 rounded-md font-bold text-sm",
                            style: "background: linear-gradient(135deg, #ffcc00 0%, #ffa500 100%); color: #120e1a; border: 1px solid #ffcc00; cursor: pointer;",
                            onclick: move |_| {
                                if !email_input().is_empty() && !token_input().is_empty() {
                                    let user = User {
                                        uid: uuid::Uuid::new_v4().to_string(),
                                        email: email_input(),
                                        name: email_input().split('@').next().unwrap_or("User").to_string(),
                                        photo_url: None,
                                        id_token: token_input(),
                                    };
                                    props.auth_service.write().set_user(user);
                                }
                            },
                            "Login"
                        }
                    }
                }
            } else {
                // Not authenticated: show OAuth + manual options
                div {
                    class: "w-full max-w-md",
                    style: "background: linear-gradient(150deg, #1c162b 0%, #120e1a 55%, #0f0c1a 100%); padding: 2rem; border-radius: 16px; border: 1px solid #3a2d56; box-shadow: 0 14px 34px #00000066;",

                    div { class: "flex items-center gap-2 mb-4",
                        SparkIcon {},
                        div { style: "color: #f7f2ff; font-weight: bold; font-size: 16px;", "Sign in to sync settings (optional)" }
                    }

                    div { class: "space-y-2 mb-4",
                        button {
                            class: "w-full px-4 py-3 rounded-lg font-bold flex items-center justify-center gap-2",
                            style: "background: linear-gradient(135deg, #4285f4 0%, #357ae8 100%); color: white; border: 1px solid #357ae8; cursor: pointer;",
                            onclick: move |_| {
                                let api_key = std::env::var("VITE_FIREBASE_API_KEY").unwrap_or_default();
                                let auth_domain = std::env::var("VITE_FIREBASE_AUTH_DOMAIN").unwrap_or_default();
                                let project_id = std::env::var("VITE_FIREBASE_PROJECT_ID").unwrap_or_default();
                                if api_key.is_empty() || auth_domain.is_empty() || project_id.is_empty() {
                                    log::error!("Missing Firebase env (VITE_FIREBASE_API_KEY/AUTH_DOMAIN/PROJECT_ID)");
                                    return;
                                }
                                let url = format!("/oauth.html?apiKey={}&authDomain={}&projectId={}&provider=google", api_key, auth_domain, project_id);
                                show_oauth.set(Some(url));
                            },
                            span { "🔵 Sign in with Google" }
                        }
                        button {
                            class: "w-full px-4 py-3 rounded-lg font-bold flex items-center justify-center gap-2",
                            style: "background: linear-gradient(135deg, #24292e 0%, #1a1e22 100%); color: white; border: 1px solid #444; cursor: pointer;",
                            onclick: move |_| {
                                let api_key = std::env::var("VITE_FIREBASE_API_KEY").unwrap_or_default();
                                let auth_domain = std::env::var("VITE_FIREBASE_AUTH_DOMAIN").unwrap_or_default();
                                let project_id = std::env::var("VITE_FIREBASE_PROJECT_ID").unwrap_or_default();
                                if api_key.is_empty() || auth_domain.is_empty() || project_id.is_empty() {
                                    log::error!("Missing Firebase env (VITE_FIREBASE_API_KEY/AUTH_DOMAIN/PROJECT_ID)");
                                    return;
                                }
                                let url = format!("/oauth.html?apiKey={}&authDomain={}&projectId={}&provider=github", api_key, auth_domain, project_id);
                                show_oauth.set(Some(url));
                            },
                            span { "⚫ Sign in with GitHub" }
                        }
                    }

                    // Manual token input
                    div { class: "mt-2",
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Firebase ID Token",
                            value: "{token_input}",
                            oninput: move |e| token_input.set(e.value()),
                        }
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Email",
                            value: "{email_input}",
                            oninput: move |e| email_input.set(e.value()),
                        }
                        button {
                            class: "w-full px-3 py-2 rounded-md font-bold text-sm",
                            style: "background: linear-gradient(135deg, #ffcc00 0%, #ffa500 100%); color: #120e1a; border: 1px solid #ffcc00; cursor: pointer;",
                            onclick: move |_| {
                                if !email_input().is_empty() && !token_input().is_empty() {
                                    let user = User {
                                        uid: uuid::Uuid::new_v4().to_string(),
                                        email: email_input(),
                                        name: email_input().split('@').next().unwrap_or("User").to_string(),
                                        photo_url: None,
                                        id_token: token_input(),
                                    };
                                    props.auth_service.write().set_user(user);
                                }
                            },
                            "Login"
                        }
                    }

                    // OAuth modal iframe
                    if let Some(url) = show_oauth() {
                        div {
                            class: "fixed inset-0 bg-black/60 flex items-center justify-center",
                            div {
                                class: "bg-[#0f0b1a] border border-[#3a2d56] rounded-xl shadow-xl w-[720px] h-[560px] overflow-hidden",
                                div { class: "flex items-center justify-between p-2 border-b", style: "border-color: #3a2d56;",
                                    div { style: "color: #a99ec3; font-size: 12px;", "Firebase Login" }
                                    button {
                                        class: "px-2 py-1 text-sm rounded-md",
                                        style: "background: #1a1426; color: #cbd5ff; border: 1px solid #3a2d56;",
                                        onclick: move |_| show_oauth.set(None),
                                        "Close"
                                    }
                                }
                                iframe { src: "{url}", style: "width: 100%; height: calc(100% - 36px); border: none; background: #0f0b1a;" }
                            }
                        }
                    }
                }
            }
        }
    }
}
