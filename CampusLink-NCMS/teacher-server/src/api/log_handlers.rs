use axum::{extract::{State, Query}, response::IntoResponse, Json};
use axum::http::header;
use serde::Deserialize;
use tracing::{info, error};

use super::handlers::ApiResponse;
use super::handlers::AppState;
use crate::domain::logs;

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub log_type: Option<String>,
    pub device_id: Option<String>,
    pub date_from: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct LogPageResponse {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub items: Vec<logs::LogEntry>,
}

pub async fn export_logs_handler(
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> impl IntoResponse {
    info!("Export logs: type={:?}, device={:?}", query.log_type, query.device_id);
    match logs::export_logs_csv(
        &state.pool,
        query.log_type.as_deref(),
        query.device_id.as_deref(),
        query.date_from.as_deref(),
    ).await {
        Ok(data) => {
            let headers = [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"operation_logs.csv\""),
            ];
            (axum::http::StatusCode::OK, headers, axum::body::Body::from(data)).into_response()
        }
        Err(e) => {
            error!("Export logs failed: {}", e);
            Json(ApiResponse::<()>::error(500, "导出日志失败".to_string())).into_response()
        }
    }
}

pub async fn query_logs_handler(
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    info!("Query logs: type={:?}, device={:?}, page={}", query.log_type, query.device_id, page);

    match logs::query_logs(
        &state.pool,
        query.log_type.as_deref(),
        query.device_id.as_deref(),
        query.date_from.as_deref(),
        page,
        page_size,
    ).await {
        Ok((items, total)) => Json(ApiResponse::success(LogPageResponse {
            total,
            page,
            page_size,
            items,
        })),
        Err(e) => {
            error!("Query logs failed: {}", e);
            Json(ApiResponse::error(500, "查询日志失败".to_string()))
        }
    }
}
