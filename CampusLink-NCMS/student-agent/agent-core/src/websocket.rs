use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn, error, debug};

use crate::config::Config;
use crate::command_handler::{CommandHandler, HeartbeatRequest};
use crate::process_guard;
use crate::crypto;  // added for message encryption

/// WebSocket 客户端
pub struct WebSocketClient {
    config: Config,
    ws_stream: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    reconnect_attempts: u32,
}

impl WebSocketClient {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ws_stream: None,
            reconnect_attempts: 0,
        }
    }

    /// 构建 WebSocket 连接 URL
    fn build_ws_url(&self) -> Result<String> {
        let device_id = self.config.device_id
            .as_ref()
            .context("Device not registered")?;
        
        let fingerprint = self.config.teacher_fingerprint
            .as_deref()
            .unwrap_or("");

        let session_key = self.config.session_key
            .as_deref()
            .unwrap_or("");

        let server_url = &self.config.teacher_server_url;
        let ws_url = if server_url.starts_with("http://") {
            server_url.replace("http://", "ws://")
        } else if server_url.starts_with("https://") {
            server_url.replace("https://", "wss://")
        } else {
            format!("ws://{}", server_url)
        };

        Ok(format!("{}/ws?device_id={}&teacher_fingerprint={}&session_key={}", ws_url, device_id, fingerprint, session_key))
    }

    /// 连接 WebSocket 服务器
    pub async fn connect(&mut self) -> Result<()> {
        let url = self.build_ws_url()?;
        info!("Connecting to WebSocket: {}", url);

        let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
            .await
            .context("Failed to connect to WebSocket server")?;

        self.ws_stream = Some(ws_stream);
        self.reconnect_attempts = 0;
        info!("WebSocket connected");

        Ok(())
    }

    /// 发送心跳
    pub async fn send_heartbeat(&mut self) -> Result<()> {
        if self.ws_stream.is_none() {
            return Err(anyhow::anyhow!("WebSocket not connected"));
        }

        let heartbeat = HeartbeatRequest {
            device_id: self.config.device_id.clone().unwrap_or_default(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            status: "online".to_string(),
            mode: self.config.current_mode.clone(),
            teacher_fingerprint: self.config.teacher_fingerprint.clone(),
        };

        // 序列化为 Protobuf（这里简化为 JSON，实际应该用 prost）
        let json = serde_json::to_string(&heartbeat)?;
        let ws_stream = self.ws_stream.as_mut().unwrap();

        // Encrypt with session key if available
        if let Some(ref key) = self.config.session_key {
            let encrypted = crypto::encrypt_message(&json, key)
                .map_err(|e| anyhow::anyhow!("encrypt: {}", e))?;
            ws_stream.send(Message::Binary(encrypted.into()))
                .await
                .context("Failed to send heartbeat")?;
        } else {
            ws_stream.send(Message::Text(json.into()))
                .await
                .context("Failed to send heartbeat")?;
        }

        debug!("Heartbeat sent");
        Ok(())
    }

    /// 接收并处理消息
    pub async fn receive_message(&mut self) -> Result<Option<String>> {
        if self.ws_stream.is_none() {
            return Ok(None);
        }

        let ws_stream = self.ws_stream.as_mut().unwrap();
        
        match ws_stream.next().await {
            Some(Ok(Message::Text(text))) => {
                debug!("Received message: {}", text);
                Ok(Some(text.to_string()))
            }
            Some(Ok(Message::Binary(data))) => {
                if let Some(ref key) = self.config.session_key {
                    match crypto::decrypt_message(&data, key) {
                        Ok(text) => {
                            debug!("Received encrypted message: {}", text);
                            Ok(Some(text))
                        }
                        Err(e) => {
                            error!("Failed to decrypt message: {}", e);
                            Ok(None)
                        }
                    }
                } else {
                    debug!("Received binary message (no session key)");
                    Ok(None)
                }
            }
            Some(Ok(Message::Ping(data))) => {
                debug!("Received ping");
                ws_stream.send(Message::Pong(data)).await?;
                Ok(None)
            }
            Some(Ok(Message::Close(frame))) => {
                warn!("WebSocket closed: {:?}", frame);
                self.ws_stream = None;
                Ok(None)
            }
            Some(Ok(_)) => Ok(None),
            Some(Err(e)) => {
                error!("WebSocket error: {}", e);
                self.ws_stream = None;
                Err(anyhow::anyhow!("WebSocket error: {}", e))
            }
            None => {
                warn!("WebSocket stream closed");
                self.ws_stream = None;
                Ok(None)
            }
        }
    }

    /// 断开连接
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(ws_stream) = self.ws_stream.as_mut() {
            ws_stream.close(None).await
                .context("Failed to close WebSocket")?;
            info!("WebSocket disconnected");
        }
        self.ws_stream = None;
        Ok(())
    }

    /// 检查是否需要重连
    pub fn should_reconnect(&self) -> bool {
        self.ws_stream.is_none()
    }

    /// 获取下次重连的等待时间（指数退避）
    pub fn get_reconnect_delay(&self) -> std::time::Duration {
        use std::time::Duration;
        
        let exp = self.reconnect_attempts.min(5);
        let secs = (1u64 << exp).min(30);
        let delay = Duration::from_secs(secs);
        
        // 添加随机抖动（0-500ms）
        let jitter = Duration::from_millis(rand::random::<u64>() % 500);
        
        delay + jitter
    }

    /// 增加重连计数
    pub fn increment_reconnect_attempts(&mut self) {
        self.reconnect_attempts = self.reconnect_attempts.saturating_add(1);
        info!("Reconnect attempt {}", self.reconnect_attempts);
    }
}

