use anyhow::Result;
use tracing::{info, error};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use tokio::sync::mpsc;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModeType {
    Open = 0,
    Teaching = 1,
    Exam = 3,
    Locked = 4,
}

impl ToString for ModeType {
    fn to_string(&self) -> String {
        match self {
            ModeType::Open => "open".to_string(),
            ModeType::Teaching => "teaching".to_string(),
            ModeType::Exam => "exam".to_string(),
            ModeType::Locked => "locked".to_string(),
        }
    }
}

/// 处理模式切换命令
pub async fn handle_mode_switch(
    config: &mut Config,
    target_mode: &str,
    unlock_tx: &Option<mpsc::UnboundedSender<String>>,
) -> Result<()> {
    info!("Switching mode from {} to {}", config.current_mode, target_mode);

    crate::attendance::stop_admission_monitor();

    let parsed_mode = parse_mode(target_mode);

    match parsed_mode {
        ModeType::Open => {
            info!("Switching to Open mode - disabling locks, starting admission");
            disable_all_locks().await?;
            config.is_locked = false;
            config.lock_pid = None;
        }
        ModeType::Teaching => {
            info!("Switching to Teaching mode - starting admission");
            disable_all_locks().await?;
            config.is_locked = false;
            config.lock_pid = None;
        }
        ModeType::Exam => {
            info!("Switching to Exam mode - enabling exam lockdown");
            enable_exam_mode(config, unlock_tx).await?;
        }
        ModeType::Locked => {
            info!("Switching to Locked mode - full lockdown");
            lock_completely(config, unlock_tx).await?;
        }
    }

    config.current_mode = target_mode.to_string();
    config.save()?;

    if matches!(parsed_mode, ModeType::Open | ModeType::Teaching) {
        crate::attendance::start_admission_monitor(config.clone());
    }

    info!("Mode switch completed: {}", target_mode);
    Ok(())
}

/// 开放模式：解除所有限制
async fn disable_all_locks() -> Result<()> {
    info!("Disabling all locks - Open mode");
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("taskkill")
            .args(["/IM", "campus-lock.exe", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("pkill")
            .arg("campus-lock")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
    Ok(())
}

/// 考试模式：全屏锁定，禁止切换应用
async fn enable_exam_mode(config: &mut Config, unlock_tx: &Option<mpsc::UnboundedSender<String>>) -> Result<()> {
    info!("Enabling Exam mode - exam lockdown active");
    spawn_lock_screen(config, unlock_tx).await?;
    Ok(())
}

/// 锁定模式：完全锁定
async fn lock_completely(config: &mut Config, unlock_tx: &Option<mpsc::UnboundedSender<String>>) -> Result<()> {
    info!("Enabling Locked mode - full lockdown");
    spawn_lock_screen(config, unlock_tx).await?;
    Ok(())
}

async fn spawn_lock_screen(
    config: &mut Config,
    unlock_tx: &Option<mpsc::UnboundedSender<String>>,
) -> Result<()> {
    if config.is_locked {
        info!("Screen already locked, skipping");
        return Ok(());
    }

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));
    let lock_exe = match &exe_dir {
        Some(dir) => dir.join("campus-lock.exe"),
        None => std::path::PathBuf::from("campus-lock.exe"),
    };

    if !lock_exe.exists() {
        error!("campus-lock.exe not found at: {:?}", lock_exe);
        return Ok(());
    }

    let password = config.lock_password.as_deref().unwrap_or("admin123");
    info!("Launching campus-lock: {:?}", lock_exe);

    match Command::new(&lock_exe)
        .arg(password)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            let pid = child.id();
            info!("campus-lock started, PID={}", pid);
            config.lock_pid = Some(pid);
            config.is_locked = true;

            // 在后台监控 campus-lock 进程，退出时通知心跳循环
            if let Some(tx) = unlock_tx {
                let tx = tx.clone();
                let device_id = config.device_id.clone().unwrap_or_default();
                tokio::task::spawn_blocking(move || {
                    let _ = child.wait();
                    info!("campus-lock (PID={}) exited, user unlocked screen", pid);
                    let msg = serde_json::json!({
                        "type": "unlock",
                        "device_id": device_id,
                        "timestamp": chrono::Utc::now().timestamp(),
                    });
                    let _ = tx.send(msg.to_string());
                });
                info!("campus-lock exit monitor started");
            }
        }
        Err(e) => {
            error!("Failed to spawn campus-lock: {}", e);
        }
    }

    Ok(())
}

pub fn parse_mode(mode_str: &str) -> ModeType {
    match mode_str {
        "teaching" => ModeType::Teaching,
        "exam" => ModeType::Exam,
        "locked" => ModeType::Locked,
        _ => ModeType::Open,
    }
}
