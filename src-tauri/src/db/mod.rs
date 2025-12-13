#![allow(dead_code)]

pub mod migrations;

use crate::state::ChatMessage;
use chrono::Utc;
use rusqlite::{Connection, Result as SqlResult};
use std::path::PathBuf;
use tauri::Manager;

pub fn get_db_path(app: &tauri::AppHandle) -> PathBuf {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .expect("Could not determine app data directory");

    std::fs::create_dir_all(&app_data_dir).ok();
    app_data_dir.join("kael.db")
}

pub fn init_db(app: &tauri::AppHandle) -> SqlResult<Connection> {
    let db_path = get_db_path(app);
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;

    migrations::run_migrations(&conn)?;

    log::info!("Database initialized at: {:?}", db_path);
    Ok(conn)
}

pub fn add_message(conn: &Connection, role: &str, text: &str) -> SqlResult<String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO chat_messages (id, role, text, timestamp, synced) 
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![&id, role, text, &now, 0],
    )?;

    log::debug!("Message added: {}", id);
    Ok(id)
}

pub fn get_chat_history(conn: &Connection) -> SqlResult<Vec<ChatMessage>> {
    let mut stmt = conn.prepare(
        "SELECT id, role, text, timestamp, synced FROM chat_messages ORDER BY timestamp ASC",
    )?;

    let messages = stmt.query_map([], |row| {
        Ok(ChatMessage {
            id: row.get(0)?,
            role: row.get(1)?,
            text: row.get(2)?,
            timestamp: row
                .get::<_, String>(3)?
                .parse()
                .unwrap_or_else(|_| Utc::now()),
            synced: row.get::<_, i32>(4)? != 0,
        })
    })?;

    let mut result = Vec::new();
    for msg in messages {
        result.push(msg?);
    }

    Ok(result)
}
