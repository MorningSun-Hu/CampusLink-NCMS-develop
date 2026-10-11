//! Window and keyboard policy for the check-in overlay.
//! Teaching must stay locked to the overlay; open mode must keep IME usable.

pub const CHECKIN_WINDOW_TITLE: &str = "CampusCheckin";

pub struct CheckinWindowPolicy {
    pub fullscreen: bool,
    pub maximized: bool,
    pub always_on_top: bool,
    pub allow_ime: bool,
}

pub fn checkin_window_policy(teaching: bool) -> CheckinWindowPolicy {
    if teaching {
        CheckinWindowPolicy {
            fullscreen: true,
            maximized: false,
            always_on_top: true,
            allow_ime: false,
        }
    } else {
        CheckinWindowPolicy {
            fullscreen: false,
            maximized: true,
            always_on_top: false,
            allow_ime: true,
        }
    }
}

pub fn is_ime_class(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    upper.contains("IME")
        || upper.contains("CICERO")
        || upper.contains("CANDIDATE")
        || upper.contains("TSF")
        || upper.contains("CTFIME")
        || upper.contains("INPUTSITE")
        || upper.contains("SOGOU")
        || upper.contains("SOPY")
        || upper.contains("QQPINYIN")
        || upper.contains("RIME")
}

pub fn should_steal_focus(foreground_class: &str, allow_ime: bool) -> bool {
    !(allow_ime && is_ime_class(foreground_class))
}

/// Virtual-key constants used by tests and the Windows hook.
pub const VK_TAB: u32 = 0x09;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_SHIFT: u32 = 0x10;
pub const VK_MENU: u32 = 0x12;
pub const VK_SPACE: u32 = 0x20;
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_LWIN: u32 = 0x5B;
pub const VK_RWIN: u32 = 0x5C;
pub const VK_APPS: u32 = 0x5D;
pub const VK_F1: u32 = 0x70;
pub const VK_F4: u32 = 0x73;
pub const VK_F11: u32 = 0x7A;
pub const VK_LSHIFT: u32 = 0xA0;
pub const VK_RSHIFT: u32 = 0xA1;
pub const VK_LMENU: u32 = 0xA4;
pub const VK_RMENU: u32 = 0xA5;
pub const VK_DELETE: u32 = 0x2E;
pub const VK_PROCESSKEY: u32 = 0xE5;
pub const VK_PACKET: u32 = 0xE7;

pub fn should_block_key(vk: u32, allow_ime: bool, alt: bool, ctrl: bool, shift: bool, win: bool) -> bool {
    if vk == VK_SHIFT || vk == VK_LSHIFT || vk == VK_RSHIFT {
        return false;
    }
    if vk == VK_PROCESSKEY || vk == VK_PACKET {
        return false;
    }
    if vk == VK_LWIN || vk == VK_RWIN || vk == VK_APPS {
        return true;
    }
    if vk == VK_MENU || vk == VK_LMENU || vk == VK_RMENU {
        return !allow_ime;
    }
    if vk == VK_F1 || vk == VK_F11 {
        return true;
    }
    if win {
        return true;
    }
    if alt && (vk == VK_TAB || vk == VK_ESCAPE || vk == VK_F4 || vk == VK_SPACE || vk == VK_RETURN) {
        return true;
    }
    if ctrl && vk == VK_ESCAPE {
        return true;
    }
    if ctrl && shift && vk == VK_ESCAPE {
        return true;
    }
    if ctrl && alt && vk == VK_DELETE {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn teaching_overlay_locks_desktop() {
        let policy = checkin_window_policy(true);
        assert!(policy.fullscreen);
        assert!(policy.always_on_top);
        assert!(!policy.allow_ime);
    }

    #[test]
    fn open_overlay_keeps_ime_usable() {
        let policy = checkin_window_policy(false);
        assert!(policy.maximized);
        assert!(!policy.always_on_top);
        assert!(policy.allow_ime);
        assert!(!policy.fullscreen);
    }

    #[test]
    fn letters_and_shift_pass_in_open_mode() {
        assert!(!should_block_key(0x41, true, false, false, false, false));
        assert!(!should_block_key(VK_SHIFT, true, false, false, true, false));
        assert!(!should_block_key(VK_SPACE, true, false, false, false, false));
        assert!(!should_block_key(VK_PROCESSKEY, true, false, false, false, false));
        assert!(!should_block_key(VK_SPACE, true, false, true, false, false));
    }

    #[test]
    fn open_mode_passes_bare_alt_but_blocks_alt_tab() {
        assert!(!should_block_key(VK_MENU, true, true, false, false, false));
        assert!(should_block_key(VK_TAB, true, true, false, false, false));
        assert!(should_block_key(VK_F4, true, true, false, false, false));
        assert!(should_block_key(VK_LWIN, true, false, false, false, false));
    }

    #[test]
    fn teaching_mode_blocks_bare_alt() {
        assert!(should_block_key(VK_MENU, false, true, false, false, false));
        assert!(should_block_key(VK_TAB, false, true, false, false, false));
    }

    #[test]
    fn focus_keeper_leaves_ime_windows_alone() {
        assert!(!should_steal_focus("IME", true));
        assert!(!should_steal_focus("Windows.UI.Input.InputSite.WindowClass", true));
        assert!(should_steal_focus("CabinetWClass", true));
        assert!(should_steal_focus("IME", false));
    }
}
