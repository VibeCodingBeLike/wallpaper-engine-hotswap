@echo off
setlocal
cd /d "%~dp0"
taskkill /F /IM we-gallery.exe >nul 2>&1
if not exist "we-gallery.exe" (
    if exist "target\release\we-gallery.exe" (
        copy /y "target\release\we-gallery.exe" "we-gallery.exe" >nul
    ) else (
        echo [!] Compiling release binary...
        cargo build --release
        copy /y "target\release\we-gallery.exe" "we-gallery.exe" >nul
    )
)
start "" "we-gallery.exe"
