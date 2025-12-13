mod auth;
mod components;
mod terminal;
mod llm;
mod firebase;
mod commands;
mod db;
mod state;
mod oauth_server;
mod webview_oauth;
mod crypto;
mod gpg;
mod ssl;
mod updater;
mod webdav;

use dioxus::prelude::*;
use crate::components::app::App;

fn main() {
    dotenv::from_filename(".env.local").ok();
    env_logger::init();

    // Initialize OAuth callback server in background
    // This spawns the server in a separate thread with its own Tokio runtime
    oauth_server::start_oauth_server();

    // Launch using Dioxus Desktop launcher
    dioxus_desktop::launch::launch(app, Default::default(), Default::default());
}

fn app() -> Element {
    rsx! { App { } }
}

