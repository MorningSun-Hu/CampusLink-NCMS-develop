#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui;
use serde_json::Value;
use std::io::Write;
use std::sync::mpsc;
use std::time::Duration;

enum Mode {
    Open,
    Teaching,
}

#[derive(Debug)]
enum WorkerResult {
    CheckedIn(String),
    NeedPasswordChange(String),
}

struct CheckinApp {
    mode: Mode,
    server_url: String,
    device_id: String,
    name_input: String,
    student_no_input: String,
    password_input: String,
    new_password_input: String,
    need_change: bool,
    pending_student_id: String,
    message: String,
    done: bool,
    tx: mpsc::Sender<Result<WorkerResult, String>>,
    rx: mpsc::Receiver<Result<WorkerResult, String>>,
}

impl CheckinApp {
    fn new(
        mode: Mode,
        server_url: String,
        device_id: String,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            mode,
            server_url,
            device_id,
            name_input: String::new(),
            student_no_input: String::new(),
            password_input: String::new(),
            new_password_input: String::new(),
            need_change: false,
            pending_student_id: String::new(),
            message: String::new(),
            done: false,
            tx,
            rx,
        }
    }

    fn start(&mut self) {
        let mode = match self.mode {
            Mode::Open => "open",
            Mode::Teaching => "teaching",
        };
        let server_url = self.server_url.clone();
        let device_id = self.device_id.clone();
        let name = self.name_input.clone();
        let student_no = self.student_no_input.clone();
        let password = self.password_input.clone();
        let tx = self.tx.clone();

        let _ = writeln!(std::io::stderr(), "campus-checkin: starting {} mode check-in", mode);

        std::thread::spawn(move || {
            let result = match mode {
                "open" => {
                    if name.trim().is_empty() {
                        Err("请输入使用者姓名".to_string())
                    } else {
                        CheckinApp::do_checkin_remote(&server_url, &device_id, name.trim().to_string())
                            .map(WorkerResult::CheckedIn)
                    }
                }
                "teaching" => {
                    if student_no.trim().is_empty() || password.is_empty() {
                        Err("请输入学号和密码".to_string())
                    } else {
                        match CheckinApp::do_student_login_remote(&server_url, &student_no, &password) {
                            Ok((student_id, password_set)) => {
                                if !password_set {
                                    Ok(WorkerResult::NeedPasswordChange(student_id))
                                } else {
                                    CheckinApp::do_checkin_remote(&server_url, &device_id, student_id)
                                        .map(WorkerResult::CheckedIn)
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
        let server_url = self.server_url.clone();
        let device_id = self.device_id.clone();
        let student_id = self.pending_student_id.clone();
        let old_password = self.password_input.clone();
        let new_password = self.new_password_input.clone();
        let tx = self.tx.clone();

        if new_password.len() < 4 {
            self.message = "新密码长度不得少于 4 位".to_string();
            return;
        }

        std::thread::spawn(move || {
            let result = match CheckinApp::do_change_password_remote(
                &server_url,
                &student_id,
                &old_password,
                &new_password,
            ) {
                Ok(_) => CheckinApp::do_checkin_remote(&server_url, &device_id, student_id)
                    .map(WorkerResult::CheckedIn),
                Err(e) => Err(e),
            };
            let _ = tx.send(result);
        });
    }

    fn do_checkin_remote(server_url: &str, device_id: &str, student_id: String) -> Result<String, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let url = format!("{}/api/attendance/check-in", server_url);
        let body = serde_json::json!({
            "device_id": device_id,
            "student_id": student_id,
            "timestamp": chrono::Utc::now().timestamp(),
        });

        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("网络错误: {}", e))?;

        let json: Value = resp.json().map_err(|e| format!("响应解析失败: {}", e))?;
        if json["code"].as_i64() == Some(0) {
            Ok("签到成功".to_string())
        } else {
            let msg = json["message"].as_str().unwrap_or("签到失败").to_string();
            Err(msg)
        }
    }

    fn do_student_login_remote(server_url: &str, student_no: &str, password: &str) -> Result<(String, bool), String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let url = format!("{}/api/auth/student-login", server_url);
        let body = serde_json::json!({
            "student_no": student_no,
            "password": password,
        });

        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("网络错误：{}", e))?;

        let json: Value = resp.json().map_err(|e| format!("响应解析失败：{}", e))?;
        if json["code"].as_i64() == Some(0) {
            let data = &json["data"];
            let student_id = data["student_id"].as_str().unwrap_or("").to_string();
            let password_set = data["password_set"].as_bool().unwrap_or(false);
            if student_id.is_empty() {
                Err("登录响应缺少学生 ID".to_string())
            } else {
                Ok((student_id, password_set))
            }
        } else {
            let msg = json["message"].as_str().unwrap_or("验证失败").to_string();
            Err(msg)
        }
    }

    fn do_change_password_remote(
        server_url: &str,
        student_id: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let url = format!("{}/api/auth/student-set-password", server_url);
        let body = serde_json::json!({
            "student_id": student_id,
            "old_password": old_password,
            "new_password": new_password,
        });

        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("网络错误：{}", e))?;

        let json: Value = resp.json().map_err(|e| format!("响应解析失败：{}", e))?;
        if json["code"].as_i64() == Some(0) {
            Ok(())
        } else {
            let msg = json["message"].as_str().unwrap_or("修改密码失败").to_string();
            Err(msg)
        }
    }
}

impl eframe::App for CheckinApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(result) = self.rx.try_recv() {
            match result {
                Ok(WorkerResult::CheckedIn(msg)) => {
                    self.message = msg;
                    self.done = true;
                    let _ = writeln!(std::io::stderr(), "campus-checkin: SUCCESS {}", self.message);
                    std::process::exit(0);
                }
                Ok(WorkerResult::NeedPasswordChange(student_id)) => {
                    self.pending_student_id = student_id;
                    self.need_change = true;
                    self.done = false;
                    self.message = "首次签到，请设置新密码".to_string();
                    let _ = writeln!(std::io::stderr(), "campus-checkin: need password change");
                }
                Err(e) => {
                    self.message = e;
                    self.done = false;
                    let _ = writeln!(std::io::stderr(), "campus-checkin: FAILED {}", self.message);
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                let title = match self.mode {
                    Mode::Open => "开放模式签到",
                    Mode::Teaching => {
                        if self.need_change {
                            "设置新密码"
                        } else {
                            "授课模式签到"
                        }
                    }
                };
                ui.heading(
                    egui::RichText::new(title)
                        .size(26.0)
                        .color(egui::Color32::from_rgb(50, 120, 220)),
                );

                ui.add_space(20.0);

                match self.mode {
                    Mode::Open => {
                        ui.label(
                            egui::RichText::new("请输入使用者姓名")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(80, 80, 80)),
                        );
                        ui.add_space(8.0);
                        ui.add_sized(
                            [320.0, 36.0],
                            egui::TextEdit::singleline(&mut self.name_input)
                                .hint_text("姓名")
                                .font(egui::TextStyle::Body),
                        );
                    }
                    Mode::Teaching => {
                        if self.need_change {
                            ui.label(
                                egui::RichText::new("首次签到需设置新密码")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(80, 80, 80)),
                            );
                            ui.add_space(8.0);
                            ui.add_sized(
                                [320.0, 36.0],
                                egui::TextEdit::singleline(&mut self.new_password_input)
                                    .password(true)
                                    .hint_text("新密码")
                                    .font(egui::TextStyle::Body),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new("请输入学号/姓名和密码进行身份验证")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(80, 80, 80)),
                            );
                            ui.add_space(8.0);
                            ui.add_sized(
                                [320.0, 36.0],
                                egui::TextEdit::singleline(&mut self.student_no_input)
                                    .hint_text("学号或姓名")
                                    .font(egui::TextStyle::Body),
                            );
                            ui.add_space(8.0);
                            ui.add_sized(
                                [320.0, 36.0],
                                egui::TextEdit::singleline(&mut self.password_input)
                                    .password(true)
                                    .hint_text("密码")
                                    .font(egui::TextStyle::Body),
                            );
                        }
                    }
                }

                ui.add_space(20.0);

                let button_text = if self.need_change { "确认修改并签到" } else { "确认签到" };
                if ui
                    .add_sized([160.0, 40.0], egui::Button::new(
                        egui::RichText::new(button_text).size(16.0),
                    ))
                    .clicked()
                {
                    if self.need_change {
                        self.start_change_password();
                    } else {
                        self.start();
                    }
                    ctx.request_repaint();
                }

                if !self.message.is_empty() {
                    ui.add_space(15.0);
                    let color = if self.done {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else {
                        egui::Color32::from_rgb(220, 50, 50)
                    };
                    ui.label(
                        egui::RichText::new(&self.message)
                            .size(14.0)
                            .color(color),
                    );
                }

                ui.add_space(20.0);
            });
        });

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !self.done {
            if self.need_change {
                self.start_change_password();
                ctx.request_repaint();
            } else {
                let should_submit = match self.mode {
                    Mode::Open => !self.name_input.trim().is_empty(),
                    Mode::Teaching => {
                        !self.student_no_input.trim().is_empty() && !self.password_input.is_empty()
                    }
                };
                if should_submit {
                    self.start();
                    ctx.request_repaint();
                }
            }
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
    ];

    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data
                .insert("cjk".to_owned(), egui::FontData::from_owned(bytes).into());
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "cjk".to_owned());
            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, "cjk".to_owned());
            ctx.set_fonts(fonts);
            return;
        }
    }
}

fn main() {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::UI::Input::Ime::ImmDisableIME;
        use windows::Win32::System::Threading::GetCurrentThreadId;
        let thread_id = GetCurrentThreadId();
        let _ = ImmDisableIME(thread_id);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        let msg = "Usage: campus-checkin <server_url> <device_id> <mode>";
        let _ = writeln!(std::io::stderr(), "{}", msg);
        std::process::exit(2);
    }

    let server_url = args[1].clone();
    let device_id = args[2].clone();
    let mode_str = args[3].clone();

    let mode = match mode_str.as_str() {
        "teaching" => Mode::Teaching,
        _ => Mode::Open,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 360.0])
            .with_decorations(true)
            .with_always_on_top(),
        ..Default::default()
    };

    let app = CheckinApp::new(mode, server_url, device_id);

    let _ = eframe::run_native(
        "CampusCheckin",
        options,
        Box::new(|cc| {
            setup_cjk_fonts(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    );

    // If the window is closed without a successful check-in, report failure.
    std::process::exit(1);
}
