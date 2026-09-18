#[path = "../scanner.rs"]
mod scanner;

fn main() {
    let items = scanner::scan_wallpapers(true);
    println!("Found {} items", items.len());
    for item in items.iter().take(10) {
        if let Some(ref p) = item.preview_path {
            let t0 = std::time::Instant::now();
            match image::open(p) {
                Ok(img) => println!("Loaded '{}' ({}x{}) in {:?}", item.title, img.width(), img.height(), t0.elapsed()),
                Err(e) => println!("Failed '{}': {}", item.title, e),
            }
        }
    }
}
