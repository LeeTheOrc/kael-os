#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use base64::{engine::general_purpose, Engine};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub uid: String,
    pub email: String,
    pub name: String,
    pub photo_url: Option<String>,
    pub id_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
}

// Simple XOR + base64 helper using the user's id_token as key material.
fn xor_encrypt(key: &str, data: &[u8]) -> String {
    let k = key.as_bytes();
    let enc: Vec<u8> = data
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ k[i % k.len()])
        .collect();
    general_purpose::STANDARD.encode(enc)
}

fn xor_decrypt(key: &str, data_b64: &str) -> Option<String> {
    let k = key.as_bytes();
    let raw = general_purpose::STANDARD.decode(data_b64).ok()?;
    let dec: Vec<u8> = raw
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ k[i % k.len()])
        .collect();
    String::from_utf8(dec).ok()
}

pub fn encrypt_secret(user: &User, plaintext: &str) -> String {
    xor_encrypt(&user.id_token, plaintext.as_bytes())
}

pub fn decrypt_secret(user: &User, ciphertext_b64: &str) -> Option<String> {
    xor_decrypt(&user.id_token, ciphertext_b64)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedKey {
    pub provider: String,        // "openai", "anthropic", "local", etc.
    pub encrypted_key: String,   // base64-encoded encrypted key
    pub created_at: String,
}

pub struct AuthService {
    current_user: Arc<Mutex<Option<User>>>,
    provider_keys: Arc<Mutex<Vec<EncryptedKey>>>,
}

impl AuthService {
    pub fn new() -> Self {
        // Try to load user from localStorage
        let stored_user = Self::load_from_storage();
        Self {
            current_user: Arc::new(Mutex::new(stored_user)),
            provider_keys: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn load_from_storage() -> Option<User> {
        if let Ok(user_json) = std::fs::read_to_string("/tmp/kael_user.json") {
            serde_json::from_str(&user_json).ok()
        } else {
            None
        }
    }

    fn save_to_storage(user: &User) {
        if let Ok(json) = serde_json::to_string(user) {
            let _ = std::fs::write("/tmp/kael_user.json", json);
        }
    }

    pub fn set_user(&self, user: User) {
        if let Ok(mut u) = self.current_user.lock() {
            Self::save_to_storage(&user);
            *u = Some(user);
        }
    }

    pub fn get_user(&self) -> Option<User> {
        if let Ok(u) = self.current_user.lock() {
            u.clone()
        } else {
            None
        }
    }

    pub fn logout(&self) {
        if let Ok(mut u) = self.current_user.lock() {
            let _ = std::fs::remove_file("/tmp/kael_user.json");
            *u = None;
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.get_user().is_some()
    }

    pub fn store_encrypted_key(&self, key: EncryptedKey) -> Result<(), String> {
        if let Ok(mut keys) = self.provider_keys.lock() {
            // Remove old key for this provider if exists
            keys.retain(|k| k.provider != key.provider);
            keys.push(key);
            Ok(())
        } else {
            Err("Failed to lock keys storage".to_string())
        }
    }

    pub fn get_encrypted_key(&self, provider: &str) -> Option<EncryptedKey> {
        if let Ok(keys) = self.provider_keys.lock() {
            keys.iter().find(|k| k.provider == provider).cloned()
        } else {
            None
        }
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AuthService {
    fn clone(&self) -> Self {
        Self {
            current_user: Arc::clone(&self.current_user),
            provider_keys: Arc::clone(&self.provider_keys),
        }
    }
}

// Firebase OAuth endpoints
pub const FIREBASE_OAUTH_GOOGLE: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const FIREBASE_OAUTH_GITHUB: &str = "https://github.com/login/oauth/authorize";

#[derive(Debug, Serialize, Deserialize)]
pub struct FirebaseAuthResponse {
    pub id_token: String,
    pub local_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct SignInResponse {
    idToken: String,
    refreshToken: String,
    expiresIn: String,
    localId: String,
    email: String,
    displayName: Option<String>,
    photoUrl: Option<String>,
}

/// Call Firebase REST `signInWithIdp` to turn a third-party credential into a Firebase session.
async fn firebase_sign_in_with_idp(post_body: String, request_uri: &str, api_key: &str) -> Result<User, String> {
    let url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signInWithIdp?key={}",
        api_key
    );

    let payload = serde_json::json!({
        "postBody": post_body,
        "requestUri": request_uri,
        "returnSecureToken": true,
        "returnIdpCredential": true,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            log::error!("Firebase IdP exchange failed: {}", e);
            format!("Firebase IdP exchange failed: {}", e)
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        log::error!("Firebase IdP HTTP {}: {}", status, body);
        return Err(format!("Firebase IdP error {}: {}", status.as_u16(), body));
    }

    let data: SignInResponse = resp.json().await.map_err(|e| {
        log::error!("Firebase IdP parse error: {}", e);
        format!("Failed to parse Firebase IdP response: {}", e)
    })?;

    log::info!("Firebase IdP response: displayName={:?}, photoUrl={:?}", data.displayName, data.photoUrl);

    let expires_in = data
        .expiresIn
        .parse::<i64>()
        .ok()
        .map(|sec| chrono::Utc::now().timestamp() + sec);

    Ok(User {
        uid: data.localId,
        email: data.email.clone(),
        name: data
            .displayName
            .clone()
            .unwrap_or_else(|| data.email.split('@').next().unwrap_or("User").to_string()),
        photo_url: data.photoUrl.clone(),
        id_token: data.idToken,
        refresh_token: Some(data.refreshToken),
        expires_in,
    })
}

pub async fn firebase_sign_in_email_password(email: &str, password: &str) -> Result<User, String> {
    let api_key = std::env::var("VITE_FIREBASE_API_KEY")
        .map_err(|_| {
            log::error!("Missing VITE_FIREBASE_API_KEY env var");
            "Missing VITE_FIREBASE_API_KEY".to_string()
        })?;
    
    let url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
        api_key
    );
    log::info!("Firebase sign-in URL: {}", url);
    
    let body = serde_json::json!({
        "email": email,
        "password": password,
        "returnSecureToken": true
    });
    
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            log::error!("Auth network error: {}", e);
            format!("Auth network error: {}", e)
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        log::error!("Auth error {} (Status {}): {}", status, status.as_u16(), text);
        return Err(format!("Auth error {}: {}", status.as_u16(), text));
    }

    let data: SignInResponse = resp
        .json()
        .await
        .map_err(|e| {
            log::error!("Auth parse error: {}", e);
            format!("Auth parse error: {}", e)
        })?;

    let expires_in = data
        .expiresIn
        .parse::<i64>()
        .ok()
        .map(|sec| chrono::Utc::now().timestamp() + sec);

    log::info!("Firebase sign-in successful for {}", email);

    Ok(User {
        uid: data.localId,
        email: data.email.clone(),
        name: data
            .displayName
            .clone()
            .unwrap_or_else(|| data.email.split('@').next().unwrap_or("User").to_string()),
        photo_url: data.photoUrl.clone(),
        id_token: data.idToken,
        refresh_token: Some(data.refreshToken),
        expires_in,
    })
}

// Exchange Google authorization code for ID token
pub async fn exchange_google_code_for_token(code: &str) -> Result<User, String> {
    let api_key = std::env::var("VITE_FIREBASE_API_KEY")
        .map_err(|_| "Missing VITE_FIREBASE_API_KEY".to_string())?;

    let client_id = std::env::var("GOOGLE_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "384654392238-k02b3cvemoee9uq87pa3a3bk0gf1hbnk.apps.googleusercontent.com".to_string());
    let client_secret = std::env::var("GOOGLE_OAUTH_CLIENT_SECRET").ok();
    let redirect_uri = std::env::var("GOOGLE_OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:5173/auth/google/callback".to_string());

    let client = reqwest::Client::new();
    let mut params = vec![
        ("code", code.to_string()),
        ("client_id", client_id.clone()),
        ("redirect_uri", redirect_uri.clone()),
        ("grant_type", "authorization_code".to_string()),
    ];

    if let Some(secret) = client_secret.clone() {
        params.push(("client_secret", secret));
    }

    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            log::error!("Google token exchange failed: {}", e);
            "Token exchange failed".to_string()
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        log::error!("Google token exchange HTTP {}: {}", status, body);
        return Err(format!("Google token exchange failed: {}", body));
    }

    let token_response: serde_json::Value = response.json().await.map_err(|e| {
        log::error!("Failed to parse Google token response: {}", e);
        "Failed to parse token response".to_string()
    })?;

    let id_token = token_response["id_token"].as_str().ok_or_else(|| {
        log::error!(
            "No id_token in Google response; client_id={} redirect_uri={} secret_present={} payload={}",
            client_id,
            redirect_uri,
            client_secret.is_some(),
            token_response
        );
        "No id_token received".to_string()
    })?;

    // Get access token to fetch user profile picture
    let access_token = token_response["access_token"].as_str();

    let post_body = format!(
        "id_token={}&providerId=google.com",
        urlencoding::encode(id_token)
    );

    let mut user = firebase_sign_in_with_idp(post_body, &redirect_uri, &api_key).await?;
    
    // If we have access token, fetch user info to get profile picture
    if let Some(access_token) = access_token {
        if let Ok(profile) = fetch_google_profile(access_token).await {
            user.photo_url = profile.get("picture").and_then(|v| v.as_str()).map(String::from);
            log::info!("Fetched Google profile picture: {:?}", user.photo_url);
        }
    }
    
    log::info!("Successfully exchanged Google code via Firebase for user: {}", user.email);
    Ok(user)
}

async fn fetch_google_profile(access_token: &str) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to fetch Google profile: {}", e);
            format!("Failed to fetch profile: {}", e)
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        log::error!("Google profile HTTP {}: {}", status, body);
        return Err(format!("Failed to fetch profile: {}", body));
    }

    response.json().await.map_err(|e| {
        log::error!("Failed to parse Google profile: {}", e);
        format!("Failed to parse profile: {}", e)
    })
}

// Exchange GitHub authorization code for access token
pub async fn exchange_github_code_for_token(code: &str) -> Result<User, String> {
    let api_key = std::env::var("VITE_FIREBASE_API_KEY")
        .map_err(|_| "Missing VITE_FIREBASE_API_KEY".to_string())?;

    let client_id = std::env::var("GITHUB_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "Ov23liqnLH8iIZOZ8sMT".to_string());
    let client_secret = std::env::var("GITHUB_OAUTH_CLIENT_SECRET")
        .map_err(|_| "Missing GITHUB_OAUTH_CLIENT_SECRET".to_string())?;
    let redirect_uri = std::env::var("GITHUB_OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:5173/auth/github/callback".to_string());

    let client = reqwest::Client::new();

    let params = [
        ("client_id", client_id.clone()),
        ("client_secret", client_secret.clone()),
        ("code", code.to_string()),
        ("redirect_uri", redirect_uri.clone()),
    ];
    
    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            log::error!("GitHub token exchange failed: {}", e);
            "Token exchange failed".to_string()
        })?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        log::error!("GitHub token exchange HTTP {}: {}", status, body);
        return Err(format!("GitHub token exchange failed: {}", body));
    }

    let token_response: serde_json::Value = response.json().await.map_err(|e| {
        log::error!("Failed to parse GitHub token response: {}", e);
        "Failed to parse token response".to_string()
    })?;
    
    let access_token = token_response["access_token"]
        .as_str()
        .ok_or_else(|| {
            log::error!("No access_token in GitHub response");
            "No access_token received".to_string()
        })?;

    let post_body = format!(
        "access_token={}&providerId=github.com",
        urlencoding::encode(access_token)
    );

    let user = firebase_sign_in_with_idp(post_body, &redirect_uri, &api_key).await?;
    log::info!("Successfully exchanged GitHub code via Firebase for user: {}", user.email);
    Ok(user)
}

fn base64_decode(s: &str) -> Result<String, Box<dyn std::error::Error>> {
    use base64::{engine::general_purpose, Engine as _};
    let bytes = general_purpose::URL_SAFE_NO_PAD.decode(s)?;
    Ok(String::from_utf8(bytes)?)
}// Build Google OAuth URL for Firebase
pub fn get_google_oauth_url() -> Result<String, String> {
    let client_id = std::env::var("GOOGLE_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "384654392238-k02b3cvemoee9uq87pa3a3bk0gf1hbnk.apps.googleusercontent.com".to_string());
    let redirect_url = std::env::var("GOOGLE_OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:5173/auth/google/callback".to_string());

    Ok(format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&access_type=offline&prompt=consent",
        client_id,
        urlencoding::encode(&redirect_url)
    ))
}

// Build GitHub OAuth URL for Firebase
pub fn get_github_oauth_url() -> Result<String, String> {
    let client_id = std::env::var("GITHUB_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "Ov23liqnLH8iIZOZ8sMT".to_string());
    let redirect_url = std::env::var("GITHUB_OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:5173/auth/github/callback".to_string());
    
    Ok(format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email",
        client_id,
        urlencoding::encode(&redirect_url)
    ))
}
pub async fn firebase_sign_up_email_password(email: &str, password: &str) -> Result<User, String> {
    let api_key = std::env::var("VITE_FIREBASE_API_KEY")
        .map_err(|_| {
            log::error!("Missing VITE_FIREBASE_API_KEY env var");
            "Missing VITE_FIREBASE_API_KEY".to_string()
        })?;
    
    let url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signUp?key={}",
        api_key
    );
    log::info!("Firebase sign-up URL: {}", url);
    
    let body = serde_json::json!({
        "email": email,
        "password": password,
        "returnSecureToken": true
    });
    
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            log::error!("Signup network error: {}", e);
            format!("Signup network error: {}", e)
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        log::error!("Signup error {} (Status {}): {}", status, status.as_u16(), text);
        return Err(format!("Signup error {}: {}", status.as_u16(), text));
    }

    let data: SignInResponse = resp
        .json()
        .await
        .map_err(|e| {
            log::error!("Signup parse error: {}", e);
            format!("Signup parse error: {}", e)
        })?;

    let expires_in = data
        .expiresIn
        .parse::<i64>()
        .ok()
        .map(|sec| chrono::Utc::now().timestamp() + sec);

    log::info!("Firebase sign-up successful for {}", email);

    Ok(User {
        uid: data.localId,
        email: data.email.clone(),
        name: data
            .displayName
            .clone()
            .unwrap_or_else(|| data.email.split('@').next().unwrap_or("User").to_string()),
        photo_url: data.photoUrl.clone(),
        id_token: data.idToken,
        refresh_token: Some(data.refreshToken),
        expires_in,
    })
}
