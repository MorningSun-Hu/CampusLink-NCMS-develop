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
#[serde(rename_all = "camelCase")]
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
    // Whitelist check
    if !check_whitelist(pool, device_code).await? {
        return Err(anyhow::anyhow!("设备 {} 未在白名单中", device_code));
    }

    let device_id = Uuid::new_v4().to_string();
    let device_name = hostname.to_string();
    
    sqlx::query(
        r#"
        INSERT INTO student_devices 
        (id, device_code, device_name, machine_fingerprint, hostname, ip_address, mac_address, 
         register_status, online_status, current_mode, agent_version, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'verified', 'offline', 'open', ?, datetime('now'), datetime('now'))
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

pub async fn device_exists(pool: &SqlitePool, device_id: &str) -> Result<bool> {
    let id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM student_devices WHERE id = ?"
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    Ok(id.is_some())
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

pub async fn update_device_mode(pool: &SqlitePool, device_id: &str, mode: &str) -> Result<()> {
    sqlx::query(
        "UPDATE student_devices SET current_mode = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(mode)
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

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub id: String,
    pub device_code: String,
    pub device_name: String,
    pub mac_address: String,
    pub status: String,
    pub created_at: String,
}

pub async fn check_whitelist(pool: &SqlitePool, device_code: &str) -> Result<bool> {
    let policy: Option<String> = sqlx::query_scalar(
        "SELECT config_value FROM system_configs WHERE config_key = 'device_register_policy'"
    )
    .fetch_optional(pool)
    .await?
    .flatten();

    if policy.as_deref() != Some("whitelist_required") {
        return Ok(true);
    }

    let exists: Option<String> = sqlx::query_scalar(
        "SELECT id FROM device_whitelist WHERE device_code = ? AND status = 'approved'"
    )
    .bind(device_code)
    .fetch_optional(pool)
    .await?;

    Ok(exists.is_some())
}

pub async fn list_whitelist(pool: &SqlitePool) -> Result<Vec<WhitelistEntry>> {
    let entries = sqlx::query_as::<_, WhitelistEntry>(
        "SELECT * FROM device_whitelist ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(entries)
}

pub async fn import_whitelist(pool: &SqlitePool, device_code: &str, device_name: &str, mac_address: &str) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT OR REPLACE INTO device_whitelist (id, device_code, device_name, mac_address, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'pending', datetime('now'), datetime('now'))"
    )
    .bind(&id)
    .bind(device_code)
    .bind(device_name)
    .bind(mac_address)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn approve_whitelist(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("UPDATE device_whitelist SET status = 'approved', updated_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_whitelist(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM device_whitelist WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_support::setup_pool;

    #[tokio::test]
    async fn device_exists_returns_true_for_registered_device() {
        let pool = setup_pool().await;
        let (device_id, _, _, _) = register_device(
            &pool,
            "DEV-REG-001",
            "fp-reg-001",
            "host-reg-001",
            "192.168.1.10",
            "aa:bb:cc",
            "test-agent",
        )
        .await
        .unwrap();

        assert!(device_exists(&pool, &device_id).await.unwrap());
        assert!(!device_exists(&pool, "nonexistent-id").await.unwrap());
    }

    #[tokio::test]
    async fn register_device_returns_meta() {
        let pool = setup_pool().await;
        let (device_id, fingerprint, initial_mode, hb) = register_device(
            &pool,
            "DEV-REG-002",
            "fp-reg-002",
            "host-reg-002",
            "192.168.1.11",
            "dd:ee:ff",
            "test-agent",
        )
        .await
        .unwrap();

        assert!(!device_id.is_empty());
        assert_eq!(fingerprint, "pending_init");
        assert_eq!(initial_mode, "open");
        assert!(hb > 0);
    }

    #[tokio::test]
    async fn whitelist_default_policy_allows_all() {
        let pool = setup_pool().await;
        assert!(check_whitelist(&pool, "ANY-DEVICE").await.unwrap());
    }

    #[tokio::test]
    async fn whitelist_required_blocks_unapproved() {
        let pool = setup_pool().await;
        sqlx::query(
            r#"UPDATE system_configs SET config_value = 'whitelist_required'
               WHERE config_key = 'device_register_policy'"#
        )
        .execute(&pool)
        .await
        .unwrap();

        assert!(!check_whitelist(&pool, "NOT-APPROVED").await.unwrap());

        import_whitelist(&pool, "NOT-APPROVED", "name", "mac1").await.unwrap();
        assert!(!check_whitelist(&pool, "NOT-APPROVED").await.unwrap());

        let entries = list_whitelist(&pool).await.unwrap();
        assert_eq!(entries.len(), 1);
        let id = entries[0].id.clone();
        approve_whitelist(&pool, &id).await.unwrap();

        assert!(check_whitelist(&pool, "NOT-APPROVED").await.unwrap());
    }
}
