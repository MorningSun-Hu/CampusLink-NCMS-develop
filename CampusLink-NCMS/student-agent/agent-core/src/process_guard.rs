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
    let url = format!("{}/api/policies/process-guard", config.teacher_server_url);

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
        let found = system
            .processes()
            .values()
            .any(|p| p.name().to_string_lossy().contains(&policy.process_name));

        if !found {
            info!(
                "Process '{}' not running, check interval={}s",
                policy.process_name, policy.check_interval_seconds
            );
            alerts.push(ProcessAlert {
                policy_id: policy.id.clone(),
                process_name: policy.process_name.clone(),
                alert_type: "process_missing".to_string(),
                message: format!("守护进程 {} 未运行", policy.process_name),
            });
        }
    }

    alerts
}

#[derive(Debug)]
pub struct ProcessAlert {
    pub policy_id: String,
    pub process_name: String,
    pub alert_type: String,
    pub message: String,
}
