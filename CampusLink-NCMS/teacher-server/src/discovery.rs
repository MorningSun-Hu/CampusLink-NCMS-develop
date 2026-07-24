use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tracing::{info, error};

pub fn start(server_port: u16) {
    tokio::spawn(async move {
        let bind_addr: SocketAddr = format!("0.0.0.0:9999").parse().unwrap();
        let socket = match UdpSocket::bind(bind_addr).await {
            Ok(s) => s,
            Err(e) => {
                error!("UDP discovery listener failed to bind: {}", e);
                return;
            }
        };

        info!("UDP discovery listener started on 0.0.0.0:9999");

        let response = format!(
            r#"{{"host":"0.0.0.0","port":{},"type":"campuslink-teacher","version":"0.1.0"}}"#,
            server_port
        );

        let mut buf = [0u8; 64];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, src)) => {
                    let msg = String::from_utf8_lossy(&buf[..len]);
                    if msg.trim() == "CAMPUSLINK_DISCOVER" {
                        info!("Discovery request from {}", src);
                        if let Err(e) = socket.send_to(response.as_bytes(), src).await {
                            error!("Failed to send discovery response to {}: {}", src, e);
                        }
                    }
                }
                Err(e) => {
                    error!("UDP recv error: {}", e);
                }
            }
        }
    });
}
