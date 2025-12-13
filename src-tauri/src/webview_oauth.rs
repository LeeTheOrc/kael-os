#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthResult {
    pub provider: String,
    pub code: String,
    pub state: String,
}

// Store the OAuth result in a thread-safe way
pub static OAUTH_RESULT: once_cell::sync::Lazy<Arc<Mutex<Option<OAuthResult>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// Extract OAuth code from callback URL
pub fn extract_oauth_code(url: &str) -> Option<OAuthResult> {
    // Try to parse the URL
    let parsed = Url::parse(url).ok()?;
    
    // Check if this is a callback URL
    if !parsed.path().contains("auth") || !parsed.path().contains("callback") {
        return None;
    }
    
    // Extract the provider from the path (e.g., /auth/google/callback)
    let path_parts: Vec<&str> = parsed.path().split('/').collect();
    let provider = if path_parts.len() >= 3 {
        path_parts[2].to_string()
    } else {
        return None;
    };
    
    // Extract code from query parameters
    let code = parsed
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, val)| val.into_owned())?;
    
    // Extract state if available
    let state = parsed
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, val)| val.into_owned())
        .unwrap_or_default();
    
    Some(OAuthResult {
        provider,
        code,
        state,
    })
}

/// Store OAuth result for retrieval
pub async fn store_oauth_result(result: OAuthResult) {
    let mut oauth_result = OAUTH_RESULT.lock().await;
    *oauth_result = Some(result);
}

/// Retrieve and clear OAuth result
pub async fn get_and_clear_oauth_result() -> Option<OAuthResult> {
    let mut oauth_result = OAUTH_RESULT.lock().await;
    oauth_result.take()
}

/// Get OAuth result from the OAuth server
pub async fn get_oauth_result_from_server(provider: &str) -> Option<OAuthResult> {
    if let Some(callback) = crate::oauth_server::OAUTH_SERVER.get_callback(provider).await {
        if let Some(code) = callback.code {
            let result = OAuthResult {
                provider: provider.to_string(),
                code,
                state: String::new(),
            };
            // Clear the callback from the server
            crate::oauth_server::OAUTH_SERVER.clear_callback(provider).await;
            return Some(result);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_google_code() {
        let url = "http://localhost:5173/auth/google/callback?code=test123&scope=email";
        let result = extract_oauth_code(url).unwrap();
        assert_eq!(result.provider, "google");
        assert_eq!(result.code, "test123");
    }

    #[test]
    fn test_extract_github_code() {
        let url = "http://localhost:5173/auth/github/callback?code=github_code_123&state=xyz";
        let result = extract_oauth_code(url).unwrap();
        assert_eq!(result.provider, "github");
        assert_eq!(result.code, "github_code_123");
        assert_eq!(result.state, "xyz");
    }
}
