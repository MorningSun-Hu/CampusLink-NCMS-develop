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
    pub class_id: Option<String>,
    pub seat_no: Option<String>,
    pub pending_checkin: i64,
    pub mode_before_lock: String,
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
    pub class_id: Option<String>,
    pub class_name: Option<String>,
    pub seat_no: Option<String>,
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
            last_seen_at: crate::domain::time_util::to_rfc3339_opt(row.last_seen_at.as_deref()),
            class_id: row.class_id,
            class_name: None,
            seat_no: row.seat_no,
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

    let candidate_id = Uuid::new_v4().to_string();
    let device_name = hostname.to_string();

    sqlx::query(
        r#"
        INSERT INTO student_devices 
        (id, device_code, device_name, machine_fingerprint, hostname, ip_address, mac_address, 
         register_status, online_status, current_mode, agent_version, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'verified', 'offline', 'open', ?, datetime('now'), datetime('now'))
        ON CONFLICT(device_code) DO UPDATE SET
            device_name = excluded.device_name,
            machine_fingerprint = excluded.machine_fingerprint,
            hostname = excluded.hostname,
            ip_address = excluded.ip_address,
            mac_address = excluded.mac_address,
            agent_version = excluded.agent_version,
            last_seen_at = datetime('now'),
            updated_at = datetime('now')
        "#
    )
    .bind(&candidate_id)
    .bind(device_code)
    .bind(&device_name)
    .bind(machine_fingerprint)
    .bind(hostname)
    .bind(ip_address)
    .bind(mac_address)
    .bind(agent_version)
    .execute(pool)
    .await?;

    // On conflict the pre-generated id is not persisted, so always read back the
    // authoritative id for this device_code. Returning the candidate id here would
    // make re-registration break every later operation keyed on the device id.
    let device_id: String = sqlx::query_scalar("SELECT id FROM student_devices WHERE device_code = ?")
        .bind(device_code)
        .fetch_one(pool)
        .await?;

    let teacher_fingerprint = crate::infrastructure::device_repository::get_config_value_string(pool, "teacher_fingerprint").await?.unwrap_or_else(|| "pending_init".to_string());
    let heartbeat_interval = crate::infrastructure::device_repository::get_config_value_u32(pool, "heartbeat_interval_seconds").await?.unwrap_or(15);

    let current_mode: String = sqlx::query_scalar("SELECT current_mode FROM student_devices WHERE id = ?")
        .bind(&device_id)
        .fetch_one(pool)
        .await?;

    Ok((device_id, teacher_fingerprint, current_mode, heartbeat_interval))
}

pub async fn list_devices(pool: &SqlitePool) -> Result<Vec<DeviceResponse>> {
    let rows = sqlx::query_as::<_, DeviceRow>(
        "SELECT * FROM student_devices ORDER BY last_seen_at DESC"
    )
    .fetch_all(pool)
    .await?;

    let mut devices = Vec::with_capacity(rows.len());
    for row in rows {
        let mut response: DeviceResponse = row.into();
        if let Some(cid) = response.class_id.as_deref() {
            response.class_name = sqlx::query_scalar::<_, String>("SELECT name FROM classes WHERE id = ?")
                .bind(cid)
                .fetch_optional(pool)
                .await?;
        }
        devices.push(response);
    }
    Ok(devices)
}

pub async fn list_devices_by_class(pool: &SqlitePool, class_id: &str) -> Result<Vec<DeviceResponse>> {
    let rows = sqlx::query_as::<_, DeviceRow>(
        "SELECT * FROM student_devices WHERE class_id = ? ORDER BY seat_no ASC"
    )
    .bind(class_id)
    .fetch_all(pool)
    .await?;

    let mut devices = Vec::with_capacity(rows.len());
    for row in rows {
        let mut response: DeviceResponse = row.into();
        if let Some(cid) = response.class_id.as_deref() {
            response.class_name = sqlx::query_scalar::<_, String>("SELECT name FROM classes WHERE id = ?")
                .bind(cid)
                .fetch_optional(pool)
                .await?;
        }
        devices.push(response);
    }
    Ok(devices)
}

