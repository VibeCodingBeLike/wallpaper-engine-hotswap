use fontdue::{Font, FontSettings};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tiny_skia::*;

use crate::config::Config;
use crate::scanner::WallpaperItem;

pub fn hex_to_color(hex: &str, alpha: f32) -> Color {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return Color::from_rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, alpha.clamp(0.0, 1.0)).unwrap_or(Color::WHITE);
        }
    }
    Color::WHITE
}

pub fn push_rounded_rect(pb: &mut PathBuilder, rect: Rect, radius: f32) {
    let x = rect.left();
    let y = rect.top();
    let w = rect.width();
    let h = rect.height();
    let r = radius.min(w / 2.0).min(h / 2.0);

    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
}

pub struct FontRenderer {
    regular: Option<Font>,
    bold: Option<Font>,
    fallbacks: Vec<Font>,
}

impl FontRenderer {
    pub fn new() -> Self {
        let regular = fs::read(r"C:\Windows\Fonts\segoeui.ttf")
            .ok()
            .and_then(|data| Font::from_bytes(data, FontSettings::default()).ok());
        let bold = fs::read(r"C:\Windows\Fonts\segoeuib.ttf")
            .ok()
            .and_then(|data| Font::from_bytes(data, FontSettings::default()).ok())
            .or_else(|| regular.clone());

        let mut fallbacks = Vec::new();
        // Fallback fonts for Chinese, Japanese, Korean, symbols, and emojis
        let fallback_paths = [
            r"C:\Windows\Fonts\msyh.ttc",    // Microsoft YaHei (Chinese, Japanese Kanji/Kana)
            r"C:\Windows\Fonts\meiryo.ttc",  // Meiryo (Japanese)
            r"C:\Windows\Fonts\malgun.ttf",  // Malgun Gothic (Korean)
            r"C:\Windows\Fonts\seguisym.ttf",// Segoe UI Symbol
            r"C:\Windows\Fonts\seguiemj.ttf",// Segoe UI Emoji
            r"C:\Windows\Fonts\YuGothR.ttc", // Yu Gothic
        ];

        for path in fallback_paths {
            if let Ok(data) = fs::read(path) {
                if let Ok(f) = Font::from_bytes(data, FontSettings::default()) {
                    fallbacks.push(f);
                }
            }
        }

        Self { regular, bold, fallbacks }
    }

    pub fn get_font_for_char<'a>(&'a self, c: char, bold: bool) -> Option<&'a Font> {
        let primary = if bold { self.bold.as_ref() } else { self.regular.as_ref() };
        if let Some(f) = primary {
            if f.lookup_glyph_index(c) != 0 {
                return Some(f);
            }
        }
        for f in &self.fallbacks {
            if f.lookup_glyph_index(c) != 0 {
                return Some(f);
            }
        }
        primary
    }

    pub fn draw_text(
        &self,
        pixmap: &mut Pixmap,
        text: &str,
        start_x: f32,
        baseline_y: f32,
        size: f32,
        color: Color,
        bold: bool,
    ) -> f32 {
        let mut current_x = start_x;
        let p_width = pixmap.width() as i32;
        let p_height = pixmap.height() as i32;
        let p_stride = pixmap.width() as usize;

        for c in text.chars() {
            let font = match self.get_font_for_char(c, bold) {
                Some(f) => f,
                None => continue,
            };

            let (metrics, bitmap) = font.rasterize(c, size);
            if !bitmap.is_empty() {
                let gx = current_x as i32 + metrics.xmin;
                let gy = (baseline_y as i32) - metrics.height as i32 - metrics.ymin;

                let cr = color.red() * color.alpha();
                let cg = color.green() * color.alpha();
                let cb = color.blue() * color.alpha();
                let ca = color.alpha();

                let data = pixmap.data_mut();

                for row in 0..metrics.height {
                    let py = gy + row as i32;
                    if py < 0 || py >= p_height {
                        continue;
                    }
                    let row_offset = (py as usize) * p_stride;

                    for col in 0..metrics.width {
                        let px = gx + col as i32;
                        if px < 0 || px >= p_width {
                            continue;
                        }

                        let cov = bitmap[row * metrics.width + col] as f32 / 255.0;
                        if cov > 0.01 {
                            let src_a = (ca * cov * 255.0).round() as u32;
                            let src_r = (cr * cov * 255.0).round() as u32;
                            let src_g = (cg * cov * 255.0).round() as u32;
                            let src_b = (cb * cov * 255.0).round() as u32;

                            let inv = 255 - src_a;
                            let byte_idx = (row_offset + px as usize) * 4;

                            let dr = data[byte_idx] as u32;
                            let dg = data[byte_idx + 1] as u32;
                            let db = data[byte_idx + 2] as u32;
                            let da = data[byte_idx + 3] as u32;

                            data[byte_idx] = (src_r + (dr * inv + 127) / 255).min(255) as u8;
                            data[byte_idx + 1] = (src_g + (dg * inv + 127) / 255).min(255) as u8;
                            data[byte_idx + 2] = (src_b + (db * inv + 127) / 255).min(255) as u8;
                            data[byte_idx + 3] = (src_a + (da * inv + 127) / 255).min(255) as u8;
                        }
                    }
                }
            }
            current_x += metrics.advance_width;
        }
        current_x - start_x
    }

    pub fn text_width(&self, text: &str, size: f32, bold: bool) -> f32 {
        let mut width = 0.0;
        for c in text.chars() {
            if let Some(font) = self.get_font_for_char(c, bold) {
                let metrics = font.metrics(c, size);
                width += metrics.advance_width;
            }
        }
        width
    }
}

