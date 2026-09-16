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
    pub seat_no: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceRecord {
    pub id: String,
    pub student_id: Option<String>,
    pub student_no: Option<String>,
    pub student_name: Option<String>,
    pub device_id: String,
    pub check_in_time: String,
    pub check_out_time: Option<String>,
    pub status: String,
    pub remarks: Option<String>,
    pub seat_no: Option<String>,
}

impl AttendanceRow {
    fn into_record(self, student_no: Option<String>, student_name: Option<String>) -> AttendanceRecord {
        AttendanceRecord {
            id: self.id,
            student_id: self.student_id,
            student_no,
            student_name,
            device_id: self.device_id,
            check_in_time: crate::domain::time_util::to_rfc3339(&self.check_in_time),
            check_out_time: crate::domain::time_util::to_rfc3339_opt(self.check_out_time.as_deref()),
            status: self.status,
            remarks: self.remarks,
            seat_no: self.seat_no,
        }
    }
}

fn name_from_remarks(remarks: Option<&str>) -> Option<String> {
    remarks
        .and_then(|r| r.strip_prefix("open-checkin:"))
        .map(|s| s.to_string())
}

async fn enrich_records(pool: &SqlitePool, rows: Vec<AttendanceRow>) -> Result<Vec<AttendanceRecord>> {
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let (student_no, student_name) = match row.student_id.as_deref() {
            Some(sid) => {
                let info: Option<(String, String)> = sqlx::query_as(
                    "SELECT student_no, name FROM students WHERE id = ?"
                )
                .bind(sid)
                .fetch_optional(pool)
                .await?;
                match info {
                    Some((no, name)) => (Some(no), Some(name)),
                    None => (None, name_from_remarks(row.remarks.as_deref())),
                }
            }
            None => (None, name_from_remarks(row.remarks.as_deref())),
        };
        result.push(row.into_record(student_no, student_name));
    }
    Ok(result)
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

    let device_mode: Option<String> = sqlx::query_scalar("SELECT current_mode FROM student_devices WHERE id = ?")
        .bind(device_id)
        .fetch_optional(pool)
        .await?;
    let device_seat: Option<String> = sqlx::query_scalar("SELECT seat_no FROM student_devices WHERE id = ?")
        .bind(device_id)
        .fetch_optional(pool)
        .await?;

    let mut seat_snapshot: Option<String> = None;

    // 授课模式：必须使用学生账号，且学生座位号需与设备座位号一致
    let (resolved_student_id, remarks) = if let Some(sid) = student_id {
        let student = sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT id, seat_no FROM students WHERE id = ?"
        )
        .bind(sid)
        .fetch_optional(pool)
        .await?;

        match student {
            Some((student_pk, student_seat)) => {
                if device_mode.as_deref() == Some("teaching") {
                    if student_seat.is_none() || device_seat.is_none() || student_seat != device_seat {
                        anyhow::bail!("座位不匹配，请在指定机器的座位签到");
                    }
                }
                seat_snapshot = student_seat.or_else(|| device_seat.clone());
                (Some(student_pk), None)
            }
            None => {
                if device_mode.as_deref() == Some("teaching") {
                    anyhow::bail!("学号或密码错误");
                }
                // 开放模式下自由填写的姓名，student_id 实际存放姓名
                (None, Some(format!("open-checkin:{}", sid)))
            }
        }
    } else {
        (None, None)
    };

    sqlx::query(
        r#"INSERT INTO attendance_records (id, student_id, device_id, check_in_time, status, remarks, seat_no, created_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))"#
    )
    .bind(&record_id)
    .bind(&resolved_student_id)
    .bind(device_id)
    .bind(&check_in_time)
    .bind(status)
    .bind(&remarks)
    .bind(&seat_snapshot)
    .execute(pool)
    .await?;

    Ok(CheckInResponse {
        record_id,
        status: status.to_string(),
    })
}

pub async fn get_statistics(pool: &SqlitePool, date: Option<&str>) -> Result<AttendanceStatistics> {
    let total = count_by_status(pool, date, "total").await?;
    let present = count_by_status(pool, date, "present").await?;
    let late = count_by_status(pool, date, "late").await?;
    let absent = count_by_status(pool, date, "absent").await?;
    let leave = count_by_status(pool, date, "leave").await?;

    let records = list_by_date(pool, date).await?;

    Ok(AttendanceStatistics {
        total,
        present,
        late,
        absent,
        leave,
        records,
    })
}

async fn count_by_status(pool: &SqlitePool, date: Option<&str>, status: &str) -> Result<i64> {
    let sql = if status == "total" {
        match date {
            Some(_) => "SELECT COUNT(*) as count FROM attendance_records WHERE date(check_in_time) = date(?)".to_string(),
            None => "SELECT COUNT(*) as count FROM attendance_records WHERE date(check_in_time) = date('now')".to_string(),
        }
    } else {
        match date {
            Some(_) => format!("SELECT COUNT(*) as count FROM attendance_records WHERE date(check_in_time) = date(?) AND status = '{}'", status),
            None => format!("SELECT COUNT(*) as count FROM attendance_records WHERE date(check_in_time) = date('now') AND status = '{}'", status),
        }
    };

    let count: (i64,) = if let Some(d) = date {
        sqlx::query_as(&sql).bind(d).fetch_one(pool).await?
    } else {
        sqlx::query_as(&sql).fetch_one(pool).await?
    };

    Ok(count.0)
}

