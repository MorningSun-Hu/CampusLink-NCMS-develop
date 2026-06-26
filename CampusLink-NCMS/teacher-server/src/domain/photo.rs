use sqlx::{SqlitePool, FromRow};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PhotoRow {
    pub id: String,
    pub alert_id: Option<i64>,
    pub inspection_id: Option<String>,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub upload_time: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PhotoRecord {
    pub id: String,
    pub alert_id: Option<i64>,
    pub inspection_id: Option<String>,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub upload_time: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PhotoUploadMetadata {
    pub alert_id: Option<i64>,
    pub inspection_id: Option<String>,
    pub description: Option<String>,
}

pub async fn create_photo(pool: &SqlitePool, file_path: &str, file_size: i64, mime_type: &str, alert_id: Option<i64>, inspection_id: Option<&str>, description: Option<&str>) -> Result<PhotoRecord> {
    let photo_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO photos (id, alert_id, inspection_id, file_path, file_size, mime_type, description, upload_time)
           VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))"#
    )
    .bind(&photo_id)
    .bind(alert_id)
    .bind(inspection_id)
    .bind(file_path)
    .bind(file_size)
    .bind(mime_type)
    .bind(description)
    .execute(pool)
    .await?;

    let row = sqlx::query_as::<_, PhotoRow>(
        "SELECT * FROM photos WHERE id = ?"
    )
    .bind(&photo_id)
    .fetch_one(pool)
    .await?;

    Ok(PhotoRecord {
        id: row.id,
        alert_id: row.alert_id,
        inspection_id: row.inspection_id,
        file_path: row.file_path,
        file_size: row.file_size,
        mime_type: row.mime_type,
        upload_time: row.upload_time,
        description: row.description,
    })
}

pub async fn list_photos(pool: &SqlitePool, inspection_id: Option<&str>) -> Result<Vec<PhotoRecord>> {
    let rows = if let Some(id) = inspection_id {
        sqlx::query_as::<_, PhotoRow>(
            "SELECT * FROM photos WHERE inspection_id = ? ORDER BY upload_time DESC"
        )
        .bind(id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, PhotoRow>(
            "SELECT * FROM photos ORDER BY upload_time DESC LIMIT 50"
        )
        .fetch_all(pool)
        .await?
    };

    Ok(rows.into_iter().map(|r| PhotoRecord {
        id: r.id,
        alert_id: r.alert_id,
        inspection_id: r.inspection_id,
        file_path: r.file_path,
        file_size: r.file_size,
        mime_type: r.mime_type,
        upload_time: r.upload_time,
        description: r.description,
    }).collect())
}

pub async fn get_photo_by_id(pool: &SqlitePool, photo_id: &str) -> Result<PhotoRecord> {
    let row = sqlx::query_as::<_, PhotoRow>(
        "SELECT * FROM photos WHERE id = ?"
    )
    .bind(photo_id)
    .fetch_one(pool)
    .await?;

    Ok(PhotoRecord {
        id: row.id,
        alert_id: row.alert_id,
        inspection_id: row.inspection_id,
        file_path: row.file_path,
        file_size: row.file_size,
        mime_type: row.mime_type,
        upload_time: row.upload_time,
        description: row.description,
    })
}
