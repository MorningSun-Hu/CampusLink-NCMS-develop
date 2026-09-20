#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod inspect;
mod keyhook;

use eframe::egui;
use inspect::{InspectionItem, InspectionReport};
use serde_json::Value;
use std::io::Write;
use std::sync::mpsc;
use std::time::Duration;

const EXIT_OK: i32 = 0;
const EXIT_RETRY: i32 = 1;
const EXIT_USAGE: i32 = 2;
const EXIT_TEACHER: i32 = 10;

enum Mode {
    Open,
    Teaching,
}

#[derive(Clone, Copy, PartialEq)]
enum Stage {
    Identity,
    ChangePassword,
    Inspect,
}

enum WorkerResult {
    IdentityOk { student_id: String },
    NeedPasswordChange { student_id: String },
    Inspected(InspectionReport),
    CheckedIn(String),
}

struct CheckinApp {
    mode: Mode,
    stage: Stage,
    server_url: String,
    device_id: String,
    lock_password: String,
    name_input: String,
    student_no_input: String,
    password_input: String,
    new_password_input: String,
    pending_student_id: String,
    identity_student_id: String,
    message: String,
    busy: bool,
    inspect_items: Vec<InspectionItem>,
    inspect_error: String,
    inspect_fail_count: u32,
    is_abnormal: bool,
    teacher_input: String,
    show_teacher: bool,
    teacher_attempts: u32,
    tx: mpsc::Sender<Result<WorkerResult, String>>,
    rx: mpsc::Receiver<Result<WorkerResult, String>>,
    ime_on: bool,
    submitted: bool,
}

