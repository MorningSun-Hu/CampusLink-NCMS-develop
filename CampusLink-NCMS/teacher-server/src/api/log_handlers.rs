use axum::{extract::{State, Query}, response::IntoResponse, Json};
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
