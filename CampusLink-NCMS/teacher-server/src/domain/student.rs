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

pub async fn create_student(pool: &SqlitePool, req: CreateStudentRequest) -> Result<StudentResponse> {
    let id = Uuid::new_v4().to_string();
    let student_no = req
        .student_no
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(generate_student_no);
    let initial_password = req.password.filter(|s| !s.is_empty()).unwrap_or_else(|| "123456".to_string());
    let password_hash = bcrypt::hash(&initial_password, 4)?;

    sqlx::query(
        r#"INSERT INTO students (id, student_no, name, password_hash, seat_no, status, class_id, password_set, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, 'active', ?, 0, datetime('now'), datetime('now'))"#
    )
    .bind(&id)
    .bind(&student_no)
    .bind(&req.name)
    .bind(&password_hash)
    .bind(&req.seat_no)
    .bind(&req.class_id)
    .execute(pool)
    .await?;

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

async fn class_name_of(pool: &SqlitePool, class_id: Option<&str>) -> Result<Option<String>> {
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
        .await?;
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
        .await?;
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

pub async fn import_student(pool: &SqlitePool, student_no: &str, name: &str, password: &str, seat_no: Option<&str>, class_id: Option<&str>) -> Result<()> {
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
    .execute(pool)
    .await?;
    Ok(())
}
