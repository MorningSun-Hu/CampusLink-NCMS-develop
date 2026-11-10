use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::config::Config;

const MAX_FILE_BYTES: u64 = 1_500_000;
const MAX_FILES: usize = 8;

#[derive(Debug, Serialize)]
pub struct AgentLogFile {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct AgentLogUpload {
    pub device_id: String,
    pub files: Vec<AgentLogFile>,
}

pub fn collect_log_files(log_dir: &Path, extra_files: &[&Path]) -> Vec<(String, String)> {
    let mut files = Vec::new();
    if log_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(log_dir) {
            let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
            paths.sort();
            for path in paths.into_iter().rev() {
                if files.len() >= MAX_FILES {
                    break;
                }
                if !path.is_file() {
                    continue;
                }
                let name = match path.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                if let Some(content) = read_log_text(&path) {
                    files.push((name, content));
                }
            }
        }
    }
    for extra in extra_files {
        if files.len() >= MAX_FILES {
            break;
        }
        if extra.is_file() {
            if let Some(name) = extra.file_name().and_then(|n| n.to_str()) {
                if files.iter().any(|(existing, _)| existing == name) {
                    continue;
                }
                if let Some(content) = read_log_text(extra) {
                    files.push((name.to_string(), content));
                }
            }
        }
    }
    files
}

fn read_log_text(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    if len > MAX_FILE_BYTES {
        file.seek(SeekFrom::End(-(MAX_FILE_BYTES as i64))).ok()?;
    }
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    Some(String::from_utf8_lossy(&buf).into_owned())
}

pub async fn upload_agent_logs(config: &Config) -> Result<()> {
    let device_id = config
        .device_id
        .clone()
        .unwrap_or_default();
    if device_id.is_empty() {
        warn!("Skip log upload: device is not registered");
        return Ok(());
    }
    let extra = [
        Path::new("campus-lock-stderr.log"),
        Path::new("logs/campus-guard.log"),
    ];
    let files = collect_log_files(Path::new("logs"), &extra)
        .into_iter()
        .map(|(name, content)| AgentLogFile { name, content })
        .collect::<Vec<_>>();
    if files.is_empty() {
        warn!("No local log files to upload");
        return Ok(());
    }
    let url = format!("{}/api/logs/agent-files", config.teacher_server_url);
    let body = AgentLogUpload {
        device_id: device_id.clone(),
        files,
    };
    info!("Uploading {} log file(s) from device {}", body.files.len(), device_id);
    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        warn!("Log upload HTTP {}", resp.status());
    } else {
        info!("Log upload completed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "campuslink-log-collect-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn collects_text_logs_and_skips_missing_extras() {
        let dir = temp_dir();
        let path = dir.join("student-agent.2026-10-21");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "hello-log").unwrap();
        let files = collect_log_files(&dir, &[Path::new("missing.log")]);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "student-agent.2026-10-21");
        assert!(files[0].1.contains("hello-log"));
    }
}