/// WebSocket 心跳循环管理器
pub struct HeartbeatLoop {
    client: WebSocketClient,
    heartbeat_interval: std::time::Duration,
    command_handler: CommandHandler,
    config: Config,
    process_guard_interval: std::time::Duration,
    unlock_rx: mpsc::UnboundedReceiver<String>,
    mode_synced: bool,
}

impl HeartbeatLoop {
    pub fn new(config: Config) -> Self {
        let client = WebSocketClient::new(config.clone());
        let (unlock_tx, unlock_rx) = mpsc::unbounded_channel();
        let command_handler = CommandHandler::new(config.clone(), Some(unlock_tx));
        
        Self {
            client,
            heartbeat_interval: std::time::Duration::from_secs(30),
            command_handler,
            config,
            process_guard_interval: std::time::Duration::from_secs(60),
            unlock_rx,
            mode_synced: false,
        }
    }

    /// 运行心跳循环
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting WebSocket heartbeat loop");

        let local = crate::mode::normalize_local_mode(&self.config.current_mode);
        self.apply_mode(&local).await;
        self.connect_with_retry().await;

        let mut heartbeat_interval = tokio::time::interval(self.heartbeat_interval);
        let mut process_guard_tick = tokio::time::interval(self.process_guard_interval);
        
