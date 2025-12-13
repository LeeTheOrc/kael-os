// src-tauri/src/components/oauth_modal.rs
// 
// Simple OAuth Modal Component (placeholder)
// 
// The actual OAuth flow should be implemented in the frontend JavaScript
// since Dioxus Desktop uses a webview which has direct access to the DOM
// and can better handle iframe communication.
//
// For now, this is a placeholder component that shows the modal structure.
// The real implementation will be in HTML/JS in the public folder.

use dioxus::prelude::*;
use crate::auth::User;

#[derive(Props, Clone, PartialEq)]
pub struct OAuthModalProps {
    pub provider: String,
    pub is_open: Signal<bool>,
    pub on_success: EventHandler<User>,
    pub on_error: EventHandler<String>,
}

#[allow(non_snake_case)]
pub fn OAuthModal(props: OAuthModalProps) -> Element {
    let provider_clone = props.provider.clone();

    if !props.is_open() {
        return rsx! { };
    }

    rsx! {
        div {
            class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50",
            onclick: move |_| {
                props.is_open.set(false);
            },
            
            div {
                class: "bg-gradient-to-br from-purple-900 via-indigo-900 to-black rounded-2xl border border-purple-700 shadow-2xl w-full max-w-md h-96 flex flex-col",
                onclick: move |e| {
                    e.stop_propagation();
                },

                // Header
                div {
                    class: "px-6 py-4 border-b border-purple-700 bg-black/30 flex justify-between items-center",
                    h2 {
                        class: "text-white font-bold text-lg",
                        match provider_clone.as_str() {
                            "google" => "Sign in with Google",
                            "github" => "Sign in with GitHub",
                            _ => "Sign in",
                        }
                    }
                    button {
                        class: "text-gray-400 hover:text-white text-2xl font-bold",
                        onclick: move |_| {
                            props.is_open.set(false);
                        },
                        "×"
                    }
                }

                // Body
                div {
                    class: "flex-1 flex items-center justify-center text-gray-300",
                    p { "OAuth modal - implement in frontend JS/HTML" }
                }
            }
        }
    }
}
