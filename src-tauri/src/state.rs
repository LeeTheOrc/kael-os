#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String, // "user" or "model"
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub synced: bool,
}

impl ChatMessage {
    pub fn new(role: String, text: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            text,
            timestamp: Utc::now(),
            synced: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaelConfig {
    pub personality_level: u8,
    pub cloud_enabled: bool,
    pub local_core_enabled: bool,
    pub auto_sync: bool,
}

impl Default for KaelConfig {
    fn default() -> Self {
        Self {
            personality_level: 7,
            cloud_enabled: true,
            local_core_enabled: true,
            auto_sync: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
