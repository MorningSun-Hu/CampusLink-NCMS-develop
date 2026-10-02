use anyhow::Result;
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

const BUILTIN_ADMIN: &str = "admin";
const MIN_PASSWORD_LEN: usize = 4;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserView {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub status: String,
    pub last_login_at: Option<String>,
    pub created_at: String,
}

fn normalize_username(username: &str) -> Result<String> {
    let username = username.trim();
    if username.is_empty() {
        anyhow::bail!("用户名不能为空");
    }
    if username.eq_ignore_ascii_case(BUILTIN_ADMIN) {
        anyhow::bail!("内置管理员账号不可创建");
    }
    Ok(username.to_string())
}

fn validate_password(password: &str) -> Result<()> {
    if password.len() < MIN_PASSWORD_LEN {
        anyhow::bail!("新密码长度不得少于 {} 位", MIN_PASSWORD_LEN);
    }
    Ok(())
}

async fn username_of(pool: &SqlitePool, id: &str) -> Result<String> {
    sqlx::query_scalar::<_, String>("SELECT username FROM admin_users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("账号不存在"))
}

fn view_columns() -> &'static str {
    "id, username, display_name, role, status, last_login_at, created_at"
}

pub async fn list_users(pool: &SqlitePool) -> Result<Vec<AdminUserView>> {
    let query = format!(
        "SELECT {} FROM admin_users ORDER BY (username = '{}') DESC, created_at ASC",
        view_columns(),
        BUILTIN_ADMIN
    );
    let users = sqlx::query_as::<_, AdminUserView>(&query)
        .fetch_all(pool)
        .await?;
    Ok(users)
}

pub async fn create_teacher(
    pool: &SqlitePool,
    username: &str,
    password: &str,
    display_name: &str,
) -> Result<AdminUserView> {
    let username = normalize_username(username)?;
    validate_password(password)?;

    let exists: Option<String> =
        sqlx::query_scalar("SELECT id FROM admin_users WHERE username = ?")
            .bind(&username)
            .fetch_optional(pool)
            .await?;
    if exists.is_some() {
        anyhow::bail!("用户名已存在");
    }

    let id = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash(password, 4)?;
    let display_name = display_name.trim();

    sqlx::query(
        "INSERT INTO admin_users (id, username, password_hash, display_name, role, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'teacher', 'active', datetime('now'), datetime('now'))",
    )
    .bind(&id)
    .bind(&username)
    .bind(&password_hash)
    .bind(display_name)
    .execute(pool)
    .await?;

    get_user(pool, &id).await
}

async fn get_user(pool: &SqlitePool, id: &str) -> Result<AdminUserView> {
    let query = format!("SELECT {} FROM admin_users WHERE id = ?", view_columns());
    sqlx::query_as::<_, AdminUserView>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("账号不存在"))
}

