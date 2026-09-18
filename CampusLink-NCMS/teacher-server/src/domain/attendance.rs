use std::collections::HashMap;

use sqlx::{Sqlite, SqlitePool, FromRow, QueryBuilder};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::usage::{self, StartSessionParams};
use crate::domain::inspection::{self, InspectionItem};

#[derive(Debug, Clone, FromRow)]
struct AttendanceJoinedRow {
    pub id: String,
    pub student_id: Option<String>,
    pub student_no: Option<String>,
    pub student_name: Option<String>,
    pub device_id: String,
    pub device_name: Option<String>,
    pub class_id: Option<String>,
    pub class_name: Option<String>,
    pub check_in_time: String,
    pub check_out_time: Option<String>,
    pub status: String,
    pub remarks: Option<String>,
    pub seat_no: Option<String>,
    pub usage_record_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceRecord {
    pub id: String,
    pub student_id: Option<String>,
    pub student_no: Option<String>,
    pub student_name: Option<String>,
    pub device_id: String,
    pub device_name: Option<String>,
    pub class_id: Option<String>,
    pub class_name: Option<String>,
    pub check_in_time: String,
    pub check_out_time: Option<String>,
    pub status: String,
    pub remarks: Option<String>,
    pub seat_no: Option<String>,
    pub usage_record_id: Option<String>,
}

impl From<AttendanceJoinedRow> for AttendanceRecord {
    fn from(row: AttendanceJoinedRow) -> Self {
        Self {
            id: row.id,
            student_id: row.student_id,
            student_no: row.student_no,
            student_name: row.student_name,
            device_id: row.device_id,
            device_name: row.device_name,
            class_id: row.class_id,
            class_name: row.class_name,
            check_in_time: crate::domain::time_util::to_rfc3339(&row.check_in_time),
            check_out_time: crate::domain::time_util::to_rfc3339_opt(row.check_out_time.as_deref()),
            status: row.status,
            remarks: row.remarks,
            seat_no: row.seat_no,
            usage_record_id: row.usage_record_id,
        }
    }
}

const SELECT_JOINED: &str = r#"
    SELECT a.id, a.student_id, s.student_no, s.name AS student_name,
           a.device_id, d.device_name, s.class_id, c.name AS class_name,
           a.check_in_time, a.check_out_time, a.status, a.remarks, a.seat_no,
           a.usage_record_id
    FROM attendance_records a
    LEFT JOIN students s ON s.id = a.student_id
    LEFT JOIN classes c ON c.id = s.class_id
    LEFT JOIN student_devices d ON d.id = a.device_id
"#;

/// 只统计授课模式签到：排除自由填写的开放模式历史记录。
const EXCLUDE_OPEN: &str = " AND (a.remarks IS NULL OR a.remarks NOT LIKE 'open-checkin:%')";

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceQuery {
    pub class_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub student_keyword: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

fn build_query(query: &AttendanceQuery) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::<Sqlite>::new(SELECT_JOINED);
    builder.push(" WHERE 1 = 1").push(EXCLUDE_OPEN);

    if let Some(class_id) = query.class_id.as_deref().filter(|v| !v.is_empty()) {
        builder.push(" AND s.class_id = ").push_bind(class_id);
    }
    if let Some(keyword) = query.student_keyword.as_deref().filter(|v| !v.is_empty()) {
        let pattern = format!("%{}%", keyword);
        builder
            .push(" AND (s.student_no LIKE ")
            .push_bind(pattern.clone())
            .push(" OR s.name LIKE ")
            .push_bind(pattern)
            .push(")");
    }
    if let Some(status) = query.status.as_deref().filter(|v| !v.is_empty()) {
        builder.push(" AND a.status = ").push_bind(status);
    }
    if let Some(start_date) = query.start_date.as_deref().filter(|v| !v.is_empty()) {
        builder.push(" AND date(a.check_in_time) >= ").push_bind(start_date);
    }
    if let Some(end_date) = query.end_date.as_deref().filter(|v| !v.is_empty()) {
        builder.push(" AND date(a.check_in_time) <= ").push_bind(end_date);
    }
    builder
}

pub async fn list_attendance(pool: &SqlitePool, query: &AttendanceQuery) -> Result<Vec<AttendanceRecord>> {
    let mut builder = build_query(query);
    builder.push(" ORDER BY a.check_in_time DESC");
    if let Some(limit) = query.limit {
        builder.push(" LIMIT ").push_bind(limit);
    }

    let rows = builder
        .build_query_as::<AttendanceJoinedRow>()
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(AttendanceRecord::from).collect())
}

