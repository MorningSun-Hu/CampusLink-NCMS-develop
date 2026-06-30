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
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStudentRequest {
    pub student_no: String,
    pub name: String,
    pub password: String,
    pub seat_no: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStudentRequest {
    pub student_no: Option<String>,
    pub name: Option<String>,
    pub password: Option<String>,
    pub seat_no: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub keyword: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentListResponse {
    pub students: Vec<StudentResponse>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

pub async fn create_student(pool: &SqlitePool, req: CreateStudentRequest) -> Result<StudentResponse> {
    let id = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash(&req.password, 4)?;

    sqlx::query(
        r#"INSERT INTO students (id, student_no, name, password_hash, seat_no, status, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, 'active', datetime('now'), datetime('now'))"#
    )
    .bind(&id)
    .bind(&req.student_no)
    .bind(&req.name)
    .bind(&password_hash)
    .bind(&req.seat_no)
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

    Ok(row.into())
}

pub async fn list_students(pool: &SqlitePool, query: StudentListQuery) -> Result<StudentListResponse> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = ((page - 1) * page_size) as i64;

    let (where_clause, bind_value) = if let Some(ref kw) = query.keyword {
        if kw.is_empty() {
            (String::new(), None)
        } else {
            (format!("WHERE name LIKE ? OR student_no LIKE ?"), Some(format!("%{}%", kw)))
        }
    } else {
        (String::new(), None)
    };

    let count_sql = format!("SELECT COUNT(*) as cnt FROM students {}", where_clause);
    let total: i64 = if let Some(ref kw) = bind_value {
        sqlx::query(&count_sql)
            .bind(kw)
            .bind(kw)
            .fetch_one(pool)
            .await?
            .get("cnt")
    } else {
        sqlx::query(&count_sql)
            .fetch_one(pool)
            .await?
            .get("cnt")
    };

    let list_sql = format!(
        "SELECT * FROM students {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_clause
    );

    let rows: Vec<StudentRow> = if let Some(ref kw) = bind_value {
        sqlx::query_as(&list_sql)
            .bind(kw)
            .bind(kw)
            .bind(page_size as i64)
            .bind(offset)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as(&list_sql)
            .bind(page_size as i64)
            .bind(offset)
            .fetch_all(pool)
            .await?
    };

    Ok(StudentListResponse {
        students: rows.into_iter().map(|r| r.into()).collect(),
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

    if let Some(password) = req.password {
        let password_hash = bcrypt::hash(&password, 4)?;
        sqlx::query(
            "UPDATE students SET student_no = ?, name = ?, password_hash = ?, seat_no = ?, status = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(&student_no)
        .bind(&name)
        .bind(&password_hash)
        .bind(&seat_no)
        .bind(&status)
        .bind(id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "UPDATE students SET student_no = ?, name = ?, seat_no = ?, status = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(&student_no)
        .bind(&name)
        .bind(&seat_no)
        .bind(&status)
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

pub async fn import_student(pool: &SqlitePool, student_no: &str, name: &str, password: &str, seat_no: Option<&str>) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash(password, 4)?;

    sqlx::query(
        "INSERT OR REPLACE INTO students (id, student_no, name, password_hash, seat_no, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 'active', datetime('now'), datetime('now'))"
    )
    .bind(&id)
    .bind(student_no)
    .bind(name)
    .bind(&password_hash)
    .bind(seat_no)
    .execute(pool)
    .await?;
    Ok(())
}
