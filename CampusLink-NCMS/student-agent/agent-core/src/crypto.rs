use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use sha2::{Sha256, Digest};
use rand::RngCore;

const CONFIG_FILE: &str = "config/config.enc";
const LEGACY_CONFIG_FILE: &str = "config/config.json";

pub fn derive_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

pub fn encrypt_config(json: &str, password: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(password);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("cipher init: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, json.as_bytes())
        .map_err(|e| format!("encrypt: {}", e))?;

    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt_config(data: &[u8], password: &str) -> Result<String, String> {
    if data.len() < 12 {
        return Err("encrypted data too short".into());
    }

    let key = derive_key(password);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("cipher init: {}", e))?;

    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("decrypt: {} - password may be wrong", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("utf8: {}", e))
}

pub fn save_encrypted(json: &str, password: &str) -> Result<(), String> {
    let encrypted = encrypt_config(json, password)?;
    std::fs::create_dir_all("config").map_err(|e| format!("mkdir: {}", e))?;
    std::fs::write(CONFIG_FILE, encrypted).map_err(|e| format!("write: {}", e))?;
    Ok(())
}

pub fn load_encrypted(password: &str) -> Result<String, String> {
    if let Ok(data) = std::fs::read(CONFIG_FILE) {
        return decrypt_config(&data, password);
    }
    // Migrate from legacy plaintext config
    if let Ok(json) = std::fs::read_to_string(LEGACY_CONFIG_FILE) {
        save_encrypted(&json, password)?;
        return Ok(json);
    }
    Err("no config file found".into())
}
