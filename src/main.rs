#![windows_subsystem = "windows"]

mod config;
mod engine_ipc;
mod hotkey;
mod render;
mod scanner;

use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use winit::event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopBuilder};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder, WindowLevel};

use config::Config;
use hotkey::GlobalHotkeyListener;
use render::{FontRenderer, HitAction, HitBox, ImageCache};
use scanner::{scan_wallpapers, WallpaperItem};

#[derive(Debug)]
enum CustomEvent {
    ToggleGallery,
}

pub fn log_debug(msg: &str) {
    let dir = config::Config::config_dir();
    let _ = std::fs::create_dir_all(&dir);
    let log_file = dir.join("debug.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(log_file) {
        use std::io::Write;
        let _ = writeln!(f, "[DEBUG] {}", msg);
    }
}

fn parse_edid_name(edid: &[u8]) -> Option<String> {
    for &offset in &[54, 72, 90, 108] {
        if offset + 18 <= edid.len() {
            let block = &edid[offset..offset + 18];
            // Descriptor tag 0x00, 0x00, 0x00, 0xFC, 0x00 indicates Monitor Name String
            if block[0] == 0 && block[1] == 0 && block[2] == 0 && block[3] == 0xFC {
                let name_bytes = &block[5..18];
                let mut name = String::new();
                for &b in name_bytes {
                    if b == b'\n' || b == b'\r' || b == 0 {
                        break;
                    }
                    if b.is_ascii_graphic() || b == b' ' {
                        name.push(b as char);
                    }
                }
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

fn get_friendly_monitor_name(adapter_gdi: &str) -> Option<String> {
    use windows_sys::Win32::Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW};
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
    };

    let mut adapter_w: Vec<u16> = adapter_gdi.encode_utf16().collect();
    adapter_w.push(0);

    let mut mon_dev: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
    mon_dev.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;

    for i in 0..8 {
        let ret = unsafe { EnumDisplayDevicesW(adapter_w.as_ptr(), i, &mut mon_dev, 0) };
        if ret == 0 {
            break;
        }

        let dev_id: String = mon_dev.DeviceID
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8 as char)
            .collect();

        // dev_id is e.g. "MONITOR\GSM5C7C\{4d36e96e-e325-11ce-bfc1-08002be10318}\0003"
        let parts: Vec<&str> = dev_id.split('\\').collect();
        if parts.len() >= 2 {
            let model_id = parts[1];
            let subkey_str = format!("SYSTEM\\CurrentControlSet\\Enum\\DISPLAY\\{}\0", model_id);
            let subkey_w: Vec<u16> = subkey_str.encode_utf16().collect();

            unsafe {
                let mut h_key: HKEY = std::mem::zeroed();
                if RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey_w.as_ptr(), 0, KEY_READ, &mut h_key) == 0 {
                    let mut index = 0u32;
                    let mut child_name = [0u16; 256];
                    let mut name_len = 256u32;

                    while RegEnumKeyExW(
                        h_key,
                        index,
                        child_name.as_mut_ptr(),
                        &mut name_len,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    ) == 0 {
                        let child_str: String = child_name[..name_len as usize].iter().map(|&c| c as u8 as char).collect();
                        let dev_param_str = format!("SYSTEM\\CurrentControlSet\\Enum\\DISPLAY\\{}\\{}\\Device Parameters\0", model_id, child_str);
                        let dev_param_w: Vec<u16> = dev_param_str.encode_utf16().collect();

                        let mut h_dev_key: HKEY = std::mem::zeroed();
                        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, dev_param_w.as_ptr(), 0, KEY_READ, &mut h_dev_key) == 0 {
                            let val_name: Vec<u16> = "EDID\0".encode_utf16().collect();
                            let mut edid_buf = [0u8; 1024];
                            let mut buf_len = 1024u32;
                            let mut val_type = 0u32;

                            if RegQueryValueExW(
                                h_dev_key,
                                val_name.as_ptr(),
                                std::ptr::null_mut(),
                                &mut val_type,
                                edid_buf.as_mut_ptr(),
                                &mut buf_len,
                            ) == 0 {
                                RegCloseKey(h_dev_key);
                                RegCloseKey(h_key);
                                if let Some(friendly) = parse_edid_name(&edid_buf[..buf_len as usize]) {
                                    return Some(friendly);
                                }
                            }
                            RegCloseKey(h_dev_key);
                        }

                        index += 1;
                        name_len = 256;
                    }
                    RegCloseKey(h_key);
                }
            }
        }
    }
    None
}

