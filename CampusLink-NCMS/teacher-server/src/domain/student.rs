use sqlx::{SqlitePool, FromRow, Row};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StudentRow {
    pub id: String,
    pub student_no: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub seat_no: Option<String>,
    pub status: String,
    pub class_id: Option<String>,
    pub password_set: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentResponse {
    pub id: String,
    pub student_no: String,
    pub name: String,
    pub seat_no: Option<String>,
    pub status: String,
    pub class_id: Option<String>,
    pub class_name: Option<String>,
    pub password_set: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<StudentRow> for StudentResponse {
    fn from(row: StudentRow) -> Self {
        Self {
            id: row.id,
            student_no: row.student_no,
            name: row.name,
            seat_no: row.seat_no,
            status: row.status,
            class_id: row.class_id,
            class_name: None,
            password_set: row.password_set != 0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStudentRequest {
    pub student_no: Option<String>,
    pub name: String,
    pub password: Option<String>,
    pub seat_no: Option<String>,
    pub class_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStudentRequest {
    pub student_no: Option<String>,
    pub name: Option<String>,
    pub password: Option<String>,
    pub seat_no: Option<String>,
    pub status: Option<String>,
    pub class_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub keyword: Option<String>,
    pub class_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentListResponse {
    pub students: Vec<StudentResponse>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

pub fn generate_student_no() -> String {
    let suffix = Uuid::new_v4().simple().to_string();
    format!("S{}", &suffix[..8].to_uppercase())
}

pub async fn find_class_id_by_name(pool: &SqlitePool, name: &str) -> Result<Option<String>> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let id = sqlx::query_scalar::<_, String>("SELECT id FROM classes WHERE name = ?")
        .bind(trimmed)
        .fetch_optional(pool)
        .await?;
    Ok(id)
}

pub async fn require_class_and_existing_seat(pool: &SqlitePool, class_id: &str, seat_no: &str) -> Result<(String, String)> {
    let class_id = class_id.trim().to_string();
    let seat_no = seat_no.trim().to_string();
    if class_id.is_empty() {
        anyhow::bail!("班级为必填项");
    }
    if seat_no.is_empty() {
        anyhow::bail!("座位号为必填项");
    }

    let class_exists: Option<String> = sqlx::query_scalar("SELECT id FROM classes WHERE id = ?")
        .bind(&class_id)
        .fetch_optional(pool)
        .await?;
    if class_exists.is_none() {
        anyhow::bail!("班级不存在");
    }

    let seat_exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM student_devices WHERE class_id = ? AND TRIM(COALESCE(seat_no, '')) = ?"
    )
    .bind(&class_id)
    .bind(&seat_no)
    .fetch_optional(pool)
    .await?;
    if seat_exists.is_none() {
        anyhow::bail!("座位号 {} 在该班级中不存在，请先在班级管理中为设备分配该座位", seat_no);
    }

    Ok((class_id, seat_no))
}

fn map_student_constraint(err: sqlx::Error) -> anyhow::Error {
    let text = err.to_string();
    if text.contains("UNIQUE") && text.contains("seat") {
        anyhow::anyhow!("该班级座位号已被其他学生占用")
    } else if text.contains("UNIQUE") {
        anyhow::anyhow!("学号已存在")
    } else {
        anyhow::Error::from(err)
    }
}

pub async fn create_student(pool: &SqlitePool, req: CreateStudentRequest) -> Result<StudentResponse> {
    let id = Uuid::new_v4().to_string();
    let student_no = req
        .student_no
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(generate_student_no);
    let initial_password = req.password.filter(|s| !s.is_empty()).unwrap_or_else(|| "123456".to_string());
    let password_hash = bcrypt::hash(&initial_password, 4)?;

    let class_id = req.class_id.as_deref().unwrap_or("");
    let seat_no = req.seat_no.as_deref().unwrap_or("");
    let (class_id, seat_no) = require_class_and_existing_seat(pool, class_id, seat_no).await?;

    sqlx::query(
        r#"INSERT INTO students (id, student_no, name, password_hash, seat_no, status, class_id, password_set, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, 'active', ?, 0, datetime('now'), datetime('now'))"#
    )
    .bind(&id)
    .bind(&student_no)
    .bind(&req.name)
    .bind(&password_hash)
    .bind(&seat_no)
    .bind(&class_id)
    .execute(pool)
    .await
    .map_err(map_student_constraint)?;

    get_student_by_id(pool, &id).await
}

pub async fn get_student_by_id(pool: &SqlitePool, id: &str) -> Result<StudentResponse> {
    let row = sqlx::query_as::<_, StudentRow>(
        "SELECT * FROM students WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    let mut response: StudentResponse = row.into();
    response.class_name = class_name_of(pool, response.class_id.as_deref()).await?;
    Ok(response)
}

pub async fn class_name_of(pool: &SqlitePool, class_id: Option<&str>) -> Result<Option<String>> {
    match class_id {
        Some(cid) => {
            let name: Option<String> = sqlx::query_scalar("SELECT name FROM classes WHERE id = ?")
                .bind(cid)
                .fetch_optional(pool)
                .await?;
            Ok(name)
        }
        None => Ok(None),
    }
}

pub async fn find_student_by_login(pool: &SqlitePool, login: &str) -> Result<Vec<StudentRow>> {
    let rows = sqlx::query_as::<_, StudentRow>(
        "SELECT * FROM students WHERE student_no = ? OR name = ? ORDER BY student_no ASC"
    )
    .bind(login)
    .bind(login)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn set_student_password(pool: &SqlitePool, id: &str, password: &str) -> Result<()> {
    let password_hash = bcrypt::hash(password, 4)?;
    sqlx::query(
        "UPDATE students SET password_hash = ?, password_set = 1, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(&password_hash)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn reset_student_password(pool: &SqlitePool, id: &str, initial_password: &str) -> Result<()> {
    let password_hash = bcrypt::hash(initial_password, 4)?;
    sqlx::query(
        "UPDATE students SET password_hash = ?, password_set = 0, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(&password_hash)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn reset_class_passwords(pool: &SqlitePool, class_id: &str, initial_password: &str) -> Result<u64> {
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM students WHERE class_id = ?")
        .bind(class_id)
        .fetch_all(pool)
        .await?;

    let password_hash = bcrypt::hash(initial_password, 4)?;
    for id in &ids {
        sqlx::query(
            "UPDATE students SET password_hash = ?, password_set = 0, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(&password_hash)
        .bind(id)
        .execute(pool)
        .await?;
    }
    Ok(ids.len() as u64)
}

pub async fn list_students(pool: &SqlitePool, query: StudentListQuery) -> Result<StudentListResponse> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = ((page - 1) * page_size) as i64;

    let mut conditions: Vec<String> = Vec::new();
    if let Some(ref kw) = query.keyword {
        if !kw.is_empty() {
            conditions.push("(name LIKE ? OR student_no LIKE ?)".to_string());
        }
    }
    if let Some(ref cid) = query.class_id {
        if !cid.is_empty() {
            conditions.push("class_id = ?".to_string());
        }
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let keyword = query.keyword.as_ref().filter(|k| !k.is_empty()).cloned();
    let class_id = query.class_id.as_ref().filter(|c| !c.is_empty()).cloned();

    let count_sql = format!("SELECT COUNT(*) as cnt FROM students {}", where_clause);
    let mut count_query = sqlx::query(&count_sql);
    if let Some(ref kw) = keyword {
        count_query = count_query.bind(format!("%{}%", kw)).bind(format!("%{}%", kw));
    }
    if let Some(ref cid) = class_id {
        count_query = count_query.bind(cid);
    }
    let total: i64 = count_query.fetch_one(pool).await?.get("cnt");

    let list_sql = format!(
        "SELECT * FROM students {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_clause
    );

    let mut list_query = sqlx::query_as::<_, StudentRow>(&list_sql);
    if let Some(ref kw) = keyword {
        list_query = list_query.bind(format!("%{}%", kw)).bind(format!("%{}%", kw));
    }
    if let Some(ref cid) = class_id {
        list_query = list_query.bind(cid);
    }
    let rows = list_query.bind(page_size as i64).bind(offset).fetch_all(pool).await?;

    let mut students = Vec::with_capacity(rows.len());
    for row in rows {
        let mut response: StudentResponse = row.into();
        response.class_name = class_name_of(pool, response.class_id.as_deref()).await?;
        students.push(response);
    }

    Ok(StudentListResponse {
        students,
        total,
        page,
        page_size,
    })
}

pub async fn update_student(pool: &SqlitePool, id: &str, req: UpdateStudentRequest) -> Result<StudentResponse> {
    let existing = sqlx::query_as::<_, StudentRow>("SELECT * FROM students WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    if existing.is_none() {
        return Err(anyhow::anyhow!("Student not found"));
    }

    let existing = existing.unwrap();

    let student_no = req.student_no.unwrap_or(existing.student_no);
    let name = req.name.unwrap_or(existing.name);
    let seat_no = req.seat_no.or(existing.seat_no);
    let status = req.status.unwrap_or(existing.status);
    let class_id = req.class_id.or(existing.class_id);
    let (class_id, seat_no) = require_class_and_existing_seat(
        pool,
        class_id.as_deref().unwrap_or(""),
        seat_no.as_deref().unwrap_or(""),
    ).await?;

    if let Some(password) = req.password {
        let password_hash = bcrypt::hash(&password, 4)?;
        sqlx::query(
            "UPDATE students SET student_no = ?, name = ?, password_hash = ?, password_set = 0, seat_no = ?, status = ?, class_id = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(&student_no)
        .bind(&name)
        .bind(&password_hash)
        .bind(&seat_no)
        .bind(&status)
        .bind(&class_id)
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_student_constraint)?;
    } else {
        sqlx::query(
            "UPDATE students SET student_no = ?, name = ?, seat_no = ?, status = ?, class_id = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(&student_no)
        .bind(&name)
        .bind(&seat_no)
        .bind(&status)
        .bind(&class_id)
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_student_constraint)?;
    }

    get_student_by_id(pool, id).await
}

pub async fn delete_student(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM students WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_all_students(pool: &SqlitePool) -> Result<Vec<StudentRow>> {
    let rows = sqlx::query_as::<_, StudentRow>(
        "SELECT * FROM students ORDER BY student_no ASC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn import_student(pool: &SqlitePool, student_no: &str, name: &str, password: &str, seat_no: &str, class_id: &str) -> Result<()> {
    let (class_id, seat_no) = require_class_and_existing_seat(pool, class_id, seat_no).await?;
    insert_imported_student(pool, student_no, name, password, &seat_no, &class_id).await
}

pub async fn insert_imported_student<'e, E>(
    executor: E,
    student_no: &str,
    name: &str,
    password: &str,
    seat_no: &str,
    class_id: &str,
) -> Result<()>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let id = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash(password, 4)?;

    sqlx::query(
        "INSERT INTO students (id, student_no, name, password_hash, seat_no, status, class_id, password_set, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 'active', ?, 0, datetime('now'), datetime('now'))
         ON CONFLICT(student_no) DO UPDATE SET
            name = excluded.name,
            password_hash = excluded.password_hash,
            seat_no = excluded.seat_no,
            class_id = excluded.class_id,
            password_set = 0,
            updated_at = datetime('now')"
    )
    .bind(&id)
    .bind(student_no)
    .bind(name)
    .bind(&password_hash)
    .bind(seat_no)
    .bind(class_id)
    .execute(executor)
    .await
    .map_err(map_student_constraint)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::class::{assign_devices, batch_set_seats, create_class, SeatAssignment};
    use crate::domain::test_support::{register_test_device, setup_pool};

    async fn class_with_seat(pool: &SqlitePool, class_name: &str, seat: &str) -> String {
        let class = create_class(pool, class_name).await.unwrap();
        let device_id = register_test_device(pool, &format!("DEV-{}", class_name)).await;
        assign_devices(pool, &class.id, &[device_id.clone()]).await.unwrap();
        batch_set_seats(pool, &class.id, &[SeatAssignment { device_id, seat_no: seat.into() }])
            .await
            .unwrap();
        class.id
    }

    #[tokio::test]
    async fn create_student_rejects_missing_class_or_seat() {
        let pool = setup_pool().await;
        let err = create_student(&pool, CreateStudentRequest {
            student_no: Some("S1".into()),
            name: "张三".into(),
            password: None,
            seat_no: None,
            class_id: None,
        }).await.unwrap_err();
        assert!(err.to_string().contains("班级为必填项"));

        let class = create_class(&pool, "一班").await.unwrap();
        let err = create_student(&pool, CreateStudentRequest {
            student_no: Some("S2".into()),
            name: "李四".into(),
            password: None,
            seat_no: None,
            class_id: Some(class.id),
        }).await.unwrap_err();
        assert!(err.to_string().contains("座位号为必填项"));
    }

    #[tokio::test]
    async fn create_student_rejects_seat_not_assigned_to_class_device() {
        let pool = setup_pool().await;
        let class = create_class(&pool, "二班").await.unwrap();
        let err = create_student(&pool, CreateStudentRequest {
            student_no: Some("S3".into()),
            name: "王五".into(),
            password: None,
            seat_no: Some("1".into()),
            class_id: Some(class.id),
        }).await.unwrap_err();
        assert!(err.to_string().contains("不存在"));
    }

    #[tokio::test]
    async fn create_student_succeeds_when_device_seat_exists() {
        let pool = setup_pool().await;
        let class_id = class_with_seat(&pool, "三班", "12").await;
        let created = create_student(&pool, CreateStudentRequest {
            student_no: Some("S4".into()),
            name: "赵六".into(),
            password: None,
            seat_no: Some("12".into()),
            class_id: Some(class_id.clone()),
        }).await.unwrap();
        assert_eq!(created.student_no, "S4");
        assert_eq!(created.seat_no.as_deref(), Some("12"));
        assert_eq!(created.class_id.as_deref(), Some(class_id.as_str()));
    }

    #[tokio::test]
    async fn import_rolls_back_when_later_row_fails() {
        let pool = setup_pool().await;
        let class_id = class_with_seat(&pool, "导入班", "1").await;
        let mut tx = pool.begin().await.unwrap();
        insert_imported_student(&mut *tx, "S10", "甲", "123456", "1", &class_id).await.unwrap();
        let err = insert_imported_student(&mut *tx, "S11", "乙", "123456", "1", &class_id).await.unwrap_err();
        assert!(err.to_string().contains("占用") || err.to_string().contains("UNIQUE"));
        drop(tx);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM students")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