#[derive(Debug, Clone)]
pub enum HitShape {
    Rect(Rect),
    Parallelogram {
        centre_x: f32,
        centre_y: f32,
        half_w: f32,
        half_h: f32,
        shear: f32,
    },
}

#[derive(Debug, Clone)]
pub struct HitBox {
    pub shape: HitShape,
    pub action: HitAction,
}

impl HitBox {
    pub fn rect(rect: Rect, action: HitAction) -> Self {
        Self {
            shape: HitShape::Rect(rect),
            action,
        }
    }

    pub fn parallelogram(
        centre_x: f32,
        centre_y: f32,
        half_w: f32,
        half_h: f32,
        shear: f32,
        action: HitAction,
    ) -> Self {
        Self {
            shape: HitShape::Parallelogram {
                centre_x,
                centre_y,
                half_w,
                half_h,
                shear,
            },
            action,
        }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        match &self.shape {
            HitShape::Rect(r) => {
                x >= r.left() && x <= r.right() && y >= r.top() && y <= r.bottom()
            }
            HitShape::Parallelogram {
                centre_x,
                centre_y,
                half_w,
                half_h,
                shear,
            } => {
                let dy = y - centre_y;
                if dy.abs() > *half_h {
                    return false;
                }
                let cx_at_y = centre_x - dy * shear;
                (x - cx_at_y).abs() <= *half_w
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum HitAction {
    SelectCard(usize),
    SwitchMonitor(usize),
    ToggleHidden,
    Close,
}

#[derive(Clone, Copy, Debug)]
pub struct RowSpan {
    pub x_start: u16,
    pub x_solid_start: u16,
    pub x_solid_end: u16,
    pub x_end: u16,
}

pub struct CachedCard {
    pub normal: Pixmap,
    pub dimmed: Pixmap,
    pub spans: Vec<RowSpan>,
}

pub struct ImageCache {
    cache: HashMap<String, CachedCard>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self { cache: HashMap::new() }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn get_or_bake_card(
        &mut self,
        path: &Path,
        card_w: u32,
        card_h: u32,
        lean: f32,
        surface_hex: &str,
        dim_amount: f32,
    ) -> Option<&CachedCard> {
        let key = format!("{}:{}_{}_{:.2}_{}_{:.2}", path.to_string_lossy(), card_w, card_h, lean, surface_hex, dim_amount);
        if !self.cache.contains_key(&key) {
            let card_bbox_w = (card_w as f32 + lean * 2.0).ceil() as u32;
            let mut baked = Pixmap::new(card_bbox_w, card_h)?;

            // 1. Construct local parallelogram path
            let centre_x = card_bbox_w as f32 / 2.0;
            let half_w = card_w as f32 / 2.0;
            let p0 = Point::from_xy(centre_x - half_w + lean, 0.0);
            let p1 = Point::from_xy(centre_x + half_w + lean, 0.0);
            let p2 = Point::from_xy(centre_x + half_w - lean, card_h as f32);
            let p3 = Point::from_xy(centre_x - half_w - lean, card_h as f32);

            let mut pb = PathBuilder::new();
            pb.move_to(p0.x, p0.y);
            pb.line_to(p1.x, p1.y);
            pb.line_to(p2.x, p2.y);
            pb.line_to(p3.x, p3.y);
            pb.close();
            let card_path = pb.finish()?;

            // 2. Default surface background fill
            let mut bg_paint = Paint::default();
            bg_paint.set_color(hex_to_color(surface_hex, 1.0));
            baked.fill_path(&card_path, &bg_paint, FillRule::Winding, Transform::identity(), None);

            // 3. Load image, resize, and draw clipped into card_path with bilinear sampling ONCE
            if let Ok(img) = image::open(path) {
                let img_rgb = img.to_rgba8();
                let (w, h) = (img_rgb.width(), img_rgb.height());

                let scale = (card_bbox_w as f32 / w as f32).max(card_h as f32 / h as f32);
                let new_w = ((w as f32 * scale).round() as u32).max(1);
                let new_h = ((h as f32 * scale).round() as u32).max(1);

                let scaled = image::imageops::resize(&img_rgb, new_w, new_h, image::imageops::FilterType::Triangle);

                if let Some(mut img_pixmap) = Pixmap::new(new_w, new_h) {
                    let pixels = img_pixmap.data_mut();
                    let raw = scaled.as_raw();
                    for (i, chunk) in raw.chunks_exact(4).enumerate() {
                        let r = chunk[0];
                        let g = chunk[1];
                        let b = chunk[2];
                        let a = chunk[3];
                        let p = ColorU8::from_rgba(r, g, b, a).premultiply();
                        pixels[i * 4] = p.red();
                        pixels[i * 4 + 1] = p.green();
                        pixels[i * 4 + 2] = p.blue();
                        pixels[i * 4 + 3] = p.alpha();
                    }

                    let offset_x = (card_bbox_w as f32 - new_w as f32) / 2.0;
                    let offset_y = (card_h as f32 - new_h as f32) / 2.0;

                    let pattern = Pattern::new(
                        img_pixmap.as_ref(),
                        SpreadMode::Pad,
                        FilterQuality::Bilinear,
                        1.0,
                        Transform::from_translate(offset_x, offset_y),
                    );
                    let mut img_paint = Paint::default();
                    img_paint.shader = pattern;

                    baked.fill_path(&card_path, &img_paint, FillRule::Winding, Transform::identity(), None);
                }
            }

            // Precompute row spans for instant SIMD/memcpy blitting
            let sw = card_bbox_w as usize;
            let sh = card_h as usize;
            let src_u32: &[u32] = bytemuck::cast_slice(baked.data());
            let mut spans = Vec::with_capacity(sh);

            for sy in 0..sh {
                let row = &src_u32[sy * sw..(sy + 1) * sw];
                let mut x_start = sw;
                let mut x_solid_start = sw;
                let mut x_solid_end = 0;
                let mut x_end = 0;

                for (x, &p) in row.iter().enumerate() {
                    let a = p >> 24;
                    if a > 0 {
                        if x_start == sw {
                            x_start = x;
                        }
                        x_end = x + 1;
                        if a >= 254 {
                            if x_solid_start == sw {
                                x_solid_start = x;
                            }
                            x_solid_end = x + 1;
                        }
                    }
                }

                if x_start == sw {
                    spans.push(RowSpan { x_start: 0, x_solid_start: 0, x_solid_end: 0, x_end: 0 });
                } else {
                    if x_solid_start == sw {
                        x_solid_start = x_start;
                        x_solid_end = x_end;
                    }
                    spans.push(RowSpan {
                        x_start: x_start as u16,
                        x_solid_start: x_solid_start as u16,
                        x_solid_end: x_solid_end as u16,
                        x_end: x_end as u16,
                    });
                }
            }

            // Pre-bake dimmed card for instant 0ms blits when unselected
            let mut dimmed = baked.clone();
            let dim_mul = ((1.0 - dim_amount.clamp(0.0, 1.0)) * 256.0).round() as u32;
            let dimmed_pixels: &mut [u32] = bytemuck::cast_slice_mut(dimmed.data_mut());
            for p in dimmed_pixels.iter_mut() {
                let sp = *p;
                let sa = sp >> 24;
                if sa == 0 {
                    continue;
                }
                let r = (((sp & 0xFF) * dim_mul) >> 8) & 0xFF;
                let g = ((((sp >> 8) & 0xFF) * dim_mul) >> 8) & 0xFF;
                let b = ((((sp >> 16) & 0xFF) * dim_mul) >> 8) & 0xFF;
                *p = (sa << 24) | (b << 16) | (g << 8) | r;
            }

            self.cache.insert(key.clone(), CachedCard { normal: baked, dimmed, spans });
        }
        self.cache.get(&key)
    }
}

#[inline(always)]
pub fn blit_card_span(
    dest: &mut Pixmap,
    src: &Pixmap,
    spans: &[RowSpan],
    draw_x: i32,
    draw_y: i32,
    dim_shade: f32,
) {
    let dw = dest.width() as i32;
    let dh = dest.height() as i32;
    let sw = src.width() as i32;
    let sh = src.height() as i32;

    let y0 = draw_y.max(0);
    let y1 = (draw_y + sh).min(dh);
    if y0 >= y1 {
        return;
    }

    let src_data: &[u32] = bytemuck::cast_slice(src.data());
    let dst_data: &mut [u32] = bytemuck::cast_slice_mut(dest.data_mut());
    let dst_stride = dw as usize;
    let src_stride = sw as usize;

    let dim = dim_shade.clamp(0.0, 1.0);
    let brightness = ((1.0 - dim) * 256.0).round() as u32;

    for y in y0..y1 {
        let sy = (y - draw_y) as usize;
        let dy = y as usize;
        let span = &spans[sy];
        if span.x_start >= span.x_end {
            continue;
        }

        let sx_start = span.x_start as i32;
        let sx_end = span.x_end as i32;
        let sx_solid_start = span.x_solid_start as i32;
        let sx_solid_end = span.x_solid_end as i32;

        let d_row_start = dy * dst_stride;
        let s_row_start = sy * src_stride;

        // 1. Left antialiased edge
        let left_edge_start = sx_start.max(-draw_x);
        let left_edge_end = sx_solid_start.min(dw - draw_x);
        for sx in left_edge_start..left_edge_end {
            let dx = draw_x + sx;
            if dx >= 0 && dx < dw {
                let sp = src_data[s_row_start + sx as usize];
                let sa = sp >> 24;
                if sa > 0 {
                    let dp = dst_data[d_row_start + dx as usize];
                    let inv = 255 - sa;
                    let rb = (sp & 0x00FF00FF) + (((dp & 0x00FF00FF) * inv + 0x00800080) >> 8);
                    let ag = ((sp >> 8) & 0x00FF00FF) + ((((dp >> 8) & 0x00FF00FF) * inv + 0x00800080) >> 8);
                    dst_data[d_row_start + dx as usize] = (rb & 0x00FF00FF) | ((ag & 0x00FF00FF) << 8);
                }
            }
        }

        // 2. Solid middle span (100% opaque, copied via hardware SIMD / memcpy)
        let solid_start = sx_solid_start.max(-draw_x);
        let solid_end = sx_solid_end.min(dw - draw_x);
        if solid_start < solid_end {
            let s_idx = s_row_start + solid_start as usize;
            let d_idx = d_row_start + (draw_x + solid_start) as usize;
            let count = (solid_end - solid_start) as usize;

            if brightness >= 255 {
                dst_data[d_idx..d_idx + count].copy_from_slice(&src_data[s_idx..s_idx + count]);
            } else {
                let s_slice = &src_data[s_idx..s_idx + count];
                let d_slice = &mut dst_data[d_idx..d_idx + count];
                for (&sp, dp) in s_slice.iter().zip(d_slice.iter_mut()) {
                    let r = (((sp & 0xFF) * brightness) >> 8) & 0xFF;
                    let g = ((((sp >> 8) & 0xFF) * brightness) >> 8) & 0xFF;
                    let b = ((((sp >> 16) & 0xFF) * brightness) >> 8) & 0xFF;
                    *dp = 0xFF000000 | (b << 16) | (g << 8) | r;
                }
            }
        }

        // 3. Right antialiased edge
        let right_edge_start = sx_solid_end.max(-draw_x);
        let right_edge_end = sx_end.min(dw - draw_x);
        for sx in right_edge_start..right_edge_end {
            let dx = draw_x + sx;
            if dx >= 0 && dx < dw {
                let sp = src_data[s_row_start + sx as usize];
                let sa = sp >> 24;
                if sa > 0 {
                    let dp = dst_data[d_row_start + dx as usize];
                    let inv = 255 - sa;
                    let rb = (sp & 0x00FF00FF) + (((dp & 0x00FF00FF) * inv + 0x00800080) >> 8);
                    let ag = ((sp >> 8) & 0x00FF00FF) + ((((dp >> 8) & 0x00FF00FF) * inv + 0x00800080) >> 8);
                    dst_data[d_row_start + dx as usize] = (rb & 0x00FF00FF) | ((ag & 0x00FF00FF) << 8);
                }
            }
        }
    }
}

pub fn render_gallery(
    pixmap: &mut Pixmap,
    fonts: &FontRenderer,
    image_cache: &mut ImageCache,
    config: &Config,
    wallpapers: &[WallpaperItem],
    active_monitor_idx: usize,
    monitor_names: &[String],
    selected_index: usize,
    scroll_offset: f32,
    scroll_velocity: f32,
    hovered_card: Option<usize>,
    show_hidden: bool,
) -> Vec<HitBox> {
    let mut hitboxes = Vec::new();
    let width = pixmap.width() as f32;
    let height = pixmap.height() as f32;

    let style = &config.style;
    let ui = &config.ui;

    // 1. Draw Translucent Backdrop (Rosé Pine Moon base #232136)
    if style.backdrop_opacity > 0.01 {
        let bg_color = hex_to_color(&style.bg_color, style.backdrop_opacity);
        pixmap.fill(bg_color);
    }

    // 2. Draw Top Bar
    if ui.show_top_bar {
        let bar_y = 28.0;
        let mut left_x = 40.0;

        // Monitor Tabs
        if ui.show_monitor_tabs {
            for (m_idx, name) in monitor_names.iter().enumerate() {
                let is_active = m_idx == active_monitor_idx;
                let text = format!("Monitor {}: {}", m_idx, name);
                let text_w = fonts.text_width(&text, 13.0, is_active);
                let btn_w = text_w + 26.0;
                let btn_h = 34.0;

                let rect = Rect::from_xywh(left_x, bar_y, btn_w, btn_h).unwrap();
                let mut pb = PathBuilder::new();
                push_rounded_rect(&mut pb, rect, 8.0);
                let path = pb.finish().unwrap();

                let mut paint = Paint::default();
                if is_active {
                    paint.set_color(hex_to_color(&style.accent_color, 1.0)); // Love #eb6f92
                } else {
                    paint.set_color(hex_to_color(&style.surface_color, 0.95)); // Surface #2a273f
                }
                pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);

                if !is_active {
                    let mut stroke = Stroke::default();
                    stroke.width = 1.0;
                    let mut border_paint = Paint::default();
                    border_paint.set_color(hex_to_color(&style.highlight_color, 0.80));
                    pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
                }

                let text_color = if is_active {
                    Color::WHITE
                } else {
                    hex_to_color(&style.text_color, 0.85)
                };
                fonts.draw_text(pixmap, &text, left_x + 13.0, bar_y + 22.0, 13.0, text_color, is_active);

                hitboxes.push(HitBox::rect(rect, HitAction::SwitchMonitor(m_idx)));

                left_x += btn_w + 12.0;
            }
        }

        // Center Title Pill
        if ui.show_title && !wallpapers.is_empty() && selected_index < wallpapers.len() {
            let wp = &wallpapers[selected_index];
            let is_hidden = config.is_excluded(&format!("Monitor{}", active_monitor_idx), &wp.id);
            let display_title = if is_hidden {
                format!("{}  [HIDDEN]", wp.title)
            } else {
                wp.title.clone()
            };

            let title_w = fonts.text_width(&display_title, 16.0, true);
            let pill_w = title_w + 42.0;
            let pill_x = (width - pill_w) / 2.0;
            let pill_h = 38.0;

            let rect = Rect::from_xywh(pill_x, bar_y - 2.0, pill_w, pill_h).unwrap();
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, rect, 10.0);
            let path = pb.finish().unwrap();

            let mut paint = Paint::default();
            paint.set_color(hex_to_color(&style.surface_color, 0.96));
            pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);

            let mut stroke = Stroke::default();
            stroke.width = 1.5;
            let mut stroke_paint = Paint::default();
            stroke_paint.set_color(hex_to_color(&style.accent_color, 0.85));
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);

            fonts.draw_text(
                pixmap,
                &display_title,
                pill_x + 21.0,
                bar_y + 23.0,
                16.0,
                hex_to_color(&style.text_color, 1.0),
                true,
            );
        }

        // Right side: Close button & Show Hidden toggle
        let mut right_x = width - 40.0;

        if ui.show_close_button {
            right_x -= 34.0;
            let rect = Rect::from_xywh(right_x, bar_y, 34.0, 34.0).unwrap();
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, rect, 8.0);
            let path = pb.finish().unwrap();

            let mut paint = Paint::default();
            paint.set_color(hex_to_color(&style.surface_color, 0.95));
            pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);

