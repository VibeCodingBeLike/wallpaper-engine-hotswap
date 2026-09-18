# =============================================================================
# Wallpaper Engine Hotswap - Automated Windows Installer
# =============================================================================
param(
    [switch]$NoStartup,
    [switch]$NoLaunch
)

$ErrorActionPreference = "Stop"

$AppName = "Wallpaper Engine Hotswap"
$AppVersion = "0.2.0"
$ExeName = "we-gallery.exe"
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\wallpaper-engine-hotswap"
$ConfigDir = if ($env:XDG_CONFIG_HOME) { Join-Path $env:XDG_CONFIG_HOME "we-gallery" } else { Join-Path $HOME ".config\we-gallery" }

Write-Host "============================================================" -ForegroundColor Magenta
Write-Host "   Installing $AppName v$AppVersion" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Magenta

# 1. Stop any currently running instance
Write-Host "[1/6] Stopping existing $AppName processes..." -ForegroundColor Yellow
Stop-Process -Name "we-gallery" -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

# 2. Locate source executable
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoDir = Split-Path -Parent $ScriptDir

$SourceExe = $null
$PossibleExeLocations = @(
    (Join-Path $ScriptDir $ExeName),
    (Join-Path $RepoDir $ExeName),
    (Join-Path $RepoDir "target\release\$ExeName"),
    (Join-Path (Get-Location) $ExeName),
    (Join-Path (Get-Location) "target\release\$ExeName")
)

foreach ($loc in $PossibleExeLocations) {
    if (Test-Path $loc) {
        $SourceExe = (Resolve-Path $loc).Path
        break
    }
}

if (-not $SourceExe) {
    Write-Host "[!] $ExeName not found. Attempting to build with cargo..." -ForegroundColor Yellow
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Push-Location $RepoDir
        cargo build --release
        Pop-Location
        $Candidate = Join-Path $RepoDir "target\release\$ExeName"
        if (Test-Path $Candidate) {
            $SourceExe = $Candidate
        }
    }
}

if (-not $SourceExe -or -not (Test-Path $SourceExe)) {
    Write-Error "Could not find or build $ExeName. Please run 'cargo build --release' first."
    exit 1
}

Write-Host "Found binary: $SourceExe" -ForegroundColor Green

# 3. Create destination directory & copy files
Write-Host "[2/6] Installing files to $InstallDir..." -ForegroundColor Yellow
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Copy-Item -Path $SourceExe -Destination (Join-Path $InstallDir $ExeName) -Force

# Copy uninstall scripts to install dir
if (Test-Path (Join-Path $ScriptDir "uninstall.ps1")) {
    Copy-Item -Path (Join-Path $ScriptDir "uninstall.ps1") -Destination (Join-Path $InstallDir "uninstall.ps1") -Force
}
if (Test-Path (Join-Path $ScriptDir "uninstall.cmd")) {
    Copy-Item -Path (Join-Path $ScriptDir "uninstall.cmd") -Destination (Join-Path $InstallDir "uninstall.cmd") -Force
}

# 4. Initialize configuration if not already present
Write-Host "[3/6] Configuring application settings..." -ForegroundColor Yellow
if (-not (Test-Path $ConfigDir)) {
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
}
$TargetConfig = Join-Path $ConfigDir "config.toml"
if (-not (Test-Path $TargetConfig)) {
    $ExampleConfig = Join-Path $RepoDir "config.example.toml"
    if (Test-Path $ExampleConfig) {
        Copy-Item -Path $ExampleConfig -Destination $TargetConfig -Force
        Write-Host "Created default config at: $TargetConfig" -ForegroundColor Green
    }
}

# 5. Create Start Menu Shortcut
Write-Host "[4/6] Creating Start Menu shortcuts..." -ForegroundColor Yellow
$StartMenuDir = [System.Environment]::GetFolderPath("Programs")
$ShortcutPath = Join-Path $StartMenuDir "$AppName.lnk"

$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = Join-Path $InstallDir $ExeName
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Description = "Wallpaper Engine Fast Gallery & Switcher"
$Shortcut.IconLocation = (Join-Path $InstallDir $ExeName) + ",0"
$Shortcut.Save()

# 6. Windows Startup entry
if (-not $NoStartup) {
    Write-Host "[5/6] Registering Windows Startup..." -ForegroundColor Yellow
    $RunKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
    Set-ItemProperty -Path $RunKey -Name "WallpaperEngineHotswap" -Value ("`"" + (Join-Path $InstallDir $ExeName) + "`"")
}

# 7. Register Windows Add/Remove Programs entry
Write-Host "[6/6] Registering with Windows Installed Apps (Add/Remove Programs)..." -ForegroundColor Yellow
$UninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\WallpaperEngineHotswap"
if (-not (Test-Path $UninstallKey)) {
    New-Item -Path $UninstallKey -Force | Out-Null
}

$InstalledExe = Join-Path $InstallDir $ExeName
$InstalledUninstallCmd = Join-Path $InstallDir "uninstall.cmd"

Set-ItemProperty -Path $UninstallKey -Name "DisplayName" -Value $AppName
Set-ItemProperty -Path $UninstallKey -Name "DisplayVersion" -Value $AppVersion
Set-ItemProperty -Path $UninstallKey -Name "Publisher" -Value "Kat"
Set-ItemProperty -Path $UninstallKey -Name "InstallLocation" -Value $InstallDir
Set-ItemProperty -Path $UninstallKey -Name "DisplayIcon" -Value "$InstalledExe,0"
Set-ItemProperty -Path $UninstallKey -Name "UninstallString" -Value "`"$InstalledUninstallCmd`""
Set-ItemProperty -Path $UninstallKey -Name "NoModify" -Value 1 -Type DWord
Set-ItemProperty -Path $UninstallKey -Name "NoRepair" -Value 1 -Type DWord

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host "   $AppName v$AppVersion installed successfully!" -ForegroundColor Green
Write-Host "   Installed to: $InstallDir" -ForegroundColor White
Write-Host "   Hotkey: Ctrl + Alt + G" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Green

if (-not $NoLaunch) {
    Write-Host "Starting $AppName..." -ForegroundColor Cyan
    Start-Process -FilePath $InstalledExe -WorkingDirectory $InstallDir
}
