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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StudentLoginRow {
    pub id: String,
    pub student_no: String,
    pub name: String,
    pub password_hash: String,
    pub status: String,
    pub password_set: i64,
}

#[derive(Debug, Deserialize)]
pub struct StudentLoginRequest {
    /// 学号或姓名
    pub student_no: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct StudentLoginResponse {
    pub token: String,
    pub student_id: String,
    pub student_no: String,
    pub name: String,
    pub password_set: bool,
    pub expires_at: i64,
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

    let verified = bcrypt::verify(&req.password, &user.password_hash)
        .map_err(|_| anyhow::anyhow!("用户名或密码错误"))?;
    if !verified {
        return Err(anyhow::anyhow!("用户名或密码错误"));
    }

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

pub async fn student_login(pool: &SqlitePool, req: &StudentLoginRequest) -> Result<StudentLoginResponse> {
    let student = sqlx::query_as::<_, StudentLoginRow>(
        "SELECT id, student_no, name, password_hash, status, password_set
         FROM students WHERE student_no = ? OR name = ? LIMIT 1"
    )
    .bind(&req.student_no)
    .bind(&req.student_no)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("学号或密码错误"))?;

    if student.status != "active" {
        return Err(anyhow::anyhow!("该学生账号已停用"));
    }

    let verified = bcrypt::verify(&req.password, &student.password_hash)
        .map_err(|_| anyhow::anyhow!("学号或密码错误"))?;
    if !verified {
        return Err(anyhow::anyhow!("学号或密码错误"));
    }

    let now = Utc::now();
    let exp = now + chrono::Duration::hours(8);
    let claims = Claims {
        sub: student.id.clone(),
        username: student.student_no.clone(),
        role: "student".to_string(),
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
    };

    let secret = get_jwt_secret(pool).await?;
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(StudentLoginResponse {
        token,
        student_id: student.id,
        student_no: student.student_no,
        name: student.name,
        password_set: student.password_set != 0,
        expires_at: exp.timestamp(),
    })
}

pub async fn change_student_password(
    pool: &SqlitePool,
    student_id: &str,
    old_password: &str,
    new_password: &str,
) -> Result<()> {
    if new_password.len() < 4 {
        anyhow::bail!("新密码长度不得少于 4 位");
    }

    let hash: Option<String> = sqlx::query_scalar("SELECT password_hash FROM students WHERE id = ?")
        .bind(student_id)
        .fetch_optional(pool)
        .await?
        .flatten();

    let hash = hash.ok_or_else(|| anyhow::anyhow!("学生不存在"))?;

    let verified = bcrypt::verify(old_password, &hash)
        .map_err(|_| anyhow::anyhow!("原密码错误"))?;
    if !verified {
        return Err(anyhow::anyhow!("原密码错误"));
    }

    crate::domain::student::set_student_password(pool, student_id, new_password).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_support::setup_pool;

    #[tokio::test]
    async fn login_issues_token_and_verify_passes() {
        let pool = setup_pool().await;
        init_default_admin(&pool).await.unwrap();

        let resp = login(&pool, &LoginRequest {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        }).await.unwrap();

        assert!(!resp.token.is_empty());
        assert_eq!(resp.username, "admin");
        assert_eq!(resp.role, "admin");
        assert!(resp.expires_at > chrono::Utc::now().timestamp());

        let claims = verify_token(&resp.token, &get_jwt_secret(&pool).await.unwrap()).unwrap();
        assert_eq!(claims.username, "admin");
        assert_eq!(claims.role, "admin");
    }

    #[tokio::test]
    async fn login_rejects_wrong_password() {
        let pool = setup_pool().await;
        init_default_admin(&pool).await.unwrap();

        let result = login(&pool, &LoginRequest {
            username: "admin".to_string(),
            password: "wrong-password".to_string(),
        }).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn verify_token_rejects_wrong_secret() {
        let pool = setup_pool().await;
        init_default_admin(&pool).await.unwrap();
        let resp = login(&pool, &LoginRequest {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        }).await.unwrap();

        assert!(verify_token(&resp.token, "wrong-secret").is_err());
    }

    #[test]
    fn token_expired_is_rejected() {
        let secret = "test-secret";
        let now = Utc::now();
        let claims = Claims {
            sub: "u1".to_string(),
            username: "admin".to_string(),
            role: "admin".to_string(),
            iat: (now - chrono::Duration::hours(2)).timestamp() as usize,
            exp: (now - chrono::Duration::hours(1)).timestamp() as usize,
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        ).unwrap();

        assert!(verify_token(&token, secret).is_err());
    }

    #[tokio::test]
    async fn student_login_verifies_account_and_issues_token() {
        let pool = setup_pool().await;
        let password_hash = bcrypt::hash("stu123", 4).unwrap();
        sqlx::query(
            "INSERT INTO students (id, student_no, name, password_hash, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, 'active', datetime('now'), datetime('now'))"
        )
        .bind("stu-1")
        .bind("2026001")
        .bind("张三")
        .bind(&password_hash)
        .execute(&pool)
        .await
        .unwrap();

        let resp = student_login(&pool, &StudentLoginRequest {
            student_no: "2026001".to_string(),
            password: "stu123".to_string(),
        }).await.unwrap();

        assert_eq!(resp.student_id, "stu-1");
        assert_eq!(resp.name, "张三");
        assert!(!resp.token.is_empty());
        assert!(resp.expires_at > chrono::Utc::now().timestamp());

        let claims = verify_token(&resp.token, &get_jwt_secret(&pool).await.unwrap()).unwrap();
        assert_eq!(claims.role, "student");
        assert_eq!(claims.username, "2026001");
    }

    #[tokio::test]
    async fn student_login_by_name_and_password_change_flow() {
        let pool = setup_pool().await;
        let password_hash = bcrypt::hash("init123", 4).unwrap();
        sqlx::query(
            "INSERT INTO students (id, student_no, name, password_hash, password_set, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, 0, 'active', datetime('now'), datetime('now'))"
        )
        .bind("stu-pwd")
        .bind("2026009")
        .bind("王五")
        .bind(&password_hash)
        .execute(&pool)
        .await
        .unwrap();

        // 初始密码登录，password_set 为 false
        let resp = student_login(&pool, &StudentLoginRequest {
            student_no: "王五".to_string(),
            password: "init123".to_string(),
        }).await.unwrap();
        assert!(!resp.password_set);

        // 首次修改密码后 password_set 变为 true
        change_student_password(&pool, "stu-pwd", "init123", "custom888").await.unwrap();
        let resp = student_login(&pool, &StudentLoginRequest {
            student_no: "2026009".to_string(),
            password: "custom888".to_string(),
        }).await.unwrap();
        assert!(resp.password_set);

        // 原密码错误时拒绝改密
        assert!(change_student_password(&pool, "stu-pwd", "wrong", "another1").await.is_err());
        // 新密码过短时拒绝
        assert!(change_student_password(&pool, "stu-pwd", "custom888", "123").await.is_err());
    }

    #[tokio::test]
    async fn student_login_rejects_wrong_password() {
        let pool = setup_pool().await;
        let password_hash = bcrypt::hash("stu123", 4).unwrap();
        sqlx::query(
            "INSERT INTO students (id, student_no, name, password_hash, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, 'active', datetime('now'), datetime('now'))"
        )
        .bind("stu-2")
        .bind("2026002")
        .bind("李四")
        .bind(&password_hash)
        .execute(&pool)
        .await
        .unwrap();

        let result = student_login(&pool, &StudentLoginRequest {
            student_no: "2026002".to_string(),
            password: "wrong".to_string(),
        }).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn student_login_rejects_unknown_student() {
        let pool = setup_pool().await;
        let result = student_login(&pool, &StudentLoginRequest {
            student_no: "9999999".to_string(),
            password: "whatever".to_string(),
        }).await;

        assert!(result.is_err());
    }
}
