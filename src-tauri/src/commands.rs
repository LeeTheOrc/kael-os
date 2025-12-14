#![allow(dead_code)]

use crate::state::{ChatMessage, KaelConfig};
use crate::webdav::{WebDavClient, WebDavConfig};
use crate::version::Version;
use crate::app_scaffold::AppTemplate;
use crate::firebase::uploader::FirebaseUploader;
use crate::github::uploader::GitHubUploader;
use std::path::Path;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{command, State, Window};

#[allow(dead_code)]
#[tauri::command]
pub fn send_message(message: String, db: State<Mutex<Connection>>) -> Result<ChatMessage, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let msg = ChatMessage::new("user".to_string(), message);

    crate::db::add_message(&conn, &msg.role, &msg.text).map_err(|e| e.to_string())?;

    Ok(msg)
}

#[allow(dead_code)]
#[tauri::command]
pub fn get_chat_history(db: State<Mutex<Connection>>) -> Result<Vec<ChatMessage>, String> {
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
pub async fn initiate_oauth(
    provider: String,
    _app_handle: tauri::AppHandle,
) -> Result<String, String> {
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
            let client_id = std::env::var("GOOGLE_OAUTH_CLIENT_ID").unwrap_or_else(|_| {
                "384654392238-k02b3cvemoee9uq87pa3a3bk0gf1hbnk.apps.googleusercontent.com"
                    .to_string()
            });

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
pub async fn poll_oauth_callback(
    provider: String,
) -> Result<Option<crate::webview_oauth::OAuthResult>, String> {
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

/// Upload a local file to WebDAV using basic auth PUT
#[tauri::command]
pub async fn webdav_upload_file(
    base_url: String,
    username: String,
    password: String,
    local_path: String,
    remote_path: String,
) -> Result<String, String> {
    let client = WebDavClient::new(WebDavConfig {
        url: base_url,
        username,
        password,
    });
    let path = std::path::Path::new(&local_path);
    client
        .upload_file(path, &remote_path)
        .await
        .map_err(|e| e.to_string())?;
    Ok("WebDAV upload complete".to_string())
}

/// Get current app version from version.json
#[tauri::command]
pub fn get_version() -> Result<Version, String> {
    let version_path = Path::new("version.json");
    Version::load(version_path).map_err(|e| format!("Failed to load version: {}", e))
}

/// Bump version to next stage (alpha → beta → release)
/// Only allows forward bumps, respects semantic versioning
#[tauri::command]
pub fn bump_version(stage: String) -> Result<String, String> {
    if !["alpha", "beta", "release"].contains(&stage.as_str()) {
        return Err("Invalid stage: must be alpha, beta, or release".to_string());
    }

    let version_path = Path::new("version.json");
    let mut version = Version::load(version_path).map_err(|e| e.to_string())?;

    // Validate stage transition
    match (version.stage.as_str(), stage.as_str()) {
        ("alpha", "alpha") => {
            version.build += 1;
        }
        ("alpha", "beta") => {
            version.minor += 1;
            version.patch = 0;
            version.stage = "beta".to_string();
            version.build = 1;
        }
        ("beta", "beta") => {
            version.build += 1;
        }
        ("beta", "release") => {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
            version.stage = "release".to_string();
            version.build = 1;
        }
        ("release", "release") => {
            version.patch += 1;
            version.build += 1;
        }
        _ => {
            return Err(format!(
                "Cannot transition from {} to {}. Allowed: alpha→beta→release",
                version.stage, stage
            ))
        }
    }

    version.timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let json = serde_json::to_string_pretty(&version)
        .map_err(|e| format!("Failed to serialize version: {}", e))?;
    std::fs::write(version_path, json).map_err(|e| e.to_string())?;

    Ok(version.to_string())
}

/// Scaffold a new app with versioning system built-in
#[tauri::command]
pub fn scaffold_app(
    app_name: String,
    app_path: String,
    description: String,
) -> Result<String, String> {
    let path = std::path::Path::new(&app_path);
    AppTemplate::scaffold(&app_name, path, &description)
        .map_err(|e| format!("Failed to scaffold app: {}", e))?;
    
    Ok(format!(
        "App '{}' scaffolded at {} with versioning system v0.0.1-alpha.1",
        app_name, app_path
    ))
}

/// Upload a file to Firebase Storage using Google Cloud Storage API
#[tauri::command]
pub async fn firebase_upload_file(
    bucket: String,
    sa_json_path: String,
    local_path: String,
    remote_path: String,
) -> Result<String, String> {
    let uploader = FirebaseUploader::new(bucket, Path::new(&sa_json_path))
        .map_err(|e| format!("Firebase config error: {}", e))?;

    let local = Path::new(&local_path);
    uploader
        .upload_file(local, &remote_path)
        .await
        .map_err(|e| format!("Firebase upload failed: {}", e))
}

/// Create a GitHub release and upload assets
#[tauri::command]
pub async fn github_create_release(
    owner: String,
    repo: String,
    token: String,
    tag: String,
    name: String,
    body: String,
) -> Result<u64, String> {
    let uploader = GitHubUploader::new(owner, repo, token);
    let release = uploader
        .create_or_get_release(&tag, &name, &body)
        .await
        .map_err(|e| e.to_string())?;
    Ok(release.id)
}

/// Upload an asset to a GitHub release
#[tauri::command]
pub async fn github_upload_asset(
    owner: String,
    repo: String,
    token: String,
    release_id: u64,
    file_path: String,
    file_name: String,
) -> Result<String, String> {
    let uploader = GitHubUploader::new(owner, repo, token);
    let path = Path::new(&file_path);
    uploader
        .upload_asset(release_id, path, &file_name)
        .await
        .map_err(|e| e.to_string())
}


// ============================================================================
// Firebase Sync Commands
// ============================================================================

use crate::services::app_projects;

/// Sync all projects with Firebase (bidirectional)
#[tauri::command]
pub async fn sync_projects(
    id_token: String,
    user_id: String,
    db: State<'_, Mutex<Connection>>,
) -> Result<usize, String> {
    let conn = db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    
    app_projects::sync_projects_with_firebase(&conn, &id_token, &user_id).await
}

/// Initialize app projects table
#[tauri::command]
pub fn init_app_projects(db: State<'_, Mutex<Connection>>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    
    app_projects::init_projects_table(&conn)
        .map_err(|e| format!("Failed to init projects table: {}", e))
}

/// Save a project locally
#[tauri::command]
pub fn save_project(
    project: crate::state::AppProject,
    db: State<'_, Mutex<Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    
    app_projects::save_project_local(&conn, &project)
        .map_err(|e| format!("Failed to save project: {}", e))
}

/// Get all projects
#[tauri::command]
pub fn get_all_projects(db: State<'_, Mutex<Connection>>) -> Result<Vec<crate::state::AppProject>, String> {
    let conn = db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    
    app_projects::get_projects_local(&conn)
        .map_err(|e| format!("Failed to get projects: {}", e))
}

/// Get active projects (not archived)
#[tauri::command]
pub fn get_active_projects(db: State<'_, Mutex<Connection>>) -> Result<Vec<crate::state::AppProject>, String> {
    let conn = db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    
    app_projects::get_active_projects(&conn)
        .map_err(|e| format!("Failed to get active projects: {}", e))
}

/// Get archived projects
#[tauri::command]
pub fn get_archived_projects(db: State<'_, Mutex<Connection>>) -> Result<Vec<crate::state::AppProject>, String> {
    let conn = db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    
    app_projects::get_archived_projects(&conn)
        .map_err(|e| format!("Failed to get archived projects: {}", e))
}

/// Archive or unarchive a project
#[tauri::command]
pub fn archive_project(
    project_id: String,
    archived: bool,
    db: State<'_, Mutex<Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    
    app_projects::toggle_archive(&conn, &project_id, archived)
        .map_err(|e| format!("Failed to archive project: {}", e))
}

/// Delete a project from local database
#[tauri::command]
pub fn delete_project(
    project_id: String,
    db: State<'_, Mutex<Connection>>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    
    app_projects::delete_project_local(&conn, &project_id)
        .map_err(|e| format!("Failed to delete project: {}", e))
}

/// Delete a project from Firebase
#[tauri::command]
pub async fn delete_project_from_cloud(
    id_token: String,
    user_id: String,
    project_id: String,
) -> Result<(), String> {
    app_projects::delete_project_from_firebase(&id_token, &user_id, &project_id).await
}
