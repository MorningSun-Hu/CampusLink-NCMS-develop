use sqlx::SqlitePool;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub log_type: Option<String>,
    pub device_id: Option<String>,
    pub operator: Option<String>,
    pub action: Option<String>,
    pub detail: Option<String>,
    pub created_at: String,
}

pub async fn query_logs(
    pool: &SqlitePool,
    log_type: Option<&str>,
    device_id: Option<&str>,
    date_from: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<LogEntry>, i64)> {
    let offset = (page - 1) * page_size;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM operation_logs")
        .fetch_one(pool)
        .await?;

    let rows = sqlx::query_as::<_, LogEntry>(
        "SELECT * FROM operation_logs ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let _ = (log_type, device_id, date_from);

    Ok((rows, count.0))
}
