use anyhow::Result;
use std::process::Command;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "campus_guard=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("CampusGuard starting...");

    loop {
        // 检查 agent-core 是否在运行
        let is_running = check_process_running("agent-core");

        if !is_running {
            warn!("agent-core is not running, attempting to restart...");
            match start_agent_core() {
                Ok(_) => info!("agent-core restarted successfully"),
                Err(e) => error!("Failed to restart agent-core: {}", e),
            }
        } else {
            info!("agent-core is running normally");
        }

        // 每分钟检查一次
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

fn check_process_running(process_name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tasklist")
            .args(&["/FI", &format!("IMAGENAME eq {}.exe", process_name)])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            return stdout.contains(&format!("{}.exe", process_name));
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("pgrep")
            .arg(process_name)
            .output();

        if let Ok(out) = output {
            return out.status.success();
        }
    }

    false
}

fn start_agent_core() -> Result<()> {
    Command::new("./agent-core")
        .spawn()?;
    Ok(())
}
