use axum::{
    extract::{State, Path, Multipart},
    response::IntoResponse,
    Json,
};
use tracing::{info, error};

use crate::api::handlers::ApiResponse;
use crate::api::handlers::AppState;
use crate::domain::device;

pub async fn list_whitelist_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match device::list_whitelist(&state.pool).await {
        Ok(entries) => Json(ApiResponse::success(entries)),
        Err(e) => {
            error!("Failed to list whitelist: {}", e);
            Json(ApiResponse::error(500, format!("查询白名单失败: {}", e)))
        }
    }
}

pub async fn import_whitelist_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.transpose() {
        match field {
            Ok(field) => {
                if field.name().unwrap_or("") != "file" {
                    continue;
                }
                let data = match field.bytes().await {
                    Ok(d) => d,
                    Err(_) => return Json(ApiResponse::error(400, "读取上传文件失败".to_string())),
                };
                return match parse_whitelist_xlsx(&state.pool, &data).await {
                    Ok(count) => {
                        info!("Imported {} whitelist entries", count);
                        Json(ApiResponse::success(serde_json::json!({"success": true, "imported": count})))
                    }
                    Err(e) => {
                        error!("Whitelist import failed: {}", e);
                        Json(ApiResponse::error(500, format!("导入失败: {}", e)))
                    }
                };
            }
            Err(_) => return Json(ApiResponse::error(400, "解析上传请求失败".to_string())),
        }
    }
    Json(ApiResponse::error(400, "未找到上传文件".to_string()))
}

async fn parse_whitelist_xlsx(pool: &sqlx::SqlitePool, data: &[u8]) -> anyhow::Result<usize> {
    use calamine::{open_workbook_from_rs, Reader, Xlsx};
    use std::io::Cursor;

    let cursor = Cursor::new(data);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)?;
    let mut count = 0;

    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        let mut rows = range.rows();
        rows.next();

        for row in rows {
            let device_code = row.get(0).map(|c| c.to_string()).unwrap_or_default();
            let device_name = row.get(1).map(|c| c.to_string()).unwrap_or_default();
            let mac_address = row.get(2).map(|c| c.to_string()).unwrap_or_default();

            if device_code.is_empty() {
                continue;
            }

            device::import_whitelist(pool, &device_code, &device_name, &mac_address).await?;
            count += 1;
        }
    }

    Ok(count)
}

pub async fn approve_whitelist_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match device::approve_whitelist(&state.pool, &id).await {
        Ok(_) => {
            info!("Whitelist entry approved: {}", id);
            Json(ApiResponse::success("ok".to_string()))
        }
        Err(e) => {
            error!("Failed to approve whitelist: {}", e);
            Json(ApiResponse::error(500, format!("批准失败: {}", e)))
        }
    }
}

pub async fn delete_whitelist_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match device::delete_whitelist(&state.pool, &id).await {
        Ok(_) => {
            info!("Whitelist entry deleted: {}", id);
            Json(ApiResponse::success("ok".to_string()))
        }
        Err(e) => {
            error!("Failed to delete whitelist: {}", e);
            Json(ApiResponse::error(500, format!("删除失败: {}", e)))
        }
    }
}
