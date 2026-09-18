use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_preset")]
    pub preset: String,
}

fn default_preset() -> String {
    "rose-pine-moon".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    #[serde(default = "default_accent_color")]
    pub accent_color: String,
    #[serde(default = "default_bg_color")]
    pub bg_color: String,
    #[serde(default = "default_surface_color")]
    pub surface_color: String,
    #[serde(default = "default_overlay_color")]
    pub overlay_color: String,
    #[serde(default = "default_muted_color")]
    pub muted_color: String,
    #[serde(default = "default_text_color")]
    pub text_color: String,
    #[serde(default = "default_highlight_color")]
    pub highlight_color: String,

    #[serde(default = "default_card_width")]
    pub card_width: u32,
    #[serde(default = "default_card_height")]
    pub card_height: u32,
    #[serde(default = "default_card_shear")]
    pub card_shear: f32,
    #[serde(default = "default_border_width")]
    pub border_width: f32,
    #[serde(default = "default_dim_unselected")]
    pub dim_unselected: f32,
    #[serde(default = "default_backdrop_opacity")]
    pub backdrop_opacity: f32,
    #[serde(default = "default_border_glow")]
    pub border_glow: bool,
    #[serde(default = "default_glow_radius")]
    pub glow_radius: f32,
    #[serde(default = "default_motion_blur")]
    pub motion_blur: bool,
    #[serde(default = "default_motion_blur_strength")]
    pub motion_blur_strength: f32,
}

fn default_accent_color() -> String { "#eb6f92".to_string() } // Love
fn default_bg_color() -> String { "#232136".to_string() }     // Base
fn default_surface_color() -> String { "#2a273f".to_string() }// Surface
fn default_overlay_color() -> String { "#393552".to_string() }// Overlay
fn default_muted_color() -> String { "#6e6a86".to_string() }  // Muted
fn default_text_color() -> String { "#e0def4".to_string() }   // Text
fn default_highlight_color() -> String { "#44415a".to_string() } // Highlight Med
fn default_card_width() -> u32 { 300 }
fn default_card_height() -> u32 { 533 }
fn default_card_shear() -> f32 { 0.21 }
fn default_border_width() -> f32 { 2.0 }
fn default_dim_unselected() -> f32 { 0.40 }
fn default_backdrop_opacity() -> f32 { 0.72 }
fn default_border_glow() -> bool { true }
fn default_glow_radius() -> f32 { 8.0 }
fn default_motion_blur() -> bool { true }
fn default_motion_blur_strength() -> f32 { 1.0 }

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            accent_color: default_accent_color(),
            bg_color: default_bg_color(),
            surface_color: default_surface_color(),
            overlay_color: default_overlay_color(),
            muted_color: default_muted_color(),
            text_color: default_text_color(),
            highlight_color: default_highlight_color(),
            card_width: default_card_width(),
            card_height: default_card_height(),
            card_shear: default_card_shear(),
            border_width: default_border_width(),
            dim_unselected: default_dim_unselected(),
            backdrop_opacity: default_backdrop_opacity(),
            border_glow: default_border_glow(),
            glow_radius: default_glow_radius(),
            motion_blur: default_motion_blur(),
            motion_blur_strength: default_motion_blur_strength(),
        }
    }
}

