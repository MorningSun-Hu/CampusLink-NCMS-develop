use sqlx::{SqlitePool, FromRow};
use anyhow::Result;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::device::{DeviceListItem, RegisterDeviceRequest, StudentDevice};

#[derive(Debug, FromRow)]
pub struct DeviceRow {
    pub id: String,
    pub student_id: Option<String>,
    pub device_code: String,
    pub device_name: String,
    pub machine_fingerprint: String,
    pub hostname: String,
    pub ip_address: String,
    pub mac_address: String,
    pub register_status: String,
    pub online_status: String,
    pub last_seen_at: Option<String>,
    pub current_mode: String,
    pub agent_version: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn register_device(
    pool: &SqlitePool,
    req: RegisterDeviceRequest,
) -> Result<RegisterDeviceResponse> {
    let device_id = Uuid::new_v4().to_string();
    let device_name = req.hostname.clone();
    
    sqlx::query(
        r#"
        INSERT INTO student_devices 
        (id, device_code, device_name, machine_fingerprint, hostname, ip_address, mac_address, 
         register_status, online_status, current_mode, agent_version, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', 'offline', 'open', ?, datetime('now'), datetime('now'))
        "#
    )
    .bind(&device_id)
    .bind(&req.device_code)
    .bind(&device_name)
    .bind(&req.machine_fingerprint)
    .bind(&req.hostname)
    .bind(&req.ip_address)
    .bind(&req.mac_address)
    .bind(&req.agent_version)
    .execute(pool)
    .await?;

    let teacher_fingerprint = get_config_value(pool, "teacher_fingerprint").await?;
    let heartbeat_interval = get_config_value::<u32>(pool, "heartbeat_interval_seconds").await?;

    Ok(RegisterDeviceResponse {
        device_id,
        teacher_fingerprint: teacher_fingerprint.unwrap_or_else(|| "pending_init".to_string()),
        initial_mode: "open".to_string(),
        heartbeat_interval_seconds: heartbeat_interval.unwrap_or(15),
    })
}

pub async fn list_devices(pool: &SqlitePool, online_status: Option<&str>) -> Result<Vec<DeviceListItem>> {
    let query = match online_status {
        Some(status) => {
            sqlx::query_as::<_, DeviceRow>(
                r#"
                SELECT id, device_code, device_name, register_status, online_status, 
                       current_mode, last_seen_at
                FROM student_devices
                WHERE online_status = ?
                ORDER BY last_seen_at DESC
                "#
            )
            .bind(status)
        }
        None => {
            sqlx::query_as::<_, DeviceRow>(
                r#"
                SELECT id, device_code, device_name, register_status, online_status, 
                       current_mode, last_seen_at
                FROM student_devices
                ORDER BY last_seen_at DESC
                "#
            )
        }
    };

    let rows = query.fetch_all(pool).await?;
    
    Ok(rows.into_iter().map(|row| DeviceListItem {
        id: row.id,
        device_code: row.device_code,
        device_name: row.device_name,
        register_status: row.register_status,
        online_status: row.online_status,
        current_mode: row.current_mode,
        last_seen_at: row.last_seen_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&Utc)),
    }).collect())
}

pub async fn update_device_heartbeat(pool: &SqlitePool, device_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE student_devices
        SET last_seen_at = datetime('now'),
            online_status = 'online',
            updated_at = datetime('now')
        WHERE id = ?
        "#
    )
    .bind(device_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_config_value<T: for<'a> sqlx::Decode<'a, sqlx::Sqlite>>(
    pool: &SqlitePool, 
    key: &str
) -> Result<Option<T>> {
    let result: Option<String> = sqlx::query_scalar(
        "SELECT config_value FROM system_configs WHERE config_key = ?"
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;

    match result {
        Some(val) => {
            if std::any::type_name::<T>() == std::any::type_name::<u32>() {
                Ok(val.parse::<u32>().ok().map(|v| unsafe { std::mem::transmute_copy(&v) }))
            } else {
                Ok(Some(val as T))
            }
        }
        None => Ok(None),
    }
}
