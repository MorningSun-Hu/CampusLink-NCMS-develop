use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

const CONCURRENT_AGENTS: usize = 120;
const TEST_DURATION_SECS: u64 = 60;
const HEARTBEAT_INTERVAL_SECS: u64 = 5;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== CampusLink NCMS 并发压测 ===");
    println!("并发数: {} agents", CONCURRENT_AGENTS);
    println!("测试时长: {} 秒", TEST_DURATION_SECS);
    println!();

    let start = Instant::now();
    let http = reqwest::Client::new();
    let (result_tx, mut result_rx) = tokio::sync::mpsc::unbounded_channel();

    println!("[阶段1] 设备注册 ({} 并发)...", CONCURRENT_AGENTS);
    let reg_start = Instant::now();

    for i in 0..CONCURRENT_AGENTS {
        let http = http.clone();
        let tx = result_tx.clone();
        tokio::spawn(async move {
            let fingerprint = format!("fp-{:04}", i);
            let payload = json!({
                "device_code": format!("ST{:04}", i),
                "machine_fingerprint": fingerprint,
                "hostname": format!("node-{:04}", i),
                "ip_address": format!("10.0.{}.{}", (i / 254) + 1, (i % 254) + 1),
                "mac_address": format!("AA:BB:CC:DD:{:02}:{:02}", (i / 256) as u8, (i % 256) as u8),
                "agent_version": "stress-1.0"
            });
            let t0 = Instant::now();
            let res = http.post("http://localhost:8080/api/devices/register")
                .json(&payload)
                .timeout(Duration::from_secs(10))
                .send().await;
            let lat = t0.elapsed().as_micros() as u64;
            match res {
                Ok(r) if r.status().is_success() => {
                    if let Ok(v) = r.json::<serde_json::Value>().await {
                        let did = v["data"]["device_id"].as_str().unwrap_or("").to_string();
                        let sk = v["data"]["session_key"].as_str().unwrap_or("").to_string();
                        let _ = tx.send((i, true, lat, did, sk));
                        return;
                    }
                }
                _ => {}
            }
            let _ = tx.send((i, false, lat, String::new(), String::new()));
        });
    }
    drop(result_tx);

    let mut agents: Vec<(usize, String, String)> = Vec::new();
    let mut reg_ok = 0usize;
    let mut reg_fail = 0usize;
    while let Some((_i, ok, _lat, did, sk)) = result_rx.recv().await {
        if ok { reg_ok += 1; agents.push((_i, did, sk)); }
        else { reg_fail += 1; }
    }

    println!("  注册: {}/{} 成功 ({} 失败), 耗时 {:?}",
        reg_ok, CONCURRENT_AGENTS, reg_fail, reg_start.elapsed());

    println!();
    println!("[阶段2] WebSocket 连接 + 心跳 ({} agents)...", agents.len());

    let ws_start = Instant::now();
    let (ws_tx, mut ws_rx) = tokio::sync::mpsc::unbounded_channel();

    for (i, did, sk) in agents {
        let tx = ws_tx.clone();
        tokio::spawn(async move {
            let url = format!("ws://127.0.0.1:8080/ws?device_id={}&teacher_fingerprint=", did);
            let t0 = Instant::now();
            let ws_conn = connect_async(&url).await;
            let conn_lat = t0.elapsed().as_micros() as u64;

            match ws_conn {
                Ok((mut ws, _)) => {
                    let mut sent = 0u64;
                    let mut ack = 0u64;
                    let end = Instant::now() + Duration::from_secs(TEST_DURATION_SECS);
                    let mut ticker = time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

                    loop {
                        tokio::select! {
                            _ = ticker.tick() => {
                                if Instant::now() >= end { break; }
                                let hb = json!({"device_id":did,"timestamp":chrono::Utc::now().timestamp(),"status":"online","mode":"open","teacher_fingerprint":""});
                                if ws.send(Message::Text(hb.to_string())).await.is_ok() {
                                    sent += 1;
                                } else { break; }
                            }
                            msg = ws.next() => {
                                match msg {
                                    Some(Ok(Message::Text(t))) if t.contains("heartbeat_ack") => { ack += 1; }
                                    Some(Ok(Message::Close(_))) => break,
                                    Some(Err(_)) => break,
                                    None => break,
                                    _ => {}
                                }
                            }
                        }
                    }
                    let _ = ws.close(None).await;
                    let _ = tx.send((i, true, sent, ack, conn_lat));
                }
                Err(_) => {
                    let _ = tx.send((i, false, 0, 0, conn_lat));
                }
            }
        });
    }
    drop(ws_tx);

    let mut ws_ok = 0u64;
    let mut ws_fail = 0u64;
    let mut total_sent = 0u64;
    let mut total_ack = 0u64;
    let mut total_lat = 0u64;
    let mut lat_count = 0u64;

    while let Some((_i, ok, sent, ack, lat)) = ws_rx.recv().await {
        if ok { ws_ok += 1; } else { ws_fail += 1; }
        total_sent += sent;
        total_ack += ack;
        total_lat += lat;
        lat_count += 1;
    }

    let total_elapsed = start.elapsed();

    println!();
    println!("=== 压测结果 ===");
    println!("总耗时: {:?}", total_elapsed);
    println!();
    println!("[HTTP 注册] {}/{} ({:.1}%)",
        reg_ok, CONCURRENT_AGENTS,
        reg_ok as f64 / CONCURRENT_AGENTS as f64 * 100.0);
    println!("[WS 连接]  {}/{} ({:.1}%)",
        ws_ok, reg_ok,
        if reg_ok > 0 { ws_ok as f64 / reg_ok as f64 * 100.0 } else { 0.0 });
    if lat_count > 0 {
        println!("[WS 延迟]  平均 {:.1}ms", (total_lat as f64 / lat_count as f64) / 1000.0);
    }
    println!("[心跳发送] {}", total_sent);
    println!("[心跳 ACK] {} ({:.1}%)",
        total_ack,
        if total_sent > 0 { total_ack as f64 / total_sent as f64 * 100.0 } else { 0.0 });
    println!("[有效 QPS]  {:.1} hb/sec", total_sent as f64 / TEST_DURATION_SECS as f64);

    let passed = reg_ok >= 100 && ws_ok >= 100 && total_sent > 0 && total_ack as f64 / total_sent as f64 > 0.8;
    println!();
    if passed {
        println!("结论: 通过 - 支持 100+ 并发设备");
    } else {
        println!("结论: 未达标");
    }

    Ok(())
}
