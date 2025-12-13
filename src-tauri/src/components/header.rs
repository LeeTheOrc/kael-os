// src-tauri/src/components/header.rs
use crate::components::icons::{CogIcon, KaelSigilIcon, SparkIcon};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct HeaderProps {
    pub show_settings: Signal<bool>,
}

#[allow(non_snake_case)]
pub fn Header(mut props: HeaderProps) -> Element {
    let mut is_menu_open = use_signal(|| false);

    rsx! {
        header {
            class: "flex items-center justify-between px-6 py-4 border-b",
            style: "background: linear-gradient(120deg, #120e1a 0%, #0f0b1f 70%, #0b0816 100%); border-color: #3a2d56; height: 64px; box-shadow: 0 12px 28px #00000077, inset 0 -1px 0 #2a1e40;",
            // Left side - Logo + App name
            div {
                class: "flex items-center gap-3",
                div {
                    class: "flex items-center justify-center font-bold",
                    style: "width: 38px; height: 38px; border-radius: 12px; background: radial-gradient(circle at 30% 30%, #e040fb 0%, #120e1a 55%, #0f0c1a 100%); color: #f7f2ff; border: 1px solid #3a2d56; box-shadow: 0 6px 16px #00000055, inset 3px 0 0 #ffcc00;",
                    KaelSigilIcon { class: "w-5 h-5" }
                },
                span { class: "font-bold text-lg tracking-wide", style: "color: #f7f2ff;", "KAEL OS" }
            },
            // Right side - Settings Menu only
            div {
                class: "relative",
                style: "position: relative;",
                button {
                    class: "p-2 rounded transition-all",
                    style: "color: #a99ec3; display: flex; align-items: center; justify-content: center; border: 1px solid transparent;",
                    onclick: move |_| is_menu_open.toggle(),
                    CogIcon { class: "w-6 h-6" }
                },
                if is_menu_open() {
                    div {
                        class: "rounded-lg shadow-2xl",
                        style: "background: radial-gradient(900px at 20% -10%, #1c162b 0%, #120e1a 65%, #0f0b1f 100%); border: 1px solid #3a2d56; width: 260px; position: absolute; top: 52px; right: 0; z-index: 1000; box-shadow: 0 18px 38px #000000aa; padding: 8px; gap: 6px; display: flex; flex-direction: column;",

                        button {
                            class: "w-full text-left",
                            style: "display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 10px; border: 1px solid #3a2d56; background: linear-gradient(135deg, #e040fb 0%, #ffcc00 60%, #7aebbe 100%); color: #120e1a; font-weight: 700; box-shadow: 0 12px 28px #00000066;",
                            onclick: move |_| {
                                let current = *props.show_settings.read();
                                props.show_settings.set(!current);
                                is_menu_open.set(false);
                            },
                            KaelSigilIcon { class: "w-4 h-4" }
                            span { "Open Settings" }
                        }

                        div { style: "height: 1px; background-color: #3a2d56; margin: 2px 0;" }

                        button {
                            class: "w-full text-left",
                            style: "display: flex; align-items: center; gap: 8px; padding: 9px 10px; border-radius: 10px; border: 1px solid #3a2d56; background: rgba(58,45,86,0.35); color: #f7f2ff;",
                            SparkIcon { class: "w-4 h-4 text-[#e040fb]" }
                            span { "Upload Avatar" }
                        }

                        button {
                            class: "w-full text-left",
                            style: "display: flex; align-items: center; gap: 8px; padding: 9px 10px; border-radius: 10px; border: 1px solid #3a2d56; background: rgba(58,45,86,0.35); color: #7aebbe;",
                            SparkIcon { class: "w-4 h-4 text-[#7aebbe]" }
                            span { "Profile" }
                        }
                    }
                }
            }
        }
    }
}
