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

    let mut conditions = Vec::new();
    if log_type.is_some() {
        conditions.push("log_type = ?");
    }
    if device_id.is_some() {
        conditions.push("device_id = ?");
    }
    if date_from.is_some() {
        conditions.push("created_at >= ?");
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) as count FROM operation_logs {}", where_clause);
    let list_sql = format!(
        "SELECT * FROM operation_logs {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_clause
    );

    let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql);
    if let Some(lt) = log_type { count_query = count_query.bind(lt); }
    if let Some(did) = device_id { count_query = count_query.bind(did); }
    if let Some(df) = date_from { count_query = count_query.bind(df); }
    let count: (i64,) = count_query.fetch_one(pool).await?;

    let mut list_query = sqlx::query_as::<_, LogEntry>(&list_sql);
    if let Some(lt) = log_type { list_query = list_query.bind(lt); }
    if let Some(did) = device_id { list_query = list_query.bind(did); }
    if let Some(df) = date_from { list_query = list_query.bind(df); }
    list_query = list_query.bind(page_size).bind(offset);

    let rows = list_query.fetch_all(pool).await?;

    Ok((rows, count.0))
}

pub async fn export_logs_csv(
    pool: &SqlitePool,
    log_type: Option<&str>,
    device_id: Option<&str>,
    date_from: Option<&str>,
) -> Result<Vec<u8>> {
    let (items, _) = query_logs(pool, log_type, device_id, date_from, 1, 10000).await?;

    let mut csv = String::from("ID,日志类型,设备ID,操作人,操作,详情,时间\n");
    for item in &items {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            item.id,
            item.log_type.as_deref().unwrap_or(""),
            item.device_id.as_deref().unwrap_or(""),
            item.operator.as_deref().unwrap_or(""),
            item.action.as_deref().unwrap_or(""),
            item.detail.as_deref().unwrap_or("").replace(',', "，"),
            item.created_at,
        ));
    }

    Ok(csv.into_bytes())
}
