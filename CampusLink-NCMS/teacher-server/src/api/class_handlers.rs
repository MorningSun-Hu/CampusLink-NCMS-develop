use axum::{
    extract::{State, Path, Multipart},
    response::IntoResponse,
    Json,
    http::header,
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

fn seat_json_err<T: serde::Serialize>(e: anyhow::Error) -> Json<ApiResponse<T>> {
    let msg = e.to_string();
    let code = if msg.contains("重复")
        || msg.contains("占用")
        || msg.contains("不属于")
        || msg.contains("必填")
        || msg.contains("没有有效")
        || msg.starts_with("第")
    {
        400
    } else {
        500
    };
    Json(ApiResponse::error(code, msg))
}

fn cell_text(cell: Option<&calamine::Data>) -> String {
    let raw = match cell {
        Some(value) => value.to_string(),
        None => return String::new(),
    };
    let trimmed = raw.trim().to_string();
    if let Ok(number) = trimmed.parse::<f64>() {
        if number.fract() == 0.0 && number.abs() < 1e15 {
            return format!("{}", number as i64);
        }
    }
    trimmed
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
        Ok(count) => {
            match class::fill_missing_seats(&state.pool, &id).await {
                Ok(seats) => Json(ApiResponse::success(serde_json::json!({
                    "assigned": count,
                    "seats": seats,
                }))),
                Err(e) => {
                    error!("Fill seats after assign failed: {}", e);
                    seat_json_err(e)
                }
            }
        }
        Err(e) => {
            error!("Assign devices failed: {}", e);
            Json(ApiResponse::error(500, "添加学生机失败".to_string()))
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
            seat_json_err(e)
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

pub async fn renumber_seats_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match class::renumber_seats(&state.pool, &id).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => {
            error!("Renumber seats failed: {}", e);
            seat_json_err(e)
        }
    }
}

pub async fn export_class_seats_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match generate_class_seats_xlsx(&state.pool, &id).await {
        Ok(data) => {
            let headers = [
                (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"class-seats.xlsx\""),
            ];
            (headers, data).into_response()
        }
        Err(e) => {
            error!("Export class seats failed: {}", e);
            Json(ApiResponse::<()>::error(500, format!("导出座位表失败: {}", e))).into_response()
        }
    }
}

pub async fn import_class_seats_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.transpose() {
        match field {
            Ok(field) => {
                let name = field.name().unwrap_or("").to_string();
                if name != "file" {
                    continue;
                }
                let data = match field.bytes().await {
                    Ok(d) => d,
                    Err(_) => return Json(ApiResponse::error(400, "读取上传文件失败".to_string())),
                };
                return match parse_and_import_seats(&state.pool, &id, &data).await {
                    Ok(count) => Json(ApiResponse::success(serde_json::json!({ "imported": count }))),
                    Err(e) => {
                        error!("Import class seats failed: {}", e);
                        seat_json_err(e)
                    }
                };
            }
            Err(_) => return Json(ApiResponse::error(400, "解析上传请求失败".to_string())),
        }
    }
    Json(ApiResponse::error(400, "未找到上传文件".to_string()))
}

async fn generate_class_seats_xlsx(pool: &sqlx::SqlitePool, class_id: &str) -> anyhow::Result<Vec<u8>> {
    use rust_xlsxwriter::{Workbook, Format};
    use std::io::Cursor;

    let devices = device::list_devices_by_class(pool, class_id).await?;
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let header_format = Format::new().set_bold();
    worksheet.write_with_format(0, 0, "设备ID", &header_format)?;
    worksheet.write_with_format(0, 1, "设备名", &header_format)?;
    worksheet.write_with_format(0, 2, "IP", &header_format)?;
    worksheet.write_with_format(0, 3, "座位号", &header_format)?;
    for (i, d) in devices.iter().enumerate() {
        let r = (i + 1) as u32;
        worksheet.write(r, 0, &d.id)?;
        worksheet.write(r, 1, &d.device_name)?;
        worksheet.write(r, 2, &d.ip_address)?;
        worksheet.write(r, 3, d.seat_no.as_deref().unwrap_or(""))?;
    }
    worksheet.set_column_width(0, 36)?;
    worksheet.set_column_width(1, 18)?;
    worksheet.set_column_width(2, 16)?;
    worksheet.set_column_width(3, 12)?;
    let mut buf = Cursor::new(Vec::new());
    workbook.save_to_writer(&mut buf)?;
    Ok(buf.into_inner())
}

async fn parse_and_import_seats(pool: &sqlx::SqlitePool, class_id: &str, data: &[u8]) -> anyhow::Result<usize> {
    use calamine::{open_workbook_from_rs, Reader, Xlsx};
    use std::io::Cursor;

    let mut workbook: Xlsx<_> = open_workbook_from_rs(Cursor::new(data))?;
    let mut pending: Vec<class::SeatAssignment> = Vec::new();
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        let mut rows = range.rows();
        rows.next();
        for (idx, row) in rows.enumerate() {
            let line = idx + 2;
            let device_id = cell_text(row.get(0));
            let seat_no = cell_text(row.get(3));
            if device_id.is_empty() && seat_no.is_empty() {
                continue;
            }
            if device_id == "设备ID" {
                continue;
            }
            if device_id.is_empty() {
                anyhow::bail!("第{}行：设备ID为必填项", line);
            }
            if seat_no.is_empty() {
                anyhow::bail!("第{}行：座位号为必填项", line);
            }
            pending.push(class::SeatAssignment { device_id, seat_no });
        }
    }
    if pending.is_empty() {
        anyhow::bail!("导入文件没有有效的座位数据");
    }
    class::batch_set_seats(pool, class_id, &pending).await?;
    Ok(pending.len())
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
        Ok(result) => {
            let checkin_device_ids: Vec<String> = if req.target_mode == "teaching" {
                result.teaching_device_ids.clone()
            } else if requires_checkin(&req.target_mode) {
                result.device_ids.clone()
            } else {
                Vec::new()
            };
            if !checkin_device_ids.is_empty() {
                dispatch_checkin_trigger(&state, &checkin_device_ids).await;
            }
            info!(
                "Class {} switched to {} (teaching: {}, locked: {})",
                id,
                req.target_mode,
                result.teaching_device_ids.len(),
                result.locked_device_ids.len()
            );
            Json(ApiResponse::success(serde_json::json!({
                "classId": id,
                "targetMode": req.target_mode,
                "affectedDevices": result.device_ids.len(),
                "teachingDeviceIds": result.teaching_device_ids,
                "lockedDeviceIds": result.locked_device_ids,
                "teachingCount": result.teaching_device_ids.len(),
                "lockedCount": result.locked_device_ids.len(),
            })))
        }
        Err(e) => {
            error!("Switch class mode failed: {}", e);
            Json(ApiResponse::error(400, e.to_string()))
        }
    }
}