fn get_cursor_pos() -> (i32, i32) {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    unsafe {
        let mut pt: POINT = std::mem::zeroed();
        GetCursorPos(&mut pt);
        (pt.x, pt.y)
    }
}

/// Applies Win32 ToolWindow and Popup styles so Komorebi and tiling window managers ignore this overlay.
fn apply_overlay_styles(window: &Window) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, GWL_STYLE,
        HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        WS_EX_APPWINDOW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    };
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    if let Ok(handle) = window.window_handle() {
        if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
            let hwnd = win32_handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;
            unsafe {
                // Remove AppWindow, add ToolWindow and TopMost so Komorebi never manages or tiles it
                let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
                let new_ex_style = (ex_style | WS_EX_TOOLWINDOW | WS_EX_TOPMOST) & !WS_EX_APPWINDOW;
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style as isize);

                // Ensure popup style with no borders
                let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
                let new_style = (style | WS_POPUP) & !0x00CF0000;
                SetWindowLongPtrW(hwnd, GWL_STYLE, new_style as isize);

                // Disable Windows default zoom-out / minimize window animations
                let disable_transitions: i32 = 1;
                windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute(
                    hwnd,
                    windows_sys::Win32::Graphics::Dwm::DWMWA_TRANSITIONS_FORCEDISABLED as u32,
                    &disable_transitions as *const _ as *const _,
                    std::mem::size_of::<i32>() as u32,
                );

                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                );
            }
        }
    }
}

fn force_window_foreground(window: &Window) {
    use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::SetFocus;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
        SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    };
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    if let Ok(handle) = window.window_handle() {
        if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
            let hwnd = win32_handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;
            unsafe {
                let fg = GetForegroundWindow();
                if !fg.is_null() {
                    let fg_tid = GetWindowThreadProcessId(fg, std::ptr::null_mut());
                    let my_tid = GetCurrentThreadId();
                    if fg_tid != 0 && my_tid != 0 && fg_tid != my_tid {
                        AttachThreadInput(my_tid, fg_tid, 1);
                        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
                        BringWindowToTop(hwnd);
                        SetForegroundWindow(hwnd);
                        SetFocus(hwnd);
                        AttachThreadInput(my_tid, fg_tid, 0);
                        return;
                    }
                }
                SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
                BringWindowToTop(hwnd);
                SetForegroundWindow(hwnd);
                SetFocus(hwnd);
            }
        }
    }
}

fn trim_memory() {
    use windows_sys::Win32::System::ProcessStatus::EmptyWorkingSet;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    unsafe {
        EmptyWorkingSet(GetCurrentProcess());
    }
}

fn matches_key(key: &KeyEvent, action_key: &str) -> bool {
    let target = action_key.trim().to_lowercase();
    match key.physical_key {
        PhysicalKey::Code(code) => match (code, target.as_str()) {
            (KeyCode::Escape, "escape" | "esc") => true,
            (KeyCode::Enter, "return" | "enter") => true,
            (KeyCode::ArrowLeft, "left") => true,
            (KeyCode::ArrowRight, "right") => true,
            (KeyCode::ArrowUp, "up") => true,
            (KeyCode::ArrowDown, "down") => true,
            (KeyCode::PageUp, "pageup" | "page_up" | "pgup") => true,
            (KeyCode::PageDown, "pagedown" | "page_down" | "pgdn") => true,
            (KeyCode::Home, "home") => true,
            (KeyCode::End, "end") => true,
            (KeyCode::Tab, "tab") => true,
            (KeyCode::KeyX, "x") => true,
            (KeyCode::KeyH, "h") => true,
            (KeyCode::KeyE, "e") => true,
            (KeyCode::Delete, "delete" | "del") => true,
            (KeyCode::KeyA, "a") => true,
            (KeyCode::KeyD, "d") => true,
            (KeyCode::KeyW, "w") => true,
            (KeyCode::KeyS, "s") => true,
            (KeyCode::KeyQ, "q") => true,
            (KeyCode::Space, "space") => true,
            _ => false,
        },
        _ => false,
    }
}

