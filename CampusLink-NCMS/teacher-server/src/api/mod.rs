mod handlers;

use axum::{routing::{get, post}, Router};
use sqlx::SqlitePool;

use handlers::{register_device_handler, list_devices_handler, ws_handler};

pub async fn create_app(pool: &SqlitePool) -> Router {
    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/devices/register", post(register_device_handler))
        .route("/api/devices", post(list_devices_handler))
        .route("/ws", get(ws_handler));

    app.with_state(pool.clone())
}

async fn health_handler() -> &'static str {
    "ok"
}
