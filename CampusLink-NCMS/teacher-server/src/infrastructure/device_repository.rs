use sqlx::SqlitePool;
use anyhow::Result;

pub async fn get_config_value_string(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let result: Option<String> = sqlx::query_scalar(
        "SELECT config_value FROM system_configs WHERE config_key = ?"
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

pub async fn get_config_value_u32(pool: &SqlitePool, key: &str) -> Result<Option<u32>> {
    let result: Option<String> = sqlx::query_scalar(
        "SELECT config_value FROM system_configs WHERE config_key = ?"
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;

    Ok(result.and_then(|v| v.parse::<u32>().ok()))
}
