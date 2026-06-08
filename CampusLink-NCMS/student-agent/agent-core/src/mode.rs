use anyhow::Result;
use tracing::info;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModeType {
    Open = 0,
    Teaching = 1,
    ConditionalOpen = 2,
    Exam = 3,
    Locked = 4,
}

impl ToString for ModeType {
    fn to_string(&self) -> String {
        match self {
            ModeType::Open => "open".to_string(),
            ModeType::Teaching => "teaching".to_string(),
            ModeType::ConditionalOpen => "conditional_open".to_string(),
            ModeType::Exam => "exam".to_string(),
            ModeType::Locked => "locked".to_string(),
        }
    }
}

/// 处理模式切换命令
pub async fn handle_mode_switch(config: &mut Config, target_mode: &str) -> Result<()> {
    info!("Switching mode from {} to {}", config.current_mode, target_mode);
    
    let parsed_mode = parse_mode(target_mode);
    
    // 执行模式对应的动作
    match parsed_mode {
        ModeType::Open => {
            info!("Switching to Open mode - disabling all restrictions");
            disable_all_locks().await?;
        }
        ModeType::Teaching => {
            info!("Switching to Teaching mode - enabling screen lock");
            enable_teaching_mode().await?;
        }
        ModeType::ConditionalOpen => {
            info!("Switching to Conditional Open mode - enabling whitelist restrictions");
            enable_conditional_mode().await?;
        }
        ModeType::Exam => {
            info!("Switching to Exam mode - enabling exam lockdown");
            enable_exam_mode().await?;
        }
        ModeType::Locked => {
            info!("Switching to Locked mode - full lockdown");
            lock_completely().await?;
        }
    }
    
    // 更新配置中的当前模式
    config.current_mode = target_mode.to_string();
    config.save()?;
    
    // TODO: 上报模式变更到教师端
    
    info!("Mode switch completed: {}", target_mode);
    Ok(())
}

/// 开放模式：解除所有限制
async fn disable_all_locks() -> Result<()> {
    info!("Disabling all locks - Open mode");
    // TODO: 调用 campus-lock 解锁
    // let mut lock_manager = LockManager::new();
    // lock_manager.unlock().await?;
    Ok(())
}

/// 授课模式：锁定屏幕，跟随教师演示
async fn enable_teaching_mode() -> Result<()> {
    info!("Enabling Teaching mode - screen lock active");
    // TODO: 实现屏幕锁定，等待教师端投屏
    // let mut lock_manager = LockManager::new();
    // lock_manager.lock().await?;
    Ok(())
}

/// 条件开放模式：仅开放白名单应用/网站
async fn enable_conditional_mode() -> Result<()> {
    info!("Enabling Conditional Open mode - whitelist restrictions active");
    // TODO: 实现应用/网站白名单过滤
    Ok(())
}

/// 考试模式：全屏锁定，禁止切换应用
async fn enable_exam_mode() -> Result<()> {
    info!("Enabling Exam mode - exam lockdown active");
    // TODO: 实现全屏锁定，禁用任务管理器等
    // let mut lock_manager = LockManager::new();
    // lock_manager.lock().await?;
    Ok(())
}

/// 锁定模式：完全锁定
async fn lock_completely() -> Result<()> {
    info!("Enabling Locked mode - full lockdown");
    // TODO: 调用 campus-lock 完全锁定
    // let mut lock_manager = LockManager::new();
    // lock_manager.lock().await?;
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
