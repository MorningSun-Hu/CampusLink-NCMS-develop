use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
pub struct StudentSetPasswordRequest {
    #[serde(alias = "studentId")]
    pub student_id: String,
    #[serde(alias = "oldPassword")]
    pub old_password: String,
    #[serde(alias = "newPassword")]
    pub new_password: String,
}

pub async fn student_set_password_handler(
    State(state): State<AppState>,
    Json(req): Json<StudentSetPasswordRequest>,
) -> impl IntoResponse {
    match auth::change_student_password(
        &state.pool,
        &req.student_id,
        &req.old_password,
        &req.new_password,
    )
    .await
    {
        Ok(_) => {
            info!("Student {} set custom password", req.student_id);
            Json(ApiResponse::success("ok".to_string()))
        }
        Err(e) => {
            error!("Student set password failed: {}", e);
            Json(ApiResponse::error(400, e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StudentSetPasswordRequest;

    #[test]
    fn set_password_request_accepts_snake_and_camel_case() {
        let snake: StudentSetPasswordRequest =
            serde_json::from_str(r#"{"student_id":"s1","old_password":"123456","new_password":"abc123"}"#).unwrap();
        assert_eq!(snake.student_id, "s1");
        assert_eq!(snake.old_password, "123456");
        assert_eq!(snake.new_password, "abc123");

        let camel: StudentSetPasswordRequest =
            serde_json::from_str(r#"{"studentId":"s2","oldPassword":"123456","newPassword":"abc123"}"#).unwrap();
        assert_eq!(camel.student_id, "s2");
        assert_eq!(camel.old_password, "123456");
        assert_eq!(camel.new_password, "abc123");
    }
}
