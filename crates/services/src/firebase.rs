/// Firebase integration for storing provider configs and API keys
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::llm::ProviderConfig;

/// Encrypted provider configuration stored in Firebase
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FirebaseProviderConfig {
    pub user_id: String,
    pub providers: Vec<ProviderConfig>,
    pub default_priority_order: Vec<String>, // ["ollama", "mistral", "gemini", ...]
    pub created_at: String,
    pub updated_at: String,
}

impl FirebaseProviderConfig {
    pub fn new(user_id: String) -> Self {
        FirebaseProviderConfig {
            user_id,
            providers: vec![],
            default_priority_order: vec!["ollama".to_string(), "mistral".to_string()],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Add a provider to the config
    pub fn add_provider(&mut self, mut config: ProviderConfig) {
        // Set priority based on position in default order
        if let Some(pos) = self.default_priority_order.iter().position(|p| p == &config.name) {
            config.priority = (pos + 1) as u32;
        } else {
            config.priority = (self.providers.len() + 1) as u32;
        }
        self.providers.push(config);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Update a provider's config
    pub fn update_provider(&mut self, name: &str, enabled: bool, api_key: Option<String>) {
        if let Some(provider) = self.providers.iter_mut().find(|p| p.name == name) {
            provider.enabled = enabled;
            if let Some(key) = api_key {
                provider.api_key = Some(key);
            }
            self.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Reorder providers by priority
    pub fn reorder_providers(&mut self, new_order: Vec<String>) {
        self.default_priority_order = new_order.clone();
        for (i, name) in new_order.iter().enumerate() {
            if let Some(provider) = self.providers.iter_mut().find(|p| &p.name == name) {
                provider.priority = (i + 1) as u32;
            }
        }
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Get enabled providers sorted by priority
    pub fn enabled_providers(&self) -> Vec<ProviderConfig> {
        let mut enabled: Vec<_> = self.providers.iter().filter(|p| p.enabled).cloned().collect();
        enabled.sort_by_key(|p| p.priority);
        enabled
    }
}

/// Storage interface for Firebase operations
/// In production, this would use Firebase Admin SDK
/// For now, this is a placeholder for the interface
#[async_trait::async_trait]
pub trait ProviderConfigStore: Send + Sync {
    /// Load user's provider config from Firebase
    async fn load_config(&self, user_id: &str) -> Result<FirebaseProviderConfig, String>;

    /// Save user's provider config to Firebase
    async fn save_config(&self, config: &FirebaseProviderConfig) -> Result<(), String>;

    /// Delete a provider from config
    async fn delete_provider(&self, user_id: &str, provider_name: &str) -> Result<(), String>;

    /// Update provider priority order
    async fn update_priority_order(
        &self,
        user_id: &str,
        order: Vec<String>,
    ) -> Result<(), String>;
}

/// Mock implementation for local development
/// In production, this would connect to Firebase Firestore
pub struct MockProviderConfigStore {
    configs: HashMap<String, FirebaseProviderConfig>,
}

impl MockProviderConfigStore {
    pub fn new() -> Self {
        MockProviderConfigStore {
            configs: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl ProviderConfigStore for MockProviderConfigStore {
    async fn load_config(&self, user_id: &str) -> Result<FirebaseProviderConfig, String> {
        Ok(self
            .configs
            .get(user_id)
            .cloned()
            .unwrap_or_else(|| FirebaseProviderConfig::new(user_id.to_string())))
    }

    async fn save_config(&self, config: &FirebaseProviderConfig) -> Result<(), String> {
        // In production, this would save to Firebase Firestore
        println!(
            "[MOCK] Saved config for user {}: {} providers",
            config.user_id,
            config.providers.len()
        );
        Ok(())
    }

    async fn delete_provider(&self, user_id: &str, provider_name: &str) -> Result<(), String> {
        println!(
            "[MOCK] Deleted provider '{}' for user {}",
            provider_name, user_id
        );
        Ok(())
    }

    async fn update_priority_order(
        &self,
        user_id: &str,
        order: Vec<String>,
    ) -> Result<(), String> {
        println!(
            "[MOCK] Updated priority order for user {}: {:?}",
            user_id, order
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firebase_config_creation() {
        let config = FirebaseProviderConfig::new("user123".to_string());
        assert_eq!(config.user_id, "user123");
        assert_eq!(config.default_priority_order.len(), 2);
    }

    #[test]
    fn test_add_provider() {
        let mut config = FirebaseProviderConfig::new("user123".to_string());
        let provider = ProviderConfig {
            name: "gemini".to_string(),
            enabled: true,
            priority: 0,
            api_key: Some("test_key".to_string()),
            custom_config: HashMap::new(),
        };
        config.add_provider(provider);
        assert_eq!(config.providers.len(), 1);
    }

    #[test]
    fn test_reorder_providers() {
        let mut config = FirebaseProviderConfig::new("user123".to_string());
        let providers = vec![
            ProviderConfig {
                name: "ollama".to_string(),
                enabled: true,
                priority: 1,
                api_key: None,
                custom_config: HashMap::new(),
            },
            ProviderConfig {
                name: "mistral".to_string(),
                enabled: true,
                priority: 2,
                api_key: Some("key".to_string()),
                custom_config: HashMap::new(),
            },
        ];
        config.providers = providers;

        config.reorder_providers(vec!["mistral".to_string(), "ollama".to_string()]);

        assert_eq!(config.providers[0].priority, 2);
        assert_eq!(config.providers[1].priority, 1);
    }
}
