use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = env::var("WEBDAV_SERVER").unwrap_or_else(|_| "leroyonline.co.za".into());
    let port = env::var("WEBDAV_PORT").unwrap_or_else(|_| "2078".into());
    let username = env::var("WEBDAV_USERNAME").unwrap_or_else(|_| "leetheorc".into());
    let password = env::var("WEBDAV_PASSWORD").unwrap_or_else(|_| "".into());
    let ssl = env::var("WEBDAV_SSL").unwrap_or_else(|_| "true".into()) == "true";
    let base_path = env::var("WEBDAV_BASE_PATH").unwrap_or_else(|_| "/public_html".into());

    let scheme = if ssl { "https" } else { "http" };
    let base_url = format!("{scheme}://{server}:{port}");

    let tmp_dir = std::env::temp_dir();
    let mut local_file: PathBuf = tmp_dir.clone();
    local_file.push("kael-os-webdav-test.txt");
    std::fs::write(&local_file, b"Hello from Kael-OS WebDAV test!\n")?;

    let remote_path = format!("{}/kael-os-webdav-test.txt", base_path);

    println!("Uploading {} to {}", local_file.display(), remote_path);

    // Perform a WebDAV PUT directly using reqwest
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .build()?;

    let url = format!("{}/{}", base_url.trim_end_matches('/'), remote_path.trim_start_matches('/'));
    let bytes = std::fs::read(&local_file)?;

    let resp = client
        .put(url.clone())
        .basic_auth(username, Some(password))
        .header("Content-Type", "text/plain")
        .body(bytes)
        .send()
        .await?;

    if !resp.status().is_success() {
        eprintln!("Upload failed: {}", resp.status());
        let text = resp.text().await.unwrap_or_default();
        eprintln!("Response body: {}", text);
        return Err("WebDAV upload failed".into());
    }

    println!("Upload complete.");
    println!("If your docroot is public_html, it may be accessible at: https://{}/kael-os-webdav-test.txt", server);
    Ok(())
}
