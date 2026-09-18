use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use windows_sys::Win32::Foundation::LPARAM;
use windows_sys::Win32::System::DataExchange::COPYDATASTRUCT;
use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, SendMessageW, WM_COPYDATA};

const MAGIC_HEADER: [u8; 16] = [
    0x99, 0x75, 0x97, 0x27,
    0x4E, 0x64, 0x8B, 0xC5,
    0x3B, 0x7E, 0x42, 0x3D,
    0x02, 0x61, 0x6B, 0x90,
];

const CMD_OPEN_WALLPAPER: usize = 1005;

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn find_wallpaper64_exe() -> Option<PathBuf> {
    let candidates = [
        r"C:\Program Files (x86)\Steam\steamapps\common\wallpaper_engine\wallpaper64.exe",
        r"C:\Program Files\Steam\steamapps\common\wallpaper_engine\wallpaper64.exe",
    ];
    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn open_wallpaper(file_path: &Path, monitor_index: u32) -> bool {
    let normalized = file_path.to_string_lossy().replace(r"\", "/");
    let location_str = format!("Monitor{}", monitor_index);

    let class_name = to_wide("WPEEventWindow");
    let hwnd = unsafe { FindWindowW(class_name.as_ptr(), std::ptr::null()) };

    if !hwnd.is_null() {
        let payload = serde_json::json!({
            "file": normalized,
            "location": location_str,
            "monitor": monitor_index
        });

        if let Ok(json_bytes) = serde_json::to_vec(&payload) {
            let mut full_buffer = Vec::with_capacity(MAGIC_HEADER.len() + json_bytes.len());
            full_buffer.extend_from_slice(&MAGIC_HEADER);
            full_buffer.extend_from_slice(&json_bytes);

            let cds = COPYDATASTRUCT {
                dwData: CMD_OPEN_WALLPAPER,
                cbData: full_buffer.len() as u32,
                lpData: full_buffer.as_mut_ptr() as *mut _,
            };

            unsafe {
                SendMessageW(hwnd, WM_COPYDATA, 0, &cds as *const _ as LPARAM);
            }
            return true;
        }
    }

    // CLI Fallback
    if let Some(exe) = find_wallpaper64_exe() {
        let _ = Command::new(exe)
            .args([
                "-control", "openWallpaper",
                "-file", &normalized,
                "-location", &location_str,
                "-monitor", &monitor_index.to_string(),
            ])
            .spawn();
        return true;
    }

    false
}

pub fn open_wallpaper_on_all(file_path: &Path, monitor_count: u32) {
    for m in 0..monitor_count {
        open_wallpaper(file_path, m);
    }
}
