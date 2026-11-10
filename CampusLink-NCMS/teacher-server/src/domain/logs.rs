use sqlx::SqlitePool;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub log_type: Option<String>,
    pub device_id: Option<String>,
    pub operator: Option<String>,
    pub action: Option<String>,
    pub detail: Option<String>,
    pub created_at: String,
}

pub async fn query_logs(
    pool: &SqlitePool,
    log_type: Option<&str>,
    device_id: Option<&str>,
    date_from: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<LogEntry>, i64)> {
    let offset = (page - 1) * page_size;

    let mut conditions = Vec::new();
    if log_type.is_some() {
        conditions.push("log_type = ?");
    }
    if device_id.is_some() {
        conditions.push("device_id = ?");
    }
    if date_from.is_some() {
        conditions.push("created_at >= ?");
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) as count FROM operation_logs {}", where_clause);
    let list_sql = format!(
        "SELECT * FROM operation_logs {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_clause
    );

    let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql);
    if let Some(lt) = log_type { count_query = count_query.bind(lt); }
    if let Some(did) = device_id { count_query = count_query.bind(did); }
    if let Some(df) = date_from { count_query = count_query.bind(df); }
    let count: (i64,) = count_query.fetch_one(pool).await?;

    let mut list_query = sqlx::query_as::<_, LogEntry>(&list_sql);
    if let Some(lt) = log_type { list_query = list_query.bind(lt); }
    if let Some(did) = device_id { list_query = list_query.bind(did); }
    if let Some(df) = date_from { list_query = list_query.bind(df); }
    list_query = list_query.bind(page_size).bind(offset);

    let rows = list_query.fetch_all(pool).await?;

    Ok((rows, count.0))
}

pub async fn export_logs_csv(
    pool: &SqlitePool,
    log_type: Option<&str>,
    device_id: Option<&str>,
    date_from: Option<&str>,
) -> Result<Vec<u8>> {
    let (items, _) = query_logs(pool, log_type, device_id, date_from, 1, 10000).await?;

    let mut csv = String::from("ID,日志类型,设备ID,操作人,操作,详情,时间\n");
    for item in &items {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            item.id,
            item.log_type.as_deref().unwrap_or(""),
            item.device_id.as_deref().unwrap_or(""),
            item.operator.as_deref().unwrap_or(""),
            item.action.as_deref().unwrap_or(""),
            item.detail.as_deref().unwrap_or("").replace(',', "，"),
            item.created_at,
        ));
    }

    Ok(csv.into_bytes())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLogFileInfo {
    pub device_id: String,
    pub file_name: String,
    pub size: u64,
}

pub fn agent_logs_root() -> std::path::PathBuf {
    std::path::PathBuf::from("logs").join("students")
}

pub fn safe_log_component(name: &str) -> Option<String> {
    if name.is_empty() || name == "." || name == ".." {
        return None;
    }
    if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_') {
        Some(name.to_string())
    } else {
        None
    }
}

pub fn save_agent_log_file(root: &std::path::Path, device_id: &str, file_name: &str, content: &str) -> Result<std::path::PathBuf> {
    let device = safe_log_component(device_id).ok_or_else(|| anyhow::anyhow!("invalid device id"))?;
    let name = safe_log_component(file_name).ok_or_else(|| anyhow::anyhow!("invalid file name"))?;
    let dir = root.join(device);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(name);
    std::fs::write(&path, content)?;
    Ok(path)
}

pub fn list_agent_log_files(root: &std::path::Path) -> Result<Vec<AgentLogFileInfo>> {
    let mut items = Vec::new();
    if !root.is_dir() {
        return Ok(items);
    }
    for device_entry in std::fs::read_dir(root)? {
        let device_entry = device_entry?;
        if !device_entry.file_type()?.is_dir() {
            continue;
        }
        let device_id = device_entry.file_name().to_string_lossy().to_string();
        for file_entry in std::fs::read_dir(device_entry.path())? {
            let file_entry = file_entry?;
            if !file_entry.file_type()?.is_file() {
                continue;
            }
            let file_name = file_entry.file_name().to_string_lossy().to_string();
            let size = file_entry.metadata()?.len();
            items.push(AgentLogFileInfo {
                device_id: device_id.clone(),
                file_name,
                size,
            });
        }
    }
    items.sort_by(|a, b| a.device_id.cmp(&b.device_id).then(a.file_name.cmp(&b.file_name)));
    Ok(items)
}

pub fn read_agent_log_file(root: &std::path::Path, device_id: &str, file_name: &str) -> Result<Vec<u8>> {
    let device = safe_log_component(device_id).ok_or_else(|| anyhow::anyhow!("invalid device id"))?;
    let name = safe_log_component(file_name).ok_or_else(|| anyhow::anyhow!("invalid file name"))?;
    let path = root.join(device).join(name);
    Ok(std::fs::read(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal_names() {
        assert!(safe_log_component("..").is_none());
        assert!(safe_log_component("../x").is_none());
        assert!(safe_log_component("a/b").is_none());
        assert_eq!(
            safe_log_component("student-agent.2026-10-21").as_deref(),
            Some("student-agent.2026-10-21")
        );
    }

    #[test]
    fn saves_and_lists_agent_logs() {
        let dir = std::env::temp_dir().join(format!("campuslink-agent-logs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        save_agent_log_file(&dir, "dev1", "student-agent.2026-10-21", "hello").unwrap();
        let items = list_agent_log_files(&dir).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].device_id, "dev1");
        assert_eq!(items[0].file_name, "student-agent.2026-10-21");
        let data = read_agent_log_file(&dir, "dev1", "student-agent.2026-10-21").unwrap();
        assert_eq!(data, b"hello");
        assert!(save_agent_log_file(&dir, "../x", "a", "no").is_err());
    }
}
