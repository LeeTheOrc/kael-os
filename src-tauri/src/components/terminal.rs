// src-tauri/src/components/terminal.rs
use dioxus::prelude::*;

use crate::components::icons::{PanelIcon, SparkIcon};

#[derive(Props, Clone, PartialEq)]
pub struct TerminalProps {
    pub term_out: Signal<String>,
}

#[allow(non_snake_case)]
pub fn TerminalPanel(props: TerminalProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-3",
            // Title / header
            div { class: "flex items-center gap-2",
                PanelIcon { class: "w-3 h-3" }
                span { style: "color: #7aebbe; font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em;", "Terminal" }
            }
            // Container card
            div {
                class: "pane-scroll",
                style: "padding: 12px; border: 1px solid #3a2d56; border-radius: 12px; background: linear-gradient(160deg, #171025 0%, #0f0b1a 55%, #0b0816 100%); box-shadow: inset 0 1px 0 #2a1e40, 0 10px 24px #00000055;",
                // Shell header
                div { style: "height: 32px; display: flex; align-items: center; gap: 8px; padding: 0 12px; background: linear-gradient(120deg, #1f1631 0%, #181024 80%, #120b1f 100%); color: #7aebbe; font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em; border: 1px solid #3a2d56; border-radius: 8px;",
                    SparkIcon { class: "w-3 h-3" }
                    span { "Shell" }
                }
                // Output area
                pre { style: "margin: 12px 0 0; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace; background: linear-gradient(180deg, #181024 0%, #120b1f 100%); padding: 10px; max-height: 260px; overflow-y: auto; border: 1px solid #3a2d56; border-radius: 10px;", "{props.term_out}" }
            }
        }
    }
}
