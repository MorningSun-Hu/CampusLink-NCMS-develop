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
        // Changing class clears the old seat to avoid unique(class_id, seat_no) conflicts.
        let affected = sqlx::query(
            r#"UPDATE student_devices
               SET seat_no = CASE WHEN class_id = ? THEN seat_no ELSE NULL END,
                   class_id = ?,
                   updated_at = datetime('now')
               WHERE id = ?"#
        )
        .bind(class_id)
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
    let mut seen = std::collections::HashSet::new();
    let mut normalized = Vec::new();
    for s in seats {
        let seat = normalize_seat(&s.seat_no);
        if seat.is_empty() {
            continue;
        }
        if !seen.insert(seat.clone()) {
            anyhow::bail!("座位号 {} 重复", seat);
        }
        normalized.push((s.device_id.clone(), seat));
    }
    if normalized.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;
    for (device_id, _) in &normalized {
        let affected = sqlx::query(
            "UPDATE student_devices SET seat_no = NULL, updated_at = datetime('now') WHERE id = ? AND class_id = ?"
        )
        .bind(device_id)
        .bind(class_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if affected == 0 {
            anyhow::bail!("设备不属于该班级，无法保存座位号");
        }
    }
    for (device_id, seat) in &normalized {
        let affected = sqlx::query(
            "UPDATE student_devices SET seat_no = ?, updated_at = datetime('now') WHERE id = ? AND class_id = ?"
        )
        .bind(seat)
        .bind(device_id)
        .bind(class_id)
        .execute(&mut *tx)
        .await
        .map_err(map_seat_constraint)?
        .rows_affected();
        if affected == 0 {
            anyhow::bail!("设备不属于该班级，无法保存座位号");
        }
    }
    tx.commit().await?;
    Ok(())
}

fn normalize_seat(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Ok(number) = trimmed.parse::<f64>() {
        if number.fract() == 0.0 && number > 0.0 && number < 1e9 {
            return format!("{}", number as i64);
        }
    }
    trimmed.to_string()
}

fn map_seat_constraint(err: sqlx::Error) -> anyhow::Error {
    let text = err.to_string();
    if text.contains("UNIQUE") {
        anyhow::anyhow!("座位号已被该班其他学生机占用")
    } else {
        anyhow::Error::from(err)
    }
}

fn next_free_seat(taken: &std::collections::HashSet<u32>) -> u32 {
    let mut n = 1u32;
    while taken.contains(&n) {
        n += 1;
    }
    n
}

pub async fn fill_missing_seats(pool: &SqlitePool, class_id: &str) -> Result<Vec<SeatAssignmentResult>> {
    let rows = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT id, seat_no FROM student_devices WHERE class_id = ? ORDER BY ip_address ASC, id ASC"
    )
    .bind(class_id)
    .fetch_all(pool)
    .await?;

    let mut taken = std::collections::HashSet::new();
    for (_, seat) in &rows {
        if let Some(seat) = seat {
            let seat = normalize_seat(seat);
            if let Ok(n) = seat.parse::<u32>() {
                taken.insert(n);
            }
        }
    }

    let mut assigned = Vec::new();
    for (device_id, seat) in rows {
        let existing = seat.as_deref().map(normalize_seat).unwrap_or_default();
        if !existing.is_empty() {
            continue;
        }
        let n = next_free_seat(&taken);
        let seat_no = n.to_string();
        sqlx::query("UPDATE student_devices SET seat_no = ?, updated_at = datetime('now') WHERE id = ? AND class_id = ?")
            .bind(&seat_no)
            .bind(&device_id)
            .bind(class_id)
            .execute(pool)
            .await
            .map_err(map_seat_constraint)?;
        taken.insert(n);
        assigned.push(SeatAssignmentResult { device_id, seat_no });
    }
    Ok(assigned)
}

