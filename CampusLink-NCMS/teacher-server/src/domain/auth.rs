use sqlx::SqlitePool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: String,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn init_default_admin(pool: &SqlitePool) -> Result<()> {
    let exists: Option<String> = sqlx::query_scalar(
        "SELECT id FROM admin_users WHERE username = 'admin'"
    )
    .fetch_optional(pool)
    .await?;

    if exists.is_some() {
        return Ok(());
    }

    let id = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash("admin123", 4)?;

    sqlx::query(
        "INSERT INTO admin_users (id, username, password_hash, display_name, role, status, created_at, updated_at)
         VALUES (?, 'admin', ?, '管理员', 'admin', 'active', datetime('now'), datetime('now'))"
    )
    .bind(&id)
    .bind(&password_hash)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn login(pool: &SqlitePool, req: &LoginRequest) -> Result<LoginResponse> {
    let user = sqlx::query_as::<_, AdminUser>(
        "SELECT * FROM admin_users WHERE username = ? AND status = 'active'"
    )
    .bind(&req.username)
    .fetch_optional(pool)
    .await?;

    let user = user.ok_or_else(|| anyhow::anyhow!("用户名或密码错误"))?;

    bcrypt::verify(&req.password, &user.password_hash)
        .map_err(|_| anyhow::anyhow!("用户名或密码错误"))?;

    let now = Utc::now();
    let exp = now + chrono::Duration::hours(8);
    let claims = Claims {
        sub: user.id.clone(),
        username: user.username.clone(),
        role: user.role.clone(),
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
    };

    let secret = get_jwt_secret(pool).await?;
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )?;

    sqlx::query("UPDATE admin_users SET last_login_at = datetime('now'), updated_at = datetime('now') WHERE id = ?")
        .bind(&user.id)
        .execute(pool)
        .await?;

    Ok(LoginResponse {
        token,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
        expires_at: exp.timestamp(),
    })
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )?;
    Ok(token_data.claims)
}

async fn get_jwt_secret(pool: &SqlitePool) -> Result<String> {
    let secret: Option<String> = sqlx::query_scalar(
        "SELECT config_value FROM system_configs WHERE config_key = 'jwt_secret'"
    )
    .fetch_optional(pool)
    .await?
    .flatten();

    if let Some(s) = secret {
        return Ok(s);
    }

    let secret = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO system_configs (id, config_key, config_value, scope, description, updated_at)
           VALUES (?, 'jwt_secret', ?, 'security', 'JWT signing secret', datetime('now'))"#
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&secret)
    .execute(pool)
    .await?;

    Ok(secret)
}
