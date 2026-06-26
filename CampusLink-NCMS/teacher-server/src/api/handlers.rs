use axum::{
    extract::{State, WebSocketUpgrade, ws::{WebSocket, Message}},
    response::IntoResponse,
    Json,
};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{info, warn, error};
use std::collections::HashMap;
use tokio::sync::broadcast;
use crate::domain::device;

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

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub ws_tx: broadcast::Sender<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_code: String,
    pub machine_fingerprint: String,
    pub hostname: String,
    pub ip_address: String,
    pub mac_address: String,
    pub agent_version: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterDeviceResponse {
    pub device_id: String,
    pub teacher_fingerprint: String,
    pub initial_mode: String,
    pub heartbeat_interval_seconds: u32,
}

#[derive(Debug, Deserialize)]
pub struct ModeSwitchRequest {
    pub target_mode: String,
    pub device_id: String,
    pub operator_name: String,
}

#[derive(Debug, Serialize)]
pub struct ModeSwitchResponse {
    pub command_id: String,
    pub device_id: String,
    pub target_mode: String,
    pub delivery_status: String,
}

pub async fn register_device_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterDeviceRequest>,
) -> impl IntoResponse {
    match device::register_device(
        &state.pool,
        &req.device_code,
        &req.machine_fingerprint,
        &req.hostname,
        &req.ip_address,
        &req.mac_address,
        &req.agent_version,
    ).await {
        Ok((device_id, teacher_fingerprint, initial_mode, heartbeat_interval)) => {
            Json(ApiResponse::success(RegisterDeviceResponse {
                device_id,
                teacher_fingerprint,
                initial_mode,
                heartbeat_interval_seconds: heartbeat_interval,
            }))
        }
        Err(e) => {
            error!("Register device failed: {}", e);
            Json(ApiResponse::error(500, "注册失败".to_string()))
        }
    }
}

pub async fn list_devices_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match device::list_devices(&state.pool).await {
        Ok(devices) => Json(ApiResponse::success(devices)),
        Err(e) => {
            error!("List devices failed: {}", e);
            Json(ApiResponse::error(500, "查询失败".to_string()))
        }
    }
}

pub async fn mode_switch_handler(
    State(state): State<AppState>,
    Json(req): Json<ModeSwitchRequest>,
) -> impl IntoResponse {
    match device::switch_device_mode(&state.pool, &req.device_id, &req.target_mode, &req.operator_name).await {
        Ok(command_id) => {
            let msg = format!(r#"{{"type":"mode_switch","device_id":"{}","mode":"{}","operator":"{}","timestamp":{}}}"#, 
                req.device_id, req.target_mode, req.operator_name,
                chrono::Utc::now().timestamp());
            
            let _ = state.ws_tx.send(msg);
            
            Json(ApiResponse::success(ModeSwitchResponse {
                command_id,
                device_id: req.device_id.clone(),
                target_mode: req.target_mode.clone(),
                delivery_status: "sent".to_string(),
            }))
        }
        Err(e) => {
            error!("Mode switch failed: {}", e);
            Json(ApiResponse::error(500, "模式切换失败".to_string()))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LockScreenRequest {
    pub device_id: String,
    pub reason: Option<String>,
}

pub async fn lock_screen_handler(
    State(state): State<AppState>,
    Json(req): Json<LockScreenRequest>,
) -> impl IntoResponse {
    let reason = req.reason.unwrap_or_else(|| "Teacher locked this device".to_string());
    let msg = format!(
        r#"{{"type":"lock_screen","reason":"{}","timestamp":{}}}"#,
        reason,
        chrono::Utc::now().timestamp()
    );
    let _ = state.ws_tx.send(msg);
    info!("Lock screen command sent for device={}", req.device_id);
    Json(ApiResponse::success(serde_json::json!({"device_id": req.device_id, "status": "lock_sent"})))
}

pub async fn unlock_handler(
    State(state): State<AppState>,
    Json(req): Json<LockScreenRequest>,
) -> impl IntoResponse {
    let msg = format!(
        r#"{{"type":"unlock","timestamp":{}}}"#,
        chrono::Utc::now().timestamp()
    );
    let _ = state.ws_tx.send(msg);
    info!("Unlock command sent for device={}", req.device_id);
    Json(ApiResponse::success(serde_json::json!({"device_id": req.device_id, "status": "unlock_sent"})))
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rx = state.ws_tx.subscribe();
    ws.on_upgrade(|socket| async move {
        let _ = handle_socket(socket, state.pool, rx).await;
    })
}

async fn handle_socket(socket: WebSocket, pool: SqlitePool, mut rx: broadcast::Receiver<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use futures_util::stream::StreamExt;
    
    let (mut sender, mut receiver) = socket.split();
    
    loop {
        tokio::select! {
            Some(Ok(msg)) = receiver.next() => {
                match msg {
                    Message::Text(text) => {
                        info!("Received: {}", text);
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(device_id) = data["device_id"].as_str() {
                                let _ = device::update_heartbeat(&pool, device_id).await;
                                if let Some(mode) = data["mode"].as_str() {
                                    let _ = device::update_device_mode(&pool, device_id, mode).await;
                                }
                            }
                        }
                        sender.send(Message::Text(r#"{"type":"heartbeat_ack","ack_code":0,"message":"ok"}"#.to_string())).await?;
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
            Ok(msg) = rx.recv() => {
                info!("Broadcasting: {}", msg);
                sender.send(Message::Text(msg)).await?;
            }
        }
    }

    Ok(())
}
