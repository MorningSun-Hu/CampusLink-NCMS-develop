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
    };

    static HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

    unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            let vk = kb.vkCode;

            let msg = wparam.0 as u32;
            if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                let alt_down = user32_get_async_key_state(VK_MENU.0 as i32);
                let ctrl_down = user32_get_async_key_state(VK_CONTROL.0 as i32);

                if vk == VK_LWIN.0 as u32 || vk == VK_RWIN.0 as u32 {
                    return LRESULT(1);
                }
                if alt_down && vk == VK_TAB.0 as u32 {
                    return LRESULT(1);
                }
                if alt_down && vk == VK_F4.0 as u32 {
                    return LRESULT(1);
                }
                if ctrl_down && vk == VK_ESCAPE.0 as u32 {
                    return LRESULT(1);
                }
                if vk == VK_F1.0 as u32 {
                    return LRESULT(1);
                }
            }
        }
        CallNextHookEx(None, code, wparam, lparam)
    }

    fn user32_get_async_key_state(vk: i32) -> bool {
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
                    WINDOWS_HOOK_ID(13), // WH_KEYBOARD_LL = 13
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
