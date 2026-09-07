pub mod handlers;
mod attendance_handlers;
mod inspection_handlers;
mod photo_handlers;
mod hardware_handlers;
mod log_handlers;
mod process_guard_handlers;
mod dashboard_handlers;
mod student_handlers;
mod settings_handlers;
mod auth_handlers;
mod whitelist_handlers;
mod repair_handlers;
mod network_account_handlers;

use axum::{routing::{get, post, delete, put}, Router, middleware};
use axum::response::Html;
use sqlx::SqlitePool;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

use handlers::{register_device_handler, list_devices_handler, mode_switch_handler, ws_handler, lock_screen_handler, unlock_handler, AppState};
use attendance_handlers::{check_in_handler, statistics_handler, retroactive_handler, list_attendance_handler, export_attendance_handler};
use inspection_handlers::{submit_inspection_handler, list_inspections_handler, list_alerts_handler, resolve_alert_handler, export_inspection_handler};
use photo_handlers::{upload_photo_handler, get_photo_handler, list_photos_handler};
use hardware_handlers::{submit_snapshot_handler, get_snapshot_handler, list_changes_handler};
use log_handlers::{query_logs_handler, export_logs_handler};
use process_guard_handlers::{create_policy_handler, update_policy_handler, list_policies_handler, delete_policy_handler};
use dashboard_handlers::dashboard_overview_handler;
use student_handlers::{create_student_handler, list_students_handler, get_student_handler, update_student_handler, delete_student_handler, import_students_handler, export_students_handler};
use settings_handlers::{get_lock_password_handler, update_lock_password_handler, get_schedule_handler, update_schedule_handler};
use auth_handlers::{login_handler, student_login_handler};
use whitelist_handlers::{list_whitelist_handler, import_whitelist_handler, approve_whitelist_handler, delete_whitelist_handler};
use repair_handlers::{create_repair_handler, list_repairs_handler, update_repair_handler, delete_repair_handler};
use network_account_handlers::{list_accounts_handler, create_account_handler, update_account_handler, delete_account_handler};
use crate::middleware::auth::auth_middleware;

pub async fn create_app(pool: &SqlitePool, database_url: String) -> Router {
    let (ws_tx, _) = broadcast::channel(100);

    // Generate or load JWT secret
    let jwt_secret = crate::infrastructure::device_repository::get_config_value_string(pool, "jwt_secret")
        .await
        .ok()
        .flatten();

    let jwt_secret = match jwt_secret {
        Some(s) => s,
        None => {
            let secret = uuid::Uuid::new_v4().to_string();
            // Persist to system_configs so login() and middleware use the same key
            let id = uuid::Uuid::new_v4().to_string();
            let _ = sqlx::query(
                r#"INSERT INTO system_configs (id, config_key, config_value, scope, description, updated_at)
                   VALUES (?, 'jwt_secret', ?, 'security', 'JWT signing secret', datetime('now'))"#
            )
            .bind(&id)
            .bind(&secret)
            .execute(pool)
            .await;
            secret
        }
    };

    // Init default admin user
    let _ = crate::domain::auth::init_default_admin(pool).await;

    let state = AppState {
        pool: pool.clone(),
        ws_tx: ws_tx.clone(),
        jwt_secret,
    };

    // Start mode switch scheduler
    crate::scheduler::start(pool.clone(), ws_tx);

    // Start database backup
    crate::infrastructure::backup::start(database_url);

    // Start UDP discovery listener
    crate::discovery::start(8080);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/student-login", post(student_login_handler))
        .route("/api/devices/register", post(register_device_handler))
        // Student agent report endpoints: authenticated by device_id (registered device)
        .route("/api/attendance/check-in", post(check_in_handler))
        .route("/api/inspection/submit", post(submit_inspection_handler))
        .route("/api/hardware/snapshot", post(submit_snapshot_handler))
        .route("/api/policies/process-guard/sync", get(list_policies_handler))
        .route("/ws", get(ws_handler));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/api/devices", post(list_devices_handler))
        .route("/api/devices/:id/mode", post(mode_switch_handler))
        .route("/api/devices/:id/lock", post(lock_screen_handler))
        .route("/api/devices/:id/unlock", post(unlock_handler))
        .route("/api/attendance/statistics", get(statistics_handler))
        .route("/api/attendance/retroactive", post(retroactive_handler))
        .route("/api/attendance", get(list_attendance_handler))
        .route("/api/attendance/export", get(export_attendance_handler))
        .route("/api/inspection", get(list_inspections_handler))
        .route("/api/inspection/export", get(export_inspection_handler))
        .route("/api/alerts", get(list_alerts_handler))
        .route("/api/alerts/:id/resolve", post(resolve_alert_handler))
        .route("/api/photos/upload", post(upload_photo_handler))
        .route("/api/photos/:id", get(get_photo_handler))
        .route("/api/photos", get(list_photos_handler))
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
        .route("/api/settings/schedule", get(get_schedule_handler))
        .route("/api/settings/schedule", put(update_schedule_handler))
        .route("/api/devices/whitelist", get(list_whitelist_handler))
        .route("/api/devices/whitelist/import", post(import_whitelist_handler))
        .route("/api/devices/whitelist/:id/approve", post(approve_whitelist_handler))
        .route("/api/devices/whitelist/:id", delete(delete_whitelist_handler))
        .route("/api/repair-orders", post(create_repair_handler))
        .route("/api/repair-orders", get(list_repairs_handler))
        .route("/api/repair-orders/:id", put(update_repair_handler))
        .route("/api/repair-orders/:id", delete(delete_repair_handler))
        .route("/api/network-accounts", get(list_accounts_handler))
        .route("/api/network-accounts", post(create_account_handler))
        .route("/api/network-accounts/:id", put(update_account_handler))
        .route("/api/network-accounts/:id", delete(delete_account_handler))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .nest_service("/assets", ServeDir::new("static/assets"))
        .fallback(serve_spa_index)
        .with_state(state);

    app
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
