use axum::{
    extract::{State, Path},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::{info, error};

use super::handlers::{ApiResponse, AppState, dispatch_checkin_trigger, requires_checkin};
use crate::domain::{class, device};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClassRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignStudentsRequest {
    pub student_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignDevicesRequest {
    pub device_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassModeRequest {
    pub target_mode: String,
    pub operator_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordsRequest {
    pub password: Option<String>,
}

pub async fn list_classes_handler(State(state): State<AppState>) -> impl IntoResponse {
    match class::list_classes(&state.pool).await {
        Ok(classes) => Json(ApiResponse::success(classes)),
        Err(e) => {
            error!("List classes failed: {}", e);
            Json(ApiResponse::error(500, "查询班级失败".to_string()))
        }
    }
}

pub async fn create_class_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateClassRequest>,
) -> impl IntoResponse {
    match class::create_class(&state.pool, &req.name).await {
        Ok(c) => Json(ApiResponse::success(c)),
        Err(e) => {
            error!("Create class failed: {}", e);
            Json(ApiResponse::error(500, format!("创建班级失败: {}", e)))
        }
    }
}

pub async fn update_class_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateClassRequest>,
) -> impl IntoResponse {
    match class::update_class(&state.pool, &id, &req.name).await {
        Ok(c) => Json(ApiResponse::success(c)),
        Err(e) => {
            error!("Update class failed: {}", e);
            Json(ApiResponse::error(500, format!("更新班级失败: {}", e)))
        }
    }
}

pub async fn delete_class_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match class::delete_class(&state.pool, &id).await {
        Ok(_) => Json(ApiResponse::success("ok".to_string())),
        Err(e) => {
            error!("Delete class failed: {}", e);
            Json(ApiResponse::error(500, format!("删除班级失败: {}", e)))
        }
    }
}

pub async fn assign_students_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AssignStudentsRequest>,
) -> impl IntoResponse {
    match class::assign_students(&state.pool, &id, &req.student_ids).await {
        Ok(count) => Json(ApiResponse::success(serde_json::json!({ "assigned": count }))),
        Err(e) => {
            error!("Assign students failed: {}", e);
            Json(ApiResponse::error(500, "学生归班失败".to_string()))
        }
    }
}

pub async fn list_class_students_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match class::list_students(&state.pool, &id).await {
        Ok(students) => Json(ApiResponse::success(students)),
        Err(e) => {
            error!("List class students failed: {}", e);
            Json(ApiResponse::error(500, "查询班级学生失败".to_string()))
        }
    }
}

pub async fn assign_devices_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AssignDevicesRequest>,
) -> impl IntoResponse {
    match class::assign_devices(&state.pool, &id, &req.device_ids).await {
        Ok(count) => Json(ApiResponse::success(serde_json::json!({ "assigned": count }))),
        Err(e) => {
            error!("Assign devices failed: {}", e);
            Json(ApiResponse::error(500, "设备归班失败".to_string()))
        }
    }
}

pub async fn list_class_devices_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match device::list_devices_by_class(&state.pool, &id).await {
        Ok(devices) => Json(ApiResponse::success(devices)),
        Err(e) => {
            error!("List class devices failed: {}", e);
            Json(ApiResponse::error(500, "查询班级设备失败".to_string()))
        }
    }
}

pub async fn batch_seats_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<class::BatchSeatRequest>,
) -> impl IntoResponse {
    match class::batch_set_seats(&state.pool, &id, &req.seats).await {
        Ok(_) => Json(ApiResponse::success("ok".to_string())),
        Err(e) => {
            error!("Batch set seats failed: {}", e);
            Json(ApiResponse::error(500, format!("设置座位号失败: {}", e)))
        }
    }
}

pub async fn auto_seats_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match class::auto_seats_by_ip(&state.pool, &id).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => {
            error!("Auto assign seats failed: {}", e);
            Json(ApiResponse::error(500, format!("自动分配座位失败: {}", e)))
        }
    }
}

pub async fn reset_class_passwords_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ResetPasswordsRequest>,
) -> impl IntoResponse {
    let password = req.password.filter(|p| !p.is_empty()).unwrap_or_else(|| "123456".to_string());
    match crate::domain::student::reset_class_passwords(&state.pool, &id, &password).await {
        Ok(count) => Json(ApiResponse::success(serde_json::json!({ "reset": count }))),
        Err(e) => {
            error!("Reset class passwords failed: {}", e);
            Json(ApiResponse::error(500, "重置密码失败".to_string()))
        }
    }
}

pub async fn switch_class_mode_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ClassModeRequest>,
) -> impl IntoResponse {
    const VALID_MODES: [&str; 4] = ["open", "teaching", "exam", "locked"];
    if !VALID_MODES.contains(&req.target_mode.as_str()) {
        return Json(ApiResponse::error(
            400,
            format!("非法模式: {}（可选: open/teaching/exam/locked）", req.target_mode),
        ));
    }

    match class::switch_class_mode(&state.pool, &id, &req.target_mode).await {
        Ok(device_ids) => {
            if requires_checkin(&req.target_mode) {
                dispatch_checkin_trigger(&state, &device_ids).await;
            }
            info!("Class {} switched to {} ({} devices)", id, req.target_mode, device_ids.len());
            Json(ApiResponse::success(serde_json::json!({
                "classId": id,
                "targetMode": req.target_mode,
                "affectedDevices": device_ids.len(),
            })))
        }
        Err(e) => {
            error!("Switch class mode failed: {}", e);
            Json(ApiResponse::error(400, e.to_string()))
        }
    }
}
