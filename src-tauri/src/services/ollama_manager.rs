// Kael-OS Ollama Manager Service
// Handles Ollama startup checks, graceful degradation, and auto-detection

use std::process::Command;

/// Status of Ollama installation and models
#[derive(Debug, Clone, PartialEq)]
pub enum OllamaStatus {
    /// All systems go - Ollama running with models available
    Ready,
    
    /// Ollama installed but not running
    NotRunning,
    
    /// Ollama not installed at all
    NotInstalled,
    
    /// Ollama running but no AI models downloaded
    MissingModels,
    
    /// Unexpected error
    Error(String),
}

impl OllamaStatus {
    pub fn is_ready(&self) -> bool {
        matches!(self, OllamaStatus::Ready)
    }
    
    pub fn user_message(&self) -> String {
        match self {
            OllamaStatus::Ready => {
                "✅ Local AI: Ready (llama:latest, phi3)".to_string()
            }
            OllamaStatus::NotRunning => {
                "⚠️  Local AI: Not running\nStart with: ollama serve".to_string()
            }
            OllamaStatus::NotInstalled => {
                "⚠️  Local AI: Not installed\nInstall from: https://ollama.ai\nThen run: ollama serve".to_string()
            }
            OllamaStatus::MissingModels => {
                "⚠️  Local AI: No models found\nDownload: ollama pull llama:latest phi3".to_string()
            }
            OllamaStatus::Error(e) => {
                format!("❌ Local AI Error: {}", e)
            }
        }
    }
}

/// Check if Ollama is installed
pub fn is_ollama_installed() -> bool {
    Command::new("which")
        .arg("ollama")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if Ollama service is running
pub async fn ping_ollama() -> bool {
    // Try to reach Ollama's API endpoint
    match reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
    {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Get list of available models from Ollama
pub async fn get_available_models() -> Result<Vec<String>, String> {
    match reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<serde_json::Value>().await {
                Ok(data) => {
                    let models: Vec<String> = data
                        .get("models")
                        .and_then(|m| m.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|item| {
                                    item.get("name").and_then(|n| n.as_str()).map(String::from)
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    
                    Ok(models)
                }
                Err(e) => Err(format!("Failed to parse Ollama response: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to connect to Ollama: {}", e)),
    }
}

/// Check if specific models are available
pub async fn has_models(required: &[&str]) -> bool {
    match get_available_models().await {
        Ok(available) => {
            required.iter().all(|model| {
                available.iter().any(|avail| avail.contains(model))
            })
        }
        Err(_) => false,
    }
}

/// Comprehensive Ollama setup check
pub async fn check_ollama_setup() -> OllamaStatus {
    // First check if Ollama is even installed
    if !is_ollama_installed() {
        return OllamaStatus::NotInstalled;
    }
    
    // Check if service is running
    if !ping_ollama().await {
        return OllamaStatus::NotRunning;
    }
    
    // Check if required models are available
    if !has_models(&["llama", "phi"]).await {
        return OllamaStatus::MissingModels;
    }
    
    OllamaStatus::Ready
}

/// Try to start Ollama service
pub fn start_ollama_service() -> Result<(), String> {
    // Try systemctl (preferred)
    if let Ok(output) = Command::new("systemctl")
        .args(&["--user", "start", "ollama.service"])
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }
    
    // Try with sudo
    if let Ok(output) = Command::new("sudo")
        .args(&["systemctl", "start", "ollama.service"])
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }
    
    // Try direct command
    if let Ok(_) = Command::new("nohup")
        .arg("ollama")
        .arg("serve")
        .spawn()
    {
        return Ok(());
    }
    
    Err("Could not start Ollama service".to_string())
}

/// Log Ollama status for debugging
pub fn log_ollama_status(status: &OllamaStatus) {
    log::info!("Ollama Status: {:?}", status);
    log::info!("User Message: {}", status.user_message());
}

/// Ensure Ollama is running, start if necessary
pub async fn ensure_ollama_running() {
    // First check if Ollama is installed
    if !is_ollama_installed() {
        log::warn!("Ollama not installed, skipping auto-start");
        return;
    }
    
    // Check if already running
    if ping_ollama().await {
        log::debug!("Ollama already running");
        return;
    }
    
    // Try to start it
    log::info!("Ollama not running, attempting to start...");
    match start_ollama_service() {
        Ok(_) => {
            log::info!("Started Ollama service");
            // Give it time to fully boot
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
        Err(e) => {
            log::warn!("Failed to start Ollama: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ollama_status_is_ready() {
        assert!(OllamaStatus::Ready.is_ready());
        assert!(!OllamaStatus::NotInstalled.is_ready());
    }
    
    #[test]
    fn test_user_messages_not_empty() {
        assert!(!OllamaStatus::Ready.user_message().is_empty());
        assert!(!OllamaStatus::NotInstalled.user_message().is_empty());
        assert!(!OllamaStatus::NotRunning.user_message().is_empty());
    }
}
