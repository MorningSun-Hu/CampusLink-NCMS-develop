use anyhow::{Context, Result};
use sqlx::{sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions}, SqlitePool};
use std::{path::{Path, PathBuf}, str::FromStr, time::Duration};
use tracing::info;

pub async fn create_sqlite_pool(database_url: &str) -> Result<SqlitePool> {
    let resolved_url = resolve_database_url(database_url);
    ensure_sqlite_parent_dir(&resolved_url)?;

    let mut options = SqliteConnectOptions::from_str(&resolved_url)?
        .create_if_missing(true)
        // Wait up to 5s for a busy writer before returning SQLITE_BUSY
        .busy_timeout(Duration::from_millis(5000))
        // Enable WAL so concurrent readers/writers don't block each other
        .journal_mode(SqliteJournalMode::Wal);

    #[cfg(feature = "sqlcipher")]
    {
        options = options.pragma("key", "campuslink_secret_key_2026".to_string());
        info!("SQLCipher encryption enabled (sqlcipher feature active)");
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;

    #[cfg(feature = "sqlcipher")]
    {
        sqlx::query("PRAGMA cipher_memory_security = ON")
            .execute(&pool)
            .await
            .ok();
    }

    Ok(pool)
}

fn resolve_database_url(database_url: &str) -> String {
    let path = database_url
        .strip_prefix("sqlite:///")
        .or_else(|| database_url.strip_prefix("sqlite://"))
        .or_else(|| database_url.strip_prefix("sqlite:"));

    if let Some(path) = path {
        if path != ":memory:" {
            let db_path = Path::new(path);
            if db_path.is_relative() {
                let exe_dir = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                    .unwrap_or_else(|| PathBuf::from("."));

                let absolute = exe_dir.join(db_path);
                let absolute_str = absolute.to_string_lossy().replace('\\', "/");
                return format!("sqlite:///{}", absolute_str);
            }
        }
    }

    database_url.to_string()
}

fn ensure_sqlite_parent_dir(database_url: &str) -> Result<()> {
    let path = database_url
        .strip_prefix("sqlite:///")
        .or_else(|| database_url.strip_prefix("sqlite://"))
        .or_else(|| database_url.strip_prefix("sqlite:"));

    if let Some(path) = path {
        if path != ":memory:" {
            if let Some(parent) = Path::new(path).parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }
    }

    Ok(())
}
