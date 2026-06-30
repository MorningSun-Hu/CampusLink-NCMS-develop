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
