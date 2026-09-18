# =============================================================================
# Wallpaper Engine Hotswap - Automated Windows Uninstaller
# =============================================================================
param(
    [switch]$PurgeConfig
)

$ErrorActionPreference = "SilentlyContinue"

$AppName = "Wallpaper Engine Hotswap"
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\wallpaper-engine-hotswap"
$ConfigDir = Join-Path $env:APPDATA "we-gallery"

Write-Host "============================================================" -ForegroundColor Magenta
Write-Host "   Uninstalling $AppName" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Magenta

# 1. Terminate running application
Write-Host "[1/5] Stopping running instances..." -ForegroundColor Yellow
Stop-Process -Name "we-gallery" -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

# 2. Remove Windows Startup entry
Write-Host "[2/5] Removing Startup entry..." -ForegroundColor Yellow
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "WallpaperEngineHotswap" -ErrorAction SilentlyContinue

# 3. Remove Start Menu Shortcut
Write-Host "[3/5] Removing Start Menu shortcuts..." -ForegroundColor Yellow
$StartMenuDir = [System.Environment]::GetFolderPath("Programs")
$ShortcutPath = Join-Path $StartMenuDir "$AppName.lnk"
if (Test-Path $ShortcutPath) {
    Remove-Item -Path $ShortcutPath -Force -ErrorAction SilentlyContinue
}

# 4. Remove Windows Add/Remove Programs Registry Key
Write-Host "[4/5] Removing Windows Installed Apps registry entry..." -ForegroundColor Yellow
$UninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\WallpaperEngineHotswap"
Remove-Item -Path $UninstallKey -Recurse -Force -ErrorAction SilentlyContinue

# 5. Remove Program Files
Write-Host "[5/5] Removing program files..." -ForegroundColor Yellow
if (Test-Path $InstallDir) {
    # If the uninstaller is currently running from inside InstallDir, schedule folder removal
    Start-Process powershell -ArgumentList "-NoProfile", "-WindowStyle", "Hidden", "-Command", "Start-Sleep -Seconds 1; Remove-Item -Path '$InstallDir' -Recurse -Force -ErrorAction SilentlyContinue"
}

if ($PurgeConfig) {
    if (Test-Path $ConfigDir) {
        Remove-Item -Path $ConfigDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "Removed configuration directory: $ConfigDir" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host "   $AppName has been successfully uninstalled." -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
