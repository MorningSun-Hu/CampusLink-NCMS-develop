use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub driver: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub fingerprint_seed: String,
    pub aes_key_rotation_days: u32,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = std::env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config/config.toml".to_string());

        let settings = config::Config::builder()
            .add_source(config::File::with_name(&config_path))
            .build()?;

        let config: Config = settings.try_deserialize()?;
        Ok(config)
    }
}
