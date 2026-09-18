#[path = "../config.rs"]
mod config;
#[path = "../scanner.rs"]
mod scanner;
#[path = "../render.rs"]
mod render;

use config::Config;
use render::{FontRenderer, ImageCache, render_gallery, apply_rsmb_directional_blur};
use scanner::scan_wallpapers;
use tiny_skia::Pixmap;

fn main() {
    let config = Config::load();
    let wallpapers = scan_wallpapers();
    println!("Loaded {} wallpapers", wallpapers.len());

    let width = 2560;
    let height = 1440;
    let mut pixmap = Pixmap::new(width, height).unwrap();

    let fonts = FontRenderer::new();
    let mut image_cache = ImageCache::new();
    let monitor_names = vec!["LG ULTRAGEAR+".to_string(), "GS27QA".to_string()];

    // Single frame detailed breakdown
    let t_start = std::time::Instant::now();
    let _ = render_gallery(
        &mut pixmap,
        &fonts,
        &mut image_cache,
        &config,
        &wallpapers,
        0,
        &monitor_names,
        0,
        0.0,
        0.0,
        None,
        false,
    );
    println!("First frame (with caching/baking): {:.2} ms", t_start.elapsed().as_secs_f64() * 1000.0);

    // Frame 2
    let _ = render_gallery(&mut pixmap, &fonts, &mut image_cache, &config, &wallpapers, 0, &monitor_names, 0, 0.5, 0.05, None, false);

    // Frame 3 (all cards already baked in image_cache)
    let t_warm = std::time::Instant::now();
    let _ = render_gallery(&mut pixmap, &fonts, &mut image_cache, &config, &wallpapers, 0, &monitor_names, 0, 0.5, 0.0, None, false);
    println!("Warm cached frame time (static, 0 blur): {:.2} ms", t_warm.elapsed().as_secs_f64() * 1000.0);

    // Frame 4 (scrolling test with RSMB motion blur)
    let t_scroll = std::time::Instant::now();
    let _ = render_gallery(&mut pixmap, &fonts, &mut image_cache, &config, &wallpapers, 0, &monitor_names, 0, 0.52, 0.0, None, false);

    // Apply RSMB 9-tap directional shutter blur on the card strip
    let t_rsmb = std::time::Instant::now();
    let y0 = 430;
    let y1 = 1010;
    let w = pixmap.width() as usize;
    let velocity_px = 18.0f32; // Simulating fast scroll
    apply_rsmb_directional_blur(&mut pixmap, w, y0, y1, velocity_px);
    println!("RSMB 9-tap Directional Motion Blur time: {:.2} ms", t_rsmb.elapsed().as_secs_f64() * 1000.0);

    pixmap.save_png("test_output.png").unwrap();
}

