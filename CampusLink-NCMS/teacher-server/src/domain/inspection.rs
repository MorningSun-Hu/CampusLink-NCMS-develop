use sqlx::{SqlitePool, FromRow};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct InspectionRow {
    pub id: String,
    pub device_id: String,
    pub device_name: Option<String>,
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
    pub device_name: Option<String>,
    pub student_id: Option<String>,
    pub inspection_type: String,
    pub item_name: String,
    pub status: String,
    pub description: Option<String>,
    pub photo_url: Option<String>,
    pub inspector: Option<String>,
    pub created_at: String,
    pub is_abnormal: bool,
}

impl From<InspectionRow> for InspectionRecord {
    fn from(row: InspectionRow) -> Self {
        let is_abnormal = row.status == "abnormal" || row.status == "missing";
        InspectionRecord {
            id: row.id,
            device_id: row.device_id,
            device_name: row.device_name,
            student_id: row.student_id,
            inspection_type: row.inspection_type,
            item_name: row.item_name,
            status: row.status,
            description: row.description,
            photo_url: row.photo_url,
            inspector: row.inspector,
            created_at: row.created_at,
            is_abnormal,
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
    pub device_name: Option<String>,
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
    let mut abnormal_count = 0;
    for item in items {
        if item.status == "abnormal" || item.status == "missing" {
            abnormal_count += 1;
        }
    }
    if is_abnormal && abnormal_count == 0 {
        abnormal_count = 1;
    }

    let item_name = if items.is_empty() {
        inspection_type.to_string()
    } else {
        items.iter().map(|i| i.item_name.as_str()).collect::<Vec<_>>().join("、")
    };
    let description = items
        .iter()
        .map(|item| {
            let label = match item.status.as_str() {
                "normal" => "正常",
                "missing" => "缺失",
                "abnormal" => "异常",
                other => other,
            };
            match &item.description {
                Some(detail) if !detail.trim().is_empty() => {
                    format!("{}:{}({})", item.item_name, label, detail.trim())
                }
                _ => format!("{}:{}", item.item_name, label),
            }
        })
        .collect::<Vec<_>>()
        .join("；");
    let status = if abnormal_count > 0 { "abnormal" } else { "normal" };
    let record_id = Uuid::new_v4().to_string();
    let description = if description.is_empty() { None } else { Some(description) };

    sqlx::query(
        r#"INSERT INTO inspection_records (id, device_id, inspection_type, item_name, status, description, created_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'))"#
    )
    .bind(&record_id)
    .bind(device_id)
    .bind(inspection_type)
    .bind(&item_name)
    .bind(status)
    .bind(&description)
    .execute(pool)
    .await?;

    Ok(InspectionSubmitResponse {
        record_ids: vec![record_id],
        total_items: items.len(),
        abnormal_count,
    })
}

pub async fn list_inspections(pool: &SqlitePool, inspection_type: Option<&str>, limit: Option<i64>) -> Result<Vec<InspectionRecord>> {
    let limit_val = limit.unwrap_or(50);

    const LIST_BY_TYPE: &str = r#"SELECT i.id, i.device_id, d.device_name, i.student_id, i.inspection_type, i.item_name, i.status, i.description, i.photo_url, i.inspector, i.created_at FROM inspection_records i LEFT JOIN student_devices d ON d.id = i.device_id WHERE i.inspection_type = ? ORDER BY i.created_at DESC LIMIT ?"#;
    const LIST_ALL: &str = r#"SELECT i.id, i.device_id, d.device_name, i.student_id, i.inspection_type, i.item_name, i.status, i.description, i.photo_url, i.inspector, i.created_at FROM inspection_records i LEFT JOIN student_devices d ON d.id = i.device_id ORDER BY i.created_at DESC LIMIT ?"#;

    let rows = if let Some(typ) = inspection_type {
        sqlx::query_as::<_, InspectionRow>(
            LIST_BY_TYPE
        )
        .bind(typ)
        .bind(limit_val)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, InspectionRow>(
            LIST_ALL
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
            r#"SELECT i.id, i.device_id, d.device_name, i.student_id, i.inspection_type, i.item_name, i.status, i.description, i.photo_url, i.inspector, i.created_at FROM inspection_records i LEFT JOIN student_devices d ON d.id = i.device_id WHERE i.status IN ('abnormal', 'missing') ORDER BY i.created_at DESC LIMIT ?"#
        )
        .bind(limit_val)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, InspectionRow>(
            r#"SELECT i.id, i.device_id, d.device_name, i.student_id, i.inspection_type, i.item_name, i.status, i.description, i.photo_url, i.inspector, i.created_at FROM inspection_records i LEFT JOIN student_devices d ON d.id = i.device_id WHERE i.status = ? ORDER BY i.created_at DESC LIMIT ?"#
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
            device_name: r.device_name,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_support::{setup_pool, register_test_device};

    fn item(name: &str, status: &str) -> InspectionItem {
        InspectionItem {
            item_name: name.to_string(),
            status: status.to_string(),
            description: None,
        }
    }

    #[tokio::test]
    async fn submit_inspection_creates_records_and_counts() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-IN-001").await;

        let items = vec![
            item("keyboard", "normal"),
            item("mouse", "abnormal"),
            item("monitor", "missing"),
        ];
        let resp = submit_inspection(&pool, &device_id, "hygiene", &items, false).await.unwrap();

        assert_eq!(resp.total_items, 3);
        assert_eq!(resp.abnormal_count, 2);
        assert_eq!(resp.record_ids.len(), 1);

        let records = list_inspections(&pool, Some("hygiene"), None).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].device_id, device_id);
        assert_eq!(records[0].device_name.as_deref(), Some("host-DEV-IN-001"));
        assert!(records[0].description.as_deref().unwrap_or("").contains("keyboard"));
    }

    #[tokio::test]
    async fn is_abnormal_flag_marks_all_abnormal() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-IN-002").await;

        let items = vec![item("desk", "normal")];
        let resp = submit_inspection(&pool, &device_id, "check", &items, true).await.unwrap();

        assert_eq!(resp.abnormal_count, 1);
        let records = list_inspections(&pool, Some("check"), None).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].status, "abnormal");
        assert!(records[0].is_abnormal);
    }

    #[tokio::test]
    async fn list_alerts_filters_abnormal_only() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-IN-003").await;

        let items = vec![item("keyboard", "abnormal")];
        submit_inspection(&pool, &device_id, "hygiene", &items, false).await.unwrap();

        let alerts = list_alerts(&pool, Some("all"), None).await.unwrap();
        assert_eq!(alerts.alerts.len(), 1);
        assert_eq!(alerts.alerts[0].item_name, "keyboard");
    }

    #[tokio::test]
    async fn resolve_alert_updates_status() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-IN-004").await;

        let items = vec![item("mouse", "abnormal")];
        let resp = submit_inspection(&pool, &device_id, "hygiene", &items, false).await.unwrap();
        let alert_id = &resp.record_ids[0];

        resolve_alert(&pool, alert_id, "resolved", Some("fixed")).await.unwrap();

        let records = list_inspections(&pool, None, None).await.unwrap();
        assert_eq!(records[0].status, "resolved");
        assert!(records[0].description.as_deref().unwrap_or("").contains("fixed"));
    }
}
