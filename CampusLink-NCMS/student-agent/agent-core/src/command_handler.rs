use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::fs::File;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};

use crate::config::Config;
use crate::mode::{
    apply_unlock_restore, handle_mode_switch, remember_mode_before_lock, resolve_restore_mode,
};

/// WebSocket 命令枚举
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketCommand {
    /// 模式切换命令
    ModeSwitch {
        #[serde(default)]
        device_id: String,
        mode: String,
        #[serde(default)]
        operator: String,
        #[serde(default)]
        timestamp: u64,
    },
    /// 锁屏命令
    LockScreen {
        #[serde(default)]
        device_id: String,
        reason: String,
        timestamp: u64,
    },
    /// 解锁命令
    Unlock {
        #[serde(default)]
        device_id: String,
        #[serde(default)]
        restore_mode: String,
        timestamp: u64,
    },
    /// 签到触发命令（模式切换后由服务端下发）
    CheckinTrigger {
        #[serde(default)]
        device_id: String,
        #[serde(default)]
        timestamp: u64,
    },
    /// 超级密码更新
    SuperPwd {
        password: String,
        #[serde(default)]
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
    pub config: Config,
    unlock_tx: Option<mpsc::UnboundedSender<String>>,
}

impl CommandHandler {
    pub fn new(config: Config, unlock_tx: Option<mpsc::UnboundedSender<String>>) -> Self {
        Self { config, unlock_tx }
    }

    pub fn unlock_tx(&self) -> &Option<mpsc::UnboundedSender<String>> {
        &self.unlock_tx
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
            WebSocketCommand::ModeSwitch { device_id, mode, operator, timestamp: _ } => {
                // Only process commands targeting this device
                if !device_id.is_empty() && device_id != self.config.device_id.as_deref().unwrap_or("") {
                    debug!("Mode switch for different device {} ignored", device_id);
                    return Ok(());
                }
                info!("Received mode switch command: mode={}, operator={}", mode, operator);
                if let Err(e) = handle_mode_switch(&mut self.config, &mode, &self.unlock_tx).await {
                    error!("Mode switch failed: {}", e);
                    return Err(e);
                }
                info!("Mode switch completed: now in {} mode", self.config.current_mode);
                println!("\n[MODE SWITCHED] Now in {} mode\n", self.config.current_mode);
            }
            
            WebSocketCommand::LockScreen { device_id, reason, timestamp: _ } => {
                if !device_id.is_empty() && device_id != self.config.device_id.as_deref().unwrap_or("") {
                    debug!("Lock screen for different device {} ignored", device_id);
                    return Ok(());
                }
                info!("Received lock screen command: reason={}", reason);
                remember_mode_before_lock(&mut self.config, true);
                if self.config.is_locked {
                    self.config.current_mode = "locked".to_string();
                    let _ = self.config.save();
                    info!("Screen already locked, mode set to locked");
                    return Ok(());
                }
                let exe_dir = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.to_path_buf()));
                let lock_exe = match &exe_dir {
                    Some(dir) => dir.join("campus-lock.exe"),
                    None => std::path::PathBuf::from("campus-lock.exe"),
                };
                if !lock_exe.exists() {
                    error!("campus-lock.exe not found at: {:?}", lock_exe);
                    println!("\n[LOCK FAILED] campus-lock.exe not found\n");
                    return Ok(());
                }
                let password = self.config.lock_password.as_deref().unwrap_or("admin123");
                let stderr_file = match File::create("campus-lock-stderr.log") {
                    Ok(f) => f,
                    Err(e) => {
                        error!("Failed to create stderr log: {}", e);
                        return Ok(());
                    }
                };
                info!("Launching lock screen: {:?}", lock_exe);
                match Command::new(&lock_exe)
                    .arg(password)
                    .stdin(Stdio::null())
                    .stderr(Stdio::from(stderr_file))
                    .spawn()
                {
                    Ok(mut child) => {
                        info!("Spawn succeeded, PID={}", child.id());
                        // Wait briefly to see if child crashes immediately
                        std::thread::sleep(std::time::Duration::from_millis(800));
                        match child.try_wait() {
                            Ok(Some(status)) => {
                                error!("campus-lock exited immediately with: {:?}", status);
                                println!("\n[LOCK FAILED] campus-lock exited early: {:?} (check campus-lock-stderr.log)\n", status);
                            }
                            Ok(None) => {
                                info!("campus-lock still running, lock screen active");
                                self.config.lock_pid = Some(child.id());
                                self.config.is_locked = true;
                                self.config.current_mode = "locked".to_string();
                                let _ = self.config.save();
                                println!("\n[SCREEN LOCKED] {}\n", reason);
                            }
                            Err(e) => {
                                error!("Failed to check child status: {}", e);
                                self.config.lock_pid = Some(child.id());
                                self.config.is_locked = true;
                                self.config.current_mode = "locked".to_string();
                                let _ = self.config.save();
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to start lock screen: {}", e);
                        println!("\n[LOCK FAILED] Cannot start {:?}: {}\n", lock_exe, e);
                    }
                }
            }
            
            WebSocketCommand::Unlock { device_id, restore_mode, timestamp: _ } => {
                if !device_id.is_empty() && device_id != self.config.device_id.as_deref().unwrap_or("") {
                    debug!("Unlock for different device {} ignored", device_id);
                    return Ok(());
                }
                info!("Received unlock command");
                let restore = resolve_restore_mode(&self.config, &restore_mode);
                let start_admission = apply_unlock_restore(&mut self.config, restore.clone());
                if start_admission {
                    crate::attendance::start_admission_monitor(self.config.clone());
                }
                info!("Lock screen process terminated, restored mode={}", restore);
                println!("\n[SCREEN UNLOCKED] Restored to {} mode\n", restore);
            }
            
            WebSocketCommand::SuperPwd { password, timestamp: _ } => {
                info!("Received super password update");
                self.config.lock_password = Some(password.clone());
                if let Err(e) = self.config.save() {
                    error!("Failed to save updated lock password: {}", e);
                } else {
                    info!("Lock password updated successfully");
                    println!("\n[PASSWORD UPDATED] Lock password has been changed\n");
                }
            }
            
            WebSocketCommand::CheckinTrigger { device_id, timestamp: _ } => {
                if !device_id.is_empty() && device_id != self.config.device_id.as_deref().unwrap_or("") {
                    debug!("Checkin trigger for different device {} ignored", device_id);
                    return Ok(());
                }
                info!("Received checkin trigger command, mode={}", self.config.current_mode);
                crate::attendance::start_admission_monitor(self.config.clone());
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teacher_fingerprint: Option<String>,
}