pub async fn renumber_seats(pool: &SqlitePool, class_id: &str) -> Result<Vec<SeatAssignmentResult>> {
    sqlx::query("UPDATE student_devices SET seat_no = NULL, updated_at = datetime('now') WHERE class_id = ?")
        .bind(class_id)
        .execute(pool)
        .await?;
    fill_missing_seats(pool, class_id).await
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

    #[tokio::test]
    async fn fill_missing_seats_starts_from_one() {
        let pool = setup_pool().await;
        let class = create_class(&pool, "座位班").await.unwrap();
        let d1 = register_test_device(&pool, "DEV-SEAT-1").await;
        let d2 = register_test_device(&pool, "DEV-SEAT-2").await;
        assign_devices(&pool, &class.id, &[d1.clone(), d2.clone()]).await.unwrap();

        let assigned = fill_missing_seats(&pool, &class.id).await.unwrap();
        assert_eq!(assigned.len(), 2);
        let mut seats: Vec<_> = assigned.iter().map(|s| s.seat_no.clone()).collect();
        seats.sort();
        assert_eq!(seats, vec!["1".to_string(), "2".to_string()]);

        batch_set_seats(&pool, &class.id, &[
            SeatAssignment { device_id: d1.clone(), seat_no: "1".into() },
            SeatAssignment { device_id: d2.clone(), seat_no: "2".into() },
        ])
            .await
            .unwrap();
        let err = batch_set_seats(&pool, &class.id, &[SeatAssignment { device_id: d2.clone(), seat_no: "1".into() }])
            .await
            .unwrap_err();
        assert!(err.to_string().contains("占用") || err.to_string().contains("重复"));
    }

    #[tokio::test]
    async fn batch_set_seats_allows_swap() {
        let pool = setup_pool().await;
        let class = create_class(&pool, "对调班").await.unwrap();
        let d1 = register_test_device(&pool, "DEV-SWAP-1").await;
        let d2 = register_test_device(&pool, "DEV-SWAP-2").await;
        assign_devices(&pool, &class.id, &[d1.clone(), d2.clone()]).await.unwrap();
        fill_missing_seats(&pool, &class.id).await.unwrap();

        batch_set_seats(&pool, &class.id, &[
            SeatAssignment { device_id: d1.clone(), seat_no: "2".into() },
            SeatAssignment { device_id: d2.clone(), seat_no: "1".into() },
        ]).await.unwrap();

        let s1: Option<String> = sqlx::query_scalar("SELECT seat_no FROM student_devices WHERE id = ?")
            .bind(&d1).fetch_one(&pool).await.unwrap();
        let s2: Option<String> = sqlx::query_scalar("SELECT seat_no FROM student_devices WHERE id = ?")
            .bind(&d2).fetch_one(&pool).await.unwrap();
        assert_eq!(s1.as_deref(), Some("2"));
        assert_eq!(s2.as_deref(), Some("1"));
    }

    #[tokio::test]
    async fn batch_set_seats_rejects_device_not_in_class() {
        let pool = setup_pool().await;
        let class = create_class(&pool, "本班").await.unwrap();
        let outsider = register_test_device(&pool, "DEV-OUT").await;
        let err = batch_set_seats(&pool, &class.id, &[SeatAssignment {
            device_id: outsider,
            seat_no: "1".into(),
        }])
            .await
            .unwrap_err();
        assert!(err.to_string().contains("不属于"));
    }

    #[tokio::test]
    async fn moving_device_to_another_class_clears_conflicting_seat() {
        let pool = setup_pool().await;
        let class_a = create_class(&pool, "A班").await.unwrap();
        let class_b = create_class(&pool, "B班").await.unwrap();
        let d1 = register_test_device(&pool, "DEV-MOVE-1").await;
        let d2 = register_test_device(&pool, "DEV-KEEP-1").await;
        assign_devices(&pool, &class_a.id, &[d1.clone()]).await.unwrap();
        assign_devices(&pool, &class_b.id, &[d2.clone()]).await.unwrap();
        fill_missing_seats(&pool, &class_a.id).await.unwrap();
        fill_missing_seats(&pool, &class_b.id).await.unwrap();

        assign_devices(&pool, &class_b.id, &[d1.clone()]).await.unwrap();
        let assigned = fill_missing_seats(&pool, &class_b.id).await.unwrap();
        assert_eq!(assigned.len(), 1);
        assert_eq!(assigned[0].device_id, d1);
        assert_eq!(assigned[0].seat_no, "2");
    }
}
