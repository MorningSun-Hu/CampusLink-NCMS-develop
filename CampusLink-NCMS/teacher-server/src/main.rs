mod app;
mod api;
mod domain;
mod infrastructure;
mod scheduler;
mod middleware;
mod discovery;
mod crypto_util;
mod proto;

use anyhow::Result;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app::Config;
use api::create_app;
use infrastructure::create_sqlite_pool;

const DEFAULT_TEACHER_CONFIG: &str = r#"[server]
host = "0.0.0.0"
port = 8080

[database]
driver = "sqlite"
url = "sqlite:data/campuslink.db"

[security]
fingerprint_seed = "campuslink-teacher-fingerprint-seed"
aes_key_rotation_days = 30
"#;

fn init_runtime() -> Result<()> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    std::env::set_current_dir(&exe_dir)?;
    for dir in ["data", "logs", "config"] {
        std::fs::create_dir_all(exe_dir.join(dir))?;
    }
    let cfg = exe_dir.join("config").join("config.toml");
    if !cfg.exists() {
        std::fs::write(&cfg, DEFAULT_TEACHER_CONFIG)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_runtime()?;

    // File appender with daily rotation, keep 14 days
    let file_appender = tracing_appender::rolling::daily("logs", "teacher-server");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "teacher_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_writer(file_appender))
        .init();

    let config = Config::load()?;
    info!("Configuration loaded successfully");

    let pool = create_sqlite_pool(&config.database.url).await?;
    info!("Database connection pool created");

    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Database migrations applied");

    let app = create_app(&pool, config.database.url.clone()).await;

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
