use axum::{
    extract::{State, Path, Query},
    response::IntoResponse,
    Json,
};
use tracing::{info, error};

use crate::api::handlers::ApiResponse;
use crate::api::handlers::AppState;
use crate::domain::repair;

pub async fn create_repair_handler(
    State(state): State<AppState>,
    Json(req): Json<repair::CreateRepairRequest>,
) -> impl IntoResponse {
    match repair::create_repair(&state.pool, req).await {
        Ok(order) => {
            info!("Repair order created: {}", order.id);
            Json(ApiResponse::success(order))
        }
        Err(e) => {
            error!("Failed to create repair: {}", e);
            Json(ApiResponse::error(500, format!("创建工单失败: {}", e)))
        }
    }
}

pub async fn list_repairs_handler(
    State(state): State<AppState>,
    Query(query): Query<repair::RepairListQuery>,
) -> impl IntoResponse {
    match repair::list_repairs(&state.pool, query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => {
            error!("Failed to list repairs: {}", e);
            Json(ApiResponse::error(500, format!("查询工单失败: {}", e)))
        }
    }
}

pub async fn update_repair_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<repair::UpdateRepairRequest>,
) -> impl IntoResponse {
    match repair::update_repair(&state.pool, &id, req).await {
        Ok(order) => {
            info!("Repair order updated: {}", id);
            Json(ApiResponse::success(order))
        }
        Err(e) => {
            error!("Failed to update repair: {}", e);
            Json(ApiResponse::error(500, format!("更新工单失败: {}", e)))
        }
    }
}

pub async fn delete_repair_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repair::delete_repair(&state.pool, &id).await {
        Ok(_) => {
            info!("Repair order deleted: {}", id);
            Json(ApiResponse::success("ok".to_string()))
        }
        Err(e) => {
            error!("Failed to delete repair: {}", e);
            Json(ApiResponse::error(500, format!("删除工单失败: {}", e)))
        }
    }
}
