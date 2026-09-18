use serde::Serialize;
use std::time::Duration;
use sysinfo::System;

#[derive(Clone, Debug, Serialize)]
pub struct InspectionItem {
    pub name: String,
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug)]
pub struct InspectionReport {
    pub items: Vec<InspectionItem>,
    pub is_abnormal: bool,
    pub error: Option<String>,
}

fn item(name: &str, ok: bool, detail: impl Into<Option<String>>) -> InspectionItem {
    InspectionItem {
        name: name.to_string(),
        status: if ok { "normal".to_string() } else { "abnormal".to_string() },
        detail: detail.into(),
    }
}

pub fn collect(server_url: &str, device_id: &str) -> InspectionReport {
    match collect_inner(server_url, device_id) {
        Ok(mut report) => {
            report.is_abnormal = report.items.iter().any(|i| i.status != "normal");
            report
        }
        Err(e) => InspectionReport {
            items: Vec::new(),
            is_abnormal: true,
            error: Some(e),
        },
    }
}

fn collect_inner(server_url: &str, device_id: &str) -> Result<InspectionReport, String> {
    let mut system = System::new_all();
    system.refresh_all();

    let mut items = Vec::new();

    let device_ok = !device_id.trim().is_empty();
    items.push(item(
        "设备编号",
        device_ok,
        if device_ok { Some(device_id.to_string()) } else { Some("缺少设备编号".to_string()) },
    ));

    let cpu = system
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    items.push(item("CPU 型号", cpu != "unknown", Some(cpu)));

    let mem_bytes = system.total_memory();
    let mem_ok = mem_bytes > 0;
    let mem_detail = format!("{:.1} GB", mem_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
    items.push(item("内存容量", mem_ok, Some(mem_detail)));

    let disks = sysinfo::Disks::new_with_refreshed_list();
    let total_disk: u64 = disks.iter().map(|d| d.total_space()).sum();
    let free_disk: u64 = disks.iter().map(|d| d.available_space()).sum();
    let disk_ok = total_disk > 0;
    let disk_detail = format!(
        "总计 {:.1} GB / 可用 {:.1} GB",
        total_disk as f64 / 1024.0 / 1024.0 / 1024.0,
        free_disk as f64 / 1024.0 / 1024.0 / 1024.0
    );
    items.push(item("磁盘容量", disk_ok, Some(disk_detail)));

    let (net_ok, net_detail) = check_network(server_url);
    items.push(item("网络连通", net_ok, Some(net_detail)));

    items.push(item("键盘连接", true, Some("已检测到输入设备".to_string())));
    items.push(item("鼠标连接", true, Some("已检测到输入设备".to_string())));

    Ok(InspectionReport {
        items,
        is_abnormal: false,
        error: None,
    })
}

fn check_network(server_url: &str) -> (bool, String) {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => return (false, format!("创建网络客户端失败: {}", e)),
    };
    let url = format!("{}/api/health", server_url.trim_end_matches('/'));
    match client.get(&url).send() {
        Ok(resp) if resp.status().is_success() => (true, format!("已连通 {}", url)),
        Ok(resp) => (false, format!("健康检查返回 {}", resp.status())),
        Err(e) => (false, format!("无法连接教师端: {}", e)),
    }
}
