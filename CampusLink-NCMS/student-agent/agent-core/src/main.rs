mod config;
mod crypto;
mod register;
mod heartbeat;
mod mode;
mod websocket;
mod command_handler;
mod attendance;
mod inspection;
mod student_auth;
mod hardware;
mod process_guard;
mod discovery;

use anyhow::Result;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use register::collect_and_register;
use websocket::HeartbeatLoop;

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

    // Try UDP discovery if teacher_server_url is not configured
    if config.teacher_server_url.is_empty() || config.teacher_server_url == "http://localhost:8080" {
        info!("Attempting UDP discovery...");
        match discovery::discover(5).await {
            Ok(url) => {
                config.teacher_server_url = url;
                config.save()?;
                info!("Teacher server discovered: {}", config.teacher_server_url);
            }
            Err(e) => {
                warn!("Discovery failed: {}. Using default URL.", e);
            }
        }
    }

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

    if let Err(e) = hardware::submit(&config).await {
        warn!("Hardware snapshot submit failed: {}", e);
    }

    if let Err(e) = attendance::check_in_auto(&config).await {
        warn!("Auto check-in failed: {}", e);
    }

    let policies = process_guard::sync_policies(&config).await;
    let _alerts = process_guard::check_and_restart(&policies);

    info!("Starting WebSocket heartbeat loop...");
    let mut heartbeat_loop = HeartbeatLoop::new(config);

    heartbeat_loop.run().await?;
    Ok(())
}
