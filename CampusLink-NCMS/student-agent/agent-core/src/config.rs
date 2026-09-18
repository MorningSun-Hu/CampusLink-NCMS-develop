use serde::{Deserialize, Serialize};
use tracing::info;

use crate::crypto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub teacher_server_url: String,
    pub device_id: Option<String>,
    pub teacher_fingerprint: Option<String>,
    pub current_mode: String,
    pub heartbeat_interval_seconds: u64,
    pub student_id: Option<String>,
    pub student_no: Option<String>,
    pub student_name: Option<String>,
    pub auth_token: Option<String>,
    pub token_expires_at: Option<String>,
    pub lock_password: Option<String>,
    pub session_key: Option<String>,
    #[serde(default)]
    pub mode_before_lock: Option<String>,
    #[serde(default)]
    pub lock_is_overlay: bool,
    #[serde(skip)]
    pub is_locked: bool,
    #[serde(skip)]
    pub lock_pid: Option<u32>,
}

impl Config {
    pub fn default() -> Self {
        Config {
            teacher_server_url: "http://localhost:8080".to_string(),
            device_id: None,
            teacher_fingerprint: None,
            current_mode: "open".to_string(),
            heartbeat_interval_seconds: 15,
            student_id: None,
            student_no: None,
            student_name: None,
            auth_token: None,
            token_expires_at: None,
            lock_password: Some("admin123".to_string()),
            session_key: None,
            mode_before_lock: None,
            lock_is_overlay: false,
            is_locked: false,
            lock_pid: None,
        }
    }

    pub fn load() -> anyhow::Result<Self> {
        let password = std::env::var("LOCK_PASSWORD")
            .unwrap_or_else(|_| "admin123".to_string());

        match crypto::load_encrypted(&password) {
            Ok(content) => {
                info!("Configuration loaded (encrypted)");
                let config: Config = serde_json::from_str(&content)?;
                Ok(config)
            }
            Err(_) => {
                let config = Config::default();
                let json = serde_json::to_string_pretty(&config)?;
                let _ = crypto::save_encrypted(&json, &password);
                Ok(config)
            }
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        let password = self.lock_password.as_ref().map(|s| s.as_str()).unwrap_or("admin123");
        crypto::save_encrypted(&content, password).map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}
