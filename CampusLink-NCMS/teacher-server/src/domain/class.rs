use sqlx::{SqlitePool, FromRow};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::student::{self, StudentResponse};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassRow {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClassRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClassRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignStudentsRequest {
    pub student_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatAssignment {
    pub device_id: String,
    pub seat_no: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SeatAssignmentResult {
    pub device_id: String,
    pub seat_no: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSeatRequest {
    pub seats: Vec<SeatAssignment>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoSeatResult {
    pub assigned: Vec<SeatAssignmentResult>,
    pub conflicts: Vec<String>,
    pub unmatched: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassModeSwitchResponse {
    pub class_id: String,
    pub target_mode: String,
    pub affected_devices: usize,
    pub device_ids: Vec<String>,
}

pub async fn create_class(pool: &SqlitePool, name: &str) -> Result<ClassRow> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!("班级名称不能为空");
    }
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO classes (id, name, created_at, updated_at) VALUES (?, ?, datetime('now'), datetime('now'))")
        .bind(&id)
        .bind(trimmed)
        .execute(pool)
        .await?;

    let row = sqlx::query_as::<_, ClassRow>("SELECT * FROM classes WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await?;
    Ok(row)
}

pub async fn list_classes(pool: &SqlitePool) -> Result<Vec<ClassRow>> {
    let rows = sqlx::query_as::<_, ClassRow>("SELECT * FROM classes ORDER BY name ASC")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn update_class(pool: &SqlitePool, id: &str, name: &str) -> Result<ClassRow> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!("班级名称不能为空");
    }
    sqlx::query("UPDATE classes SET name = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(trimmed)
        .bind(id)
        .execute(pool)
        .await?;

    let row = sqlx::query_as::<_, ClassRow>("SELECT * FROM classes WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;
    Ok(row)
}

pub async fn delete_class(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("UPDATE students SET class_id = NULL WHERE class_id = ?").bind(id).execute(pool).await?;
    sqlx::query("UPDATE student_devices SET class_id = NULL WHERE class_id = ?").bind(id).execute(pool).await?;
    sqlx::query("DELETE FROM classes WHERE id = ?").bind(id).execute(pool).await?;
    Ok(())
}

pub async fn assign_students(pool: &SqlitePool, class_id: &str, student_ids: &[String]) -> Result<u64> {
    let mut count = 0u64;
    for sid in student_ids {
        let affected = sqlx::query("UPDATE students SET class_id = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(class_id)
            .bind(sid)
            .execute(pool)
            .await?
            .rows_affected();
        count += affected;
    }
    Ok(count)
}

pub async fn assign_devices(pool: &SqlitePool, class_id: &str, device_ids: &[String]) -> Result<u64> {
    let mut count = 0u64;
    for did in device_ids {
        let affected = sqlx::query("UPDATE student_devices SET class_id = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(class_id)
            .bind(did)
            .execute(pool)
            .await?
            .rows_affected();
        count += affected;
    }
    Ok(count)
}

pub async fn list_students(pool: &SqlitePool, class_id: &str) -> Result<Vec<StudentResponse>> {
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM students WHERE class_id = ? ORDER BY seat_no ASC, student_no ASC")
        .bind(class_id)
        .fetch_all(pool)
        .await?;
    let mut result = Vec::with_capacity(ids.len());
    for id in ids {
        result.push(student::get_student_by_id(pool, &id).await?);
    }
    Ok(result)
}

pub async fn has_students(pool: &SqlitePool, class_id: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM students WHERE class_id = ?")
        .bind(class_id)
        .fetch_one(pool)
        .await?;
    Ok(count > 0)
}

fn ip_last_octet(ip: &str) -> Option<u32> {
    let ip = ip.trim();
    let last = ip.rsplit('.').next()?;
    last.parse::<u32>().ok()
}

pub async fn batch_set_seats(pool: &SqlitePool, class_id: &str, seats: &[SeatAssignment]) -> Result<()> {
    for s in seats {
        sqlx::query("UPDATE student_devices SET seat_no = ?, updated_at = datetime('now') WHERE id = ? AND class_id = ?")
            .bind(&s.seat_no)
            .bind(&s.device_id)
            .bind(class_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn auto_seats_by_ip(pool: &SqlitePool, class_id: &str) -> Result<AutoSeatResult> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT id, ip_address, seat_no FROM student_devices WHERE class_id = ? ORDER BY ip_address ASC"
    )
    .bind(class_id)
    .fetch_all(pool)
    .await?;

    let mut taken: Vec<String> = rows.iter().filter_map(|(_, _, s)| s.clone()).collect();
    let mut assigned = Vec::new();
    let mut conflicts = Vec::new();
    let mut unmatched = Vec::new();

    for (device_id, ip, existing) in rows {
        if existing.is_some() {
            continue;
        }
        let octet = match ip_last_octet(&ip) {
            Some(o) if o > 0 => o,
            _ => {
                unmatched.push(ip);
                continue;
            }
        };
        let mut candidate = octet;
        if taken.contains(&candidate.to_string()) {
            conflicts.push(candidate.to_string());
            while taken.contains(&candidate.to_string()) {
                candidate += 1;
            }
        }
        let seat = candidate.to_string();
        sqlx::query("UPDATE student_devices SET seat_no = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(&seat)
            .bind(&device_id)
            .execute(pool)
            .await?;
        taken.push(seat.clone());
        assigned.push(SeatAssignmentResult { device_id, seat_no: seat });
    }

    Ok(AutoSeatResult {
        assigned,
        conflicts,
        unmatched,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassModeResult {
    pub device_ids: Vec<String>,
    pub teaching_device_ids: Vec<String>,
    pub locked_device_ids: Vec<String>,
}

pub async fn class_student_seats(pool: &SqlitePool, class_id: &str) -> Result<Vec<String>> {
    let seats = sqlx::query_scalar::<_, String>(
        "SELECT seat_no FROM students WHERE class_id = ? AND seat_no IS NOT NULL AND TRIM(seat_no) != ''"
    )
    .bind(class_id)
    .fetch_all(pool)
    .await?;
    Ok(seats)
}

fn seat_matches(device_seat: &Option<String>, student_seats: &[String]) -> bool {
    match device_seat {
        Some(s) if !s.trim().is_empty() => student_seats.iter().any(|x| x.trim() == s.trim()),
        _ => false,
    }
}

pub async fn teaching_target_for_device(pool: &SqlitePool, class_id: &str, device_id: &str) -> Result<String> {
    if !has_students(pool, class_id).await? {
        anyhow::bail!("请先补充学生信息");
    }

    let device_seat: Option<String> = sqlx::query_scalar("SELECT seat_no FROM student_devices WHERE id = ?")
        .bind(device_id)
        .fetch_optional(pool)
        .await?
        .flatten();
    let student_seats = class_student_seats(pool, class_id).await?;

    Ok(if seat_matches(&device_seat, &student_seats) { "teaching" } else { "locked" }.to_string())
}

pub async fn switch_class_mode(pool: &SqlitePool, class_id: &str, target_mode: &str) -> Result<ClassModeResult> {
    if target_mode == "teaching" {
        if !has_students(pool, class_id).await? {
            anyhow::bail!("请先补充学生信息");
        }

        let rows = sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT id, seat_no FROM student_devices WHERE class_id = ?"
        )
        .bind(class_id)
        .fetch_all(pool)
        .await?;
        let student_seats = class_student_seats(pool, class_id).await?;

        let mut teaching_device_ids = Vec::new();
        let mut locked_device_ids = Vec::new();
        for (device_id, seat_no) in rows {
            let _ = crate::domain::usage::close_open_session(pool, &device_id).await;
            let effective = if seat_matches(&seat_no, &student_seats) { "teaching" } else { "locked" };
            sqlx::query("UPDATE student_devices SET current_mode = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(effective)
                .bind(&device_id)
                .execute(pool)
                .await?;
            if effective == "teaching" {
                teaching_device_ids.push(device_id);
            } else {
                locked_device_ids.push(device_id);
            }
        }

        let mut device_ids = teaching_device_ids.clone();
        device_ids.extend(locked_device_ids.iter().cloned());
        return Ok(ClassModeResult { device_ids, teaching_device_ids, locked_device_ids });
    }

    let device_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM student_devices WHERE class_id = ?")
        .bind(class_id)
        .fetch_all(pool)
        .await?;

    for did in &device_ids {
        let _ = crate::domain::usage::close_open_session(pool, did).await;
        sqlx::query("UPDATE student_devices SET current_mode = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(target_mode)
            .bind(did)
            .execute(pool)
            .await?;
    }

    Ok(ClassModeResult {
        device_ids,
        teaching_device_ids: Vec::new(),
        locked_device_ids: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_support::{setup_pool, register_test_device, create_test_student};

    #[tokio::test]
    async fn class_crud_and_teaching_switch_requires_students() {
        let pool = setup_pool().await;
        let class = create_class(&pool, "三年级一班").await.unwrap();
        assert_eq!(class.name, "三年级一班");

        // 无学生时不得切换到授课模式
        assert!(switch_class_mode(&pool, &class.id, "teaching").await.is_err());

        let sid = create_test_student(&pool, "STU-CLASS-1").await;
        assign_students(&pool, &class.id, &[sid.clone()]).await.unwrap();
        assert!(has_students(&pool, &class.id).await.unwrap());
        assert_eq!(list_students(&pool, &class.id).await.unwrap().len(), 1);

        let did = register_test_device(&pool, "DEV-CLASS-1").await;
        assign_devices(&pool, &class.id, &[did.clone()]).await.unwrap();
        let switched = switch_class_mode(&pool, &class.id, "teaching").await.unwrap();
        assert_eq!(switched.device_ids, vec![did.clone()]);
        assert!(switched.teaching_device_ids.is_empty());
        assert_eq!(switched.locked_device_ids, vec![did.clone()]);

        batch_set_seats(&pool, &class.id, &[SeatAssignment { device_id: did.clone(), seat_no: "10".into() }])
            .await
            .unwrap();
        let seats: Option<String> = sqlx::query_scalar("SELECT seat_no FROM student_devices WHERE id = ?")
            .bind(&did)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(seats.as_deref(), Some("10"));

        delete_class(&pool, &class.id).await.unwrap();
        assert!(list_classes(&pool).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn auto_seats_assigns_last_octet_by_ip() {
        let pool = setup_pool().await;
        let class = create_class(&pool, "三班").await.unwrap();
        let d1 = register_test_device(&pool, "DEV-IP-1").await;
        let d2 = register_test_device(&pool, "DEV-IP-2").await;

        sqlx::query("UPDATE student_devices SET ip_address = '192.168.0.21' WHERE id = ?").bind(&d1).execute(&pool).await.unwrap();
        sqlx::query("UPDATE student_devices SET ip_address = '192.168.0.22' WHERE id = ?").bind(&d2).execute(&pool).await.unwrap();
        assign_devices(&pool, &class.id, &[d1.clone(), d2.clone()]).await.unwrap();

        let result = auto_seats_by_ip(&pool, &class.id).await.unwrap();
        assert_eq!(result.assigned.len(), 2);
        assert!(result.assigned.iter().any(|s| s.seat_no == "21"));
        assert!(result.assigned.iter().any(|s| s.seat_no == "22"));
    }
}
