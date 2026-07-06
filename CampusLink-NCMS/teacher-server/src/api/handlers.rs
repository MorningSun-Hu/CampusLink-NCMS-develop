use axum::{
    extract::{State, WebSocketUpgrade, ws::{WebSocket, Message}, Query},
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
use crate::crypto_util;

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
    pub jwt_secret: String,
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
    pub session_key: String,
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
            let session_key = {
                let key = format!("session_key_{}", req.device_code);
                let existing: Option<String> = sqlx::query_scalar(
                    "SELECT config_value FROM system_configs WHERE config_key = ?"
                )
                .bind(&key)
                .fetch_optional(&state.pool)
                .await
                .ok()
                .flatten();
                existing.unwrap_or_else(|| {
                    let new_key: [u8; 32] = rand::random();
                    let hex_key = hex::encode(new_key);
                    let id = uuid::Uuid::new_v4().to_string();
                    let _ = sqlx::query(
                        r#"INSERT INTO system_configs (id, config_key, config_value, scope, description, updated_at)
                           VALUES (?, ?, ?, 'security', 'Session key for device', datetime('now'))
                           ON CONFLICT(config_key) DO UPDATE SET config_value = ?, updated_at = datetime('now')"#
                    )
                    .bind(&id)
                    .bind(&key)
                    .bind(&hex_key)
                    .bind(&hex_key)
                    .execute(&state.pool);
                    hex_key
                })
            };
            Json(ApiResponse::success(RegisterDeviceResponse {
                device_id,
                teacher_fingerprint,
                initial_mode,
                heartbeat_interval_seconds: heartbeat_interval,
                session_key,
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
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let rx = state.ws_tx.subscribe();
    let client_fingerprint = params.get("teacher_fingerprint").cloned().unwrap_or_default();
    let device_id = params.get("device_id").cloned().unwrap_or_default();
    let session_key = params.get("session_key").cloned().unwrap_or_default();

    ws.on_upgrade(move |socket| async move {
        let server_fingerprint = crate::infrastructure::device_repository::get_config_value_string(&pool, "teacher_fingerprint")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "pending_init".to_string());

        if !client_fingerprint.is_empty() && server_fingerprint != "pending_init" && client_fingerprint != server_fingerprint {
            warn!("WebSocket rejected: teacher_fingerprint mismatch for device={}", device_id);
            let (mut sender, _) = socket.split();
            let _ = sender.send(Message::Text(
                r#"{"type":"auth_error","message":"teacher_fingerprint mismatch"}"#.to_string()
            )).await;
            let _ = sender.close().await;
            return;
        }

        info!("WebSocket connected: device={}, encrypted={}", device_id, !session_key.is_empty());
        let _ = handle_socket(socket, pool, rx, session_key).await;
    })
}

async fn handle_socket(socket: WebSocket, pool: SqlitePool, mut rx: broadcast::Receiver<String>, session_key: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use futures_util::stream::StreamExt;
    
    let (mut sender, mut receiver) = socket.split();
    let has_key = !session_key.is_empty();
    
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
                        let ack = r#"{"type":"heartbeat_ack","ack_code":0,"message":"ok"}"#;
                        if has_key {
                            let enc = crypto_util::encrypt_message(ack, &session_key).unwrap_or_else(|_| ack.as_bytes().to_vec());
                            let _ = sender.send(Message::Binary(enc)).await;
                        } else {
                            let _ = sender.send(Message::Text(ack.to_string())).await;
                        }
                    }
                    Message::Binary(data) => {
                        if has_key {
                            match crypto_util::decrypt_message(&data, &session_key) {
                                Ok(text) => {
                                    info!("Received encrypted: {}", text);
                                    if let Ok(msg_data) = serde_json::from_str::<serde_json::Value>(&text) {
                                        if let Some(device_id) = msg_data["device_id"].as_str() {
                                            let _ = device::update_heartbeat(&pool, device_id).await;
                                            if let Some(mode) = msg_data["mode"].as_str() {
                                                let _ = device::update_device_mode(&pool, device_id, mode).await;
                                            }
                                        }
                                    }
                                    let ack = r#"{"type":"heartbeat_ack","ack_code":0,"message":"ok"}"#;
                                    let enc = crypto_util::encrypt_message(ack, &session_key).unwrap_or_else(|_| ack.as_bytes().to_vec());
                                    let _ = sender.send(Message::Binary(enc)).await;
                                }
                                Err(e) => {
                                    warn!("Failed to decrypt binary message: {}", e);
                                }
                            }
                        }
                    }
                    Message::Ping(data) => {
                        let _ = sender.send(Message::Pong(data)).await;
                    }
                    Message::Close(_) => {
                        break;
                    }
                    _ => {}
                }
            }
            Ok(msg) = rx.recv() => {
                info!("Broadcasting: {}", msg);
                if has_key {
                    match crypto_util::encrypt_message(&msg, &session_key) {
                        Ok(enc) => { let _ = sender.send(Message::Binary(enc)).await; }
                        Err(e) => {
                            warn!("Failed to encrypt broadcast: {}", e);
                            let _ = sender.send(Message::Text(msg)).await;
                        }
                    }
                } else {
                    let _ = sender.send(Message::Text(msg)).await;
                }
            }
        }
    }

    Ok(())
}