        loop {
            tokio::select! {
                // 定时发送心跳
                _ = heartbeat_interval.tick() => {
                    if let Err(e) = self.client.send_heartbeat().await {
                        warn!("Failed to send heartbeat: {}", e);
                    }
                }

                // 定时检查进程守护
                _ = process_guard_tick.tick() => {
                    let policies = process_guard::sync_policies(&self.config).await;
                    let alerts = process_guard::check_and_restart(&policies);
                    process_guard::report_alerts(&self.config, &alerts).await;
                    // Also guard campus-guard itself
                    if let Some(alert) = process_guard::guard_campus_guard() {
                        process_guard::report_alerts(&self.config, &[alert]).await;
                    }
                }

                // 接收解锁通知（campus-lock 进程退出）
                unlock_msg = self.unlock_rx.recv() => {
                    if let Some(msg) = unlock_msg {
                        info!("Unlock notification received: {}", msg);
                        let restore = crate::mode::resolve_restore_mode(&self.command_handler.config, "");
                        let start_admission = crate::mode::apply_unlock_restore(
                            &mut self.command_handler.config,
                            restore.clone(),
                        );
                        self.config.current_mode = self.command_handler.config.current_mode.clone();
                        self.config.is_locked = false;
                        self.config.lock_pid = None;
                        self.config.mode_before_lock = self.command_handler.config.mode_before_lock.clone();
                        self.config.lock_is_overlay = self.command_handler.config.lock_is_overlay;
                        if start_admission {
                            crate::attendance::start_admission_monitor(self.config.clone());
                        }
                        // 立即上报解锁状态到教师端
                        if let Err(e) = self.client.send_heartbeat().await {
                            warn!("Failed to send unlock status update: {}", e);
                        }
                        println!("\n[SCREEN UNLOCKED] Restored to {} mode\n", restore);
                    }
                }
                
                // 接收消息
                msg = self.client.receive_message() => {
                    match msg {
                        Ok(Some(text)) => {
                            if let Err(e) = self.command_handler.handle_command(&text).await {
                                error!("Failed to handle command: {}", e);
                            }
                            // 同步 command_handler 的 Config 状态回心跳循环
                            self.config.current_mode = self.command_handler.config.current_mode.clone();
                            self.config.is_locked = self.command_handler.config.is_locked;
                            self.config.lock_pid = self.command_handler.config.lock_pid;
                            self.config.mode_before_lock = self.command_handler.config.mode_before_lock.clone();
                            self.config.lock_is_overlay = self.command_handler.config.lock_is_overlay;
                        }
                        Ok(None) => {
                            warn!("Connection lost, reconnecting");
                            self.connect_with_retry().await;
                        }
                        Err(e) => {
                            error!("Receive error: {}", e);
                            self.connect_with_retry().await;
                        }
                    }
                }
            }
        }
    }

    fn sync_config_from_handler(&mut self) {
        self.config.current_mode = self.command_handler.config.current_mode.clone();
        self.config.is_locked = self.command_handler.config.is_locked;
        self.config.lock_pid = self.command_handler.config.lock_pid;
        self.config.mode_before_lock = self.command_handler.config.mode_before_lock.clone();
        self.config.lock_is_overlay = self.command_handler.config.lock_is_overlay;
        self.config.device_id = self.command_handler.config.device_id.clone();
        self.config.session_key = self.command_handler.config.session_key.clone();
        self.config.teacher_fingerprint = self.command_handler.config.teacher_fingerprint.clone();
    }

    async fn apply_mode(&mut self, target: &str) {
        let unlock_tx = self.command_handler.unlock_tx().clone();
        if let Err(e) = crate::mode::handle_mode_switch(
            &mut self.command_handler.config,
            target,
            &unlock_tx,
        )
        .await
        {
            warn!("Failed to apply mode {}: {}", target, e);
        }
        self.sync_config_from_handler();
    }

    async fn connect_with_retry(&mut self) {
        loop {
            if self.config.device_id.is_none() {
                match crate::register::collect_and_register(&mut self.config).await {
                    Ok(_) => {
                        self.command_handler.config.device_id = self.config.device_id.clone();
                        self.command_handler.config.session_key = self.config.session_key.clone();
                        self.command_handler.config.teacher_fingerprint =
                            self.config.teacher_fingerprint.clone();
                        self.command_handler.config.heartbeat_interval_seconds =
                            self.config.heartbeat_interval_seconds;
                        self.client = WebSocketClient::new(self.config.clone());
                        info!("Registration completed");
                    }
                    Err(e) => {
                        self.client.increment_reconnect_attempts();
                        let delay = self.client.get_reconnect_delay();
                        warn!("Registration failed: {}, retry in {:?}", e, delay);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                }
            }

            match self.client.connect().await {
                Ok(()) => {
                    if !self.mode_synced {
                        match self.sync_teacher_mode().await {
                            Ok(()) => self.mode_synced = true,
                            Err(e) => warn!("Teacher mode sync failed: {}", e),
                        }
                    }
                    return;
                }
                Err(e) => {
                    self.client.increment_reconnect_attempts();
                    let delay = self.client.get_reconnect_delay();
                    warn!("WebSocket connect failed: {}, retry in {:?}", e, delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    async fn sync_teacher_mode(&mut self) -> Result<()> {
        let device_id = self
            .config
            .device_id
            .as_deref()
            .context("Device not registered")?;
        let teacher_mode =
            crate::register::fetch_teacher_mode(&self.config.teacher_server_url, device_id).await?;
        let next = crate::mode::resolve_synced_mode(&self.config.current_mode, &teacher_mode);
        if next != self.config.current_mode {
            info!(
                "Syncing mode {} -> {} (teacher={})",
                self.config.current_mode, next, teacher_mode
            );
            self.apply_mode(&next).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn should_reconnect_has_no_attempt_cap() {
        let mut client = WebSocketClient::new(Config::default());
        client.reconnect_attempts = 100;
        assert!(client.should_reconnect());
    }

    #[test]
    fn reconnect_delay_is_capped_near_30s() {
        let mut client = WebSocketClient::new(Config::default());
        client.reconnect_attempts = 40;
        let delay = client.get_reconnect_delay();
        assert!(delay <= Duration::from_secs(30) + Duration::from_millis(499));
        assert!(delay >= Duration::from_secs(30));
    }
}
