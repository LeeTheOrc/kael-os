use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::system_context::{SystemContext, CommandTranslator};

pub mod providers;

/// Core LLM Provider trait - all providers must implement this
#[async_trait::async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generate a response for the given prompt
    async fn complete(&self, prompt: &str) -> Result<String, String>;
    
    /// Provider identifier (e.g., "ollama", "gemini", "copilot")
    fn name(&self) -> &'static str;
    
    /// Whether this provider requires an API key
    fn requires_api_key(&self) -> bool;
    
    /// Check if provider is available/configured
    fn is_available(&self) -> bool;
}

/// User's LLM provider configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub enabled: bool,
    pub priority: u32,
    pub api_key: Option<String>, // encrypted in Firebase
    pub custom_config: HashMap<String, String>, // provider-specific settings
}

/// Service for managing multiple LLM providers with fallback
pub struct LLMService {
    providers: Vec<(ProviderConfig, Box<dyn LLMProvider>)>,
    system_context: SystemContext,
}

impl LLMService {
    /// Create new LLM service with configured providers
    pub fn new(configs: Vec<(ProviderConfig, Box<dyn LLMProvider>)>) -> Self {
        let mut providers = configs;
        // Sort by priority (lower number = higher priority)
        providers.sort_by_key(|(config, _)| config.priority);
        
        LLMService {
            providers,
            system_context: SystemContext::arch_linux(),
        }
    }

    /// Try each enabled provider in priority order until one succeeds
    pub async fn complete(&self, user_prompt: &str) -> Result<(String, String), String> {
        // Build full prompt with system context
        let full_prompt = format!(
            "{}\n\nUser Question: {}",
            self.system_context.build_system_prompt(),
            user_prompt
        );

        for (config, provider) in &self.providers {
            if !config.enabled || !provider.is_available() {
                continue;
            }

            match provider.complete(&full_prompt).await {
                Ok(response) => {
                    // Post-process: translate commands if needed
                    let translated = CommandTranslator::translate(&response);
                    return Ok((translated, provider.name().to_string()));
                }
                Err(e) => {
                    eprintln!(
                        "LLM provider {} failed: {}. Trying next...",
                        provider.name(),
                        e
                    );
                    // Continue to next provider
                }
            }
        }

        Err("All LLM providers failed or disabled".to_string())
    }

    /// List all available providers
    pub fn available_providers(&self) -> Vec<&str> {
        self.providers
            .iter()
            .filter(|(config, provider)| config.enabled && provider.is_available())
            .map(|(_, provider)| provider.name())
            .collect()
    }

    /// Get provider status
    pub fn provider_status(&self) -> Vec<(String, bool, u32)> {
        self.providers
            .iter()
            .map(|(config, provider)| {
                (
                    provider.name().to_string(),
                    config.enabled && provider.is_available(),
                    config.priority,
                )
            })
            .collect()
    }

    /// Get system context
    pub fn system_context(&self) -> &SystemContext {
        &self.system_context
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    struct MockProvider {
        response: String,
    }

    #[async_trait::async_trait]
    impl LLMProvider for MockProvider {
        async fn complete(&self, _prompt: &str) -> Result<String, String> {
            Ok(self.response.clone())
        }

        fn name(&self) -> &'static str {
            "mock"
        }

        fn requires_api_key(&self) -> bool {
            false
        }

        fn is_available(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_llm_service_fallback() {
        let configs = vec![
            (
                ProviderConfig {
                    name: "mock1".to_string(),
                    enabled: true,
                    priority: 1,
                    api_key: None,
                    custom_config: HashMap::new(),
                },
                Box::new(MockProvider {
                    response: "Mock 1 response".to_string(),
                }) as Box<dyn LLMProvider>,
            ),
            (
                ProviderConfig {
                    name: "mock2".to_string(),
                    enabled: true,
                    priority: 2,
                    api_key: None,
                    custom_config: HashMap::new(),
                },
                Box::new(MockProvider {
                    response: "Mock 2 response".to_string(),
                }) as Box<dyn LLMProvider>,
            ),
        ];

        let service = LLMService::new(configs);
        let (response, provider) = service.complete("test").await.unwrap();

        assert_eq!(response, "Mock 1 response");
        assert_eq!(provider, "mock");
    }
}
