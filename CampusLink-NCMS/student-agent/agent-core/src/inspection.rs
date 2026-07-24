use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::Config;

#[derive(Debug, Serialize)]
pub struct InspectionItem {
    pub item_name: String,
    pub status: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InspectionSubmitRequest {
    pub device_id: String,
    pub inspection_type: String,
    pub items: Vec<InspectionItem>,
    pub is_abnormal: bool,
}

#[derive(Debug, Deserialize)]
pub struct InspectionSubmitResponse {
    pub record_ids: Vec<String>,
    pub total_items: usize,
    pub abnormal_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

pub async fn submit_inspection(config: &Config, inspection_type: &str, items: Vec<InspectionItem>, is_abnormal: bool) -> Result<InspectionSubmitResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/inspection/submit", config.teacher_server_url);

    let request = InspectionSubmitRequest {
        device_id: config.device_id.clone().unwrap_or_default(),
        inspection_type: inspection_type.to_string(),
        items,
        is_abnormal,
    };

    info!("Submitting {} inspection to {}", inspection_type, url);
    let resp = client.post(&url).json(&request).send().await?;
    let api_resp: ApiResponse<InspectionSubmitResponse> = resp.json().await?;

    if api_resp.code == 0 {
        let data = api_resp.data.unwrap();
        info!("Inspection submitted: {} items, {} abnormal", data.total_items, data.abnormal_count);
        Ok(data)
    } else {
        Err(anyhow::anyhow!("Inspection submit failed: {}", api_resp.message))
    }
}

pub async fn report_abnormal(config: &Config, alert_type: &str, description: &str) -> Result<InspectionSubmitResponse> {
    let items = vec![
        InspectionItem {
            item_name: alert_type.to_string(),
            status: "abnormal".to_string(),
            description: Some(description.to_string()),
        }
    ];

    submit_inspection(config, alert_type, items, true).await
}

pub fn build_hygiene_items() -> Vec<InspectionItem> {
    vec![
        InspectionItem { item_name: "keyboard".to_string(), status: "normal".to_string(), description: None },
        InspectionItem { item_name: "mouse".to_string(), status: "normal".to_string(), description: None },
        InspectionItem { item_name: "monitor".to_string(), status: "normal".to_string(), description: None },
        InspectionItem { item_name: "desk".to_string(), status: "normal".to_string(), description: None },
    ]
}

pub fn build_equipment_items() -> Vec<InspectionItem> {
    vec![
        InspectionItem { item_name: "camera".to_string(), status: "normal".to_string(), description: None },
        InspectionItem { item_name: "microphone".to_string(), status: "normal".to_string(), description: None },
        InspectionItem { item_name: "headphone".to_string(), status: "normal".to_string(), description: None },
    ]
}
