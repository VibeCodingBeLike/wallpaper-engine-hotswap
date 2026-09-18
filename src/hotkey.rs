use std::thread;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

pub fn parse_hotkey(hotkey_str: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();
    let mut modifiers = MOD_NOREPEAT as u32;
    let mut vk = 0u32;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= MOD_CONTROL as u32,
            "alt" => modifiers |= MOD_ALT as u32,
            "shift" => modifiers |= MOD_SHIFT as u32,
            "win" | "super" | "windows" => modifiers |= MOD_WIN as u32,
            "space" => vk = 0x20,
            "tab" => vk = 0x09,
            "return" | "enter" => vk = 0x0D,
            "escape" | "esc" => vk = 0x1B,
            s if s.len() == 1 => {
                let c = s.chars().next().unwrap();
                if c.is_ascii_alphanumeric() {
                    vk = c.to_ascii_uppercase() as u32;
                }
            }
            s if s.starts_with('f') => {
                if let Ok(num) = s[1..].parse::<u32>() {
                    if (1..=24).contains(&num) {
                        vk = 0x70 + (num - 1);
                    }
                }
            }
            _ => {}
        }
    }

    if vk != 0 {
        Some((modifiers, vk))
    } else {
        None
    }
}

pub struct GlobalHotkeyListener {
    _handle: thread::JoinHandle<()>,
}

impl GlobalHotkeyListener {
    pub fn spawn<F>(hotkey_str: String, on_trigger: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        let handle = thread::spawn(move || {
            let (modifiers, vk) = match parse_hotkey(&hotkey_str) {
                Some(pair) => pair,
                None => {
                    // Fallback to Ctrl+Alt+G
                    (MOD_CONTROL as u32 | MOD_ALT as u32 | MOD_NOREPEAT as u32, b'G' as u32)
                }
            };

            const HOTKEY_ID: i32 = 1001;
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{PeekMessageW, PM_NOREMOVE};
                use windows_sys::Win32::Foundation::GetLastError;

                // Force message queue initialization for this thread
                let mut msg: MSG = std::mem::zeroed();
                PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_NOREMOVE);

                let mut ok = RegisterHotKey(std::ptr::null_mut(), HOTKEY_ID, modifiers, vk);
                if ok == 0 {
                    let err = GetLastError();
                    crate::log_debug(&format!("RegisterHotKey with MOD_NOREPEAT failed (err {}), retrying without it...", err));
                    let fallback_mod = modifiers & !(MOD_NOREPEAT as u32);
                    ok = RegisterHotKey(std::ptr::null_mut(), HOTKEY_ID, fallback_mod, vk);
                    if ok == 0 {
                        crate::log_debug(&format!("RegisterHotKey completely failed! LastError: {}", GetLastError()));
                    } else {
                        crate::log_debug("RegisterHotKey succeeded (without MOD_NOREPEAT)");
                    }
                } else {
                    crate::log_debug("RegisterHotKey succeeded with MOD_NOREPEAT");
                }

                while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                    if msg.message == WM_HOTKEY && msg.wParam == HOTKEY_ID as usize {
                        crate::log_debug("WM_HOTKEY received in hotkey thread -> firing on_trigger");
                        on_trigger();
                    }
                }
                UnregisterHotKey(std::ptr::null_mut(), HOTKEY_ID);
            }
        });

        Self { _handle: handle }
    }
}
