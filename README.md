# Wallpaper Engine Hotswap 🌸

[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows_10%20%2F%2011-0078D6.svg?style=flat-square&logo=windows)](https://microsoft.com/windows)
[![Refresh Rate](https://img.shields.io/badge/Performance-240Hz%2B%20V--Sync-eb6f92.svg?style=flat-square)](#performance--architecture)
[![SIMD](https://img.shields.io/badge/SIMD-AVX2%20Vectorized-9ccfd8.svg?style=flat-square)](#performance--architecture)
[![Theme](https://img.shields.io/badge/Theme-Ros%C3%A9%20Pine%20Moon-ea9a97.svg?style=flat-square)](https://rosepinetheme.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-f6c177.svg?style=flat-square)](LICENSE)

A ultra-fast, native Rust stand-alone Wallpaper Engine gallery and quick-switcher for Windows, styled in the **Rosé Pine Moon** palette.

Inspired by YASB's `wallpaperswidget`, but completely decoupled: zero Python runtime dependencies, instant global hotkey response, 0% idle CPU usage, and custom-tailored for high refresh rate gaming monitors (**240Hz / 360Hz**).

---

## ✨ Features

- **🚀 Instant Global Hotkey**: Summon and dismiss anywhere in Windows via `Ctrl + Alt + G` (customizable).
- **🏎️ 240Hz+ Ultra-Low Frame Time**: Custom SIMD software rasterizer achieving **~1.9 ms static frames (>500 FPS)** and **~2.7 ms scroll frames (>370 FPS)**.
- **🎬 ReelSmart-Style Motion Blur (RSMB)**: Velocity-adaptive 7-tap directional shutter blur vectorized via **AVX2 SIMD** (<0.8ms). Fast flicks streak authentically while slow scrolls stay razor-sharp.
- **🖥️ Multi-Monitor EDID Detection**: Reads raw Windows GDI & EDID hardware descriptors to show real monitor names (e.g. `LG ULTRAGEAR+`, `GS27QA`) instead of generic device paths.
- **🎯 Pixel-Perfect Parallelogram Hitboxes**: Exact mathematical point-in-polygon hit testing matching the leaning card shear angle—zero misclicks or overlapping edge conflicts.
- **🪟 Real Wallpaper Engine IPC**: Direct Win32 IPC communication with `wallpaper32.exe` / `wallpaper64.exe` to inspect active wallpapers per-monitor and apply wallpapers with zero lag.
- **🚫 Per-Monitor Exclusion System**: Press `x` to exclude/hide wallpapers per monitor. Press `h` to view excluded wallpapers with a distinct accent tint and `[HIDDEN]` badge.
- **🌸 Rosé Pine Moon Aesthetic**: Parallelogram leaning cards, soft bloom aura glow, customizable card shear slope, translucent backdrops, and smooth 100ms fade-out window transitions.
- **🌏 Full CJK & Emoji Unicode Rendering**: Comprehensive font fallback hierarchy supporting English, Japanese (Kanji/Kana), Chinese, Korean, symbols, and emojis.
- **🔋 Zero Background Resource Usage**: Pauses CPU completely via OS event pump when hidden (0% CPU / 0% GPU).

---

## 🎮 Keyboard Controls & Navigation

| Key | Action |
| :--- | :--- |
| `Ctrl + Alt + G` | Toggle gallery window (Global hotkey from any application) |
| `Left` / `Right` | Scroll by 1 card |
| `PageUp` / `PageDown` | Fast scroll by 5 cards |
| `Home` / `End` | Jump to first / last wallpaper |
| `Return` (Enter) | Apply selected wallpaper to active monitor |
| `Ctrl + Return` | Apply selected wallpaper across **all** connected monitors |
| `Tab` | Switch active monitor tab |
| `x` | Toggle exclude/hide status for selected wallpaper on current monitor |
| `h` | Toggle display of hidden/excluded wallpapers |
| `e` | Open selected wallpaper directory in Windows Explorer |
| `Escape` | Smooth fade-out and close |
| **Mouse Wheel** | Fluid momentum scroll |
| **Left Click** | Select card / switch monitor / toggle buttons |
| **Double Click** | Apply wallpaper & close |

---

## ⚡ Performance & Architecture

Standard software renderers struggle to hit 240 FPS at 1440p / 4K. `wallpaper-engine-hotswap` was engineered from scratch with a custom rendering pipeline:

```
[ Input / Mouse Wheel ]
           │
           ▼
[ Frame-rate Independent Exponential Velocity Decay ]
           │
           ▼
[ Pass 1: SIMD RowSpan Blitting (Pre-baked Cards) ]  ───► 0.77 ms
           │
           ▼
[ Pass 1.5: RSMB AVX2 7-Tap Directional Blur ]       ───► 0.80 ms
           │
           ▼
[ Pass 2: Superior Glow Aura & Selection Outlines ]  ───► 0.35 ms
           │
           ▼
[ Softbuffer Vectorized AVX2 Presentation (vpshufb) ] ───► 0.15 ms
                                                   ──────────────
                                              Total: ~2.07 ms (480+ FPS)
```

- **RowSpan Memcpy Blitting**: Bypasses scanline rasterization by caching pre-computed spans and copying solid pixels directly via hardware SIMD memcpy.
- **AVX2 Directional RSMB**: Convolves 8 RGBA pixels concurrently per 256-bit vector with 7-tap shutter weights (`[12, 28, 56, 64, 56, 28, 12]`), avoiding scalar division or border smearing.
- **Zero Heap Allocations in Render Loop**: Pixmaps and frame buffers are retained across frames, eliminating OS heap churn.

---

## 📦 Installation & Setup

### Option 1: Automated Installer (Recommended)
1. Download the latest release from [Releases](https://github.com/Kat/wallpaper-engine-hotswap/releases).
2. Extract the archive and double-click **`install.cmd`** (or run `powershell -ExecutionPolicy Bypass -File installer/install.ps1`).
3. The installer will:
   - Install to `%LOCALAPPDATA%\Programs\wallpaper-engine-hotswap` (no Administrator rights required).
   - Create a Windows Start Menu shortcut.
   - Register in Windows **Settings > Installed Apps (Add or Remove Programs)** with a one-click uninstaller.
   - Set up automatic launch on Windows login (optional).

### Option 2: Portable Usage
If you prefer running without installing:
1. Double-click **`start.bat`**.
2. Press `Ctrl + Alt + G` to summon the gallery whenever you want!

### Uninstallation
- Open Windows **Settings > Apps > Installed Apps**, search for `Wallpaper Engine Hotswap`, and click **Uninstall**.
- Alternatively, run **`uninstall.cmd`** in the program folder.

---

## 🛠️ Building From Source

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.75+ recommended)
- Windows 10 or 11 (64-bit)
- Optional: [Inno Setup 6](https://jrsoftware.org/isdl.php) (if building single-file setup wizard `.exe`)

### Build Steps
```powershell
# Clone repository
git clone https://github.com/Kat/wallpaper-engine-hotswap.git
cd wallpaper-engine-hotswap

# Build optimized release binary
cargo build --release

# Run
.\target\release\we-gallery.exe
```

### Generate Installer & Packages
To generate a distributable zip and standalone Setup wizard:
```powershell
.\installer\generate_installer.bat
```
Output packages will be generated in the `dist/` directory.

---

## ⚙️ Configuration Reference

Configuration is stored in **`%APPDATA%\we-gallery\config.toml`**. A documented template is provided in [`config.example.toml`](config.example.toml).

```toml
[theme]
preset = "rose-pine-moon"

[style]
accent_color = "#eb6f92"       # Rosé Pine Love
bg_color = "#232136"           # Base backdrop
surface_color = "#2a273f"      # Card surface
card_width = 300               # Width of each card in pixels
card_height = 533              # Height of each card in pixels
card_shear = 0.21              # Shear slope for leaning parallelogram cards
border_glow = true             # Bloom aura around selected card
glow_radius = 8.0              # Glow radius in pixels
motion_blur = true             # ReelSmart-style directional motion blur
motion_blur_strength = 1.0     # Streak multiplier

[keybinds]
toggle_gallery = "ctrl+alt+g"  # Global summon hotkey
apply_wallpaper = "return"
apply_to_all = "ctrl+return"
close = "escape"
```

---

## 📄 License

Distributed under the **MIT License**. See [`LICENSE`](LICENSE) for details.
