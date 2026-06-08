mod config;
mod register;
mod heartbeat;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use register::collect_and_register;
use heartbeat::start_heartbeat_loop;

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

    // 加载配置
    let mut config = Config::load().unwrap_or_else(|_| Config::default());
    info!("Configuration loaded");

    // 如果未注册，则执行注册
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

    // 启动心跳循环
    let ws_url = format!("{}/ws", config.teacher_server_url.replace("http", "ws"));
    let device_id = config.device_id.clone().unwrap();
    let interval = config.heartbeat_interval_seconds;

    info!("Starting heartbeat loop, interval: {}s", interval);

    // 这里简化处理，实际应该处理模式切换消息
    if let Err(e) = start_heartbeat_loop(ws_url, device_id, interval).await {
        error!("Heartbeat loop error: {}", e);
    }

    Ok(())
}
