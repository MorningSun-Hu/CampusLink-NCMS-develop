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
        // 优先使用环境变量
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            Ok(Self::default_with_db_url(&db_url))
        } else {
            // 回退到配置文件
            Self::load_from_file()
        }
    }

    fn load_from_file() -> Result<Self> {
        let config_path = std::env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config/config.toml".to_string());

        let settings = config::Config::builder()
            .add_source(config::File::with_name(&config_path))
            .build()?;

        let config: Config = settings.try_deserialize()?;
        Ok(config)
    }

    fn default_with_db_url(db_url: &str) -> Self {
        Config {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            database: DatabaseConfig {
                driver: "sqlite".to_string(),
                url: db_url.to_string(),
            },
            security: SecurityConfig {
                fingerprint_seed: "campus-link-2026".to_string(),
                aes_key_rotation_days: 30,
            },
        }
    }
}
