use crate::components::icons::{PanelIcon, SendIcon, SparkIcon};
#[allow(unused_imports)]
use crate::llm::{self, LLMProvider, LLMRequest};
use crate::services::command_rewriter::{self, AIDecision, KaelOSPersonality, UserContext};
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

fn provider_label_to_enum(label: &str) -> Option<llm::LLMProvider> {
    match label {
        "Ollama (Local)" => Some(llm::LLMProvider::Ollama),
        "Mistral AI" => Some(llm::LLMProvider::Mistral),
        "Google Gemini" => Some(llm::LLMProvider::Gemini),
        "GitHub Copilot" => Some(llm::LLMProvider::Copilot),
        "GitHub Copilot CLI" => Some(llm::LLMProvider::CopilotCLI),
        "Office 365 AI" => Some(llm::LLMProvider::Office365AI),
        "Google One AI" => Some(llm::LLMProvider::GoogleOneAI),
        _ => None,
    }
}

fn provider_enum_to_label(provider: &llm::LLMProvider) -> &'static str {
    match provider {
        llm::LLMProvider::Ollama => "Ollama (Local)",
        llm::LLMProvider::Mistral => "Mistral AI",
        llm::LLMProvider::Gemini => "Google Gemini",
        llm::LLMProvider::Copilot => "GitHub Copilot",
        llm::LLMProvider::CopilotCLI => "GitHub Copilot CLI",
        llm::LLMProvider::Office365AI => "Office 365 AI",
        llm::LLMProvider::GoogleOneAI => "Google One AI",
    }
}

fn build_provider_order() -> Vec<String> {
    if let Ok(json) = std::fs::read_to_string("/tmp/kael_provider_order.json") {
        if let Ok(list) = serde_json::from_str::<Vec<String>>(&json) {
            if !list.is_empty() {
                return list;
            }
        }
    }
    vec![
        "Ollama (Local)".to_string(),
        "Mistral AI".to_string(),
        "Google Gemini".to_string(),
        "GitHub Copilot".to_string(),
        "GitHub Copilot CLI".to_string(),
        "Office 365 AI".to_string(),
        "Google One AI".to_string(),
    ]
}

fn next_provider_after(
    current_label: &str,
    order: &[String],
) -> Option<(llm::LLMProvider, Vec<llm::LLMProvider>)> {
    let mut providers = Vec::new();
    for label in order {
        if let Some(p) = provider_label_to_enum(label) {
            providers.push((label, p));
        }
    }

    let pos = providers.iter().position(|(label, _)| label.as_str() == current_label)?;
    if pos + 1 >= providers.len() {
        return None;
    }
    let next = providers[pos + 1].1.clone();
    let rest = providers[(pos + 2)..]
        .iter()
        .map(|(_, p)| p.clone())
        .collect::<Vec<_>>();
    Some((next, rest))
}

