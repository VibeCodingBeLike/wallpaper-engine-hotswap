use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]

#[derive(Debug, Clone)]
pub struct WallpaperItem {
    pub id: String,
    pub title: String,
    pub folder_path: PathBuf,
    pub project_json_path: PathBuf,
    pub file_target: PathBuf,
    pub preview_path: Option<PathBuf>,
    pub wallpaper_type: String,
}

#[derive(Deserialize)]
struct ProjectJson {
    title: Option<String>,
    file: Option<String>,
    preview: Option<String>,
    #[serde(rename = "type")]
    wp_type: Option<String>,
    category: Option<String>,
}

fn find_steam_registry_path() -> Option<PathBuf> {
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_SZ,
    };

    let subkey = "Software\\Valve\\Steam\0";
    let subkey_w: Vec<u16> = subkey.encode_utf16().collect();
    let mut hkey: HKEY = std::ptr::null_mut();

    unsafe {
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey_w.as_ptr(), 0, KEY_READ, &mut hkey) != 0 || hkey.is_null() {
            return None;
        }
    }

    let value_name = "SteamPath\0";
    let value_name_w: Vec<u16> = value_name.encode_utf16().collect();
    let mut type_out = 0u32;
    let mut buffer: [u16; 512] = [0; 512];
    let mut bytes_len = (buffer.len() * std::mem::size_of::<u16>()) as u32;

    let status = unsafe {
        RegQueryValueExW(
            hkey,
            value_name_w.as_ptr(),
            std::ptr::null_mut(),
            &mut type_out,
            buffer.as_mut_ptr() as *mut u8,
            &mut bytes_len,
        )
    };

    unsafe {
        RegCloseKey(hkey);
    }

    if status != 0 || type_out != REG_SZ {
        return None;
    }

    let used = (bytes_len as usize / std::mem::size_of::<u16>())
        .min(buffer.len())
        .max(1);
    let string_end = buffer[..used]
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(used);
    let steam_path = String::from_utf16_lossy(&buffer[..string_end]).trim().to_string();
    if steam_path.is_empty() {
        return None;
    }

    let mut path = PathBuf::from(steam_path.replace('/', "\\"));
    path = path.canonicalize().unwrap_or(path);
    if path.exists() {
        Some(path)
    } else {
        Some(PathBuf::from(steam_path.replace('/', "\\")))
    }
}

