use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::config::Config;

pub const PEER_PORT: u16 = 9998;
pub const PING: &[u8] = b"CAMPUSLINK_PEER_PING";

#[derive(Clone, Debug)]
pub struct NetguardShared {
    pub mode: String,
    pub overlay_locked: bool,
    pub reason: Option<String>,
    pub device_id: String,
    pub lock_password: String,
    pub teacher_server_url: String,
}

impl NetguardShared {
    pub fn from_config(config: &Config) -> Self {
        Self {
            mode: config.current_mode.clone(),
            overlay_locked: false,
            reason: None,
            device_id: config.device_id.clone().unwrap_or_default(),
            lock_password: config.lock_password.clone().unwrap_or_else(|| "admin123".to_string()),
            teacher_server_url: config.teacher_server_url.clone(),
        }
    }

    fn to_config(&self) -> Config {
        let mut cfg = Config::default();
        cfg.teacher_server_url = self.teacher_server_url.clone();
        cfg.device_id = if self.device_id.is_empty() {
            None
        } else {
            Some(self.device_id.clone())
        };
        cfg.lock_password = Some(self.lock_password.clone());
        cfg.current_mode = self.mode.clone();
        cfg
    }
}

#[derive(Clone, Debug)]
pub enum NetguardEvent {
    Locked { reason: String },
    Recovered { reason: String },
    Cleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeerResult {
    NeighborTeacherUp,
    NeighborTeacherDown,
    NoReply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Stay,
    LockPhysical,
    LockFirewall,
    Recover,
    Clear,
}

#[derive(Debug, Deserialize)]
struct PeerPong {
    #[serde(rename = "type")]
    msg_type: String,
    device_id: String,
    teacher_reachable: bool,
}

pub fn decide(
    teaching: bool,
    overlay_locked: bool,
    link_up: Option<bool>,
    teacher_reachable: bool,
    peer: PeerResult,
) -> Decision {
    if overlay_locked {
        if !teaching {
            return Decision::Clear;
        }
        if teacher_reachable && link_up.unwrap_or(true) {
            return Decision::Recover;
        }
        return Decision::Stay;
    }
    if !teaching {
        return Decision::Stay;
    }
    if link_up == Some(false) {
        return Decision::LockPhysical;
    }
    if teacher_reachable {
        return Decision::Stay;
    }
    match peer {
        PeerResult::NeighborTeacherUp => Decision::LockFirewall,
        PeerResult::NeighborTeacherDown | PeerResult::NoReply => Decision::Stay,
    }
}

pub fn physical_link_up() -> Option<bool> {
    #[cfg(target_os = "windows")]
    {
        windows_link_up()
    }
    #[cfg(not(target_os = "windows"))]
    {
        linux_link_up()
    }
}

#[cfg(not(target_os = "windows"))]
fn linux_link_up() -> Option<bool> {
    let entries = std::fs::read_dir("/sys/class/net").ok()?;
    let mut saw = false;
    let mut any_up = false;
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "lo" {
            continue;
        }
        saw = true;
        let path = format!("/sys/class/net/{}/operstate", name);
        if let Ok(state) = std::fs::read_to_string(&path) {
            if state.trim() == "up" {
                any_up = true;
            }
        }
    }
    if !saw {
        return Some(false);
    }
    Some(any_up)
}

#[cfg(target_os = "windows")]
fn windows_link_up() -> Option<bool> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let out = Command::new("netsh")
        .args(["interface", "show", "interface"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut saw = false;
    let mut any_up = false;
    for line in text.lines() {
        let lower = line.to_lowercase();
        if lower.contains("loopback") || line.contains("环回") {
            continue;
        }
        if lower.contains("disconnected") || line.contains("已断开") {
            saw = true;
            continue;
        }
        if lower.contains("connected") || line.contains("已连接") {
            saw = true;
            any_up = true;
        }
    }
    if !saw {
        return None;
    }
    Some(any_up)
}

pub fn spawn(
    shared: Arc<Mutex<NetguardShared>>,
    teacher_reachable: Arc<AtomicBool>,
    unlock_tx: Option<mpsc::UnboundedSender<String>>,
    event_tx: mpsc::UnboundedSender<NetguardEvent>,
) {
    let listen_id = Arc::new(Mutex::new(String::new()));
    let listen_id_for_server = listen_id.clone();
    let reachable_for_server = teacher_reachable.clone();
    tokio::spawn(async move {
        run_peer_listener(reachable_for_server, listen_id_for_server).await;
    });
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            tick_once(&shared, &teacher_reachable, &unlock_tx, &event_tx, &listen_id).await;
        }
    });
}

