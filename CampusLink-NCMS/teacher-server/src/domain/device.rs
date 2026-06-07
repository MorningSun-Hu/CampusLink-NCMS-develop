use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentDevice {
    pub id: String,
    pub student_id: Option<String>,
    pub device_code: String,
    pub device_name: String,
    pub machine_fingerprint: String,
    pub hostname: String,
    pub ip_address: String,
    pub mac_address: String,
    pub register_status: String,
    pub online_status: String,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub current_mode: String,
    pub agent_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_code: String,
    pub machine_fingerprint: String,
    pub hostname: String,
    pub ip_address: String,
    pub mac_address: String,
    pub agent_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeviceResponse {
    pub device_id: String,
    pub teacher_fingerprint: String,
    pub initial_mode: String,
    pub heartbeat_interval_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceListItem {
    pub id: String,
    pub device_code: String,
    pub device_name: String,
    pub register_status: String,
    pub online_status: String,
    pub current_mode: String,
    pub last_seen_at: Option<DateTime<Utc>>,
}