impl CheckinApp {
    fn new(mode: Mode, server_url: String, device_id: String, lock_password: String) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            mode,
            stage: Stage::Identity,
            server_url,
            device_id,
            lock_password,
            name_input: String::new(),
            student_no_input: String::new(),
            password_input: String::new(),
            new_password_input: String::new(),
            pending_student_id: String::new(),
            identity_student_id: String::new(),
            message: String::new(),
            busy: false,
            inspect_items: Vec::new(),
            inspect_error: String::new(),
            inspect_fail_count: 0,
            is_abnormal: false,
            teacher_input: String::new(),
            show_teacher: false,
            teacher_attempts: 0,
            tx,
            rx,
            ime_on: false,
            submitted: false,
        }
    }

    fn start_identity(&mut self) {
        if self.busy {
            return;
        }
        let mode = match self.mode {
            Mode::Open => "open",
            Mode::Teaching => "teaching",
        };
        let server_url = self.server_url.clone();
        let name = self.name_input.clone();
        let student_no = self.student_no_input.clone();
        let password = self.password_input.clone();
        let tx = self.tx.clone();
        self.busy = true;
        self.message = "正在验证身份...".to_string();

        std::thread::spawn(move || {
            let result = match mode {
                "open" => {
                    if name.trim().is_empty() {
                        Err("请输入使用者姓名".to_string())
                    } else {
                        Ok(WorkerResult::IdentityOk {
                            student_id: name.trim().to_string(),
                        })
                    }
                }
                "teaching" => {
                    if student_no.trim().is_empty() || password.is_empty() {
                        Err("请输入学号和密码".to_string())
                    } else {
                        match do_student_login(&server_url, &student_no, &password) {
                            Ok((student_id, password_set)) => {
                                if !password_set {
                                    Ok(WorkerResult::NeedPasswordChange { student_id })
                                } else {
                                    Ok(WorkerResult::IdentityOk { student_id })
                                }
                            }
                            Err(e) => Err(e),
                        }
                    }
                }
                _ => Err("未知签到模式".to_string()),
            };
            let _ = tx.send(result);
        });
    }

    fn start_change_password(&mut self) {
        if self.busy {
            return;
        }
        if self.new_password_input.len() < 4 {
            self.message = "新密码长度不得少于 4 位".to_string();
            return;
        }
        let server_url = self.server_url.clone();
        let student_id = self.pending_student_id.clone();
        let old_password = self.password_input.clone();
        let new_password = self.new_password_input.clone();
        let tx = self.tx.clone();
        self.busy = true;
        self.message = "正在修改密码...".to_string();

        std::thread::spawn(move || {
            let result = do_change_password(&server_url, &student_id, &old_password, &new_password)
                .map(|_| WorkerResult::IdentityOk { student_id });
            let _ = tx.send(result);
        });
    }

    fn start_inspect(&mut self) {
        if self.busy {
            return;
        }
        let server_url = self.server_url.clone();
        let device_id = self.device_id.clone();
        let tx = self.tx.clone();
        self.busy = true;
        self.message = "正在采集环境与设备信息...".to_string();
        std::thread::spawn(move || {
            let report = inspect::collect(&server_url, &device_id);
            let _ = tx.send(Ok(WorkerResult::Inspected(report)));
        });
    }

    fn start_submit(&mut self) {
        if self.busy || self.submitted {
            return;
        }
        let server_url = self.server_url.clone();
        let device_id = self.device_id.clone();
        let student_id = self.identity_student_id.clone();
        let items = self.inspect_items.clone();
        let is_abnormal = self.is_abnormal || !self.inspect_error.is_empty();
        let tx = self.tx.clone();
        self.busy = true;
        self.submitted = true;
        self.message = "正在提交签到...".to_string();
        std::thread::spawn(move || {
            let result = do_checkin(&server_url, &device_id, &student_id, &items, is_abnormal)
                .map(WorkerResult::CheckedIn);
            let _ = tx.send(result);
        });
    }

    fn try_teacher_unlock(&mut self) {
        if self.teacher_input == self.lock_password && !self.lock_password.is_empty() {
            close_session(&self.server_url, &self.device_id, Some("teacher_unlock"));
            let _ = writeln!(std::io::stderr(), "campus-checkin: teacher unlock");
            exit_with(EXIT_TEACHER);
        }
        self.teacher_attempts += 1;
        self.teacher_input.clear();
        self.message = format!("教师密码错误，已尝试 {} 次", self.teacher_attempts);
    }

    fn apply_worker(&mut self, result: Result<WorkerResult, String>) {
        self.busy = false;
        match result {
            Ok(WorkerResult::IdentityOk { student_id }) => {
                self.identity_student_id = student_id;
                self.stage = Stage::Inspect;
                self.message = String::new();
                self.start_inspect();
            }
            Ok(WorkerResult::NeedPasswordChange { student_id }) => {
                self.pending_student_id = student_id;
                self.stage = Stage::ChangePassword;
                self.message = "首次签到，请设置新密码".to_string();
            }
            Ok(WorkerResult::Inspected(report)) => {
                if let Some(err) = report.error {
                    self.inspect_fail_count += 1;
                    self.inspect_error = err.clone();
                    self.is_abnormal = true;
                    self.message = format!("检查采集失败：{}", err);
                } else {
                    self.inspect_error.clear();
                    self.inspect_items = report.items;
                    self.is_abnormal = report.is_abnormal;
                    self.message = if self.is_abnormal {
                        "检查存在异常项，仍可提交签到进入桌面".to_string()
                    } else {
                        "检查完成，请确认后提交签到".to_string()
                    };
                }
            }
            Ok(WorkerResult::CheckedIn(msg)) => {
                let _ = writeln!(std::io::stderr(), "campus-checkin: SUCCESS {}", msg);
                exit_with(EXIT_OK);
            }
            Err(e) => {
                self.message = e;
                self.submitted = false;
            }
        }
    }
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())
}

