#[cfg(target_os = "windows")]
mod keyhook_impl {
    use crate::overlay::{self, CHECKIN_WINDOW_TITLE};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use windows::core::HSTRING;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, VK_CONTROL,
        VK_DELETE, VK_ESCAPE, VK_F4, VK_LCONTROL, VK_LMENU, VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU,
        VK_RWIN,
        VK_SHIFT, VK_SPACE, VK_TAB, RegisterHotKey,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        BringWindowToTop, CallNextHookEx, DispatchMessageW, EnumWindows, FindWindowW,
        GetClassNameW, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId,
        SetForegroundWindow, SetWindowPos, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
        HWND_TOP, HWND_TOPMOST, KBDLLHOOKSTRUCT, MSG, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    const LLKHF_ALTDOWN: u32 = 0x20;
    static HOOK_RUNNING: AtomicBool = AtomicBool::new(false);
    static GUARDS_ON: AtomicBool = AtomicBool::new(false);
    static ALLOW_IME: AtomicBool = AtomicBool::new(false);

    fn key_down(vk: i32) -> bool {
        extern "system" {
            fn GetAsyncKeyState(vKey: i32) -> i16;
        }
        (unsafe { GetAsyncKeyState(vk) } as u16 & 0x8000) != 0
    }

    fn alt_down(flags: u32) -> bool {
        (flags & LLKHF_ALTDOWN) != 0
            || key_down(VK_MENU.0 as i32)
            || key_down(VK_LMENU.0 as i32)
            || key_down(VK_RMENU.0 as i32)
    }

    fn ctrl_down() -> bool {
        key_down(VK_CONTROL.0 as i32)
            || key_down(VK_LCONTROL.0 as i32)
            || key_down(VK_RCONTROL.0 as i32)
    }

    fn shift_down() -> bool {
        key_down(VK_SHIFT.0 as i32)
    }

    fn win_down() -> bool {
        key_down(VK_LWIN.0 as i32) || key_down(VK_RWIN.0 as i32)
    }

    fn should_block(vk: u32, flags: u32) -> bool {
        overlay::should_block_key(
            vk,
            ALLOW_IME.load(Ordering::SeqCst),
            alt_down(flags),
            ctrl_down(),
            shift_down(),
            win_down(),
        )
    }

    unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            let msg = wparam.0 as u32;
            if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN || msg == WM_KEYUP || msg == WM_SYSKEYUP {
                if should_block(kb.vkCode, kb.flags.0) {
                    return LRESULT(1);
                }
            }
        }
        CallNextHookEx(None, code, wparam, lparam)
    }

    fn register_hotkeys() {
        unsafe {
            let hwnd = HWND::default();
            let _ = RegisterHotKey(
                hwnd,
                1,
                HOT_KEY_MODIFIERS(MOD_ALT.0 | MOD_NOREPEAT.0),
                VK_TAB.0 as u32,
            );
            let _ = RegisterHotKey(
                hwnd,
                2,
                HOT_KEY_MODIFIERS(MOD_ALT.0 | MOD_NOREPEAT.0),
                VK_ESCAPE.0 as u32,
            );
            let _ = RegisterHotKey(
                hwnd,
                3,
                HOT_KEY_MODIFIERS(MOD_ALT.0 | MOD_NOREPEAT.0),
                VK_F4.0 as u32,
            );
            let _ = RegisterHotKey(
                hwnd,
                4,
                HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_NOREPEAT.0),
                VK_ESCAPE.0 as u32,
            );
            let _ = RegisterHotKey(
                hwnd,
                5,
                HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0 | MOD_NOREPEAT.0),
                VK_ESCAPE.0 as u32,
            );
            let _ = RegisterHotKey(
                hwnd,
                6,
                HOT_KEY_MODIFIERS(MOD_WIN.0 | MOD_NOREPEAT.0),
                VK_TAB.0 as u32,
            );
            let _ = RegisterHotKey(
                hwnd,
                7,
                HOT_KEY_MODIFIERS(MOD_ALT.0 | MOD_NOREPEAT.0),
                VK_SPACE.0 as u32,
            );
            let _ = RegisterHotKey(
                hwnd,
                8,
                HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0 | MOD_NOREPEAT.0),
                VK_DELETE.0 as u32,
            );
            let _ = RegisterHotKey(
                hwnd,
                9,
                HOT_KEY_MODIFIERS(MOD_WIN.0 | MOD_NOREPEAT.0),
                b'D' as u32,
            );
        }
    }

    fn silent_taskkill(image: &str) {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("taskkill")
            .args(["/IM", image, "/F"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }

    fn start_taskmgr_guard() {
        std::thread::spawn(|| {
            while GUARDS_ON.load(Ordering::SeqCst) {
                silent_taskkill("taskmgr.exe");
                std::thread::sleep(Duration::from_millis(1000));
            }
        });
    }

    fn window_class_name(hwnd: HWND) -> String {
        let mut buf = [0u16; 256];
        let len = unsafe { GetClassNameW(hwnd, &mut buf) };
        if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }

    fn find_checkin_hwnd() -> HWND {
        let title = HSTRING::from(CHECKIN_WINDOW_TITLE);
        unsafe { FindWindowW(None, &title).unwrap_or_default() }
    }

    unsafe extern "system" fn raise_ime_enum(hwnd: HWND, _lparam: LPARAM) -> BOOL {
        if overlay::is_ime_class(&window_class_name(hwnd)) {
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
        BOOL(1)
    }

    fn start_ime_raiser() {
        std::thread::spawn(|| {
            while GUARDS_ON.load(Ordering::SeqCst) {
                if ALLOW_IME.load(Ordering::SeqCst) {
                    unsafe {
                        let _ = EnumWindows(Some(raise_ime_enum), LPARAM(0));
                    }
                }
                std::thread::sleep(Duration::from_millis(150));
            }
        });
    }

    fn restore_foreground(hwnd: HWND) {
        unsafe {
            let fg = GetForegroundWindow();
            if fg == hwnd || hwnd.is_invalid() {
                return;
            }
            let fg_thread = GetWindowThreadProcessId(fg, None);
            let self_thread = GetCurrentThreadId();
            let _ = AttachThreadInput(self_thread, fg_thread, true);
            let _ = BringWindowToTop(hwnd);
            let _ = SetForegroundWindow(hwnd);
            let insert_after = if ALLOW_IME.load(Ordering::SeqCst) {
                HWND_TOP
            } else {
                HWND_TOPMOST
            };
            let _ = SetWindowPos(
                hwnd,
                insert_after,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            let _ = AttachThreadInput(self_thread, fg_thread, false);
        }
    }

    fn start_focus_keeper() {
        std::thread::spawn(|| {
            while GUARDS_ON.load(Ordering::SeqCst) {
                let hwnd = find_checkin_hwnd();
                if !hwnd.is_invalid() {
                    let fg = unsafe { GetForegroundWindow() };
                    if fg != hwnd {
                        let class = window_class_name(fg);
                        if overlay::should_steal_focus(&class, ALLOW_IME.load(Ordering::SeqCst)) {
                            restore_foreground(hwnd);
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(120));
            }
        });
    }

    pub fn start(allow_ime: bool) {
        if HOOK_RUNNING.swap(true, Ordering::SeqCst) {
            return;
        }
        GUARDS_ON.store(true, Ordering::SeqCst);
        ALLOW_IME.store(allow_ime, Ordering::SeqCst);
        start_taskmgr_guard();
        start_focus_keeper();
        if allow_ime {
            start_ime_raiser();
        }
        std::thread::spawn(|| {
            unsafe {
                let module = GetModuleHandleW(None).unwrap_or_default();
                let hook = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(keyboard_proc),
                    module,
                    0,
                );
                register_hotkeys();
                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                if let Ok(h) = hook {
                    let _ = UnhookWindowsHookEx(h);
                }
            }
            HOOK_RUNNING.store(false, Ordering::SeqCst);
        });
    }

    pub fn stop() {
        GUARDS_ON.store(false, Ordering::SeqCst);
    }
}

pub fn start_keyboard_hook(allow_ime: bool) {
    #[cfg(target_os = "windows")]
    keyhook_impl::start(allow_ime);
    #[cfg(not(target_os = "windows"))]
    let _ = allow_ime;
}

pub fn stop_keyboard_hook() {
    #[cfg(target_os = "windows")]
    keyhook_impl::stop();
}
