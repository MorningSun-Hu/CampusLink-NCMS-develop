use anyhow::Result;
use serde::Deserialize;
use tokio::net::UdpSocket;
use tracing::{info, warn};
use std::net::SocketAddr;

const DISCOVERY_PORT: u16 = 9999;
const DISCOVERY_MESSAGE: &[u8] = b"CAMPUSLINK_DISCOVER";

#[derive(Debug, Deserialize)]
struct DiscoveryResponse {
    host: String,
    port: u16,
    #[serde(rename = "type")]
    service_type: String,
    version: String,
}

pub async fn discover(max_retries: u32) -> Result<String> {
    let broadcast_addr: SocketAddr = format!("255.255.255.255:{}", DISCOVERY_PORT).parse()?;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;

    let mut buf = [0u8; 512];

    for attempt in 1..=max_retries {
        info!("Discovery attempt {}/{}", attempt, max_retries);

        // Send broadcast
        if let Err(e) = socket.send_to(DISCOVERY_MESSAGE, broadcast_addr).await {
            warn!("Failed to send discovery broadcast: {}", e);
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            continue;
        }

        // Wait for response with timeout
        match tokio::time::timeout(tokio::time::Duration::from_secs(3), socket.recv_from(&mut buf)).await {
            Ok(Ok((len, _src))) => {
                let response = String::from_utf8_lossy(&buf[..len]);
                if let Ok(info) = serde_json::from_str::<DiscoveryResponse>(&response) {
                    if info.service_type == "campuslink-teacher" {
                        let url = format!("http://{}:{}", info.host, info.port);
                        info!("Discovered teacher server at {}", url);
                        return Ok(url);
                    }
                }
                warn!("Received invalid discovery response: {}", response);
            }
            Ok(Err(e)) => {
                warn!("Discovery recv error: {}", e);
            }
            Err(_) => {
                warn!("Discovery timeout (attempt {}/{})", attempt, max_retries);
            }
        }

        if attempt < max_retries {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    anyhow::bail!("No teacher server discovered after {} attempts", max_retries)
}
