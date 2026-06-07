use axum::{routing::get, Router};
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::app::Config;

pub async fn create_app(config: &Config, pool: &SqlitePool) -> Router {
    let app = Router::new()
        .route("/api/health", get(health_handler))
        .layer(TraceLayer::new_for_http());

    info!("Application created successfully");
    app.with_state(pool.clone())
}

async fn health_handler() -> &'static str {
    "ok"
}
