use crate::components::icons::{PanelIcon, SendIcon, SparkIcon};
#[allow(unused_imports)]
use crate::llm::{self, LLMProvider, LLMRequest};
use crate::terminal::PtyTerminal;
use dioxus::events::Key;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// Detect if query is system-related and should use local Ollama
fn is_system_query(s: &str) -> bool {
    let s_lower = s.to_lowercase();
    let system_keywords = [
        // Package management
        "pacman",
        "paru",
        "yay",
        "aur",
        "package",
        "install",
        "update",
        "upgrade",
        // System administration
        "systemd",
        "systemctl",
        "journalctl",
        "service",
        "daemon",
        "boot",
        // KDE/Plasma
        "kde",
        "plasma",
        "kwin",
        "krunner",
        "konsole",
        "dolphin",
        "kconfig",
        // Desktop/Display
        "wayland",
        "x11",
        "xorg",
        "display",
        "screen",
        "monitor",
        "compositor",
        // File system
        "filesystem",
        "partition",
        "mount",
        "fstab",
        "disk",
        "df",
        "du",
        // System info
        "uname",
        "arch",
        "kernel",
        "cpu",
        "memory",
        "ram",
        "hardware",
        // Terminal/Shell
        "terminal",
        "shell",
        "bash",
        "zsh",
        "fish",
        "pty",
        "tty",
        // System commands
        "chmod",
        "chown",
        "sudo",
        "permissions",
        "grep",
        "sed",
        "awk",
        // How are you / system status
        "how are you",
        "status",
        "health",
        "working",
    ];

    system_keywords
        .iter()
        .any(|keyword| s_lower.contains(keyword))
}

