use axum::{extract::{State, Path}, response::IntoResponse, Json};
use axum::http::header;
use serde::Deserialize;
use tracing::error;

use super::handlers::{ApiResponse, AppState};
use super::handlers::{dispatch_checkin_trigger, requires_checkin};
use crate::domain::usage::{self, UsageQuery, UsageRecord};

pub async fn list_usage_handler(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<UsageQuery>,
) -> impl IntoResponse {
    match usage::list_usage(&state.pool, &query).await {
        Ok(records) => Json(ApiResponse::success(records)),
        Err(e) => {
            error!("List usage failed: {}", e);
            Json(ApiResponse::error(500, "查询使用记录失败".to_string()))
        }
    }
}

pub async fn end_usage_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match usage::end_session(&state.pool, &id).await {
        Ok(record) => {
            trigger_checkin_if_needed(&state, &record.device_id).await;
            Json(ApiResponse::success(record))
        }
        Err(e) => {
            error!("End usage session failed: {}", e);
            Json(ApiResponse::error(400, e.to_string()))
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct CloseUsageRequest {
    pub device_id: String,
    pub reason: Option<String>,
}

pub async fn close_usage_handler(
    State(state): State<AppState>,
    Json(req): Json<CloseUsageRequest>,
) -> impl IntoResponse {
    if req.device_id.trim().is_empty() {
        return Json(ApiResponse::error(400, "缺少 device_id".to_string()));
    }
    match usage::close_open_session(&state.pool, &req.device_id).await {
        Ok(closed_id) => {
            if req.reason.as_deref() == Some("teacher_unlock") {
                let log_id = uuid::Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    r#"INSERT INTO operation_logs (id, log_type, operator, target_id, content, extra_payload, created_at)
                       VALUES (?, 'teacher_unlock', 'teacher', ?, '教师超级密码解锁签到准入', '{}', datetime('now'))"#
                )
                .bind(&log_id)
                .bind(&req.device_id)
                .execute(&state.pool)
                .await;
            } else {
                trigger_checkin_if_needed(&state, &req.device_id).await;
            }
            Json(ApiResponse::success(serde_json::json!({
                "closed": closed_id.is_some(),
                "sessionId": closed_id,
            })))
        }
        Err(e) => {
            error!("Close usage session failed: {}", e);
            Json(ApiResponse::error(500, "结束使用会话失败".to_string()))
        }
    }
}

async fn trigger_checkin_if_needed(state: &AppState, device_id: &str) {
    let mode: String = sqlx::query_scalar("SELECT current_mode FROM student_devices WHERE id = ?")
        .bind(device_id)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    if requires_checkin(&mode) {
        dispatch_checkin_trigger(state, &[device_id.to_string()]).await;
    }
}

pub async fn export_usage_handler(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<UsageQuery>,
) -> impl IntoResponse {
    match generate_usage_csv(&state.pool, &query).await {
        Ok(data) => {
            let headers = [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"usage.csv\""),
            ];
            (headers, data).into_response()
        }
        Err(e) => {
            error!("Export usage failed: {}", e);
            Json(ApiResponse::<()>::error(500, format!("导出失败: {}", e))).into_response()
        }
    }
}

fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn format_duration(seconds: Option<i64>) -> String {
    match seconds {
        Some(s) if s >= 0 => {
            let h = s / 3600;
            let m = (s % 3600) / 60;
            let sec = s % 60;
            format!("{:02}:{:02}:{:02}", h, m, sec)
        }
        _ => String::new(),
    }
}

async fn generate_usage_csv(pool: &sqlx::SqlitePool, query: &UsageQuery) -> anyhow::Result<Vec<u8>> {
    let records: Vec<UsageRecord> = usage::export_rows(pool, query).await?;
    let mut csv = String::from("\u{feff}设备,班级,座位号,使用人,开始时间,结束时间,使用时长,检查结果\n");
    for r in records {
        let inspection = if r.inspection_ok {
            "正常".to_string()
        } else {
            r.inspection_summary.clone().unwrap_or_else(|| "异常".to_string())
        };
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            csv_cell(r.device_name.as_deref().unwrap_or(&r.device_id)),
            csv_cell(r.class_name.as_deref().unwrap_or("")),
            csv_cell(r.seat_no.as_deref().unwrap_or("")),
            csv_cell(&r.user_name),
            csv_cell(&r.start_time),
            csv_cell(r.end_time.as_deref().unwrap_or("")),
            csv_cell(&format_duration(r.duration_seconds)),
            csv_cell(&inspection),
        ));
    }
    Ok(csv.into_bytes())
}
