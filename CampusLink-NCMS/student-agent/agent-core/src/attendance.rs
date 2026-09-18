use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tracing::{info, warn, error};

use crate::config::Config;

static ADMISSION_GEN: AtomicU64 = AtomicU64::new(0);

const EXIT_OK: i32 = 0;
const EXIT_TEACHER: i32 = 10;

#[derive(Debug, Serialize)]
pub struct CheckInRequest {
    pub device_id: String,
    pub student_id: Option<String>,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CheckInResponse {
    pub record_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

pub async fn check_in(config: &Config) -> Result<CheckInResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/attendance/check-in", config.teacher_server_url);

    let request = CheckInRequest {
        device_id: config.device_id.clone().unwrap_or_default(),
        student_id: config.student_id.clone(),
        timestamp: Some(chrono::Utc::now().timestamp() as u64),
    };

    info!("Sending check-in request to {}", url);
    let resp = client.post(&url).json(&request).send().await?;
    let api_resp: ApiResponse<CheckInResponse> = resp.json().await?;

    if api_resp.code == 0 {
        let data = api_resp.data.unwrap();
        info!("Check-in successful: record_id={}", data.record_id);
        Ok(data)
    } else {
        Err(anyhow::anyhow!("Check-in failed: {}", api_resp.message))
    }
}

pub async fn check_in_auto(config: &Config) -> Result<()> {
    match check_in(config).await {
        Ok(resp) => {
            info!("Auto check-in completed: record_id={}, status={}", resp.record_id, resp.status);
            Ok(())
        }
        Err(e) => {
            tracing::warn!("Auto check-in failed (non-fatal): {}", e);
            Ok(())
        }
    }
}

fn resolve_checkin_exe() -> PathBuf {
    let dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    let win = dir.join("campus-checkin.exe");
    if win.exists() {
        return win;
    }
    dir.join("campus-checkin")
}

fn kill_checkin_process() {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = Command::new("taskkill")
            .args(["/IM", "campus-checkin.exe", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("pkill")
            .arg("-f")
            .arg("campus-checkin")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
}

pub fn stop_admission_monitor() {
    ADMISSION_GEN.fetch_add(1, Ordering::SeqCst);
    kill_checkin_process();
}

/// Launch (or restart) the fullscreen admission UI and keep relaunching until
/// check-in completes, the teacher unlocks, or a newer monitor generation starts.
pub fn start_admission_monitor(config: Config) {
    let mode = config.current_mode.clone();
    if mode != "open" && mode != "teaching" {
        info!("Mode {} does not require admission, skipping", mode);
        return;
    }

    let gen = ADMISSION_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    kill_checkin_process();

    tokio::spawn(async move {
        info!("Admission monitor started (gen={}, mode={})", gen, mode);
        loop {
            if ADMISSION_GEN.load(Ordering::SeqCst) != gen {
                info!("Admission monitor gen={} cancelled", gen);
                break;
            }

            let exe = resolve_checkin_exe();
            if !exe.exists() {
                warn!("campus-checkin not found at {:?}, retrying", exe);
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }

            let server_url = config.teacher_server_url.clone();
            let device_id = config.device_id.clone().unwrap_or_default();
            let lock_password = config.lock_password.clone().unwrap_or_else(|| "admin123".to_string());
            let run_mode = config.current_mode.clone();
            let exe_clone = exe.clone();

            info!("Launching campus-checkin (mode={}) at {:?}", run_mode, exe_clone);
            let wait_result = tokio::task::spawn_blocking(move || {
                let mut child = Command::new(&exe_clone)
                    .arg(&server_url)
                    .arg(&device_id)
                    .arg(&run_mode)
                    .arg(&lock_password)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()?;
                child.wait()
            })
            .await;

            if ADMISSION_GEN.load(Ordering::SeqCst) != gen {
                info!("Admission monitor gen={} cancelled after process exit", gen);
                break;
            }

            match wait_result {
                Ok(Ok(status)) => {
                    let code = status.code();
                    if code == Some(EXIT_OK) {
                        info!("Admission completed (exit 0)");
                        break;
                    }
                    if code == Some(EXIT_TEACHER) {
                        info!("Admission unlocked by teacher super password (exit 10)");
                        break;
                    }
                    warn!("campus-checkin exited unexpectedly ({:?}), relaunching", code);
                }
                Ok(Err(e)) => {
                    error!("Failed to start campus-checkin: {}", e);
                }
                Err(e) => {
                    error!("Join error waiting for campus-checkin: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        info!("Admission monitor gen={} stopped", gen);
    });
}

/// Backward-compatible wrapper used by existing call sites.
pub async fn check_in_interactive(config: &Config) -> Result<()> {
    start_admission_monitor(config.clone());
    Ok(())
}
