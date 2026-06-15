use sqlx::{SqlitePool, FromRow};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AttendanceRow {
    pub id: String,
    pub student_id: Option<String>,
    pub device_id: String,
    pub check_in_time: String,
    pub check_out_time: Option<String>,
    pub status: String,
    pub remarks: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct AttendanceRecord {
    pub id: String,
    pub student_id: Option<String>,
    pub device_id: String,
    pub check_in_time: String,
    pub check_out_time: Option<String>,
    pub status: String,
    pub remarks: Option<String>,
}

impl From<AttendanceRow> for AttendanceRecord {
    fn from(row: AttendanceRow) -> Self {
        AttendanceRecord {
            id: row.id,
            student_id: row.student_id,
            device_id: row.device_id,
            check_in_time: row.check_in_time,
            check_out_time: row.check_out_time,
            status: row.status,
            remarks: row.remarks,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CheckInRequest {
    pub device_id: String,
    pub student_id: Option<String>,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CheckInResponse {
    pub record_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AttendanceStatistics {
    pub total: i64,
    pub present: i64,
    pub late: i64,
    pub absent: i64,
    pub leave: i64,
    pub records: Vec<AttendanceRecord>,
}

#[derive(Debug, Deserialize)]
pub struct RetroactiveRequest {
    pub student_id: String,
    pub device_id: String,
    pub check_in_time: String,
    pub remarks: Option<String>,
}

pub async fn check_in(pool: &SqlitePool, device_id: &str, student_id: Option<&str>, timestamp: Option<u64>) -> Result<CheckInResponse> {
    let record_id = Uuid::new_v4().to_string();
    let status = "present";

    let check_in_time = timestamp
        .map(|t| {
            let dt = chrono::DateTime::from_timestamp(t as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now());
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

    sqlx::query(
        r#"INSERT INTO attendance_records (id, student_id, device_id, check_in_time, status, created_at)
           VALUES (?, ?, ?, ?, ?, datetime('now'))"#
    )
    .bind(&record_id)
    .bind(student_id)
    .bind(device_id)
    .bind(&check_in_time)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(CheckInResponse {
        record_id,
        status: status.to_string(),
    })
}

pub async fn get_statistics(pool: &SqlitePool, date: Option<&str>) -> Result<AttendanceStatistics> {
    let date_filter = date.unwrap_or("date('now')");

    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) as count FROM attendance_records WHERE date(check_in_time) = date(?)"#
    )
    .bind(date_filter)
    .fetch_one(pool)
    .await?;

    let present: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) as count FROM attendance_records WHERE date(check_in_time) = date(?) AND status = 'present'"#
    )
    .bind(date_filter)
    .fetch_one(pool)
    .await?;

    let late: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) as count FROM attendance_records WHERE date(check_in_time) = date(?) AND status = 'late'"#
    )
    .bind(date_filter)
    .fetch_one(pool)
    .await?;

    let absent: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) as count FROM attendance_records WHERE date(check_in_time) = date(?) AND status = 'absent'"#
    )
    .bind(date_filter)
    .fetch_one(pool)
    .await?;

    let leave: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) as count FROM attendance_records WHERE date(check_in_time) = date(?) AND status = 'leave'"#
    )
    .bind(date_filter)
    .fetch_one(pool)
    .await?;

    let records = sqlx::query_as::<_, AttendanceRow>(
        r#"SELECT * FROM attendance_records WHERE date(check_in_time) = date(?) ORDER BY check_in_time DESC"#
    )
    .bind(date_filter)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| r.into())
    .collect();

    Ok(AttendanceStatistics {
        total: total.0,
        present: present.0,
        late: late.0,
        absent: absent.0,
        leave: leave.0,
        records,
    })
}

pub async fn retroactive_check_in(pool: &SqlitePool, student_id: &str, device_id: &str, check_in_time: &str, remarks: Option<&str>) -> Result<CheckInResponse> {
    let record_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO attendance_records (id, student_id, device_id, check_in_time, status, remarks, created_at)
           VALUES (?, ?, ?, ?, 'present', ?, datetime('now'))"#
    )
    .bind(&record_id)
    .bind(student_id)
    .bind(device_id)
    .bind(check_in_time)
    .bind(remarks)
    .execute(pool)
    .await?;

    Ok(CheckInResponse {
        record_id,
        status: "present".to_string(),
    })
}

pub async fn list_attendance(pool: &SqlitePool, limit: Option<i64>) -> Result<Vec<AttendanceRecord>> {
    let limit_val = limit.unwrap_or(50);
    let rows = sqlx::query_as::<_, AttendanceRow>(
        "SELECT * FROM attendance_records ORDER BY check_in_time DESC LIMIT ?"
    )
    .bind(limit_val)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into()).collect())
}
