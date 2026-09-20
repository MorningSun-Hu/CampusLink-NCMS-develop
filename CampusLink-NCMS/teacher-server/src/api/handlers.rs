use axum::{
    extract::{State, WebSocketUpgrade, ws::{WebSocket, Message}, Query},
    response::IntoResponse,
    Json,
};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use prost::Message as ProstMessage;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{info, warn, error};
use std::collections::HashMap;
use tokio::sync::broadcast;
use crate::domain::device;
use crate::domain::usage;
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
    #[serde(default, alias = "classId")]
    pub class_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ModeSwitchResponse {
    pub command_id: String,
    pub device_id: String,
    pub target_mode: String,
    #[serde(rename = "effectiveMode")]
    pub effective_mode: String,
    #[serde(rename = "admissionBlocked")]
    pub admission_blocked: bool,
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
    const VALID_MODES: [&str; 4] = ["open", "teaching", "exam", "locked"];
    if !VALID_MODES.contains(&req.target_mode.as_str()) {
        return Json(ApiResponse::<ModeSwitchResponse>::error(
            400,
            format!("非法模式: {}（可选: open/teaching/exam/locked）", req.target_mode),
        ));
    }
    match device::switch_device_mode(
        &state.pool,
        &req.device_id,
        &req.target_mode,
        &req.operator_name,
        req.class_id.as_deref(),
    ).await {
        Ok(outcome) => {
            let effective_mode = outcome.effective_mode.clone();
            let msg = format!(r#"{{"type":"mode_switch","device_id":"{}","mode":"{}","operator":"{}","timestamp":{}}}"#, 
                req.device_id, effective_mode, req.operator_name,
                chrono::Utc::now().timestamp());

            // Encode as protobuf binary for efficient transport
            let proto_msg = crate::proto::campuslink::ncms::v1::ModeSwitchCommand {
                meta: Some(crate::proto::campuslink::ncms::v1::MessageMeta {
                    request_id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                    protocol_version: "1.0.0".to_string(),
                }),
                command_id: outcome.command_id.clone(),
                target_device_id: req.device_id.clone(),
                target_mode: match effective_mode.as_str() {
                    "open" => 0, "teaching" => 1, "exam" => 3, "locked" => 4,
                    _ => 0,
                },
                effective_at: chrono::Utc::now().timestamp(),
                operator_name: req.operator_name,
            };
            let proto_bytes = prost::Message::encode_to_vec(&proto_msg);
            // Send protobuf via broadcast channel as json wrapper for String compatibility
            let _ = state.ws_tx.send(msg);
            let _ = state.ws_tx.send(format!("proto:{}", base64_encode(&proto_bytes)));

            if requires_checkin(&effective_mode) {
                dispatch_checkin_trigger(&state, &[req.device_id.clone()]).await;
            }

            Json(ApiResponse::success(ModeSwitchResponse {
                command_id: outcome.command_id,
                device_id: req.device_id.clone(),
                target_mode: req.target_mode.clone(),
                effective_mode,
                delivery_status: "sent".to_string(),
                admission_blocked: outcome.admission_blocked,
            }))
        }
        Err(e) => {
            error!("Mode switch failed: {}", e);
            Json(ApiResponse::error(400, e.to_string()))
        }
    }
}

pub fn requires_checkin(mode: &str) -> bool {
    matches!(mode, "open" | "teaching")
}