fn do_student_login(server_url: &str, student_no: &str, password: &str) -> Result<(String, bool), String> {
    let client = http_client()?;
    let url = format!("{}/api/auth/student-login", server_url);
    let body = serde_json::json!({
        "student_no": student_no,
        "password": password,
    });
    let resp = client.post(&url).json(&body).send().map_err(|e| format!("网络错误：{}", e))?;
    let json: Value = resp.json().map_err(|e| format!("响应解析失败：{}", e))?;
    if json["code"].as_i64() == Some(0) {
        let data = &json["data"];
        let student_id = data["student_id"].as_str().or_else(|| data["studentId"].as_str()).unwrap_or("").to_string();
        let password_set = data["password_set"].as_bool().or_else(|| data["passwordSet"].as_bool()).unwrap_or(false);
        if student_id.is_empty() {
            Err("登录响应缺少学生 ID".to_string())
        } else {
            Ok((student_id, password_set))
        }
    } else {
        Err(json["message"].as_str().unwrap_or("验证失败").to_string())
    }
}

fn do_change_password(server_url: &str, student_id: &str, old_password: &str, new_password: &str) -> Result<(), String> {
    let client = http_client()?;
    let url = format!("{}/api/auth/student-set-password", server_url);
    let body = serde_json::json!({
        "student_id": student_id,
        "old_password": old_password,
        "new_password": new_password,
    });
    let resp = client.post(&url).json(&body).send().map_err(|e| format!("网络错误：{}", e))?;
    let json: Value = resp.json().map_err(|e| format!("响应解析失败：{}", e))?;
    if json["code"].as_i64() == Some(0) {
        Ok(())
    } else {
        Err(json["message"].as_str().unwrap_or("修改密码失败").to_string())
    }
}

fn do_checkin(
    server_url: &str,
    device_id: &str,
    student_id: &str,
    items: &[InspectionItem],
    is_abnormal: bool,
) -> Result<String, String> {
    let client = http_client()?;
    let url = format!("{}/api/attendance/check-in", server_url);
    let inspection_items: Vec<Value> = items
        .iter()
        .map(|i| {
            serde_json::json!({
                "name": i.name,
                "status": i.status,
                "detail": i.detail,
            })
        })
        .collect();
    let body = serde_json::json!({
        "device_id": device_id,
        "student_id": student_id,
        "timestamp": chrono::Utc::now().timestamp(),
        "inspection_items": inspection_items,
        "is_abnormal": is_abnormal,
    });
    let resp = client.post(&url).json(&body).send().map_err(|e| format!("网络错误: {}", e))?;
    let json: Value = resp.json().map_err(|e| format!("响应解析失败: {}", e))?;
    if json["code"].as_i64() == Some(0) {
        Ok("签到成功".to_string())
    } else {
        Err(json["message"].as_str().unwrap_or("签到失败").to_string())
    }
}

fn close_session(server_url: &str, device_id: &str, reason: Option<&str>) {
    let client = match http_client() {
        Ok(c) => c,
        Err(_) => return,
    };
    let url = format!("{}/api/usage/close", server_url);
    let mut body = serde_json::json!({ "device_id": device_id });
    if let Some(r) = reason {
        body["reason"] = serde_json::Value::String(r.to_string());
    }
    let _ = client.post(&url).json(&body).send();
}

fn exit_with(code: i32) -> ! {
    keyhook::stop_keyboard_hook();
    std::process::exit(code);
}

