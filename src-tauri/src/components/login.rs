use dioxus::prelude::*;
use dioxus_desktop::{use_window, Config, DesktopService};
use std::rc::Weak;
use tao::dpi::LogicalSize;
use tao::window::WindowBuilder;
use tokio::time::{sleep, Duration};
use crate::auth::{AuthService, User, firebase_sign_in_email_password, firebase_sign_up_email_password, get_google_oauth_url, exchange_google_code_for_token};
use crate::components::icons::SparkIcon;
use crate::oauth_server::OAUTH_SERVER;

#[derive(Props, Clone, PartialEq)]
pub struct LoginProps {
    pub auth_service: Signal<AuthService>,
}

#[derive(Props, Clone, PartialEq)]
struct OAuthWebviewProps {
    provider: String,
    url: String,
}

#[allow(non_snake_case)]
fn OAuthWebview(props: OAuthWebviewProps) -> Element {
    let window = use_window();
    let url = props.url.clone();

    // Push the OAuth URL into the dedicated webview window as soon as it mounts.
    use_effect(move || {
        let _ = window.webview.load_url(&url);
    });

    rsx! {
        div { class: "h-full w-full flex items-center justify-center bg-[#0f0b1a] text-[#cbd5ff]",
            div { class: "text-center space-y-2",
                div { class: "text-lg font-bold", "Opening {props.provider} sign-in" }
                div { class: "text-sm text-[#a99ec3]", "You can close this window after completing the login." }
            }
        }
    }
}

fn open_oauth_webview(
    desktop: &dioxus_desktop::DesktopContext,
    provider: &str,
    url: &str,
) -> Weak<DesktopService> {
    let cfg = Config::new().with_window(
        WindowBuilder::new()
            .with_title(format!("{provider} Sign In"))
            .with_inner_size(LogicalSize::new(480.0, 720.0))
            .with_resizable(true),
    );

    let dom = VirtualDom::new_with_props(
        OAuthWebview,
        OAuthWebviewProps {
            provider: provider.to_string(),
            url: url.to_string(),
        },
    );

    desktop.new_window(dom, cfg)
}

fn spawn_oauth_poll(
    provider: String,
    mut auth_service: Signal<AuthService>,
    mut show_oauth_message: Signal<Option<String>>,
    mut oauth_window_handle: Signal<Option<Weak<DesktopService>>>,
) {
    spawn(async move {
        let mut attempts = 0;
        let max_attempts = 240; // 120 seconds with 500ms polling

        loop {
            attempts += 1;
            log::debug!("OAuth poll attempt {} for {}", attempts, provider);

            if let Some(callback) = OAUTH_SERVER.get_callback(&provider).await {
                OAUTH_SERVER.clear_callback(&provider).await;

                log::info!("OAuth callback received for {}: code={:?}, error={:?}", 
                    provider, 
                    callback.code.is_some(), 
                    callback.error);

                if let Some(err) = callback.error {
                    log::warn!("OAuth error from provider: {}", err);
                    show_oauth_message.set(Some(format!("error_oauth:{}", err)));
                    break;
                } else if let Some(code) = callback.code.clone() {
                    if code.is_empty() {
                        log::warn!("OAuth code is empty");
                        show_oauth_message.set(Some("error_empty_code".to_string()));
                        break;
                    }
                    
                    log::info!("Exchanging OAuth code for {}: {}", provider, code);
                    
                    let result = if provider == "google" {
                        log::debug!("Calling exchange_google_code_for_token");
                        exchange_google_code_for_token(&code).await
                    } else {
                        Err("Invalid provider".to_string())
                    };

                    match result {
                        Ok(user) => {
                            log::info!("OAuth succeeded for {}, user: {}", provider, user.email);
                            auth_service.write().set_user(user);
                            show_oauth_message.set(None);
                        }
                        Err(e) => {
                            log::error!("OAuth exchange failed: {}", e);
                            show_oauth_message.set(Some(format!("error_exchange:{}", e)));
                        }
                    }
                }

                break;
            }

            if attempts >= max_attempts {
                log::warn!("OAuth poll timeout for {}", provider);
                show_oauth_message.set(Some("error_timeout".to_string()));
                break;
            }

            sleep(Duration::from_millis(500)).await;
        }

        // Close the OAuth window
        if let Some(handle) = oauth_window_handle() {
            if let Some(win) = handle.upgrade() {
                log::debug!("Closing OAuth window");
                win.close();
            }
        }
        oauth_window_handle.set(None);
    });
}

