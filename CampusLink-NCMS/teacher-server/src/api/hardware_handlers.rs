use axum::{extract::{State, Query}, response::IntoResponse, Json};
use serde::Deserialize;
use tracing::{info, error};

use super::handlers::ApiResponse;
use super::handlers::AppState;
use crate::domain::hardware;

#[derive(Debug, Deserialize)]
pub struct HardwareQuery {
    pub device_id: Option<String>,
    pub limit: Option<i64>,
}

pub async fn submit_snapshot_handler(
    State(state): State<AppState>,
    Json(req): Json<hardware::HardwareSnapshotSubmitRequest>,
) -> impl IntoResponse {
    info!("Hardware snapshot submit for device={}", req.device_id);
    match hardware::submit_snapshot(&state.pool, &req).await {
        Ok(snapshot) => Json(ApiResponse::success(snapshot)),
        Err(e) => {
            error!("Submit hardware snapshot failed: {}", e);
            Json(ApiResponse::error(500, format!("硬件快照提交失败: {}", e)))
        }
    }
}

pub async fn get_snapshot_handler(
    State(state): State<AppState>,
    Query(query): Query<HardwareQuery>,
) -> impl IntoResponse {
    let device_id = match &query.device_id {
        Some(id) => id.clone(),
        None => return Json(ApiResponse::error(400, "缺少 device_id 参数".to_string())),
    };
    match hardware::get_snapshot(&state.pool, &device_id).await {
        Ok(Some(snapshot)) => Json(ApiResponse::success(snapshot)),
        Ok(None) => Json(ApiResponse::error(404, "未找到硬件快照".to_string())),
        Err(e) => {
            error!("Get hardware snapshot failed: {}", e);
            Json(ApiResponse::error(500, "查询硬件快照失败".to_string()))
        }
    }
}

pub async fn list_changes_handler(
    State(state): State<AppState>,
    Query(query): Query<HardwareQuery>,
) -> impl IntoResponse {
    match hardware::list_changes(&state.pool, query.device_id.as_deref(), query.limit).await {
        Ok(changes) => Json(ApiResponse::success(changes)),
        Err(e) => {
            error!("List hardware changes failed: {}", e);
            Json(ApiResponse::error(500, "查询硬件变更记录失败".to_string()))
        }
    }
}