fn main() {
    log_debug("we-gallery starting up...");

    // Single-instance check
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW;
        let title: Vec<u16> = "Wallpaper Engine Gallery\0".encode_utf16().collect();
        let existing = FindWindowW(std::ptr::null(), title.as_ptr());
        if !existing.is_null() {
            log_debug("Another instance of we-gallery is already running. Exiting duplicate.");
            return;
        }
    }

    let mut config = Config::load();
    log_debug(&format!("Config loaded. Hotkey configured as: {}", config.keybinds.toggle_gallery));

    let event_loop = EventLoopBuilder::<CustomEvent>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    // Start native global hotkey listener (0.00% background CPU)
    let hotkey_combo = config.keybinds.toggle_gallery.clone();
    let proxy_clone = proxy.clone();
    let _hotkey_listener = GlobalHotkeyListener::spawn(hotkey_combo, move || {
        log_debug("Hotkey triggered callback -> sending CustomEvent::ToggleGallery");
        let _ = proxy_clone.send_event(CustomEvent::ToggleGallery);
    });

    // Create frameless translucent window
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Wallpaper Engine Gallery")
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_visible(false)
            .build(&event_loop)
            .unwrap(),
    );

    apply_overlay_styles(&window);

    let context = softbuffer::Context::new(window.clone()).unwrap();
    let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

    let fonts = FontRenderer::new();
    let mut image_cache = ImageCache::new();

    let mut all_wallpapers = scan_wallpapers();
    let mut displayed_wallpapers: Vec<WallpaperItem>;

    let mut is_visible = false;
    let mut is_closing = false;
    let mut fade_alpha = 1.0f32;
    let mut active_monitor_idx = 0usize;
    let mut selected_index = 0usize;
    let mut scroll_offset = 0.0f32;
    let mut target_scroll = 0.0f32;
    let mut is_animating_scroll = false;
    let mut hovered_card: Option<usize> = None;
    let mut current_hitboxes: Vec<HitBox> = Vec::new();
    let mut cursor_pos = (0.0f32, 0.0f32);
    let mut show_hidden = config.behavior.show_excluded;
    let mut cached_pixmap: Option<tiny_skia::Pixmap> = None;

    let mut last_click_time = Instant::now();
    let mut last_click_card: Option<usize> = None;
    let mut last_frame_time = Instant::now();
    let mut last_config_mtime = Config::last_modified();
    let mut last_config_check = Instant::now();

    let refresh_displayed = |all: &[WallpaperItem], cfg: &Config, mon: usize, show_h: bool| -> Vec<WallpaperItem> {
        let mon_key = format!("Monitor{}", mon);
        if show_h {
            all.to_vec()
        } else {
            all.iter().filter(|w| !cfg.is_excluded(&mon_key, &w.id)).cloned().collect()
        }
    };

    displayed_wallpapers = refresh_displayed(&all_wallpapers, &config, active_monitor_idx, show_hidden);

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Wait);

        match event {
            Event::UserEvent(CustomEvent::ToggleGallery) => {
                log_debug(&format!("ToggleGallery event fired! Current is_visible: {}, is_closing: {}", is_visible, is_closing));
                if is_visible {
                    if !is_closing {
                        log_debug("Starting fade-out gallery window...");
                        is_closing = true;
                        window.request_redraw();
                    }
                } else {
                    log_debug("Showing gallery window...");
                    is_closing = false;
                    fade_alpha = 1.0;
                    last_frame_time = Instant::now();
                    last_config_mtime = Config::last_modified();
                    config = Config::load();
                    image_cache.clear();
                    cached_pixmap = None;
                    show_hidden = config.behavior.show_excluded;
                    all_wallpapers = scan_wallpapers();

                    // Detect monitor from current cursor position
                    let (cx, cy) = get_cursor_pos();
                    let monitors: Vec<_> = window.available_monitors().collect();
                    let mut matched_idx = 0;

                    for (i, m) in monitors.iter().enumerate() {
                        let pos = m.position();
                        let size = m.size();
                        if cx >= pos.x && cx < pos.x + size.width as i32 && cy >= pos.y && cy < pos.y + size.height as i32 {
                            matched_idx = i;
                            break;
                        }
                    }

                    active_monitor_idx = matched_idx;
                    displayed_wallpapers = refresh_displayed(&all_wallpapers, &config, active_monitor_idx, show_hidden);

                    if let Some(m) = monitors.get(active_monitor_idx) {
                        window.set_outer_position(m.position());
                        let _ = window.request_inner_size(m.size());
                    }

                    let mut start_idx = 0usize;
                    if let Some(active_file) = scanner::get_active_wallpaper_file(active_monitor_idx) {
                        log_debug(&format!("Found active wallpaper file for Monitor{}: {}", active_monitor_idx, active_file));
                        if let Some(idx) = scanner::find_active_wallpaper_index(&displayed_wallpapers, &active_file) {
                            log_debug(&format!("Matched active wallpaper index: {}", idx));
                            start_idx = idx;
                        }
                    }

                    selected_index = start_idx;
                    scroll_offset = start_idx as f32;
                    target_scroll = start_idx as f32;

                    is_visible = true;
                    window.set_visible(true);
                    force_window_foreground(&window);
                    window.request_redraw();
                }
            }

            Event::WindowEvent { event, .. } => {
                if !is_visible {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => {
                        if is_visible && !is_closing {
                            is_closing = true;
                            window.request_redraw();
                        }
                    }

                    WindowEvent::Focused(false) => {
                        if config.ui.close_on_focus_loss && is_visible && !is_closing {
                            is_closing = true;
                            window.request_redraw();
                        }
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        if is_closing {
                            return;
                        }
                        cursor_pos = (position.x as f32, position.y as f32);
                        let mut found_hover = None;
                        for hb in &current_hitboxes {
                            if let HitAction::SelectCard(idx) = hb.action {
                                if hb.contains(cursor_pos.0, cursor_pos.1) {
                                    found_hover = Some(idx);
                                    break;
                                }
                            }
                        }
                        if found_hover != hovered_card {
                            hovered_card = found_hover;
                            window.request_redraw();
                        }
                    }

                    WindowEvent::MouseWheel { delta, .. } => {
                        if is_closing {
                            return;
                        }
                        let count = displayed_wallpapers.len();
                        if count > 0 {
                            let step = match delta {
                                MouseScrollDelta::LineDelta(_, y) => {
                                    if y > 0.0 { -1 } else { 1 }
                                }
                                MouseScrollDelta::PixelDelta(pos) => {
                                    if pos.y > 0.0 { -1 } else { 1 }
                                }
                            };
                            let new_idx = if step < 0 {
                                if selected_index == 0 { count - 1 } else { selected_index - 1 }
                            } else {
                                (selected_index + 1) % count
                            };
                            selected_index = new_idx;
                            target_scroll += step as f32;
                            window.request_redraw();
                        }
                    }

                    WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                        if is_closing {
                            return;
                        }
                        if button == MouseButton::Left {
                            let mut hit_action = None;
                            for hb in &current_hitboxes {
                                if hb.contains(cursor_pos.0, cursor_pos.1) {
                                    hit_action = Some(hb.action.clone());
                                    break;
                                }
                            }

                            match hit_action {
                                Some(HitAction::Close) => {
                                    is_closing = true;
                                    window.request_redraw();
                                }
                                Some(HitAction::ToggleHidden) => {
                                    show_hidden = !show_hidden;
                                    config.behavior.show_excluded = show_hidden;
                                    let _ = config.save();
                                    displayed_wallpapers = refresh_displayed(&all_wallpapers, &config, active_monitor_idx, show_hidden);
                                    selected_index = selected_index.min(displayed_wallpapers.len().saturating_sub(1));
                                    target_scroll = selected_index as f32;
                                    window.request_redraw();
                                }
                                Some(HitAction::SwitchMonitor(m_idx)) => {
                                    let monitors: Vec<_> = window.available_monitors().collect();
                                    if m_idx < monitors.len() {
                                        active_monitor_idx = m_idx;
                                        displayed_wallpapers = refresh_displayed(&all_wallpapers, &config, active_monitor_idx, show_hidden);
                                        let m = &monitors[active_monitor_idx];
                                        window.set_outer_position(m.position());
                                        let _ = window.request_inner_size(m.size());

                                        let mut start_idx = 0usize;
                                        if let Some(active_file) = scanner::get_active_wallpaper_file(active_monitor_idx) {
                                            if let Some(idx) = scanner::find_active_wallpaper_index(&displayed_wallpapers, &active_file) {
                                                start_idx = idx;
                                            }
                                        }

                                        selected_index = start_idx;
                                        scroll_offset = start_idx as f32;
                                        target_scroll = start_idx as f32;
                                        window.request_redraw();
                                    }
                                }
                                Some(HitAction::SelectCard(idx)) => {
                                    let now = Instant::now();
                                    if last_click_card == Some(idx) && now.duration_since(last_click_time).as_millis() < 400 {
                                        // Double click -> Apply wallpaper & start smooth fade out
                                        if let Some(wp) = displayed_wallpapers.get(idx) {
                                            engine_ipc::open_wallpaper(&wp.file_target, active_monitor_idx as u32);
                                            is_closing = true;
                                            window.request_redraw();
                                        }
                                    } else {
                                        selected_index = idx;
                                        target_scroll = idx as f32;
                                        last_click_time = now;
                                        last_click_card = Some(idx);
                                        window.request_redraw();
                                    }
                                }
                                None => {
                                    if config.ui.close_on_backdrop_click {
                                        is_closing = true;
                                        window.request_redraw();
                                    }
                                }
                            }
                        }
                    }

                    WindowEvent::KeyboardInput { event: key, .. } => {
                        if is_closing {
                            return;
                        }
                        if key.state == ElementState::Pressed {
                            let kb = &config.keybinds;

                            if matches_key(&key, &kb.close) {
                                is_closing = true;
                                window.request_redraw();
                            } else if matches_key(&key, &kb.apply_wallpaper) {
                                if let Some(wp) = displayed_wallpapers.get(selected_index) {
                                    engine_ipc::open_wallpaper(&wp.file_target, active_monitor_idx as u32);
                                    is_closing = true;
                                    window.request_redraw();
                                }
                            } else if matches_key(&key, &kb.apply_to_all) {
                                if let Some(wp) = displayed_wallpapers.get(selected_index) {
                                    let mon_count = window.available_monitors().count() as u32;
                                    engine_ipc::open_wallpaper_on_all(&wp.file_target, mon_count);
                                    is_closing = true;
                                    window.request_redraw();
                                }
                            } else if matches_key(&key, &kb.left) {
                                let count = displayed_wallpapers.len();
                                if count > 0 {
                                    selected_index = if selected_index == 0 { count - 1 } else { selected_index - 1 };
                                    target_scroll -= 1.0;
                                    window.request_redraw();
                                }
                            } else if matches_key(&key, &kb.right) {
                                let count = displayed_wallpapers.len();
                                if count > 0 {
                                    selected_index = (selected_index + 1) % count;
                                    target_scroll += 1.0;
                                    window.request_redraw();
                                }
                            } else if matches_key(&key, &kb.page_left) {
                                let count = displayed_wallpapers.len();
                                if count > 0 {
                                    selected_index = selected_index.saturating_sub(4);
                                    target_scroll = selected_index as f32;
                                    window.request_redraw();
                                }
                            } else if matches_key(&key, &kb.page_right) {
                                let count = displayed_wallpapers.len();
                                if count > 0 {
                                    selected_index = (selected_index + 4).min(count - 1);
                                    target_scroll = selected_index as f32;
                                    window.request_redraw();
                                }
                            } else if matches_key(&key, &kb.first) {
                                selected_index = 0;
                                target_scroll = 0.0;
                                window.request_redraw();
                            } else if matches_key(&key, &kb.last) {
                                if !displayed_wallpapers.is_empty() {
                                    selected_index = displayed_wallpapers.len() - 1;
                                    target_scroll = selected_index as f32;
                                    window.request_redraw();
                                }
                            } else if matches_key(&key, &kb.switch_monitor) {
                                let count = window.available_monitors().count();
                                if count > 0 {
                                    active_monitor_idx = (active_monitor_idx + 1) % count;
                                    displayed_wallpapers = refresh_displayed(&all_wallpapers, &config, active_monitor_idx, show_hidden);
                                    let monitors: Vec<_> = window.available_monitors().collect();
                                    if let Some(m) = monitors.get(active_monitor_idx) {
                                        window.set_outer_position(m.position());
                                        let _ = window.request_inner_size(m.size());
                                    }

                                    let mut start_idx = 0usize;
                                    if let Some(active_file) = scanner::get_active_wallpaper_file(active_monitor_idx) {
                                        if let Some(idx) = scanner::find_active_wallpaper_index(&displayed_wallpapers, &active_file) {
                                            start_idx = idx;
                                        }
                                    }

                                    selected_index = start_idx;
                                    scroll_offset = start_idx as f32;
                                    target_scroll = start_idx as f32;
                                    window.request_redraw();
                                }
                            } else if matches_key(&key, &kb.toggle_exclude) {
                                if let Some(wp) = displayed_wallpapers.get(selected_index) {
                                    let mon_key = format!("Monitor{}", active_monitor_idx);
                                    config.toggle_excluded(&mon_key, &wp.id);
                                    displayed_wallpapers = refresh_displayed(&all_wallpapers, &config, active_monitor_idx, show_hidden);
                                    selected_index = selected_index.min(displayed_wallpapers.len().saturating_sub(1));
                                    target_scroll = selected_index as f32;
                                    window.request_redraw();
                                }
                            } else if matches_key(&key, &kb.toggle_hidden) {
                                show_hidden = !show_hidden;
                                config.behavior.show_excluded = show_hidden;
                                let _ = config.save();
                                displayed_wallpapers = refresh_displayed(&all_wallpapers, &config, active_monitor_idx, show_hidden);
                                selected_index = selected_index.min(displayed_wallpapers.len().saturating_sub(1));
                                target_scroll = selected_index as f32;
                                window.request_redraw();
                            } else if matches_key(&key, &kb.open_in_explorer) {
                                if let Some(wp) = displayed_wallpapers.get(selected_index) {
                                    let _ = Command::new("explorer").arg(&wp.folder_path).spawn();
                                }
                            } else if matches_key(&key, &kb.reload_config) || matches_key(&key, "f5") || matches_key(&key, "ctrl+r") {
                                log_debug("Manual reload config triggered (F5 / Ctrl+R)");
                                last_config_mtime = Config::last_modified();
                                config = Config::load();
                                image_cache.clear();
                                cached_pixmap = None;
                                all_wallpapers = scan_wallpapers();
                                show_hidden = config.behavior.show_excluded;
                                displayed_wallpapers = refresh_displayed(&all_wallpapers, &config, active_monitor_idx, show_hidden);
                                selected_index = selected_index.min(displayed_wallpapers.len().saturating_sub(1));
                                target_scroll = selected_index as f32;
                                window.request_redraw();
                            }
                        }
                    }

                    WindowEvent::RedrawRequested => {
                        let now = Instant::now();
                        let dt = now.duration_since(last_frame_time).as_secs_f32().clamp(0.001, 0.050);
                        last_frame_time = now;

                        if is_closing {
                            let fade_speed = dt / 0.10; // Smooth 100ms fade-out at any refresh rate (240Hz, 360Hz, etc.)
                            fade_alpha = (fade_alpha - fade_speed).max(0.0);
                            if fade_alpha <= 0.01 {
                                is_visible = false;
                                is_closing = false;
                                fade_alpha = 1.0;
                                window.set_visible(false);
                                cached_pixmap = None;
                                trim_memory();
                                target.set_control_flow(ControlFlow::Wait);
                                return;
                            }
                        }

                        let size = window.inner_size();
                        if size.width == 0 || size.height == 0 {
                            return;
                        }

                        let _ = surface.resize(
                            std::num::NonZeroU32::new(size.width).unwrap(),
                            std::num::NonZeroU32::new(size.height).unwrap(),
                        );

                        // Reuse internal RGBA pixmap to avoid 15MB re-allocation on every frame
                        let pixmap = match cached_pixmap {
                            Some(ref mut p) if p.width() == size.width && p.height() == size.height => {
                                p.fill(tiny_skia::Color::TRANSPARENT);
                                p
                            }
                            _ => {
                                cached_pixmap = tiny_skia::Pixmap::new(size.width, size.height);
                                cached_pixmap.as_mut().unwrap()
                            }
                        };

                        // Smooth delta-time based animation interpolation
                        let diff = target_scroll - scroll_offset;
                        is_animating_scroll = diff.abs() > 0.001;
                        let scroll_velocity = if is_animating_scroll {
                            // Frame-rate independent exponential decay matching 0.28 factor at 60Hz
                            let factor = 1.0 - (1.0 - 0.28f32).powf(dt * 60.0);
                            let step = diff * factor;
                            scroll_offset += step;
                            step
                        } else {
                            scroll_offset = target_scroll;
                            0.0
                        };

                        let monitors = window.available_monitors();
                        let monitor_names: Vec<String> = monitors
                            .enumerate()
                            .map(|(i, m)| {
                                let gdi_name = m.name().unwrap_or_default();
                                get_friendly_monitor_name(&gdi_name)
                                    .unwrap_or_else(|| format!("Display {}", i))
                            })
                            .collect();

                        current_hitboxes = render::render_gallery(
                            pixmap,
                            &fonts,
                            &mut image_cache,
                            &config,
                            &displayed_wallpapers,
                            active_monitor_idx,
                            &monitor_names,
                            selected_index,
                            scroll_offset,
                            scroll_velocity,
                            hovered_card,
                            show_hidden,
                        );

                        // Convert tiny_skia RGBA to Windows softbuffer 0x00RRGGBB (scaling by fade_alpha if closing)
                        let mut buffer = surface.buffer_mut().unwrap();
                        let src_u32: &[u32] = bytemuck::cast_slice(pixmap.data());

                        if fade_alpha < 0.999 {
                            let alpha_mul = (fade_alpha * 256.0).round() as u32;
                            for (&sp, dst) in src_u32.iter().zip(buffer.iter_mut()) {
                                let r = (((sp & 0xFF) * alpha_mul) >> 8) & 0xFF;
                                let g = ((((sp >> 8) & 0xFF) * alpha_mul) >> 8) & 0xFF;
                                let b = ((((sp >> 16) & 0xFF) * alpha_mul) >> 8) & 0xFF;
                                let a = ((((sp >> 24) & 0xFF) * alpha_mul) >> 8) & 0xFF;
                                *dst = (a << 24) | (r << 16) | (g << 8) | b;
                            }
                        } else {
                            // Ultra-fast 32-bit swap of R and B (vectorizes with AVX2 vpshufb)
                            for (&sp, dst) in src_u32.iter().zip(buffer.iter_mut()) {
                                *dst = (sp & 0xFF00FF00) | ((sp & 0x000000FF) << 16) | ((sp & 0x00FF0000) >> 16);
                            }
                        }

                        buffer.present().unwrap();

                        // Synchronize with DWM hardware vertical blank (uncapped V-Sync, natively supporting 240Hz+)
                        unsafe {
                            windows_sys::Win32::Graphics::Dwm::DwmFlush();
                        }

                        if is_closing || is_animating_scroll {
                            window.request_redraw();
                            target.set_control_flow(ControlFlow::Poll);
                        } else if is_visible {
                            target.set_control_flow(ControlFlow::wait_duration(std::time::Duration::from_millis(250)));
                        } else {
                            target.set_control_flow(ControlFlow::Wait);
                        }
                    }

                    _ => {}
                }
            }

            Event::AboutToWait => {
                if is_visible && !is_closing {
                    let now = Instant::now();
                    if now.duration_since(last_config_check).as_millis() >= 250 {
                        last_config_check = now;
                        let current_mtime = Config::last_modified();
                        if current_mtime != last_config_mtime {
                            last_config_mtime = current_mtime;
                            log_debug("Auto hot-reload: config.toml change detected on disk!");
                            config = Config::load();
                            image_cache.clear();
                            cached_pixmap = None;
                            all_wallpapers = scan_wallpapers();
                            show_hidden = config.behavior.show_excluded;
                            displayed_wallpapers = refresh_displayed(&all_wallpapers, &config, active_monitor_idx, show_hidden);
                            selected_index = selected_index.min(displayed_wallpapers.len().saturating_sub(1));
                            target_scroll = selected_index as f32;
                            window.request_redraw();
                        }
                    }
                    if !is_animating_scroll {
                        target.set_control_flow(ControlFlow::wait_duration(std::time::Duration::from_millis(250)));
                    }
                }
            }

            _ => {}
        }
    }).unwrap();
}
