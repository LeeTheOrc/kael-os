//! App Projects Service
//! Handles Firebase sync and local database persistence for app projects

use crate::state::{AppProject, AppStatus};
use rusqlite::Connection;
use serde_json::json;

const PROJECTS_TABLE: &str = r#"
    CREATE TABLE IF NOT EXISTS app_projects (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT NOT NULL,
        status TEXT NOT NULL,
        version TEXT NOT NULL,
        archived INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        synced INTEGER NOT NULL DEFAULT 0,
        firebase_synced_at TEXT
    )
"#;

/// Initialize app projects table in local database
pub fn init_projects_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(PROJECTS_TABLE, [])?;
    Ok(())
}

/// Save a project to local database
pub fn save_project_local(conn: &Connection, project: &AppProject) -> rusqlite::Result<()> {
    let status_str = match project.status {
        AppStatus::Making => "making",
        AppStatus::Want => "want",
        AppStatus::Testing => "testing",
        AppStatus::Done => "done",
    };

    conn.execute(
        "INSERT OR REPLACE INTO app_projects 
        (id, name, description, status, version, archived, created_at, updated_at, synced)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)",
        rusqlite::params![
            &project.id,
            &project.name,
            &project.description,
            status_str,
            &project.version,
            if project.archived { 1 } else { 0 },
            project.created_at.to_rfc3339(),
            project.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Get all projects from local database
pub fn get_projects_local(conn: &Connection) -> rusqlite::Result<Vec<AppProject>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, status, version, archived, created_at, updated_at 
         FROM app_projects ORDER BY updated_at DESC",
    )?;

    let projects = stmt.query_map([], |row| {
        let status_str: String = row.get(3)?;
        let status = match status_str.as_str() {
            "making" => AppStatus::Making,
            "testing" => AppStatus::Testing,
            "done" => AppStatus::Done,
            _ => AppStatus::Want,
        };

        Ok(AppProject {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            status,
            version: row.get(4)?,
            archived: row.get::<_, i32>(5)? != 0,
            created_at: row.get::<_, String>(6)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: row.get::<_, String>(7)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
        })
    })?;

    let mut result = Vec::new();
    for project in projects {
        result.push(project?);
    }
    Ok(result)
}

/// Get active projects (not archived)
pub fn get_active_projects(conn: &Connection) -> rusqlite::Result<Vec<AppProject>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, status, version, archived, created_at, updated_at 
         FROM app_projects WHERE archived = 0 ORDER BY updated_at DESC",
    )?;

    let projects = stmt.query_map([], |row| {
        let status_str: String = row.get(3)?;
        let status = match status_str.as_str() {
            "making" => AppStatus::Making,
            "testing" => AppStatus::Testing,
            "done" => AppStatus::Done,
            _ => AppStatus::Want,
        };

        Ok(AppProject {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            status,
            version: row.get(4)?,
            archived: row.get::<_, i32>(5)? != 0,
            created_at: row.get::<_, String>(6)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: row.get::<_, String>(7)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
        })
    })?;

    let mut result = Vec::new();
    for project in projects {
        result.push(project?);
    }
    Ok(result)
}

/// Get archived projects
pub fn get_archived_projects(conn: &Connection) -> rusqlite::Result<Vec<AppProject>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, status, version, archived, created_at, updated_at 
         FROM app_projects WHERE archived = 1 ORDER BY updated_at DESC",
    )?;

    let projects = stmt.query_map([], |row| {
        let status_str: String = row.get(3)?;
        let status = match status_str.as_str() {
            "making" => AppStatus::Making,
            "testing" => AppStatus::Testing,
            "done" => AppStatus::Done,
            _ => AppStatus::Want,
        };

        Ok(AppProject {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            status,
            version: row.get(4)?,
            archived: row.get::<_, i32>(5)? != 0,
            created_at: row.get::<_, String>(6)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: row.get::<_, String>(7)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
        })
    })?;

    let mut result = Vec::new();
    for project in projects {
        result.push(project?);
    }
    Ok(result)
}

