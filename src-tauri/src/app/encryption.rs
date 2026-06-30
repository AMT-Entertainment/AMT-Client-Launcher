use aes_gcm::aead::{Aead, KeyInit, OsRng, rand_core::RngCore};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

use crate::LAUNCHER_DIRECTORY;

/// Derive a 256-bit key from the user's profile identifier
fn derive_key(profile_id: &str) -> [u8; 32] {
    let mut hasher = Sha1::new();
    hasher.update(b"amt-client-v1-encryption-key-");
    hasher.update(profile_id.as_bytes());
    let hash = hasher.finalize();

    let mut key = [0u8; 32];
    key[..20].copy_from_slice(&hash);
    // Fill remaining bytes with a second hash
    let mut hasher2 = Sha1::new();
    hasher2.update(b"amt-client-v1-encryption-salt-");
    hasher2.update(profile_id.as_bytes());
    let hash2 = hasher2.finalize();
    key[20..].copy_from_slice(&hash2[..12]);
    key
}

/// Encrypt a plaintext string using AES-256-GCM.
/// Returns base64-encoded ciphertext with nonce prepended.
pub fn encrypt(plaintext: &str, profile_id: &str) -> Result<String, String> {
    let key = derive_key(profile_id);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

/// Decrypt a base64-encoded ciphertext that was encrypted with `encrypt`.
pub fn decrypt(encoded: &str, profile_id: &str) -> Result<String, String> {
    let key = derive_key(profile_id);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let combined = BASE64.decode(encoded).map_err(|e| e.to_string())?;
    if combined.len() < 12 {
        return Err("Invalid ciphertext: too short".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

/// Get or create the encryption key file for a user profile.
/// Returns the path to the key hint file (not the actual key, which is derived).
pub fn key_hint_path(profile_id: &str) -> PathBuf {
    LAUNCHER_DIRECTORY
        .config_dir()
        .join("amt_keys")
        .join(format!("{}.hint", profile_id))
}

/// Mark that encryption keys have been initialized for this profile.
pub async fn init_profile_keys(profile_id: &str) -> Result<(), String> {
    let path = key_hint_path(profile_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create keys directory: {}", e))?;
    }
    if !path.exists() {
        fs::write(&path, b"initialized")
            .await
            .map_err(|e| format!("Failed to write key hint: {}", e))?;
        info!("Encryption keys initialized for profile: {}", profile_id);
    }
    Ok(())
}

/// Check if encryption is set up for a given profile
pub async fn has_profile_keys(profile_id: &str) -> bool {
    key_hint_path(profile_id).exists()
}
