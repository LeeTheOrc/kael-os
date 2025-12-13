#![allow(dead_code)]

use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
pub struct OAuthCallback {
    pub code: Option<String>,
    pub error: Option<String>,
    pub provider: String,
}

pub struct OAuthServer {
    callback_data: Arc<Mutex<HashMap<String, OAuthCallback>>>,
}

pub static OAUTH_SERVER: Lazy<OAuthServer> = Lazy::new(OAuthServer::new);

impl OAuthServer {
    pub fn new() -> Self {
        Self {
            callback_data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn run_server(&self) {
        let callback_data = Arc::clone(&self.callback_data);
        
        let listener = match tokio::net::TcpListener::bind("127.0.0.1:5173").await {
            Ok(l) => l,
            Err(e) => {
                log::error!("Failed to bind OAuth server: {}", e);
                return;
            }
        };
        
        log::info!("OAuth callback server listening on 127.0.0.1:5173");
        
        loop {
            match listener.accept().await {
                Ok((socket, _)) => {
                    let data = Arc::clone(&callback_data);
                    tokio::spawn(handle_connection(socket, data));
                }
                Err(e) => {
                    log::error!("OAuth server accept error: {}", e);
                }
            }
        }
    }

    pub async fn get_callback(&self, provider: &str) -> Option<OAuthCallback> {
        let data = self.callback_data.lock().await;
        data.get(provider).cloned()
    }

    pub async fn clear_callback(&self, provider: &str) {
        let mut data = self.callback_data.lock().await;
        data.remove(provider);
    }
}

/// Start the OAuth server in a background thread with its own Tokio runtime
pub fn start_oauth_server() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            let server = &*OAUTH_SERVER;
            server.run_server().await;
        });
    });
}

async fn handle_connection(
    mut socket: tokio::net::TcpStream,
    callback_data: Arc<Mutex<HashMap<String, OAuthCallback>>>,
) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = vec![0; 4096];
    match socket.read(&mut buf).await {
        Ok(n) if n > 0 => {
            let request = String::from_utf8_lossy(&buf[..n]);
            log::info!("OAuth server received request ({}bytes): {}", n, request.lines().next().unwrap_or(""));
            
            // Parse the request line
            if let Some(request_line) = request.lines().next() {
                if request_line.contains("/auth/google/callback") {
                    log::info!("Handling Google OAuth callback");
                    handle_oauth_callback(&request, "google", &callback_data).await;
                } else if request_line.contains("/auth/github/callback") {
                    log::info!("Handling GitHub OAuth callback");
                    handle_oauth_callback(&request, "github", &callback_data).await;
                } else {
                    log::debug!("Received non-callback request: {}", request_line);
                }
            }

            // Send response
            let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 120\r\n\r\n<html><body><p>Authentication successful!</p><p>You can close this window and return to the app.</p></body></html>";
            let _ = socket.write_all(response).await;
        }
        _ => {
            log::debug!("OAuth server: no data received");
        }
    }
}

async fn handle_oauth_callback(
    request: &str,
    provider: &str,
    callback_data: &Arc<Mutex<HashMap<String, OAuthCallback>>>,
) {
    // Extract code or error from query string
    let code = extract_param(request, "code");
    let error = extract_param(request, "error");
    let error_description = extract_param(request, "error_description");

    if code.is_some() {
        log::info!("OAuth callback received for {}: code present", provider);
    } else if error.is_some() {
        log::warn!("OAuth callback error for {}: {:?} - {:?}", provider, error, error_description);
    } else {
        log::warn!("OAuth callback received but no code or error for {}", provider);
    }

    let callback = OAuthCallback {
        code,
        error,
        provider: provider.to_string(),
    };

    let mut data = callback_data.lock().await;
    data.insert(provider.to_string(), callback);
    
    log::info!("OAuth callback stored for provider: {}", provider);
}

fn extract_param(request: &str, param: &str) -> Option<String> {
    // Look for param=value in the request
    let search = format!("{}=", param);
    if let Some(start) = request.find(&search) {
        let value_start = start + search.len();
        let remaining = &request[value_start..];
        
        // Find the end of the parameter (either & or space)
        let value_end = remaining
            .find(|c: char| c == '&' || c == ' ')
            .unwrap_or(remaining.len());
        
        let raw_value = &remaining[..value_end];
        
        // URL decode the value
        let decoded = urlencoding::decode(raw_value)
            .ok()
            .map(|cow| cow.into_owned())
            .unwrap_or_else(|| raw_value.to_string());
        
        log::debug!("Extracted param {}: {} -> {}", param, raw_value, decoded);
        
        if decoded.is_empty() {
            None
        } else {
            Some(decoded)
        }
    } else {
        None
    }
}
