// src-tauri/src/main.rs
mod auth;
mod components;
mod llm;
mod terminal;
mod firebase;
mod commands;
mod db;
mod state;

use dioxus::prelude::*;
use crate::components::app::App;

fn main() {
    // Load .env.local file
    dotenv::from_filename(".env.local").ok();

    // Initialize logger
    env_logger::init();

    let _db_connection = db::initialize_database().expect("Failed to initialize database");

    // Launch the Dioxus Desktop application
    dioxus_desktop::launch(app);
}

fn app() -> Element {
    rsx! {
        App { }
    }
}