/// Archive/unarchive a project
pub fn toggle_archive(conn: &Connection, project_id: &str, archived: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE app_projects SET archived = ?, updated_at = ? WHERE id = ?",
        rusqlite::params![if archived { 1 } else { 0 }, chrono::Utc::now().to_rfc3339(), project_id],
    )?;
    Ok(())
}

/// Delete a project from local database
pub fn delete_project_local(conn: &Connection, project_id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM app_projects WHERE id = ?", rusqlite::params![project_id])?;
    Ok(())
}

/// Prepare project data for Firebase sync
pub fn project_to_firebase_doc(project: &AppProject) -> serde_json::Value {
    json!({
        "fields": {
            "id": { "stringValue": project.id },
            "name": { "stringValue": project.name },
            "description": { "stringValue": project.description },
            "status": { "stringValue": match project.status {
                AppStatus::Making => "making",
                AppStatus::Want => "want",
                AppStatus::Testing => "testing",
                AppStatus::Done => "done",
            }},
            "version": { "stringValue": project.version },
            "archived": { "booleanValue": project.archived },
            "created_at": { "stringValue": project.created_at.to_rfc3339() },
            "updated_at": { "stringValue": project.updated_at.to_rfc3339() },
        }
    })
}

/// Mark projects as synced in local database
pub fn mark_synced(conn: &Connection, project_ids: &[String]) -> rusqlite::Result<()> {
    for id in project_ids {
        conn.execute(
            "UPDATE app_projects SET synced = 1, firebase_synced_at = ? WHERE id = ?",
            rusqlite::params![chrono::Utc::now().to_rfc3339(), id],
        )?;
    }
    Ok(())
}

// ============================================================================
// Firebase Sync Implementation
// ============================================================================

use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
struct FirestoreDocument {
    name: String,
    fields: FirestoreFields,
    #[serde(rename = "updateTime")]
    update_time: Option<String>,
}

