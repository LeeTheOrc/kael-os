use crate::state::{ChatMessage, KaelConfig};
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{command, State, Window};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[tauri::command]
pub fn send_message(
    message: String,
    db: State<Mutex<Connection>>,
) -> Result<ChatMessage, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let msg = ChatMessage::new("user".to_string(), message);
    
    crate::db::add_message(&conn, &msg.role, &msg.text)
        .map_err(|e| e.to_string())?;
    
    Ok(msg)
}

#[allow(dead_code)]
#[tauri::command]
pub fn get_chat_history(
    db: State<Mutex<Connection>>,
) -> Result<Vec<ChatMessage>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    crate::db::get_chat_history(&conn).map_err(|e| e.to_string())
}

#[allow(dead_code)]
#[tauri::command]
pub fn execute_script(script: String) -> Result<String, String> {
    log::info!("Executing script: {}", script);
    Ok(format!("Script executed: {}", script))
}

#[allow(dead_code)]
#[tauri::command]
pub fn execute_terminal_command(command: String) -> Result<String, String> {
    log::info!("Terminal command: {}", command);
    Ok(format!("Command output: {}", command))
}

#[allow(dead_code)]
#[tauri::command]
pub fn get_kael_config() -> Result<KaelConfig, String> {
    Ok(KaelConfig::default())
}

#[allow(dead_code)]
#[tauri::command]
pub fn save_kael_config(config: KaelConfig) -> Result<(), String> {
    log::info!("Saving Kael config: {:?}", config);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OauthResult {
    pub id_token: String,
    pub email: String,
    pub name: String,
    pub photo_url: Option<String>,
}

#[derive(Debug, Default)]
pub struct OauthResultState(pub Mutex<Option<OauthResult>>);

#[allow(dead_code)]
#[command]
pub fn oauth_result(
    _window: Window,
    state: State<'_, OauthResultState>,
    result: OauthResult,
) -> Result<(), String> {
    *state.0.lock().unwrap() = Some(result);
    // Optionally emit an event to the window if you prefer a push-based approach
    // window.emit("oauth_success", "User data received").unwrap();
    Ok(())
}

#[allow(dead_code)]
#[command]
pub fn get_oauth_result(state: State<'_, OauthResultState>) -> Result<Option<OauthResult>, String> {
    let mut result_state = state.0.lock().unwrap();
    if let Some(result) = result_state.take() {
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn initiate_oauth(provider: String, _app_handle: tauri::AppHandle) -> Result<String, String> {
    log::info!("Initiating OAuth for provider: {}", provider);
    
    // Get Firebase config from environment
    let api_key = std::env::var("VITE_FIREBASE_API_KEY")
        .map_err(|_| "VITE_FIREBASE_API_KEY not set in environment")?;
    let auth_domain = std::env::var("VITE_FIREBASE_AUTH_DOMAIN")
        .map_err(|_| "VITE_FIREBASE_AUTH_DOMAIN not set in environment")?;
    
    // Build Firebase OAuth URL
    let redirect_uri = "http://localhost:5173/__/auth/handler"; // Tauri deep link
    let _provider_id = match provider.as_str() {
        "google" => "google.com",
        "github" => "github.com",
        _ => return Err("Invalid provider".to_string()),
    };
    
    let _auth_url = format!(
        "https://{}/v1/accounts:signInWithIdp?key={}",
        auth_domain, api_key
    );
    
    // Build a web URL that can be loaded inside our in-app webview
    // Note: the frontend should open this URL inside the embedded webview window.
    let oauth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile",
        api_key, redirect_uri
    );

    log::info!("OAuth URL (in-app): {}", oauth_url);
    Ok(oauth_url)
}

/// Get OAuth URL for in-app WebView login
#[allow(dead_code)]
#[tauri::command]
pub fn get_oauth_url(provider: String) -> Result<String, String> {
    let oauth_url = match provider.as_str() {
        "google" => {
            let client_id = std::env::var("GOOGLE_OAUTH_CLIENT_ID")
                .unwrap_or_else(|_| "384654392238-k02b3cvemoee9uq87pa3a3bk0gf1hbnk.apps.googleusercontent.com".to_string());
            
            format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri=http://localhost:5173/auth/google/callback&response_type=code&scope=email%20profile%20openid",
                client_id
            )
        }
        "github" => {
            let client_id = std::env::var("GITHUB_OAUTH_CLIENT_ID")
                .unwrap_or_else(|_| "Ov23liqnLH8iIZOZ8sMT".to_string());
            
            format!(
                "https://github.com/login/oauth/authorize?client_id={}&redirect_uri=http://localhost:5173/auth/github/callback&scope=user:email",
                client_id
            )
        }
        _ => return Err("Invalid provider".to_string()),
    };
    
    log::info!("OAuth URL for {}: {}", provider, oauth_url);
    Ok(oauth_url)
}

/// Store OAuth result when callback is detected
#[allow(dead_code)]
#[tauri::command]
pub async fn store_oauth_code(
    provider: String,
    code: String,
    state: Option<String>,
) -> Result<(), String> {
    let oauth_result = crate::webview_oauth::OAuthResult {
        provider,
        code,
        state: state.unwrap_or_default(),
    };
    
    crate::webview_oauth::store_oauth_result(oauth_result).await;
    log::info!("Stored OAuth result");
    Ok(())
}

/// Get stored OAuth code (called after modal closes)
#[allow(dead_code)]
#[tauri::command]
pub async fn get_stored_oauth_code() -> Result<Option<crate::webview_oauth::OAuthResult>, String> {
    let result = crate::webview_oauth::get_and_clear_oauth_result().await;
    Ok(result)
}

/// Poll for OAuth callback result from the server
#[allow(dead_code)]
#[tauri::command]
pub async fn poll_oauth_callback(provider: String) -> Result<Option<crate::webview_oauth::OAuthResult>, String> {
    // First check if result is already stored locally
    if let Some(result) = crate::webview_oauth::get_and_clear_oauth_result().await {
        return Ok(Some(result));
    }
    
    // Then check the OAuth server
    let result = crate::webview_oauth::get_oauth_result_from_server(&provider).await;
    Ok(result)
}

/// Exchange OAuth code for token (called with the code from WebView)
#[allow(dead_code)]
#[tauri::command]
pub async fn exchange_oauth_token(
    provider: String,
    code: String,
) -> Result<crate::auth::User, String> {
    match provider.as_str() {
        "google" => crate::auth::exchange_google_code_for_token(&code).await,
        "github" => crate::auth::exchange_github_code_for_token(&code).await,
        _ => Err("Invalid provider".to_string()),
    }
}

