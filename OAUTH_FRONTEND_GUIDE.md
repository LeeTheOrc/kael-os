# Kael-OS OAuth Frontend Integration Guide

## Quick Start

You have **3 Tauri commands** available from your frontend code:

### 1. Get OAuth URL
```rust
// Fetch the OAuth login URL for a provider
let url = match invoke::<_, String>("get_oauth_url", json!({"provider": "google"})).await {
    Ok(url) => url,
    Err(e) => return eprintln!("Error: {}", e),
};
// Result: "https://accounts.google.com/o/oauth2/v2/auth?client_id=...&redirect_uri=..."
```

### 2. Poll for Callback Code
```rust
// After user logs in, poll for the authorization code
let result = invoke::<_, Option<OAuthResult>>(
    "poll_oauth_callback", 
    json!({"provider": "google"})
).await.ok().flatten();

// Result: Some({ provider: "google", code: "4/0AX4X...", state: "" })
```

### 3. Exchange Code for Token
```rust
// Exchange the code for an access token and user info
let user = invoke::<_, User>(
    "exchange_oauth_token",
    json!({"provider": "google", "code": "4/0AX4X..."})
).await?;

// Result: User { uid, email, name, photo_url, id_token, refresh_token, ... }
```

## Implementation Pattern

Here's the recommended flow for your login component:

### Step 1: State Management
```rust
#[allow(non_snake_case)]
pub fn LoginPanel(mut props: LoginProps) -> Element {
    // Existing state...
    let mut show_oauth_modal = use_signal(|| false);
    let mut oauth_provider = use_signal(String::new);
    let mut oauth_url = use_signal(String::new);
    let mut oauth_loading = use_signal(|| false);
    let mut oauth_error = use_signal::<Option<String>>(|| None);
```

### Step 2: Add OAuth Button
```rust
// Replace the existing browser-based OAuth button with in-app modal version
button {
    class: "w-full px-4 py-3 rounded-lg font-bold",
    style: "background: linear-gradient(135deg, #4285f4 0%, #357ae8 100%); color: white;",
    onclick: move |_| {
        show_oauth_modal.set(true);
        oauth_provider.set("google".to_string());
        
        // Fetch OAuth URL
        let provider = "google".to_string();
        spawn(async move {
            match invoke::<_, String>("get_oauth_url", json!({"provider": provider})).await {
                Ok(url) => {
                    oauth_url.set(url);
                    oauth_loading.set(false);
                }
                Err(e) => {
                    oauth_error.set(Some(e));
                    oauth_loading.set(false);
                }
            }
        });
    },
    "🔵 Sign in with Google"
}

button {
    class: "w-full px-4 py-3 rounded-lg font-bold",
    style: "background: linear-gradient(135deg, #24292e 0%, #1a1e22 100%); color: white;",
    onclick: move |_| {
        show_oauth_modal.set(true);
        oauth_provider.set("github".to_string());
        
        let provider = "github".to_string();
        spawn(async move {
            match invoke::<_, String>("get_oauth_url", json!({"provider": provider})).await {
                Ok(url) => {
                    oauth_url.set(url);
                    oauth_loading.set(false);
                }
                Err(e) => {
                    oauth_error.set(Some(e));
                    oauth_loading.set(false);
                }
            }
        });
    },
    "⚫ Sign in with GitHub"
}
```

