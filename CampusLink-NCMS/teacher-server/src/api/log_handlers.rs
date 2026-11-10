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

#[derive(Debug, Deserialize)]
pub struct AgentLogFilePayload {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct AgentLogUploadRequest {
    pub device_id: String,
    pub files: Vec<AgentLogFilePayload>,
}

#[derive(Debug, Deserialize)]
pub struct AgentLogDownloadQuery {
    pub device_id: String,
    pub file: String,
}

pub async fn collect_student_logs_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let ts = chrono::Utc::now().timestamp();
    let msg = format!(r#"{{"type":"collect_logs","device_id":"","timestamp":{}}}"#, ts);
    match state.ws_tx.send(msg) {
        Ok(_) => {
            info!("Broadcast collect_logs to online students");
            Json(ApiResponse::success("已向在线学生机下发收集指令"))
        }
        Err(_) => Json(ApiResponse::error(500, "当前没有在线学生机可接收指令".to_string())),
    }
}

pub async fn upload_agent_logs_handler(
    State(state): State<AppState>,
    Json(req): Json<AgentLogUploadRequest>,
) -> impl IntoResponse {
    if req.device_id.is_empty() {
        return Json(ApiResponse::<usize>::error(400, "device_id 不能为空".to_string()));
    }
    if !crate::domain::device::device_exists(&state.pool, &req.device_id).await.unwrap_or(false) {
        return Json(ApiResponse::<usize>::error(401, "设备未注册，请先注册".to_string()));
    }
    if req.files.len() > 8 {
        return Json(ApiResponse::<usize>::error(400, "日志文件数量超出限制".to_string()));
    }
    let root = logs::agent_logs_root();
    let mut saved = 0usize;
    for file in &req.files {
        if file.content.len() > 2_000_000 {
            continue;
        }
        match logs::save_agent_log_file(&root, &req.device_id, &file.name, &file.content) {
            Ok(_) => saved += 1,
            Err(e) => error!("Save agent log {} failed: {}", file.name, e),
        }
    }
    info!("Saved {} agent log file(s) from {}", saved, req.device_id);
    Json(ApiResponse::success(saved))
}

pub async fn list_agent_logs_handler() -> impl IntoResponse {
    match logs::list_agent_log_files(&logs::agent_logs_root()) {
        Ok(items) => Json(ApiResponse::success(items)),
        Err(e) => {
            error!("List agent logs failed: {}", e);
            Json(ApiResponse::error(500, "列出学生机日志失败".to_string()))
        }
    }
}

pub async fn download_agent_log_handler(
    Query(query): Query<AgentLogDownloadQuery>,
) -> impl IntoResponse {
    match logs::read_agent_log_file(&logs::agent_logs_root(), &query.device_id, &query.file) {
        Ok(data) => {
            let headers = [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"student-agent.log\""),
            ];
            (axum::http::StatusCode::OK, headers, axum::body::Body::from(data)).into_response()
        }
        Err(e) => {
            error!("Download agent log failed: {}", e);
            Json(ApiResponse::<()>::error(404, "日志文件不存在".to_string())).into_response()
        }
    }
}
