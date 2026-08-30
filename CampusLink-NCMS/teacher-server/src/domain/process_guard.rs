use sqlx::SqlitePool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct ProcessGuardPolicyRow {
    pub id: String,
    pub device_id: Option<String>,
    pub process_name: String,
    pub check_interval_seconds: i64,
    pub max_restart_attempts: i64,
    pub enabled: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ProcessGuardPolicy {
    pub id: String,
    pub device_id: Option<String>,
    pub process_name: String,
    pub check_interval_seconds: i64,
    pub max_restart_attempts: i64,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ProcessGuardPolicyRow> for ProcessGuardPolicy {
    fn from(row: ProcessGuardPolicyRow) -> Self {
        ProcessGuardPolicy {
            id: row.id,
            device_id: row.device_id,
            process_name: row.process_name,
            check_interval_seconds: row.check_interval_seconds,
            max_restart_attempts: row.max_restart_attempts,
            enabled: row.enabled.unwrap_or(1) != 0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePolicyRequest {
    pub device_id: Option<String>,
    pub process_name: String,
    pub check_interval_seconds: Option<i64>,
    pub max_restart_attempts: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePolicyRequest {
    pub process_name: Option<String>,
    pub check_interval_seconds: Option<i64>,
    pub max_restart_attempts: Option<i64>,
    pub enabled: Option<bool>,
}

pub async fn create_policy(pool: &SqlitePool, req: &CreatePolicyRequest) -> Result<ProcessGuardPolicy> {
    let id = Uuid::new_v4().to_string();
    let interval = req.check_interval_seconds.unwrap_or(60);
    let max_attempts = req.max_restart_attempts.unwrap_or(3);

    sqlx::query(
        r#"INSERT INTO process_guard_policies (id, device_id, process_name, check_interval_seconds, max_restart_attempts, enabled, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, 1, datetime('now'), datetime('now'))"#
    )
    .bind(&id)
    .bind(&req.device_id)
    .bind(&req.process_name)
    .bind(interval)
    .bind(max_attempts)
    .execute(pool)
    .await?;

    let row = sqlx::query_as::<_, ProcessGuardPolicyRow>(
        "SELECT * FROM process_guard_policies WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

pub async fn update_policy(pool: &SqlitePool, id: &str, req: &UpdatePolicyRequest) -> Result<Option<ProcessGuardPolicy>> {
    let existing = sqlx::query_as::<_, ProcessGuardPolicyRow>(
        "SELECT * FROM process_guard_policies WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let existing = match existing {
        Some(p) => p,
        None => return Ok(None),
    };

    let name = req.process_name.as_deref().unwrap_or(&existing.process_name);
    let interval = req.check_interval_seconds.unwrap_or(existing.check_interval_seconds);
    let attempts = req.max_restart_attempts.unwrap_or(existing.max_restart_attempts);
    let enabled = req.enabled.map(|v| v as i64).unwrap_or(existing.enabled.unwrap_or(1));

    sqlx::query(
        r#"UPDATE process_guard_policies SET process_name = ?, check_interval_seconds = ?, max_restart_attempts = ?, enabled = ?, updated_at = datetime('now')
           WHERE id = ?"#
    )
    .bind(name)
    .bind(interval)
    .bind(attempts)
    .bind(enabled)
    .bind(id)
    .execute(pool)
    .await?;

    let row = sqlx::query_as::<_, ProcessGuardPolicyRow>(
        "SELECT * FROM process_guard_policies WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(Some(row.into()))
}

pub async fn list_policies(pool: &SqlitePool, device_id: Option<&str>) -> Result<Vec<ProcessGuardPolicy>> {
    let rows = if let Some(did) = device_id {
        sqlx::query_as::<_, ProcessGuardPolicyRow>(
            "SELECT * FROM process_guard_policies WHERE (device_id = ? OR device_id IS NULL) AND enabled = 1 ORDER BY created_at DESC"
        )
        .bind(did)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, ProcessGuardPolicyRow>(
            "SELECT * FROM process_guard_policies WHERE enabled = 1 ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?
    };

    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn delete_policy(pool: &SqlitePool, id: &str) -> Result<bool> {
    let result = sqlx::query("DELETE FROM process_guard_policies WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_support::{setup_pool, register_test_device};

    #[tokio::test]
    async fn create_and_list_policies_by_device() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-PG-001").await;

        let global = create_policy(&pool, &CreatePolicyRequest {
            device_id: None,
            process_name: "chrome".to_string(),
            check_interval_seconds: None,
            max_restart_attempts: None,
        }).await.unwrap();
        let scoped = create_policy(&pool, &CreatePolicyRequest {
            device_id: Some(device_id.clone()),
            process_name: "agent".to_string(),
            check_interval_seconds: Some(30),
            max_restart_attempts: Some(5),
        }).await.unwrap();

        assert_eq!(global.check_interval_seconds, 60);
        assert_eq!(global.max_restart_attempts, 3);
        assert_eq!(scoped.check_interval_seconds, 30);
        assert_eq!(scoped.max_restart_attempts, 5);

        let for_device = list_policies(&pool, Some(&device_id)).await.unwrap();
        assert_eq!(for_device.len(), 2);

        let all = list_policies(&pool, None).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn scoped_policy_not_visible_to_other_device() {
        let pool = setup_pool().await;
        let d1 = register_test_device(&pool, "DEV-PG-002").await;
        let d2 = register_test_device(&pool, "DEV-PG-003").await;

        create_policy(&pool, &CreatePolicyRequest {
            device_id: Some(d1.clone()),
            process_name: "scoped".to_string(),
            check_interval_seconds: None,
            max_restart_attempts: None,
        }).await.unwrap();

        let for_d2 = list_policies(&pool, Some(&d2)).await.unwrap();
        assert_eq!(for_d2.len(), 0);
    }

    #[tokio::test]
    async fn update_policy_changes_fields() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-PG-004").await;

        let policy = create_policy(&pool, &CreatePolicyRequest {
            device_id: Some(device_id),
            process_name: "app".to_string(),
            check_interval_seconds: None,
            max_restart_attempts: None,
        }).await.unwrap();

        let updated = update_policy(&pool, &policy.id, &UpdatePolicyRequest {
            process_name: Some("newapp".to_string()),
            check_interval_seconds: Some(10),
            max_restart_attempts: None,
            enabled: Some(false),
        }).await.unwrap().unwrap();

        assert_eq!(updated.process_name, "newapp");
        assert_eq!(updated.check_interval_seconds, 10);
        assert!(!updated.enabled);
    }

    #[tokio::test]
    async fn delete_policy_returns_existence() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-PG-005").await;

        let policy = create_policy(&pool, &CreatePolicyRequest {
            device_id: Some(device_id),
            process_name: "tmp".to_string(),
            check_interval_seconds: None,
            max_restart_attempts: None,
        }).await.unwrap();

        assert!(delete_policy(&pool, &policy.id).await.unwrap());
        assert!(!delete_policy(&pool, &policy.id).await.unwrap());
    }
}