fn deserialize_flexible_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct BoolOrNumVisitor;

    impl<'de> serde::de::Visitor<'de> for BoolOrNumVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a boolean or boolean-like value (true/false, 1/0, 'true'/'false')")
        }

        fn visit_bool<E>(self, v: bool) -> Result<bool, E> {
            Ok(v)
        }

        fn visit_i64<E>(self, v: i64) -> Result<bool, E> {
            Ok(v != 0)
        }

        fn visit_u64<E>(self, v: u64) -> Result<bool, E> {
            Ok(v != 0)
        }

        fn visit_f64<E>(self, v: f64) -> Result<bool, E> {
            Ok(v > 0.0)
        }

        fn visit_str<E>(self, v: &str) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            match v.to_lowercase().trim() {
                "true" | "yes" | "1" | "on" | "enable" | "enabled" => Ok(true),
                "false" | "no" | "0" | "off" | "disable" | "disabled" => Ok(false),
                _ => Err(E::custom(format!("invalid boolean string '{}'", v))),
            }
        }
    }

    deserializer.deserialize_any(BoolOrNumVisitor)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool", alias = "top_bar")]
    pub show_top_bar: bool,
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool", alias = "monitor_tabs")]
    pub show_monitor_tabs: bool,
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool", alias = "title")]
    pub show_title: bool,
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool", alias = "hidden_button")]
    pub show_hidden_button: bool,
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool", alias = "close_button")]
    pub show_close_button: bool,
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool", alias = "hint_bar")]
    pub show_hint_bar: bool,
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool", alias = "hidden_badge")]
    pub show_hidden_badge: bool,
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool", alias = "tint_excluded_cards")]
    pub tint_excluded: bool,
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool", alias = "dim_unselected_cards")]
    pub dim_unselected: bool,
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool", alias = "selection_border", alias = "show_border")]
    pub show_selection_border: bool,
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool")]
    pub close_on_backdrop_click: bool,
    #[serde(default = "default_true", deserialize_with = "deserialize_flexible_bool")]
    pub close_on_focus_loss: bool,

    // Fallbacks: allow style & geometry settings under [ui] as well
    #[serde(default)]
    pub backdrop_opacity: Option<f32>,
    #[serde(default)]
    pub card_width: Option<u32>,
    #[serde(default)]
    pub card_height: Option<u32>,
    #[serde(default)]
    pub card_shear: Option<f32>,
    #[serde(default)]
    pub border_width: Option<f32>,
    #[serde(default)]
    pub border_glow: Option<bool>,
    #[serde(default)]
    pub glow_radius: Option<f32>,
    #[serde(default)]
    pub motion_blur: Option<bool>,
    #[serde(default)]
    pub motion_blur_strength: Option<f32>,
}

fn default_true() -> bool { true }

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_top_bar: true,
            show_monitor_tabs: true,
            show_title: true,
            show_hidden_button: true,
            show_close_button: true,
            show_hint_bar: true,
            show_hidden_badge: true,
            tint_excluded: true,
            dim_unselected: true,
            show_selection_border: true,
            close_on_backdrop_click: true,
            close_on_focus_loss: true,
            backdrop_opacity: None,
            card_width: None,
            card_height: None,
            card_shear: None,
            border_width: None,
            border_glow: None,
            glow_radius: None,
            motion_blur: None,
            motion_blur_strength: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindsConfig {
    #[serde(default = "default_toggle_gallery")]
    pub toggle_gallery: String,
    #[serde(default = "default_apply_wallpaper")]
    pub apply_wallpaper: String,
    #[serde(default = "default_apply_to_all")]
    pub apply_to_all: String,
    #[serde(default = "default_close")]
    pub close: String,
    #[serde(default = "default_left")]
    pub left: String,
    #[serde(default = "default_right")]
    pub right: String,
    #[serde(default = "default_page_left")]
    pub page_left: String,
    #[serde(default = "default_page_right")]
    pub page_right: String,
    #[serde(default = "default_first")]
    pub first: String,
    #[serde(default = "default_last")]
    pub last: String,
    #[serde(default = "default_switch_monitor")]
    pub switch_monitor: String,
    #[serde(default = "default_toggle_exclude")]
    pub toggle_exclude: String,
    #[serde(default = "default_toggle_hidden")]
    pub toggle_hidden: String,
    #[serde(default = "default_open_in_explorer")]
    pub open_in_explorer: String,
    #[serde(default = "default_reload_config")]
    pub reload_config: String,
}

