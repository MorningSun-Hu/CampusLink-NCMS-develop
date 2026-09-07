use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use tracing::{info, warn};

use crate::config::Config;

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

/// Performs an interactive check-in for modes that require user input.
///
/// Mode-based check-in rules:
/// - open: launch campus-checkin so the user can enter their name
/// - teaching: launch campus-checkin which strictly verifies the student
///   account (student_no + password) before checking in
/// - exam / locked: no check-in required, returns Ok immediately
pub async fn check_in_interactive(config: &Config) -> Result<()> {
    let mode = config.current_mode.as_str();
    if mode != "open" && mode != "teaching" {
        info!("Mode {} does not require check-in, skipping", mode);
        return Ok(());
    }

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));
    let checkin_exe = match &exe_dir {
        Some(dir) => dir.join("campus-checkin.exe"),
        None => std::path::PathBuf::from("campus-checkin.exe"),
    };

    if !checkin_exe.exists() {
        warn!("campus-checkin.exe not found at: {:?}, falling back to auto check-in", checkin_exe);
        return check_in_auto(config).await;
    }

    let server_url = config.teacher_server_url.clone();
    let device_id = config.device_id.clone().unwrap_or_default();

    info!("Launching campus-checkin (mode={}) at {:?}", mode, checkin_exe);
    let mut child = match Command::new(&checkin_exe)
        .arg(&server_url)
        .arg(&device_id)
        .arg(mode)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            warn!("Failed to start campus-checkin: {}, falling back to auto check-in", e);
            return check_in_auto(config).await;
        }
    };

    let status = tokio::task::spawn_blocking(move || child.wait())
        .await
        .map_err(|e| anyhow::anyhow!("Join error waiting for campus-checkin: {}", e))??;

    if status.success() {
        info!("Interactive check-in succeeded (campus-checkin exit 0)");
        Ok(())
    } else {
        warn!("Interactive check-in was not completed (exit {:?})", status.code());
        Ok(())
    }
}
