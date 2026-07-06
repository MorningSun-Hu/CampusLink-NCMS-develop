use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

pub fn encrypt_message(plaintext: &str, session_key: &str) -> Result<Vec<u8>, String> {
    let key_bytes = hex::decode(session_key).map_err(|e| format!("hex decode: {}", e))?;
    if key_bytes.len() != 32 {
        return Err("session_key must be 32 bytes".into());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("cipher init: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("encrypt: {}", e))?;

    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt_message(data: &[u8], session_key: &str) -> Result<String, String> {
    if data.len() < 12 {
        return Err("encrypted data too short".into());
    }

    let key_bytes = hex::decode(session_key).map_err(|e| format!("hex decode: {}", e))?;
    if key_bytes.len() != 32 {
        return Err("session_key must be 32 bytes".into());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("cipher init: {}", e))?;

    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("decrypt: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("utf8: {}", e))
}