fn default_toggle_gallery() -> String { "ctrl+alt+g".to_string() }
fn default_apply_wallpaper() -> String { "return".to_string() }
fn default_apply_to_all() -> String { "ctrl+return".to_string() }
fn default_close() -> String { "escape".to_string() }
fn default_left() -> String { "left".to_string() }
fn default_right() -> String { "right".to_string() }
fn default_page_left() -> String { "pageup".to_string() }
fn default_page_right() -> String { "pagedown".to_string() }
fn default_first() -> String { "home".to_string() }
fn default_last() -> String { "end".to_string() }
fn default_switch_monitor() -> String { "tab".to_string() }
fn default_toggle_exclude() -> String { "x".to_string() }
fn default_toggle_hidden() -> String { "h".to_string() }
fn default_open_in_explorer() -> String { "e".to_string() }
fn default_reload_config() -> String { "f5".to_string() }

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            toggle_gallery: default_toggle_gallery(),
            apply_wallpaper: default_apply_wallpaper(),
            apply_to_all: default_apply_to_all(),
            close: default_close(),
            left: default_left(),
            right: default_right(),
            page_left: default_page_left(),
            page_right: default_page_right(),
            first: default_first(),
            last: default_last(),
            switch_monitor: default_switch_monitor(),
            toggle_exclude: default_toggle_exclude(),
            toggle_hidden: default_toggle_hidden(),
            open_in_explorer: default_open_in_explorer(),
            reload_config: default_reload_config(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    #[serde(default = "default_true")]
    pub follow_cursor: bool,
    #[serde(default = "default_false")]
    pub show_excluded: bool,
}

fn default_false() -> bool { false }

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            follow_cursor: true,
            show_excluded: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub style: StyleConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub keybinds: KeybindsConfig,
    #[serde(default)]
    pub behavior: BehaviorConfig,
    #[serde(default)]
    pub excluded_wallpapers: HashMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        let mut excluded = HashMap::new();
        excluded.insert("Monitor0".to_string(), Vec::new());
        excluded.insert("Monitor1".to_string(), Vec::new());

        Self {
            theme: ThemeConfig {
                preset: "rose-pine-moon".to_string(),
            },
            style: StyleConfig::default(),
            ui: UiConfig::default(),
            keybinds: KeybindsConfig::default(),
            behavior: BehaviorConfig::default(),
            excluded_wallpapers: excluded,
        }
    }
}

