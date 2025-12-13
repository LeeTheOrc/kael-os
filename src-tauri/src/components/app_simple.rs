// Minimal working Kael-OS component
use dioxus::prelude::*;
use crate::auth::AuthService;
use crate::components::icons::KaelSigilIcon;

#[allow(non_snake_case)]
pub fn App() -> Element {
    let mut terminal_input = use_signal(String::new);
    let mut auth_service = use_signal(|| AuthService::new());
    let mut show_settings = use_signal(|| false);
    
    let auth = auth_service.read();
    
    rsx! {
        div {
            class: "w-full h-screen flex flex-col bg-gradient-to-br from-[#0f0c1a] to-[#120e1a]",
            style: "font-family: 'Monaco', 'Menlo', monospace; color: #cbd5ff;",

            // Header
            div {
                class: "flex items-center justify-between p-4 border-b",
                style: "border-color: #3a2d56; background: linear-gradient(90deg, #1c162b 0%, #120e1a 100%);",
                
                div { class: "flex items-center gap-3",
                    KaelSigilIcon {},
                    div { style: "font-weight: bold; font-size: 18px;", "Kael-OS" }
                }
                
                button {
                    onclick: move |_| show_settings.toggle(),
                    class: "px-3 py-2 rounded-md text-sm",
                    style: "background: #2a1f40; color: #7aebbe; border: 1px solid #3a2d56; cursor: pointer;",
                    "⚙️ Settings"
                }
            }

            // Main content
            div {
                class: "flex-1 flex overflow-hidden",
                
                // Chat panel
                div {
                    class: "flex-1 flex flex-col p-4",
                    style: "border-right: 1px solid #3a2d56;",
                    
                    div { class: "flex-1 overflow-y-auto mb-4", 
                        style: "background: #0f0b1a; border: 1px solid #3a2d56; border-radius: 8px; padding: 12px;",
                        div { style: "color: #7aebbe;", "👋 Welcome to Kael-OS!" }
                        div { style: "color: #a99ec3; font-size: 12px; margin-top: 8px;", "Type commands or send messages to AI" }
                    }
                    
                    div { class: "flex gap-2",
                        input {
                            class: "flex-1 p-2 rounded-md",
                            style: "background: #1a1426; border: 1px solid #3a2d56; color: #f7f2ff;",
                            placeholder: "Type command or message...",
                            value: "{terminal_input}",
                            oninput: move |e| terminal_input.set(e.value()),
                            onkeydown: move |event| {
                                if event.key() == Key::Enter {
                                    // Handle input
                                    terminal_input.set(String::new());
                                }
                            },
                        }
                        button {
                            class: "px-4 py-2 rounded-md font-bold",
                            style: "background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); color: #120e1a; border: 1px solid #ffcc00; cursor: pointer;",
                            "Send"
                        }
                    }
                }

                // Right panel
                if show_settings() {
                    div {
                        class: "w-80 flex flex-col p-4",
                        style: "background: #1c162b; border-left: 1px solid #3a2d56; overflow-y-auto;",
                        
                        div { style: "font-weight: bold; margin-bottom: 16px; color: #ffcc00;", "Settings" }
                        
                        div { style: "margin-bottom: 16px;",
                            div { style: "font-size: 12px; color: #a99ec3; margin-bottom: 8px;", "Authentication" }
                            if auth.is_authenticated() {
                                div { style: "padding: 8px; background: #0f0b1a; border-radius: 4px;",
                                    div { style: "color: #7aebbe; font-size: 12px;", "✓ Logged In" }
                                }
                            } else {
                                div { style: "padding: 8px; background: #0f0b1a; border-radius: 4px;",
                                    div { style: "color: #a99ec3; font-size: 12px;", "Not logged in (optional)" }
                                }
                            }
                        }
                        
                        div { style: "font-size: 12px; color: #a99ec3;", "Kael-OS v0.1.0" }
                    }
                } else {
                    // Terminal panel placeholder
                    div {
                        class: "w-80 flex flex-col p-4",
                        style: "background: #1c162b; border-left: 1px solid #3a2d56;",
                        
                        div { style: "font-weight: bold; margin-bottom: 16px; color: #7aebbe;", "Terminal" }
                        div { style: "flex-1; background: #0f0b1a; border-radius: 4px; padding: 8px; font-size: 12px; color: #a99ec3;",
                            "$" span { style: "color: #cbd5ff;", " ready" }
                        }
                    }
                }
            }
        }
    }
}
