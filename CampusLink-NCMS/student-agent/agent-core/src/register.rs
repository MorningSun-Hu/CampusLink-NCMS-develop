use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_code: String,
    pub machine_fingerprint: String,
    pub hostname: String,
    pub ip_address: String,
    pub mac_address: String,
    pub agent_version: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceResponse {
    pub device_id: String,
    pub teacher_fingerprint: String,
    pub initial_mode: String,
    pub heartbeat_interval_seconds: u32,
    pub session_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

pub async fn register_device(
    client: &Client,
    base_url: &str,
    request: RegisterDeviceRequest,
) -> Result<RegisterDeviceResponse> {
    let url = format!("{}/api/devices/register", base_url);
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?
        .json::<ApiResponse<RegisterDeviceResponse>>()
        .await?;

    if let Some(data) = response.data {
        info!("Device registered successfully: {}", data.device_id);
        Ok(data)
    } else {
        anyhow::bail!("Register failed: {}", response.message)
    }
}

pub fn generate_machine_fingerprint() -> String {
    // 简单实现，生产环境需要更复杂的指纹生成
    format!("fingerprint-{}", uuid::Uuid::new_v4())
}

fn get_hostname() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

pub async fn collect_and_register(config: &mut Config) -> Result<()> {
    let client = Client::new();
    
    // 采集本机信息
    let hostname = get_hostname();
    let ip_address = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    let mac_address = mac_address::get_mac_address()
        .map_err(|e| anyhow::anyhow!("Failed to get MAC address: {}", e))?
        .map(|m| m.to_string())
        .unwrap_or_else(|| "00:00:00:00:00:00".to_string());
    let machine_fingerprint = generate_machine_fingerprint();
    let device_code = format!("DEV-{}", hostname[..8].to_uppercase());

    info!("Collecting machine info: hostname={}, ip={}, mac={}", hostname, ip_address, mac_address);

    let request = RegisterDeviceRequest {
        device_code,
        machine_fingerprint,
        hostname,
        ip_address,
        mac_address,
        agent_version: "0.1.0".to_string(),
    };

    let response = register_device(&client, &config.teacher_server_url, request).await?;

    config.device_id = Some(response.device_id);
    config.teacher_fingerprint = Some(response.teacher_fingerprint);
    config.current_mode = response.initial_mode.clone();
    config.heartbeat_interval_seconds = response.heartbeat_interval_seconds as u64;
    config.session_key = Some(response.session_key);
    config.save()?;

    info!("Registration completed, device_id: {}", config.device_id.as_ref().unwrap());

    Ok(())
}
