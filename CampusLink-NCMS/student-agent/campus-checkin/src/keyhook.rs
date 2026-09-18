#[cfg(target_os = "windows")]
mod keyhook_impl {
    use std::sync::atomic::{AtomicBool, Ordering};
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx,
        GetMessageW, TranslateMessage, DispatchMessageW,
        HHOOK, KBDLLHOOKSTRUCT, MSG, WINDOWS_HOOK_ID,
        WM_KEYDOWN, WM_SYSKEYDOWN,
    };
    use windows::Win32::Foundation::{WPARAM, LPARAM, LRESULT};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        VK_LWIN, VK_RWIN, VK_MENU, VK_TAB, VK_F4, VK_CONTROL, VK_ESCAPE, VK_F1,
        VK_SHIFT, VK_F11, VK_RETURN, VK_LCONTROL, VK_RCONTROL, VK_LMENU, VK_RMENU,
    };

    static HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

    unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            let vk = kb.vkCode;
            let msg = wparam.0 as u32;
            if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                let alt_down = key_down(VK_MENU.0 as i32) || key_down(VK_LMENU.0 as i32) || key_down(VK_RMENU.0 as i32);
                let ctrl_down = key_down(VK_CONTROL.0 as i32) || key_down(VK_LCONTROL.0 as i32) || key_down(VK_RCONTROL.0 as i32);
                let shift_down = key_down(VK_SHIFT.0 as i32);

                // Windows keys
                if vk == VK_LWIN.0 as u32 || vk == VK_RWIN.0 as u32 {
                    return LRESULT(1);
                }
                // Alt+Tab
                if alt_down && vk == VK_TAB.0 as u32 {
                    return LRESULT(1);
                }
                // Alt+F4
                if alt_down && vk == VK_F4.0 as u32 {
                    return LRESULT(1);
                }
                // Ctrl+Esc
                if ctrl_down && vk == VK_ESCAPE.0 as u32 {
                    return LRESULT(1);
                }
                // Ctrl+Shift+Esc (Task Manager)
                if ctrl_down && shift_down && vk == VK_ESCAPE.0 as u32 {
                    return LRESULT(1);
                }
                // F1 / F11 (help / fullscreen toggle)
                if vk == VK_F1.0 as u32 || vk == VK_F11.0 as u32 {
                    return LRESULT(1);
                }
                // Alt+Enter (fullscreen toggle)
                if alt_down && vk == VK_RETURN.0 as u32 {
                    return LRESULT(1);
                }
            }
        }
        CallNextHookEx(None, code, wparam, lparam)
    }

    fn key_down(vk: i32) -> bool {
        extern "system" {
            fn GetAsyncKeyState(vKey: i32) -> i16;
        }
        (unsafe { GetAsyncKeyState(vk) } as u16 & 0x8000) != 0
    }

    pub fn start() {
        if HOOK_RUNNING.swap(true, Ordering::SeqCst) {
            return;
        }
        std::thread::spawn(move || {
            unsafe {
                let hook: HHOOK = SetWindowsHookExW(
                    WINDOWS_HOOK_ID(13),
                    Some(keyboard_proc),
                    None,
                    0,
                )
                .expect("Failed to install keyboard hook");
                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                let _ = UnhookWindowsHookEx(hook);
                HOOK_RUNNING.store(false, Ordering::SeqCst);
            }
        });
    }
}

pub fn start_keyboard_hook() {
    #[cfg(target_os = "windows")]
    keyhook_impl::start();
}
