use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use campus_lock::LockManager;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "campus_lock=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("CampusLock starting...");

    let mut lock_manager = LockManager::new();
    
    // 测试锁屏
    info!("Testing lock...");
    lock_manager.lock().await?;
    
    info!("Lock test completed");
    
    // 等待 5 秒后解锁
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    info!("Testing unlock...");
    lock_manager.unlock().await?;
    
    info!("Unlock test completed");

    Ok(())
}
