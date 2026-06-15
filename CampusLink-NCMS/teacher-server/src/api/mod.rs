mod handlers;
mod attendance_handlers;
mod inspection_handlers;
mod photo_handlers;

use axum::{routing::{get, post}, Router};
use sqlx::SqlitePool;
use tokio::sync::broadcast;

use handlers::{register_device_handler, list_devices_handler, mode_switch_handler, ws_handler, AppState};
use attendance_handlers::{check_in_handler, statistics_handler, retroactive_handler, list_attendance_handler};
use inspection_handlers::{submit_inspection_handler, list_inspections_handler, list_alerts_handler, resolve_alert_handler};
use photo_handlers::{upload_photo_handler, get_photo_handler, list_photos_handler};

pub async fn create_app(pool: &SqlitePool) -> Router {
    let (ws_tx, _) = broadcast::channel(100);
    
    let state = AppState {
        pool: pool.clone(),
        ws_tx,
    };

    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/devices/register", post(register_device_handler))
        .route("/api/devices", post(list_devices_handler))
        .route("/api/devices/:id/mode", post(mode_switch_handler))
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
        .route("/api/photos", get(list_photos_handler));

    app.with_state(state)
}

async fn health_handler() -> &'static str {
    "ok"
}
