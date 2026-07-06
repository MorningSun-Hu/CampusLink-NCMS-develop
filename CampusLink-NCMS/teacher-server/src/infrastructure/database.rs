use anyhow::{Context, Result};
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool};
use std::{path::{Path, PathBuf}, str::FromStr};

pub async fn create_sqlite_pool(database_url: &str) -> Result<SqlitePool> {
    let resolved_url = resolve_database_url(database_url);
    ensure_sqlite_parent_dir(&resolved_url)?;

    let options = SqliteConnectOptions::from_str(&resolved_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;

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