#[derive(Debug, Deserialize)]
pub struct InspectionItemInput {
    pub name: String,
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckInInspection {
    pub items: Vec<InspectionItemInput>,
    pub is_abnormal: Option<bool>,
}

fn is_normal_status(status: &str) -> bool {
    matches!(
        status.to_ascii_lowercase().as_str(),
        "ok" | "normal" | "pass" | "passed" | "good" | "healthy"
    )
}

pub fn summarize_inspection(inspection: Option<&CheckInInspection>) -> (bool, Option<String>) {
    let Some(inspection) = inspection else {
        return (true, None);
    };

    let mut abnormal: Vec<String> = Vec::new();
    for item in &inspection.items {
        if !is_normal_status(&item.status) {
            abnormal.push(match item.detail.as_deref().filter(|d| !d.is_empty()) {
                Some(detail) => format!("{}：{}", item.name, detail),
                None => item.name.clone(),
            });
        }
    }

    if abnormal.is_empty() {
        if inspection.is_abnormal.unwrap_or(false) {
            return (false, Some("使用环境存在异常".to_string()));
        }
        return (true, None);
    }

    (false, Some(abnormal.join("；")))
}

async fn persist_checkin_inspection(pool: &SqlitePool, device_id: &str, inspection: &CheckInInspection) -> Result<()> {
    if inspection.items.is_empty() {
        return Ok(());
    }

    let mut items: Vec<InspectionItem> = inspection
        .items
        .iter()
        .map(|item| {
            let status = if is_normal_status(&item.status) {
                "normal".to_string()
            } else {
                "abnormal".to_string()
            };
            InspectionItem {
                item_name: item.name.clone(),
                status,
                description: item.detail.clone(),
            }
        })
        .collect();

    let flagged = inspection.is_abnormal.unwrap_or(false);
    if flagged && items.iter().all(|item| item.status == "normal") {
        items.push(InspectionItem {
            item_name: "环境检查".to_string(),
            status: "abnormal".to_string(),
            description: Some("使用环境存在异常".to_string()),
        });
    }

    let is_abnormal = flagged || items.iter().any(|item| item.status == "abnormal");
    inspection::submit_inspection(pool, device_id, "checkin", &items, is_abnormal).await?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct CheckInResponse {
    pub record_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_record_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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

fn format_check_in_time(timestamp: Option<u64>) -> String {
    timestamp
        .map(|t| {
            let dt = chrono::DateTime::from_timestamp(t as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now());
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
}

pub async fn check_in(
    pool: &SqlitePool,
    device_id: &str,
    student_id: Option<&str>,
    timestamp: Option<u64>,
    inspection: Option<CheckInInspection>,
) -> Result<CheckInResponse> {
    let check_in_time = format_check_in_time(timestamp);
    let (inspection_ok, inspection_summary) = summarize_inspection(inspection.as_ref());

    let device = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT current_mode, class_id, seat_no FROM student_devices WHERE id = ?"
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("设备未注册"))?;

    let (mode, device_class_id, device_seat) = device;

    if mode == "teaching" {
        let sid = student_id.ok_or_else(|| anyhow::anyhow!("授课模式请使用学生账号签到"))?;
        let student = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>)>(
            "SELECT id, student_no, name, seat_no, class_id FROM students WHERE id = ?"
        )
        .bind(sid)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("学号或密码错误"))?;

        let (student_pk, _student_no, name, student_seat, student_class) = student;
        if student_seat.is_none() || device_seat.is_none() || student_seat != device_seat {
            anyhow::bail!("座位不匹配，请在指定机器的座位签到");
        }

        let seat_snapshot = student_seat.or_else(|| device_seat.clone());
        let usage_record_id = usage::start_session(
            pool,
            StartSessionParams {
                device_id,
                class_id: student_class.as_deref().or(device_class_id.as_deref()),
                seat_no: seat_snapshot.as_deref(),
                student_id: Some(&student_pk),
                user_name: &name,
                mode: &mode,
                inspection_ok,
                inspection_summary: inspection_summary.as_deref(),
            },
        )
        .await?;

        let record_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO attendance_records
               (id, student_id, device_id, check_in_time, status, remarks, seat_no, usage_record_id, created_at)
               VALUES (?, ?, ?, ?, 'present', NULL, ?, ?, datetime('now'))"#
        )
        .bind(&record_id)
        .bind(&student_pk)
        .bind(device_id)
        .bind(&check_in_time)
        .bind(&seat_snapshot)
        .bind(&usage_record_id)
        .execute(pool)
        .await?;

        if let Some(insp) = inspection.as_ref() {
            if let Err(e) = persist_checkin_inspection(pool, device_id, insp).await {
                tracing::warn!("Persist check-in inspection failed: {}", e);
            }
        }

        return Ok(CheckInResponse {
            record_id,
            status: "present".to_string(),
            usage_record_id: Some(usage_record_id),
        });
    }

    // 开放模式（以及其他非授课模式）：仅生成使用记录，不计入考勤
    let user_name = student_id
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("未知用户")
        .to_string();

    let usage_record_id = usage::start_session(
        pool,
        StartSessionParams {
            device_id,
            class_id: device_class_id.as_deref(),
            seat_no: device_seat.as_deref(),
            student_id: None,
            user_name: &user_name,
            mode: &mode,
            inspection_ok,
            inspection_summary: inspection_summary.as_deref(),
        },
    )
    .await?;

    if let Some(insp) = inspection.as_ref() {
        if let Err(e) = persist_checkin_inspection(pool, device_id, insp).await {
            tracing::warn!("Persist check-in inspection failed: {}", e);
        }
    }

    Ok(CheckInResponse {
        record_id: usage_record_id.clone(),
        status: "in_use".to_string(),
        usage_record_id: Some(usage_record_id),
    })
}

pub async fn get_statistics(pool: &SqlitePool, date: Option<&str>) -> Result<AttendanceStatistics> {
    let total = count_by_status(pool, date, "total").await?;
    let present = count_by_status(pool, date, "present").await?;
    let late = count_by_status(pool, date, "late").await?;
    let absent = count_by_status(pool, date, "absent").await?;
    let leave = count_by_status(pool, date, "leave").await?;

    let records = list_attendance(
        pool,
        &AttendanceQuery {
            start_date: date.map(str::to_string),
            end_date: date.map(str::to_string),
            ..Default::default()
        },
    )
    .await?;

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
        format!(
            "SELECT COUNT(*) FROM attendance_records a WHERE date(a.check_in_time) = {} {}",
            if date.is_some() { "date(?)" } else { "date('now')" },
            EXCLUDE_OPEN
        )
    } else {
        format!(
            "SELECT COUNT(*) FROM attendance_records a WHERE date(a.check_in_time) = {} AND a.status = '{}' {}",
            if date.is_some() { "date(?)" } else { "date('now')" },
            status,
            EXCLUDE_OPEN
        )
    };

