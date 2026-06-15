use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;
use tracing::{info, error};

use super::handlers::ApiResponse;
use super::handlers::AppState;
use crate::domain::inspection::{self, InspectionItem as DomainInspectionItem};

#[derive(Debug, Deserialize)]
pub struct InspectionSubmitRequest {
    pub device_id: String,
    pub inspection_type: String,
    pub items: Vec<InspectionItem>,
    pub is_abnormal: bool,
}

#[derive(Debug, Deserialize)]
pub struct InspectionItem {
    pub item_name: String,
    pub status: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InspectionQuery {
    pub inspection_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AlertQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveAlertRequest {
    pub status: String,
    pub remarks: Option<String>,
}

pub async fn submit_inspection_handler(
    State(state): State<AppState>,
    Json(req): Json<InspectionSubmitRequest>,
) -> impl IntoResponse {
    info!("Inspection submit: device_id={}, type={}", req.device_id, req.inspection_type);
    let domain_items: Vec<DomainInspectionItem> = req.items.into_iter().map(|i| DomainInspectionItem {
        item_name: i.item_name,
        status: i.status,
        description: i.description,
    }).collect();

    match inspection::submit_inspection(
        &state.pool,
        &req.device_id,
        &req.inspection_type,
        &domain_items,
        req.is_abnormal,
    ).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => {
            error!("Submit inspection failed: {}", e);
            Json(ApiResponse::error(500, "提交检查失败".to_string()))
        }
    }
}

pub async fn list_inspections_handler(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<InspectionQuery>,
) -> impl IntoResponse {
    match inspection::list_inspections(&state.pool, query.inspection_type.as_deref(), None).await {
        Ok(records) => Json(ApiResponse::success(records)),
        Err(e) => {
            error!("List inspections failed: {}", e);
            Json(ApiResponse::error(500, "查询检查记录失败".to_string()))
        }
    }
}

pub async fn list_alerts_handler(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<AlertQuery>,
) -> impl IntoResponse {
    match inspection::list_alerts(&state.pool, query.status.as_deref(), None).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => {
            error!("List alerts failed: {}", e);
            Json(ApiResponse::error(500, "查询告警失败".to_string()))
        }
    }
}

pub async fn resolve_alert_handler(
    State(state): State<AppState>,
    axum::extract::Path(alert_id): axum::extract::Path<String>,
    Json(req): Json<ResolveAlertRequest>,
) -> impl IntoResponse {
    match inspection::resolve_alert(&state.pool, &alert_id, &req.status, req.remarks.as_deref()).await {
        Ok(()) => Json(ApiResponse::success("ok")),
        Err(e) => {
            error!("Resolve alert failed: {}", e);
            Json(ApiResponse::error(500, "处理告警失败".to_string()))
        }
    }
}
