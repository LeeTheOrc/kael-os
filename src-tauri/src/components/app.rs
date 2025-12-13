// src-tauri/src/components/app.rs
use dioxus::prelude::*;

use crate::auth::AuthService;
use crate::components::chat::ChatPanel;
use crate::components::header::Header;
use crate::components::icons::{KaelSigilIcon, PanelIcon, SparkIcon};
use crate::components::settings::SettingsPanel;
use crate::components::terminal::TerminalPanel;

// Strip ANSI escape sequences from text (robustly skips ESC sequences)
fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_escape = false;
    for ch in text.chars() {
        if !in_escape {
            if ch == '\x1b' {
                in_escape = true;
            } else {
                out.push(ch);
            }
        } else {
            // Inside an escape: skip until we hit a final byte in the range '@'..='~'
            // This covers CSI (ESC [ ... final) and other ANSI sequences
            if ('@'..='~').contains(&ch) {
                in_escape = false;
            }
        }
    }
    out
}

#[allow(non_snake_case)]
pub fn App() -> Element {
    let quick_actions = vec![
        (
            "New Script",
            "Launch a fresh Kael script pad",
            "linear-gradient(135deg, #e040fb 0%, #ffcc00 60%, #7aebbe 100%)",
        ),
        (
            "Sync Repo",
            "Pull latest mission changes",
            "linear-gradient(135deg, #7aebbe 0%, #5af0c8 60%, #ffcc00 100%)",
        ),
        (
            "Deploy",
            "Ship build to forge target",
            "linear-gradient(135deg, #ffcc00 0%, #ff9f0a 60%, #e040fb 100%)",
        ),
    ];

    let pinned_panels = vec![
        ("Terminal", "Active", "#7aebbe"),
        ("Firebase", "Linked", "#ffcc00"),
        ("Local DB", "Online", "#e040fb"),
    ];

    let mut terminal_output = use_signal(String::new);
    let mut pty_ready = use_signal(|| false);
    let current_command = use_signal(String::new);
    let auth_service = use_signal(|| AuthService::new());
    let show_settings = use_signal(|| false);
    let pty_instance = use_signal(|| {
        use crate::terminal::PtyTerminal;
        PtyTerminal::new()
    });

    // Spawn PTY terminal session on mount
    use_effect(move || {
        spawn(async move {
            let pty = pty_instance();
            if let Err(e) = pty.ensure_session().await {
                eprintln!("PTY init failed: {e}");
                return;
            }
            pty_ready.set(true);

            // Stream output
            if let Ok(rx) = pty.get_output_receiver().await {
                spawn(async move {
                    while let Ok(chunk) = rx.recv().await {
                        let text = String::from_utf8_lossy(&chunk).to_string();
                        let clean_text = strip_ansi(&text);
                        terminal_output.write().push_str(&clean_text);
                    }
                });
            }
        });
    });

    rsx! {
        style { {include_str!("../../assets/tailwind.out.css")} }
        style { "
            * {{ margin: 0; padding: 0; box-sizing: border-box; }}
            .app-root {{
                height: 100vh;
                display: flex;
                flex-direction: column;
                background: radial-gradient(1200px at 20% 0%, #1c162b 0%, #120e1a 45%, #0b0816 100%);
                color: #f7f2ff;
                font-family: 'Inter', system-ui, -apple-system, sans-serif;
            }}
            .main-container {{
                display: flex;
                flex: 1;
                overflow: hidden;
                position: relative;
            }}
            .resizable-left {{
                flex: 0 0 256px;
                background: linear-gradient(180deg, #1c162b 0%, #140f22 55%, #0f0b1a 100%);
                border-right: 1px solid #3a2d56;
                overflow-y: auto;
                padding: 1rem;
                min-width: 150px;
                max-width: 70%;
            }}
            .left-card {{
                border: 1px solid #3a2d56;
                border-radius: 12px;
                background: linear-gradient(160deg, #1a1326 0%, #100b1c 65%, #0b0816 100%);
                box-shadow: 0 14px 28px #00000055, inset 0 1px 0 #2a1e40;
            }}
            .section-label {{
                color: #a99ec3;
                letter-spacing: 0.08em;
                font-size: 12px;
                text-transform: uppercase;
            }}
            .chip {{
                display: inline-flex;
                align-items: center;
                gap: 6px;
                padding: 8px 10px;
                border-radius: 10px;
                background: rgba(58,45,86,0.35);
                border: 1px solid #3a2d56;
                color: #f7f2ff;
                font-size: 12px;
                box-shadow: inset 0 1px 0 #2a1e40;
            }}
            .splitter {{
                flex: 0 0 6px;
                background-color: #3a2d56;
                cursor: col-resize;
                user-select: none;
                transition: background-color 0.2s;
            }}
            .splitter:hover {{
                background-color: #ffcc00;
            }}
            .splitter.dragging {{
                background-color: #ffcc00;
            }}
            .chat-container {{
                flex: 1;
                min-width: 200px;
                overflow: hidden;
            }}
            .resizable-right {{
                flex: 0 0 320px;
                background: linear-gradient(180deg, #1c162b 0%, #140f22 55%, #0f0b1a 100%);
                border-left: 1px solid #3a2d56;
                overflow-y: auto;
                padding: 1rem;
                min-width: 150px;
                max-width: 70%;
            }}
            .status-card {{
                border: 1px solid #3a2d56;
                border-radius: 14px;
                padding: 14px;
                background: linear-gradient(150deg, #1a1426 0%, #110d1d 60%, #0d0a16 100%);
                box-shadow: 0 12px 26px #00000055, inset 0 1px 0 #2a1e40;
            }}
            .pane-scroll::-webkit-scrollbar {{ width: 8px; }}
            .pane-scroll::-webkit-scrollbar-track {{ background: #0f0b1a; border-radius: 10px; }}
            .pane-scroll::-webkit-scrollbar-thumb {{ background: linear-gradient(180deg, #3a2d56 0%, #2a1e40 100%); border-radius: 10px; border: 1px solid #120e1a; }}
            .pane-scroll::-webkit-scrollbar-thumb:hover {{ background: linear-gradient(180deg, #ffcc00 0%, #e040fb 100%); }}
        " }
        div {
            class: "app-root",
            Header { show_settings: show_settings.clone() },

            if show_settings() {
                SettingsPanel { auth_service: auth_service.clone(), show_settings: show_settings.clone() }
            } else {
                div {
                    class: "main-container",
                    // Left Panel
                    aside {
                    class: "resizable-left pane-scroll",
                    div { class: "flex items-center gap-3 mb-4",
                        div { class: "p-2 rounded-lg border", style: "border-color: #3a2d56; background: radial-gradient(circle at 30% 30%, #e040fb 0%, #120e1a 55%, #0f0c1a 100%); box-shadow: inset 2px 0 0 #ffcc00;",
                            KaelSigilIcon { class: "w-5 h-5" }
                        }
                        h2 { class: "font-bold text-lg", style: "color: #ffcc00; letter-spacing: 0.02em;", "Project Explorer" }
                    }

                    div { class: "left-card p-3 mb-4",
                        div { class: "flex items-center justify-between mb-3",
                            span { class: "section-label", "Quick Actions" }
                            SparkIcon { class: "w-4 h-4 text-[#ffcc00]" }
                        }
                        for (title, desc, gradient) in quick_actions.iter() {
                            button {
                                class: "w-full text-left mb-2 last:mb-0",
                                style: "padding: 10px 12px; border-radius: 10px; border: 1px solid #3a2d56; color: #120e1a; background: {gradient}; box-shadow: 0 12px 24px #00000066; font-weight: 700;",
                                div { style: "font-size: 14px;", "{title}" }
                                div { style: "font-size: 12px; color: #0f0b1a; opacity: 0.8;", "{desc}" }
                            }
                        }
                    }

                    div { class: "left-card p-3 mb-4",
                        div { class: "flex items-center justify-between mb-3",
                            span { class: "section-label", "Terminal Status" }
                            if current_command() != "" { PanelIcon { class: "w-4 h-4 text-[#7aebbe]" } } else { PanelIcon { class: "w-4 h-4 text-[#3a2d56]" } }
                        }
                        if current_command() != "" {
                            div { style: "padding: 10px; background: linear-gradient(135deg, #1f1631 0%, #181024 100%); border: 1px solid #3a2d56; border-radius: 8px; color: #7aebbe; font-size: 12px; font-family: monospace; overflow-x: auto; word-break: break-all;",
                                "$ {current_command()}"
                            }
                        } else {
                            span { style: "color: #a99ec3; font-size: 13px;", "No active command" }
                        }
                    }

                    div { class: "left-card p-3",
                        div { class: "flex items-center justify-between mb-3",
                            span { class: "section-label", "Pinned Panels" }
                            PanelIcon { class: "w-4 h-4 text-[#7aebbe]" }
                        }
                        for (panel, state, color) in pinned_panels.iter() {
                            div { class: "flex items-center justify-between mb-2 last:mb-0",
                                div { class: "flex items-center gap-2",
                                    div { class: "chip", style: "border-color: #3a2d56; color: {color};", KaelSigilIcon { class: "w-4 h-4" } }
                                    span { style: "color: #f7f2ff;", "{panel}" }
                                }
                                span { style: "color: {color}; font-size: 12px;", "{state}" }
                            }
                        }
                    }

                    div { class: "left-card p-3",
                        div { class: "flex items-center justify-between mb-3",
                            span { class: "section-label", "Chat History" }
                            PanelIcon { class: "w-4 h-4 text-[#ffcc00]" }
                        }
                        button {
                            class: "w-full mb-2",
                            style: "padding: 8px 12px; border-radius: 8px; border: 1px solid #3a2d56; background: linear-gradient(135deg, #7aebbe 0%, #5af0c8 100%); color: #120e1a; font-weight: 600; font-size: 13px;",
                            onclick: move |_| {
                                use chrono::Local;
                                let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
                                let filename = format!("kael_chat_{}.txt", timestamp);
                                
                                if let Ok(json_content) = std::fs::read_to_string("/tmp/kael_chat_history.json") {
                                    if let Ok(messages) = serde_json::from_str::<Vec<serde_json::Value>>(&json_content) {
                                        let mut text_content = format!("Kael Chat Export - {}\n", Local::now().format("%Y-%m-%d %H:%M:%S"));
                                        text_content.push_str(&"=".repeat(60));
                                        text_content.push_str("\n\n");
                                        
                                        for msg in messages {
                                            if let (Some(author), Some(text)) = (msg.get("author").and_then(|a| a.as_str()), msg.get("text").and_then(|t| t.as_str())) {
                                                text_content.push_str(&format!("[{}]\n{}\n\n", author, text));
                                            }
                                        }
                                        
                                        let save_path = format!("/tmp/{}", filename);
                                        match std::fs::write(&save_path, text_content) {
                                            Ok(_) => log::info!("Chat saved to: {}", save_path),
                                            Err(e) => log::error!("Failed to save chat: {}", e),
                                        }
                                    }
                                }
                            },
                            "💾 Save Chat"
                        }
                        button {
                            class: "w-full",
                            style: "padding: 8px 12px; border-radius: 8px; border: 1px solid #3a2d56; background: linear-gradient(135deg, #ff6b6b 0%, #ff8787 100%); color: white; font-weight: 600; font-size: 13px;",
                            onclick: move |_| {
                                match std::fs::remove_file("/tmp/kael_chat_history.json") {
                                    Ok(_) => {
                                        log::info!("Chat history cleared");
                                        // Optionally reload the page or reset messages
                                    },
                                    Err(e) => log::error!("Failed to clear chat: {}", e),
                                }
                            },
                            "🗑️ Delete Chat"
                        }
                    }
                }
                // Left Splitter
                div {
                    class: "splitter",
                },
                // Center Chat + Terminal
                div {
                    class: "chat-container",
                    style: "display: flex; flex-direction: column; flex: 1; gap: 16px;",
                    {
                        let user_photo_url = auth_service.read().get_user().and_then(|u| u.photo_url);
                        let user_name = auth_service.read().get_user().map(|u| u.name).unwrap_or_else(|| "Architect".to_string());
                        rsx! {
                            ChatPanel { 
                                term_out: terminal_output.clone(), 
                                pty: pty_instance.clone(), 
                                current_cmd: current_command.clone(),
                                user_photo_url: user_photo_url,
                                user_name: user_name,
                                auth_service: auth_service.clone(),
                            }
                        }
                    }
                    TerminalPanel { term_out: terminal_output.clone() }
                },
                // Right Splitter
                div {
                    class: "splitter",
                },
                // Right Panel
                aside {
                    class: "resizable-right pane-scroll",
                    h2 { class: "font-bold text-lg mb-4", style: "color: #7aebbe; letter-spacing: 0.02em;", "SYSTEM BLUEPRINT" }

                    div { class: "status-card mb-3",
                        div { class: "flex items-center gap-2 mb-2", KaelSigilIcon { class: "w-4 h-4 text-[#ffcc00]" }, span { style: "color: #f7f2ff; font-weight: 700;", "AI Providers" } }
                        p { style: "margin: 0; color: #cbd5ff; font-size: 13px;", "Ollama + Mistral active. Gemini staged." }
                        div { style: "margin-top: 8px; display: flex; gap: 8px; flex-wrap: wrap;",
                            span { class: "chip", style: "color: #ffcc00;", "Primary" }
                            span { class: "chip", style: "color: #e040fb;", "Fallback" }
                            span { class: "chip", style: "color: #7aebbe;", "Local" }
                        }
                    }

                    div { class: "status-card mb-3",
                        div { class: "flex items-center gap-2 mb-2", SparkIcon { class: "w-4 h-4 text-[#e040fb]" }, span { style: "color: #f7f2ff; font-weight: 700;", "Build Status" } }
                        p { style: "margin: 0; color: #cbd5ff; font-size: 13px;", "Dev profile ready. Latest shell synced." }
                        div { style: "margin-top: 8px; display: flex; gap: 8px; flex-wrap: wrap;",
                            span { class: "chip", style: "color: #7aebbe;", "Rust" }
                            span { class: "chip", style: "color: #ffcc00;", "Tauri" }
                            span { class: "chip", style: "color: #e040fb;", "Forge" }
                        }
                    }

                    div { class: "status-card",
                        div { class: "flex items-center gap-2 mb-2", PanelIcon { class: "w-4 h-4 text-[#7aebbe]" }, span { style: "color: #f7f2ff; font-weight: 700;", "Runtime" } }
                        p { style: "margin: 0; color: #cbd5ff; font-size: 13px;", "Arch + paru translator wired." }
                        div { style: "margin-top: 8px; display: flex; gap: 8px; flex-wrap: wrap;",
                            span { class: "chip", style: "color: #7aebbe;", "Arch" }
                            span { class: "chip", style: "color: #ffcc00;", "Paru" }
                            span { class: "chip", style: "color: #e040fb;", "LLM Ready" }
                        }
                    }
                }
                }
            }
        }
    }
}
