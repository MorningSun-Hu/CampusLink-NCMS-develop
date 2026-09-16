use axum::{extract::State, response::IntoResponse, Json};
use axum::http::header;
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

    if !crate::domain::device::device_exists(&state.pool, &req.device_id).await.unwrap_or(false) {
        return Json(ApiResponse::error(401, "设备未注册，请先注册".to_string()));
    }

    match attendance::check_in(&state.pool, &req.device_id, req.student_id.as_deref(), req.timestamp).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => {
            error!("Check-in failed: {}", e);
            Json(ApiResponse::error(400, e.to_string()))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ContextQuery {
    pub device_id: String,
}

pub async fn attendance_context_handler(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ContextQuery>,
) -> impl IntoResponse {
    match attendance::attendance_context(&state.pool, &query.device_id).await {
        Ok(ctx) => Json(ApiResponse::success(ctx)),
        Err(e) => {
            error!("Get attendance context failed: {}", e);
            Json(ApiResponse::error(404, e.to_string()))
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

pub async fn export_attendance_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match generate_attendance_csv(&state.pool).await {
        Ok(data) => {
            let headers = [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"attendance.csv\""),
            ];
            (headers, data).into_response()
        }
        Err(e) => {
            error!("Export attendance failed: {}", e);
            Json(ApiResponse::<()>::error(500, format!("导出失败: {}", e))).into_response()
        }
    }
}

async fn generate_attendance_csv(pool: &sqlx::SqlitePool) -> anyhow::Result<Vec<u8>> {
    let records = attendance::list_attendance(pool, Some(10000)).await?;
    let mut csv = String::from("ID,学生ID,设备ID,签到时间,签退时间,状态,备注,创建时间\n");
    for r in records {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            r.id,
            r.student_id.as_deref().unwrap_or(""),
            r.device_id,
            r.check_in_time,
            r.check_out_time.as_deref().unwrap_or(""),
            r.status,
            r.remarks.as_deref().unwrap_or(""),
            "", // created_at not in record
        ));
    }
    Ok(csv.into_bytes())
}
