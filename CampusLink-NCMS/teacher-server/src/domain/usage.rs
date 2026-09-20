use sqlx::{SqlitePool, FromRow, QueryBuilder, Sqlite};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::time_util::{to_rfc3339, to_rfc3339_opt};

#[derive(Debug, Clone, FromRow)]
struct UsageJoinedRow {
    pub id: String,
    pub device_id: String,
    pub device_name: Option<String>,
    pub class_id: Option<String>,
    pub class_name: Option<String>,
    pub seat_no: Option<String>,
    pub student_id: Option<String>,
    pub student_no: Option<String>,
    pub student_name: Option<String>,
    pub user_name: String,
    pub mode: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_seconds: Option<i64>,
    pub inspection_ok: i64,
    pub inspection_summary: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageRecord {
    pub id: String,
    pub device_id: String,
    pub device_name: Option<String>,
    pub class_id: Option<String>,
    pub class_name: Option<String>,
    pub seat_no: Option<String>,
    pub student_id: Option<String>,
    pub student_no: Option<String>,
    pub student_name: Option<String>,
    pub user_name: String,
    pub mode: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_seconds: Option<i64>,
    pub inspection_ok: bool,
    pub inspection_summary: Option<String>,
}

impl From<UsageJoinedRow> for UsageRecord {
    fn from(row: UsageJoinedRow) -> Self {
        Self {
            id: row.id,
            device_id: row.device_id,
            device_name: row.device_name,
            class_id: row.class_id,
            class_name: row.class_name,
            seat_no: row.seat_no,
            student_id: row.student_id,
            student_no: row.student_no,
            student_name: row.student_name,
            user_name: row.user_name,
            mode: row.mode,
            start_time: to_rfc3339(&row.start_time),
            end_time: to_rfc3339_opt(row.end_time.as_deref()),
            duration_seconds: row.duration_seconds,
            inspection_ok: row.inspection_ok != 0,
            inspection_summary: row.inspection_summary,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageQuery {
    pub device_id: Option<String>,
    pub class_id: Option<String>,
    pub student_keyword: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

pub struct StartSessionParams<'a> {
    pub device_id: &'a str,
    pub class_id: Option<&'a str>,
    pub seat_no: Option<&'a str>,
    pub student_id: Option<&'a str>,
    pub user_name: &'a str,
    pub mode: &'a str,
    pub inspection_ok: bool,
    pub inspection_summary: Option<&'a str>,
}

const SELECT_JOINED: &str = r#"
    SELECT u.id, u.device_id, d.device_name, u.class_id, c.name AS class_name,
           u.seat_no, u.student_id, s.student_no, s.name AS student_name,
           u.user_name, u.mode, u.start_time, u.end_time, u.duration_seconds,
           u.inspection_ok, u.inspection_summary
    FROM device_usage_records u
    LEFT JOIN student_devices d ON d.id = u.device_id
    LEFT JOIN classes c ON c.id = u.class_id
    LEFT JOIN students s ON s.id = u.student_id
"#;

/// 关闭某设备当前未结束的使用会话，返回被关闭的会话 id。
pub async fn close_open_session(pool: &SqlitePool, device_id: &str) -> Result<Option<String>> {
    let open_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM device_usage_records WHERE device_id = ? AND end_time IS NULL ORDER BY start_time DESC LIMIT 1"
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    let Some(id) = open_id else {
        return Ok(None);
    };

    sqlx::query(
        r#"UPDATE device_usage_records
           SET end_time = datetime('now'),
               duration_seconds = MAX(0, CAST((julianday(datetime('now')) - julianday(start_time)) * 86400 AS INTEGER))
           WHERE id = ? AND end_time IS NULL"#
    )
    .bind(&id)
    .execute(pool)
    .await?;

    Ok(Some(id))
}

/// 开启一条使用会话。开启前先关闭该设备已有的未结束会话，保证同一设备至多一条未结束会话。
pub async fn start_session(pool: &SqlitePool, params: StartSessionParams<'_>) -> Result<String> {
    close_open_session(pool, params.device_id).await?;

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO device_usage_records
           (id, device_id, class_id, seat_no, student_id, user_name, mode, start_time,
            inspection_ok, inspection_summary, created_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'), ?, ?, datetime('now'))"#
    )
    .bind(&id)
    .bind(params.device_id)
    .bind(params.class_id)
    .bind(params.seat_no)
    .bind(params.student_id)
    .bind(params.user_name)
    .bind(params.mode)
    .bind(if params.inspection_ok { 1 } else { 0 })
    .bind(params.inspection_summary)
    .execute(pool)
        .await?;

    Ok(id)
}

async fn find_open_session(
    pool: &SqlitePool,
    device_id: &str,
) -> Result<Option<(String, String, Option<String>, String)>> {
    Ok(sqlx::query_as(
        r#"SELECT id, user_name, student_id, mode
           FROM device_usage_records
           WHERE device_id = ? AND end_time IS NULL
           ORDER BY start_time DESC LIMIT 1"#
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?)
}

/// 同一设备、同一使用人、同一模式的未结束会话直接复用，避免重复签到拆成多条。
pub async fn reuse_or_start_session(pool: &SqlitePool, params: StartSessionParams<'_>) -> Result<String> {
    if let Some((id, user_name, student_id, mode)) = find_open_session(pool, params.device_id).await? {
        let same_user = user_name == params.user_name
            && student_id.as_deref() == params.student_id
            && mode == params.mode;
        if same_user {
            return Ok(id);
        }
    }
    start_session(pool, params).await
}

/// 考试模式连接后自动登记使用记录；已有考试会话则复用。
pub async fn ensure_exam_session(pool: &SqlitePool, device_id: &str) -> Result<Option<String>> {
    if let Some((id, _, _, mode)) = find_open_session(pool, device_id).await? {
        if mode == "exam" {
            return Ok(Some(id));
        }
    }

    let device: Option<(Option<String>, Option<String>, String)> = sqlx::query_as(
        "SELECT class_id, seat_no, COALESCE(device_name, '') FROM student_devices WHERE id = ?"
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    let Some((class_id, seat_no, device_name)) = device else {
        return Ok(None);
    };
    let user_name = if device_name.trim().is_empty() {
        "考试"
    } else {
        device_name.as_str()
    };

    let id = start_session(
        pool,
        StartSessionParams {
            device_id,
            class_id: class_id.as_deref(),
            seat_no: seat_no.as_deref(),
            student_id: None,
            user_name,
            mode: "exam",
            inspection_ok: true,
            inspection_summary: Some("考试模式自动登记"),
        },
    )
    .await?;
    Ok(Some(id))
}

/// 教师手动结束会话，重复结束返回当前状态而不覆盖结束时间。
pub async fn end_session(pool: &SqlitePool, id: &str) -> Result<UsageRecord> {
    sqlx::query(
        r#"UPDATE device_usage_records
           SET end_time = datetime('now'),
               duration_seconds = MAX(0, CAST((julianday(datetime('now')) - julianday(start_time)) * 86400 AS INTEGER))
           WHERE id = ? AND end_time IS NULL"#
    )
    .bind(id)
    .execute(pool)
    .await?;

    get_usage(pool, id).await
}

pub async fn get_usage(pool: &SqlitePool, id: &str) -> Result<UsageRecord> {
    let sql = format!("{} WHERE u.id = ?", SELECT_JOINED);
    let row = sqlx::query_as::<_, UsageJoinedRow>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("使用记录不存在"))?;
    Ok(row.into())
}

pub async fn list_usage(pool: &SqlitePool, query: &UsageQuery) -> Result<Vec<UsageRecord>> {
    let mut builder = QueryBuilder::<Sqlite>::new(SELECT_JOINED);
    builder.push(" WHERE 1 = 1");

    if let Some(device_id) = query.device_id.as_deref().filter(|v| !v.is_empty()) {
        builder.push(" AND u.device_id = ").push_bind(device_id);
    }
    if let Some(class_id) = query.class_id.as_deref().filter(|v| !v.is_empty()) {
        builder.push(" AND u.class_id = ").push_bind(class_id);
    }
    if let Some(keyword) = query.student_keyword.as_deref().filter(|v| !v.is_empty()) {
        let pattern = format!("%{}%", keyword);
        builder
            .push(" AND (s.student_no LIKE ")
            .push_bind(pattern.clone())
            .push(" OR s.name LIKE ")
            .push_bind(pattern.clone())
            .push(" OR u.user_name LIKE ")
            .push_bind(pattern)
            .push(")");
    }
    if let Some(start_date) = query.start_date.as_deref().filter(|v| !v.is_empty()) {
        builder.push(" AND date(u.start_time, '+8 hours') >= ").push_bind(start_date);
    }
    if let Some(end_date) = query.end_date.as_deref().filter(|v| !v.is_empty()) {
        builder.push(" AND date(u.start_time, '+8 hours') <= ").push_bind(end_date);
    }

    builder.push(" ORDER BY u.start_time DESC");

    let rows = builder
        .build_query_as::<UsageJoinedRow>()
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(UsageRecord::from).collect())
}

pub async fn export_rows(pool: &SqlitePool, query: &UsageQuery) -> Result<Vec<UsageRecord>> {
    list_usage(pool, query).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_support::{register_test_device, setup_pool};

    #[tokio::test]
    async fn ensure_exam_session_reuses_open_exam_record() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-EXAM-1").await;

        let first = ensure_exam_session(&pool, &device_id).await.unwrap();
        let second = ensure_exam_session(&pool, &device_id).await.unwrap();
        assert_eq!(first, second);

        let rows = list_usage(&pool, &UsageQuery::default()).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].mode, "exam");
        assert!(rows[0].end_time.is_none());
    }
}