pub async fn update_user(
    pool: &SqlitePool,
    id: &str,
    display_name: Option<String>,
    status: Option<String>,
) -> Result<AdminUserView> {
    let username = username_of(pool, id).await?;
    if username == BUILTIN_ADMIN {
        anyhow::bail!("内置管理员账号不可修改");
    }

    if let Some(status) = status.as_deref() {
        if !matches!(status, "active" | "disabled") {
            anyhow::bail!("账号状态非法");
        }
    }

    if let Some(display_name) = display_name {
        sqlx::query("UPDATE admin_users SET display_name = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(display_name.trim())
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(status) = status {
        sqlx::query("UPDATE admin_users SET status = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?;
    }

    get_user(pool, id).await
}

pub async fn delete_user(pool: &SqlitePool, id: &str) -> Result<()> {
    let username = username_of(pool, id).await?;
    if username == BUILTIN_ADMIN {
        anyhow::bail!("内置管理员账号不可删除");
    }

    sqlx::query("DELETE FROM admin_users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn reset_teacher_password(pool: &SqlitePool, id: &str, new_password: &str) -> Result<()> {
    let username = username_of(pool, id).await?;
    if username == BUILTIN_ADMIN {
        anyhow::bail!("内置管理员密码仅可由本人修改");
    }
    validate_password(new_password)?;

    let password_hash = bcrypt::hash(new_password, 4)?;
    sqlx::query("UPDATE admin_users SET password_hash = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(&password_hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn change_own_password(
    pool: &SqlitePool,
    user_id: &str,
    old_password: &str,
    new_password: &str,
) -> Result<()> {
    validate_password(new_password)?;

    let password_hash = sqlx::query_scalar::<_, String>(
        "SELECT password_hash FROM admin_users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("账号不存在"))?;

    let verified = bcrypt::verify(old_password, &password_hash)
        .map_err(|_| anyhow::anyhow!("当前密码不正确"))?;
    if !verified {
        anyhow::bail!("当前密码不正确");
    }

    let new_hash = bcrypt::hash(new_password, 4)?;
    sqlx::query("UPDATE admin_users SET password_hash = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(&new_hash)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth::{init_default_admin, login, LoginRequest};
    use crate::domain::test_support::setup_pool;

    async fn seed(pool: &SqlitePool) {
        init_default_admin(pool).await.unwrap();
    }

    #[tokio::test]
    async fn create_teacher_then_login_uses_teacher_role() {
        let pool = setup_pool().await;
        seed(&pool).await;

        let created = create_teacher(&pool, "teacher-a", "pass1234", "张老师").await.unwrap();
        assert_eq!(created.role, "teacher");
        assert_eq!(created.status, "active");

        let resp = login(
            &pool,
            &LoginRequest { username: "teacher-a".to_string(), password: "pass1234".to_string() },
        )
        .await
        .unwrap();
        assert_eq!(resp.role, "teacher");
    }

    #[tokio::test]
    async fn create_teacher_rejects_duplicate_and_admin_name() {
        let pool = setup_pool().await;
        seed(&pool).await;

        create_teacher(&pool, "teacher-a", "pass1234", "甲").await.unwrap();
        assert!(create_teacher(&pool, "teacher-a", "pass1234", "乙").await.is_err());
        assert!(create_teacher(&pool, "admin", "pass1234", "伪管理员").await.is_err());
    }

    #[tokio::test]
    async fn list_users_hides_password_hash() {
        let pool = setup_pool().await;
        seed(&pool).await;
        create_teacher(&pool, "teacher-a", "pass1234", "张老师").await.unwrap();

        let users = list_users(&pool).await.unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "admin");
    }

    #[tokio::test]
    async fn disabled_teacher_cannot_login() {
        let pool = setup_pool().await;
        seed(&pool).await;
        let teacher = create_teacher(&pool, "teacher-a", "pass1234", "张老师").await.unwrap();

        update_user(&pool, &teacher.id, None, Some("disabled".to_string())).await.unwrap();

        let result = login(
            &pool,
            &LoginRequest { username: "teacher-a".to_string(), password: "pass1234".to_string() },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_and_update_reject_builtin_admin() {
        let pool = setup_pool().await;
        seed(&pool).await;

        let admin_id: String =
            sqlx::query_scalar("SELECT id FROM admin_users WHERE username = 'admin'")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert!(delete_user(&pool, &admin_id).await.is_err());
        assert!(update_user(&pool, &admin_id, Some("新名字".to_string()), None).await.is_err());
        assert!(update_user(&pool, &admin_id, None, Some("disabled".to_string())).await.is_err());
    }

    #[tokio::test]
    async fn admin_can_reset_teacher_password_but_not_admin() {
        let pool = setup_pool().await;
        seed(&pool).await;
        let teacher = create_teacher(&pool, "teacher-a", "pass1234", "张老师").await.unwrap();

        reset_teacher_password(&pool, &teacher.id, "newpass1").await.unwrap();
        let resp = login(
            &pool,
            &LoginRequest { username: "teacher-a".to_string(), password: "newpass1".to_string() },
        )
        .await;
        assert!(resp.is_ok());

        let admin_id: String =
            sqlx::query_scalar("SELECT id FROM admin_users WHERE username = 'admin'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(reset_teacher_password(&pool, &admin_id, "hacked1").await.is_err());
    }

    #[tokio::test]
    async fn change_own_password_verifies_old_password() {
        let pool = setup_pool().await;
        seed(&pool).await;
        let teacher = create_teacher(&pool, "teacher-a", "pass1234", "张老师").await.unwrap();

        assert!(change_own_password(&pool, &teacher.id, "wrong", "newpass1").await.is_err());
        assert!(change_own_password(&pool, &teacher.id, "pass1234", "abc").await.is_err());

        change_own_password(&pool, &teacher.id, "pass1234", "newpass1").await.unwrap();
        assert!(login(
            &pool,
            &LoginRequest { username: "teacher-a".to_string(), password: "pass1234".to_string() },
        )
        .await
        .is_err());
        assert!(login(
            &pool,
            &LoginRequest { username: "teacher-a".to_string(), password: "newpass1".to_string() },
        )
        .await
        .is_ok());
    }
}
