mod app;
mod api;
mod infrastructure;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app::Config;
use api::create_app;
use infrastructure::create_sqlite_pool;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "teacher_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = Config::load()?;
    info!("Configuration loaded successfully");

    // 创建数据库连接池
    let pool = create_sqlite_pool(&config.database.url).await?;
    info!("Database connection pool created");

    // 创建应用
    let app = create_app(&config, &pool).await;

    // 启动服务
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
