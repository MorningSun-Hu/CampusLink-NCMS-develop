use axum::{extract::{State, Path, Query}, response::IntoResponse, Json};
use serde::Deserialize;
use tracing::{info, error};

use super::handlers::ApiResponse;
use super::handlers::AppState;
use crate::domain::process_guard;

#[derive(Debug, Deserialize)]
pub struct PolicyQuery {
    pub device_id: Option<String>,
}

pub async fn create_policy_handler(
    State(state): State<AppState>,
    Json(req): Json<process_guard::CreatePolicyRequest>,
) -> impl IntoResponse {
    info!("Create process guard policy: proc={}", req.process_name);
    match process_guard::create_policy(&state.pool, &req).await {
        Ok(policy) => Json(ApiResponse::success(policy)),
        Err(e) => {
            error!("Create policy failed: {}", e);
            Json(ApiResponse::error(500, "创建守护策略失败".to_string()))
        }
    }
}

pub async fn update_policy_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<process_guard::UpdatePolicyRequest>,
) -> impl IntoResponse {
    info!("Update process guard policy: id={}", id);
    match process_guard::update_policy(&state.pool, &id, &req).await {
        Ok(Some(policy)) => Json(ApiResponse::success(policy)),
        Ok(None) => Json(ApiResponse::error(404, "策略不存在".to_string())),
        Err(e) => {
            error!("Update policy failed: {}", e);
            Json(ApiResponse::error(500, "更新守护策略失败".to_string()))
        }
    }
}

pub async fn list_policies_handler(
    State(state): State<AppState>,
    Query(query): Query<PolicyQuery>,
) -> impl IntoResponse {
    match process_guard::list_policies(&state.pool, query.device_id.as_deref()).await {
        Ok(policies) => Json(ApiResponse::success(policies)),
        Err(e) => {
            error!("List policies failed: {}", e);
            Json(ApiResponse::error(500, "查询守护策略失败".to_string()))
        }
    }
}

pub async fn delete_policy_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Delete process guard policy: id={}", id);
    match process_guard::delete_policy(&state.pool, &id).await {
        Ok(true) => Json(ApiResponse::success("ok")),
        Ok(false) => Json(ApiResponse::error(404, "策略不存在".to_string())),
        Err(e) => {
            error!("Delete policy failed: {}", e);
            Json(ApiResponse::error(500, "删除守护策略失败".to_string()))
        }
    }
}