### Step 3: OAuth Modal with Iframe
```rust
// Modal that displays the OAuth login page
if show_oauth_modal() {
    div {
        class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50",
        onclick: move |_| { show_oauth_modal.set(false); },
        
        div {
            class: "bg-gradient-to-br from-purple-900 via-indigo-900 to-black rounded-2xl border border-purple-700 w-full max-w-md h-96 flex flex-col",
            onclick: move |e| { e.stop_propagation(); },
            
            // Header
            div {
                class: "px-6 py-4 border-b border-purple-700 flex justify-between items-center",
                h2 {
                    class: "text-white font-bold",
                    if oauth_provider() == "google" { "Sign in with Google" } 
                    else { "Sign in with GitHub" }
                }
                button {
                    onclick: move |_| { show_oauth_modal.set(false); },
                    "×"
                }
            }
            
            // Body - Loading or Error or Iframe
            div {
                class: "flex-1 overflow-hidden bg-white",
                
                if oauth_loading() {
                    div {
                        class: "flex items-center justify-center h-full",
                        p { class: "text-gray-400", "Loading..." }
                    }
                } else if let Some(error) = oauth_error() {
                    div {
                        class: "flex flex-col items-center justify-center h-full p-4",
                        p { class: "text-red-500 text-center mb-4", "{error}" }
                        button {
                            onclick: move |_| { show_oauth_modal.set(false); },
                            "Close"
                        }
                    }
                } else if !oauth_url().is_empty() {
                    iframe {
                        src: "{oauth_url()}",
                        class: "w-full h-full border-none"
                    }
                }
            }
        }
    }
    
    // Start polling for callback after iframe loads
    use_effect(move || {
        if !show_oauth_modal() { return; }
        
        let provider = oauth_provider().clone();
        let auth_service = props.auth_service.clone();
        
        spawn(async move {
            // Poll for callback (timeout after 5 minutes)
            for attempt in 0..600 {
                // Small delay before polling
                gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;
                
                // Check if modal was closed
                if !show_oauth_modal() { break; }
                
                // Poll for the OAuth code
                match invoke::<_, Option<_>>("poll_oauth_callback", json!({"provider": provider.clone()})).await {
                    Ok(Some(result)) => {
                        // Got the code! Now exchange it for a token
                        match invoke::<_, User>(
                            "exchange_oauth_token",
                            json!({"provider": result.provider, "code": result.code})
                        ).await {
                            Ok(user) => {
                                // Success! Log in the user
                                auth_service.write().set_user(user);
                                show_oauth_modal.set(false);
                                break;
                            }
                            Err(e) => {
                                oauth_error.set(Some(format!("Token exchange failed: {}", e)));
                                break;
                            }
                        }
                    }
                    Ok(None) => {
                        // Code not yet available, keep polling
                    }
                    Err(e) => {
                        if attempt == 0 {
                            oauth_error.set(Some(format!("Polling error: {}", e)));
                        }
                        // Continue polling
                    }
                }
                
                // Timeout after 5 minutes
                if attempt >= 599 {
                    oauth_error.set(Some("OAuth login timeout".to_string()));
                    break;
                }
            }
        });
    });
}
```

## Migration Checklist

- [ ] Remove existing browser-based OAuth code (the `open::that(&url)` calls)
- [ ] Replace OAuth buttons with in-app modal version (above)
- [ ] Add state signals for: `show_oauth_modal`, `oauth_provider`, `oauth_url`, etc.
- [ ] Test Google OAuth
- [ ] Test GitHub OAuth
- [ ] Verify user is logged in with correct info (email, name, photo)
- [ ] Test modal closes after successful login
- [ ] Test modal closes when user clicks X button

## What Happens Behind the Scenes

1. **User clicks "Sign in with Google"**
   - Modal opens, iframe is created
   - Tauri command `get_oauth_url("google")` is called
   - OAuth URL is returned and iframe src is set

2. **Iframe loads OAuth login page**
   - User sees Google login form (inside your app!)
   - User enters credentials and grants permission

3. **User clicks "Allow"**
   - Google redirects to `http://localhost:5173/auth/google/callback?code=...`
   - This request goes to your Rust OAuth server running in the background
   - Server captures the code and stores it in memory

4. **Frontend polls for callback**
   - Every 500ms, frontend calls `poll_oauth_callback("google")`
   - When code is found, it's returned to the frontend
   - Frontend immediately calls `exchange_oauth_token("google", code)`

5. **Token exchange**
   - Rust backend POSTs the code to Google's token endpoint
   - Receives JWT token and decodes it
   - Extracts user info (name, email, photo_url)
   - Returns User struct to frontend

6. **Auto-login**
   - Frontend calls `auth_service.write().set_user(user)`
   - Modal closes automatically
   - User is logged in!

## Key Differences from Browser-Based OAuth

| Aspect | Browser-Based | In-App |
|--------|---------------|--------|
| **Window** | Opens new browser window | Modal in app |
| **Control** | Lost to browser | Full control |
| **Redirect** | Browser navigates away | Iframe loads |
| **Code capture** | User has to copy-paste | Auto-detected |
| **User experience** | Disruptive | Seamless |
| **Security** | Depends on browser | Same as app |

## Troubleshooting

### Modal shows blank iframe
- Check browser console for errors
- Verify OAuth URL is being fetched correctly
- Make sure `oauth_url` signal is being set

### Code never detected
- Check that OAuth server is running (app should log this)
- Verify user clicked "Allow" on OAuth consent screen
- Check app logs for "OAuth callback received"

### Token exchange fails
- Check that OAuth credentials are correct in `.env.local`
- Verify the code matches the format expected by OAuth provider
- Check error message in `oauth_error` signal

### Modal doesn't close after login
- Check that `auth_service.write().set_user(user)` is being called
- Verify `show_oauth_modal.set(false)` is being executed
- Check browser console for any JavaScript errors

## Dependencies

Make sure your login component imports are updated:

```rust
use crate::auth::{AuthService, User, ...};
use dioxus::prelude::*;
use serde_json::json;
use gloo_timers; // for sleep in polling
```

## Next Steps

1. Integrate this code into your login component
2. Test with Google OAuth first
3. Then test with GitHub OAuth
4. Get user feedback on the experience
5. Refine styling and UX as needed

Your backend is 100% ready. This is just the frontend glue to make it work!
