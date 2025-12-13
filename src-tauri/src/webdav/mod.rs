//! WebDAV file transfer module for saving files to cPanel webhosting
//! Provides simple interface for uploading files to WebDAV-enabled servers

#![allow(dead_code)]

use std::path::Path;
use reqwest::Client;
use std::error::Error;

#[derive(Clone, Debug)]
pub struct WebDavConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

pub struct WebDavClient {
    client: Client,
    config: WebDavConfig,
}

impl WebDavClient {
    /// Create a new WebDAV client
    pub fn new(config: WebDavConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Upload a file to WebDAV server
    pub async fn upload_file(
        &self,
        local_path: &Path,
        remote_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        let file_content = tokio::fs::read(local_path).await?;

        let upload_url = format!(
            "{}/{}",
            self.config.url.trim_end_matches('/'),
            remote_path.trim_start_matches('/')
        );

        let response = self
            .client
            .put(&upload_url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .body(file_content)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("WebDAV upload failed: {}", response.status()).into())
        }
    }

    /// Create a directory on WebDAV server
    pub async fn create_directory(&self, remote_path: &str) -> Result<(), Box<dyn Error>> {
        let dir_url = format!(
            "{}/{}",
            self.config.url.trim_end_matches('/'),
            remote_path.trim_start_matches('/')
        );

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"MKCOL")?, &dir_url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 405 {
            // 405 means directory already exists
            Ok(())
        } else {
            Err(format!("WebDAV mkdir failed: {}", response.status()).into())
        }
    }

    /// Download a file from WebDAV server
    pub async fn download_file(
        &self,
        remote_path: &str,
        local_path: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let download_url = format!(
            "{}/{}",
            self.config.url.trim_end_matches('/'),
            remote_path.trim_start_matches('/')
        );

        let response = self
            .client
            .get(&download_url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;

        if response.status().is_success() {
            let content = response.bytes().await?;
            tokio::fs::write(local_path, content).await?;
            Ok(())
        } else {
            Err(format!("WebDAV download failed: {}", response.status()).into())
        }
    }

    /// List files in WebDAV directory
    pub async fn list_directory(
        &self,
        remote_path: &str,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let list_url = format!(
            "{}/{}",
            self.config.url.trim_end_matches('/'),
            remote_path.trim_start_matches('/')
        );

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"PROPFIND")?, &list_url)
            .header("Depth", "1")
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;

        if response.status().is_success() {
            let xml = response.text().await?;
            // Simple XML parsing for file names
            let mut files = Vec::new();
            for line in xml.lines() {
                if let Some(start) = line.find("<D:href>") {
                    if let Some(end) = line[start + 8..].find("</D:href>") {
                        let href = &line[start + 8..start + 8 + end];
                        if !href.ends_with('/') {
                            files.push(href.to_string());
                        }
                    }
                }
            }
            Ok(files)
        } else {
            Err(format!("WebDAV list failed: {}", response.status()).into())
        }
    }

    /// Delete a file from WebDAV server
    pub async fn delete_file(&self, remote_path: &str) -> Result<(), Box<dyn Error>> {
        let delete_url = format!(
            "{}/{}",
            self.config.url.trim_end_matches('/'),
            remote_path.trim_start_matches('/')
        );

        let response = self
            .client
            .delete(&delete_url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 404 {
            Ok(())
        } else {
            Err(format!("WebDAV delete failed: {}", response.status()).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webdav_config_creation() {
        let config = WebDavConfig {
            url: "https://example.com/webdav".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
        };

        assert_eq!(config.username, "user");
    }
}
