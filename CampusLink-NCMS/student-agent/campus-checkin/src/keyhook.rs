#[cfg(target_os = "windows")]
mod keyhook_impl {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, VK_APPS,
        VK_CONTROL, VK_DELETE, VK_ESCAPE, VK_F1, VK_F4, VK_F11, VK_LCONTROL, VK_LMENU, VK_LWIN,
        VK_MENU, VK_RCONTROL, VK_RETURN, VK_RMENU, VK_RWIN, VK_SHIFT, VK_SPACE, VK_TAB,
        RegisterHotKey,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW,
        SystemParametersInfoW, TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG,
        SPI_SETSCREENSAVERRUNNING, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WH_KEYBOARD_LL,
        WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    const LLKHF_ALTDOWN: u32 = 0x20;
    static HOOK_RUNNING: AtomicBool = AtomicBool::new(false);
    static GUARDS_ON: AtomicBool = AtomicBool::new(false);

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
        if vk == VK_LWIN.0 as u32 || vk == VK_RWIN.0 as u32 || vk == VK_APPS.0 as u32 {
            return true;
        }
        if vk == VK_MENU.0 as u32 || vk == VK_LMENU.0 as u32 || vk == VK_RMENU.0 as u32 {
            return true;
        }
        if vk == VK_F1.0 as u32 || vk == VK_F11.0 as u32 {
            return true;
        }

        let alt = alt_down(flags);
        let ctrl = ctrl_down();
        let shift = shift_down();
        let win = win_down();

        if win {
            return true;
        }
        if alt && (vk == VK_TAB.0 as u32
            || vk == VK_ESCAPE.0 as u32
            || vk == VK_F4.0 as u32
            || vk == VK_SPACE.0 as u32
            || vk == VK_RETURN.0 as u32)
        {
            return true;
        }
        if ctrl && vk == VK_ESCAPE.0 as u32 {
            return true;
        }
        if ctrl && shift && vk == VK_ESCAPE.0 as u32 {
            return true;
        }
        if ctrl && alt && vk == VK_DELETE.0 as u32 {
            return true;
        }
        false
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

    fn set_screensaver_running(on: bool) {
        unsafe {
            let _ = SystemParametersInfoW(
                SPI_SETSCREENSAVERRUNNING,
                u32::from(on),
                None,
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
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

    pub fn start() {
        if HOOK_RUNNING.swap(true, Ordering::SeqCst) {
            return;
        }
        GUARDS_ON.store(true, Ordering::SeqCst);
        set_screensaver_running(true);
        start_taskmgr_guard();
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
        set_screensaver_running(false);
    }
}

pub fn start_keyboard_hook() {
    #[cfg(target_os = "windows")]
    keyhook_impl::start();
}

pub fn stop_keyboard_hook() {
    #[cfg(target_os = "windows")]
    keyhook_impl::stop();
}
