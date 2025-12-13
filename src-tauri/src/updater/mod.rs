#![allow(dead_code)]

// src-tauri/src/updater/mod.rs
use serde::{Deserialize, Serialize};
use std::error::Error;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub released: DateTime<Utc>,
    pub changelog: String,
    pub platforms: PlatformReleases,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformReleases {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<PlatformInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linux: Option<PlatformInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos: Option<PlatformInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android: Option<PlatformInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub mirrors: Vec<String>,
    pub signature_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckRequest {
    pub platform: String,
    pub arch: String,
    pub current_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResponse {
    pub update_available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub version_info: Option<VersionInfo>,
    pub error: Option<String>,
}

/// Platform detection
pub fn detect_platform() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else {
        "unknown".to_string()
    }
}

pub fn detect_arch() -> String {
    if cfg!(target_arch = "x86_64") {
        "x86_64".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "aarch64".to_string()
    } else if cfg!(target_arch = "arm") {
        "armv7".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Check for updates from the version server
pub async fn check_for_updates(
    current_version: &str,
    update_server_url: &str,
) -> Result<UpdateCheckResponse, Box<dyn Error>> {
    let platform = detect_platform();
    let arch = detect_arch();

    let url = format!(
        "{}/check?platform={}&arch={}&version={}",
        update_server_url, platform, arch, current_version
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    let check_response: UpdateCheckResponse = response.json().await?;

    Ok(check_response)
}

/// Fetch full version manifest
pub async fn fetch_manifest(
    manifest_url: &str,
) -> Result<VersionInfo, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(manifest_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    let manifest: VersionInfo = response.json().await?;

    Ok(manifest)
}

/// Download from mirrors with fallback
pub async fn download_from_mirrors(
    mirrors: &[String],
    filename: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut last_error = None;

    for mirror in mirrors {
        let url = format!("{}/{}", mirror, filename);
        log::info!("Attempting download from: {}", url);

        match reqwest::Client::new()
            .get(&url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                log::info!("Successfully downloaded from mirror: {}", mirror);
                return Ok(response.bytes().await?.to_vec());
            }
            Ok(response) => {
                let error = format!("HTTP {}: {}", response.status(), mirror);
                log::warn!("{}", error);
                last_error = Some(error);
            }
            Err(e) => {
                let error = format!("Connection error: {} ({})", mirror, e);
                log::warn!("{}", error);
                last_error = Some(error);
            }
        }
    }

    Err(format!(
        "All mirrors failed. Last error: {}",
        last_error.unwrap_or_else(|| "Unknown error".to_string())
    )
    .into())
}

/// Verify SHA256 checksum
pub fn verify_checksum(data: &[u8], expected_sha256: &str) -> Result<bool, Box<dyn Error>> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let computed = format!("{:x}", result);

    Ok(computed.to_lowercase() == expected_sha256.to_lowercase())
}

/// Compare versions (returns true if new_version > current_version)
pub fn should_update(current: &str, new: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .take(3)
            .map(|part| part.parse::<u32>().unwrap_or(0))
            .collect()
    };

    let current_parts = parse_version(current);
    let new_parts = parse_version(new);

    for i in 0..3 {
        let c = current_parts.get(i).copied().unwrap_or(0);
        let n = new_parts.get(i).copied().unwrap_or(0);

        if n > c {
            return true;
        } else if n < c {
            return false;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(should_update("0.1.0", "0.2.0"));
        assert!(should_update("0.1.0", "0.1.1"));
        assert!(!should_update("0.2.0", "0.1.0"));
        assert!(!should_update("0.1.0", "0.1.0"));
    }

    #[test]
    fn test_sha256_verification() {
        let data = b"test data";
        let correct_hash = "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9";
        assert!(verify_checksum(data, correct_hash).unwrap());

        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(!verify_checksum(data, wrong_hash).unwrap());
    }
}
