mod app;
mod api;
mod domain;
mod infrastructure;
mod scheduler;
mod middleware;
mod discovery;
mod crypto_util;
mod proto;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app::Config;
use api::create_app;
use infrastructure::create_sqlite_pool;

#[tokio::main]
async fn main() -> Result<()> {
    // File appender with daily rotation, keep 14 days
    let file_appender = tracing_appender::rolling::daily("logs", "teacher-server");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "teacher_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_writer(file_appender))
        .init();

    let config = Config::load()?;
    info!("Configuration loaded successfully");

    let pool = create_sqlite_pool(&config.database.url).await?;
    info!("Database connection pool created");

    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Database migrations applied");

    let app = create_app(&pool).await;

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
