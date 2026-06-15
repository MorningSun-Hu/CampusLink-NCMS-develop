use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

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
