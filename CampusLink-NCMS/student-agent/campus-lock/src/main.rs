use eframe::egui;
use std::io::Write;
use std::sync::{Arc, Mutex};

mod locker;
mod keyhook;

fn main() {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::UI::Input::Ime::ImmDisableIME;
        use windows::Win32::System::Threading::GetCurrentThreadId;
        let thread_id = GetCurrentThreadId();
        ImmDisableIME(thread_id);
    }

    #[cfg(target_os = "windows")]
    keyhook::start_keyboard_hook();

    if let Err(e) = run() {
        let mut f = std::fs::File::create("campus-lock-error.log").unwrap_or_else(|_| {
            let _ = writeln!(std::io::stderr(), "FATAL: {}", e);
            std::process::exit(1);
        });
        let _ = writeln!(f, "campus-lock startup error: {}", e);
        let _ = writeln!(std::io::stderr(), "campus-lock error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let super_password = if args.len() >= 2 {
        args[1].clone()
    } else {
        let msg = "用法: campus-lock <超级密码>";
        let _ = writeln!(std::io::stderr(), "{}", msg);
        return Err(msg.into());
    };

    let _ = writeln!(std::io::stderr(), "campus-lock starting with password={}", "*".repeat(super_password.len()));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_mouse_passthrough(false),
        ..Default::default()
    };

    let _ = writeln!(std::io::stderr(), "campus-lock: calling eframe::run_native");

    eframe::run_native(
        "CampusLock",
        options,
        Box::new(|cc| {
            let _ = writeln!(std::io::stderr(), "campus-lock: creating LockApp");
            setup_cjk_fonts(&cc.egui_ctx);
            Ok(Box::new(LockApp::new(super_password.clone())))
        }),
    )?;

    Ok(())
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

struct LockApp {
    super_password: String,
    password_input: String,
    attempts: u32,
    max_attempts: u32,
    message: String,
    success: bool,
    unlocked: Arc<Mutex<bool>>,
}

impl LockApp {
    fn new(super_password: String) -> Self {
        Self {
            super_password,
            password_input: String::new(),
            attempts: 0,
            max_attempts: 5,
            message: String::new(),
            success: false,
            unlocked: Arc::new(Mutex::new(false)),
        }
    }

    fn remaining(&self) -> u32 {
        self.max_attempts.saturating_sub(self.attempts)
    }

    fn try_unlock(&mut self) {
        if self.success {
            return;
        }
        if self.password_input == self.super_password {
            self.success = true;
            self.message = "解锁成功，正在进入桌面".to_string();
            *self.unlocked.lock().unwrap() = true;
        } else {
            self.attempts += 1;
            self.password_input.clear();
            let left = self.remaining();
            if left == 0 {
                self.message = "密码错误次数过多，请联系教师".to_string();
            } else {
                self.message = format!("密码错误，还可尝试 {} 次", left);
            }
        }
    }
}

impl eframe::App for LockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if *self.unlocked.lock().unwrap() && self.success {
            std::process::exit(0);
        }

        let bg = egui::Color32::from_rgb(12, 16, 24);
        let title = egui::Color32::from_rgb(220, 70, 70);
        let muted = egui::Color32::from_rgb(170, 176, 188);
        let ok = egui::Color32::from_rgb(80, 210, 120);
        let danger = egui::Color32::from_rgb(230, 80, 80);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(bg))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.18);

                    // 标题区
                    ui.heading(
                        egui::RichText::new("屏幕已锁定")
                            .size(44.0)
                            .color(title),
                    );

                    ui.add_space(18.0);

                    // 说明区
                    ui.label(
                        egui::RichText::new("本机已被校园管理系统锁定")
                            .size(18.0)
                            .color(muted),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("请输入教师超级密码后解锁")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(140, 148, 162)),
                    );

                    ui.add_space(28.0);

                    // 密码输入区
                    ui.add_sized(
                        [320.0, 40.0],
                        egui::TextEdit::singleline(&mut self.password_input)
                            .password(true)
                            .hint_text("请输入超级密码")
                            .font(egui::TextStyle::Body),
                    );

                    ui.add_space(14.0);

                    // 操作区
                    if ui
                        .add_sized(
                            [160.0, 40.0],
                            egui::Button::new(egui::RichText::new("解锁").size(16.0)),
                        )
                        .clicked()
                    {
                        self.try_unlock();
                        ctx.request_repaint();
                    }

                    ui.add_space(18.0);

                    // 状态区
                    ui.label(
                        egui::RichText::new(format!("剩余尝试次数：{}/{}", self.remaining(), self.max_attempts))
                            .size(13.0)
                            .color(egui::Color32::from_rgb(120, 128, 144)),
                    );

                    if !self.message.is_empty() {
                        ui.add_space(10.0);
                        let color = if self.success { ok } else { danger };
                        ui.label(
                            egui::RichText::new(&self.message)
                                .size(15.0)
                                .color(color),
                        );
                    }
                });
            });

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !self.password_input.is_empty() {
            self.try_unlock();
            ctx.request_repaint();
        }
    }
}
