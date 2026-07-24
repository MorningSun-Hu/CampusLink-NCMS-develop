use axum::{
    extract::{State, Path, Query, Multipart},
    response::IntoResponse,
    Json,
    http::header,
};
use tracing::{info, error};

use crate::api::handlers::ApiResponse;
use crate::api::handlers::AppState;
use crate::domain::student;

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
            Json(ApiResponse::error(500, format!("创建学生失败: {}", e)))
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
            Json(ApiResponse::error(500, format!("更新学生失败: {}", e)))
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
                        Json(ApiResponse::error(500, format!("导入失败: {}", e)))
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
    let mut count = 0;

    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        let mut rows = range.rows();
        // Skip header
        rows.next();

        for row in rows {
            let student_no = row.get(0).map(|c| c.to_string()).unwrap_or_default();
            let name = row.get(1).map(|c| c.to_string()).unwrap_or_default();
            let password = row.get(2).map(|c| c.to_string()).unwrap_or_else(|| "123456".to_string());
            let seat_no = row.get(3).map(|c| c.to_string());
            let seat_no = seat_no.as_deref().filter(|s| !s.is_empty());

            if student_no.is_empty() || name.is_empty() {
                continue;
            }

            student::import_student(pool, &student_no, &name, &password, seat_no).await?;
            count += 1;
        }
    }

    Ok(count)
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
    use rust_xlsxwriter::{Workbook, Format, Color};
    use std::io::Cursor;

    let rows = student::list_all_students(pool).await?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let header_format = Format::new().set_bold();

    worksheet.write_with_format(0, 0, "学号", &header_format)?;
    worksheet.write_with_format(0, 1, "姓名", &header_format)?;
    worksheet.write_with_format(0, 2, "座位号", &header_format)?;
    worksheet.write_with_format(0, 3, "状态", &header_format)?;
    worksheet.write_with_format(0, 4, "创建时间", &header_format)?;

    for (i, row) in rows.iter().enumerate() {
        let r = (i + 1) as u32;
        worksheet.write(r, 0, &row.student_no)?;
        worksheet.write(r, 1, &row.name)?;
        worksheet.write(r, 2, row.seat_no.as_deref().unwrap_or(""))?;
        worksheet.write(r, 3, &row.status)?;
        worksheet.write(r, 4, &row.created_at)?;
    }

    let mut buf = Cursor::new(Vec::new());
    workbook.save_to_writer(&mut buf)?;
    Ok(buf.into_inner())
}
