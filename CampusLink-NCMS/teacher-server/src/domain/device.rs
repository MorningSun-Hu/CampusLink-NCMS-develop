use sqlx::{SqlitePool, FromRow};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceResponse {
    pub id: String,
    pub device_code: String,
    pub device_name: String,
    pub hostname: String,
    pub ip_address: String,
    pub register_status: String,
    pub online_status: String,
    pub current_mode: String,
    pub last_seen_at: Option<String>,
}

impl From<DeviceRow> for DeviceResponse {
    fn from(row: DeviceRow) -> Self {
        DeviceResponse {
            id: row.id,
            device_code: row.device_code,
            device_name: row.device_name,
            hostname: row.hostname,
            ip_address: row.ip_address,
            register_status: row.register_status,
            online_status: row.online_status,
            current_mode: row.current_mode,
            last_seen_at: row.last_seen_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ModeSwitchRequest {
    pub target_mode: String,
    pub effective_at: Option<i64>,
    pub operator_name: String,
}

#[derive(Debug, Serialize)]
pub struct ModeSwitchResponse {
    pub command_id: String,
    pub device_id: String,
    pub target_mode: String,
    pub delivery_status: String,
}

pub async fn register_device(pool: &SqlitePool, device_code: &str, machine_fingerprint: &str, hostname: &str, ip_address: &str, mac_address: &str, agent_version: &str) -> Result<(String, String, String, u32)> {
    let device_id = Uuid::new_v4().to_string();
    let device_name = hostname.to_string();
    
    sqlx::query(
        r#"
        INSERT INTO student_devices 
        (id, device_code, device_name, machine_fingerprint, hostname, ip_address, mac_address, 
         register_status, online_status, current_mode, agent_version, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', 'offline', 'open', ?, datetime('now'), datetime('now'))
        ON CONFLICT(device_code) DO UPDATE SET
            last_seen_at = datetime('now'),
            updated_at = datetime('now')
        "#
    )
    .bind(&device_id)
    .bind(device_code)
    .bind(&device_name)
    .bind(machine_fingerprint)
    .bind(hostname)
    .bind(ip_address)
    .bind(mac_address)
    .bind(agent_version)
    .execute(pool)
    .await?;

    let teacher_fingerprint = crate::infrastructure::device_repository::get_config_value_string(pool, "teacher_fingerprint").await?.unwrap_or_else(|| "pending_init".to_string());
    let heartbeat_interval = crate::infrastructure::device_repository::get_config_value_u32(pool, "heartbeat_interval_seconds").await?.unwrap_or(15);

    Ok((device_id, teacher_fingerprint, "open".to_string(), heartbeat_interval))
}

pub async fn list_devices(pool: &SqlitePool) -> Result<Vec<DeviceResponse>> {
    let rows = sqlx::query_as::<_, DeviceRow>(
        "SELECT * FROM student_devices ORDER BY last_seen_at DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn update_heartbeat(pool: &SqlitePool, device_id: &str) -> Result<()> {
    sqlx::query(
        "UPDATE student_devices SET last_seen_at = datetime('now'), online_status = 'online', updated_at = datetime('now') WHERE id = ?"
    )
    .bind(device_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn switch_device_mode(pool: &SqlitePool, device_id: &str, target_mode: &str, operator_name: &str) -> Result<String> {
    let command_id = Uuid::new_v4().to_string();

    sqlx::query(
        "UPDATE student_devices SET current_mode = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(target_mode)
    .bind(device_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"INSERT INTO operation_logs (id, log_type, operator, target_id, content, extra_payload, created_at)
           VALUES (?, 'mode_switch', ?, ?, ?, '{}', datetime('now'))"#
    )
    .bind(&command_id)
    .bind(operator_name)
    .bind(device_id)
    .bind(format!("Switched to {}", target_mode))
    .execute(pool)
    .await?;

    Ok(command_id)
}
