/// Local SQLite-backed storage for messages, sessions, and cached API responses.
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub author: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

pub struct StorageManager {
    db_path: String,
}

impl StorageManager {
    pub fn new(db_path: &str) -> Result<Self> {
        let manager = StorageManager {
            db_path: db_path.to_string(),
        };
        manager.init_db()?;
        Ok(manager)
    }

    fn init_db(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS chat_messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                author TEXT NOT NULL,
                text TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    pub fn save_message(&self, session_id: &str, author: &str, text: &str) -> Result<ChatMessage> {
        let conn = Connection::open(&self.db_path)?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        conn.execute(
            "INSERT INTO chat_messages (id, session_id, author, text, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&id, session_id, author, text, now.to_rfc3339()),
        )?;

        Ok(ChatMessage {
            id,
            session_id: session_id.to_string(),
            author: author.to_string(),
            text: text.to_string(),
            created_at: now,
        })
    }

    pub fn load_session(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, author, text, created_at FROM chat_messages WHERE session_id = ?1 ORDER BY created_at",
        )?;

        let messages = stmt
            .query_map([session_id], |row| {
                Ok(ChatMessage {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    author: row.get(2)?,
                    text: row.get(3)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }
}
