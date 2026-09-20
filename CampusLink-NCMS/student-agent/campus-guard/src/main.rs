use anyhow::Result;
use std::process::Command;
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_runtime() -> Result<()> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    std::env::set_current_dir(&exe_dir)?;
    std::fs::create_dir_all(exe_dir.join("config"))?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_runtime()?;

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
            name: "student",
            executable: "student",
            restart_window_secs: 60,
        },
    ];

    loop {
        for proc in &guard_processes {
            let is_running = check_process_running(proc.name);
            if !is_running {
                warn!("{} is not running, attempting restart...", proc.name);
                match start_process(proc.executable) {
                    Ok(_) => {
                        info!("{} restarted successfully", proc.name);
                    }
                    Err(e) => error!("Failed to restart {}: {}", proc.name, e),
                }
            }
        }

        info!(
            "Guard check complete: student={}",
            check_process_running("student"),
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
            .creation_flags(0x08000000)
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
    let exe_dir = std::env::current_exe()?
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    #[cfg(target_os = "windows")]
    {
        let path = exe_dir.join(format!("{}.exe", executable));
        Command::new("cmd")
            .current_dir(&exe_dir)
            .args(["/C", "start", "", &path.to_string_lossy()])
            .creation_flags(0x08000000)
            .spawn()?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let path = exe_dir.join(executable);
        Command::new("sh")
            .current_dir(&exe_dir)
            .args(["-c", &path.to_string_lossy()])
            .spawn()?;
    }

    Ok(())
}