// Simple classifier: treat as command if it looks like a shell command
fn is_command(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    if s.starts_with("!") {
        return true;
    }
    let verbs = [
        "cd",
        "ls",
        "pwd",
        "cat",
        "echo",
        "touch",
        "rm",
        "mv",
        "cp",
        "mkdir",
        "rmdir",
        "git",
        "cargo",
        "python",
        "pip",
        "rustc",
        "curl",
        "wget",
        "tar",
        "zip",
        "unzip",
        "grep",
        "sed",
        "awk",
        "find",
        "ps",
        "top",
        "kill",
        "chmod",
        "chown",
        "sudo",
        "pacman",
        "apt",
        "apt-get",
        "yum",
        "brew",
        "dnf",
        "zypper",
        "npm",
        "yarn",
        "pnpm",
        "node",
        "docker",
        "docker-compose",
        "systemctl",
        "journalctl",
        "service",
        "which",
        "whereis",
        "file",
        "lsof",
        "make",
        "ninja",
        "cmake",
        "gcc",
        "clang",
        "go",
        "ruby",
        "php",
        "test",
        "[ ",
        "head",
        "tail",
        "wc",
        "sort",
        "uniq",
        "cut",
        "paste",
        "tr",
    ];
    let first = s.split_whitespace().next().unwrap_or("");
    verbs.contains(&first)
        || s.contains('|')
        || s.contains('>')
        || s.contains("&&")
        || s.contains(";")
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Message {
    author: String,
    text: String,
    is_streaming: bool,
}

#[derive(Props, Clone, PartialEq)]
pub struct ChatProps {
    pub term_out: Signal<String>,
    pub pty: Signal<PtyTerminal>,
    pub current_cmd: Signal<String>,
    pub user_photo_url: Option<String>,
    pub user_name: String,
    pub auth_service: Signal<crate::auth::AuthService>,
}

#[allow(non_snake_case)]
pub fn ChatPanel(mut props: ChatProps) -> Element {
    let load_messages = || -> Vec<Message> {
        if let Ok(json) = std::fs::read_to_string("/tmp/kael_chat_history.json") {
            serde_json::from_str(&json).unwrap_or_else(|_| {
                vec![Message {
                    author: "Kael".to_string(),
                    text: "Greetings, Architect! I am Kael, your partner in creation.".to_string(),
                    is_streaming: false,
                }]
            })
        } else {
            vec![Message {
                author: "Kael".to_string(),
                text: "Greetings, Architect! I am Kael, your partner in creation.".to_string(),
                is_streaming: false,
            }]
        }
    };

    let save_messages = |messages: &[Message]| {
        if let Ok(json) = serde_json::to_string(messages) {
            let _ = std::fs::write("/tmp/kael_chat_history.json", json);
        }
    };

    let mut messages = use_signal(load_messages);
    let mut user_input = use_signal(String::new);
    let mut echo_commands = use_signal(|| false);
    let mut sudo_pending = use_signal(|| Option::<String>::None);
    let mut sudo_pw = use_signal(String::new);
    let pty = props.pty;

    // Warm up local AI and warn if unavailable at startup
    use_effect(move || {
        let mut msgs = messages.clone();
        spawn(async move {
            let local_ok = llm::ping_local().await;
            if !local_ok {
                let mut current = msgs.write();
                let already_noted = current
                    .iter()
                    .any(|m| m.author == "Kael" && m.text.contains("Local AI is not responding"));
                if !already_noted {
                    current.push(Message {
                        author: "Kael".to_string(),
                        text: "Heads up: Local AI service is not responding. I'll auto-try cloud providers if you enable API keys in Settings → Providers.".to_string(),
                        is_streaming: false,
                    });
                    save_messages(&current);
                }
            } else {
                // Send a tiny warm-up prompt so first real reply is fast
                let warmed = llm::warm_local_model("mistral").await;
                if !warmed {
                    let mut current = msgs.write();
                    current.push(Message {
                        author: "Kael".to_string(),
                        text: "Local AI responded to ping but warm-up prompt failed. I will still try cloud fallbacks if needed.".to_string(),
                        is_streaming: false,
                    });
                    save_messages(&current);
                }
            }
        });
    });

    // When user submits input, classify and dispatch
    let _on_submit = move || {
        let input = user_input().clone();
        if input.is_empty() {
            return;
        }

        // Add user message to chat
        messages.write().push(Message {
            author: "Architect".to_string(),
            text: input.clone(),
            is_streaming: false,
        });
        save_messages(&messages.read());

        if is_command(&input) {
            // It's a command, send to terminal
            let pty_write = pty.read().clone();
            let input_clone = input.clone();

            spawn(async move {
                let _ = pty_write.write_line(&input_clone).await;
            });
        } else {
            // It's a prompt, send to LLM with fallback
            let mut msgs = messages.clone();
            let input_clone = input.clone();
            spawn(async move {
                let user_opt = props.auth_service.read().get_user();
                // Auto-select provider based on query type
                let primary_provider = if is_system_query(&input_clone) {
                    LLMProvider::Ollama // Use local Ollama for system queries
                } else {
                    LLMProvider::Ollama // Default to Ollama (can be changed based on settings)
                };

                // Build fallback chain - try other providers if primary fails
                let fallback_providers = vec![
                    (LLMProvider::CopilotCLI, None), // Try CLI first (no key needed)
                    (LLMProvider::Gemini, None),     // Google Cloud AI
                    (LLMProvider::Mistral, None),    // Mistral AI
                    (LLMProvider::Copilot, None),    // GitHub Models API (needs org)
                    (LLMProvider::Office365AI, None),
                    (LLMProvider::GoogleOneAI, None),
                ];

                let req = LLMRequest {
                    provider: primary_provider,
                    model: String::new(), // resolved per provider in fallback helper
                    prompt: input_clone,
                    api_key: None,
                    system: Some(llm::get_kael_system_prompt()),
                };

                let user_ref = user_opt.as_ref();
                match llm::send_request_with_fallback(req, user_ref, fallback_providers).await {
                    Ok(res) => {
                        msgs.write().push(Message {
                            author: "Kael".to_string(),
                            text: res.content,
                            is_streaming: false,
                        });
                        save_messages(&msgs.read());
                    }
                    Err(e) => {
                        msgs.write().push(Message {
                            author: "Kael".to_string(),
                            text: format!("❌ All AI providers failed:\n\n{}\n\n💡 Tip: Enable cloud providers and add API keys in Settings → Providers tab.", e),
                            is_streaming: false,
                        });
                        save_messages(&msgs.read());
                    }
                }
            });
        }
        user_input.set(String::new());
    };

    // Simple classifier: treat as command if it looks like a shell command
    let is_command = |s: &str| {
        let s = s.trim();
        if s.is_empty() {
            return false;
        }
        if s.starts_with("!") {
            return true;
        }
        let verbs = [
            "cd",
            "ls",
            "pwd",
            "cat",
            "echo",
            "touch",
            "rm",
            "mv",
            "cp",
            "mkdir",
            "rmdir",
            "git",
            "cargo",
            "python",
            "pip",
            "rustc",
            "curl",
            "wget",
            "tar",
            "zip",
            "unzip",
            "grep",
            "sed",
            "awk",
            "find",
            "ps",
            "top",
            "kill",
            "chmod",
            "chown",
            "sudo",
            "pacman",
            "apt",
            "apt-get",
            "yum",
            "brew",
            "dnf",
            "zypper",
            "npm",
            "yarn",
            "pnpm",
            "node",
            "docker",
            "docker-compose",
            "systemctl",
            "journalctl",
            "service",
            "which",
            "whereis",
            "file",
            "lsof",
            "make",
            "ninja",
            "cmake",
            "gcc",
            "clang",
            "go",
            "ruby",
            "php",
            "test",
            "[ ",
            "head",
            "tail",
            "wc",
            "sort",
            "uniq",
            "cut",
            "paste",
            "tr",
        ];
        let first = s.split_whitespace().next().unwrap_or("");
        verbs.contains(&first)
            || s.contains('|')
            || s.contains('>')
            || s.contains("&&")
            || s.contains(";")
    };

    rsx! {
        // Central Panel
        main {
            class: "flex-1 flex flex-col p-4",
            style: "display: flex; flex-direction: column; overflow: hidden;",
            // Messages scroll area
            div {
                class: "flex-1 overflow-y-auto mb-4 pane-scroll",
                style: "flex: 1; overflow-y: auto; padding: 12px; border: 1px solid #3a2d56; border-radius: 12px; background: linear-gradient(160deg, #171025 0%, #0f0b1a 50%, #0b0816 100%); box-shadow: inset 0 1px 0 #2a1e40, 0 10px 24px #00000055;",
                for message in messages() {
                    if message.author == "Kael" {
                            div {
                                class: "flex gap-3 mb-4 items-start",
                                div { style: "width: 48px; height: 48px; border-radius: 50%; background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); display: flex; align-items: center; justify-content: center; flex-shrink: 0;",
                                    span { style: "font-weight: bold; color: #120e1a; font-size: 20px;", "K" }
                                }
                                div {
                                    // Kael bubble (AI or terminal output)
                                    style: "max-width: 720px; word-wrap: break-word; word-break: break-word; overflow-wrap: break-word; background: linear-gradient(150deg, #1c162b 0%, #120e1a 55%, #0f0c1a 100%); color: #f7f2ff; padding: 14px 16px; border-radius: 12px; border: 1px solid #3a2d56; box-shadow: 0 14px 34px #00000066, inset 4px 0 0 #ffcc00, inset 0 1px 0 #ffd166; font-size: 15px; line-height: 1.55;",
                                    if is_command(&message.text) {
                                        div { class: "flex items-center gap-2 mb-2", style: "color: #7aebbe; font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em;",
                                            PanelIcon { class: "w-3 h-3" }
                                            span { "Terminal" }
                                        }
                                    }
                                    // Render monospace style if content looks like terminal output
                                    {
                                        let txt = message.text.clone();
                                        let monospace = txt.contains('\n') || txt.starts_with("$");
                                        rsx!{
                                            if monospace {
                                                    div { style: "position: relative; border-radius: 12px; overflow: hidden; border: 1px solid #3a2d56; box-shadow: inset 0 1px 0 #2a1e40;",
                                                        div { style: "height: 32px; display: flex; align-items: center; gap: 8px; padding: 0 12px; background: linear-gradient(120deg, #1f1631 0%, #181024 80%, #120b1f 100%); color: #7aebbe; font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em; border-bottom: 1px solid #3a2d56;",
                                                            SparkIcon { class: "w-3 h-3" }
                                                            span { "Shell" }
                                                        }
                                                        pre { style: "margin: 0; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace; background: linear-gradient(180deg, #181024 0%, #120b1f 100%); padding: 10px 40px 10px 10px; max-height: 300px; overflow-y: auto;", "{txt}", if message.is_streaming { " ⏳" } else { "" } }
                                                        button {
                                                            style: "position: absolute; top: 8px; right: 8px; background: #1f1631; color: #f7f2ff; border: 1px solid #3a2d56; border-radius: 8px; padding: 4px 8px; font-size: 12px;",
                                                            onclick: move |_| {
                                                                if let Ok(mut cb) = arboard::Clipboard::new() { let _ = cb.set_text(txt.clone()); }
                                                            },
                                                            "Copy"
                                                        }
                                                    }
                                            } else {
                                                p { style: "margin: 0; word-wrap: break-word; word-break: break-word; overflow-wrap: break-word;", "{txt}", if message.is_streaming { " ⏳" } else { "" } }
                                            }
                                        }
                                    }
                                }
                            }
                    } else {
                            div {
                                class: "flex justify-end gap-3 mb-4 items-start",
                                div {
                                    style: "max-width: 720px; word-wrap: break-word; word-break: break-word; overflow-wrap: break-word; background: linear-gradient(150deg, #2a1a33 0%, #1d1326 60%, #120b1f 100%); color: #ffe9f0; padding: 14px 16px; border-radius: 12px; border: 1px solid #4b305a; box-shadow: 0 12px 30px #00000055, inset -4px 0 0 #e040fb, inset 0 1px 0 #ffcc00; font-size: 15px; line-height: 1.55;",
                                    if is_command(&message.text) {
                                        div { class: "flex items-center gap-2 mb-2", style: "color: #ffcc00; font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em;",
                                            PanelIcon { class: "w-3 h-3" }
                                            span { "Command" }
                                        }
                                    }
                                    p { style: "margin: 0;", "{message.text}" }
                                }
                                if let Some(photo) = props.user_photo_url.clone() {
                                    img { src: "{photo}", style: "width: 48px; height: 48px; border-radius: 50%; border: 2px solid #ffcc00; flex-shrink: 0;" }
                                } else {
                                    div { style: "width: 48px; height: 48px; border-radius: 50%; background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); display: flex; align-items: center; justify-content: center; flex-shrink: 0;",
                                        span { style: "font-weight: bold; color: #120e1a; font-size: 18px;", "{props.user_name.chars().next().unwrap_or('A')}" }
                                    }
                                }
                            }
                    }
                }
            }
            // Input area at bottom
            div {
                class: "flex items-center gap-3 p-3 rounded-xl border",
                style: "background: radial-gradient(1200px at 10% 10%, #201637 0%, #160f28 45%, #120b1f 100%); border: 1px solid #4a3a62; box-shadow: 0 14px 40px #00000077, inset 0 1px 0 #2a1e40; flex-shrink: 0; border-radius: 16px;",
                // Echo toggle
                button {
                    class: "px-2 py-1 rounded-md border",
                    style: "border-color: #3a2d56; background: linear-gradient(135deg, #1f1631 0%, #181024 80%, #120b1f 100%); color: #a99ec3; font-size: 12px; letter-spacing: 0.04em;",
                    onclick: move |_| echo_commands.set(!echo_commands()),
                    SparkIcon { class: "w-3 h-3" }
                    span { style: "margin-left: 6px;", if echo_commands() { "Echo cmds: ON" } else { "Echo cmds: OFF" } }
                }
                input {
                    class: "w-full p-3 rounded-lg border focus:outline-none focus:ring-2",
                    style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff; box-shadow: inset 0 0 0 9999px rgba(255,255,255,0.00);",
                    placeholder: "Converse with Kael... (or type shell commands, y/n answers, etc.)",
                    value: "{user_input()}",
                    oninput: move |event| user_input.set(event.value()),
                    onkeydown: move |event| {
                        if event.key() == Key::Enter {
                            let input_text = user_input();
                            if input_text.trim().is_empty() {
                                return;
                            }

                            // Check if it's a command
                            if is_command(&input_text) {
                                let cmd = if input_text.starts_with('!') {
                                    input_text.trim_start_matches('!').to_string()
                                } else {
                                    input_text.clone()
                                };

                                // Show command in chat if echo enabled
                                if echo_commands() {
                                    messages.write().push(Message {
                                        author: "Architect".to_string(),
                                        text: input_text.clone(),
                                        is_streaming: false
                                    });
                                }

                                // Handle sudo separately
                                if cmd.starts_with("sudo ") || cmd == "sudo" {
                                    sudo_pending.set(Some(cmd.clone()));
                                    props.current_cmd.set(cmd);
                                } else {
                                    // Regular command - send to PTY
                                    let p = pty();
                                    let cmd_display = cmd.clone();
                                    props.current_cmd.set(cmd_display);
                                    spawn(async move {
                                        if let Err(e) = p.write_line(&cmd).await {
                                            eprintln!("PTY write error: {e}");
                                        }
                                    });
                                }
                            } else {
                                // Not a command: treat as chat to LLM
                                messages.write().push(Message {
                                    author: "Architect".to_string(),
                                    text: input_text.clone(),
                                    is_streaming: false,
                                });
                                save_messages(&messages.read());
                                let mut msgs = messages.clone();
                                let prompt = input_text.clone();
                                spawn(async move {
                                    // Auto-select provider based on query type
                                    let provider = if is_system_query(&prompt) {
                                        llm::LLMProvider::Ollama
                                    } else {
                                        llm::LLMProvider::Ollama
                                    };

                                    let req = llm::LLMRequest {
                                        provider,
                                        model: "mistral".to_string(),
                                        prompt,
                                        api_key: None,
                                        system: Some(llm::get_kael_system_prompt()),
                                    };
                                    match llm::send_request(req, None).await {
                                        Ok(res) => {
                                            msgs.write().push(Message {
                                                author: "Kael".to_string(),
                                                text: res.content.clone(),
                                                is_streaming: false,
                                            });
                                            log::info!("LLM response saved to messages");
                                            save_messages(&msgs.read());
                                        }
                                        Err(e) => {
                                            msgs.write().push(Message {
                                                author: "Kael".to_string(),
                                                text: format!("Error: {}", e),
                                                is_streaming: false,
                                            });
                                            log::error!("LLM error, saving to messages");
                                            save_messages(&msgs.read());
                                        }
                                    }
                                });
                            }
                            user_input.set("".to_string());
                        }
                    }
                }
                // Sudo prompt (appears only when needed)
                if sudo_pending().is_some() {
                    div { class: "flex items-center gap-2 px-2 py-2 rounded-md border", style: "border-color: #3a2d56; background: linear-gradient(135deg, #1f1631 0%, #181024 80%, #120b1f 100%);",
                        span { style: "color: #a99ec3; font-size: 12px;", "sudo password:" }
                        input {
                            class: "p-2 rounded-md border",
                            style: "background-color: #0f0b1a; border-color: #3a2a50; color: #f7f2ff;",
                            value: "{sudo_pw}",
                            oninput: move |e| sudo_pw.set(e.value()),
                            r#type: "password",
                            placeholder: "••••••••",
                        }
                        button { class: "px-2 py-1 rounded-md font-bold", style: "background: linear-gradient(135deg, #e040fb 0%, #ffcc00 60%, #7aebbe 100%); color: #120e1a; border: 1px solid #ffcc00;",
                            onclick: move |_| {
                                if let Some(cmd) = sudo_pending() {
                                    let p = pty();
                                    let pw = sudo_pw();
                                    spawn(async move {
                                        // Write password then command
                                        let sudo_line = format!("sudo {}", cmd.strip_prefix("sudo ").unwrap_or(&cmd));
                                        if let Err(e) = p.write_line(&sudo_line).await {
                                            eprintln!("PTY sudo error: {e}");
                                        }
                                        // PTY will prompt for password, send it
                                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                        if let Err(e) = p.write_line(&pw).await {
                                            eprintln!("PTY password error: {e}");
                                        }
                                    });
                                    sudo_pw.set(String::new());
                                    sudo_pending.set(None);
                                }
                            },
                            "Submit"
                        }
                    }
                }
                button {
                    class: "px-3 py-2 rounded-lg font-bold transition-colors",
                    style: "background: linear-gradient(135deg, #e040fb 0%, #ffcc00 45%, #7aebbe 100%); color: #120e1a; flex-shrink: 0; border: 1px solid #ffcc00; border-radius: 12px; box-shadow: 0 10px 26px #00000088;",
                    onclick: move |_| {
                        if !user_input().is_empty() {
                            let input_text = user_input();
                            if input_text.trim().is_empty() {
                                return;
                            }

                            if is_command(&input_text) {
                                let cmd = if input_text.starts_with('!') {
                                    input_text.trim_start_matches('!').to_string()
                                } else {
                                    input_text.clone()
                                };

                                if echo_commands() {
                                    messages.write().push(Message {
                                        author: "Architect".to_string(),
                                        text: input_text.clone(),
                                        is_streaming: false
                                    });
                                }

                                if cmd.starts_with("sudo ") || cmd == "sudo" {
                                    sudo_pending.set(Some(cmd.clone()));
                                    props.current_cmd.set(cmd);
                                } else {
                                    let p = pty();
                                    let cmd_display = cmd.clone();
                                    props.current_cmd.set(cmd_display);
                                    spawn(async move {
                                        if let Err(e) = p.write_line(&cmd).await {
                                            eprintln!("PTY write error: {e}");
                                        }
                                    });
                                }
                            } else {
                                // Send to LLM as chat
                                messages.write().push(Message {
                                    author: "Architect".to_string(),
                                    text: input_text.clone(),
                                    is_streaming: false
                                });
                                save_messages(&messages.read());
                                let mut msgs = messages.clone();
                                let prompt = input_text.clone();
                                spawn(async move {
                                    // Auto-select provider based on query type
                                    let provider = if is_system_query(&prompt) {
                                        llm::LLMProvider::Ollama
                                    } else {
                                        llm::LLMProvider::Ollama
                                    };

                                    let req = llm::LLMRequest {
                                        provider,
                                        model: "mistral".to_string(),
                                        prompt,
                                        api_key: None,
                                        system: Some(llm::get_kael_system_prompt()),
                                    };
                                    match llm::send_request(req, None).await {
                                        Ok(res) => {
                                            msgs.write().push(Message {
                                                author: "Kael".to_string(),
                                                text: res.content,
                                                is_streaming: false,
                                            });
                                            save_messages(&msgs.read());
                                        }
                                        Err(e) => {
                                            msgs.write().push(Message {
                                                author: "Kael".to_string(),
                                                text: format!("Error: {}", e),
                                                is_streaming: false,
                                            });
                                            save_messages(&msgs.read());
                                        }
                                    }
                                });
                            }
                            user_input.set("".to_string());
                        }
                    },
                    SendIcon { class: "w-6 h-6" }
                }
            }
        }
    }
}
