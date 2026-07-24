use axum::{extract::State, extract::Path, response::IntoResponse, Json};
use tracing::error;

use super::handlers::{ApiResponse, AppState};
use crate::domain::network_account;

pub async fn list_accounts_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match network_account::list_accounts(&state.pool).await {
        Ok(accounts) => Json(ApiResponse::success(accounts)),
        Err(e) => {
            error!("List network accounts failed: {}", e);
            Json(ApiResponse::error(500, "查询网络账号失败".to_string()))
        }
    }
}

pub async fn create_account_handler(
    State(state): State<AppState>,
    Json(req): Json<network_account::CreateNetworkAccountRequest>,
) -> impl IntoResponse {
    match network_account::create_account(&state.pool, req).await {
        Ok(account) => Json(ApiResponse::success(account)),
        Err(e) => {
            error!("Create network account failed: {}", e);
            Json(ApiResponse::error(500, "创建网络账号失败".to_string()))
        }
    }
}

pub async fn update_account_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<network_account::UpdateNetworkAccountRequest>,
) -> impl IntoResponse {
    match network_account::update_account(&state.pool, &id, req).await {
        Ok(account) => Json(ApiResponse::success(account)),
        Err(e) => {
            error!("Update network account failed: {}", e);
            Json(ApiResponse::error(500, "更新网络账号失败".to_string()))
        }
    }
}

pub async fn delete_account_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match network_account::delete_account(&state.pool, &id).await {
        Ok(()) => Json(ApiResponse::success("ok")),
        Err(e) => {
            error!("Delete network account failed: {}", e);
            Json(ApiResponse::error(500, "删除网络账号失败".to_string()))
        }
    }
}
