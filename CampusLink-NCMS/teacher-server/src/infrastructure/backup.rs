use std::path::Path;
use tracing::{info, warn, error};
use chrono::Local;
use tokio::time::{self, Duration, MissedTickBehavior};

pub fn start() {
    tokio::spawn(async move {
        // Run initial backup on startup
        if let Err(e) = perform_backup().await {
            error!("Initial backup failed: {}", e);
        }
        cleanup_old_backups();

        let mut ticker = time::interval(Duration::from_secs(86400)); // 24h
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;
            if let Err(e) = perform_backup().await {
                error!("Scheduled backup failed: {}", e);
            }
            cleanup_old_backups();
        }
    });
}

async fn perform_backup() -> Result<(), Box<dyn std::error::Error>> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let backup_dir = format!("data/backup/{}", today);
    let source = "data/campuslink.db";
    let dest = format!("{}/campuslink.db", backup_dir);

    if !Path::new(source).exists() {
        warn!("Source database not found: {}", source);
        return Ok(());
    }

    if Path::new(&dest).exists() {
        info!("Backup already exists for today: {}", today);
        return Ok(());
    }

    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup dir {}: {}", backup_dir, e))?;

    std::fs::copy(source, &dest)
        .map_err(|e| format!("Failed to copy database: {}", e))?;

    info!("Database backup created: {}", dest);
    Ok(())
}

fn cleanup_old_backups() {
    let backup_root = "data/backup";
    let Ok(entries) = std::fs::read_dir(backup_root) else {
        return;
    };

    let now = Local::now().naive_local().date();
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if let Ok(date) = chrono::NaiveDate::parse_from_str(&dir_name, "%Y-%m-%d") {
                let age = (now - date).num_days();
                if age > 7 {
                    if let Err(e) = std::fs::remove_dir_all(entry.path()) {
                        warn!("Failed to remove old backup {}: {}", dir_name, e);
                    } else {
                        info!("Removed old backup: {}", dir_name);
                    }
                }
            }
        }
    }
}
