use axum::{
    extract::{Multipart, State, Path},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::{info, error};
use std::path::PathBuf;

use super::handlers::ApiResponse;
use super::handlers::AppState;
use crate::domain::photo;

#[derive(Debug, Deserialize)]
pub struct PhotoUploadQuery {
    pub alert_id: Option<i64>,
    pub inspection_id: Option<String>,
    pub description: Option<String>,
}

pub async fn upload_photo_handler(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<PhotoUploadQuery>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let upload_dir = PathBuf::from("data/uploads");
    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        error!("Failed to create upload dir: {}", e);
        return Json(ApiResponse::error(500, "创建上传目录失败".to_string()));
    }

    let mut results = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name()
            .unwrap_or("unknown.bin")
            .to_string();

        let content_type = field.content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        if !content_type.starts_with("image/") {
            continue;
        }

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to read upload data: {}", e);
                continue;
            }
        };

        if data.len() > 2 * 1024 * 1024 {
            continue;
        }

        let file_name = format!("{}_{}", uuid::Uuid::new_v4(), file_name);
        let file_path = upload_dir.join(&file_name);
        let file_path_str = file_path.to_string_lossy().to_string();

        if let Err(e) = std::fs::write(&file_path, &data) {
            error!("Failed to save uploaded file: {}", e);
            continue;
        }

        match photo::create_photo(
            &state.pool,
            &file_path_str,
            data.len() as i64,
            &content_type,
            query.alert_id,
            query.inspection_id.as_deref(),
            query.description.as_deref(),
        ).await {
            Ok(record) => {
                info!("Photo uploaded: id={}", record.id);
                results.push(record);
            }
            Err(e) => {
                error!("Failed to save photo record: {}", e);
            }
        }
    }

    if results.is_empty() {
        Json(ApiResponse::error(400, "无有效图片上传".to_string()))
    } else {
        Json(ApiResponse::success(results))
    }
}

pub async fn get_photo_handler(
    State(state): State<AppState>,
    Path(photo_id): Path<String>,
) -> impl IntoResponse {
    match photo::get_photo_by_id(&state.pool, &photo_id).await {
        Ok(record) => {
            match std::fs::read(&record.file_path) {
                Ok(data) => {
                    let content_type = record.mime_type.clone();
                    let headers = [
                        ("Content-Type", content_type),
                        ("Content-Disposition", format!("inline; filename=\"{}\"", record.id)),
                    ];
                    (axum::http::StatusCode::OK, headers, axum::body::Body::from(data)).into_response()
                }
                Err(e) => {
                    error!("Failed to read photo file: {}", e);
                    Json(ApiResponse::<String>::error(404, "图片文件不存在".to_string())).into_response()
                }
            }
        }
        Err(e) => {
            error!("Photo not found: {}", e);
                    Json(ApiResponse::<String>::error(404, "图片记录不存在".to_string())).into_response()
        }
    }
}

pub async fn list_photos_handler(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<InspectionPhotoQuery>,
) -> impl IntoResponse {
    match photo::list_photos(&state.pool, query.inspection_id.as_deref()).await {
        Ok(photos) => Json(ApiResponse::success(photos)),
        Err(e) => {
            error!("List photos failed: {}", e);
            Json(ApiResponse::error(500, "查询图片失败".to_string()))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct InspectionPhotoQuery {
    pub inspection_id: Option<String>,
}