            let mut stroke = Stroke::default();
            stroke.width = 1.0;
            let mut stroke_paint = Paint::default();
            stroke_paint.set_color(hex_to_color(&style.highlight_color, 0.80));
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);

            fonts.draw_text(pixmap, "✕", right_x + 10.0, bar_y + 22.0, 14.0, hex_to_color(&style.text_color, 0.90), true);

            hitboxes.push(HitBox::rect(rect, HitAction::Close));
            right_x -= 14.0;
        }

        if ui.show_hidden_button {
            let label = if show_hidden { "Showing Hidden" } else { "Show Hidden" };
            let text_w = fonts.text_width(label, 13.0, show_hidden);
            let btn_w = text_w + 26.0;
            right_x -= btn_w;

            let rect = Rect::from_xywh(right_x, bar_y, btn_w, 34.0).unwrap();
            let mut pb = PathBuilder::new();
            push_rounded_rect(&mut pb, rect, 8.0);
            let path = pb.finish().unwrap();

            let mut paint = Paint::default();
            if show_hidden {
                paint.set_color(hex_to_color(&style.accent_color, 1.0));
            } else {
                paint.set_color(hex_to_color(&style.surface_color, 0.95));
            }
            pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);

            if !show_hidden {
                let mut stroke = Stroke::default();
                stroke.width = 1.0;
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color(hex_to_color(&style.highlight_color, 0.80));
                pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }

            let text_color = if show_hidden { Color::WHITE } else { hex_to_color(&style.text_color, 0.85) };
            fonts.draw_text(pixmap, label, right_x + 13.0, bar_y + 22.0, 13.0, text_color, show_hidden);

            hitboxes.push(HitBox::rect(rect, HitAction::ToggleHidden));
        }
    }

    // 3. Draw Center Strip of Parallelogram Leaning Cards
    let card_w = style.card_width as f32;
    let card_h = style.card_height as f32;
    let half_w = card_w / 2.0;
    let half_h = card_h / 2.0;
    let centre_y = height / 2.0;
    let lean = style.card_shear * half_h;
    let card_bbox_w = (card_w + lean * 2.0).ceil() as u32;

    let count = wallpapers.len();
    if count > 0 {
        let step = card_w; // Edge to edge strip math matching YASB
        let visible_radius = ((width / (2.0 * step)).ceil() as i32) + 2;
        let center_idx = scroll_offset.round() as i32;

        let mut card_spots = Vec::new();
        for offset in -visible_radius..=visible_radius {
            let raw_idx = center_idx + offset;
            let idx = if raw_idx >= 0 {
                (raw_idx as usize) % count
            } else {
                count - ((-raw_idx as usize) % count)
            };
            let idx = idx % count;

            let distance = raw_idx as f32 - scroll_offset;
            let spot_x = distance * step;
            let centre_x = width / 2.0 + spot_x;

            // Check screen bounds
            if centre_x + half_w + lean < 0.0 || centre_x - half_w - lean > width {
                continue;
            }

            card_spots.push((idx, distance, centre_x));
        }

        // Sort by distance from center descending: furthest drawn first, center card drawn last on top!
        card_spots.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap_or(std::cmp::Ordering::Equal));

        struct CardBorderData {
            card_path: tiny_skia::Path,
            centre_x: f32,
            strength: f32,
            is_hovered: bool,
            is_excluded: bool,
        }

        let mut borders_to_render = Vec::new();
        let mut card_hitboxes = Vec::new();

        // PASS 1: Render all card backgrounds, preview images, dims, and tints
        for &(idx, distance, centre_x) in &card_spots {
            let p0 = Point::from_xy(centre_x - half_w + lean, centre_y - half_h);
            let p1 = Point::from_xy(centre_x + half_w + lean, centre_y - half_h);
            let p2 = Point::from_xy(centre_x + half_w - lean, centre_y + half_h);
            let p3 = Point::from_xy(centre_x - half_w - lean, centre_y + half_h);

            let mut pb = PathBuilder::new();
            pb.move_to(p0.x, p0.y);
            pb.line_to(p1.x, p1.y);
            pb.line_to(p2.x, p2.y);
            pb.line_to(p3.x, p3.y);
            pb.close();

            if let Some(card_path) = pb.finish() {
                let focus = (1.0 - distance.abs()).clamp(0.0, 1.0);
                let wp = &wallpapers[idx];
                let is_excluded = config.is_excluded(&format!("Monitor{}", active_monitor_idx), &wp.id);

                let draw_x = (centre_x - card_bbox_w as f32 / 2.0).round() as i32;
                let draw_y = (centre_y - half_h).round() as i32;

                let dim_shade = if ui.dim_unselected && style.dim_unselected > 0.0 {
                    style.dim_unselected * (1.0 - focus)
                } else {
                    0.0
                };

                let mut rendered = false;
                if let Some(ref prev_path) = wp.preview_path {
                    if let Some(card) = image_cache.get_or_bake_card(
                        prev_path,
                        style.card_width,
                        style.card_height,
                        lean,
                        &style.surface_color,
                        style.dim_unselected,
                    ) {
                        if !ui.dim_unselected || style.dim_unselected <= 0.0 {
                            blit_card_span(pixmap, &card.normal, &card.spans, draw_x, draw_y, 0.0);
                        } else if focus <= 0.01 {
                            blit_card_span(pixmap, &card.dimmed, &card.spans, draw_x, draw_y, 0.0);
                        } else if focus >= 0.99 {
                            blit_card_span(pixmap, &card.normal, &card.spans, draw_x, draw_y, 0.0);
                        } else {
                            let dim_shade = style.dim_unselected * (1.0 - focus);
                            blit_card_span(pixmap, &card.normal, &card.spans, draw_x, draw_y, dim_shade);
                        }
                        rendered = true;
                    }
                }
                if !rendered {
                    let mut card_paint = Paint::default();
                    card_paint.set_color(hex_to_color(&style.surface_color, 1.0));
                    pixmap.fill_path(&card_path, &card_paint, FillRule::Winding, Transform::identity(), None);
                    if dim_shade > 0.01 {
                        let mut dim_paint = Paint::default();
                        dim_paint.set_color(Color::from_rgba(0.0, 0.0, 0.0, dim_shade).unwrap());
                        pixmap.fill_path(&card_path, &dim_paint, FillRule::Winding, Transform::identity(), None);
                    }
                }

                // Tint excluded cards
                if is_excluded && ui.tint_excluded {
                    let mut tint_paint = Paint::default();
                    tint_paint.set_color(hex_to_color(&style.accent_color, 0.45));
                    pixmap.fill_path(&card_path, &tint_paint, FillRule::Winding, Transform::identity(), None);
                }

                let is_hovered = Some(idx) == hovered_card;
                let strength = if is_hovered { 1.0 } else { focus.powi(3) };

                if (ui.show_selection_border && strength > 0.01) || (is_excluded && ui.show_hidden_badge && strength > 0.1) {
                    borders_to_render.push(CardBorderData {
                        card_path: card_path.clone(),
                        centre_x,
                        strength,
                        is_hovered,
                        is_excluded,
                    });
                }

                // Hitbox for clicking card (exact parallelogram matching drawn shape)
                card_hitboxes.push(HitBox::parallelogram(
                    centre_x,
                    centre_y,
                    half_w,
                    half_h,
                    style.card_shear,
                    HitAction::SelectCard(idx),
                ));
            }
        }

        // Push card hitboxes in front-to-back order (center card drawn last is on top!)
        hitboxes.extend(card_hitboxes.into_iter().rev());

        // Optional RSMB Directional Motion Blur along scroll velocity
        if style.motion_blur {
            let velocity_px = scroll_velocity.abs() * card_w * style.motion_blur_strength;
            if velocity_px > 1.2 {
                let y0 = (centre_y - half_h).max(0.0) as usize;
                let y1 = (centre_y + half_h).min(height) as usize;
                let w = pixmap.width() as usize;
                apply_rsmb_directional_blur(pixmap, w, y0, y1, velocity_px);
            }
        }

        // PASS 2: Render Badges, Selection Borders, and Glow ABOVE all card images
        // Sort so the hovered card's border is rendered LAST on the very top of everything
        borders_to_render.sort_by_key(|b| if b.is_hovered { 2 } else if b.strength > 0.9 { 1 } else { 0 });

        for b in borders_to_render {
            // [HIDDEN] Badge
            if b.is_excluded && ui.show_hidden_badge && b.strength > 0.1 {
                let badge_w = 90.0;
                let badge_h = 24.0;
                let badge_x = b.centre_x - badge_w / 2.0;
                let badge_y = centre_y + half_h - 38.0;

                let b_rect = Rect::from_xywh(badge_x, badge_y, badge_w, badge_h).unwrap();
                let mut b_pb = PathBuilder::new();
                push_rounded_rect(&mut b_pb, b_rect, 6.0);
                if let Some(b_path) = b_pb.finish() {
                    let mut b_paint = Paint::default();
                    b_paint.set_color(hex_to_color(&style.accent_color, 0.95));
                    pixmap.fill_path(&b_path, &b_paint, FillRule::Winding, Transform::identity(), None);

                    fonts.draw_text(pixmap, "HIDDEN", badge_x + 18.0, badge_y + 16.0, 11.0, Color::WHITE, true);
                }
            }

            // Selection Border & Glow (Accent Color #eb6f92) - renders superior over all images
            if ui.show_selection_border && b.strength > 0.01 {
                // Soft continuous bloom aura around selected card
                if style.border_glow && b.strength > 0.15 {
                    let glow_color = hex_to_color(&style.accent_color, 1.0);
                    let glow_radius = style.glow_radius.clamp(2.0, 20.0);
                    let steps = 4;
                    // Draw from widest spread to tightest with quadratic falloff
                    for step in (1..=steps).rev() {
                        let t = step as f32 / steps as f32;
                        let spread = t * glow_radius;
                        let falloff = (1.0 - t).powi(2);
                        let glow_alpha = 0.07 * falloff * b.strength;

                        if glow_alpha > 0.001 {
                            if let Some(c) = Color::from_rgba(glow_color.red(), glow_color.green(), glow_color.blue(), glow_alpha) {
                                let mut glow_stroke = Stroke::default();
                                glow_stroke.width = style.border_width + spread * 2.0;
                                glow_stroke.line_join = LineJoin::Round;
                                glow_stroke.line_cap = LineCap::Round;
                                let mut glow_paint = Paint::default();
                                glow_paint.set_color(c);
                                pixmap.stroke_path(&b.card_path, &glow_paint, &glow_stroke, Transform::identity(), None);
                            }
                        }
                    }
                }
                let mut stroke = Stroke::default();
                stroke.width = style.border_width;
                stroke.line_join = LineJoin::Round;
                stroke.line_cap = LineCap::Round;
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color(hex_to_color(&style.accent_color, b.strength));
                pixmap.stroke_path(&b.card_path, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
    }

    // 4. Draw Bottom Hint Bar
    if ui.show_hint_bar {
        let kb = &config.keybinds;
        // Title-case each key token for a polished look (e.g. "ctrl+alt+g" → "Ctrl+Alt+G")
        let fmt_key = |s: &str| -> String {
            s.split('+')
                .map(|tok| {
                    let t = tok.trim().to_lowercase();
                    match t.as_str() {
                        "ctrl" => "Ctrl".to_string(),
                        "alt" => "Alt".to_string(),
                        "shift" => "Shift".to_string(),
                        "win" | "super" | "windows" => "Win".to_string(),
                        "hyper" => "Hyper".to_string(),
                        "return" | "enter" => "Enter".to_string(),
                        "escape" | "esc" => "Esc".to_string(),
                        "space" => "Space".to_string(),
                        "tab" => "Tab".to_string(),
                        "left" => "←".to_string(),
                        "right" => "→".to_string(),
                        "up" => "↑".to_string(),
                        "down" => "↓".to_string(),
                        "pageup" | "page_up" | "pgup" => "PgUp".to_string(),
                        "pagedown" | "page_down" | "pgdn" => "PgDn".to_string(),
                        "home" => "Home".to_string(),
                        "end" => "End".to_string(),
                        "delete" | "del" => "Del".to_string(),
                        _ => {
                            let mut chars = t.chars();
                            match chars.next() {
                                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                                None => String::new(),
                            }
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join("+")
        };
        let mut hints = Vec::new();
        for item in &ui.hint_bar_items {
            let item_trimmed = item.trim();
            if item_trimmed.is_empty() {
                continue;
            }
            // Allow custom label syntax: "action:Custom Label" or "key:Custom Label"
            let (key_req, custom_label) = if let Some((k, l)) = item_trimmed.split_once(':') {
                (k.trim().to_lowercase(), Some(l.trim()))
            } else {
                (item_trimmed.to_lowercase(), None)
            };

            let hint_str = match key_req.as_str() {
                "scroll" | "navigate" | "left_right" => {
                    let label = custom_label.unwrap_or("Scroll");
                    format!("{}/{} {}", fmt_key(&kb.left), fmt_key(&kb.right), label)
                }
                "page" | "page_scroll" | "page_left_right" => {
                    let label = custom_label.unwrap_or("Page");
                    format!("{}/{} {}", fmt_key(&kb.page_left), fmt_key(&kb.page_right), label)
                }
                "jump" | "first_last" | "home_end" => {
                    let label = custom_label.unwrap_or("Jump");
                    format!("{}/{} {}", fmt_key(&kb.first), fmt_key(&kb.last), label)
                }
                "apply" | "apply_wallpaper" | "enter" => {
                    let label = custom_label.unwrap_or("Apply");
                    format!("{} {}", fmt_key(&kb.apply_wallpaper), label)
                }
                "all" | "apply_all" | "apply_to_all" => {
                    let label = custom_label.unwrap_or("All");
                    format!("{} {}", fmt_key(&kb.apply_to_all), label)
                }
                "exclude" | "toggle_exclude" => {
                    let label = custom_label.unwrap_or("Exclude");
                    format!("{} {}", fmt_key(&kb.toggle_exclude), label)
                }
                "hidden" | "toggle_hidden" => {
                    let label = custom_label.unwrap_or("Hidden");
                    format!("{} {}", fmt_key(&kb.toggle_hidden), label)
                }
                "monitor" | "switch_monitor" => {
                    let label = custom_label.unwrap_or("Monitor");
                    format!("{} {}", fmt_key(&kb.switch_monitor), label)
                }
                "explorer" | "open_in_explorer" | "folder" => {
                    let label = custom_label.unwrap_or("Explorer");
                    format!("{} {}", fmt_key(&kb.open_in_explorer), label)
                }
                "reload" | "reload_config" => {
                    let label = custom_label.unwrap_or("Reload");
                    format!("{} {}", fmt_key(&kb.reload_config), label)
                }
                "close" | "exit" | "escape" => {
                    let label = custom_label.unwrap_or("Close");
                    format!("{} {}", fmt_key(&kb.close), label)
                }
                "toggle" | "toggle_gallery" | "summon" => {
                    let label = custom_label.unwrap_or("Summon");
                    format!("{} {}", fmt_key(&kb.toggle_gallery), label)
                }
                custom if custom_label.is_some() => {
                    format!("{} {}", fmt_key(custom), custom_label.unwrap())
                }
                _ => continue,
            };
            hints.push(hint_str);
        }

        if hints.is_empty() {
            return hitboxes;
        }

        let hint_text = hints.join("  •  ");

        let hint_w = fonts.text_width(&hint_text, 13.0, false);
        let pill_w = hint_w + 44.0;
        let pill_x = (width - pill_w) / 2.0;
        let pill_y = height - 48.0;
        let pill_h = 32.0;

        let rect = Rect::from_xywh(pill_x, pill_y, pill_w, pill_h).unwrap();
        let mut pb = PathBuilder::new();
        push_rounded_rect(&mut pb, rect, 16.0);
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(hex_to_color(&style.surface_color, 0.95));
            pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);

            let mut stroke = Stroke::default();
            stroke.width = 1.0;
            let mut stroke_paint = Paint::default();
            stroke_paint.set_color(hex_to_color(&style.highlight_color, 0.80));
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);

            fonts.draw_text(
                pixmap,
                &hint_text,
                pill_x + 22.0,
                pill_y + 21.0,
                13.0,
                hex_to_color(&style.muted_color, 0.95),
                false,
            );
        }
    }

    hitboxes
}

pub fn apply_rsmb_directional_blur(
    pixmap: &mut Pixmap,
    w: usize,
    y0: usize,
    y1: usize,
    velocity_px: f32,
) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                apply_rsmb_directional_blur_avx2(pixmap, w, y0, y1, velocity_px);
                return;
            }
        }
    }
    apply_rsmb_directional_blur_scalar(pixmap, w, y0, y1, velocity_px);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn apply_rsmb_directional_blur_avx2(
    pixmap: &mut Pixmap,
    w: usize,
    y0: usize,
    y1: usize,
    velocity_px: f32,
) {
    use std::arch::x86_64::*;

    let streak_len = (velocity_px.abs() * 0.75).clamp(2.0, 24.0);
    let step = (streak_len / 3.0).round().max(1.0) as i32;
    // 7-tap Gaussian shutter: weights sum to 256
    let weights: [i16; 7] = [12, 28, 56, 64, 56, 28, 12];
    let offsets: [i32; 7] = [-3 * step, -2 * step, -step, 0, step, 2 * step, 3 * step];
    let max_offset = (3 * step) as usize;

    let pixels: &mut [u32] = bytemuck::cast_slice_mut(pixmap.data_mut());
    let mut row_buf = vec![0u32; w];

    let w_vecs = [
        _mm256_set1_epi16(weights[0]),
        _mm256_set1_epi16(weights[1]),
        _mm256_set1_epi16(weights[2]),
        _mm256_set1_epi16(weights[3]),
        _mm256_set1_epi16(weights[4]),
        _mm256_set1_epi16(weights[5]),
        _mm256_set1_epi16(weights[6]),
    ];
    let zero = _mm256_setzero_si256();

    for y in y0..y1 {
        let row_start = y * w;
        let row = &pixels[row_start..row_start + w];
        row_buf.copy_from_slice(row);

        let dst = &mut pixels[row_start..row_start + w];

        let safe_start = (max_offset + 7) & !7; // Align to 8 pixels
        let safe_end = (w.saturating_sub(max_offset + 8)) & !7;

        // Hot AVX2 loop: 8 pixels per iteration
        let mut x = safe_start;
        while x < safe_end {
            let mut sum_lo = _mm256_setzero_si256();
            let mut sum_hi = _mm256_setzero_si256();

            for i in 0..7 {
                let tap_ptr = row_buf.as_ptr().offset((x as i32 + offsets[i]) as isize) as *const __m256i;
                let tap_pix = _mm256_loadu_si256(tap_ptr);
                let w_i = w_vecs[i];

                let lo = _mm256_unpacklo_epi8(tap_pix, zero);
                let hi = _mm256_unpackhi_epi8(tap_pix, zero);

                sum_lo = _mm256_add_epi16(sum_lo, _mm256_mullo_epi16(lo, w_i));
                sum_hi = _mm256_add_epi16(sum_hi, _mm256_mullo_epi16(hi, w_i));
            }

            // Divide by 256
            let res_lo = _mm256_srli_epi16(sum_lo, 8);
            let res_hi = _mm256_srli_epi16(sum_hi, 8);

            let packed = _mm256_packus_epi16(res_lo, res_hi);
            // AVX2 lane fixup: packus crosses 128-bit lane order [0, 2, 1, 3] -> permute to [0, 2, 1, 3] inversion = 0xD8
            let ordered = _mm256_permute4x64_epi64(packed, 0xD8);

            _mm256_storeu_si256(dst.as_mut_ptr().add(x) as *mut __m256i, ordered);
            x += 8;
        }

        // Left boundary scalar fallback
        for bx in 0..safe_start {
            let mut sum_rb = 0u32;
            let mut sum_ag = 0u32;
            for i in 0..7 {
                let sx = (bx as i32 + offsets[i]).clamp(0, w as i32 - 1) as usize;
                let sp = row_buf[sx];
                let w_i = weights[i] as u32;
                sum_rb += (sp & 0x00FF00FF) * w_i;
                sum_ag += ((sp >> 8) & 0x00FF00FF) * w_i;
            }
            let rb = (sum_rb >> 8) & 0x00FF00FF;
            let ag = sum_ag & 0xFF00FF00;
            dst[bx] = rb | ag;
        }

        // Right boundary scalar fallback
        for bx in safe_end..w {
            let mut sum_rb = 0u32;
            let mut sum_ag = 0u32;
            for i in 0..7 {
                let sx = (bx as i32 + offsets[i]).clamp(0, w as i32 - 1) as usize;
                let sp = row_buf[sx];
                let w_i = weights[i] as u32;
                sum_rb += (sp & 0x00FF00FF) * w_i;
                sum_ag += ((sp >> 8) & 0x00FF00FF) * w_i;
            }
            let rb = (sum_rb >> 8) & 0x00FF00FF;
            let ag = sum_ag & 0xFF00FF00;
            dst[bx] = rb | ag;
        }
    }
}

