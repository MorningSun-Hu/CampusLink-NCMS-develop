use axum::{
    extract::{State, WebSocketUpgrade, ws::{WebSocket, Message}},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;

use crate::domain::device::{RegisterDeviceRequest, RegisterDeviceResponse};
use crate::infrastructure::device_repository::{register_device, list_devices};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: i32, message: String) -> Self {
        ApiResponse {
            code,
            message,
            data: None,
        }
    }
}

pub async fn register_device_handler(
    State(pool): State<SqlitePool>,
    Json(req): Json<RegisterDeviceRequest>,
) -> impl IntoResponse {
    match register_device(&pool, req).await {
        Ok(resp) => Json(ApiResponse::success(resp)),
        Err(e) => {
            info!("Register device failed: {}", e);
            Json(ApiResponse::error(500, "注册失败".to_string()))
        }
    }
}

pub async fn list_devices_handler(
    State(pool): State<SqlitePool>,
) -> impl IntoResponse {
    match list_devices(&pool, None).await {
        Ok(devices) => Json(ApiResponse::success(devices)),
        Err(e) => {
            info!("List devices failed: {}", e);
            Json(ApiResponse::error(500, "查询失败".to_string()))
        }
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_pool): State<SqlitePool>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        let _ = handle_socket(socket).await;
    })
}

async fn handle_socket(socket: WebSocket) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (mut sender, mut receiver) = socket.split();
    
    while let Some(msg) = receiver.recv().await {
        match msg {
            Message::Text(text) => {
                info!("Received WebSocket message: {}", text);
                sender.send(Message::Text("ack".to_string())).await?;
            }
            Message::Ping(data) => {
                sender.send(Message::Pong(data)).await?;
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
