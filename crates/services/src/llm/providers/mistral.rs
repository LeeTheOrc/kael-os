use crate::llm::LLMProvider;
use serde_json::json;

#[allow(dead_code)]
pub struct MistralProvider {
    api_key: String,
    model: String,
}

impl MistralProvider {
    pub fn new(api_key: String) -> Self {
        MistralProvider {
            api_key,
            model: "mistral-small-latest".to_string(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

#[async_trait::async_trait]
impl LLMProvider for MistralProvider {
    async fn complete(&self, prompt: &str) -> Result<String, String> {
        if self.api_key.is_empty() {
            return Err("Mistral API key not configured".to_string());
        }

        let client = reqwest::Client::new();
        let payload = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7,
        });

        match client
            .post("https://api.mistral.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                match resp.json::<serde_json::Value>().await {
                    Ok(body) => {
                        if let Some(content) = body
                            .get("choices")
                            .and_then(|v| v.get(0))
                            .and_then(|v| v.get("message"))
                            .and_then(|v| v.get("content"))
                            .and_then(|v| v.as_str())
                        {
                            Ok(content.to_string())
                        } else {
                            Err("Invalid response format from Mistral".to_string())
                        }
                    }
                    Err(e) => Err(format!("Failed to parse Mistral response: {}", e)),
                }
            }
            Err(e) => Err(format!("Mistral request failed: {}", e)),
        }
    }

    fn name(&self) -> &'static str {
        "mistral"
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}
