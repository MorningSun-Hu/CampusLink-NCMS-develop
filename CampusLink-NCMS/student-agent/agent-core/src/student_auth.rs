use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::Config;

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub student_id: String,
    pub student_no: String,
    pub name: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

pub async fn login(config: &Config, username: &str, password: &str) -> Result<LoginResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/auth/login", config.teacher_server_url);

    let request = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };

    info!("Logging in as {} at {}", username, url);
    let resp = client.post(&url).json(&request).send().await
        .context("Failed to send login request")?;

    let api_resp: ApiResponse<LoginResponse> = resp.json().await
        .context("Failed to parse login response")?;

    if api_resp.code == 0 {
        let data = api_resp.data.context("Login response missing data")?;
        info!("Login successful: student_id={}, name={}", data.student_id, data.name);
        Ok(data)
    } else {
        Err(anyhow::anyhow!("Login failed: {}", api_resp.message))
    }
}

pub fn is_logged_in(config: &Config) -> bool {
    config.student_id.is_some() && config.auth_token.is_some()
}

pub fn save_login_state(config: &mut Config, student_id: &str, student_no: &str, name: &str, token: &str, expires_at: &str) -> Result<()> {
    config.student_id = Some(student_id.to_string());
    config.student_no = Some(student_no.to_string());
    config.student_name = Some(name.to_string());
    config.auth_token = Some(token.to_string());
    config.token_expires_at = Some(expires_at.to_string());
    config.save()?;
    info!("Login state saved for student {}", student_no);
    Ok(())
}

pub fn clear_login_state(config: &mut Config) -> Result<()> {
    config.student_id = None;
    config.student_no = None;
    config.student_name = None;
    config.auth_token = None;
    config.token_expires_at = None;
    config.save()?;
    info!("Login state cleared");
    Ok(())
}