fn parse_libraryfolders_vdf(vdf_path: &std::path::Path, libraries: &mut Vec<PathBuf>) {
    let Ok(content) = fs::read_to_string(vdf_path) else {
        return;
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("\"path\"") {
            continue;
        }

        let parts: Vec<&str> = trimmed.split('"').collect();
        if parts.len() < 4 {
            continue;
        }

        let path_str = parts[3].replace(r"\\", r"\");
        let path = PathBuf::from(path_str.replace('/', "\\"));
        if path.exists() && !libraries.iter().any(|lib| lib == &path) {
            libraries.push(path);
        }
    }
}

fn find_steam_libraries() -> Vec<PathBuf> {
    let mut libraries = Vec::new();

    for root in [
        find_steam_registry_path(),
        Some(PathBuf::from(r"C:\Program Files (x86)\Steam")),
        Some(PathBuf::from(r"C:\Program Files\Steam")),
    ] {
        let Some(root) = root else {
            continue;
        };

        if root.exists() && !libraries.iter().any(|lib| lib == &root) {
            libraries.push(root.clone());
        }

        let vdf = root.join("steamapps").join("libraryfolders.vdf");
        if vdf.exists() {
            parse_libraryfolders_vdf(&vdf, &mut libraries);
        }
    }

    libraries.into_iter().filter(|p| p.exists()).collect()
}

pub fn scan_wallpapers(include_default_projects: bool) -> Vec<WallpaperItem> {
    let mut items = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    let mut search_dirs = Vec::new();
    for lib in find_steam_libraries() {
        let ws = lib.join("steamapps").join("workshop").join("content").join("431960");
        if ws.exists() {
            search_dirs.push(ws);
        }
        let we_projects = lib.join("steamapps").join("common").join("wallpaper_engine").join("projects");
        if we_projects.exists() {
            search_dirs.push(we_projects.join("myprojects"));
            if include_default_projects {
                search_dirs.push(we_projects.join("defaultprojects"));
            }
        }
    }

    for dir in search_dirs {
        if !dir.exists() {
            continue;
        }
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let id = entry.file_name().to_string_lossy().to_string();
            if seen_ids.contains(&id) {
                continue;
            }

            let pj_path = path.join("project.json");
            if !pj_path.exists() {
                continue;
            }

            let pj_content = fs::read_to_string(&pj_path).unwrap_or_default();
            let pj: ProjectJson = match serde_json::from_str(&pj_content) {
                Ok(p) => p,
                Err(_) => continue,
            };

            // Filter out non-wallpaper assets (editor effects, presets, shaders, templates)
            if let Some(ref cat) = pj.category {
                let cat_lower = cat.to_lowercase();
                if cat_lower == "asset" || cat_lower == "preset" || cat_lower == "template" {
                    continue;
                }
            }

            // Skip items with missing or blank title (templates, internal assets)
            let title = match pj.title {
                Some(t) if !t.trim().is_empty() => t,
                _ => continue,
            };

            let file_rel = pj.file.unwrap_or_else(|| "scene.json".to_string());
            let file_lower = file_rel.to_lowercase();
            if file_lower.ends_with("effect.json") || file_lower.ends_with("shader.json") {
                continue;
            }
            let file_target = path.join(&file_rel);

            let preview_rel = pj.preview.unwrap_or_else(|| "preview.jpg".to_string());
            let mut preview_path = Some(path.join(preview_rel));

            if let Some(ref p) = preview_path {
                if !p.exists() {
                    preview_path = None;
                    for cand in &["preview.jpg", "preview.gif", "preview.png", "preview.jpeg"] {
                        let cp = path.join(cand);
                        if cp.exists() {
                            preview_path = Some(cp);
                            break;
                        }
                    }
                }
            }

            seen_ids.insert(id.clone());
            items.push(WallpaperItem {
                id,
                title,
                folder_path: path,
                project_json_path: pj_path.clone(),
                file_target: if file_target.exists() { file_target } else { pj_path },
                preview_path,
                wallpaper_type: pj.wp_type.unwrap_or_else(|| "scene".to_string()),
            });
        }
    }

    items.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    items
}

pub fn get_active_wallpaper_file(monitor_idx: usize) -> Option<String> {
    let mut config_paths = Vec::new();
    for lib in find_steam_libraries() {
        let cp = lib.join("steamapps").join("common").join("wallpaper_engine").join("config.json");
        if cp.exists() {
            config_paths.push(cp);
        }
    }

    let mon_key = format!("Monitor{}", monitor_idx);

    for cp in config_paths {
        if let Ok(data) = fs::read_to_string(&cp) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(obj) = val.as_object() {
                    for (_user, user_val) in obj {
                        if let Some(file) = user_val
                            .get("general")
                            .and_then(|g| g.get("wallpaperconfig"))
                            .and_then(|wc| wc.get("selectedwallpapers"))
                            .and_then(|sw| sw.get(&mon_key))
                            .and_then(|m| m.get("file"))
                            .and_then(|f| f.as_str())
                        {
                            return Some(file.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn find_active_wallpaper_index(wallpapers: &[WallpaperItem], active_file: &str) -> Option<usize> {
    let norm = active_file.to_lowercase().replace('/', "\\");

    // 1. Try exact match on file_target
    for (i, wp) in wallpapers.iter().enumerate() {
        let ft = wp.file_target.to_string_lossy().to_lowercase().replace('/', "\\");
        if ft == norm {
            return Some(i);
        }
    }

    // 2. Try prefix match on folder_path
    for (i, wp) in wallpapers.iter().enumerate() {
        let fp = wp.folder_path.to_string_lossy().to_lowercase().replace('/', "\\");
        if norm.starts_with(&fp) {
            return Some(i);
        }
    }

    // 3. Try contains wallpaper id
    for (i, wp) in wallpapers.iter().enumerate() {
        if norm.contains(&wp.id.to_lowercase()) {
            return Some(i);
        }
    }

    None
}