async fn run_peer_listener(teacher_reachable: Arc<AtomicBool>, device_id: Arc<Mutex<String>>) {
    let sock = match UdpSocket::bind(("0.0.0.0", PEER_PORT)).await {
        Ok(s) => s,
        Err(e) => {
            warn!("netguard peer listen bind {} failed: {}", PEER_PORT, e);
            return;
        }
    };
    let mut buf = [0u8; 512];
    loop {
        match sock.recv_from(&mut buf).await {
            Ok((len, src)) => {
                if &buf[..len] != PING {
                    continue;
                }
                let id = device_id.lock().map(|g| g.clone()).unwrap_or_default();
                let pong = serde_json::json!({
                    "type": "peer_pong",
                    "device_id": id,
                    "teacher_reachable": teacher_reachable.load(Ordering::Relaxed),
                });
                if let Err(e) = sock.send_to(pong.to_string().as_bytes(), src).await {
                    warn!("netguard pong send failed: {}", e);
                }
            }
            Err(e) => {
                warn!("netguard peer recv failed: {}", e);
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

async fn tick_once(
    shared: &Arc<Mutex<NetguardShared>>,
    teacher_reachable: &Arc<AtomicBool>,
    unlock_tx: &Option<mpsc::UnboundedSender<String>>,
    event_tx: &mpsc::UnboundedSender<NetguardEvent>,
    listen_id: &Arc<Mutex<String>>,
) {
    let snap = match shared.lock() {
        Ok(g) => g.clone(),
        Err(_) => return,
    };
    if let Ok(mut id) = listen_id.lock() {
        *id = snap.device_id.clone();
    }

    let reachable = teacher_reachable.load(Ordering::Relaxed);
    let link = physical_link_up();
    let teaching = snap.mode == "teaching";
    let need_probe = teaching && !snap.overlay_locked && !reachable && link != Some(false);
    let peer = if need_probe {
        probe_neighbors(&snap.device_id).await
    } else {
        PeerResult::NoReply
    };

    match decide(teaching, snap.overlay_locked, link, reachable, peer) {
        Decision::Stay => {}
        Decision::LockPhysical => {
            if lock_overlay(shared, unlock_tx, "physical_link").await {
                let _ = event_tx.send(NetguardEvent::Locked {
                    reason: "physical_link".to_string(),
                });
            }
        }
        Decision::LockFirewall => {
            if lock_overlay(shared, unlock_tx, "firewall_block").await {
                let _ = event_tx.send(NetguardEvent::Locked {
                    reason: "firewall_block".to_string(),
                });
            }
        }
        Decision::Recover => {
            let reason = snap.reason.clone().unwrap_or_else(|| "physical_link".to_string());
            crate::mode::kill_lock_process();
            if let Ok(mut g) = shared.lock() {
                g.overlay_locked = false;
                g.reason = None;
            }
            let _ = event_tx.send(NetguardEvent::Recovered {
                reason: reason.clone(),
            });
            report_recover(&snap, &reason).await;
        }
        Decision::Clear => {
            crate::mode::kill_lock_process();
            if let Ok(mut g) = shared.lock() {
                g.overlay_locked = false;
                g.reason = None;
            }
            let _ = event_tx.send(NetguardEvent::Cleared);
        }
    }
}

async fn lock_overlay(
    shared: &Arc<Mutex<NetguardShared>>,
    unlock_tx: &Option<mpsc::UnboundedSender<String>>,
    reason: &str,
) -> bool {
    let mut cfg = {
        let g = match shared.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };
        if g.overlay_locked {
            return false;
        }
        g.to_config()
    };
    if let Err(e) = crate::mode::start_overlay_lock(&mut cfg, unlock_tx).await {
        warn!("netguard overlay lock failed: {}", e);
        return false;
    }
    if !cfg.is_locked {
        warn!("netguard could not start campus-lock");
        return false;
    }
    if let Ok(mut g) = shared.lock() {
        g.overlay_locked = true;
        g.reason = Some(reason.to_string());
    }
    info!("netguard overlay lock started reason={}", reason);
    true
}

async fn probe_neighbors(self_id: &str) -> PeerResult {
    let sock = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            warn!("netguard probe bind failed: {}", e);
            return PeerResult::NoReply;
        }
    };
    if let Err(e) = sock.set_broadcast(true) {
        warn!("netguard probe broadcast flag failed: {}", e);
        return PeerResult::NoReply;
    }
    let target = format!("255.255.255.255:{}", PEER_PORT);
    for attempt in 0..2 {
        if let Err(e) = sock.send_to(PING, &target).await {
            warn!("netguard probe send failed: {}", e);
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut buf = [0u8; 512];
        let mut saw_true = false;
        let mut saw_false = false;
        loop {
            let remain = deadline.saturating_duration_since(Instant::now());
            if remain.is_zero() {
                break;
            }
            match tokio::time::timeout(remain, sock.recv_from(&mut buf)).await {
                Ok(Ok((len, _))) => {
                    let parsed = serde_json::from_slice::<PeerPong>(&buf[..len]).ok();
                    let Some(pong) = parsed else { continue };
                    if pong.msg_type != "peer_pong" {
                        continue;
                    }
                    if !self_id.is_empty() && pong.device_id == self_id {
                        continue;
                    }
                    if pong.teacher_reachable {
                        saw_true = true;
                    } else {
                        saw_false = true;
                    }
                }
                _ => break,
            }
        }
        if saw_true {
            return PeerResult::NeighborTeacherUp;
        }
        if saw_false {
            return PeerResult::NeighborTeacherDown;
        }
        if attempt == 0 {
            info!("netguard peer ping no reply, retrying");
        }
    }
    PeerResult::NoReply
}

async fn report_recover(snap: &NetguardShared, reason: &str) {
    let host = hostname::get()
        .ok()
        .map(|h| h.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| snap.device_id.clone());
    let description = match reason {
        "firewall_block" => format!("设备 {} 故意阻断教师机后已恢复", host),
        _ => format!("设备 {} 物理链路断开后已恢复", host),
    };
    let cfg = snap.to_config();
    let items = vec![crate::inspection::InspectionItem {
        item_name: reason.to_string(),
        status: "abnormal".to_string(),
        description: Some(description.clone()),
    }];
    match crate::inspection::submit_inspection(&cfg, "network", items.clone(), true).await {
        Ok(_) => info!("netguard recovery alert submitted reason={}", reason),
        Err(e) => {
            warn!("netguard recovery alert failed: {}, retrying", e);
            tokio::time::sleep(Duration::from_secs(2)).await;
            if let Err(e2) = crate::inspection::submit_inspection(&cfg, "network", items, true).await {
                warn!("netguard recovery alert retry failed: {}", e2);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_disconnect_locks() {
        assert_eq!(
            decide(true, false, Some(false), false, PeerResult::NoReply),
            Decision::LockPhysical
        );
    }

    #[test]
    fn neighbor_teacher_up_locks_firewall() {
        assert_eq!(
            decide(true, false, Some(true), false, PeerResult::NeighborTeacherUp),
            Decision::LockFirewall
        );
    }

    #[test]
    fn neighbor_all_down_keeps_teaching() {
        assert_eq!(
            decide(true, false, Some(true), false, PeerResult::NeighborTeacherDown),
            Decision::Stay
        );
        assert_eq!(
            decide(true, false, Some(true), false, PeerResult::NoReply),
            Decision::Stay
        );
    }

    #[test]
    fn recover_when_link_and_teacher_back() {
        assert_eq!(
            decide(true, true, Some(true), true, PeerResult::NoReply),
            Decision::Recover
        );
    }
}