pub async fn set_device_class(pool: &SqlitePool, device_id: &str, class_id: Option<&str>) -> Result<()> {
    sqlx::query("UPDATE student_devices SET class_id = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(class_id)
        .bind(device_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_device_seat(pool: &SqlitePool, device_id: &str, seat_no: Option<&str>) -> Result<()> {
    sqlx::query("UPDATE student_devices SET seat_no = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(seat_no)
        .bind(device_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_pending_checkin(pool: &SqlitePool, device_id: &str, pending: bool) -> Result<()> {
    sqlx::query("UPDATE student_devices SET pending_checkin = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(if pending { 1 } else { 0 })
        .bind(device_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn consume_pending_checkin(pool: &SqlitePool, device_id: &str) -> Result<bool> {
    let pending: Option<i64> = sqlx::query_scalar::<_, i64>("SELECT pending_checkin FROM student_devices WHERE id = ?")
        .bind(device_id)
        .fetch_optional(pool)
        .await?;
    if pending == Some(1) {
        set_pending_checkin(pool, device_id, false).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn get_device_seat(pool: &SqlitePool, device_id: &str) -> Result<Option<String>> {
    let seat: Option<String> = sqlx::query_scalar("SELECT seat_no FROM student_devices WHERE id = ?")
        .bind(device_id)
        .fetch_optional(pool)
        .await?;
    Ok(seat)
}

pub async fn list_online_devices_by_class(pool: &SqlitePool, class_id: &str) -> Result<Vec<String>> {
    let ids: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM student_devices WHERE class_id = ? AND online_status = 'online'"
    )
    .bind(class_id)
    .fetch_all(pool)
    .await?;
    Ok(ids)
}

pub async fn is_online(pool: &SqlitePool, device_id: &str) -> Result<bool> {
    let status: Option<String> = sqlx::query_scalar("SELECT online_status FROM student_devices WHERE id = ?")
        .bind(device_id)
        .fetch_optional(pool)
        .await?;
    Ok(status.as_deref() == Some("online"))
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

pub async fn remember_mode_and_lock(pool: &SqlitePool, device_id: &str) -> Result<()> {
    sqlx::query(
        r#"UPDATE student_devices
           SET mode_before_lock = CASE
                 WHEN current_mode != 'locked' THEN current_mode
                 ELSE mode_before_lock
               END,
               current_mode = 'locked',
               updated_at = datetime('now')
           WHERE id = ?"#
    )
    .bind(device_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub fn sanitize_restore_mode(mode: &str) -> String {
    match mode {
        "teaching" | "open" | "exam" => mode.to_string(),
        _ => "open".to_string(),
    }
}

pub async fn unlock_restore_mode(pool: &SqlitePool, device_id: &str) -> Result<String> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT current_mode, mode_before_lock FROM student_devices WHERE id = ?"
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;
    let (current, before) = row.unwrap_or_else(|| ("open".to_string(), "open".to_string()));
    if current != "locked" {
        return Ok(sanitize_restore_mode(&current));
    }
    let restore = sanitize_restore_mode(&before);
    update_device_mode(pool, device_id, &restore).await?;
    Ok(restore)
}

pub struct ModeSwitchOutcome {
    pub command_id: String,
    pub effective_mode: String,
    pub admission_blocked: bool,
}

pub async fn switch_device_mode(
    pool: &SqlitePool,
    device_id: &str,
    target_mode: &str,
    operator_name: &str,
    class_id: Option<&str>,
) -> Result<ModeSwitchOutcome> {
    let command_id = Uuid::new_v4().to_string();

    let effective_mode = if target_mode == "teaching" {
        let class_id = match class_id {
            Some(id) if !id.trim().is_empty() => id,
            _ => anyhow::bail!("切换到授课模式前请先选择班级"),
        };
        let exists: Option<String> = sqlx::query_scalar("SELECT id FROM classes WHERE id = ?")
            .bind(class_id)
            .fetch_optional(pool)
            .await?;
        if exists.is_none() {
            anyhow::bail!("班级不存在");
        }
        crate::domain::class::teaching_target_for_device(pool, class_id, device_id).await?
    } else {
        target_mode.to_string()
    };

    let _ = crate::domain::usage::close_open_session(pool, device_id).await;

    if effective_mode == "locked" {
        sqlx::query(
            r#"UPDATE student_devices
               SET mode_before_lock = CASE
                     WHEN current_mode != 'locked' THEN current_mode
                     ELSE mode_before_lock
                   END
               WHERE id = ?"#
        )
        .bind(device_id)
        .execute(pool)
        .await?;
    }

    sqlx::query(
        "UPDATE student_devices SET current_mode = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(&effective_mode)
    .bind(device_id)
    .execute(pool)
    .await?;

    if effective_mode == "exam" {
        let _ = crate::domain::usage::ensure_exam_session(pool, device_id).await;
    }

    let content = if effective_mode == target_mode {
        format!("Switched to {}", effective_mode)
    } else {
        format!("Switched to {} (admission denied, forced {})", target_mode, effective_mode)
    };

    sqlx::query(
        r#"INSERT INTO operation_logs (id, log_type, operator, target_id, content, extra_payload, created_at)
           VALUES (?, 'mode_switch', ?, ?, ?, '{}', datetime('now'))"#
    )
    .bind(&command_id)
    .bind(operator_name)
    .bind(device_id)
    .bind(content)
    .execute(pool)
    .await?;

    Ok(ModeSwitchOutcome {
        command_id,
        admission_blocked: effective_mode != target_mode,
        effective_mode,
    })
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
    async fn re_register_same_device_code_keeps_stable_id() {
        let pool = setup_pool().await;
        let (first_id, _, _, _) = register_device(
            &pool,
            "DEV-REG-DUP",
            "fp-dup-1",
            "host-dup-1",
            "192.168.1.20",
            "aa:aa:aa",
            "test-agent",
        )
        .await
        .unwrap();
        let (second_id, _, _, _) = register_device(
            &pool,
            "DEV-REG-DUP",
            "fp-dup-2",
            "host-dup-2",
            "192.168.1.21",
            "bb:bb:bb",
            "test-agent-2",
        )
        .await
        .unwrap();

        assert_eq!(first_id, second_id);
        assert!(device_exists(&pool, &second_id).await.unwrap());
        let ip: String = sqlx::query_scalar("SELECT ip_address FROM student_devices WHERE device_code = ?")
            .bind("DEV-REG-DUP")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(ip, "192.168.1.21");
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
    async fn re_register_returns_persisted_current_mode() {
        let pool = setup_pool().await;
        let (device_id, _, _, _) = register_device(
            &pool,
            "DEV-REG-MODE",
            "fp-reg-mode",
            "host-reg-mode",
            "192.168.1.31",
            "aa:bb:cc",
            "test-agent",
        )
        .await
        .unwrap();

        sqlx::query("UPDATE student_devices SET current_mode = 'teaching' WHERE id = ?")
            .bind(&device_id)
            .execute(&pool)
            .await
            .unwrap();

        let (_, _, initial_mode, _) = register_device(
            &pool,
            "DEV-REG-MODE",
            "fp-reg-mode-2",
            "host-reg-mode",
            "192.168.1.32",
            "aa:bb:cc",
            "test-agent",
        )
        .await
        .unwrap();

        assert_eq!(initial_mode, "teaching");
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

    #[tokio::test]
    async fn unlock_keeps_current_mode_when_not_locked() {
        let pool = setup_pool().await;
        let (device_id, _, _, _) = register_device(
            &pool,
            "DEV-UNLOCK-KEEP",
            "fp-unlock-keep",
            "host-unlock-keep",
            "192.168.1.40",
            "aa:bb:cc",
            "test-agent",
        )
        .await
        .unwrap();
        update_device_mode(&pool, &device_id, "teaching").await.unwrap();

        let restore = unlock_restore_mode(&pool, &device_id).await.unwrap();
        assert_eq!(restore, "teaching");
        let mode: String = sqlx::query_scalar("SELECT current_mode FROM student_devices WHERE id = ?")
            .bind(&device_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(mode, "teaching");
    }

    #[tokio::test]
    async fn unlock_restores_mode_before_lock() {
        let pool = setup_pool().await;
        let (device_id, _, _, _) = register_device(
            &pool,
            "DEV-UNLOCK-LOCK",
            "fp-unlock-lock",
            "host-unlock-lock",
            "192.168.1.41",
            "aa:bb:cc",
            "test-agent",
        )
        .await
        .unwrap();
        update_device_mode(&pool, &device_id, "teaching").await.unwrap();
        remember_mode_and_lock(&pool, &device_id).await.unwrap();

        let restore = unlock_restore_mode(&pool, &device_id).await.unwrap();
        assert_eq!(restore, "teaching");
    }
}
