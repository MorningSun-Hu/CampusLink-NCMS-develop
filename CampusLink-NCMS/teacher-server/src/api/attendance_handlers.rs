use axum::{extract::State, response::IntoResponse, Json};
use axum::http::header;
use serde::Deserialize;
use tracing::{info, error};

use super::handlers::ApiResponse;
use super::handlers::AppState;
use crate::domain::attendance::{self, AttendanceQuery};

#[derive(Debug, Deserialize)]
pub struct InspectionItemRequest {
    pub name: String,
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckInRequest {
    pub device_id: String,
    pub student_id: Option<String>,
    pub timestamp: Option<u64>,
    #[serde(default)]
    pub inspection_items: Option<Vec<InspectionItemRequest>>,
    #[serde(default)]
    pub is_abnormal: Option<bool>,
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

    let inspection = req.inspection_items.map(|items| attendance::CheckInInspection {
        items: items
            .into_iter()
            .map(|item| attendance::InspectionItemInput {
                name: item.name,
                status: item.status,
                detail: item.detail,
            })
            .collect(),
        is_abnormal: req.is_abnormal,
    });

    match attendance::check_in(
        &state.pool,
        &req.device_id,
        req.student_id.as_deref(),
        req.timestamp,
        inspection,
    ).await {
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
    axum::extract::Query(query): axum::extract::Query<AttendanceQuery>,
) -> impl IntoResponse {
    match attendance::list_attendance(&state.pool, &query).await {
        Ok(records) => Json(ApiResponse::success(records)),
        Err(e) => {
            error!("List attendance failed: {}", e);
            Json(ApiResponse::error(500, "查询签到记录失败".to_string()))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardQuery {
    pub class_id: Option<String>,
    pub date: Option<String>,
}

pub async fn attendance_board_handler(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<BoardQuery>,
) -> impl IntoResponse {
    match attendance::attendance_board(&state.pool, query.class_id.as_deref(), query.date.as_deref()).await {
        Ok(board) => Json(ApiResponse::success(board)),
        Err(e) => {
            error!("Get attendance board failed: {}", e);
            Json(ApiResponse::error(500, "查询考勤看板失败".to_string()))
        }
    }
}

pub async fn export_attendance_handler(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<AttendanceQuery>,
) -> impl IntoResponse {
    match generate_attendance_csv(&state.pool, &query).await {
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

async fn generate_attendance_csv(pool: &sqlx::SqlitePool, query: &AttendanceQuery) -> anyhow::Result<Vec<u8>> {
    let query = AttendanceQuery {
        limit: query.limit.or(Some(10000)),
        ..query.clone()
    };
    let records = attendance::list_attendance(pool, &query).await?;
    let mut csv = String::from("\u{feff}学生姓名,学号,班级,座位号,设备,签到时间,状态\n");
    for r in records {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            r.student_name.as_deref().unwrap_or(""),
            r.student_no.as_deref().unwrap_or(""),
            r.class_name.as_deref().unwrap_or(""),
            r.seat_no.as_deref().unwrap_or(""),
            r.device_name.as_deref().unwrap_or(&r.device_id),
            r.check_in_time,
            r.status,
        ));
    }
    Ok(csv.into_bytes())
}
