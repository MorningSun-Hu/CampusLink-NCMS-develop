use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};

use crate::config::Config;
use crate::mode::handle_mode_switch;

/// WebSocket 命令枚举
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketCommand {
    /// 模式切换命令
    ModeSwitch {
        mode: String,
        operator: String,
        timestamp: u64,
    },
    /// 锁屏命令
    LockScreen {
        reason: String,
        timestamp: u64,
    },
    /// 解锁命令
    Unlock {
        timestamp: u64,
    },
    /// 心跳响应
    HeartbeatAck {
        ack_code: i32,
        message: Option<String>,
    },
    /// 其他未知命令
    Unknown {
        raw: String,
    },
}

/// 命令处理器
pub struct CommandHandler {
    config: Config,
}

impl CommandHandler {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 处理 WebSocket 命令
    pub async fn handle_command(&mut self, message: &str) -> Result<()> {
        info!("Processing command: {}", message);

        // 尝试解析为 JSON 格式的命令
        // 注意：实际应该使用 Protobuf 解析，这里先用 JSON 简化实现
        let command: WebSocketCommand = match serde_json::from_str(message) {
            Ok(cmd) => cmd,
            Err(e) => {
                warn!("Failed to parse command as JSON: {}", e);
                // 如果是 Protobuf 二进制数据，这里需要特殊处理
                // 暂时作为未知命令处理
                return Ok(());
            }
        };

        match command {
            WebSocketCommand::ModeSwitch { mode, operator, timestamp: _ } => {
                info!("Received mode switch command: mode={}, operator={}", mode, operator);
                if let Err(e) = handle_mode_switch(&mut self.config, &mode).await {
                    error!("Mode switch failed: {}", e);
                    return Err(e);
                }
                info!("Mode switch completed");
            }
            
            WebSocketCommand::LockScreen { reason, timestamp: _ } => {
                info!("Received lock screen command: reason={}", reason);
                // TODO: 调用 campus-lock 执行锁屏
                warn!("Lock screen not yet implemented");
            }
            
            WebSocketCommand::Unlock { timestamp: _ } => {
                info!("Received unlock command");
                // TODO: 调用 campus-lock 执行解锁
                warn!("Unlock not yet implemented");
            }
            
            WebSocketCommand::HeartbeatAck { ack_code, message } => {
                debug!("Heartbeat acknowledged: code={}, msg={:?}", ack_code, message);
            }
            
            WebSocketCommand::Unknown { raw } => {
                warn!("Unknown command received: {}", raw);
            }
        }

        Ok(())
    }
}

/// 心跳请求结构（用于发送）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub device_id: String,
    pub timestamp: u64,
    pub status: String,
    pub mode: String,
}