#[allow(non_snake_case)]
pub fn LoginPanel(mut props: LoginProps) -> Element {
    let desktop = use_window();
    let mut email_input = use_signal(String::new);
    let mut token_input = use_signal(String::new);
    let mut password_input = use_signal(String::new);
    let login_error = use_signal(|| None as Option<String>);
    let mut show_oauth_message = use_signal(|| None as Option<String>);
    let mut oauth_window_handle = use_signal(|| None as Option<Weak<DesktopService>>);

    let auth = props.auth_service.read();

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
                                        refresh_token: None,
                                        expires_in: None,
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

                    // OAuth button
                    div { class: "mb-4",
                        button {
                            class: "w-full px-4 py-3 rounded-lg font-bold flex items-center justify-center gap-2",
                            style: "background: linear-gradient(135deg, #4285f4 0%, #357ae8 100%); color: white; border: 1px solid #357ae8; cursor: pointer;",
                            onclick: {
                                let desktop = desktop.clone();
                                move |_| {
                                    match get_google_oauth_url() {
                                        Ok(url) => {
                                            log::info!("Opening Google OAuth in embedded webview");
                                            if let Some(handle) = oauth_window_handle() {
                                                if let Some(win) = handle.upgrade() { win.close(); }
                                            }

                                            let weak = open_oauth_webview(&desktop, "Google", &url);
                                            oauth_window_handle.set(Some(weak));
                                            show_oauth_message.set(Some("opening_webview_google".to_string()));
                                            spawn_oauth_poll("google".to_string(), props.auth_service.clone(), show_oauth_message.clone(), oauth_window_handle.clone());
                                        }
                                        Err(e) => {
                                            log::error!("Failed to build Google OAuth URL: {}", e);
                                            show_oauth_message.set(Some(format!("error_config: {}", e)));
                                        }
                                    }
                                }
                            },
                            span { "🔵 Sign in with Google" }
                        }
                    }
                    div { class: "mt-2",
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Email",
                            value: "{email_input()}",
                            oninput: move |e| email_input.set(e.value()),
                        }
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Password",
                            r#type: "password",
                            value: "{password_input()}",
                            oninput: move |e| password_input.set(e.value()),
                        }
                        if let Some(err) = login_error() { div { style: "color:#ff6b6b; font-size:12px; margin-bottom:8px;", "{err}" } }
                        div { class: "flex gap-2",
                            button {
                                class: "px-3 py-2 rounded-md font-bold text-sm",
                                style: "background: linear-gradient(135deg, #ffcc00 0%, #ffa500 100%); color: #120e1a; border: 1px solid #ffcc00; cursor: pointer;",
                                onclick: move |_| {
                                    let email = email_input();
                                    let pass = password_input();
                                    if email.is_empty() || pass.is_empty() { return; }
                                    let mut service = props.auth_service.clone();
                                    let mut set_err = login_error.clone();
                                    spawn(async move {
                                        match firebase_sign_in_email_password(&email, &pass).await {
                                            Ok(user) => {
                                                service.write().set_user(user);
                                                set_err.set(None);
                                            }
                                            Err(e) => set_err.set(Some(e)),
                                        }
                                    });
                                },
                                "Sign In"
                            }
                            button {
                                class: "px-3 py-2 rounded-md font-bold text-sm",
                                style: "background: linear-gradient(135deg, #7aebbe 0%, #4fd1c5 100%); color: #120e1a; border: 1px solid #4fd1c5; cursor: pointer;",
                                onclick: move |_| {
                                    let email = email_input();
                                    let pass = password_input();
                                    if email.is_empty() || pass.is_empty() { return; }
                                    let mut service = props.auth_service.clone();
                                    let mut set_err = login_error.clone();
                                    spawn(async move {
                                        match firebase_sign_up_email_password(&email, &pass).await {
                                            Ok(user) => {
                                                service.write().set_user(user);
                                                set_err.set(None);
                                            }
                                            Err(e) => set_err.set(Some(e)),
                                        }
                                    });
                                },
                                "Create Account"
                            }
                        }
                    }

                    // Manual token input (advanced)
                    div { class: "mt-2",
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Firebase ID Token",
                            value: "{token_input()}",
                            oninput: move |e| token_input.set(e.value()),
                        }
                        input {
                            class: "w-full p-2 rounded-md border mb-2",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; font-size: 12px;",
                            placeholder: "Email",
                            value: "{email_input()}",
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
                                        refresh_token: None,
                                        expires_in: None,
                                    };
                                    props.auth_service.write().set_user(user);
                                }
                            },
                            "Login"
                        }
                    }

                    // OAuth status modal
                    if let Some(message) = show_oauth_message() {
                        if message == "opening_webview_google" || message == "opening_webview_github" {
                            div {
                                class: "fixed inset-0 bg-black/60 flex items-center justify-center",
                                style: "z-index: 9999;",
                                div {
                                    class: "bg-[#0f0b1a] border border-[#3a2d56] rounded-xl shadow-xl w-[450px] p-6",
                                    div { class: "flex items-center justify-between mb-4",
                                        div { style: "color: #ffcc00; font-weight: bold; font-size: 16px;",
                                            if message == "opening_webview_google" { "Google Authentication" } else { "GitHub Authentication" }
                                        }
                                        button {
                                            class: "px-2 py-1 text-sm rounded-md",
                                            style: "background: #1a1426; color: #cbd5ff; border: 1px solid #3a2d56; cursor: pointer;",
                                            onclick: move |_| {
                                                if let Some(handle) = oauth_window_handle() {
                                                    if let Some(win) = handle.upgrade() { win.close(); }
                                                }
                                                oauth_window_handle.set(None);
                                                show_oauth_message.set(None);
                                            },
                                            "✕"
                                        }
                                    }
                                    div { style: "color: #a99ec3; line-height: 1.8; font-size: 14px;",
                                        p { "A secure in-app browser window opened for authentication." }
                                        p { style: "margin-top: 12px; font-size: 13px; color: #7aebbe;",
                                            "✓ After you sign in, we'll detect the callback and close the window automatically."
                                        }
                                    }
                                    button {
                                        class: "w-full px-4 py-2 rounded-lg font-bold mt-6",
                                        style: "background: #1a1426; color: #ffcc00; border: 1px solid #3a2d56; cursor: pointer;",
                                        onclick: move |_| show_oauth_message.set(Some("waiting_callback".to_string())),
                                        "Continue"
                                    }
                                }
                            }
                        } else if message == "waiting_callback" {
                            div {
                                class: "fixed inset-0 bg-black/60 flex items-center justify-center",
                                style: "z-index: 9999;",
                                div {
                                    class: "bg-[#0f0b1a] border border-[#3a2d56] rounded-xl shadow-xl w-[450px] p-6",
                                    div { class: "flex items-center justify-between mb-4",
                                        div { style: "color: #7aebbe; font-weight: bold; font-size: 16px;", "Waiting for authentication" }
                                        button {
                                            class: "px-2 py-1 text-sm rounded-md",
                                            style: "background: #1a1426; color: #cbd5ff; border: 1px solid #3a2d56; cursor: pointer;",
                                            onclick: move |_| show_oauth_message.set(None),
                                            "✕"
                                        }
                                    }
                                    div { style: "color: #a99ec3; line-height: 1.6; font-size: 13px;",
                                        p { "Complete the login in the in-app browser window. This dialog will close once the callback is received." }
                                    }
                                }
                            }
                        } else if message == "error_browser" {
                            div {
                                class: "fixed inset-0 bg-black/60 flex items-center justify-center",
                                style: "z-index: 9999;",
                                div {
                                    class: "bg-[#0f0b1a] border border-[#3a2d56] rounded-xl shadow-xl w-[450px] p-6",
                                    div { class: "flex items-center justify-between mb-4",
                                        div { style: "color: #ff6b6b; font-weight: bold; font-size: 16px;", "Browser Error" }
                                        button {
                                            class: "px-2 py-1 text-sm rounded-md",
                                            style: "background: #1a1426; color: #cbd5ff; border: 1px solid #3a2d56; cursor: pointer;",
                                            onclick: move |_| show_oauth_message.set(None),
                                            "✕"
                                        }
                                    }
                                    div { style: "color: #a99ec3; line-height: 1.6; font-size: 13px;",
                                        p { "Could not open your default browser. Please try again or use Email/Password login." }
                                    }
                                    button {
                                        class: "w-full px-4 py-2 rounded-lg font-bold mt-4",
                                        style: "background: #1a1426; color: #ffcc00; border: 1px solid #3a2d56; cursor: pointer;",
                                        onclick: move |_| show_oauth_message.set(None),
                                        "Close"
                                    }
                                }
                            }
                        } else if message == "error_timeout" {
                            div {
                                class: "fixed inset-0 bg-black/60 flex items-center justify-center",
                                style: "z-index: 9999;",
                                div {
                                    class: "bg-[#0f0b1a] border border-[#3a2d56] rounded-xl shadow-xl w-[450px] p-6",
                                    div { class: "flex items-center justify-between mb-4",
                                        div { style: "color: #ff6b6b; font-weight: bold; font-size: 16px;", "OAuth Timeout" }
                                        button {
                                            class: "px-2 py-1 text-sm rounded-md",
                                            style: "background: #1a1426; color: #cbd5ff; border: 1px solid #3a2d56; cursor: pointer;",
                                            onclick: move |_| show_oauth_message.set(None),
                                            "✕"
                                        }
                                    }
                                    div { style: "color: #a99ec3; line-height: 1.6; font-size: 13px;",
                                        p { "We didn't receive the callback in time. Please try again." }
                                    }
                                }
                            }
                        } else if let Some(err) = message.strip_prefix("error_exchange:") {
                            div {
                                class: "fixed inset-0 bg-black/60 flex items-center justify-center",
                                style: "z-index: 9999;",
                                div {
                                    class: "bg-[#0f0b1a] border border-[#3a2d56] rounded-xl shadow-xl w-[450px] p-6",
                                    div { class: "flex items-center justify-between mb-4",
                                        div { style: "color: #ff6b6b; font-weight: bold; font-size: 16px;", "Exchange Error" }
                                        button {
                                            class: "px-2 py-1 text-sm rounded-md",
                                            style: "background: #1a1426; color: #cbd5ff; border: 1px solid #3a2d56; cursor: pointer;",
                                            onclick: move |_| show_oauth_message.set(None),
                                            "✕"
                                        }
                                    }
                                    div { style: "color: #a99ec3; line-height: 1.6; font-size: 13px;",
                                        p { "Failed to exchange the OAuth code: {err}" }
                                    }
                                }
                            }
                        } else if let Some(err) = message.strip_prefix("error_oauth:") {
                            div {
                                class: "fixed inset-0 bg-black/60 flex items-center justify-center",
                                style: "z-index: 9999;",
                                div {
                                    class: "bg-[#0f0b1a] border border-[#3a2d56] rounded-xl shadow-xl w-[450px] p-6",
                                    div { class: "flex items-center justify-between mb-4",
                                        div { style: "color: #ff6b6b; font-weight: bold; font-size: 16px;", "OAuth Error" }
                                        button {
                                            class: "px-2 py-1 text-sm rounded-md",
                                            style: "background: #1a1426; color: #cbd5ff; border: 1px solid #3a2d56; cursor: pointer;",
                                            onclick: move |_| show_oauth_message.set(None),
                                            "✕"
                                        }
                                    }
                                    div { style: "color: #a99ec3; line-height: 1.6; font-size: 13px;",
                                        p { "Authentication failed: {err}" }
                                    }
                                }
                            }
                        } else if message.starts_with("error_config:") {
                            div {
                                class: "fixed inset-0 bg-black/60 flex items-center justify-center",
                                style: "z-index: 9999;",
                                div {
                                    class: "bg-[#0f0b1a] border border-[#3a2d56] rounded-xl shadow-xl w-[450px] p-6",
                                    div { class: "flex items-center justify-between mb-4",
                                        div { style: "color: #ff6b6b; font-weight: bold; font-size: 16px;", "Configuration Error" }
                                        button {
                                            class: "px-2 py-1 text-sm rounded-md",
                                            style: "background: #1a1426; color: #cbd5ff; border: 1px solid #3a2d56; cursor: pointer;",
                                            onclick: move |_| show_oauth_message.set(None),
                                            "✕"
                                        }
                                    }
                                    div { style: "color: #a99ec3; line-height: 1.6; font-size: 13px;",
                                        p { "Firebase credentials are missing. Please check your .env.local file." }
                                    }
                                    button {
                                        class: "w-full px-4 py-2 rounded-lg font-bold mt-4",
                                        style: "background: #1a1426; color: #ffcc00; border: 1px solid #3a2d56; cursor: pointer;",
                                        onclick: move |_| show_oauth_message.set(None),
                                        "Close"
                                    }
                                }
                            }
                        }
                    }

                }
            }
        }
    }
}
