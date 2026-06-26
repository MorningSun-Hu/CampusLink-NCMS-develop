use sqlx::SqlitePool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct HardwareSnapshotRow {
    pub id: String,
    pub device_id: String,
    pub cpu_model: Option<String>,
    pub cpu_cores: Option<i32>,
    pub total_memory_bytes: Option<i64>,
    pub disk_info: Option<String>,
    pub mac_addresses: Option<String>,
    pub gpu_info: Option<String>,
    pub os_version: Option<String>,
    pub hostname: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct HardwareSnapshot {
    pub id: String,
    pub device_id: String,
    pub cpu_model: Option<String>,
    pub cpu_cores: Option<i32>,
    pub total_memory_bytes: Option<i64>,
    pub disk_info: Option<String>,
    pub mac_addresses: Option<String>,
    pub gpu_info: Option<String>,
    pub os_version: Option<String>,
    pub hostname: Option<String>,
    pub created_at: String,
}

impl From<HardwareSnapshotRow> for HardwareSnapshot {
    fn from(row: HardwareSnapshotRow) -> Self {
        HardwareSnapshot {
            id: row.id,
            device_id: row.device_id,
            cpu_model: row.cpu_model,
            cpu_cores: row.cpu_cores,
            total_memory_bytes: row.total_memory_bytes,
            disk_info: row.disk_info,
            mac_addresses: row.mac_addresses,
            gpu_info: row.gpu_info,
            os_version: row.os_version,
            hostname: row.hostname,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HardwareSnapshotSubmitRequest {
    pub device_id: String,
    pub cpu_model: Option<String>,
    pub cpu_cores: Option<i32>,
    pub total_memory_bytes: Option<i64>,
    pub disk_info: Option<String>,
    pub mac_addresses: Option<String>,
    pub gpu_info: Option<String>,
    pub os_version: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct HardwareChangeRow {
    pub id: String,
    pub device_id: String,
    pub change_type: String,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub detected_at: String,
}

#[derive(Debug, Serialize)]
pub struct HardwareChange {
    pub id: String,
    pub device_id: String,
    pub change_type: String,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub detected_at: String,
}

impl From<HardwareChangeRow> for HardwareChange {
    fn from(row: HardwareChangeRow) -> Self {
        HardwareChange {
            id: row.id,
            device_id: row.device_id,
            change_type: row.change_type,
            field_name: row.field_name,
            old_value: row.old_value,
            new_value: row.new_value,
            detected_at: row.detected_at,
        }
    }
}

pub async fn submit_snapshot(pool: &SqlitePool, req: &HardwareSnapshotSubmitRequest) -> Result<HardwareSnapshot> {
    let id = Uuid::new_v4().to_string();

    let prev = sqlx::query_as::<_, HardwareSnapshotRow>(
        "SELECT * FROM hardware_snapshots WHERE device_id = ? ORDER BY created_at DESC LIMIT 1"
    )
    .bind(&req.device_id)
    .fetch_optional(pool)
    .await?;

    if let Some(prev_snapshot) = prev {
        detect_changes(pool, &req.device_id, &prev_snapshot, req).await?;
    }

    sqlx::query(
        r#"INSERT INTO hardware_snapshots (id, device_id, cpu_model, cpu_cores, total_memory_bytes, disk_info, mac_addresses, gpu_info, os_version, hostname, created_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))"#
    )
    .bind(&id)
    .bind(&req.device_id)
    .bind(&req.cpu_model)
    .bind(req.cpu_cores)
    .bind(req.total_memory_bytes)
    .bind(&req.disk_info)
    .bind(&req.mac_addresses)
    .bind(&req.gpu_info)
    .bind(&req.os_version)
    .bind(&req.hostname)
    .execute(pool)
    .await?;

    let row = sqlx::query_as::<_, HardwareSnapshotRow>(
        "SELECT * FROM hardware_snapshots WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

async fn detect_changes(pool: &SqlitePool, device_id: &str, prev: &HardwareSnapshotRow, current: &HardwareSnapshotSubmitRequest) -> Result<()> {
    let fields: Vec<(&str, Option<&str>, Option<String>)> = vec![
        ("cpu_model", prev.cpu_model.as_deref(), current.cpu_model.clone()),
        ("disk_info", prev.disk_info.as_deref(), current.disk_info.clone()),
        ("mac_addresses", prev.mac_addresses.as_deref(), current.mac_addresses.clone()),
        ("gpu_info", prev.gpu_info.as_deref(), current.gpu_info.clone()),
        ("os_version", prev.os_version.as_deref(), current.os_version.clone()),
    ];

    for (field_name, old_val, new_val) in &fields {
        let old_str = old_val.unwrap_or("");
        let new_str = new_val.as_deref().unwrap_or("");
        if old_str != new_str {
            let change_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"INSERT INTO hardware_changes (id, device_id, change_type, field_name, old_value, new_value, detected_at)
                   VALUES (?, ?, 'modified', ?, ?, ?, datetime('now'))"#
            )
            .bind(&change_id)
            .bind(device_id)
            .bind(field_name)
            .bind(*old_val)
            .bind(new_val.as_deref())
            .execute(pool)
            .await?;

            let log_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"INSERT INTO operation_logs (id, log_type, device_id, action, detail, created_at)
                   VALUES (?, 'alert', ?, 'hardware_change', ?, datetime('now'))"#
            )
            .bind(&log_id)
            .bind(device_id)
            .bind(format!("{}: '{}' -> '{}'", field_name, old_str, new_str))
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

pub async fn get_snapshot(pool: &SqlitePool, device_id: &str) -> Result<Option<HardwareSnapshot>> {
    let row = sqlx::query_as::<_, HardwareSnapshotRow>(
        "SELECT * FROM hardware_snapshots WHERE device_id = ? ORDER BY created_at DESC LIMIT 1"
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.into()))
}

pub async fn list_changes(pool: &SqlitePool, device_id: Option<&str>, limit: Option<i64>) -> Result<Vec<HardwareChange>> {
    let limit_val = limit.unwrap_or(50);
    let rows = if let Some(did) = device_id {
        sqlx::query_as::<_, HardwareChangeRow>(
            "SELECT * FROM hardware_changes WHERE device_id = ? ORDER BY detected_at DESC LIMIT ?"
        )
        .bind(did)
        .bind(limit_val)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, HardwareChangeRow>(
            "SELECT * FROM hardware_changes ORDER BY detected_at DESC LIMIT ?"
        )
        .bind(limit_val)
        .fetch_all(pool)
        .await?
    };

    Ok(rows.into_iter().map(|r| r.into()).collect())
}
