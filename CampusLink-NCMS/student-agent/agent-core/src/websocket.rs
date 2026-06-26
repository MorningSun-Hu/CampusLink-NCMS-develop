use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn, error, debug};

use crate::config::Config;
use crate::command_handler::{CommandHandler, HeartbeatRequest};
use crate::process_guard;

/// WebSocket 客户端
pub struct WebSocketClient {
    config: Config,
    ws_stream: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
}

impl WebSocketClient {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ws_stream: None,
            reconnect_attempts: 0,
            max_reconnect_attempts: 5,
        }
    }

    /// 构建 WebSocket 连接 URL
    fn build_ws_url(&self) -> Result<String> {
        let device_id = self.config.device_id
            .as_ref()
            .context("Device not registered")?;
        
        // 从 teacher_server_url 提取 host 和 port
        let server_url = &self.config.teacher_server_url;
        let ws_url = if server_url.starts_with("http://") {
            server_url.replace("http://", "ws://")
        } else if server_url.starts_with("https://") {
            server_url.replace("https://", "wss://")
        } else {
            format!("ws://{}", server_url)
        };

        Ok(format!("{}/ws?device_id={}", ws_url, device_id))
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
        };

        // 序列化为 Protobuf（这里简化为 JSON，实际应该用 prost）
        let json = serde_json::to_string(&heartbeat)?;
        let ws_stream = self.ws_stream.as_mut().unwrap();
        
        ws_stream.send(Message::Text(json.into()))
            .await
            .context("Failed to send heartbeat")?;

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
        self.ws_stream.is_none() && self.reconnect_attempts < self.max_reconnect_attempts
    }

    /// 获取下次重连的等待时间（指数退避）
    pub fn get_reconnect_delay(&self) -> std::time::Duration {
        use std::time::Duration;
        
        let base_delay = Duration::from_secs(1);
        let delay = base_delay * (2u32.pow(self.reconnect_attempts as u32));
        
        // 添加随机抖动（0-500ms）
        let jitter = Duration::from_millis(rand::random::<u64>() % 500);
        
        delay + jitter
    }

    /// 增加重连计数
    pub fn increment_reconnect_attempts(&mut self) {
        self.reconnect_attempts += 1;
        info!("Reconnect attempt {}/{}", self.reconnect_attempts, self.max_reconnect_attempts);
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
        }
    }

    /// 运行心跳循环
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting WebSocket heartbeat loop");

        // 初始连接
        if let Err(e) = self.client.connect().await {
            error!("Initial WebSocket connection failed: {}", e);
            return Err(e);
        }

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
                }

                // 接收解锁通知（campus-lock 进程退出）
                unlock_msg = self.unlock_rx.recv() => {
                    if let Some(msg) = unlock_msg {
                        info!("Unlock notification received: {}", msg);
                        self.config.current_mode = "open".to_string();
                        self.config.is_locked = false;
                        self.config.lock_pid = None;
                        if let Err(e) = self.config.save() {
                            error!("Failed to save config after unlock: {}", e);
                        }
                        // 立即上报解锁状态到教师端
                        if let Err(e) = self.client.send_heartbeat().await {
                            warn!("Failed to send unlock status update: {}", e);
                        }
                        println!("\n[SCREEN UNLOCKED] Device returned to open mode\n");
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
                        }
                        Ok(None) => {
                            if self.client.should_reconnect() {
                                self.client.increment_reconnect_attempts();
                                let delay = self.client.get_reconnect_delay();
                                warn!("Connection lost, reconnecting in {:?}", delay);
                                tokio::time::sleep(delay).await;
                                
                                if let Err(e) = self.client.connect().await {
                                    error!("Reconnection failed: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Receive error: {}", e);
                            if self.client.should_reconnect() {
                                self.client.increment_reconnect_attempts();
                                let delay = self.client.get_reconnect_delay();
                                tokio::time::sleep(delay).await;
                                
                                if let Err(e) = self.client.connect().await {
                                    error!("Reconnection failed: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
