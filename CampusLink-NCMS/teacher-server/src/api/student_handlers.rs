use axum::{
    extract::{State, Path, Query, Multipart},
    response::IntoResponse,
    Json,
    http::header,
};
use tracing::{info, error};

use serde::{Deserialize, Serialize};

use crate::api::handlers::ApiResponse;
use crate::api::handlers::AppState;
use crate::domain::student;

fn student_err<T: Serialize>(e: anyhow::Error) -> Json<ApiResponse<T>> {
    let msg = e.to_string();
    let code = if msg.contains("必填")
        || msg.contains("不存在")
        || msg.contains("占用")
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

pub async fn create_student_handler(
    State(state): State<AppState>,
    Json(req): Json<student::CreateStudentRequest>,
) -> impl IntoResponse {
    match student::create_student(&state.pool, req).await {
        Ok(s) => {
            info!("Student created: {} - {}", s.student_no, s.name);
            Json(ApiResponse::success(s))
        }
        Err(e) => {
            error!("Failed to create student: {}", e);
            student_err(e)
        }
    }
}

pub async fn list_students_handler(
    State(state): State<AppState>,
    Query(query): Query<student::StudentListQuery>,
) -> impl IntoResponse {
    match student::list_students(&state.pool, query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => {
            error!("Failed to list students: {}", e);
            Json(ApiResponse::error(500, format!("查询学生列表失败: {}", e)))
        }
    }
}

pub async fn get_student_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match student::get_student_by_id(&state.pool, &id).await {
        Ok(s) => Json(ApiResponse::success(s)),
        Err(e) => {
            error!("Failed to get student: {}", e);
            Json(ApiResponse::error(404, "学生不存在".to_string()))
        }
    }
}

pub async fn update_student_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<student::UpdateStudentRequest>,
) -> impl IntoResponse {
    match student::update_student(&state.pool, &id, req).await {
        Ok(s) => {
            info!("Student updated: {} - {}", s.student_no, s.name);
            Json(ApiResponse::success(s))
        }
        Err(e) => {
            error!("Failed to update student: {}", e);
            student_err(e)
        }
    }
}

pub async fn delete_student_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match student::delete_student(&state.pool, &id).await {
        Ok(_) => {
            info!("Student deleted: {}", id);
            Json(ApiResponse::success("ok".to_string()))
        }
        Err(e) => {
            error!("Failed to delete student: {}", e);
            Json(ApiResponse::error(500, format!("删除学生失败: {}", e)))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    pub password: Option<String>,
}

pub async fn reset_student_password_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    let password = req.password.filter(|p| !p.is_empty()).unwrap_or_else(|| "123456".to_string());
    match student::reset_student_password(&state.pool, &id, &password).await {
        Ok(_) => Json(ApiResponse::success("ok".to_string())),
        Err(e) => {
            error!("Reset student password failed: {}", e);
            Json(ApiResponse::error(500, "重置密码失败".to_string()))
        }
    }
}

pub async fn import_students_handler(
    State(state): State<AppState>,
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
                return match parse_and_import(&state.pool, &data).await {
                    Ok(count) => {
                        info!("Imported {} students via xlsx", count);
                        Json(ApiResponse::success(ImportResult { success: true, imported: count }))
                    }
                    Err(e) => {
                        error!("Import failed: {}", e);
                        student_err(e)
                    }
                };
            }
            Err(_) => return Json(ApiResponse::error(400, "解析上传请求失败".to_string())),
        }
    }
    Json(ApiResponse::error(400, "未找到上传文件".to_string()))
}

async fn parse_and_import(pool: &sqlx::SqlitePool, data: &[u8]) -> anyhow::Result<usize> {
    use calamine::{open_workbook_from_rs, Reader, Xlsx};
    use std::io::Cursor;

    let cursor = Cursor::new(data);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)?;
    struct PendingRow {
        line: usize,
        student_no: String,
        name: String,
        password: String,
        class_id: String,
        seat_no: String,
    }
    let mut pending: Vec<PendingRow> = Vec::new();

    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        let mut rows = range.rows();
        rows.next();

        for (idx, row) in rows.enumerate() {
            let line = idx + 2;
            let student_no = cell_text(row.get(0));
            let name = cell_text(row.get(1));
            let class_name = cell_text(row.get(2));
            let seat_no = cell_text(row.get(3));
            let password = cell_text(row.get(4));

            if student_no.is_empty() && name.is_empty() && class_name.is_empty() && seat_no.is_empty() {
                continue;
            }
            if student_no == "学号" || name == "姓名" {
                continue;
            }
            if name.is_empty() {
                anyhow::bail!("第{}行：姓名为必填项", line);
            }
            if class_name.is_empty() {
                anyhow::bail!("第{}行：班级为必填项", line);
            }
            if seat_no.is_empty() {
                anyhow::bail!("第{}行：座位号为必填项", line);
            }

            let class_id = student::find_class_id_by_name(pool, &class_name).await?
                .ok_or_else(|| anyhow::anyhow!("第{}行：班级「{}」不存在", line, class_name))?;

            if let Err(e) = student::require_class_and_existing_seat(pool, &class_id, &seat_no).await {
                anyhow::bail!("第{}行：{}", line, e);
            }

            let student_no = if student_no.trim().is_empty() {
                student::generate_student_no()
            } else {
                student_no
            };
            let occupied: Option<String> = sqlx::query_scalar(
                "SELECT student_no FROM students WHERE class_id = ? AND TRIM(COALESCE(seat_no, '')) = ?"
            )
            .bind(&class_id)
            .bind(&seat_no)
            .fetch_optional(pool)
            .await?;
            if let Some(ref existing_no) = occupied {
                if existing_no != &student_no {
                    anyhow::bail!("第{}行：座位号 {} 已被学号 {} 占用", line, seat_no, existing_no);
                }
            }
            let password = if password.is_empty() { "123456".to_string() } else { password };

            pending.push(PendingRow {
                line,
                student_no,
                name,
                password,
                class_id,
                seat_no,
            });
        }
    }

    if pending.is_empty() {
        anyhow::bail!("导入文件没有有效的学生数据");
    }
    let mut seen_seats = std::collections::HashSet::new();
    for row in &pending {
        if !seen_seats.insert((row.class_id.clone(), row.seat_no.clone())) {
            anyhow::bail!("第{}行：座位号 {} 在导入文件中重复", row.line, row.seat_no);
        }
    }

    let mut tx = pool.begin().await?;
    for row in &pending {
        if let Err(e) = student::insert_imported_student(&mut *tx, &row.student_no, &row.name, &row.password, &row.seat_no, &row.class_id).await {
            anyhow::bail!("第{}行：{}", row.line, e);
        }
    }
    tx.commit().await?;

    Ok(pending.len())
}

