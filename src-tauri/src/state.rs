#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppStatus {
    #[serde(rename = "making")]
    Making,
    #[serde(rename = "want")]
    Want,
    #[serde(rename = "testing")]
    Testing,
    #[serde(rename = "done")]
    Done,
}

impl AppStatus {
    pub fn color(&self) -> &'static str {
        match self {
            AppStatus::Making => "#e040fb",  // Magenta - actively working
            AppStatus::Want => "#ffcc00",    // Yellow - planned/wanted
            AppStatus::Testing => "#7aebbe", // Cyan - beta testing
            AppStatus::Done => "#4ecca3",    // Green - completed
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AppStatus::Making => "Making",
            AppStatus::Want => "Want to Make",
            AppStatus::Testing => "Testing",
            AppStatus::Done => "Done",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppProject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: AppStatus,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub archived: bool,
}

impl AppProject {
    pub fn new(name: String, description: String, status: AppStatus) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            status,
            version: "0.0.1-alpha.1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            archived: false,
        }
    }
}
