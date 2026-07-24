use anyhow::Result;
use serde::Serialize;
use sysinfo::System;
use tracing::info;

use crate::config::Config;

#[derive(Debug, Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct GpuInfo {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct HardwareSnapshot {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub total_memory_bytes: u64,
    pub disk_info: String,
    pub mac_addresses: String,
    pub gpu_info: String,
    pub os_version: String,
    pub hostname: String,
}

pub fn collect() -> Result<HardwareSnapshot> {
    let mut system = System::new_all();
    system.refresh_all();

    let cpu_model = system
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let cpu_cores = system.physical_core_count().unwrap_or(1) as u32;

    let total_memory_bytes = system.total_memory();

    let disks: Vec<DiskInfo> = sysinfo::Disks::new_with_refreshed_list()
        .iter()
        .map(|d| DiskInfo {
            name: d.name().to_string_lossy().to_string(),
            total_bytes: d.total_space(),
            free_bytes: d.available_space(),
        })
        .collect();

    let disk_info = serde_json::to_string(&disks).unwrap_or_default();

    let mac_addresses: Vec<String> = mac_address::get_mac_address()
        .ok()
        .flatten()
        .map(|ma| ma.to_string())
        .into_iter()
        .collect();
    let mac_addresses = serde_json::to_string(&mac_addresses).unwrap_or_default();

    let gpu_info = detect_gpus();

    let os_version = format!("{} {}", sysinfo::System::name().unwrap_or_default(), sysinfo::System::os_version().unwrap_or_default());

    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    Ok(HardwareSnapshot {
        cpu_model,
        cpu_cores,
        total_memory_bytes,
        disk_info,
        mac_addresses,
        gpu_info,
        os_version,
        hostname,
    })
}

fn detect_gpus() -> String {
    #[cfg(target_os = "windows")]
    {
        match std::process::Command::new("wmic")
            .args(["path", "win32_videocontroller", "get", "name"])
            .output()
        {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout);
                let gpus: Vec<GpuInfo> = text
                    .lines()
                    .skip(1)
                    .filter_map(|line| {
                        let name = line.trim().to_string();
                        if name.is_empty() || name.eq_ignore_ascii_case("Name") {
                            None
                        } else {
                            Some(GpuInfo { name })
                        }
                    })
                    .collect();
                serde_json::to_string(&gpus).unwrap_or_else(|_| "[]".to_string())
            }
            Err(_) => "[]".to_string(),
        }
    }
    #[cfg(target_os = "linux")]
    {
        match std::process::Command::new("sh")
            .arg("-c")
            .arg("lspci | grep -i vga || true")
            .output()
        {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout);
                let gpus: Vec<GpuInfo> = text
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        if line.is_empty() { None } else {
                            Some(GpuInfo { name: line.to_string() })
                        }
                    })
                    .collect();
                serde_json::to_string(&gpus).unwrap_or_else(|_| "[]".to_string())
            }
            Err(_) => "[]".to_string(),
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        "[]".to_string()
    }
}

pub async fn submit(config: &Config) -> Result<()> {
    let snapshot = collect()?;
    info!(
        "Hardware snapshot: cpu={}, cores={}, memory={}MB",
        snapshot.cpu_model,
        snapshot.cpu_cores,
        snapshot.total_memory_bytes / 1024 / 1024
    );

    let client = reqwest::Client::new();
    let url = format!("{}/api/hardware/snapshot", config.teacher_server_url);

    let body = serde_json::json!({
        "device_id": config.device_id.clone().unwrap_or_default(),
        "cpu_model": snapshot.cpu_model,
        "cpu_cores": snapshot.cpu_cores,
        "total_memory_bytes": snapshot.total_memory_bytes as i64,
        "disk_info": snapshot.disk_info,
        "mac_addresses": snapshot.mac_addresses,
        "gpu_info": snapshot.gpu_info,
        "os_version": snapshot.os_version,
        "hostname": snapshot.hostname,
        "peripherals": detect_peripherals(),
    });

    let resp = client.post(&url).json(&body).send().await?;
    let status = resp.status();
    if status.is_success() {
        info!("Hardware snapshot submitted successfully");
    } else {
        tracing::warn!("Hardware snapshot submit returned HTTP {}", status);
    }

    Ok(())
}

#[derive(Debug, Serialize, Clone)]
pub struct PeripheralInfo {
    pub device_type: String,
    pub name: String,
    pub connected: bool,
}

pub fn detect_peripherals() -> Vec<PeripheralInfo> {
    let mut peripherals = Vec::new();

    let has_keyboard = detect_device_type("keyboard");
    let has_mouse = detect_device_type("mouse");

    peripherals.push(PeripheralInfo {
        device_type: "keyboard".to_string(),
        name: "USB/PS2 Keyboard".to_string(),
        connected: has_keyboard,
    });
    peripherals.push(PeripheralInfo {
        device_type: "mouse".to_string(),
        name: "USB/PS2 Mouse".to_string(),
        connected: has_mouse,
    });

    peripherals
}

#[cfg(target_os = "windows")]
fn detect_device_type(device_type: &str) -> bool {
    let class = match device_type {
        "keyboard" => "Keyboard",
        "mouse" => "Mouse",
        _ => return false,
    };
    std::process::Command::new("wmic")
        .args(["path", "Win32_Keyboard", "get", "Name"])
        .output()
        .map(|o| {
            let text = String::from_utf8_lossy(&o.stdout);
            text.lines().filter(|l| !l.trim().is_empty() && !l.contains("Name")).count() > 0
        })
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn detect_device_type(device_type: &str) -> bool {
    let path = match device_type {
        "keyboard" => "/dev/input/by-path",
        "mouse" => "/dev/input/by-path",
        _ => return false,
    };
    std::fs::read_dir(path).map(|entries| entries.count() > 0).unwrap_or(false)
}