impl eframe::App for CheckinApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            close_session(&self.server_url, &self.device_id, None);
            exit_with(EXIT_RETRY);
        }

        if let Ok(result) = self.rx.try_recv() {
            self.apply_worker(result);
        }

        let bg = egui::Color32::from_rgb(18, 22, 32);
        let card = egui::Color32::from_rgb(28, 34, 48);
        let accent = egui::Color32::from_rgb(70, 140, 230);
        let muted = egui::Color32::from_rgb(160, 168, 184);
        let danger = egui::Color32::from_rgb(220, 80, 80);
        let ok = egui::Color32::from_rgb(80, 200, 120);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(bg))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(48.0);
                    ui.heading(
                        egui::RichText::new("设备签到准入")
                            .size(36.0)
                            .color(egui::Color32::WHITE),
                    );
                    ui.add_space(6.0);
                    let subtitle = match (self.stage, &self.mode) {
                        (Stage::Identity, Mode::Open) => "开放模式：请填写使用者姓名",
                        (Stage::Identity, Mode::Teaching) => "授课模式：请使用学号登录",
                        (Stage::ChangePassword, _) => "首次签到，请设置新密码",
                        (Stage::Inspect, _) => "环境与设备检查完成后，确认结果并进入桌面",
                    };
                    ui.label(egui::RichText::new(subtitle).size(16.0).color(muted));
                    ui.add_space(24.0);
                });

                let card_w: f32 = if self.stage == Stage::Inspect { 720.0 } else { 560.0 };
                let card_w = card_w.min(ui.available_width() - 40.0).max(280.0);
                ui.vertical_centered(|ui| {
                    egui::Frame::NONE
                        .fill(card)
                        .corner_radius(12.0)
                        .inner_margin(egui::Margin::same(24))
                        .show(ui, |ui| {
                            ui.set_min_width((card_w - 48.0).max(240.0));
                            ui.set_max_width((card_w - 48.0).max(240.0));
                            match self.stage {
                                Stage::Identity => self.ui_identity(ui, accent),
                                Stage::ChangePassword => self.ui_password(ui, accent),
                                Stage::Inspect => self.ui_inspect(ui, accent, muted, danger, ok),
                            }

                            self.ui_status_message(ui, muted, danger, ok);

                            if self.stage != Stage::Inspect {
                                ui.add_space(18.0);
                                ui.separator();
                                ui.add_space(8.0);
                                self.ui_teacher_unlock(ui, muted);
                            }
                        });
                });
            });

        let ime_commit = ctx.input(|i| {
            i.events.iter().any(|e| matches!(e, egui::Event::Ime(egui::ImeEvent::Commit(_))))
        });
        if !ime_commit && ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !self.busy {
            match self.stage {
                Stage::Identity => self.start_identity(),
                Stage::ChangePassword => self.start_change_password(),
                Stage::Inspect => {
                    if self.can_enter_desktop() {
                        self.start_submit();
                    }
                }
            }
        }
        if self.busy {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}

impl CheckinApp {
    fn can_enter_desktop(&self) -> bool {
        !self.busy && !self.submitted && (!self.inspect_items.is_empty() || self.inspect_fail_count >= 2)
    }

    fn ui_status_message(&self, ui: &mut egui::Ui, muted: egui::Color32, danger: egui::Color32, ok: egui::Color32) {
        if self.message.is_empty() {
            return;
        }
        ui.add_space(12.0);
        let color = if self.message.contains("成功") || self.message.contains("完成") {
            ok
        } else if self.message.contains("异常") {
            egui::Color32::from_rgb(230, 170, 70)
        } else if self.busy {
            muted
        } else {
            danger
        };
        ui.label(egui::RichText::new(&self.message).size(14.0).color(color));
    }

    fn ui_teacher_unlock(&mut self, ui: &mut egui::Ui, muted: egui::Color32) {
        if ui.link(egui::RichText::new("教师解锁").color(muted)).clicked() {
            self.show_teacher = !self.show_teacher;
        }
        if self.show_teacher {
            ui.add_space(8.0);
            let teacher_resp = ui.add_sized(
                [ui.available_width(), 32.0],
                egui::TextEdit::singleline(&mut self.teacher_input)
                    .password(true)
                    .hint_text("教师超级密码"),
            );
            if teacher_resp.has_focus() {
                disable_ime();
                self.ime_on = false;
            }
            ui.add_space(8.0);
            if ui
                .add_sized([ui.available_width(), 32.0], egui::Button::new("确认解锁"))
                .clicked()
            {
                self.try_teacher_unlock();
            }
        }
    }

