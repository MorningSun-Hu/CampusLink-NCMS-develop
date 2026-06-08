use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use futures_util::stream::SplitSink;
use tokio_tungstenite::WebSocketStream;
use tokio::sync::mpsc;
use tracing::{info, error, warn};

pub struct HeartbeatClient {
    ws_tx: Option<SplitSink<WebSocketStream, tungstenite::Message>>,
}

impl HeartbeatClient {
    pub fn new() -> Self {
        HeartbeatClient { ws_tx: None }
    }

    pub async fn connect(&mut self, ws_url: &str) -> Result<()> {
        let (ws_stream, _) = connect_async(ws_url).await?;
        let (ws_tx, mut ws_rx) = ws_stream.split();
        self.ws_tx = Some(ws_tx);

        info!("WebSocket connected to {}", ws_url);

        Ok(())
    }

    pub async fn send_heartbeat(&mut self, device_id: &str) -> Result<()> {
        if let Some(ref mut tx) = self.ws_tx {
            let heartbeat_msg = format!(r#"{{"type":"heartbeat","device_id":"{}"}}"#, device_id);
            tx.send(tungstenite::Message::Text(heartbeat_msg)).await?;
            info!("Heartbeat sent for device: {}", device_id);
        }
        Ok(())
    }

    pub async fn receive_messages(&mut self, device_id: String, mode_tx: mpsc::Sender<String>) -> Result<()> {
        if let Some((_, mut ws_rx)) = self.ws_tx.as_ref().map(|_| ()) {
            // 需要重新建立连接来获取 rx
            // 简化处理，这里只做示意
            info!("Starting to receive messages for device: {}", device_id);
        }
        Ok(())
    }
}

pub async fn start_heartbeat_loop(
    ws_url: String,
    device_id: String,
    interval_seconds: u64,
) -> Result<()> {
    let mut client = HeartbeatClient::new();
    
    // 连接 WebSocket
    match client.connect(&ws_url).await {
        Ok(_) => info!("Connected to WebSocket"),
        Err(e) => {
            error!("Failed to connect WebSocket: {}", e);
            return Err(e);
        }
    }

    // 定时发送心跳
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(interval_seconds)).await;
        
        match client.send_heartbeat(&device_id).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to send heartbeat: {}", e);
                // 尝试重连
                if let Err(e) = client.connect(&ws_url).await {
                    error!("Reconnect failed: {}", e);
                }
            }
        }
    }
}
