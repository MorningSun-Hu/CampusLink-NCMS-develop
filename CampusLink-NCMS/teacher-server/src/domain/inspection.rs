use sqlx::{SqlitePool, FromRow};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct InspectionRow {
    pub id: String,
    pub device_id: String,
    pub student_id: Option<String>,
    pub inspection_type: String,
    pub item_name: String,
    pub status: String,
    pub description: Option<String>,
    pub photo_url: Option<String>,
    pub inspector: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct InspectionRecord {
    pub id: String,
    pub device_id: String,
    pub student_id: Option<String>,
    pub inspection_type: String,
    pub item_name: String,
    pub status: String,
    pub description: Option<String>,
    pub photo_url: Option<String>,
    pub inspector: Option<String>,
    pub created_at: String,
}

impl From<InspectionRow> for InspectionRecord {
    fn from(row: InspectionRow) -> Self {
        InspectionRecord {
            id: row.id,
            device_id: row.device_id,
            student_id: row.student_id,
            inspection_type: row.inspection_type,
            item_name: row.item_name,
            status: row.status,
            description: row.description,
            photo_url: row.photo_url,
            inspector: row.inspector,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct InspectionSubmitRequest {
    pub device_id: String,
    pub inspection_type: String,
    pub items: Vec<InspectionItem>,
    pub is_abnormal: bool,
}

#[derive(Debug, Deserialize)]
pub struct InspectionItem {
    pub item_name: String,
    pub status: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InspectionSubmitResponse {
    pub record_ids: Vec<String>,
    pub total_items: usize,
    pub abnormal_count: usize,
}

#[derive(Debug, Serialize)]
pub struct AlertRecord {
    pub id: String,
    pub device_id: String,
    pub student_id: Option<String>,
    pub inspection_type: String,
    pub item_name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub photo_urls: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AlertListResponse {
    pub total: usize,
    pub pending: usize,
    pub resolved: usize,
    pub alerts: Vec<AlertRecord>,
}

pub async fn submit_inspection(pool: &SqlitePool, device_id: &str, inspection_type: &str, items: &[InspectionItem], is_abnormal: bool) -> Result<InspectionSubmitResponse> {
    let mut record_ids = Vec::new();
    let mut abnormal_count = 0;

    for item in items {
        let record_id = Uuid::new_v4().to_string();

        if is_abnormal || item.status == "abnormal" || item.status == "missing" {
            abnormal_count += 1;
        }

        sqlx::query(
            r#"INSERT INTO inspection_records (id, device_id, inspection_type, item_name, status, description, created_at)
               VALUES (?, ?, ?, ?, ?, ?, datetime('now'))"#
        )
        .bind(&record_id)
        .bind(device_id)
        .bind(inspection_type)
        .bind(&item.item_name)
        .bind(&item.status)
        .bind(&item.description)
        .execute(pool)
        .await?;

        record_ids.push(record_id);
    }

    Ok(InspectionSubmitResponse {
        record_ids,
        total_items: items.len(),
        abnormal_count,
    })
}

pub async fn list_inspections(pool: &SqlitePool, inspection_type: Option<&str>, limit: Option<i64>) -> Result<Vec<InspectionRecord>> {
    let limit_val = limit.unwrap_or(50);

    let rows = if let Some(typ) = inspection_type {
        sqlx::query_as::<_, InspectionRow>(
            "SELECT * FROM inspection_records WHERE inspection_type = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(typ)
        .bind(limit_val)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, InspectionRow>(
            "SELECT * FROM inspection_records ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit_val)
        .fetch_all(pool)
        .await?
    };

    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn list_alerts(pool: &SqlitePool, status: Option<&str>, limit: Option<i64>) -> Result<AlertListResponse> {
    let limit_val = limit.unwrap_or(50);

    let show_all = status.map_or(false, |s| s == "all");

    let abnormal_rows = if show_all {
        sqlx::query_as::<_, InspectionRow>(
            r#"SELECT * FROM inspection_records WHERE status IN ('abnormal', 'missing') ORDER BY created_at DESC LIMIT ?"#
        )
        .bind(limit_val)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, InspectionRow>(
            r#"SELECT * FROM inspection_records WHERE status = ? ORDER BY created_at DESC LIMIT ?"#
        )
        .bind(status.unwrap_or("pending"))
        .bind(limit_val)
        .fetch_all(pool)
        .await?
    };

    let pending: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) as count FROM inspection_records WHERE status IN ('abnormal', 'missing')"
    )
    .fetch_one(pool)
    .await?;

    let resolved: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) as count FROM inspection_records WHERE status = 'normal'"
    )
    .fetch_one(pool)
    .await?;

    let alerts: Vec<AlertRecord> = abnormal_rows.into_iter().map(|r| {
        AlertRecord {
            id: r.id,
            device_id: r.device_id,
            student_id: r.student_id,
            inspection_type: r.inspection_type,
            item_name: r.item_name,
            description: r.description,
            status: r.status.clone(),
            created_at: r.created_at,
            photo_urls: Vec::new(),
        }
    }).collect();

    let total = alerts.len();

    Ok(AlertListResponse {
        total,
        pending: pending.0 as usize,
        resolved: resolved.0 as usize,
        alerts,
    })
}

pub async fn resolve_alert(pool: &SqlitePool, alert_id: &str, new_status: &str, remarks: Option<&str>) -> Result<()> {
    sqlx::query(
        r#"UPDATE inspection_records SET status = ?, description = CASE WHEN description IS NULL THEN ? ELSE description || '; ' || ? END WHERE id = ?"#
    )
    .bind(new_status)
    .bind(remarks)
    .bind(remarks)
    .bind(alert_id)
    .execute(pool)
    .await?;

    Ok(())
}
