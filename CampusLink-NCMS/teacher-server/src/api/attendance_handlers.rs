use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;
use tracing::{info, error};

use super::handlers::ApiResponse;
use super::handlers::AppState;
use crate::domain::attendance;

#[derive(Debug, Deserialize)]
pub struct CheckInRequest {
    pub device_id: String,
    pub student_id: Option<String>,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct RetroactiveRequest {
    pub student_id: String,
    pub device_id: String,
    pub check_in_time: String,
    pub remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatisticsQuery {
    pub date: Option<String>,
}

pub async fn check_in_handler(
    State(state): State<AppState>,
    Json(req): Json<CheckInRequest>,
) -> impl IntoResponse {
    info!("Check-in request: device_id={}", req.device_id);
    match attendance::check_in(&state.pool, &req.device_id, req.student_id.as_deref(), req.timestamp).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => {
            error!("Check-in failed: {}", e);
            Json(ApiResponse::error(500, "签到失败".to_string()))
        }
    }
}

pub async fn statistics_handler(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<StatisticsQuery>,
) -> impl IntoResponse {
    match attendance::get_statistics(&state.pool, query.date.as_deref()).await {
        Ok(stats) => Json(ApiResponse::success(stats)),
        Err(e) => {
            error!("Get attendance statistics failed: {}", e);
            Json(ApiResponse::error(500, "查询签到统计失败".to_string()))
        }
    }
}

pub async fn retroactive_handler(
    State(state): State<AppState>,
    Json(req): Json<RetroactiveRequest>,
) -> impl IntoResponse {
    match attendance::retroactive_check_in(
        &state.pool,
        &req.student_id,
        &req.device_id,
        &req.check_in_time,
        req.remarks.as_deref(),
    ).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => {
            error!("Retroactive check-in failed: {}", e);
            Json(ApiResponse::error(500, "补签失败".to_string()))
        }
    }
}

pub async fn list_attendance_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match attendance::list_attendance(&state.pool, None).await {
        Ok(records) => Json(ApiResponse::success(records)),
        Err(e) => {
            error!("List attendance failed: {}", e);
            Json(ApiResponse::error(500, "查询签到记录失败".to_string()))
        }
    }
}
