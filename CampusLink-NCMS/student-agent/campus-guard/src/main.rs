use anyhow::Result;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static EXIT_COUNTER: AtomicU32 = AtomicU32::new(0);
const MAX_EXIT_COUNT: u32 = 10;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "campus_guard=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("CampusGuard starting - process supervisor");
    
    let guard_processes = vec![
        GuardedProcess {
            name: "agent-core",
            executable: "agent-core",
            restart_window_secs: 60,
        },
        GuardedProcess {
            name: "campus-lock",
            executable: "campus-lock",
            restart_window_secs: 60,
        },
    ];

    loop {
        for proc in &guard_processes {
            let is_running = check_process_running(proc.name);
            if !is_running {
                let count = EXIT_COUNTER.load(Ordering::Relaxed);
                if count >= MAX_EXIT_COUNT {
                    error!(
                        "{} exit count exceeded max ({}), stopping auto-restart",
                        proc.name, MAX_EXIT_COUNT
                    );
                    continue;
                }

                warn!("{} is not running, attempting restart...", proc.name);
                match start_process(proc.executable) {
                    Ok(_) => {
                        info!("{} restarted successfully", proc.name);
                        EXIT_COUNTER.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => error!("Failed to restart {}: {}", proc.name, e),
                }
            }
        }

        info!(
            "Guard check complete: lagent_core={}, campus_lock={}, exit_count={}",
            check_process_running("agent-core"),
            check_process_running("campus-lock"),
            EXIT_COUNTER.load(Ordering::Relaxed),
        );

        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}

struct GuardedProcess {
    name: &'static str,
    executable: &'static str,
    restart_window_secs: u64,
}

fn check_process_running(process_name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let filter = format!("IMAGENAME eq {}.exe", process_name);
        let output = Command::new("tasklist")
            .args(["/FI", &filter, "/FO", "CSV", "/NH"])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            return stdout.to_lowercase().contains(&process_name.to_lowercase());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("pgrep")
            .arg("-x")
            .arg(process_name)
            .output();

        if let Ok(out) = output {
            return out.status.success();
        }
    }

    false
}

fn start_process(executable: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &format!("{}.exe", executable)])
            .spawn()?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let exe = if cfg!(windows) {
            format!("{}.exe", executable)
        } else {
            format!("./{}", executable)
        };
        Command::new("sh")
            .args(["-c", &exe])
            .spawn()?;
    }

    Ok(())
}
