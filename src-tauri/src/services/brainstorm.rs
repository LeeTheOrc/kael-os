use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrainstormIdea {
    pub id: String,
    pub category: String,
    pub prompt: String,
    pub ideas: String,
    pub generated_at: Option<String>,
    pub starred: bool,
    pub on_demand: bool,
}

#[derive(Debug, Serialize)]
struct OnDemandRequest {
    category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_prompt: Option<String>,
}

#[derive(Debug, Serialize)]
struct ToggleStarRequest {
    idea_id: String,
    starred: bool,
}

/// Fetch all brainstorm ideas from Firestore cache
pub async fn fetch_brainstorm_ideas(user: &crate::auth::User) -> Result<Vec<BrainstormIdea>, String> {
    log::info!("📥 Fetching brainstorm ideas from Firestore...");
    
    let url = format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/brainstorm_cache",
        std::env::var("FIREBASE_PROJECT_ID").unwrap_or_else(|_| "your-project-id".to_string())
    );
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", user.id_token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch ideas: {}", e))?;
    
    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        return Err(format!("Firestore error: {}", error));
    }
    
    let data: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let mut ideas = Vec::new();
    
    if let Some(documents) = data.get("documents").and_then(|d| d.as_array()) {
        for doc in documents {
            if let Some(fields) = doc.get("fields") {
                let idea = BrainstormIdea {
                    id: doc.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.split('/').last().unwrap_or("").to_string())
                        .unwrap_or_default(),
                    category: fields.get("category")
                        .and_then(|f| f.get("stringValue"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    prompt: fields.get("prompt")
                        .and_then(|f| f.get("stringValue"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    ideas: fields.get("ideas")
                        .and_then(|f| f.get("stringValue"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    generated_at: fields.get("generated_at")
                        .and_then(|f| f.get("timestampValue"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    starred: fields.get("starred")
                        .and_then(|f| f.get("booleanValue"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    on_demand: fields.get("on_demand")
                        .and_then(|f| f.get("booleanValue"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                };
                ideas.push(idea);
            }
        }
    }
    
    log::info!("✅ Fetched {} brainstorm ideas", ideas.len());
    Ok(ideas)
}

/// Request new ideas on-demand from Cloud Function
pub async fn request_new_ideas(
    user: &crate::auth::User,
    category: &str,
    custom_prompt: Option<String>,
) -> Result<BrainstormIdea, String> {
    log::info!("🎯 Requesting new {} ideas...", category);
    
    let project_id = std::env::var("FIREBASE_PROJECT_ID")
        .unwrap_or_else(|_| "your-project-id".to_string());
    let region = "us-central1"; // Change if deployed to different region
    
    let url = format!(
        "https://{}-{}.cloudfunctions.net/onDemandBrainstorm",
        region, project_id
    );
    
    let request = OnDemandRequest {
        category: category.to_string(),
        custom_prompt: custom_prompt.clone(),
    };
    
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", user.id_token))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to call function: {}", e))?;
    
    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        return Err(format!("Function error: {}", error));
    }
    
    let result: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let idea = BrainstormIdea {
        id: result.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        category: result.get("category")
            .and_then(|v| v.as_str())
            .unwrap_or(category)
            .to_string(),
        prompt: custom_prompt.unwrap_or_default(),
        ideas: result.get("ideas")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        generated_at: Some(chrono::Utc::now().to_rfc3339()),
        starred: false,
        on_demand: true,
    };
    
    log::info!("✅ Generated new ideas: {}", idea.id);
    Ok(idea)
}

/// Toggle star status on an idea
pub async fn toggle_star_idea(
    user: &crate::auth::User,
    idea_id: &str,
    starred: bool,
) -> Result<(), String> {
    log::info!("⭐ {} idea {}...", if starred { "Starring" } else { "Unstarring" }, idea_id);
    
    let project_id = std::env::var("FIREBASE_PROJECT_ID")
        .unwrap_or_else(|_| "your-project-id".to_string());
    let region = "us-central1";
    
    let url = format!(
        "https://{}-{}.cloudfunctions.net/toggleStarIdea",
        region, project_id
    );
    
    let request = ToggleStarRequest {
        idea_id: idea_id.to_string(),
        starred,
    };
    
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", user.id_token))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to call function: {}", e))?;
    
    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        return Err(format!("Function error: {}", error));
    }
    
    log::info!("✅ Idea {} updated", idea_id);
    Ok(())
}

/// Cache ideas locally for offline access
pub fn cache_ideas_locally(ideas: &[BrainstormIdea]) -> Result<(), String> {
    let cache_path = "/tmp/kael_brainstorm_cache.json";
    let json = serde_json::to_string_pretty(ideas)
        .map_err(|e| format!("Failed to serialize ideas: {}", e))?;
    
    std::fs::write(cache_path, json)
        .map_err(|e| format!("Failed to write cache: {}", e))?;
    
    log::info!("💾 Cached {} ideas locally", ideas.len());
    Ok(())
}

/// Load cached ideas from local storage
pub fn load_cached_ideas() -> Result<Vec<BrainstormIdea>, String> {
    let cache_path = "/tmp/kael_brainstorm_cache.json";
    
    if !std::path::Path::new(cache_path).exists() {
        return Ok(Vec::new());
    }
    
    let json = std::fs::read_to_string(cache_path)
        .map_err(|e| format!("Failed to read cache: {}", e))?;
    
    let ideas: Vec<BrainstormIdea> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse cache: {}", e))?;
    
    log::info!("📂 Loaded {} cached ideas", ideas.len());
    Ok(ideas)
}
