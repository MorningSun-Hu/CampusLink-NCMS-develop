mod handlers;
mod attendance_handlers;
mod inspection_handlers;
mod photo_handlers;
mod hardware_handlers;
mod log_handlers;
mod process_guard_handlers;
mod dashboard_handlers;
mod student_handlers;
mod settings_handlers;

use axum::{routing::{get, post, delete, put}, Router};
use axum::response::Html;
use sqlx::SqlitePool;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

use handlers::{register_device_handler, list_devices_handler, mode_switch_handler, ws_handler, lock_screen_handler, unlock_handler, AppState};
use attendance_handlers::{check_in_handler, statistics_handler, retroactive_handler, list_attendance_handler};
use inspection_handlers::{submit_inspection_handler, list_inspections_handler, list_alerts_handler, resolve_alert_handler};
use photo_handlers::{upload_photo_handler, get_photo_handler, list_photos_handler};
use hardware_handlers::{submit_snapshot_handler, get_snapshot_handler, list_changes_handler};
use log_handlers::{query_logs_handler, export_logs_handler};
use process_guard_handlers::{create_policy_handler, update_policy_handler, list_policies_handler, delete_policy_handler};
use dashboard_handlers::dashboard_overview_handler;
use student_handlers::{create_student_handler, list_students_handler, get_student_handler, update_student_handler, delete_student_handler, import_students_handler, export_students_handler};
use settings_handlers::{get_lock_password_handler, update_lock_password_handler};

pub async fn create_app(pool: &SqlitePool) -> Router {
    let (ws_tx, _) = broadcast::channel(100);
    
    let state = AppState {
        pool: pool.clone(),
        ws_tx: ws_tx.clone(),
    };

    // Start mode switch scheduler
    crate::scheduler::start(pool.clone(), ws_tx);

    // Start database backup
    crate::infrastructure::backup::start();

    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/devices/register", post(register_device_handler))
        .route("/api/devices", post(list_devices_handler))
        .route("/api/devices/:id/mode", post(mode_switch_handler))
        .route("/api/devices/:id/lock", post(lock_screen_handler))
        .route("/api/devices/:id/unlock", post(unlock_handler))
        .route("/ws", get(ws_handler))
        .route("/api/attendance/check-in", post(check_in_handler))
        .route("/api/attendance/statistics", get(statistics_handler))
        .route("/api/attendance/retroactive", post(retroactive_handler))
        .route("/api/attendance", get(list_attendance_handler))
        .route("/api/inspection/submit", post(submit_inspection_handler))
        .route("/api/inspection", get(list_inspections_handler))
        .route("/api/alerts", get(list_alerts_handler))
        .route("/api/alerts/:id/resolve", post(resolve_alert_handler))
        .route("/api/photos/upload", post(upload_photo_handler))
        .route("/api/photos/:id", get(get_photo_handler))
        .route("/api/photos", get(list_photos_handler))
        .route("/api/hardware/snapshot", post(submit_snapshot_handler))
        .route("/api/hardware/snapshot", get(get_snapshot_handler))
        .route("/api/hardware/changes", get(list_changes_handler))
        .route("/api/logs", get(query_logs_handler))
        .route("/api/logs/export", get(export_logs_handler))
        .route("/api/policies/process-guard", post(create_policy_handler))
        .route("/api/policies/process-guard/:id", post(update_policy_handler))
        .route("/api/policies/process-guard", get(list_policies_handler))
        .route("/api/policies/process-guard/:id", delete(delete_policy_handler))
        .route("/api/dashboard/overview", get(dashboard_overview_handler))
        .route("/api/students", post(create_student_handler))
        .route("/api/students", get(list_students_handler))
        .route("/api/students/import", post(import_students_handler))
        .route("/api/students/export", get(export_students_handler))
        .route("/api/students/:id", get(get_student_handler))
        .route("/api/students/:id", put(update_student_handler))
        .route("/api/students/:id", delete(delete_student_handler))
        .route("/api/settings/lock-password", get(get_lock_password_handler))
        .route("/api/settings/lock-password", put(update_lock_password_handler))
        .nest_service("/assets", ServeDir::new("static/assets"))
        .fallback(serve_spa_index);

    app.with_state(state)
}

async fn serve_spa_index() -> Html<String> {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(html) => Html(html),
        Err(_) => Html("<h1>Frontend not found</h1><p>Place built frontend in static/ directory</p>".into()),
    }
}

async fn health_handler() -> &'static str {
    "ok"
}
