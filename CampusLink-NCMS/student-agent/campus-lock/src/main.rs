use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "campus_lock=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("CampusLock starting (placeholder)...");
    
    // P3 阶段占位实现，P4 将实现锁屏功能
    info!("CampusLock placeholder - waiting for P4 implementation");

    Ok(())
}
