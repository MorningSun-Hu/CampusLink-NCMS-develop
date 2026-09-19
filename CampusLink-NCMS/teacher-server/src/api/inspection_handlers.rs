use axum::{extract::State, response::IntoResponse, Json};
use axum::http::header;
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

    if !crate::domain::device::device_exists(&state.pool, &req.device_id).await.unwrap_or(false) {
        return Json(ApiResponse::error(401, "设备未注册，请先注册".to_string()));
    }

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

pub async fn export_inspection_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match generate_inspection_csv(&state.pool).await {
        Ok(data) => {
            let headers = [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"inspections.csv\""),
            ];
            (headers, data).into_response()
        }
        Err(e) => {
            error!("Export inspection failed: {}", e);
            Json(ApiResponse::<()>::error(500, format!("导出失败: {}", e))).into_response()
        }
    }
}

async fn generate_inspection_csv(pool: &sqlx::SqlitePool) -> anyhow::Result<Vec<u8>> {
    let records = inspection::list_inspections(pool, None, Some(10000)).await?;
    let mut csv = String::from("ID,设备ID,设备名称,检查类型,检查项目,状态,描述,检查人,创建时间\n");
    for r in records {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            r.id,
            r.device_id,
            r.device_name.as_deref().unwrap_or(""),
            r.inspection_type,
            r.item_name,
            r.status,
            r.description.as_deref().unwrap_or(""),
            r.inspector.as_deref().unwrap_or(""),
            r.created_at,
        ));
    }
    Ok(csv.into_bytes())
}