#[derive(Deserialize)]
struct FirestoreFields {
    id: Option<FirestoreValue>,
    name: Option<FirestoreValue>,
    description: Option<FirestoreValue>,
    status: Option<FirestoreValue>,
    version: Option<FirestoreValue>,
    archived: Option<FirestoreBoolValue>,
    created_at: Option<FirestoreValue>,
    updated_at: Option<FirestoreValue>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct FirestoreValue {
    stringValue: Option<String>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct FirestoreBoolValue {
    booleanValue: Option<bool>,
}

#[derive(Deserialize)]
struct FirestoreListResponse {
    documents: Option<Vec<FirestoreDocument>>,
}

/// Sync projects bidirectionally with Firebase Firestore
pub async fn sync_projects_with_firebase(
    conn: &Connection,
    id_token: &str,
    user_id: &str,
) -> Result<usize, String> {
    let project_id = std::env::var("VITE_FIREBASE_PROJECT_ID")
        .map_err(|_| "Missing VITE_FIREBASE_PROJECT_ID in environment".to_string())?;
    
    let client = Client::new();
    let base_url = format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/projects",
        project_id, user_id
    );

    // Step 1: Fetch all projects from Firebase
    let firebase_projects = fetch_firebase_projects(&client, &base_url, id_token).await?;
    
    // Step 2: Get all local projects
    let local_projects = get_projects_local(conn)
        .map_err(|e| format!("Failed to get local projects: {}", e))?;

    // Step 3: Merge and resolve conflicts
    let mut synced_count = 0;
    
    // Upload local projects that don't exist in Firebase or are newer
    for local_proj in &local_projects {
        if let Some(firebase_proj) = firebase_projects.iter().find(|p| p.id == local_proj.id) {
            // Project exists in both - compare timestamps
            if local_proj.updated_at > firebase_proj.updated_at {
                // Local is newer - upload
                upload_project_to_firebase(&client, &base_url, id_token, local_proj).await?;
                synced_count += 1;
            } else if firebase_proj.updated_at > local_proj.updated_at {
                // Firebase is newer - download
                save_project_local(conn, firebase_proj)
                    .map_err(|e| format!("Failed to save Firebase project locally: {}", e))?;
                synced_count += 1;
            }
            // If equal timestamps, they're in sync already
        } else {
            // Project only exists locally - upload
            upload_project_to_firebase(&client, &base_url, id_token, local_proj).await?;
            synced_count += 1;
        }
    }

    // Download Firebase projects that don't exist locally
    for firebase_proj in &firebase_projects {
        if !local_projects.iter().any(|p| p.id == firebase_proj.id) {
            save_project_local(conn, firebase_proj)
                .map_err(|e| format!("Failed to save new Firebase project: {}", e))?;
            synced_count += 1;
        }
    }

    // Mark all projects as synced
    let all_project_ids: Vec<String> = local_projects.iter().map(|p| p.id.clone()).collect();
    mark_synced(conn, &all_project_ids)
        .map_err(|e| format!("Failed to mark projects as synced: {}", e))?;

    Ok(synced_count)
}

/// Fetch all projects from Firebase Firestore
async fn fetch_firebase_projects(
    client: &Client,
    base_url: &str,
    id_token: &str,
) -> Result<Vec<AppProject>, String> {
    let response = client
        .get(base_url)
        .bearer_auth(id_token)
        .send()
        .await
        .map_err(|e| format!("Firebase fetch error: {}", e))?;

    if response.status().as_u16() == 404 {
        // Collection doesn't exist yet - return empty vec
        return Ok(Vec::new());
    }

    if !response.status().is_success() {
        return Err(format!("Firebase returned status: {}", response.status()));
    }

    let data: FirestoreListResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Firebase response: {}", e))?;

    let mut projects = Vec::new();
    if let Some(docs) = data.documents {
        for doc in docs {
            if let Some(project) = firestore_doc_to_project(doc) {
                projects.push(project);
            }
        }
    }

    Ok(projects)
}

/// Convert Firestore document to AppProject
fn firestore_doc_to_project(doc: FirestoreDocument) -> Option<AppProject> {
    let f = doc.fields;

    let id = f.id?.stringValue?;
    let name = f.name?.stringValue?;
    let description = f.description?.stringValue.unwrap_or_default();
    let status_str = f.status?.stringValue?;
    let version = f.version?.stringValue.unwrap_or_else(|| "1.0.0".to_string());
    let archived = f.archived?.booleanValue.unwrap_or(false);
    let created_at = f.created_at?.stringValue?
        .parse().unwrap_or_else(|_| chrono::Utc::now());
    let updated_at = f.updated_at?.stringValue?
        .parse().unwrap_or_else(|_| chrono::Utc::now());

    let status = match status_str.as_str() {
        "making" => AppStatus::Making,
        "testing" => AppStatus::Testing,
        "done" => AppStatus::Done,
        _ => AppStatus::Want,
    };

    Some(AppProject {
        id,
        name,
        description,
        status,
        version,
        archived,
        created_at,
        updated_at,
    })
}

/// Upload a project to Firebase Firestore
async fn upload_project_to_firebase(
    client: &Client,
    base_url: &str,
    id_token: &str,
    project: &AppProject,
) -> Result<(), String> {
    let doc_url = format!("{}/{}", base_url, project.id);
    let doc_data = project_to_firebase_doc(project);

    let response = client
        .patch(&doc_url)
        .bearer_auth(id_token)
        .json(&doc_data)
        .send()
        .await
        .map_err(|e| format!("Firebase upload error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Firebase upload failed with status: {}", response.status()));
    }

    Ok(())
}

/// Delete a project from Firebase Firestore
pub async fn delete_project_from_firebase(
    id_token: &str,
    user_id: &str,
    project_id: &str,
) -> Result<(), String> {
    let project_id_env = std::env::var("VITE_FIREBASE_PROJECT_ID")
        .map_err(|_| "Missing VITE_FIREBASE_PROJECT_ID".to_string())?;
    
    let doc_url = format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/projects/{}",
        project_id_env, user_id, project_id
    );

    let client = Client::new();
    let response = client
        .delete(&doc_url)
        .bearer_auth(id_token)
        .send()
        .await
        .map_err(|e| format!("Firebase delete error: {}", e))?;

    if !response.status().is_success() && response.status().as_u16() != 404 {
        return Err(format!("Firebase delete failed with status: {}", response.status()));
    }

    Ok(())
}
