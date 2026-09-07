use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use tracing::{info, error};

use crate::api::handlers::ApiResponse;
use crate::api::handlers::AppState;
use crate::domain::auth;

pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<auth::LoginRequest>,
) -> impl IntoResponse {
    match auth::login(&state.pool, &req).await {
        Ok(resp) => {
            info!("User logged in: {}", resp.username);
            Json(ApiResponse::success(resp))
        }
        Err(e) => {
            error!("Login failed: {}", e);
            Json(ApiResponse::error(401, "用户名或密码错误".to_string()))
        }
    }
}

pub async fn student_login_handler(
    State(state): State<AppState>,
    Json(req): Json<auth::StudentLoginRequest>,
) -> impl IntoResponse {
    match auth::student_login(&state.pool, &req).await {
        Ok(resp) => {
            info!("Student logged in: {}", resp.student_no);
            Json(ApiResponse::success(resp))
        }
        Err(e) => {
            error!("Student login failed: {}", e);
            Json(ApiResponse::error(401, e.to_string()))
        }
    }
}
