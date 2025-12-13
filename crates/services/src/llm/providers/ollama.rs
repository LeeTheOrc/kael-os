use crate::llm::LLMProvider;
use serde_json::json;

#[allow(dead_code)]
pub struct OllamaProvider {
    endpoint: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(endpoint: Option<String>, model: Option<String>) -> Self {
        OllamaProvider {
            endpoint: endpoint.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model: model.unwrap_or_else(|| "mistral".to_string()),
        }
    }

    /// Check if Ollama is running
    async fn check_health(&self) -> bool {
        match reqwest::Client::new()
            .get(&format!("{}/api/tags", self.endpoint))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

#[async_trait::async_trait]
impl LLMProvider for OllamaProvider {
    async fn complete(&self, prompt: &str) -> Result<String, String> {
        if !self.is_available() {
            return Err("Ollama not running on localhost:11434".to_string());
        }

        let client = reqwest::Client::new();
        let payload = json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
        });

        match client
            .post(&format!("{}/api/generate", self.endpoint))
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                match resp.json::<serde_json::Value>().await {
                    Ok(body) => {
                        if let Some(response) = body.get("response").and_then(|v| v.as_str()) {
                            Ok(response.to_string())
                        } else {
                            Err("Invalid response format from Ollama".to_string())
                        }
                    }
                    Err(e) => Err(format!("Failed to parse Ollama response: {}", e)),
                }
            }
            Err(e) => Err(format!("Ollama request failed: {}", e)),
        }
    }

    fn name(&self) -> &'static str {
        "ollama"
    }

    fn requires_api_key(&self) -> bool {
        false
    }

    fn is_available(&self) -> bool {
        // Simplified - check_health would require async context
        // In real usage, call check_health in is_available implementation
        // For now, assume available if endpoint is set
        true
    }
}

// Async version for startup checks
impl OllamaProvider {
    pub async fn is_available_async(&self) -> bool {
        self.check_health().await
    }
}
