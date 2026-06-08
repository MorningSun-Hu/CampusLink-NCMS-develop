mod config;
mod register;
mod heartbeat;
mod mode;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use register::collect_and_register;
use mode::handle_mode_switch;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "agent_core=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("CampusLink Agent Core starting...");

    let mut config = Config::load().unwrap_or_else(|_| Config::default());
    info!("Configuration loaded");

    if config.device_id.is_none() {
        info!("Device not registered, starting registration...");
        match collect_and_register(&mut config).await {
            Ok(_) => info!("Registration completed"),
            Err(e) => {
                error!("Registration failed: {}", e);
                return Err(e);
            }
        }
    } else {
        info!("Device already registered: {}", config.device_id.as_ref().unwrap());
    }

    info!("Agent Core ready, current mode: {}", config.current_mode);
    
    // P4 占位：实际应该启动 WebSocket 心跳循环并处理模式切换消息
    // 简化处理，仅打印状态
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        info!("Agent running normally, mode: {}", config.current_mode);
    }
}
