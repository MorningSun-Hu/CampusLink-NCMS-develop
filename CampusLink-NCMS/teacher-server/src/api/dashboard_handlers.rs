use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use sqlx::SqlitePool;
use tracing::error;

use super::handlers::ApiResponse;
use super::handlers::AppState;

#[derive(Debug, Serialize)]
pub struct DashboardOverview {
    pub student_count: i64,
    pub registered_device_count: i64,
    pub online_device_count: i64,
    pub offline_device_count: i64,
}

pub async fn dashboard_overview_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match get_dashboard_overview(&state.pool).await {
        Ok(data) => Json(ApiResponse::success(data)),
        Err(e) => {
            error!("Dashboard overview failed: {}", e);
            Json(ApiResponse::error(500, "获取仪表盘数据失败".to_string()))
        }
    }
}

async fn get_dashboard_overview(pool: &SqlitePool) -> Result<DashboardOverview, anyhow::Error> {
    let student_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM students")
        .fetch_one(pool)
        .await?;

    let registered_device_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM student_devices")
        .fetch_one(pool)
        .await?;

    let online_device_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM student_devices WHERE online_status = 'online'"
    )
    .fetch_one(pool)
    .await?;

    let offline_device_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM student_devices WHERE online_status = 'offline'"
    )
    .fetch_one(pool)
    .await?;

    Ok(DashboardOverview {
        student_count: student_count.0,
        registered_device_count: registered_device_count.0,
        online_device_count: online_device_count.0,
        offline_device_count: offline_device_count.0,
    })
}
