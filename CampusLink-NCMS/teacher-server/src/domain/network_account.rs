use sqlx::SqlitePool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAccount {
    pub id: String,
    pub account_name: String,
    pub password: String,
    pub device_id: Option<String>,
    pub status: String,
    pub login_url: Option<String>,
    pub logout_url: Option<String>,
    pub auto_login: i32,
    pub auto_logout: i32,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNetworkAccountRequest {
    pub account_name: String,
    pub password: String,
    pub device_id: Option<String>,
    pub login_url: Option<String>,
    pub logout_url: Option<String>,
    pub auto_login: Option<bool>,
    pub auto_logout: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNetworkAccountRequest {
    pub account_name: Option<String>,
    pub password: Option<String>,
    pub device_id: Option<String>,
    pub status: Option<String>,
    pub login_url: Option<String>,
    pub logout_url: Option<String>,
    pub auto_login: Option<bool>,
    pub auto_logout: Option<bool>,
    pub description: Option<String>,
}

pub async fn list_accounts(pool: &SqlitePool) -> Result<Vec<NetworkAccount>> {
    sqlx::query_as::<_, NetworkAccount>("SELECT * FROM network_accounts ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}

pub async fn get_account(pool: &SqlitePool, id: &str) -> Result<NetworkAccount> {
    sqlx::query_as::<_, NetworkAccount>("SELECT * FROM network_accounts WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}

pub async fn create_account(pool: &SqlitePool, req: CreateNetworkAccountRequest) -> Result<NetworkAccount> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO network_accounts (id, account_name, password, device_id, login_url, logout_url, auto_login, auto_logout, description, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
    )
    .bind(&id)
    .bind(&req.account_name)
    .bind(&req.password)
    .bind(&req.device_id)
    .bind(&req.login_url)
    .bind(&req.logout_url)
    .bind(req.auto_login.unwrap_or(false) as i32)
    .bind(req.auto_logout.unwrap_or(false) as i32)
    .bind(&req.description)
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("{}", e))?;
    get_account(pool, &id).await
}

pub async fn update_account(pool: &SqlitePool, id: &str, req: UpdateNetworkAccountRequest) -> Result<NetworkAccount> {
    let existing = get_account(pool, id).await?;

    let account_name = req.account_name.unwrap_or(existing.account_name);
    let password = req.password.unwrap_or(existing.password);
    let device_id = req.device_id.or(existing.device_id);
    let status = req.status.unwrap_or(existing.status);
    let login_url = req.login_url.or(existing.login_url);
    let logout_url = req.logout_url.or(existing.logout_url);
    let auto_login = req.auto_login.map(|v| v as i32).unwrap_or(existing.auto_login);
    let auto_logout = req.auto_logout.map(|v| v as i32).unwrap_or(existing.auto_logout);
    let description = req.description.or(existing.description);

    sqlx::query(
        "UPDATE network_accounts SET account_name=?, password=?, device_id=?, status=?, login_url=?, logout_url=?, auto_login=?, auto_logout=?, description=?, updated_at=datetime('now') WHERE id=?"
    )
    .bind(&account_name)
    .bind(&password)
    .bind(&device_id)
    .bind(&status)
    .bind(&login_url)
    .bind(&logout_url)
    .bind(auto_login)
    .bind(auto_logout)
    .bind(&description)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("{}", e))?;
    get_account(pool, id).await
}

pub async fn delete_account(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM network_accounts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}