fn increment_usage(provider_label: String) {
    let path = "/tmp/kael_provider_usage.json";
    let mut map: std::collections::BTreeMap<String, u64> = if let Ok(s) = std::fs::read_to_string(path) {
        serde_json::from_str(&s).unwrap_or_default()
    } else {
        std::collections::BTreeMap::new()
    };
    *map.entry(provider_label).or_insert(0) += 1;
    if let Ok(json) = serde_json::to_string(&map) {
        let _ = std::fs::write(path, json);
    }
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

#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Message {
    pub author: String,
    pub text: String,
    pub is_streaming: bool,
    pub provider: Option<String>,
    pub prompt: Option<String>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ChatProps {
    pub term_out: Signal<String>,
    pub pty: Signal<PtyTerminal>,
    pub current_cmd: Signal<String>,
    pub user_photo_url: Option<String>,
    pub user_name: String,
    pub auth_service: Signal<crate::auth::AuthService>,
    pub clear_chat_trigger: Signal<bool>,
    pub messages_out: Signal<Vec<Message>>, // live messages for export
    #[props(default = use_signal(|| String::new()))]
    pub last_provider: Signal<String>,
    #[props(default = use_signal(|| false))]
    pub hybrid_assist: Signal<bool>,
}

#[allow(non_snake_case)]
pub fn ChatPanel(mut props: ChatProps) -> Element {
    // Persistent clipboard instance to avoid quick drop issues on Linux
    let mut clipboard = use_signal(|| {
        arboard::Clipboard::new().ok()
    });
    let load_messages = || -> Vec<Message> {
        if let Ok(json) = std::fs::read_to_string("/tmp/kael_chat_history.json") {
            serde_json::from_str(&json).unwrap_or_else(|_| {
                vec![Message {
                    author: "Kael".to_string(),
                    text: "Greetings, Architect! I am Kael, your partner in creation.".to_string(),
                    is_streaming: false,
                    provider: None,
                    prompt: None,
                }]
            })
        } else {
            vec![Message {
                author: "Kael".to_string(),
                text: "Greetings, Architect! I am Kael, your partner in creation.".to_string(),
                is_streaming: false,
                provider: None,
                prompt: None,
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
    
    // Load user context for smart reformatting (lazy initialization)
    let mut user_context = use_signal(|| None::<UserContext>);
    let _personality = use_signal(|| KaelOSPersonality::default());
    
    // Initialize context on first load
    use_effect(move || {
        if user_context().is_none() {
            spawn(async move {
                if let Ok(ctx) = command_rewriter::build_user_context().await {
                    user_context.set(Some(ctx));
                }
            });
        }
    });
    let mut sudo_pw = use_signal(String::new);
    let pty = props.pty;

    // Listen for clear trigger and reset messages + persist
    {
        let mut msgs = messages.clone();
        let trig = props.clear_chat_trigger.clone();
        use_effect(move || {
            if trig() {
                msgs.set(vec![Message {
                    author: "Kael".to_string(),
                    text: "Chat cleared. Ready for a fresh start.".to_string(),
                    is_streaming: false,
                    ..Default::default()
                }]);
                save_messages(&msgs.read());
            }
        });
    }

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
                        ..Default::default()
                    });
                    save_messages(&current);
                }
            } else {
                // Send a tiny warm-up prompt so first real reply is fast
                // Try llama:latest first (primary), then phi3 (failover)
                let mut warmed = false;
                
                // Try primary: llama:latest
                warmed = llm::warm_local_model("llama:latest").await;
                
                // If llama failed, try failover: phi3
                if !warmed {
                    warmed = llm::warm_local_model("phi3").await;
                }
                
                if !warmed {
                    let mut current = msgs.write();
                    current.push(Message {
                        author: "Kael".to_string(),
                        text: "Local AI responded to ping but warm-up prompt failed. I will still try cloud fallbacks if needed.".to_string(),
                        is_streaming: false,
                        ..Default::default()
                    });
                    save_messages(&current);
                }
            }
        });
    });

    // Auto-scroll to bottom when messages change
    use_effect(move || {
        let _msg_count = messages().len();
        // Update external messages signal for exports
        props.messages_out.set(messages());
        spawn(async move {
            // Small delay to ensure DOM is updated
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let script = r#"
                const container = document.getElementById('chat-messages');
                if (container) {
                    container.scrollTop = container.scrollHeight;
                }
            "#;
            eval(script);
        });
    });

    // When user submits input, classify and dispatch
    let _on_submit = move || {
        let input = user_input().clone();
        if input.is_empty() {
            return;
        }

        // Smart reformatting: check if input needs correction
        let mut corrected_input = input.clone();
        let mut correction_notes = Vec::new();
        
        if let Some(ctx) = user_context() {
            // Apply context-aware command rewriting
            let (rewritten, corrections) = command_rewriter::rewrite_command(&input, &ctx);
            corrected_input = rewritten;
            correction_notes = corrections;
        }
        
        // Determine if we should handle locally or escalate
        let _escalate_decision = user_context()
            .as_ref()
            .map(|ctx| command_rewriter::should_escalate(&input, ctx))
            .unwrap_or(AIDecision::AskForClarification("System context not loaded yet".to_string()));

        // Add user message to chat
        let display_text = if !correction_notes.is_empty() {
            format!("{}\n\n🔧 Auto-corrections:\n{}", 
                input, 
                correction_notes.iter().map(|c| format!("  • {}", c)).collect::<Vec<_>>().join("\n")
            )
        } else {
            input.clone()
        };

        messages.write().push(Message {
            author: "Architect".to_string(),
            text: display_text,
            is_streaming: false,
            provider: None,
            prompt: None,
        });
        save_messages(&messages.read());

        if is_command(&corrected_input) {
            // It's a command, send to terminal (using corrected version)
            let pty_write = pty.read().clone();
            let input_clone = corrected_input.clone();

            spawn(async move {
                let _ = pty_write.write_line(&input_clone).await;
            });
        } else {
            // It's a prompt, check escalation decision
            let mut msgs = messages.clone();
            let input_clone = corrected_input.clone();
            
            spawn(async move {
                let user_opt = props.auth_service.read().get_user();
                
                // Respect escalation decision for provider selection
                let primary_provider = match _escalate_decision {
                    AIDecision::HandleLocally(_) => LLMProvider::Ollama,
                    AIDecision::EscalateToCloud(_) => {
                        // Try to use best cloud provider from user's preference
                        if let Ok(json) = std::fs::read_to_string("/tmp/kael_last_cloud_provider.json") {
                            if let Ok(provider_name) = serde_json::from_str::<String>(&json) {
                                match provider_name.as_str() {
                                    "Mistral AI" => LLMProvider::Mistral,
                                    "Google Gemini" => LLMProvider::Gemini,
                                    "GitHub Copilot" => LLMProvider::Copilot,
                                    _ => LLMProvider::Ollama,
                                }
                            } else {
                                LLMProvider::Ollama
                            }
                        } else {
                            LLMProvider::Ollama
                        }
                    }
                    AIDecision::AskForClarification(_) => LLMProvider::Ollama,
                };
                
                // Use this provider
                let selected_provider = if is_system_query(&input_clone) {
                    LLMProvider::Ollama // Use local Ollama for system queries (override)
                } else {
                    primary_provider // Use escalation decision
                };

                // Build fallback chain - prefer cloud APIs before CLI
                let fallback_providers = {
                    // If user saved a provider order, honor it
                    fn map_name(n: &str) -> Option<LLMProvider> {
                        match n {
                            "Ollama (Local)" => Some(LLMProvider::Ollama),
                            "Mistral AI" => Some(LLMProvider::Mistral),
                            "Google Gemini" => Some(LLMProvider::Gemini),
                            "GitHub Copilot" => Some(LLMProvider::Copilot),
                            "GitHub Copilot CLI" => Some(LLMProvider::CopilotCLI),
                            "Office 365 AI" => Some(LLMProvider::Office365AI),
                            "Google One AI" => Some(LLMProvider::GoogleOneAI),
                            _ => None,
                        }
                    }
                    if let Ok(json) = std::fs::read_to_string("/tmp/kael_provider_order.json") {
                        if let Ok(list) = serde_json::from_str::<Vec<String>>(&json) {
                            let mut out = Vec::new();
                            for name in list {
                                if let Some(p) = map_name(&name) {
                                    if p != LLMProvider::Ollama { // exclude primary local
                                        out.push((p, None));
                                    }
                                }
                            }
                            if !out.is_empty() { out } else {
                                vec![
                                    (LLMProvider::Mistral, None),
                                    (LLMProvider::Gemini, None),
                                    (LLMProvider::Copilot, None),
                                    (LLMProvider::CopilotCLI, None),
                                    (LLMProvider::Office365AI, None),
                                    (LLMProvider::GoogleOneAI, None),
                                ]
                            }
                        } else {
                            vec![
                                (LLMProvider::Mistral, None),
                                (LLMProvider::Gemini, None),
                                (LLMProvider::Copilot, None),
                                (LLMProvider::CopilotCLI, None),
                                (LLMProvider::Office365AI, None),
                                (LLMProvider::GoogleOneAI, None),
                            ]
                        }
                    } else {
                        vec![
                            (LLMProvider::Mistral, None),
                            (LLMProvider::Gemini, None),
                            (LLMProvider::Copilot, None),
                            (LLMProvider::CopilotCLI, None),
                            (LLMProvider::Office365AI, None),
                            (LLMProvider::GoogleOneAI, None),
                        ]
                    }
                };

                let prompt_for_save = input_clone.clone();
                let req = LLMRequest {
                    provider: selected_provider,
                    model: String::new(), // resolved per provider in fallback helper
                    prompt: input_clone,
                    api_key: None,
                    system: Some(llm::get_kael_system_prompt()),
                };

                let user_ref = user_opt.as_ref();
                match llm::send_request_with_fallback(req, user_ref, fallback_providers).await {
                    Ok(res) => {
                        // Track provider usage counts
                        {
                            let path = "/tmp/kael_provider_usage.json";
                            let mut map: std::collections::BTreeMap<String, u64> = if let Ok(s) = std::fs::read_to_string(path) {
                                serde_json::from_str(&s).unwrap_or_default()
                            } else { std::collections::BTreeMap::new() };
                            let name = match res.provider {
                                llm::LLMProvider::Ollama => "Ollama (Local)",
                                llm::LLMProvider::Mistral => "Mistral AI",
                                llm::LLMProvider::Gemini => "Google Gemini",
                                llm::LLMProvider::Copilot => "GitHub Copilot",
                                llm::LLMProvider::CopilotCLI => "GitHub Copilot CLI",
                                llm::LLMProvider::Office365AI => "Office 365 AI",
                                llm::LLMProvider::GoogleOneAI => "Google One AI",
                            }.to_string();
                            *map.entry(name).or_insert(0) += 1;
                            if let Ok(json) = serde_json::to_string(&map) { let _ = std::fs::write(path, json); }
                        }
                        msgs.write().push(Message {
                            author: "Kael".to_string(),
                            text: res.content,
                            is_streaming: false,
                            provider: Some(match res.provider {
                                llm::LLMProvider::Ollama => "Ollama (Local)".to_string(),
                                llm::LLMProvider::Mistral => "Mistral AI".to_string(),
                                llm::LLMProvider::Gemini => "Google Gemini".to_string(),
                                llm::LLMProvider::Copilot => "GitHub Copilot".to_string(),
                                llm::LLMProvider::CopilotCLI => "GitHub Copilot CLI".to_string(),
                                llm::LLMProvider::Office365AI => "Office 365 AI".to_string(),
                                llm::LLMProvider::GoogleOneAI => "Google One AI".to_string(),
                            }),
                            prompt: Some(prompt_for_save.clone()),
                        });
                        save_messages(&msgs.read());
                    }
                    Err(e) => {
                        msgs.write().push(Message {
                            author: "Kael".to_string(),
                            text: format!("❌ All AI providers failed:\n\n{}\n\n💡 Tip: Enable cloud providers and add API keys in Settings → Providers tab.", e),
                            is_streaming: false,
                            prompt: Some(prompt_for_save.clone()),
                            ..Default::default()
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
                id: "chat-messages",
                class: "flex-1 overflow-y-auto mb-4 pane-scroll",
                style: "flex: 1; overflow-y: auto; scroll-behavior: smooth; padding: 12px; border: 1px solid #3a2d56; border-radius: 12px; background: linear-gradient(160deg, #171025 0%, #0f0b1a 50%, #0b0816 100%); box-shadow: inset 0 1px 0 #2a1e40, 0 10px 24px #00000055;",
                // Provider status badge (top right of messages area)
                {
                    let lp = (props.last_provider)();
                    rsx!{
                        div { style: "display: flex; justify-content: flex-end; margin-bottom: 8px;",
                            if !lp.is_empty() {
                                span { class: "chip", style: "color: #ffcc00;", "Provider: {lp}" }
                            }
                        }
                    }
                }
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
                                                        pre { style: "margin: 0; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace; background: linear-gradient(180deg, #181024 0%, #120b1f 100%); padding: 10px 40px 10px 10px; max-height: 300px; overflow-y: auto; white-space: pre-wrap; word-wrap: break-word; overflow-wrap: break-word;", "{txt}", if message.is_streaming { " ⏳" } else { "" } }
                                                        button {
                                                            style: "position: absolute; top: 8px; right: 8px; background: #1f1631; color: #f7f2ff; border: 1px solid #3a2d56; border-radius: 8px; padding: 4px 8px; font-size: 12px;",
                                                            onclick: move |_| {
                                                                {
                                                                    let mut opt = clipboard.write();
                                                                    if let Some(cb) = opt.as_mut() {
                                                                        let _ = cb.set_text(txt.clone());
                                                                    } else if let Ok(mut cb) = arboard::Clipboard::new() {
                                                                        let _ = cb.set_text(txt.clone());
                                                                        *opt = Some(cb);
                                                                    }
                                                                }
                                                            },
                                                            "Copy"
                                                        }
                                                    }
                                            } else {
                                                p { style: "margin: 0; word-wrap: break-word; word-break: break-word; overflow-wrap: break-word;", "{txt}", if message.is_streaming { " ⏳" } else { "" } }
                                                if let Some(prov) = message.provider.as_ref() {
                                                    div { style: "margin-top: 6px; color: #a99ec3; font-size: 12px;", "via {prov}" }
                                                }
                                            }
                                        }
                                    }
                                    // Try next provider button (Hybrid Assist)
                                    if (props.hybrid_assist)() {
                                        if let Some(cur_prov) = message.provider.as_ref() {
                                            if let Some(orig_prompt) = message.prompt.as_ref() {
                                                if let Some((next_provider, rest)) = next_provider_after(cur_prov, &build_provider_order()) {
                                                    {
                                                        let prompt_clone = orig_prompt.clone();
                                                        let mut lp = props.last_provider.clone();
                                                        let mut msgs = messages.clone();
                                                        let auth_signal = props.auth_service.clone();
                                                        let remaining = rest.to_vec();
                                                        rsx! {
                                                            button { style: "margin-top: 8px; padding: 6px 10px; border-radius: 6px; border: 1px solid #3a2d56; background: linear-gradient(135deg, #1f1631 0%, #181024 100%); color: #a99ec3; font-size: 12px;",
                                                                onclick: move |_| {
                                                                    let user_opt = auth_signal.read().get_user();
                                                                    let prompt_value = prompt_clone.clone();
                                                                    let req = llm::LLMRequest {
                                                                        provider: next_provider.clone(),
                                                                        model: String::new(),
                                                                        prompt: prompt_value.clone(),
                                                                        api_key: None,
                                                                        system: Some(llm::get_kael_system_prompt()),
                                                                    };
                                                                    let mut fb = Vec::new();
                                                                    for p in remaining.clone() { fb.push((p, None)); }
                                                                    spawn(async move {
                                                                        let prompt_saved = prompt_value.clone();
                                                                        match llm::send_request_with_fallback(req, user_opt.as_ref(), fb).await {
                                                                            Ok(res) => {
                                                                                let provider_label = provider_enum_to_label(&res.provider).to_string();
                                                                                lp.set(provider_label.clone());
                                                                                increment_usage(provider_label.clone());
                                                                                msgs.write().push(Message {
                                                                                    author: "Kael".to_string(),
                                                                                    text: res.content,
                                                                                    is_streaming: false,
                                                                                    provider: Some(provider_label),
                                                                                    prompt: Some(prompt_saved.clone()),
                                                                                });
                                                                                save_messages(&msgs.read());
                                                                            }
                                                                            Err(e) => {
                                                                                msgs.write().push(Message {
                                                                                    author: "Kael".to_string(),
                                                                                    text: format!("❌ Next provider failed: {}", e),
                                                                                    is_streaming: false,
                                                                                    provider: None,
                                                                                    prompt: Some(prompt_saved.clone()),
                                                                                });
                                                                                save_messages(&msgs.read());
                                                                            }
                                                                        }
                                                                    });
                                                                },
                                                                "Try next provider"
                                                            }
                                                        }
                                                    }
                                                }
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
                                        is_streaming: false,
                                        ..Default::default()
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
                                // Not a command: treat as chat to LLM with fallback providers
                                messages.write().push(Message {
                                    author: "Architect".to_string(),
                                    text: input_text.clone(),
                                    is_streaming: false,
                                    ..Default::default()
                                });
                                save_messages(&messages.read());

                                let mut msgs = messages.clone();
                                let prompt = input_text.clone();
                                let auth_service = props.auth_service.clone();

                                spawn(async move {
                                    log::info!("🔍 Sending prompt to LLM: {}", prompt);

                                    // Auto-select provider based on query type
                                    let primary_provider = if is_system_query(&prompt) {
                                        llm::LLMProvider::Ollama
                                    } else {
                                        llm::LLMProvider::Ollama
                                    };
                                    log::info!("📍 Primary provider: {:?}", primary_provider);

                                    // Fallback chain of cloud providers (keys loaded lazily from Firebase)
                                    let fallback_providers = vec![
                                        (llm::LLMProvider::Mistral, None),
                                        (llm::LLMProvider::Gemini, None),
                                        (llm::LLMProvider::Copilot, None),
                                        (llm::LLMProvider::CopilotCLI, None),
                                    ];

                                    let req = llm::LLMRequest {
                                        provider: primary_provider,
                                        model: String::new(),
                                        prompt: prompt.clone(),
                                        api_key: None,
                                        system: Some(llm::get_kael_system_prompt()),
                                    };

                                    let user_opt = auth_service.read().get_user();
                                    log::info!("👤 User authenticated: {}", user_opt.is_some());

                                    match llm::send_request_with_fallback(req, user_opt.as_ref(), fallback_providers).await {
                                        Ok(res) => {
                                            log::info!("✅ Response provider: {:?}", res.provider);
                                            // Update last provider badge
                                            props.last_provider.set(match res.provider {
                                                llm::LLMProvider::Ollama => "Ollama (Local)".to_string(),
                                                llm::LLMProvider::Mistral => "Mistral AI".to_string(),
                                                llm::LLMProvider::Gemini => "Google Gemini".to_string(),
                                                llm::LLMProvider::Copilot => "GitHub Copilot".to_string(),
                                                llm::LLMProvider::CopilotCLI => "GitHub Copilot CLI".to_string(),
                                                llm::LLMProvider::Office365AI => "Office 365 AI".to_string(),
                                                llm::LLMProvider::GoogleOneAI => "Google One AI".to_string(),
                                            });
                                            msgs.write().push(Message {
                                                author: "Kael".to_string(),
                                                text: res.content,
                                                is_streaming: false,
                                                provider: Some(match res.provider {
                                                    llm::LLMProvider::Ollama => "Ollama (Local)".to_string(),
                                                    llm::LLMProvider::Mistral => "Mistral AI".to_string(),
                                                    llm::LLMProvider::Gemini => "Google Gemini".to_string(),
                                                    llm::LLMProvider::Copilot => "GitHub Copilot".to_string(),
                                                    llm::LLMProvider::CopilotCLI => "GitHub Copilot CLI".to_string(),
                                                    llm::LLMProvider::Office365AI => "Office 365 AI".to_string(),
                                                    llm::LLMProvider::GoogleOneAI => "Google One AI".to_string(),
                                                }),
                                                prompt: Some(prompt.clone()),
                                            });
                                            save_messages(&msgs.read());
                                        }
                                        Err(e) => {
                                            log::error!("❌ All providers failed: {}", e);
                                            msgs.write().push(Message {
                                                author: "Kael".to_string(),
                                                text: format!("❌ All providers failed: {}\n\n💡 Check API keys in Settings → Providers. (Mistral/Gemini/Copilot)", e),
                                                is_streaming: false,
                                                prompt: Some(prompt.clone()),
                                                ..Default::default()
                                            });
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
                                        is_streaming: false,
                                        ..Default::default()
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
                                    is_streaming: false,
                                    ..Default::default()
                                });
                                save_messages(&messages.read());
                                let mut msgs = messages.clone();
                                let prompt = input_text.clone();
                                let auth_service = props.auth_service.clone();
                                let mut lp = props.last_provider.clone();
                                spawn(async move {
                                    // Auto-select provider based on query type
                                    let primary_provider = if is_system_query(&prompt) {
                                        llm::LLMProvider::Ollama
                                    } else {
                                        llm::LLMProvider::Ollama
                                    };

                                    // Fallback chain of cloud providers
                                    let fallback_providers = vec![
                                        (llm::LLMProvider::Mistral, None),
                                        (llm::LLMProvider::Gemini, None),
                                        (llm::LLMProvider::Copilot, None),
                                        (llm::LLMProvider::CopilotCLI, None),
                                    ];

                                    let req = llm::LLMRequest {
                                        provider: primary_provider,
                                        model: String::new(),
                                        prompt: prompt.clone(),
                                        api_key: None,
                                        system: Some(llm::get_kael_system_prompt()),
                                    };

                                    let user_opt = auth_service.read().get_user();
                                    match llm::send_request_with_fallback(req, user_opt.as_ref(), fallback_providers).await {
                                        Ok(res) => {
                                            lp.set(match res.provider {
                                                llm::LLMProvider::Ollama => "Ollama (Local)".to_string(),
                                                llm::LLMProvider::Mistral => "Mistral AI".to_string(),
                                                llm::LLMProvider::Gemini => "Google Gemini".to_string(),
                                                llm::LLMProvider::Copilot => "GitHub Copilot".to_string(),
                                                llm::LLMProvider::CopilotCLI => "GitHub Copilot CLI".to_string(),
                                                llm::LLMProvider::Office365AI => "Office 365 AI".to_string(),
                                                llm::LLMProvider::GoogleOneAI => "Google One AI".to_string(),
                                            });
                                            msgs.write().push(Message {
                                                author: "Kael".to_string(),
                                                text: res.content,
                                                is_streaming: false,
                                                provider: Some(match res.provider {
                                                    llm::LLMProvider::Ollama => "Ollama (Local)".to_string(),
                                                    llm::LLMProvider::Mistral => "Mistral AI".to_string(),
                                                    llm::LLMProvider::Gemini => "Google Gemini".to_string(),
                                                    llm::LLMProvider::Copilot => "GitHub Copilot".to_string(),
                                                    llm::LLMProvider::CopilotCLI => "GitHub Copilot CLI".to_string(),
                                                    llm::LLMProvider::Office365AI => "Office 365 AI".to_string(),
                                                    llm::LLMProvider::GoogleOneAI => "Google One AI".to_string(),
                                                }),
                                                prompt: Some(prompt.clone()),
                                            });
                                            save_messages(&msgs.read());
                                        }
                                        Err(e) => {
                                            msgs.write().push(Message {
                                                author: "Kael".to_string(),
                                                text: format!("❌ Error: {}\n\n💡 Make sure you've added API keys in Settings → Providers tab for cloud fallbacks.", e),
                                                is_streaming: false,
                                                prompt: Some(prompt.clone()),
                                                ..Default::default()
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