async fn list_by_date(pool: &SqlitePool, date: Option<&str>) -> Result<Vec<AttendanceRecord>> {
    let sql = match date {
        Some(_) => "SELECT * FROM attendance_records WHERE date(check_in_time) = date(?) ORDER BY check_in_time DESC".to_string(),
        None => "SELECT * FROM attendance_records WHERE date(check_in_time) = date('now') ORDER BY check_in_time DESC".to_string(),
    };

    let rows = if let Some(d) = date {
        sqlx::query_as::<_, AttendanceRow>(&sql).bind(d).fetch_all(pool).await?
    } else {
        sqlx::query_as::<_, AttendanceRow>(&sql).fetch_all(pool).await?
    };

    enrich_records(pool, rows).await
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

    enrich_records(pool, rows).await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceContext {
    pub device_id: String,
    pub mode: String,
    pub requires_checkin: bool,
    pub class_id: Option<String>,
}

pub async fn attendance_context(pool: &SqlitePool, device_id: &str) -> Result<AttendanceContext> {
    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT current_mode, class_id FROM student_devices WHERE id = ?"
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("设备未注册"))?;

    let (mode, class_id) = row;
    let requires_checkin = matches!(mode.as_str(), "open" | "teaching");

    Ok(AttendanceContext {
        device_id: device_id.to_string(),
        mode,
        requires_checkin,
        class_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_support::{setup_pool, register_test_device, create_test_student};

    #[tokio::test]
    async fn check_in_creates_present_record() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-TEST-001").await;
        let student_id = create_test_student(&pool, "STU-001").await;

        let resp = check_in(&pool, &device_id, Some(&student_id), Some(1_700_000_000)).await.unwrap();
        assert_eq!(resp.status, "present");

        let records = list_attendance(&pool, None).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].device_id, device_id);
        assert_eq!(records[0].student_id.as_deref(), Some(student_id.as_str()));
        assert_eq!(records[0].status, "present");
    }

    #[tokio::test]
    async fn check_in_defaults_to_now_when_no_timestamp() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-TEST-002").await;

        let resp = check_in(&pool, &device_id, None, None).await.unwrap();
        assert_eq!(resp.status, "present");

        let records = list_attendance(&pool, None).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].student_id, None);
    }

    #[tokio::test]
    async fn get_statistics_counts_by_status() {
        let pool = setup_pool().await;
        let d1 = register_test_device(&pool, "DEV-TEST-003").await;
        let d2 = register_test_device(&pool, "DEV-TEST-004").await;

        check_in(&pool, &d1, None, None).await.unwrap();
        check_in(&pool, &d2, None, None).await.unwrap();

        sqlx::query(
            "UPDATE attendance_records SET status = 'late' WHERE device_id = ?"
        )
        .bind(&d2)
        .execute(&pool)
        .await
        .unwrap();

        let stats = get_statistics(&pool, None).await.unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.present, 1);
        assert_eq!(stats.late, 1);
    }

    #[tokio::test]
    async fn check_in_with_unknown_student_degrades_to_remarks() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-TEST-005").await;

        // Free-form name from open-mode check-in is not in the students table.
        let resp = check_in(&pool, &device_id, Some("张三"), Some(1_700_000_000)).await.unwrap();
        assert_eq!(resp.status, "present");

        let records = list_attendance(&pool, None).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].student_id, None);
        assert_eq!(records[0].remarks.as_deref(), Some("open-checkin:张三"));
    }

    #[tokio::test]
    async fn retroactive_check_in_inserts_with_remarks() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-TEST-005").await;
        let student_id = create_test_student(&pool, "STU-005").await;

        let resp = retroactive_check_in(&pool, &student_id, &device_id, "2026-08-28 09:00:00", Some("补签")).await.unwrap();
        assert_eq!(resp.status, "present");

        let records = list_attendance(&pool, None).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].remarks.as_deref(), Some("补签"));
        assert_eq!(records[0].check_in_time, "2026-08-28T09:00:00Z");
    }

    #[tokio::test]
    async fn teaching_check_in_requires_matching_seat() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-SEAT-001").await;
        let student_id = create_test_student(&pool, "STU-SEAT-001").await;

        crate::domain::device::update_device_mode(&pool, &device_id, "teaching").await.unwrap();
        sqlx::query("UPDATE student_devices SET seat_no = '5' WHERE id = ?").bind(&device_id).execute(&pool).await.unwrap();
        sqlx::query("UPDATE students SET seat_no = '5' WHERE id = ?").bind(&student_id).execute(&pool).await.unwrap();

        check_in(&pool, &device_id, Some(&student_id), Some(1_700_000_000)).await.unwrap();
        let records = list_attendance(&pool, None).await.unwrap();
        assert_eq!(records[0].seat_no.as_deref(), Some("5"));
        assert_eq!(records[0].student_no.as_deref(), Some("STU-SEAT-001"));

        // 座位不匹配时拒绝
        sqlx::query("UPDATE students SET seat_no = '6' WHERE id = ?").bind(&student_id).execute(&pool).await.unwrap();
        assert!(check_in(&pool, &device_id, Some(&student_id), Some(1_700_000_001)).await.is_err());
    }

    #[tokio::test]
    async fn attendance_context_flags_checkin_requirement() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-CTX-001").await;

        let ctx = attendance_context(&pool, &device_id).await.unwrap();
        assert_eq!(ctx.mode, "open");
        assert!(ctx.requires_checkin);

        crate::domain::device::update_device_mode(&pool, &device_id, "locked").await.unwrap();
        let ctx = attendance_context(&pool, &device_id).await.unwrap();
        assert!(!ctx.requires_checkin);
    }
}
