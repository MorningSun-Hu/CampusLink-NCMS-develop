use axum::{extract::Extension, extract::Path, extract::State, response::IntoResponse, Json};
use serde::Deserialize;
use tracing::error;

use super::handlers::{ApiResponse, AppState};
use crate::domain::auth::Claims;
use crate::domain::user;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetUserPasswordRequest {
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeOwnPasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

fn error_code(message: &str) -> i32 {
    if message.contains("不存在") {
        404
    } else {
        400
    }
}

pub async fn list_users_handler(State(state): State<AppState>) -> impl IntoResponse {
    match user::list_users(&state.pool).await {
        Ok(users) => Json(ApiResponse::success(users)),
        Err(e) => {
            error!("List users failed: {}", e);
            Json(ApiResponse::error(500, "查询账号失败".to_string()))
        }
    }
}

pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    match user::create_teacher(&state.pool, &req.username, &req.password, &req.display_name).await {
        Ok(created) => Json(ApiResponse::success(created)),
        Err(e) => {
            let message = e.to_string();
            Json(ApiResponse::error(error_code(&message), message))
        }
    }
}

pub async fn update_user_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    match user::update_user(&state.pool, &id, req.display_name, req.status).await {
        Ok(updated) => Json(ApiResponse::success(updated)),
        Err(e) => {
            let message = e.to_string();
            Json(ApiResponse::error(error_code(&message), message))
        }
    }
}

pub async fn delete_user_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match user::delete_user(&state.pool, &id).await {
        Ok(()) => Json(ApiResponse::success("ok".to_string())),
        Err(e) => {
            let message = e.to_string();
            Json(ApiResponse::error(error_code(&message), message))
        }
    }
}

pub async fn reset_user_password_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ResetUserPasswordRequest>,
) -> impl IntoResponse {
    match user::reset_teacher_password(&state.pool, &id, &req.new_password).await {
        Ok(()) => Json(ApiResponse::success("ok".to_string())),
        Err(e) => {
            let message = e.to_string();
            Json(ApiResponse::error(error_code(&message), message))
        }
    }
}

pub async fn change_own_password_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ChangeOwnPasswordRequest>,
) -> impl IntoResponse {
    match user::change_own_password(&state.pool, &claims.sub, &req.old_password, &req.new_password).await {
        Ok(()) => Json(ApiResponse::success("ok".to_string())),
        Err(e) => {
            let message = e.to_string();
            Json(ApiResponse::error(error_code(&message), message))
        }
    }
}
