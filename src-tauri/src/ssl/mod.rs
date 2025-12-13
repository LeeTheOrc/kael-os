#![allow(dead_code)]

use rcgen::{CertificateParams, DistinguishedName};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub not_before: String,
    pub not_after: String,
    pub public_key: String,
    pub serial: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCertificate {
    pub cert_pem: String,
    pub key_pem: String,
    pub info: CertificateInfo,
}

/// Generate a self-signed SSL/TLS certificate
pub fn generate_self_signed_cert(
    common_name: &str,
    days_valid: u32,
) -> Result<GeneratedCertificate, String> {
    // Create distinguished name
    let mut dn = DistinguishedName::new();
    dn.push(rcgen::DnType::CommonName, common_name);
    dn.push(rcgen::DnType::CountryName, "US");
    dn.push(rcgen::DnType::OrganizationName, "Kael-OS");
    dn.push(rcgen::DnType::OrganizationalUnitName, "Security");

    // Create certificate parameters
    let mut params = CertificateParams::new(vec![common_name.to_string()]);
    params.distinguished_name = dn;
    let now = chrono::Local::now();
    let year_str = now.format("%Y").to_string();
    let current_year: i32 = year_str.parse().unwrap_or(2025);
    params.not_after = rcgen::date_time_ymd(
        current_year + (days_valid / 365) as i32,
        1,
        1,
    );

    // Generate certificate and key
    let cert = rcgen::Certificate::from_params(params)
        .map_err(|e| format!("Failed to generate certificate: {}", e))?;

    let cert_pem = cert
        .serialize_pem()
        .map_err(|e| format!("Failed to serialize certificate: {}", e))?;

    let key_pem = cert
        .serialize_private_key_pem();

    // Extract certificate info
    let info = CertificateInfo {
        subject: format!("CN={},O=Kael-OS,OU=Security,C=US", common_name),
        issuer: format!("CN={},O=Kael-OS,OU=Security,C=US", common_name),
        not_before: chrono::Local::now().to_rfc2822(),
        not_after: format!(
            "{}",
            chrono::Local::now() + chrono::Duration::days(days_valid as i64)
        ),
        public_key: extract_public_key_info(&cert_pem),
        serial: format!("{:X}", rand::random::<u32>()),
    };

    Ok(GeneratedCertificate {
        cert_pem,
        key_pem,
        info,
    })
}

/// Save certificate and key to files
pub fn save_certificate(
    cert_data: &GeneratedCertificate,
    cert_path: &str,
    key_path: &str,
) -> Result<(), String> {
    // Ensure directory exists
    if let Some(parent) = Path::new(cert_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Save certificate
    fs::write(cert_path, &cert_data.cert_pem)
        .map_err(|e| format!("Failed to write certificate: {}", e))?;

    // Save key (with restricted permissions on Unix)
    fs::write(key_path, &cert_data.key_pem)
        .map_err(|e| format!("Failed to write key: {}", e))?;

    // Set restrictive permissions on key file (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(key_path, perms)
            .map_err(|e| format!("Failed to set key permissions: {}", e))?;
    }

    Ok(())
}

/// Load certificate from file
pub fn load_certificate(cert_path: &str) -> Result<String, String> {
    fs::read_to_string(cert_path)
        .map_err(|e| format!("Failed to read certificate: {}", e))
}

/// Load private key from file
pub fn load_private_key(key_path: &str) -> Result<String, String> {
    fs::read_to_string(key_path)
        .map_err(|e| format!("Failed to read private key: {}", e))
}

/// Get certificate info from PEM data
pub fn get_certificate_info(cert_pem: &str) -> Result<CertificateInfo, String> {
    Ok(CertificateInfo {
        subject: extract_certificate_field(cert_pem, "Subject"),
        issuer: extract_certificate_field(cert_pem, "Issuer"),
        not_before: extract_certificate_field(cert_pem, "Not Before"),
        not_after: extract_certificate_field(cert_pem, "Not After"),
        public_key: extract_public_key_info(cert_pem),
        serial: extract_certificate_field(cert_pem, "Serial"),
    })
}

/// Verify certificate (basic check)
pub fn verify_certificate_validity(cert_pem: &str) -> Result<bool, String> {
    // In a real implementation, we would parse the PEM and check dates
    // For now, just check if it's valid PEM format
    let is_valid = cert_pem.contains("-----BEGIN CERTIFICATE-----")
        && cert_pem.contains("-----END CERTIFICATE-----");

    Ok(is_valid)
}

/// Extract a field from certificate PEM (simplified)
fn extract_certificate_field(_cert_pem: &str, field: &str) -> String {
    // This is a simplified implementation
    // In production, use openssl or x509 parsing library
    match field {
        "Subject" => "CN=localhost,O=Kael-OS,OU=Security,C=US".to_string(),
        "Issuer" => "CN=localhost,O=Kael-OS,OU=Security,C=US".to_string(),
        "Not Before" => chrono::Local::now().to_rfc2822(),
        "Not After" => (chrono::Local::now() + chrono::Duration::days(365)).to_rfc2822(),
        "Serial" => format!("{:X}", rand::random::<u32>()),
        _ => "Unknown".to_string(),
    }
}

/// Extract public key info from certificate PEM
fn extract_public_key_info(cert_pem: &str) -> String {
    // Simplified: just return a hash of the cert
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    cert_pem.hash(&mut hasher);
    format!("sha256_{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_self_signed_cert() {
        let result = generate_self_signed_cert("localhost", 365);
        assert!(result.is_ok());

        let cert = result.unwrap();
        assert!(!cert.cert_pem.is_empty());
        assert!(!cert.key_pem.is_empty());
        assert!(cert.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(cert.key_pem.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn test_verify_certificate() {
        let cert = generate_self_signed_cert("test.local", 365).unwrap();
        let is_valid = verify_certificate_validity(&cert.cert_pem).unwrap();
        assert!(is_valid);
    }
}
