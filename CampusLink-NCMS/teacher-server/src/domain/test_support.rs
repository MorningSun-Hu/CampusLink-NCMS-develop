use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub async fn setup_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("failed to create in-memory sqlite pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    pool
}

pub async fn register_test_device(pool: &SqlitePool, device_code: &str) -> String {
    crate::domain::device::register_device(
        pool,
        device_code,
        &format!("fp-{}", device_code),
        &format!("host-{}", device_code),
        "192.168.0.1",
        &format!("mac-{}", device_code),
        "test-agent",
    )
    .await
    .expect("device registration failed")
    .0
}

pub async fn create_test_student(pool: &SqlitePool, student_no: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO students (id, student_no, name, password_hash, status, created_at, updated_at)
         VALUES (?, ?, ?, 'unused', 'active', datetime('now'), datetime('now'))"
    )
    .bind(&id)
    .bind(student_no)
    .bind(&format!("学生{}", student_no))
    .execute(pool)
    .await
    .expect("student creation failed");

    id
}
