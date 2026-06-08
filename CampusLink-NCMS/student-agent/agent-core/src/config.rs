use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub teacher_server_url: String,
    pub device_id: Option<String>,
    pub teacher_fingerprint: Option<String>,
    pub current_mode: String,
    pub heartbeat_interval_seconds: u64,
}

impl Config {
    pub fn default() -> Self {
        Config {
            teacher_server_url: "http://localhost:8080".to_string(),
            device_id: None,
            teacher_fingerprint: None,
            current_mode: "open".to_string(),
            heartbeat_interval_seconds: 15,
        }
    }

    pub fn load() -> anyhow::Result<Self> {
        let config_path = "config/config.json";
        if let Ok(content) = std::fs::read_to_string(config_path) {
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::create_dir_all("config")?;
        std::fs::write("config/config.json", content)?;
        Ok(())
    }
}
