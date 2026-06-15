use anyhow::Result;
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool};
use std::{path::Path, str::FromStr};

pub async fn create_sqlite_pool(database_url: &str) -> Result<SqlitePool> {
    ensure_sqlite_parent_dir(database_url)?;

    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;

    Ok(pool)
}

fn ensure_sqlite_parent_dir(database_url: &str) -> Result<()> {
    let path = database_url
        .strip_prefix("sqlite:///")
        .or_else(|| database_url.strip_prefix("sqlite://"))
        .or_else(|| database_url.strip_prefix("sqlite:"));

    if let Some(path) = path {
        if path != ":memory:" {
            if let Some(parent) = Path::new(path).parent() {
                std::fs::create_dir_all(parent)?;
            }
        }
    }

    Ok(())
}