#[derive(serde::Serialize)]
struct ImportResult {
    success: bool,
    imported: usize,
}

pub async fn export_students_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match generate_xlsx(&state.pool).await {
        Ok(data) => {
            let headers = [
                (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"students.xlsx\""),
            ];
            (headers, data).into_response()
        }
        Err(e) => {
            error!("Export failed: {}", e);
            Json(ApiResponse::<()>::error(500, format!("导出失败: {}", e))).into_response()
        }
    }
}

async fn generate_xlsx(pool: &sqlx::SqlitePool) -> anyhow::Result<Vec<u8>> {
    use rust_xlsxwriter::{Workbook, Format};
    use std::io::Cursor;

    let rows = student::list_all_students(pool).await?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let header_format = Format::new().set_bold();

    worksheet.write_with_format(0, 0, "学号", &header_format)?;
    worksheet.write_with_format(0, 1, "姓名", &header_format)?;
    worksheet.write_with_format(0, 2, "班级", &header_format)?;
    worksheet.write_with_format(0, 3, "座位号", &header_format)?;
    worksheet.write_with_format(0, 4, "状态", &header_format)?;
    worksheet.write_with_format(0, 5, "创建时间", &header_format)?;

    for (i, row) in rows.iter().enumerate() {
        let r = (i + 1) as u32;
        let class_name = student::class_name_of(pool, row.class_id.as_deref()).await?.unwrap_or_default();
        worksheet.write(r, 0, &row.student_no)?;
        worksheet.write(r, 1, &row.name)?;
        worksheet.write(r, 2, class_name)?;
        worksheet.write(r, 3, row.seat_no.as_deref().unwrap_or(""))?;
        worksheet.write(r, 4, &row.status)?;
        worksheet.write(r, 5, &row.created_at)?;
    }

    let mut buf = Cursor::new(Vec::new());
    workbook.save_to_writer(&mut buf)?;
    Ok(buf.into_inner())
}

pub async fn student_import_template_handler() -> impl IntoResponse {
    match generate_import_template() {
        Ok(data) => {
            let headers = [
                (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"students-template.xlsx\""),
            ];
            (headers, data).into_response()
        }
        Err(e) => {
            error!("Generate student template failed: {}", e);
            Json(ApiResponse::<()>::error(500, "生成导入模板失败".to_string())).into_response()
        }
    }
}

fn generate_import_template() -> anyhow::Result<Vec<u8>> {
    use rust_xlsxwriter::{Workbook, Format};
    use std::io::Cursor;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let header_format = Format::new().set_bold();

    worksheet.write_with_format(0, 0, "学号", &header_format)?;
    worksheet.write_with_format(0, 1, "姓名", &header_format)?;
    worksheet.write_with_format(0, 2, "班级", &header_format)?;
    worksheet.write_with_format(0, 3, "座位号", &header_format)?;
    worksheet.write_with_format(0, 4, "密码", &header_format)?;
    worksheet.write(1, 0, "S0001")?;
    worksheet.write(1, 1, "张三")?;
    worksheet.write(1, 2, "请填写已有班级名称")?;
    worksheet.write(1, 3, "请填写该班设备已分配的座位号")?;
    worksheet.write(1, 4, "123456")?;
    worksheet.set_column_width(0, 16)?;
    worksheet.set_column_width(1, 12)?;
    worksheet.set_column_width(2, 24)?;
    worksheet.set_column_width(3, 36)?;
    worksheet.set_column_width(4, 12)?;

    let mut buf = Cursor::new(Vec::new());
    workbook.save_to_writer(&mut buf)?;
    Ok(buf.into_inner())
}