    fn ui_inspect_actions(&mut self, ui: &mut egui::Ui, accent: egui::Color32, muted: egui::Color32) {
        let can_enter = self.can_enter_desktop();
        let primary = if self.busy {
            if self.inspect_items.is_empty() && self.inspect_error.is_empty() {
                "正在检查设备..."
            } else {
                "正在提交签到..."
            }
        } else {
            "进入桌面"
        };
        let primary_fill = if can_enter {
            accent
        } else {
            egui::Color32::from_rgb(70, 78, 94)
        };
        if ui
            .add_enabled(
                can_enter,
                egui::Button::new(egui::RichText::new(primary).size(18.0).color(egui::Color32::WHITE))
                    .fill(primary_fill)
                    .min_size(egui::vec2(ui.available_width(), 46.0)),
            )
            .clicked()
        {
            self.start_submit();
        }
        ui.add_space(8.0);
        if ui
            .add_enabled(
                !self.busy,
                egui::Button::new(egui::RichText::new("重新检查").size(15.0))
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
            )
            .clicked()
        {
            self.start_inspect();
        }
        ui.add_space(8.0);
        self.ui_teacher_unlock(ui, muted);
    }

    fn ui_identity(&mut self, ui: &mut egui::Ui, accent: egui::Color32) {
        match self.mode {
            Mode::Open => {
                ui.label(egui::RichText::new("使用者姓名").color(accent));
                ui.add_space(6.0);
                let name_resp = ui.add_sized(
                    [ui.available_width(), 36.0],
                    egui::TextEdit::singleline(&mut self.name_input).hint_text("请输入姓名"),
                );
                if name_resp.has_focus() && !self.ime_on {
                    enable_chinese_ime();
                    self.ime_on = true;
                } else if !name_resp.has_focus() && self.ime_on {
                    disable_ime();
                    self.ime_on = false;
                }
            }
            Mode::Teaching => {
                ui.label(egui::RichText::new("学号").color(accent));
                ui.add_space(6.0);
                let no_resp = ui.add_sized(
                    [ui.available_width(), 36.0],
                    egui::TextEdit::singleline(&mut self.student_no_input).hint_text("请输入学号"),
                );
                ui.add_space(10.0);
                ui.label(egui::RichText::new("密码").color(accent));
                ui.add_space(6.0);
                let pwd_resp = ui.add_sized(
                    [ui.available_width(), 36.0],
                    egui::TextEdit::singleline(&mut self.password_input)
                        .password(true)
                        .hint_text("初始密码 123456"),
                );
                if no_resp.has_focus() || pwd_resp.has_focus() {
                    disable_ime();
                    self.ime_on = false;
                }
            }
        }
        ui.add_space(16.0);
        let label = if self.busy { "处理中..." } else { "下一步" };
        if ui.add_enabled(!self.busy, egui::Button::new(egui::RichText::new(label).size(16.0)).min_size(egui::vec2(ui.available_width(), 40.0))).clicked() {
            self.start_identity();
        }
    }

    fn ui_password(&mut self, ui: &mut egui::Ui, accent: egui::Color32) {
        ui.label(egui::RichText::new("新密码").color(accent));
        ui.add_space(6.0);
        let pwd_resp = ui.add_sized(
            [ui.available_width(), 36.0],
            egui::TextEdit::singleline(&mut self.new_password_input)
                .password(true)
                .hint_text("不少于 4 位"),
        );
        if pwd_resp.has_focus() {
            disable_ime();
            self.ime_on = false;
        }
        ui.add_space(16.0);
        let label = if self.busy { "处理中..." } else { "确认修改" };
        if ui.add_enabled(!self.busy, egui::Button::new(egui::RichText::new(label).size(16.0)).min_size(egui::vec2(ui.available_width(), 40.0))).clicked() {
            self.start_change_password();
        }
    }

