mod handlers;

use axum::{routing::{get, post}, Router};
use sqlx::SqlitePool;
use tokio::sync::broadcast;

use handlers::{register_device_handler, list_devices_handler, mode_switch_handler, ws_handler, AppState};

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
        .route("/ws", get(ws_handler));

    app.with_state(state)
}

async fn health_handler() -> &'static str {
    "ok"
}
