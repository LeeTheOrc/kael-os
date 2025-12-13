// OAuth Integration Guide for Kael-OS Login Component
//
// This document explains how to use the new in-app OAuth system that's been set up.
//
// ARCHITECTURE:
// 1. OAuth callback server runs on localhost:5173 (separate thread)
// 2. When user clicks "Sign in with Google/GitHub", the oauth_url is fetched
// 3. OAuth page loads in an iframe within an app modal (fully contained)
// 4. User logs in with their provider
// 5. Provider redirects to http://localhost:5173/auth/{provider}/callback?code=...
// 6. OAuth server captures the code in memory
// 7. Frontend polls for the code via `poll_oauth_callback` command
// 8. When code is found, exchange it for a token via `exchange_oauth_token` command
// 9. User is logged in!
//
// AVAILABLE TAURI COMMANDS:
//
// 1. get_oauth_url(provider: String) -> Result<String, String>
//    Gets the OAuth URL for the given provider (google or github)
//    Usage:
//      const url = await invoke('get_oauth_url', { provider: 'google' });
//
// 2. poll_oauth_callback(provider: String) -> Result<Option<OAuthResult>, String>
//    Polls for an OAuth callback result from the server
//    Returns: { provider: string, code: string, state: string } or null
//    Usage:
//      const result = await invoke('poll_oauth_callback', { provider: 'google' });
//      if (result) {
//        // Code received!
//        const { provider, code } = result;
//      }
//
// 3. exchange_oauth_token(provider: String, code: String) -> Result<User, String>
//    Exchanges the OAuth code for a token
//    Returns: { uid, email, name, photo_url, id_token, refresh_token, expires_in }
//    Usage:
//      const user = await invoke('exchange_oauth_token', { 
//        provider: 'google', 
//        code: 'auth_code_123' 
//      });
//
// IMPLEMENTATION EXAMPLE (Dioxus component):
//
// ```rust
// #[allow(non_snake_case)]
// pub fn OAuthModal(mut props: OAuthModalProps) -> Element {
//     let mut show_modal = use_signal(|| false);
//     let mut current_provider = use_signal(|| String::new());
//
//     let open_oauth_modal = move |provider: String| {
//         show_modal.set(true);
//         current_provider.set(provider.clone());
//         
//         // Fetch OAuth URL and start polling
//         spawn(async move {
//             match invoke::<_, String>("get_oauth_url", &SerializedMessage::json(&serde_json::json!({
//                 "provider": provider.clone()
//             })).unwrap()).await {
//                 Ok(url) => {
//                     // Iframe now loads this URL
//                     // Start polling for callback
//                     poll_for_callback(provider.clone()).await;
//                 }
//                 Err(e) => log::error!("Failed to get OAuth URL: {}", e),
//             }
//         });
//     };
//
//     let poll_for_callback = |provider: String| {
//         Box::pin(async move {
//             for _ in 0..600 { // Poll for 5 minutes (600 * 500ms)
//                 tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
//                 
//                 match invoke::<_, Option<OAuthResult>>("poll_oauth_callback", &SerializedMessage::json(&serde_json::json!({
//                     "provider": provider.clone()
//                 })).unwrap()).await {
//                     Ok(Some(result)) => {
//                         // Got the code! Exchange it for a token
//                         match invoke::<_, User>("exchange_oauth_token", &SerializedMessage::json(&serde_json::json!({
//                             "provider": result.provider,
//                             "code": result.code
//                         })).unwrap()).await {
//                             Ok(user) => {
//                                 // Login successful!
//                                 props.auth_service.write().set_user(user);
//                                 show_modal.set(false);
//                             }
//                             Err(e) => log::error!("Token exchange failed: {}", e),
//                         }
//                         break;
//                     }
//                     Ok(None) => {} // Not yet
//                     Err(e) => {
//                         log::error!("Poll error: {}", e);
//                         break;
//                     }
//                 }
//             }
//         })
//     };
//
//     rsx! {
//         // OAuth buttons
//         button {
//             onclick: move |_| open_oauth_modal("google".to_string()),
//             "Sign in with Google"
//         }
//         
//         // Modal with iframe
//         if show_modal() {
//             div {
//                 class: "modal",
//                 iframe {
//                     src: "https://accounts.google.com/o/oauth2/v2/auth?...",
//                 }
//             }
//         }
//     }
// }
// ```
//
// FLOW DIAGRAM:
//
// User clicks "Sign in with Google"
//          ↓
// invoke('get_oauth_url', { provider: 'google' })
//          ↓
// OAuth URL returned: https://accounts.google.com/o/oauth2/v2/auth?...
//          ↓
// Iframe loads OAuth URL
//          ↓
// User enters credentials and grants permission
//          ↓
// Provider redirects to: http://localhost:5173/auth/google/callback?code=...
//          ↓
// OAuth Server captures code (stored in HashMap)
//          ↓
// Frontend polling: invoke('poll_oauth_callback', { provider: 'google' })
//          ↓
// Found! Returns: { provider: 'google', code: 'xyz123', state: '' }
//          ↓
// invoke('exchange_oauth_token', { provider: 'google', code: 'xyz123' })
//          ↓
// Token exchanged and user info extracted
//          ↓
// User struct returned: { uid, email, name, photo_url, ... }
//          ↓
// Set user in auth_service
//          ↓
// Close modal and show authenticated UI
//
// ENVIRONMENT SETUP:
//
// Make sure your .env.local has:
// - GOOGLE_OAUTH_CLIENT_ID=384654392238-k02b3cvemoee9uq87pa3a3bk0gf1hbnk.apps.googleusercontent.com
// - GITHUB_OAUTH_CLIENT_ID=Ov23liqnLH8iIZOZ8sMT
//
// These are already configured in the Rust code as defaults if not in .env.local
//
// TESTING:
//
// 1. Run the app: cargo run --manifest-path src-tauri/Cargo.toml
// 2. Click "Sign in with Google"
// 3. You should see the Google login page (NOT in browser, in-app!)
// 4. Log in with your Google account
// 5. Google redirects to http://localhost:5173/auth/google/callback?code=...
// 6. Frontend detects code and exchanges it for token
// 7. User is logged in!
//
// TROUBLESHOOTING:
//
// - "OAuth server not listening": Make sure oauth_server is initialized in main.rs
// - "Code never found": Check if OAuth callback server is capturing the request
//   Check logs for: "OAuth callback received for provider: google"
// - "Token exchange failed": Check if your OAuth credentials are correct
// - "Callback URL is wrong": Must be http://localhost:5173/auth/{provider}/callback
//   This is configured in the OAuth console as the redirect URI
