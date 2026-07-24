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
        let msg = "Usage: campus-lock <super_password>";
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
        Box::new(|_cc| {
            let _ = writeln!(std::io::stderr(), "campus-lock: creating LockApp");
            Ok(Box::new(LockApp::new(super_password.clone())))
        }),
    )?;

    Ok(())
}

struct LockApp {
    super_password: String,
    password_input: String,
    attempts: u32,
    max_attempts: u32,
    message: String,
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
            unlocked: Arc::new(Mutex::new(false)),
        }
    }
}

impl eframe::App for LockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if *self.unlocked.lock().unwrap() {
            std::process::exit(0);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.25);

                ui.heading(
                    egui::RichText::new("SCREEN LOCKED")
                        .size(48.0)
                        .color(egui::Color32::from_rgb(220, 50, 50)),
                );

                ui.add_space(20.0);

                ui.label(
                    egui::RichText::new("This workstation has been locked by the campus management system.")
                        .size(18.0)
                        .color(egui::Color32::from_rgb(180, 180, 180)),
                );

                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new("Enter the super password to unlock.")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(150, 150, 150)),
                );

                ui.add_space(30.0);

                let _password_response = ui.add_sized(
                    [300.0, 40.0],
                    egui::TextEdit::singleline(&mut self.password_input)
                        .password(true)
                        .hint_text("Password")
                        .font(egui::TextStyle::Body),
                );

                ui.add_space(10.0);

                if ui
                    .add_sized([120.0, 36.0], egui::Button::new(
                        egui::RichText::new("Unlock").size(16.0),
                    ))
                    .clicked()
                {
                    self.try_unlock();
                    ctx.request_repaint();
                }

                if !self.message.is_empty() {
                    ui.add_space(15.0);
                    let color = if self.message.contains("granted") || self.message.contains("unlocked") {
                        egui::Color32::from_rgb(50, 220, 50)
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

                ui.label(
                    egui::RichText::new(format!(
                        "Attempts: {}/{}",
                        self.attempts, self.max_attempts
                    ))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(120, 120, 120)),
                );
            });
        });

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !self.password_input.is_empty() {
            self.try_unlock();
            ctx.request_repaint();
        }
    }
}

impl LockApp {
    fn try_unlock(&mut self) {
        if self.password_input == self.super_password {
            self.message = "Access granted. Screen unlocked.".to_string();
            *self.unlocked.lock().unwrap() = true;
        } else {
            self.attempts += 1;
            self.password_input.clear();

            if self.attempts >= self.max_attempts {
                self.message = format!(
                    "Too many failed attempts. {} remaining.",
                    self.max_attempts.saturating_sub(self.attempts)
                );
            } else {
                self.message = format!(
                    "Incorrect password. {} attempts remaining.",
                    self.max_attempts - self.attempts
                );
            }
        }
    }
}
