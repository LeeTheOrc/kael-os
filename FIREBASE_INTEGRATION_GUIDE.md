# Firebase Integration Guide for App Tracker

## Quick Setup

### 1. Initialize DB Table on App Start

Add to your app initialization (in `main.rs` or app setup):

```rust
use crate::services::app_projects;

// When initializing the app
pub fn init_app() {
    let db = get_db_connection();
    app_projects::init_projects_table(&db).expect("Failed to init projects table");
}
```

### 2. Load Projects from Local Cache

```rust
use crate::services::app_projects;

// On app startup
let db = get_db_connection();
let cached_projects = app_projects::get_active_projects(&db)?;
// Use cached_projects to populate UI
```

### 3. Save Projects to Local Cache

Already integrated in app.rs:
```rust
on_add: move |new_project: AppProject| {
    projects.write().push(new_project.clone());
    save_projects(&projects());  // Saves to JSON
    // TODO: Also save to local DB:
    // crate::services::app_projects::save_project_local(&db, &new_project)?;
},
```

## Firebase Sync Implementation

### Example: Sync Projects to Firestore

```rust
use crate::firebase;
use crate::services::app_projects;

pub async fn sync_projects_to_firebase(
    user: &crate::auth::User,
    db: &Connection,
) -> Result<(), String> {
    // Get all projects that haven't been synced
    let projects = app_projects::get_projects_local(&db)
        .map_err(|e| format!("DB error: {}", e))?;
    
    let project_id = firebase::project_id()?;
    let client = reqwest::Client::new();
    let mut synced_ids = Vec::new();
    
    for project in projects.iter() {
        // Skip if already synced
        // (you would track this in your DB)
        
        let doc = app_projects::project_to_firebase_doc(project);
        let url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/app_projects/{}",
            project_id, user.uid, project.id
        );
        
        let response = client
            .patch(&url)
            .bearer_auth(&user.id_token)
            .json(&serde_json::json!({ "fields": doc["fields"] }))
            .send()
            .await
            .map_err(|e| format!("Sync error: {}", e))?;
        
        if response.status().is_success() {
            synced_ids.push(project.id.clone());
        }
    }
    
    // Mark as synced in local DB
    app_projects::mark_synced(&db, &synced_ids)
        .map_err(|e| format!("Mark synced error: {}", e))?;
    
    Ok(())
}
```

### Example: Pull Projects from Firestore

```rust
pub async fn pull_projects_from_firebase(
    user: &crate::auth::User,
    db: &Connection,
) -> Result<Vec<AppProject>, String> {
    let project_id = firebase::project_id()?;
    let client = reqwest::Client::new();
    
    let url = format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/app_projects",
        project_id, user.uid
    );
    
    let response = client
        .get(&url)
        .bearer_auth(&user.id_token)
        .send()
        .await
        .map_err(|e| format!("Fetch error: {}", e))?;
    
    if !response.status().is_success() {
        return Ok(vec![]); // Empty collection returns 404
    }
    
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let mut projects = Vec::new();
    
    if let Some(docs) = data["documents"].as_array() {
        for doc in docs {
            if let Some(fields) = doc["fields"].as_object() {
                let project = AppProject {
                    id: fields
                        .get("id")
                        .and_then(|v| v["stringValue"].as_str())
                        .unwrap_or("")
                        .to_string(),
                    name: fields
                        .get("name")
                        .and_then(|v| v["stringValue"].as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: fields
                        .get("description")
                        .and_then(|v| v["stringValue"].as_str())
                        .unwrap_or("")
                        .to_string(),
                    status: match fields
                        .get("status")
                        .and_then(|v| v["stringValue"].as_str())
                        .unwrap_or("")
                    {
                        "making" => AppStatus::Making,
                        "testing" => AppStatus::Testing,
                        "done" => AppStatus::Done,
                        _ => AppStatus::Want,
                    },
                    version: fields
                        .get("version")
                        .and_then(|v| v["stringValue"].as_str())
                        .unwrap_or("0.0.1")
                        .to_string(),
                    created_at: fields
                        .get("created_at")
                        .and_then(|v| v["stringValue"].as_str())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_else(chrono::Utc::now),
                    updated_at: fields
                        .get("updated_at")
                        .and_then(|v| v["stringValue"].as_str())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_else(chrono::Utc::now),
                    archived: fields
                        .get("archived")
                        .and_then(|v| v["booleanValue"].as_bool())
                        .unwrap_or(false),
                };
                
                // Save to local DB
                app_projects::save_project_local(&db, &project)?;
                projects.push(project);
            }
        }
    }
    
    Ok(projects)
}
```

