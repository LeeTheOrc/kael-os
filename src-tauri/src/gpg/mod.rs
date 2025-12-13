#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgKey {
    pub key_id: String,
    pub fingerprint: String,
    pub email: String,
    pub name: String,
    pub expires: Option<String>,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedKey {
    pub key_type: String, // "private" or "public"
    pub armored: String,   // ASCII-armored key data
}

/// List all GPG secret keys on the system
pub async fn list_secret_keys() -> Result<Vec<GpgKey>, String> {
    let output = Command::new("gpg")
        .args(&["--list-secret-keys", "--keyid-format=long", "--with-colons"])
        .output()
        .map_err(|e| format!("Failed to run gpg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("GPG error: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let keys = parse_gpg_keys(&stdout);

    Ok(keys)
}

/// List all GPG public keys on the system
pub async fn list_public_keys() -> Result<Vec<GpgKey>, String> {
    let output = Command::new("gpg")
        .args(&["--list-public-keys", "--keyid-format=long", "--with-colons"])
        .output()
        .map_err(|e| format!("Failed to run gpg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("GPG error: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let keys = parse_gpg_keys(&stdout);

    Ok(keys)
}

/// Export a private key in ASCII-armored format
pub async fn export_private_key(key_id: &str) -> Result<ExportedKey, String> {
    let output = Command::new("gpg")
        .args(&["--armor", "--export-secret-keys", key_id])
        .output()
        .map_err(|e| format!("Failed to run gpg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("GPG error: {}", stderr));
    }

    let armored = String::from_utf8_lossy(&output.stdout).to_string();

    if armored.is_empty() {
        return Err("No key found with that ID".to_string());
    }

    Ok(ExportedKey {
        key_type: "private".to_string(),
        armored,
    })
}

/// Export a public key in ASCII-armored format
pub async fn export_public_key(key_id: &str) -> Result<ExportedKey, String> {
    let output = Command::new("gpg")
        .args(&["--armor", "--export", key_id])
        .output()
        .map_err(|e| format!("Failed to run gpg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("GPG error: {}", stderr));
    }

    let armored = String::from_utf8_lossy(&output.stdout).to_string();

    if armored.is_empty() {
        return Err("No key found with that ID".to_string());
    }

    Ok(ExportedKey {
        key_type: "public".to_string(),
        armored,
    })
}

/// Import a key from ASCII-armored format
pub async fn import_key(armored_key: &str) -> Result<String, String> {
    let output = Command::new("gpg")
        .args(&["--import"])
        .arg("--")
        .output()
        .map_err(|e| format!("Failed to run gpg: {}", e))?;

    // Write key data to stdin
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("gpg")
        .args(&["--import"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn gpg: {}", e))?;

    {
        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        stdin
            .write_all(armored_key.as_bytes())
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
    }

    let _output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for gpg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("GPG import error: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

/// Get the fingerprint of a key
pub async fn get_key_fingerprint(key_id: &str) -> Result<String, String> {
    let output = Command::new("gpg")
        .args(&["--fingerprint", "--with-colons", key_id])
        .output()
        .map_err(|e| format!("Failed to run gpg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("GPG error: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse fpr line (fingerprint line)
    for line in stdout.lines() {
        if line.starts_with("fpr") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 9 {
                return Ok(parts[9].to_string());
            }
        }
    }

    Err("Fingerprint not found".to_string())
}

/// Parse GPG output in colon-separated format
fn parse_gpg_keys(output: &str) -> Vec<GpgKey> {
    let mut keys = Vec::new();
    let mut current_key: Option<GpgKey> = None;

    for line in output.lines() {
        let fields: Vec<&str> = line.split(':').collect();

        if fields.is_empty() {
            continue;
        }

        if fields[0] == "sec" || fields[0] == "pub" {
            // Save previous key if exists
            if let Some(key) = current_key.take() {
                keys.push(key);
            }

            // Create new key
            if fields.len() > 4 {
                let key_id = if fields.len() > 4 && !fields[4].is_empty() {
                    fields[4].to_string()
                } else {
                    String::new()
                };

                let created = if fields.len() > 5 && !fields[5].is_empty() {
                    fields[5].to_string()
                } else {
                    String::new()
                };

                let expires = if fields.len() > 6 && !fields[6].is_empty() {
                    Some(fields[6].to_string())
                } else {
                    None
                };

                current_key = Some(GpgKey {
                    key_id,
                    fingerprint: String::new(),
                    email: String::new(),
                    name: String::new(),
                    expires,
                    created,
                });
            }
        } else if fields[0] == "uid" && current_key.is_some() {
            if let Some(ref mut key) = current_key {
                if fields.len() > 9 {
                    let uid = fields[9];
                    // Parse "Name <email>" format
                    if let Some(email_start) = uid.find('<') {
                        if let Some(email_end) = uid.find('>') {
                            key.name = uid[..email_start].trim().to_string();
                            key.email = uid[email_start + 1..email_end].to_string();
                        }
                    } else {
                        key.name = uid.to_string();
                    }
                }
            }
        } else if fields[0] == "fpr" && current_key.is_some() {
            if let Some(ref mut key) = current_key {
                if fields.len() > 9 {
                    key.fingerprint = fields[9].to_string();
                }
            }
        }
    }

    // Don't forget the last key
    if let Some(key) = current_key {
        keys.push(key);
    }

    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gpg_output() {
        let sample_output = r#"sec:-:4096:1:1234567890ABCDEF:1234567890::::scESC:::::::23::0:
uid:u::::1234567890::1234567890ABCDEF::John Doe <john@example.com>::::::::::0:
fpr:::::::::1234567890ABCDEF1234567890ABCDEF::"#;

        let keys = parse_gpg_keys(sample_output);
        assert!(!keys.is_empty());
        let key = &keys[0];
        assert_eq!(key.name, "John Doe");
        assert_eq!(key.email, "john@example.com");
    }
}