    let count: i64 = if let Some(d) = date {
        sqlx::query_scalar(&sql).bind(d).fetch_one(pool).await?
    } else {
        sqlx::query_scalar(&sql).fetch_one(pool).await?
    };

    Ok(count)
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
        usage_record_id: None,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardStudent {
    pub student_id: String,
    pub student_no: String,
    pub student_name: String,
    pub seat_no: Option<String>,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub check_in_time: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceBoard {
    pub class_id: Option<String>,
    pub class_name: Option<String>,
    pub date: String,
    pub total_students: i64,
    pub present_count: i64,
    pub absent_count: i64,
    pub attendance_rate: f64,
    pub present: Vec<BoardStudent>,
    pub absent: Vec<BoardStudent>,
}

fn attendance_counts_as_present(status: &str) -> bool {
    matches!(status, "present" | "late" | "leave")
}

pub async fn attendance_board(
    pool: &SqlitePool,
    class_id: Option<&str>,
    date: Option<&str>,
) -> Result<AttendanceBoard> {
    let date = date
        .filter(|v| !v.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

    let class_name: Option<String> = match class_id {
        Some(id) => sqlx::query_scalar("SELECT name FROM classes WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .flatten(),
        None => None,
    };

    let mut students_builder = QueryBuilder::<Sqlite>::new(
        "SELECT id, student_no, name, seat_no FROM students WHERE 1 = 1"
    );
    if let Some(id) = class_id.filter(|v| !v.is_empty()) {
        students_builder.push(" AND class_id = ").push_bind(id);
    }
    students_builder.push(" ORDER BY seat_no ASC, student_no ASC");
    let students = students_builder
        .build_query_as::<(String, String, String, Option<String>)>()
        .fetch_all(pool)
        .await?;

    let mut records_builder = QueryBuilder::<Sqlite>::new(
        r#"SELECT a.student_id, a.check_in_time, a.status, a.device_id, d.device_name, a.seat_no
           FROM attendance_records a
           LEFT JOIN student_devices d ON d.id = a.device_id
           WHERE date(a.check_in_time) = date("#,
    );
    records_builder.push_bind(&date);
    records_builder.push(")");
    records_builder.push(EXCLUDE_OPEN);
    records_builder.push(" AND a.student_id IS NOT NULL");
    if let Some(id) = class_id.filter(|v| !v.is_empty()) {
        records_builder
            .push(" AND a.student_id IN (SELECT id FROM students WHERE class_id = ")
            .push_bind(id)
            .push(")");
    }
    let records = records_builder
        .build_query_as::<(String, String, String, String, Option<String>, Option<String>)>()
        .fetch_all(pool)
        .await?;

    let mut present_map: HashMap<String, (String, String, String, Option<String>, Option<String>)> =
        HashMap::new();
    for (student_id, check_in_time, status, device_id, device_name, seat_no) in records {
        present_map.insert(student_id, (check_in_time, status, device_id, device_name, seat_no));
    }

    let mut present = Vec::new();
    let mut absent = Vec::new();
    for (student_id, student_no, student_name, seat_no) in students {
        match present_map.remove(&student_id) {
            Some((check_in_time, status, device_id, device_name, record_seat)) if attendance_counts_as_present(&status) => {
                present.push(BoardStudent {
                    student_id,
                    student_no,
                    student_name,
                    seat_no: record_seat.or(seat_no),
                    device_id: Some(device_id),
                    device_name,
                    check_in_time: Some(crate::domain::time_util::to_rfc3339(&check_in_time)),
                    status,
                });
            }
            _ => absent.push(BoardStudent {
                student_id,
                student_no,
                student_name,
                seat_no,
                device_id: None,
                device_name: None,
                check_in_time: None,
                status: "absent".to_string(),
            }),
        }
    }

    let total_students = (present.len() + absent.len()) as i64;
    let present_count = present.len() as i64;
    let absent_count = absent.len() as i64;
    let attendance_rate = if total_students == 0 {
        0.0
    } else {
        ((present_count as f64 / total_students as f64) * 10000.0).round() / 10000.0
    };

    Ok(AttendanceBoard {
        class_id: class_id.map(str::to_string),
        class_name,
        date,
        total_students,
        present_count,
        absent_count,
        attendance_rate,
        present,
        absent,
    })
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

    async fn make_teaching(pool: &SqlitePool, device_id: &str, student_id: &str, seat: &str) {
        crate::domain::device::update_device_mode(pool, device_id, "teaching").await.unwrap();
        sqlx::query("UPDATE student_devices SET seat_no = ? WHERE id = ?").bind(seat).bind(device_id).execute(pool).await.unwrap();
        sqlx::query("UPDATE students SET seat_no = ? WHERE id = ?").bind(seat).bind(student_id).execute(pool).await.unwrap();
    }

    #[tokio::test]
    async fn check_in_creates_present_record() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-TEST-001").await;
        let student_id = create_test_student(&pool, "STU-001").await;
        make_teaching(&pool, &device_id, &student_id, "5").await;

        let resp = check_in(&pool, &device_id, Some(&student_id), Some(1_700_000_000), None).await.unwrap();
        assert_eq!(resp.status, "present");
        assert!(resp.usage_record_id.is_some());

        let records = list_attendance(&pool, &AttendanceQuery::default()).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].device_id, device_id);
        assert_eq!(records[0].student_id.as_deref(), Some(student_id.as_str()));
        assert_eq!(records[0].status, "present");
        assert_eq!(records[0].seat_no.as_deref(), Some("5"));
    }

    #[tokio::test]
    async fn open_mode_check_in_only_creates_usage_record() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-TEST-002").await;

        let resp = check_in(&pool, &device_id, Some("张三"), None, None).await.unwrap();
        assert_eq!(resp.status, "in_use");

        let attendance = list_attendance(&pool, &AttendanceQuery::default()).await.unwrap();
        assert!(attendance.is_empty());

        let usage = usage::list_usage(&pool, &usage::UsageQuery::default()).await.unwrap();
        assert_eq!(usage.len(), 1);
        assert_eq!(usage[0].user_name, "张三");
        assert_eq!(usage[0].mode, "open");
        assert_eq!(usage[0].student_id, None);
    }

    #[tokio::test]
    async fn get_statistics_counts_by_status() {
        let pool = setup_pool().await;
        let d1 = register_test_device(&pool, "DEV-TEST-003").await;
        let d2 = register_test_device(&pool, "DEV-TEST-004").await;
        let s1 = create_test_student(&pool, "STU-003").await;
        let s2 = create_test_student(&pool, "STU-004").await;
        make_teaching(&pool, &d1, &s1, "1").await;
        make_teaching(&pool, &d2, &s2, "2").await;

        check_in(&pool, &d1, Some(&s1), None, None).await.unwrap();
        check_in(&pool, &d2, Some(&s2), None, None).await.unwrap();

        sqlx::query("UPDATE attendance_records SET status = 'late' WHERE device_id = ?")
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
    async fn check_in_with_unknown_student_degrades_to_usage() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-TEST-005").await;

        // Free-form name from open-mode check-in is not in the students table.
        let resp = check_in(&pool, &device_id, Some("李四"), Some(1_700_000_000), None).await.unwrap();
        assert_eq!(resp.status, "in_use");

        let attendance = list_attendance(&pool, &AttendanceQuery::default()).await.unwrap();
        assert!(attendance.is_empty());

        let usage = usage::list_usage(&pool, &usage::UsageQuery::default()).await.unwrap();
        assert_eq!(usage[0].user_name, "李四");
    }

    #[tokio::test]
    async fn retroactive_check_in_inserts_with_remarks() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-TEST-006").await;
        let student_id = create_test_student(&pool, "STU-006").await;

        let resp = retroactive_check_in(&pool, &student_id, &device_id, "2026-08-28 09:00:00", Some("补签")).await.unwrap();
        assert_eq!(resp.status, "present");

        let records = list_attendance(&pool, &AttendanceQuery::default()).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].remarks.as_deref(), Some("补签"));
        assert_eq!(records[0].check_in_time, "2026-08-28T09:00:00Z");
    }

    #[tokio::test]
    async fn teaching_check_in_requires_matching_seat() {
        let pool = setup_pool().await;
        let device_id = register_test_device(&pool, "DEV-SEAT-001").await;
        let student_id = create_test_student(&pool, "STU-SEAT-001").await;
        make_teaching(&pool, &device_id, &student_id, "5").await;

        check_in(&pool, &device_id, Some(&student_id), Some(1_700_000_000), None).await.unwrap();
        let records = list_attendance(&pool, &AttendanceQuery::default()).await.unwrap();
        assert_eq!(records[0].seat_no.as_deref(), Some("5"));
        assert_eq!(records[0].student_no.as_deref(), Some("STU-SEAT-001"));

        // 座位不匹配时拒绝
        sqlx::query("UPDATE students SET seat_no = '6' WHERE id = ?").bind(&student_id).execute(&pool).await.unwrap();
        assert!(check_in(&pool, &device_id, Some(&student_id), Some(1_700_000_001), None).await.is_err());
    }

    #[tokio::test]
    async fn attendance_board_counts_present_and_absent() {
        let pool = setup_pool().await;
        let class = crate::domain::class::create_class(&pool, "一班").await.unwrap();
        let s1 = create_test_student(&pool, "STU-B-1").await;
        let s2 = create_test_student(&pool, "STU-B-2").await;
        crate::domain::class::assign_students(&pool, &class.id, &[s1.clone(), s2.clone()]).await.unwrap();

        let device_id = register_test_device(&pool, "DEV-BOARD-1").await;
        make_teaching(&pool, &device_id, &s1, "5").await;
        check_in(&pool, &device_id, Some(&s1), None, None).await.unwrap();

        let board = attendance_board(&pool, Some(&class.id), None).await.unwrap();
        assert_eq!(board.total_students, 2);
        assert_eq!(board.present_count, 1);
        assert_eq!(board.absent_count, 1);
        assert_eq!(board.present_count + board.absent_count, board.total_students);
        assert_eq!(board.attendance_rate, 0.5);
        assert_eq!(board.present[0].student_id, s1);
        assert_eq!(board.absent[0].student_id, s2);
    }

    #[tokio::test]
    async fn attendance_board_ignores_open_mode_records() {
        let pool = setup_pool().await;
        let class = crate::domain::class::create_class(&pool, "二班").await.unwrap();
        let s1 = create_test_student(&pool, "STU-OPEN-1").await;
        crate::domain::class::assign_students(&pool, &class.id, &[s1.clone()]).await.unwrap();

        let device_id = register_test_device(&pool, "DEV-OPEN-1").await;
        check_in(&pool, &device_id, Some("王五"), None, None).await.unwrap();

        let board = attendance_board(&pool, Some(&class.id), None).await.unwrap();
        assert_eq!(board.total_students, 1);
        assert_eq!(board.present_count, 0);
        assert_eq!(board.absent_count, 1);
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
