use sqlx::{SqlitePool, Row};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairOrder {
    pub id: String,
    pub device_id: String,
    pub device_name: String,
    pub reporter: String,
    pub issue_type: String,
    pub description: String,
    pub status: String,
    pub assigned_to: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepairRequest {
    pub device_id: String,
    pub device_name: String,
    pub reporter: String,
    pub issue_type: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRepairRequest {
    pub status: Option<String>,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairListResponse {
    pub orders: Vec<RepairOrder>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

pub async fn create_repair(pool: &SqlitePool, req: CreateRepairRequest) -> Result<RepairOrder> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO repair_orders (id, device_id, device_name, reporter, issue_type, description, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 'pending', datetime('now'), datetime('now'))"
    )
    .bind(&id)
    .bind(&req.device_id)
    .bind(&req.device_name)
    .bind(&req.reporter)
    .bind(&req.issue_type)
    .bind(&req.description)
    .execute(pool)
    .await?;
    get_repair(pool, &id).await
}

pub async fn get_repair(pool: &SqlitePool, id: &str) -> Result<RepairOrder> {
    sqlx::query_as::<_, RepairOrder>("SELECT * FROM repair_orders WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}

pub async fn list_repairs(pool: &SqlitePool, query: RepairListQuery) -> Result<RepairListResponse> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = ((page - 1) * page_size) as i64;

    let (where_clause, bind_value) = if let Some(ref status) = query.status {
        if status.is_empty() || status == "all" {
            (String::new(), None)
        } else {
            ("WHERE status = ?".to_string(), Some(status.clone()))
        }
    } else {
        (String::new(), None)
    };

    let count_sql = format!("SELECT COUNT(*) as cnt FROM repair_orders {}", where_clause);
    let total: i64 = if let Some(ref s) = bind_value {
        sqlx::query(&count_sql).bind(s).fetch_one(pool).await.map_err(|e| anyhow::anyhow!("{}", e))?.get("cnt")
    } else {
        sqlx::query(&count_sql).fetch_one(pool).await.map_err(|e| anyhow::anyhow!("{}", e))?.get("cnt")
    };

    let list_sql = format!("SELECT * FROM repair_orders {} ORDER BY created_at DESC LIMIT ? OFFSET ?", where_clause);
    let orders = if let Some(ref s) = bind_value {
        sqlx::query_as::<_, RepairOrder>(&list_sql).bind(s).bind(page_size as i64).bind(offset).fetch_all(pool).await.map_err(|e| anyhow::anyhow!("{}", e))?
    } else {
        sqlx::query_as::<_, RepairOrder>(&list_sql).bind(page_size as i64).bind(offset).fetch_all(pool).await.map_err(|e| anyhow::anyhow!("{}", e))?
    };

    Ok(RepairListResponse { orders, total, page, page_size })
}

pub async fn update_repair(pool: &SqlitePool, id: &str, req: UpdateRepairRequest) -> Result<RepairOrder> {
    if let Some(status) = &req.status {
        if status == "completed" {
            sqlx::query("UPDATE repair_orders SET status = ?, assigned_to = ?, resolved_at = datetime('now'), updated_at = datetime('now') WHERE id = ?")
                .bind(status)
                .bind(req.assigned_to.as_deref().unwrap_or(""))
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
        } else {
            sqlx::query("UPDATE repair_orders SET status = ?, assigned_to = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(status)
                .bind(req.assigned_to.as_deref().unwrap_or(""))
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
        }
    }
    get_repair(pool, id).await
}

pub async fn delete_repair(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM repair_orders WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}
