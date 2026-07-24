use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;

use crate::api::handlers::{ApiResponse, AppState};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockPasswordStatus {
    pub configured: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLockPasswordRequest {
    pub password: String,
}

pub async fn get_lock_password_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query_scalar::<_, String>(
        "SELECT config_value FROM system_configs WHERE config_key = 'lock_password'"
    )
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(val)) => Json(ApiResponse::success(LockPasswordStatus { configured: !val.is_empty() })),
        Ok(None) => Json(ApiResponse::success(LockPasswordStatus { configured: false })),
        Err(_) => Json(ApiResponse::success(LockPasswordStatus { configured: false })),
    }
}

pub async fn update_lock_password_handler(
    State(state): State<AppState>,
    Json(req): Json<UpdateLockPasswordRequest>,
) -> impl IntoResponse {
    if req.password.is_empty() {
        return Json(ApiResponse::error(400, "密码不能为空".to_string()));
    }

    let result = sqlx::query(
        r#"INSERT INTO system_configs (id, config_key, config_value, scope, description, updated_at)
           VALUES (?, 'lock_password', ?, 'global', '锁屏超级密码', datetime('now'))
           ON CONFLICT(config_key) DO UPDATE SET config_value = ?, updated_at = datetime('now')"#
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&req.password)
    .bind(&req.password)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let msg = format!(r#"{{"type":"super_pwd","password":"{}","timestamp":{}}}"#, req.password, chrono::Utc::now().timestamp());
            let _ = state.ws_tx.send(msg);
            info!("Lock password updated and broadcast to all devices");
            Json(ApiResponse::success("ok".to_string()))
        }
        Err(e) => {
            Json(ApiResponse::error(500, format!("更新失败: {}", e)))
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleConfig {
    pub enabled: bool,
    pub time: Option<String>,
    pub target_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateScheduleRequest {
    pub enabled: bool,
    pub time: Option<String>,
    pub target_mode: Option<String>,
}

pub async fn get_schedule_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let enabled = sqlx::query_scalar::<_, String>(
        "SELECT config_value FROM system_configs WHERE config_key = 'schedule_mode'"
    )
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();

    let time = sqlx::query_scalar::<_, String>(
        "SELECT config_value FROM system_configs WHERE config_key = 'schedule_time'"
    )
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();

    let target_mode = sqlx::query_scalar::<_, String>(
        "SELECT config_value FROM system_configs WHERE config_key = 'target_mode'"
    )
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();

    Json(ApiResponse::success(ScheduleConfig {
        enabled: enabled.as_deref() == Some("enabled"),
        time,
        target_mode,
    }))
}

pub async fn update_schedule_handler(
    State(state): State<AppState>,
    Json(req): Json<UpdateScheduleRequest>,
) -> impl IntoResponse {
    let enabled_val = if req.enabled { "enabled" } else { "disabled" };
    let _ = sqlx::query(
        r#"INSERT INTO system_configs (id, config_key, config_value, scope, description, updated_at)
           VALUES (?, 'schedule_mode', ?, 'schedule', 'Mode switch scheduler', datetime('now'))
           ON CONFLICT(config_key) DO UPDATE SET config_value = ?, updated_at = datetime('now')"#
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(enabled_val)
    .bind(enabled_val)
    .execute(&state.pool)
    .await;

    if let Some(ref time) = req.time {
        let _ = sqlx::query(
            r#"INSERT INTO system_configs (id, config_key, config_value, scope, description, updated_at)
               VALUES (?, 'schedule_time', ?, 'schedule', 'Scheduled mode switch time (HH:MM)', datetime('now'))
               ON CONFLICT(config_key) DO UPDATE SET config_value = ?, updated_at = datetime('now')"#
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(time)
        .bind(time)
        .execute(&state.pool)
        .await;
    }

    if let Some(ref mode) = req.target_mode {
        let _ = sqlx::query(
            r#"INSERT INTO system_configs (id, config_key, config_value, scope, description, updated_at)
               VALUES (?, 'target_mode', ?, 'schedule', 'Target mode for scheduled switch', datetime('now'))
               ON CONFLICT(config_key) DO UPDATE SET config_value = ?, updated_at = datetime('now')"#
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(mode)
        .bind(mode)
        .execute(&state.pool)
        .await;
    }

    info!("Schedule config updated: enabled={}, time={:?}, mode={:?}", req.enabled, req.time, req.target_mode);
    Json(ApiResponse::success("ok"))
}