    fn ui_inspect(&mut self, ui: &mut egui::Ui, accent: egui::Color32, muted: egui::Color32, danger: egui::Color32, ok: egui::Color32) {
        if !self.inspect_items.is_empty() {
            egui::ScrollArea::vertical()
                .max_height(240.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    egui::Grid::new("inspect_grid")
                        .num_columns(3)
                        .spacing([16.0, 10.0])
                        .min_col_width(72.0)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("检查项").color(accent));
                            ui.label(egui::RichText::new("结果").color(accent));
                            ui.label(egui::RichText::new("详情").color(accent));
                            ui.end_row();
                            for item in &self.inspect_items {
                                let (mark, color) = if item.status == "normal" {
                                    ("正常", ok)
                                } else {
                                    ("异常", danger)
                                };
                                ui.label(egui::RichText::new(&item.name).color(egui::Color32::WHITE));
                                ui.label(egui::RichText::new(mark).color(color));
                                let detail = item.detail.clone().unwrap_or_default();
                                ui.label(
                                    egui::RichText::new(detail)
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(140, 148, 164)),
                                );
                                ui.end_row();
                            }
                        });
                });
        } else if !self.inspect_error.is_empty() {
            ui.label(egui::RichText::new(&self.inspect_error).size(15.0).color(danger));
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("可点击「重新检查」，连续失败两次后仍可进入桌面")
                    .size(13.0)
                    .color(egui::Color32::from_rgb(140, 148, 164)),
            );
        } else {
            ui.label(egui::RichText::new("正在检查设备与网络，请稍候...").size(15.0).color(accent));
        }
        ui.add_space(16.0);
        self.ui_inspect_actions(ui, accent, muted);
    }
}

fn enable_chinese_ime() {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::core::w;
        use windows::Win32::Foundation::BOOL;
        use windows::Win32::UI::Input::Ime::{ImmGetContext, ImmReleaseContext, ImmSetOpenStatus};
        use windows::Win32::UI::Input::KeyboardAndMouse::{LoadKeyboardLayoutW, KLF_ACTIVATE};
        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

        let _ = LoadKeyboardLayoutW(w!("00000804"), KLF_ACTIVATE);
        let hwnd = GetForegroundWindow();
        let himc = ImmGetContext(hwnd);
        if !himc.0.is_null() {
            let _ = ImmSetOpenStatus(himc, BOOL::from(true));
            let _ = ImmReleaseContext(hwnd, himc);
        }
    }
}

fn disable_ime() {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::Foundation::BOOL;
        use windows::Win32::UI::Input::Ime::{ImmGetContext, ImmReleaseContext, ImmSetOpenStatus};
        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

        let hwnd = GetForegroundWindow();
        let himc = ImmGetContext(hwnd);
        if !himc.0.is_null() {
            let _ = ImmSetOpenStatus(himc, BOOL::from(false));
            let _ = ImmReleaseContext(hwnd, himc);
        }
    }
}

fn setup_cjk_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let candidates = [
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\msyhbd.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
        "C:\\Windows\\Fonts\\simsun.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert("cjk".to_owned(), egui::FontData::from_owned(bytes).into());
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "cjk".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "cjk".to_owned());
            ctx.set_fonts(fonts);
            return;
        }
    }
}

fn main() {
    keyhook::start_keyboard_hook();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        let msg = "Usage: campus-checkin <server_url> <device_id> <mode> [lock_password]";
        let _ = writeln!(std::io::stderr(), "{}", msg);
        exit_with(EXIT_USAGE);
    }

    let server_url = args[1].clone();
    let device_id = args[2].clone();
    let mode_str = args[3].clone();
    let lock_password = args.get(4).cloned().unwrap_or_else(|| "admin123".to_string());
    let mode = match mode_str.as_str() {
        "teaching" => Mode::Teaching,
        _ => Mode::Open,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_close_button(false)
            .with_mouse_passthrough(false),
        ..Default::default()
    };

    let close_url = server_url.clone();
    let close_device = device_id.clone();
    let app = CheckinApp::new(mode, server_url, device_id, lock_password);
    let _ = eframe::run_native(
        "CampusCheckin",
        options,
        Box::new(|cc| {
            setup_cjk_fonts(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    );

    close_session(&close_url, &close_device, None);
    exit_with(EXIT_RETRY);
}
