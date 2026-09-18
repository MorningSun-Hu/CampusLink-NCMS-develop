use serde::Deserialize;
use tracing::{info, warn};

use crate::config::Config;

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessGuardPolicy {
    pub id: String,
    pub device_id: Option<String>,
    pub process_name: String,
    pub check_interval_seconds: i64,
    pub max_restart_attempts: i64,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

pub async fn sync_policies(config: &Config) -> Vec<ProcessGuardPolicy> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/policies/process-guard/sync", config.teacher_server_url);

    let device_id = config.device_id.clone().unwrap_or_default();
    let full_url = format!("{}?device_id={}", url, device_id);

    match client.get(&full_url).send().await {
        Ok(resp) => match resp.json::<ApiResponse<Vec<ProcessGuardPolicy>>>().await {
            Ok(api_resp) if api_resp.code == 0 => {
                let policies = api_resp.data.unwrap_or_default();
                info!("Synced {} process guard policies", policies.len());
                policies
            }
            Ok(api_resp) => {
                warn!("Failed to sync policies: {}", api_resp.message);
                vec![]
            }
            Err(e) => {
                warn!("Failed to parse policies response: {}", e);
                vec![]
            }
        },
        Err(e) => {
            warn!("Failed to fetch policies: {}", e);
            vec![]
        }
    }
}

pub fn check_and_restart(policies: &[ProcessGuardPolicy]) -> Vec<ProcessAlert> {
    let mut alerts = Vec::new();
    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    for policy in policies {
        if !policy.enabled {
            continue;
        }

        let found = system
            .processes()
            .values()
            .any(|p| p.name().to_string_lossy().contains(&policy.process_name));

        if !found {
            info!(
                "Process '{}' not running, attempting restart",
                policy.process_name
            );

            match std::process::Command::new(&policy.process_name).spawn() {
                Ok(child) => {
                    info!("Process '{}' restarted with PID {}", policy.process_name, child.id());
                }
                Err(e) => {
                    warn!("Failed to restart process '{}': {}", policy.process_name, e);
                    alerts.push(ProcessAlert {
                        policy_id: policy.id.clone(),
                        process_name: policy.process_name.clone(),
                        alert_type: "process_restart_failed".to_string(),
                        message: format!("进程 {} 重启失败: {}", policy.process_name, e),
                    });
                }
            }
        }
    }

    alerts
}

pub async fn report_alerts(config: &Config, alerts: &[ProcessAlert]) {
    if alerts.is_empty() {
        return;
    }

    let client = reqwest::Client::new();

    for alert in alerts {
        let url = format!("{}/api/inspection/submit", config.teacher_server_url);
        let body = serde_json::json!({
            "device_id": config.device_id.clone().unwrap_or_default(),
            "inspection_type": "process_guard",
            "item_name": alert.process_name,
            "status": "abnormal",
            "description": alert.message,
        });

        match client.post(&url).json(&body).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("Alert reported: {}", alert.message);
                } else {
                    warn!("Failed to report alert, HTTP {}", resp.status());
                }
            }
            Err(e) => {
                warn!("Failed to send alert report: {}", e);
            }
        }
    }
}

#[derive(Debug)]
pub struct ProcessAlert {
    pub policy_id: String,
    pub process_name: String,
    pub alert_type: String,
    pub message: String,
}

#[cfg(target_os = "windows")]
fn is_process_running(name: &str) -> bool {
    let filter = format!("IMAGENAME eq {}.exe", name);
    if let Ok(output) = std::process::Command::new("tasklist")
        .args(["/FI", &filter, "/FO", "CSV", "/NH"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.to_lowercase().contains(&name.to_lowercase());
    }
    false
}

#[cfg(not(target_os = "windows"))]
fn is_process_running(name: &str) -> bool {
    std::process::Command::new("pgrep")
        .arg("-x")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn guard_campus_guard() -> Option<ProcessAlert> {
    if is_process_running("campus-guard") {
        return None;
    }

    warn!("campus-guard not running, attempting restart");

    #[cfg(target_os = "windows")]
    let result = {
        let dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let path = dir.join("campus-guard.exe");
        std::process::Command::new("cmd")
            .current_dir(&dir)
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()
    };

    #[cfg(not(target_os = "windows"))]
    let result = {
        let dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        std::process::Command::new(dir.join("campus-guard"))
            .current_dir(&dir)
            .spawn()
    };

    match result {
        Ok(child) => {
            info!("campus-guard restarted with PID {}", child.id());
            None
        }
        Err(e) => {
            let alert = ProcessAlert {
                policy_id: "builtin-campus-guard".to_string(),
                process_name: "campus-guard".to_string(),
                alert_type: "process_guard".to_string(),
                message: format!("campus-guard 重启失败: {}", e),
            };
            Some(alert)
        }
    }
}
