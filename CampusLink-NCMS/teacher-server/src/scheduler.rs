use sqlx::SqlitePool;
use tokio::sync::broadcast;
use tokio::time::{self, Duration, MissedTickBehavior};
use tracing::{info, warn, error};
use chrono::Local;

pub fn start(pool: SqlitePool, ws_tx: broadcast::Sender<String>) {
    tokio::spawn(async move {
        let mut ticker = time::interval(Duration::from_secs(30));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        info!("Scheduler started (interval=30s)");

        loop {
            ticker.tick().await;
            if let Err(e) = check_and_execute(&pool, &ws_tx).await {
                error!("Scheduler execution failed: {}", e);
            }
        }
    });
}

async fn check_and_execute(pool: &SqlitePool, ws_tx: &broadcast::Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
    let schedule_mode: Option<String> = sqlx::query_scalar(
        "SELECT config_value FROM system_configs WHERE config_key = 'schedule_mode'"
    )
    .fetch_optional(pool)
    .await?
    .flatten();

    if schedule_mode.as_deref() != Some("enabled") {
        return Ok(());
    }

    let schedule_time: Option<String> = sqlx::query_scalar(
        "SELECT config_value FROM system_configs WHERE config_key = 'schedule_time'"
    )
    .fetch_optional(pool)
    .await?
    .flatten();

    let Some(ref time_str) = schedule_time else {
        return Ok(());
    };

    let target_mode: Option<String> = sqlx::query_scalar(
        "SELECT config_value FROM system_configs WHERE config_key = 'target_mode'"
    )
    .fetch_optional(pool)
    .await?
    .flatten();

    let Some(target_mode) = target_mode else {
        return Ok(());
    };

    let now = Local::now();
    let current_hhmm = now.format("%H:%M").to_string();

    if current_hhmm == *time_str {
        let last_executed_key = format!("schedule_last_{}", time_str);
        let last_executed: Option<String> = sqlx::query_scalar(
            "SELECT config_value FROM system_configs WHERE config_key = ?"
        )
        .bind(&last_executed_key)
        .fetch_optional(pool)
        .await?
        .flatten();

        let today = now.format("%Y-%m-%d").to_string();
        if last_executed.as_deref() == Some(&today) {
            return Ok(());
        }

        info!("Scheduled mode switch triggered: {} -> {}", time_str, target_mode);

        // Mark as executed for today
        sqlx::query(
            r#"INSERT INTO system_configs (id, config_key, config_value, scope, description, updated_at)
               VALUES (?, ?, ?, 'schedule', 'Auto-set scheduler execution date', datetime('now'))
               ON CONFLICT(config_key) DO UPDATE SET config_value = ?, updated_at = datetime('now')"#
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&last_executed_key)
        .bind(&today)
        .bind(&today)
        .execute(pool)
        .await?;

        let device_rows = sqlx::query_as::<_, (String,)>(
            "SELECT id FROM student_devices WHERE register_status = 'verified' AND online_status = 'online'"
        )
        .fetch_all(pool)
        .await?;

        for (device_id,) in device_rows {
            let msg = format!(
                r#"{{"type":"mode_switch","device_id":"{}","target_mode":"{}","operator":"scheduler","timestamp":{}}}"#,
                device_id,
                target_mode,
                chrono::Utc::now().timestamp()
            );
            if let Err(e) = ws_tx.send(msg) {
                warn!("Failed to broadcast schedule to device {}: {}", device_id, e);
            }
        }
    }

    Ok(())
}
