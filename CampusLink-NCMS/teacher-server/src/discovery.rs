use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tracing::{info, error};

fn local_ip() -> String {
    match if_addrs::get_if_addrs() {
        Ok(addrs) => {
            for a in &addrs {
                if let std::net::IpAddr::V4(v4) = a.ip() {
                    if !v4.is_loopback() && !v4.is_link_local() {
                        return v4.to_string();
                    }
                }
            }
            error!("No usable IPv4 interface found, falling back to 0.0.0.0");
            "0.0.0.0".to_string()
        }
        Err(e) => {
            error!("Failed to enumerate interfaces, falling back to 0.0.0.0: {}", e);
            "0.0.0.0".to_string()
        }
    }
}

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

        let host = local_ip();
        info!("UDP discovery advertising host: {}", host);

        let response = format!(
            r#"{{"host":"{}","port":{},"type":"campuslink-teacher","version":"0.1.0"}}"#,
            host, server_port
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
