#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use reqwest::Client;

use crate::auth::{decrypt_secret, encrypt_secret};

fn project_id() -> Result<String, String> {
    std::env::var("VITE_FIREBASE_PROJECT_ID").map_err(|_| "Missing VITE_FIREBASE_PROJECT_ID".to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct User {
    pub uid: String,
    pub email: String,
}

#[derive(Deserialize)]
struct FirestoreList {
    documents: Option<Vec<FirestoreDoc>>,    
}

#[derive(Deserialize)]
struct FirestoreDoc {
    name: String,
    fields: Option<FirestoreFields>,
}

#[derive(Deserialize)]
struct FirestoreFields {
    name: Option<FirestoreString>,
    value: Option<FirestoreString>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct FirestoreString { stringValue: String }

pub async fn get_api_keys(user: &crate::auth::User) -> Result<Vec<ApiKey>, String> {
    let project = project_id()?;
    let url = format!("https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/api_keys",
        project, user.uid);
    let client = Client::new();
    let resp = client
        .get(url)
        .bearer_auth(&user.id_token)
        .send()
        .await
        .map_err(|e| format!("Firestore network error: {}", e))?;
    if !resp.status().is_success() {
        // Empty collection returns 404; treat as empty
        return Ok(vec![]);
    }
    let data: FirestoreList = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    let mut out = vec![];
    if let Some(docs) = data.documents {
        for d in docs {
            if let Some(f) = d.fields {
                let id = d.name.split('/').last().unwrap_or("").to_string();
                let name = f.name.map(|x| x.stringValue).unwrap_or_default();
                let value_enc = f.value.map(|x| x.stringValue).unwrap_or_default();
                let value = decrypt_secret(user, &value_enc).unwrap_or_default();
                out.push(ApiKey { id, name, value });
            }
        }
    }
    Ok(out)
}

#[derive(Serialize)]
struct FirestoreWrite<'a> {
    fields: FirestoreWriteFields<'a>,
}

#[derive(Serialize)]
struct FirestoreWriteFields<'a> {
    name: FirestoreWriteString<'a>,
    value: FirestoreWriteString<'a>,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct FirestoreWriteString<'a> { stringValue: &'a str }

pub async fn save_api_key(user: &crate::auth::User, name: &str, plaintext_value: &str) -> Result<(), String> {
    let project = project_id()?;
    let doc_id = name.to_lowercase().replace(' ', "_");
    let url = format!("https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/api_keys?documentId={}",
        project, user.uid, doc_id);
    let client = Client::new();
    let encrypted = encrypt_secret(user, plaintext_value);
    let body = FirestoreWrite { fields: FirestoreWriteFields { name: FirestoreWriteString { stringValue: name }, value: FirestoreWriteString { stringValue: &encrypted } } };
    let resp = client
        .post(url)
        .bearer_auth(&user.id_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Firestore network error: {}", e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Firestore save error {}: {}", status, text));
    }
    Ok(())
}

pub async fn delete_api_key(user: &crate::auth::User, id: &str) -> Result<(), String> {
    let project = project_id()?;
    let url = format!("https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/api_keys/{}",
        project, user.uid, id);
    let client = Client::new();
    let resp = client
        .delete(url)
        .bearer_auth(&user.id_token)
        .send()
        .await
        .map_err(|e| format!("Firestore network error: {}", e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Firestore delete error {}: {}", status, text));
    }
    Ok(())
}