fn apply_rsmb_directional_blur_scalar(
    pixmap: &mut Pixmap,
    w: usize,
    y0: usize,
    y1: usize,
    velocity_px: f32,
) {
    let streak_len = (velocity_px.abs() * 0.75).clamp(2.0, 24.0);
    let step = (streak_len / 3.0).round().max(1.0) as i32;
    let weights: [u32; 7] = [12, 28, 56, 64, 56, 28, 12];
    let offsets: [i32; 7] = [-3 * step, -2 * step, -step, 0, step, 2 * step, 3 * step];

    let pixels: &mut [u32] = bytemuck::cast_slice_mut(pixmap.data_mut());
    let mut row_buf = vec![0u32; w];

    for y in y0..y1 {
        let row_start = y * w;
        let row = &pixels[row_start..row_start + w];
        row_buf.copy_from_slice(row);
        let dst = &mut pixels[row_start..row_start + w];

        for bx in 0..w {
            let mut sum_rb = 0u32;
            let mut sum_ag = 0u32;
            for i in 0..7 {
                let sx = (bx as i32 + offsets[i]).clamp(0, w as i32 - 1) as usize;
                let sp = row_buf[sx];
                let w_i = weights[i];
                sum_rb += (sp & 0x00FF00FF) * w_i;
                sum_ag += ((sp >> 8) & 0x00FF00FF) * w_i;
            }
            let rb = (sum_rb >> 8) & 0x00FF00FF;
            let ag = sum_ag & 0xFF00FF00;
            dst[bx] = rb | ag;
        }
    }
}
