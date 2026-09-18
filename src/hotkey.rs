use std::thread;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

pub fn parse_hotkey(hotkey_str: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();
    let mut modifiers = MOD_NOREPEAT as u32;
    let mut vk = 0u32;

    for part in parts {
        match part.to_lowercase().as_str() {
            // Hyper = Ctrl + Shift + Win + Alt (all four modifiers at once)
            "hyper" => modifiers |= MOD_CONTROL as u32 | MOD_SHIFT as u32 | MOD_WIN as u32 | MOD_ALT as u32,
            "ctrl" | "control" => modifiers |= MOD_CONTROL as u32,
            "alt" => modifiers |= MOD_ALT as u32,
            "shift" => modifiers |= MOD_SHIFT as u32,
            "win" | "super" | "windows" => modifiers |= MOD_WIN as u32,
            "space" => vk = 0x20,
            "tab" => vk = 0x09,
            "return" | "enter" => vk = 0x0D,
            "escape" | "esc" => vk = 0x1B,
            "backspace" | "back" => vk = 0x08,
            "delete" | "del" => vk = 0x2E,
            "insert" | "ins" => vk = 0x2D,
            "home" => vk = 0x24,
            "end" => vk = 0x23,
            "pageup" | "page_up" | "pgup" | "prior" => vk = 0x21,
            "pagedown" | "page_down" | "pgdn" | "next" => vk = 0x22,
            "up" | "arrowup" | "arrow_up" => vk = 0x26,
            "down" | "arrowdown" | "arrow_down" => vk = 0x28,
            "left" | "arrowleft" | "arrow_left" => vk = 0x25,
            "right" | "arrowright" | "arrow_right" => vk = 0x27,
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
        crate::log_debug(&format!("parse_hotkey('{}') -> modifiers={:#x}, vk={:#x}", hotkey_str, modifiers, vk));
        Some((modifiers, vk))
    } else {
        crate::log_debug(&format!("parse_hotkey('{}') FAILED: no recognized key", hotkey_str));
        None
    }
}

pub struct GlobalHotkeyListener {
    thread_id: u32,
    handle: Option<thread::JoinHandle<()>>,
}

impl GlobalHotkeyListener {
    pub fn spawn<F>(hotkey_str: String, on_trigger: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = thread::spawn(move || {
            let tid = unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() };
            let _ = tx.send(tid);

            let (modifiers, vk) = match parse_hotkey(&hotkey_str) {
                Some(pair) => pair,
                None => {
                    crate::log_debug(&format!("Hotkey '{}' failed to parse, falling back to Ctrl+Alt+G", hotkey_str));
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
                crate::log_debug("Hotkey listener thread exiting cleanly");
            }
        });

        let thread_id = rx.recv().unwrap_or(0);
        Self {
            thread_id,
            handle: Some(handle),
        }
    }
}

impl Drop for GlobalHotkeyListener {
    fn drop(&mut self) {
        if self.thread_id != 0 {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};
                PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0);
            }
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
