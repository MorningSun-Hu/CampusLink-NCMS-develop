use anyhow::Result;
use tracing::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModeType {
    Open = 0,
    Teaching = 1,
    ConditionalOpen = 2,
    Exam = 3,
    Locked = 4,
}

pub async fn handle_mode_switch(command_id: &str, device_id: &str, target_mode: &str) -> Result<()> {
    info!("Received mode switch command: {} -> {} for device {}", command_id, target_mode, device_id);
    
    // P4 占位实现：仅记录日志，P5 将实现实际锁屏等功能
    info!("Mode applied: {} (placeholder)", target_mode);
    
    Ok(())
}

pub fn parse_mode(mode_str: &str) -> ModeType {
    match mode_str {
        "teaching" => ModeType::Teaching,
        "conditional_open" => ModeType::ConditionalOpen,
        "exam" => ModeType::Exam,
        "locked" => ModeType::Locked,
        _ => ModeType::Open,
    }
}
