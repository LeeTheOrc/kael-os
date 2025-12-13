#![allow(dead_code)]

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key,
};
use aes_gcm::Nonce;
use aes_gcm::aead::consts::U12;
use pbkdf2::pbkdf2_hmac;
use rand::Rng;
use sha2::Sha256;
use std::error::Error;
use base64::{Engine, engine::general_purpose};

const B64_ENGINE: general_purpose::GeneralPurpose = general_purpose::STANDARD;

/// Derive a 256-bit key from a passphrase using PBKDF2
pub fn derive_key_from_passphrase(passphrase: &str, salt: &[u8; 16]) -> Key<Aes256Gcm> {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, 100_000, &mut key);
    Key::<Aes256Gcm>::from(key)
}

/// Encrypt data with AES-256-GCM using a passphrase
/// Returns: (salt + nonce + ciphertext) base64 encoded
pub fn encrypt_with_passphrase(
    plaintext: &str,
    passphrase: &str,
) -> Result<String, Box<dyn Error>> {
    let mut rng = rand::thread_rng();

    // Generate random 16-byte salt
    let salt: [u8; 16] = rng.gen();

    // Generate random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes);
    let nonce = Nonce::<U12>::from_slice(&nonce_bytes);

    // Derive key from passphrase
    let key = derive_key_from_passphrase(passphrase, &salt);
    let cipher = Aes256Gcm::new(&key);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Combine salt + nonce + ciphertext and encode
    let mut combined = Vec::new();
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(B64_ENGINE.encode(&combined))
}

    // Decrypt data with AES-256-GCM using a passphrase
pub fn decrypt_with_passphrase(
    encrypted_data: &str,
    passphrase: &str,
) -> Result<String, Box<dyn Error>> {
    // Decode base64
    let combined = B64_ENGINE.decode(encrypted_data)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    if combined.len() < 28 {
        return Err("Invalid encrypted data: too short".into());
    }

    // Extract salt (first 16 bytes)
    let salt: [u8; 16] = combined[0..16].try_into()?;

    // Extract nonce (next 12 bytes)
    let nonce = Nonce::<U12>::from_slice(&combined[16..28]);

    // Extract ciphertext (rest)
    let ciphertext = &combined[28..];

    // Derive key from passphrase using same salt
    let key = derive_key_from_passphrase(passphrase, &salt);
    let cipher = Aes256Gcm::new(&key);

    // Decrypt
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    Ok(String::from_utf8(plaintext_bytes)?)
}

/// Encrypt with a derived key directly (e.g., from id_token)
pub fn encrypt_with_key(plaintext: &str, key: &str) -> Result<String, Box<dyn Error>> {
    let mut rng = rand::thread_rng();

    // Hash the key to get consistent 256-bit value
    let mut key_bytes = [0u8; 32];
    pbkdf2_hmac::<Sha256>(key.as_bytes(), b"kael-os-key", 100_000, &mut key_bytes);
    let key = Key::<Aes256Gcm>::from(key_bytes);

    // Generate random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes);
    let nonce = Nonce::<U12>::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new(&key);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Combine nonce + ciphertext and encode
    let mut combined = Vec::new();
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(B64_ENGINE.encode(&combined))
}

/// Decrypt with a derived key
pub fn decrypt_with_key(encrypted_data: &str, key: &str) -> Result<String, Box<dyn Error>> {
    // Decode base64
    let combined = B64_ENGINE.decode(encrypted_data)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    if combined.len() < 12 {
        return Err("Invalid encrypted data: too short".into());
    }

    // Extract nonce (first 12 bytes)
    let nonce = Nonce::<U12>::from_slice(&combined[0..12]);

    // Extract ciphertext (rest)
    let ciphertext = &combined[12..];

    // Hash the key
    let mut key_bytes = [0u8; 32];
    pbkdf2_hmac::<Sha256>(key.as_bytes(), b"kael-os-key", 100_000, &mut key_bytes);
    let key = Key::<Aes256Gcm>::from(key_bytes);

    let cipher = Aes256Gcm::new(&key);

    // Decrypt
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    Ok(String::from_utf8(plaintext_bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_with_passphrase() {
        let plaintext = "super secret api key";
        let passphrase = "my-strong-password";

        let encrypted = encrypt_with_passphrase(plaintext, passphrase).unwrap();
        let decrypted = decrypt_with_passphrase(&encrypted, passphrase).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_with_key() {
        let plaintext = "another secret";
        let key = "id_token_value";

        let encrypted = encrypt_with_key(plaintext, key).unwrap();
        let decrypted = decrypt_with_key(&encrypted, key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let plaintext = "secret";
        let passphrase1 = "correct-password";
        let passphrase2 = "wrong-password";

        let encrypted = encrypt_with_passphrase(plaintext, passphrase1).unwrap();
        let result = decrypt_with_passphrase(&encrypted, passphrase2);

        assert!(result.is_err());
    }
}
