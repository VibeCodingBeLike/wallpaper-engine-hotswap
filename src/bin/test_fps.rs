use std::time::Instant;

fn main() {
    println!("Testing DwmFlush refresh rate on your system...");
    let start = Instant::now();
    let frames = 120;
    for _ in 0..frames {
        unsafe {
            windows_sys::Win32::Graphics::Dwm::DwmFlush();
        }
    }
    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    let fps = frames as f64 / secs;
    println!("Elapsed for {} DwmFlush calls: {:.2} ms", frames, elapsed.as_secs_f64() * 1000.0);
    println!("Measured DWM Refresh Rate: {:.1} Hz (FPS)", fps);
}
