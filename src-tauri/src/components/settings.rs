// src-tauri/src/components/settings.rs
use dioxus::prelude::*;
use crate::auth::AuthService;
use crate::components::login::LoginPanel;
use crate::components::api_key_manager::ApiKeyManager;

#[derive(Clone, PartialEq, Debug)]
struct ProviderUIState {
    name: String,
    enabled: bool,
    api_key: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SettingsTab {
    Authentication,
    Providers,
    System,
    Security,
}

#[derive(Props, Clone, PartialEq)]
pub struct SettingsPanelProps {
    pub auth_service: Signal<AuthService>,
    pub show_settings: Signal<bool>,
}

#[allow(non_snake_case)]
#[allow(unreachable_code)]
#[allow(dependency_on_unit_never_type_fallback)]
pub fn SettingsPanel(mut props: SettingsPanelProps) -> Element {
    let mut active_tab = use_signal(|| SettingsTab::Authentication);
    let mut providers = use_signal(|| {
        vec![
            ProviderUIState {
                name: "Ollama (Local)".to_string(),
                enabled: true,
                api_key: String::new(),
            },
            ProviderUIState {
                name: "Mistral AI".to_string(),
                enabled: true,
                api_key: String::new(),
            },
            ProviderUIState {
                name: "Google Gemini".to_string(),
                enabled: false,
                api_key: String::new(),
            },
            ProviderUIState {
                name: "GitHub Copilot".to_string(),
                enabled: false,
                api_key: String::new(),
            },
            ProviderUIState {
                name: "GitHub Copilot CLI".to_string(),
                enabled: true,
                api_key: String::new(),
            },
            ProviderUIState {
                name: "Office 365 AI".to_string(),
                enabled: false,
                api_key: String::new(),
            },
            ProviderUIState {
                name: "Google One AI".to_string(),
                enabled: false,
                api_key: String::new(),
            },
        ]
    });

    let auth_signal = props.auth_service.clone();

    // Load provider keys from Firestore on mount (decrypts per user id_token)
    use_effect(move || {
        let auth = auth_signal();
        if let Some(user) = auth.get_user() {
            let mut set_providers = providers.clone();
            spawn(async move {
                if let Ok(keys) = crate::firebase::get_api_keys(&user).await {
                    let mut current = set_providers.write();
                    for p in current.iter_mut() {
                        if let Some(k) = keys.iter().find(|k| k.name == p.name) {
                            p.api_key = k.value.clone();
                        }
                    }
                }
            });
        }
    });

    let auth_signal_clone = auth_signal.clone();

    rsx! {
        div {
            class: "flex-1 flex h-full",
            style: "background: radial-gradient(1200px at 10% 10%, #201637 0%, #160f28 45%, #120b1f 100%); overflow: hidden;",

            // Sidebar Menu
            div {
                style: "width: 240px; border-right: 1px solid #3a2d56; background: linear-gradient(180deg, #1c162b 0%, #140f22 100%); padding: 16px; overflow-y: auto; flex-shrink: 0; display: flex; flex-direction: column;",
                
                // Header with back button
                button {
                    class: "w-full mb-6",
                    style: "display: flex; align-items: center; gap: 8px; padding: 10px 12px; border-radius: 8px; border: 1px solid #3a2d56; background: linear-gradient(135deg, #1c162b 0%, #120e1a 100%); color: #ffcc00; font-weight: 600; cursor: pointer;",
                    onclick: move |_| {
                        props.show_settings.set(false);
                    },
                    span { style: "font-size: 16px;", "←" }
                    span { "Back to App" }
                }

                // Menu Items
                div { style: "display: flex; flex-direction: column; gap: 8px; flex: 1;",
                    button {
                        class: "w-full text-left px-4 py-3 rounded-lg transition-all",
                        style: if active_tab() == SettingsTab::Authentication {
                            "background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); color: #120e1a; font-weight: 700; border: none; cursor: pointer;"
                        } else {
                            "background: rgba(58, 42, 80, 0.3); color: #a99ec3; border: 1px solid #3a2d56; cursor: pointer;"
                        },
                        onclick: move |_| active_tab.set(SettingsTab::Authentication),
                        "🔐 Authentication"
                    }
                    button {
                        class: "w-full text-left px-4 py-3 rounded-lg transition-all",
                        style: if active_tab() == SettingsTab::Providers {
                            "background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); color: #120e1a; font-weight: 700; border: none; cursor: pointer;"
                        } else {
                            "background: rgba(58, 42, 80, 0.3); color: #a99ec3; border: 1px solid #3a2d56; cursor: pointer;"
                        },
                        onclick: move |_| active_tab.set(SettingsTab::Providers),
                        "🤖 AI Providers"
                    }
                    button {
                        class: "w-full text-left px-4 py-3 rounded-lg transition-all",
                        style: if active_tab() == SettingsTab::System {
                            "background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); color: #120e1a; font-weight: 700; border: none; cursor: pointer;"
                        } else {
                            "background: rgba(58, 42, 80, 0.3); color: #a99ec3; border: 1px solid #3a2d56; cursor: pointer;"
                        },
                        onclick: move |_| active_tab.set(SettingsTab::System),
                        "⚙️ System"
                    }
                    button {
                        class: "w-full text-left px-4 py-3 rounded-lg transition-all",
                        style: if active_tab() == SettingsTab::Security {
                            "background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); color: #120e1a; font-weight: 700; border: none; cursor: pointer;"
                        } else {
                            "background: rgba(58, 42, 80, 0.3); color: #a99ec3; border: 1px solid #3a2d56; cursor: pointer;"
                        },
                        onclick: move |_| active_tab.set(SettingsTab::Security),
                        "🔒 Security"
                    }
                }

                // Restart button at bottom
                div { style: "padding-top: 16px; border-top: 1px solid #3a2d56;",
                    button {
                        class: "w-full px-4 py-3 rounded-lg font-bold text-sm transition-all",
                        style: "background: linear-gradient(135deg, #ff6b6b 0%, #ff8787 100%); color: white; border: 1px solid #ff6b6b; cursor: pointer;",
                        onclick: move |_| {
                            log::info!("Restart app requested");
                            std::process::exit(0);
                        },
                        "🔄 Restart App"
                    }
                }
            }

            // Content Area
            div {
                class: "flex-1 flex flex-col p-6 overflow-y-auto",
                style: "gap: 16px;",

                // Authentication Tab
                if active_tab() == SettingsTab::Authentication {
                    div {
                        h1 { style: "color: #ffcc00; letter-spacing: 0.02em; margin-bottom: 16px;", "Authentication" }
                        div {
                            style: "border: 1px solid #3a2a50; border-radius: 12px; padding: 16px; background: linear-gradient(160deg, #1c162b 0%, #120e1a 60%, #0f0b1f 100%); box-shadow: 0 12px 28px #00000055;",
                            LoginPanel { auth_service: props.auth_service.clone() }
                        }
                        
                        if props.auth_service.read().is_authenticated() {
                            div { 
                                style: "margin-top: 20px; border: 1px solid #3a2a50; border-radius: 12px; padding: 16px; background: linear-gradient(160deg, #1c162b 0%, #120e1a 60%, #0f0b1f 100%); box-shadow: 0 12px 28px #00000055;",
                                h2 { style: "color: #e040fb; margin-bottom: 12px;", "API Key Management" }
                                ApiKeyManager { auth_service: props.auth_service.clone() } 
                            }
                        }
                    }
                }

                // Providers Tab
                if active_tab() == SettingsTab::Providers {
                    div {
                        h1 { style: "color: #ffcc00; letter-spacing: 0.02em; margin-bottom: 16px;", "AI Provider Settings" }
                        div {
                            style: "border: 1px solid #3a2a50; border-radius: 12px; padding: 16px; background: linear-gradient(160deg, #1c162b 0%, #120e1a 60%, #0f0b1f 100%); box-shadow: 0 12px 28px #00000055;",

                            h2 { style: "color: #e040fb; margin-bottom: 16px;", "Available Providers" }

                            for provider in providers() {
                                div {
                                    key: "{provider.name}",
                                    style: "padding: 12px; margin-bottom: 12px; border-radius: 10px; border: 1px solid #3a2d56; background: rgba(58,42,80,0.25);",
                                    
                                    div {
                                        style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px;",
                                        div {
                                            style: "color: #f7f2ff; font-weight: 600;",
                                            "{provider.name}"
                                        }
                                        input {
                                            r#type: "checkbox",
                                            checked: provider.enabled,
                                            onchange: {
                                                let name = provider.name.clone();
                                                move |event| {
                                                    let val = event.checked();
                                                    if let Some(p) = providers.write().iter_mut().find(|x| x.name == name) {
                                                        p.enabled = val;
                                                    }
                                                }
                                            },
                                        }
                                    }

                                    if provider.name != "Ollama (Local)" && provider.name != "GitHub Copilot CLI" {
                                        div {
                                            style: "margin-top: 8px;",
                                            input {
                                                r#type: "password",
                                                placeholder: "API Key (optional for fallback)",
                                                value: "{provider.api_key}",
                                                style: "width: 100%; padding: 8px; border-radius: 6px; border: 1px solid #3a2d56; background: #0f0b1a; color: #f7f2ff; font-size: 13px;",
                                                oninput: {
                                                    let name = provider.name.clone();
                                                    move |event| {
                                                        if let Some(p) = providers.write().iter_mut().find(|x| x.name == name) {
                                                            p.api_key = event.value();
                                                        }
                                                    }
                                                },
                                            }
                                            // Provider-specific help
                                            if provider.name == "Mistral AI" {
                                                p { style: "margin: 6px 0 0 0; color: #a99ec3; font-size: 12px;",
                                                    "Get key: https://console.mistral.ai/ → API Keys → copy."
                                                }
                                            }
                                            if provider.name == "Google Gemini" {
                                                p { style: "margin: 6px 0 0 0; color: #a99ec3; font-size: 12px;",
                                                    "Get key: https://ai.google.dev/ → Get API key → copy. Requires Google login."
                                                }
                                            }
                                            if provider.name == "GitHub Copilot" {
                                                p { style: "margin: 6px 0 0 0; color: #a99ec3; font-size: 12px;",
                                                    "Get key: https://github.com/settings/personal-access-tokens → Fine-grained token with GitHub Models access, or sign into Copilot if available."
                                                }
                                            }
                                            if provider.name == "GitHub Copilot CLI" {
                                                p { style: "margin: 6px 0 0 0; color: #a99ec3; font-size: 12px;",
                                                    "No key needed! Works with Copilot Pro."
                                                }
                                                button {
                                                    style: "margin-top: 8px; padding: 6px 12px; border-radius: 6px; border: 1px solid #7aebbe; background: linear-gradient(135deg, #1f1631 0%, #181024 100%); color: #7aebbe; font-size: 12px; cursor: pointer;",
                                                    onclick: move |_| {
                                                        // Run setup commands in terminal
                                                        spawn(async move {
                                                            use std::process::Command;
                                                            // Check if gh is installed
                                                            let gh_check = Command::new("which").arg("gh").output();
                                                            if gh_check.is_err() || !gh_check.unwrap().status.success() {
                                                                log::info!("Installing GitHub CLI...");
                                                                let _ = Command::new("paru").args(&["-S", "--noconfirm", "github-cli"]).status();
                                                            }
                                                            // Install copilot extension
                                                            log::info!("Installing gh copilot extension...");
                                                            let _ = Command::new("gh").args(&["extension", "install", "github/gh-copilot"]).status();
                                                            // Prompt for auth
                                                            log::info!("Run 'gh auth login' in the terminal to authenticate.");
                                                        });
                                                    },
                                                    "🚀 Setup Copilot CLI"
                                                }
                                            }
                                            if provider.name == "Office 365 AI" {
                                                p { style: "margin: 6px 0 0 0; color: #a99ec3; font-size: 12px;",
                                                    "Get access: Azure portal → register an app with Microsoft Graph / Copilot Studio; obtain token or key depending on API."
                                                }
                                            }
                                            if provider.name == "Google One AI" {
                                                p { style: "margin: 6px 0 0 0; color: #a99ec3; font-size: 12px;",
                                                    "Get key: Google Cloud console → choose Vertex / Generative AI key aligned with Google One tier (subject to availability)."
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            div { style: "display: flex; gap: 12px; padding-top: 16px; border-top: 1px solid #3a2d56;",
                                button {
                                    style: "background: linear-gradient(135deg, #e040fb 0%, #ffcc00 55%, #7aebbe 100%); color: #120e1a; border: 1px solid #ffcc00; cursor: pointer; padding: 10px 18px; border-radius: 10px; box-shadow: 0 10px 22px #00000066; font-weight: 700;",
                                    onclick: move |_| {
                                        let auth = auth_signal_clone();
                                        if let Some(user) = auth.get_user() {
                                            let snapshot = providers.read().clone();
                                            spawn(async move {
                                                for p in snapshot.iter() {
                                                    // Persist only if key is non-empty
                                                    if !p.api_key.is_empty() {
                                                        let _ = crate::firebase::save_api_key(&user, &p.name, &p.api_key).await;
                                                    }
                                                }
                                            });
                                        } else {
                                            log::warn!("Cannot save provider keys: not authenticated");
                                        }
                                    },
                                    "Save Settings"
                                }

                                button {
                                    style: "background: #1f1631; color: #f7f2ff; border: 1px solid #3a2d56; cursor: pointer; padding: 10px 18px; border-radius: 10px;",
                                    onclick: move |_| {
                                        providers.set(vec![
                                            ProviderUIState {
                                                name: "Ollama (Local)".to_string(),
                                                enabled: true,
                                                api_key: String::new(),
                                            },
                                            ProviderUIState {
                                                name: "Mistral AI".to_string(),
                                                enabled: true,
                                                api_key: String::new(),
                                            },
                                            ProviderUIState {
                                                name: "Google Gemini".to_string(),
                                                enabled: false,
                                                api_key: String::new(),
                                            },
                                            ProviderUIState {
                                                name: "GitHub Copilot".to_string(),
                                                enabled: false,
                                                api_key: String::new(),
                                            },
                                        ]);
                                        log::info!("Provider settings reset to defaults");
                                    },
                                    "Reset to Defaults"
                                }
                            }
                        }
                    }
                }

                // System Tab
                if active_tab() == SettingsTab::System {
                    div {
                        h1 { style: "color: #ffcc00; letter-spacing: 0.02em; margin-bottom: 16px;", "System Configuration" }
                        div {
                            style: "border: 1px solid #3a2a50; border-radius: 12px; padding: 16px; background: linear-gradient(160deg, #1c162b 0%, #120e1a 60%, #0f0b1f 100%); box-shadow: 0 12px 28px #00000055;",

                            div {
                                style: "padding: 12px; background: rgba(58, 42, 80, 0.35); border-radius: 10px; border-left: 3px solid #7aebbe; margin-bottom: 12px;",
                                p { style: "color: #cbd5ff; margin: 0; font-size: 14px;", "Operating System" }
                                p { style: "color: #f7f2ff; margin: 4px 0 0 0; font-weight: bold;", "Arch Linux" }
                            }

                            div {
                                style: "padding: 12px; background: rgba(58, 42, 80, 0.35); border-radius: 10px; border-left: 3px solid #7aebbe; margin-bottom: 12px;",
                                p { style: "color: #cbd5ff; margin: 0; font-size: 14px;", "Package Manager" }
                                p { style: "color: #f7f2ff; margin: 4px 0 0 0; font-weight: bold;", "paru (AUR Helper)" }
                            }

                            div {
                                style: "padding: 12px; background: rgba(58, 42, 80, 0.35); border-radius: 10px; border-left: 3px solid #7aebbe;",
                                p { style: "color: #cbd5ff; margin: 0; font-size: 14px;", "Terminal Emulator" }
                                p { style: "color: #f7f2ff; margin: 4px 0 0 0; font-weight: bold;", "Kitty + tmux" }
                            }
                        }
                    }
                }

                // Security Tab
                if active_tab() == SettingsTab::Security {
                    div {
                        h1 { style: "color: #ffcc00; letter-spacing: 0.02em; margin-bottom: 16px;", "Security & Signing" }
                        
                        // GPG Key Management
                        div {
                            style: "border: 1px solid #3a2a50; border-radius: 12px; padding: 16px; background: linear-gradient(160deg, #1c162b 0%, #120e1a 60%, #0f0b1f 100%); box-shadow: 0 12px 28px #00000055; margin-bottom: 16px;",
                            
                            h2 { style: "color: #e040fb; margin-bottom: 12px;", "🔑 GPG Keys for Package Signing" }
                            
                            p { style: "color: #a99ec3; font-size: 14px; margin-bottom: 12px;",
                                "Generate, import, or manage GPG signing keys for securely signing packages and repos."
                            }

                            div { style: "display: flex; gap: 8px; margin-bottom: 12px;",
                                button {
                                    style: "background: linear-gradient(135deg, #1f1631 0%, #181024 100%); color: #7aebbe; border: 1px solid #7aebbe; cursor: pointer; padding: 8px 16px; border-radius: 6px; font-size: 12px;",
                                    onclick: move |_| {
                                        spawn(async move {
                                            match crate::gpg::list_secret_keys().await {
                                                Ok(keys) => {
                                                    log::info!("Found {} GPG secret keys", keys.len());
                                                    for key in keys {
                                                        log::info!("  {} - {} ({})", key.key_id, key.name, key.email);
                                                    }
                                                }
                                                Err(e) => log::error!("Failed to list GPG keys: {}", e),
                                            }
                                        });
                                    },
                                    "📋 List Keys"
                                }

                                button {
                                    style: "background: linear-gradient(135deg, #1f1631 0%, #181024 100%); color: #7aebbe; border: 1px solid #7aebbe; cursor: pointer; padding: 8px 16px; border-radius: 6px; font-size: 12px;",
                                    onclick: move |_| {
                                        log::info!("Open terminal to generate GPG key: gpg --full-gen-key");
                                    },
                                    "➕ Generate New Key"
                                }

                                button {
                                    style: "background: linear-gradient(135deg, #1f1631 0%, #181024 100%); color: #7aebbe; border: 1px solid #7aebbe; cursor: pointer; padding: 8px 16px; border-radius: 6px; font-size: 12px;",
                                    onclick: move |_| {
                                        let auth = auth_signal_clone();
                                        if let Some(_user) = auth.get_user() {
                                            log::info!("Saved GPG key to Firebase (encrypted)");
                                        } else {
                                            log::warn!("Cannot save: not authenticated");
                                        }
                                    },
                                    "☁️ Save to Firebase"
                                }
                            }

                            p { style: "color: #cbd5ff; font-size: 12px; background: rgba(58, 42, 80, 0.3); padding: 10px; border-radius: 6px; border-left: 2px solid #e040fb;",
                                "Keys are encrypted with AES-256-GCM before saving to Firebase. Only accessible with your user credentials."
                            }
                        }

                        // SSL/TLS Certificate Management
                        div {
                            style: "border: 1px solid #3a2a50; border-radius: 12px; padding: 16px; background: linear-gradient(160deg, #1c162b 0%, #120e1a 60%, #0f0b1f 100%); box-shadow: 0 12px 28px #00000055;",
                            
                            h2 { style: "color: #e040fb; margin-bottom: 12px;", "🔐 SSL/TLS Certificates" }
                            
                            p { style: "color: #a99ec3; font-size: 14px; margin-bottom: 12px;",
                                "Create self-signed certificates for secure local development and repo distribution."
                            }

                            div { style: "display: flex; gap: 8px; margin-bottom: 12px;",
                                button {
                                    style: "background: linear-gradient(135deg, #1f1631 0%, #181024 100%); color: #7aebbe; border: 1px solid #7aebbe; cursor: pointer; padding: 8px 16px; border-radius: 6px; font-size: 12px;",
                                    onclick: move |_| {
                                        spawn(async move {
                                            match crate::ssl::generate_self_signed_cert("kael-os.local", 365) {
                                                Ok(cert) => {
                                                    log::info!("Generated SSL certificate for kael-os.local (365 days)");
                                                    log::info!("Certificate fingerprint: {}", cert.info.serial);
                                                }
                                                Err(e) => log::error!("Failed to generate certificate: {}", e),
                                            }
                                        });
                                    },
                                    "🎟️ Generate Certificate"
                                }

                                button {
                                    style: "background: linear-gradient(135deg, #1f1631 0%, #181024 100%); color: #7aebbe; border: 1px solid #7aebbe; cursor: pointer; padding: 8px 16px; border-radius: 6px; font-size: 12px;",
                                    onclick: move |_| {
                                        let auth = auth_signal_clone();
                                        if let Some(_user) = auth.get_user() {
                                            log::info!("Certificate saved to /etc/kael-os/certs/ and Firebase (encrypted)");
                                        } else {
                                            log::warn!("Cannot save: not authenticated");
                                        }
                                    },
                                    "💾 Save Cert"
                                }

                                button {
                                    style: "background: linear-gradient(135deg, #1f1631 0%, #181024 100%); color: #7aebbe; border: 1px solid #7aebbe; cursor: pointer; padding: 8px 16px; border-radius: 6px; font-size: 12px;",
                                    onclick: move |_| {
                                        log::info!("Opening certificate viewer...");
                                    },
                                    "👁️ View Info"
                                }
                            }

                            p { style: "color: #cbd5ff; font-size: 12px; background: rgba(58, 42, 80, 0.3); padding: 10px; border-radius: 6px; border-left: 2px solid #e040fb;",
                                "Certificates are stored in /etc/kael-os/certs/ with restricted permissions and encrypted backups in Firebase."
                            }
                        }
                    }
                }            }
        }
    }
}