pub async fn dispatch_checkin_trigger(state: &AppState, device_ids: &[String]) {
    let ts = chrono::Utc::now().timestamp();
    for device_id in device_ids {
        let online = device::is_online(&state.pool, device_id).await.unwrap_or(false);
        if online {
            let msg = format!(
                r#"{{"type":"checkin_trigger","device_id":"{}","timestamp":{}}}"#,
                device_id, ts
            );
            let _ = state.ws_tx.send(msg);
        } else if let Err(e) = device::set_pending_checkin(&state.pool, device_id, true).await {
            warn!("Failed to set pending checkin for {}: {}", device_id, e);
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 63) as usize] as char);
        result.push(CHARS[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 63) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
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
    let _ = device::remember_mode_and_lock(&state.pool, &req.device_id).await;
    let msg = format!(
        r#"{{"type":"lock_screen","device_id":"{}","reason":"{}","timestamp":{}}}"#,
        req.device_id,
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
    let restore = device::unlock_restore_mode(&state.pool, &req.device_id)
        .await
        .unwrap_or_else(|_| "open".to_string());
    let msg = format!(
        r#"{{"type":"unlock","device_id":"{}","restore_mode":"{}","timestamp":{}}}"#,
        req.device_id,
        restore,
        chrono::Utc::now().timestamp()
    );
    let _ = state.ws_tx.send(msg);
    info!("Unlock command sent for device={}, restore_mode={}", req.device_id, restore);
    Json(ApiResponse::success(serde_json::json!({"device_id": req.device_id, "status": "unlock_sent", "restore_mode": restore})))
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
                        if let Some(device_id) = process_heartbeat_json(&pool, &text).await {
                            resend_pending_trigger(&pool, &device_id, &mut sender, has_key, &session_key).await;
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
                                    if let Some(device_id) = process_heartbeat_json(&pool, &text).await {
                                        resend_pending_trigger(&pool, &device_id, &mut sender, has_key, &session_key).await;
                                    }
                                    let ack = r#"{"type":"heartbeat_ack","ack_code":0,"message":"ok"}"#;
                                    let enc = crypto_util::encrypt_message(ack, &session_key).unwrap_or_else(|_| ack.as_bytes().to_vec());
                                    let _ = sender.send(Message::Binary(enc)).await;
                                }
                                Err(_) => {
                                    // Try decoding as raw protobuf
                                    if let Ok(hb) = crate::proto::campuslink::ncms::v1::HeartbeatRequest::decode(data.as_ref()) {
                                        info!("Received protobuf heartbeat: device_id={}", hb.device_id);
                                        let _ = device::update_heartbeat(&pool, &hb.device_id).await;
                                        let mode_str = mode_value_to_str(hb.current_mode);
                                        let _ = device::update_device_mode(&pool, &hb.device_id, &mode_str).await;
                                        resend_pending_trigger(&pool, &hb.device_id, &mut sender, has_key, &session_key).await;
                                    }
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
                if msg.starts_with("proto:") {
                    // Protobuf binary message wrapped in base64
                    let b64_part = &msg[6..];
                    if let Ok(data) = base64_decode(b64_part) {
                        if has_key {
                            match crypto_util::encrypt_message(&String::from_utf8_lossy(&data), &session_key) {
                                Ok(enc) => { let _ = sender.send(Message::Binary(enc)).await; }
                                Err(e) => { warn!("Failed to encrypt proto broadcast: {}", e); }
                            }
                        } else {
                            let _ = sender.send(Message::Binary(data)).await;
                        }
                    }
                } else {
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
    }

    Ok(())
}

fn mode_value_to_str(v: i32) -> &'static str {
    match v {
        1 => "teaching", 3 => "exam", 4 => "locked", _ => "open",
    }
}

async fn process_heartbeat_json(pool: &SqlitePool, text: &str) -> Option<String> {
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(device_id) = data["device_id"].as_str() {
            let _ = device::update_heartbeat(pool, device_id).await;
            if let Some(mode) = data["mode"].as_str() {
                if matches!(mode, "open" | "teaching" | "exam" | "locked") {
                    let _ = device::update_device_mode(pool, device_id, mode).await;
                }
                if mode == "exam" {
                    let _ = usage::ensure_exam_session(pool, device_id).await;
                }
            }
            return Some(device_id.to_string());
        }
    }
    None
}

async fn resend_pending_trigger(
    pool: &SqlitePool,
    device_id: &str,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    has_key: bool,
    session_key: &str,
) {
    match device::consume_pending_checkin(pool, device_id).await {
        Ok(true) => {
            let trigger = format!(
                r#"{{"type":"checkin_trigger","device_id":"{}","timestamp":{}}}"#,
                device_id,
                chrono::Utc::now().timestamp()
            );
            if has_key {
                if let Ok(enc) = crypto_util::encrypt_message(&trigger, session_key) {
                    let _ = sender.send(Message::Binary(enc)).await;
                }
            } else {
                let _ = sender.send(Message::Text(trigger)).await;
            }
            info!("Resent pending checkin trigger to device={}", device_id);
        }
        Ok(false) => {}
        Err(e) => warn!("Pending checkin lookup failed for {}: {}", device_id, e),
    }
}

fn base64_decode(s: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    const DECODE: [i8; 128] = {
        let mut table = [-1i8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 { table[chars[i] as usize] = i as i8; i += 1; }
        table
    };
    let mut output = Vec::new();
    let bytes: Vec<u8> = s.bytes().filter(|&b| b != b'\n' && b != b'\r').collect();
    for chunk in bytes.chunks(4) {
        if chunk.len() < 2 { continue; }
        let v0 = DECODE[chunk[0] as usize];
        let v1 = DECODE[chunk[1] as usize];
        if v0 < 0 || v1 < 0 { continue; }
        let mut val = ((v0 as u32) << 2) | ((v1 as u32) >> 4);
        output.push(val as u8);
        if chunk.len() > 2 && chunk[2] != b'=' {
            let v2 = DECODE[chunk[2] as usize];
            if v2 >= 0 {
                val = (((v1 as u32) << 4) | ((v2 as u32) >> 2)) & 0xFF;
                output.push(val as u8);
            }
        }
        if chunk.len() > 3 && chunk[3] != b'=' {
            let v2 = DECODE[chunk[2] as usize];
            let v3 = DECODE[chunk[3] as usize];
            if v2 >= 0 && v3 >= 0 {
                val = (((v2 as u32) << 6) | (v3 as u32)) & 0xFF;
                output.push(val as u8);
            }
        }
    }
    Ok(output)
}