### Example: Auto-Sync on Changes

```rust
// In app component
on_status_change: move |(project_id, new_status): (String, AppStatus)| {
    if let Some(project) = projects.write().iter_mut().find(|p| p.id == project_id) {
        project.status = new_status;
    }
    save_projects(&projects());
    
    // TODO: Add background Firebase sync
    // spawn(async move {
    //     if let Err(e) = sync_to_firebase_async(&user, &db).await {
    //         log::error!("Firebase sync failed: {}", e);
    //     }
    // });
},
```

## Database Structure

### Firebase Firestore Collection Path
```
users/{uid}/app_projects/{project_id}
```

### Document Structure
```json
{
  "id": { "stringValue": "uuid" },
  "name": { "stringValue": "App Name" },
  "description": { "stringValue": "Description" },
  "status": { "stringValue": "making|want|testing|done" },
  "version": { "stringValue": "0.0.1" },
  "archived": { "booleanValue": false },
  "created_at": { "stringValue": "2025-12-14T10:30:00Z" },
  "updated_at": { "stringValue": "2025-12-14T10:30:00Z" }
}
```

### SQLite Table Schema
```sql
CREATE TABLE app_projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,        -- 'making', 'want', 'testing', 'done'
    version TEXT NOT NULL,
    archived INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    synced INTEGER NOT NULL DEFAULT 0,
    firebase_synced_at TEXT
);
```

## Data Flow Diagrams

### Push to Firebase
```
Local Project Updated
         ↓
Local DB Saved
         ↓
Service: project_to_firebase_doc()
         ↓
PATCH /firestore/documents/...
         ↓
mark_synced() in DB
```

### Pull from Firebase
```
Pull Request from Firestore
         ↓
Parse Firestore Response
         ↓
Create AppProject structs
         ↓
save_project_local() for each
         ↓
Update UI with synced projects
```

### Offline-First Strategy
```
User Edits Project
         ↓
Save to Local DB (immediate)
         ↓
Update UI (immediate)
         ↓
Queue Firebase Sync
         ↓
On Connection: Sync unsynced items
```

## Configuration

Add these to your .env.local:
```env
VITE_FIREBASE_PROJECT_ID=your-project-id
VITE_FIREBASE_API_KEY=your-api-key
```

## Error Handling

```rust
// Graceful fallback to local DB on Firebase errors
match sync_to_firebase(&user, &db).await {
    Ok(_) => log::info!("Firebase sync successful"),
    Err(e) => {
        log::warn!("Firebase sync failed, using local cache: {}", e);
        // App continues working with local data
    }
}
```

## Testing

### Unit Tests for App Projects Service
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_save_and_load_project() {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        init_projects_table(&db).unwrap();
        
        let project = AppProject::new(
            "Test".to_string(),
            "Test app".to_string(),
            AppStatus::Making
        );
        
        save_project_local(&db, &project).unwrap();
        let loaded = get_projects_local(&db).unwrap();
        
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Test");
    }
}
```

## Performance Considerations

1. **Batch Syncs**: Sync multiple projects in one Firebase request
2. **Async Operations**: Don't block UI during Firebase operations
3. **Local Cache First**: Always serve from local DB for instant response
4. **Delta Sync**: Only sync changed projects (track with `synced` flag)
5. **Rate Limiting**: Implement backoff for Firebase API

## Security Notes

- User ID from `auth::User` ensures data isolation
- Bearer token validates Firebase access
- All sensitive data in `.env.local` (not committed)
- Local DB stores unencrypted (consider encryption for production)
