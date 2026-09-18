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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_true")]
    pub show_top_bar: bool,
    #[serde(default = "default_true")]
    pub show_monitor_tabs: bool,
    #[serde(default = "default_true")]
    pub show_title: bool,
    #[serde(default = "default_true")]
    pub show_hidden_button: bool,
    #[serde(default = "default_true")]
    pub show_close_button: bool,
    #[serde(default = "default_true")]
    pub show_hint_bar: bool,
    #[serde(default = "default_true")]
    pub show_hidden_badge: bool,
    #[serde(default = "default_true")]
    pub tint_excluded: bool,
    #[serde(default = "default_true")]
    pub dim_unselected: bool,
    #[serde(default = "default_true")]
    pub show_selection_border: bool,
    #[serde(default = "default_true")]
    pub close_on_backdrop_click: bool,
    #[serde(default = "default_true")]
    pub close_on_focus_loss: bool,
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

impl Config {
    pub fn config_path() -> PathBuf {
        let base = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")));
        base.join("we-gallery").join("config.toml")
    }

    pub fn last_modified() -> Option<std::time::SystemTime> {
        fs::metadata(Self::config_path()).ok().and_then(|m| m.modified().ok())
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(val) = toml::from_str::<toml::Value>(&contents) {
                    let mut cfg = Config::default();
                    if let Some(preset) = val.get("theme").and_then(|t| t.get("preset")).and_then(|p| p.as_str()) {
                        cfg.theme.preset = preset.to_string();
                        cfg.apply_theme_preset();
                    }
                    if let Ok(user_cfg) = toml::from_str::<Config>(&contents) {
                        if let Some(style_val) = val.get("style").and_then(|s| s.as_table()) {
                            let mut final_style = cfg.style.clone();
                            if style_val.contains_key("accent_color") { final_style.accent_color = user_cfg.style.accent_color.clone(); }
                            if style_val.contains_key("bg_color") { final_style.bg_color = user_cfg.style.bg_color.clone(); }
                            if style_val.contains_key("surface_color") { final_style.surface_color = user_cfg.style.surface_color.clone(); }
                            if style_val.contains_key("overlay_color") { final_style.overlay_color = user_cfg.style.overlay_color.clone(); }
                            if style_val.contains_key("muted_color") { final_style.muted_color = user_cfg.style.muted_color.clone(); }
                            if style_val.contains_key("text_color") { final_style.text_color = user_cfg.style.text_color.clone(); }
                            if style_val.contains_key("highlight_color") { final_style.highlight_color = user_cfg.style.highlight_color.clone(); }
                            if style_val.contains_key("card_width") { final_style.card_width = user_cfg.style.card_width; }
                            if style_val.contains_key("card_height") { final_style.card_height = user_cfg.style.card_height; }
                            if style_val.contains_key("card_shear") { final_style.card_shear = user_cfg.style.card_shear; }
                            if style_val.contains_key("border_width") { final_style.border_width = user_cfg.style.border_width; }
                            if style_val.contains_key("dim_unselected") { final_style.dim_unselected = user_cfg.style.dim_unselected; }
                            if style_val.contains_key("backdrop_opacity") { final_style.backdrop_opacity = user_cfg.style.backdrop_opacity; }
                            if style_val.contains_key("border_glow") { final_style.border_glow = user_cfg.style.border_glow; }
                            if style_val.contains_key("glow_radius") { final_style.glow_radius = user_cfg.style.glow_radius; }
                            if style_val.contains_key("motion_blur") { final_style.motion_blur = user_cfg.style.motion_blur; }
                            if style_val.contains_key("motion_blur_strength") { final_style.motion_blur_strength = user_cfg.style.motion_blur_strength; }
                            cfg = user_cfg;
                            cfg.style = final_style;
                        } else {
                            cfg = user_cfg;
                        }
                        return cfg;
                    }
                }
            }
        }
        let cfg = Self::default();
        let _ = cfg.save();
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