pub fn log_debug(msg: &str) {
    let dir = Config::config_dir();
    let _ = fs::create_dir_all(&dir);
    let log_file = dir.join("debug.log");
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(log_file) {
        use std::io::Write;
        let _ = writeln!(f, "[DEBUG] {}", msg);
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg).join("we-gallery");
        }
        if let Some(home) = dirs::home_dir() {
            return home.join(".config").join("we-gallery");
        }
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("we-gallery")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn last_modified() -> Option<std::time::SystemTime> {
        fs::metadata(Self::config_path()).ok().and_then(|m| m.modified().ok())
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            // Check legacy AppData path to migrate seamlessly if it exists
            if let Some(legacy_base) = dirs::config_dir() {
                let legacy_path = legacy_base.join("we-gallery").join("config.toml");
                if legacy_path.exists() {
                    if let Ok(contents) = fs::read_to_string(&legacy_path) {
                        let _ = fs::create_dir_all(Self::config_dir());
                        let _ = fs::write(&path, contents);
                    }
                }
            }
        }

        if !path.exists() {
            let cfg = Self::default();
            let _ = cfg.save();
            return cfg;
        }

        let contents = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                log_debug(&format!("Failed to read config {}: {}", path.display(), e));
                return Self::default();
            }
        };

        log_debug(&format!("Loading configuration from {}", path.display()));

        let val: toml::Value = match toml::from_str(&contents) {
            Ok(v) => v,
            Err(e) => {
                log_debug(&format!("TOML syntax error in {}: {}", path.display(), e));
                return Self::default();
            }
        };

        let mut cfg = Self::default();

        // 1. Theme preset
        if let Some(preset) = val.get("theme").and_then(|t| t.get("preset")).and_then(|p| p.as_str()) {
            cfg.theme.preset = preset.to_string();
            cfg.apply_theme_preset();
        }

        // 2. Style section
        if let Some(style_val) = val.get("style") {
            match style_val.clone().try_into::<StyleConfig>() {
                Ok(user_style) => {
                    if let Some(tbl) = style_val.as_table() {
                        if tbl.contains_key("accent_color") { cfg.style.accent_color = user_style.accent_color; }
                        if tbl.contains_key("bg_color") { cfg.style.bg_color = user_style.bg_color; }
                        if tbl.contains_key("surface_color") { cfg.style.surface_color = user_style.surface_color; }
                        if tbl.contains_key("overlay_color") { cfg.style.overlay_color = user_style.overlay_color; }
                        if tbl.contains_key("muted_color") { cfg.style.muted_color = user_style.muted_color; }
                        if tbl.contains_key("text_color") { cfg.style.text_color = user_style.text_color; }
                        if tbl.contains_key("highlight_color") { cfg.style.highlight_color = user_style.highlight_color; }
                        if tbl.contains_key("card_width") { cfg.style.card_width = user_style.card_width; }
                        if tbl.contains_key("card_height") { cfg.style.card_height = user_style.card_height; }
                        if tbl.contains_key("card_shear") { cfg.style.card_shear = user_style.card_shear; }
                        if tbl.contains_key("border_width") { cfg.style.border_width = user_style.border_width; }
                        if tbl.contains_key("dim_unselected") { cfg.style.dim_unselected = user_style.dim_unselected; }
                        if tbl.contains_key("backdrop_opacity") { cfg.style.backdrop_opacity = user_style.backdrop_opacity; }
                        if tbl.contains_key("border_glow") { cfg.style.border_glow = user_style.border_glow; }
                        if tbl.contains_key("glow_radius") { cfg.style.glow_radius = user_style.glow_radius; }
                        if tbl.contains_key("motion_blur") { cfg.style.motion_blur = user_style.motion_blur; }
                        if tbl.contains_key("motion_blur_strength") { cfg.style.motion_blur_strength = user_style.motion_blur_strength; }
                    }
                }
                Err(e) => {
                    log_debug(&format!("Failed to deserialize [style] section: {}", e));
                }
            }
        }

        // 3. UI section
        if let Some(ui_val) = val.get("ui") {
            match ui_val.clone().try_into::<UiConfig>() {
                Ok(user_ui) => {
                    if let Some(bo) = user_ui.backdrop_opacity { cfg.style.backdrop_opacity = bo; }
                    if let Some(cw) = user_ui.card_width { cfg.style.card_width = cw; }
                    if let Some(ch) = user_ui.card_height { cfg.style.card_height = ch; }
                    if let Some(cs) = user_ui.card_shear { cfg.style.card_shear = cs; }
                    if let Some(bw) = user_ui.border_width { cfg.style.border_width = bw; }
                    if let Some(bg) = user_ui.border_glow { cfg.style.border_glow = bg; }
                    if let Some(gr) = user_ui.glow_radius { cfg.style.glow_radius = gr; }
                    if let Some(mb) = user_ui.motion_blur { cfg.style.motion_blur = mb; }
                    if let Some(mbs) = user_ui.motion_blur_strength { cfg.style.motion_blur_strength = mbs; }
                    log_debug(&format!(
                        "Loaded [ui] config: show_top_bar={}, show_hint_bar={}, show_selection_border={}, dim_unselected={}",
                        user_ui.show_top_bar, user_ui.show_hint_bar, user_ui.show_selection_border, user_ui.dim_unselected
                    ));
                    cfg.ui = user_ui;
                }
                Err(e) => {
                    log_debug(&format!("Failed to deserialize [ui] section: {}", e));
                }
            }
        }

        // 4. Keybinds section
        if let Some(kb_val) = val.get("keybinds") {
            match kb_val.clone().try_into::<KeybindsConfig>() {
                Ok(user_kb) => cfg.keybinds = user_kb,
                Err(e) => log_debug(&format!("Failed to deserialize [keybinds] section: {}", e)),
            }
        }

        // 5. Behavior section
        if let Some(beh_val) = val.get("behavior") {
            match beh_val.clone().try_into::<BehaviorConfig>() {
                Ok(user_beh) => cfg.behavior = user_beh,
                Err(e) => log_debug(&format!("Failed to deserialize [behavior] section: {}", e)),
            }
        }

        // 6. Excluded Wallpapers section
        if let Some(ex_val) = val.get("excluded_wallpapers") {
            if let Ok(user_ex) = ex_val.clone().try_into::<HashMap<String, Vec<String>>>() {
                cfg.excluded_wallpapers = user_ex;
            }
        }

        cfg
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).unwrap_or_default();
        fs::write(path, content)
    }

    pub fn apply_theme_preset(&mut self) {
        match self.theme.preset.to_lowercase().as_str() {
            "rose-pine-moon" => {
                // Rosé Pine Moon exact palette
                self.style.accent_color = "#eb6f92".to_string(); // Love
                self.style.bg_color = "#232136".to_string();     // Base
                self.style.surface_color = "#2a273f".to_string();// Surface
                self.style.overlay_color = "#393552".to_string();// Overlay
                self.style.muted_color = "#6e6a86".to_string();  // Muted
                self.style.text_color = "#e0def4".to_string();   // Text
                self.style.highlight_color = "#44415a".to_string(); // Highlight Med
            }
            "rose-pine" => {
                self.style.accent_color = "#eb6f92".to_string();
                self.style.bg_color = "#191724".to_string();
                self.style.surface_color = "#26233a".to_string();
                self.style.overlay_color = "#312f44".to_string();
                self.style.muted_color = "#6e6a86".to_string();
                self.style.text_color = "#e0def4".to_string();
                self.style.highlight_color = "#403d52".to_string();
            }
            "catppuccin-mocha" => {
                self.style.accent_color = "#f5c2e7".to_string(); // Pink
                self.style.bg_color = "#1e1e2e".to_string();     // Base
                self.style.surface_color = "#313244".to_string();// Surface0
                self.style.overlay_color = "#45475a".to_string();// Surface1
                self.style.muted_color = "#6c7086".to_string();  // Overlay0
                self.style.text_color = "#cdd6f4".to_string();   // Text
                self.style.highlight_color = "#585b70".to_string();
            }
            "tokyo-night" => {
                self.style.accent_color = "#bb9af7".to_string();
                self.style.bg_color = "#1a1b26".to_string();
                self.style.surface_color = "#24283b".to_string();
                self.style.overlay_color = "#414868".to_string();
                self.style.muted_color = "#565f89".to_string();
                self.style.text_color = "#c0caf5".to_string();
                self.style.highlight_color = "#7aa2f7".to_string();
            }
            "nord" => {
                self.style.accent_color = "#88c0d0".to_string();
                self.style.bg_color = "#2e3440".to_string();
                self.style.surface_color = "#3b4252".to_string();
                self.style.overlay_color = "#434c5e".to_string();
                self.style.muted_color = "#4c566a".to_string();
                self.style.text_color = "#eceff4".to_string();
                self.style.highlight_color = "#81a1c1".to_string();
            }
            _ => {} // custom
        }
    }

    pub fn is_excluded(&self, monitor_key: &str, id: &str) -> bool {
        self.excluded_wallpapers
            .get(monitor_key)
            .map(|list| list.iter().any(|item| item == id))
            .unwrap_or(false)
    }

    pub fn toggle_excluded(&mut self, monitor_key: &str, id: &str) -> bool {
        let list = self.excluded_wallpapers.entry(monitor_key.to_string()).or_default();
        if let Some(pos) = list.iter().position(|x| x == id) {
            list.remove(pos);
            let _ = self.save();
            false
        } else {
            list.push(id.to_string());
            let _ = self.save();
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flexible_ui_parsing() {
        let toml_str = r#"
        [ui]
        show_top_bar = false
        show_hint_bar = "false"
        tint_excluded_cards = false
        dim_unselected = 0.5
        backdrop_opacity = 0.45
        card_width = 320
        "#;

        let val: toml::Value = toml::from_str(toml_str).unwrap();
        let ui_val = val.get("ui").unwrap();
        let ui: UiConfig = ui_val.clone().try_into().unwrap();

        assert_eq!(ui.show_top_bar, false);
        assert_eq!(ui.show_hint_bar, false);
        assert_eq!(ui.tint_excluded, false);
        assert_eq!(ui.dim_unselected, true);
        assert_eq!(ui.backdrop_opacity, Some(0.45));
        assert_eq!(ui.card_width, Some(320));
    }

    #[test]
    fn test_flexible_ui_bool_variants() {
        let toml_str = r#"
        [ui]
        show_top_bar = 0
        show_hint_bar = "off"
        show_title = "no"
        show_close_button = "disabled"
        show_hidden_button = 1
        "#;

        let val: toml::Value = toml::from_str(toml_str).unwrap();
        let ui_val = val.get("ui").unwrap();
        let ui: UiConfig = ui_val.clone().try_into().unwrap();

        assert_eq!(ui.show_top_bar, false);
        assert_eq!(ui.show_hint_bar, false);
        assert_eq!(ui.show_title, false);
        assert_eq!(ui.show_close_button, false);
        assert_eq!(ui.show_hidden_button, true);
    }
}
