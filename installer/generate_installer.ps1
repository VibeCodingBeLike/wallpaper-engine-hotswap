# =============================================================================
# Wallpaper Engine Hotswap - Build & Package Generator
# =============================================================================
param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$AppName = "Wallpaper Engine Hotswap"
$AppVersion = "0.2.0"
$ExeName = "we-gallery.exe"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoDir = Split-Path -Parent $ScriptDir
$DistDir = Join-Path $RepoDir "dist"

Write-Host "============================================================" -ForegroundColor Magenta
Write-Host "   Generating Installer & Packages for $AppName v$AppVersion" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Magenta

# 1. Build optimized release executable
if (-not $SkipBuild) {
    Write-Host "[1/4] Compiling release binary with cargo..." -ForegroundColor Yellow
    Push-Location $RepoDir
    try {
        cargo build --release
    } finally {
        Pop-Location
    }
}

$ReleaseExe = Join-Path $RepoDir "target\release\$ExeName"
if (-not (Test-Path $ReleaseExe)) {
    Write-Error "Release executable not found at: $ReleaseExe"
    exit 1
}

Write-Host "Release binary verified: $ReleaseExe" -ForegroundColor Green

# 2. Create / clean dist directory
Write-Host "[2/4] Preparing output directory: $DistDir..." -ForegroundColor Yellow
if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
}

# 3. Check for Inno Setup (ISCC) to build standalone Setup.exe
Write-Host "[3/4] Checking for Inno Setup compiler (ISCC)..." -ForegroundColor Yellow
$IsccPath = $null
$PossibleIscc = @(
    "iscc",
    "C:\Program Files (x86)\Inno Setup 6\iscc.exe",
    "C:\Program Files\Inno Setup 6\iscc.exe",
    "C:\Program Files (x86)\Inno Setup 5\iscc.exe"
)

foreach ($cand in $PossibleIscc) {
    if (Get-Command $cand -ErrorAction SilentlyContinue) {
        $IsccPath = $cand
        break
    } elseif (Test-Path $cand) {
        $IsccPath = $cand
        break
    }
}

if ($IsccPath) {
    Write-Host "Compiling standalone Setup.exe with Inno Setup: $IsccPath" -ForegroundColor Green
    & $IsccPath (Join-Path $ScriptDir "installer.iss")
} else {
    Write-Host "[i] Inno Setup compiler not found. Skipping Setup.exe compilation." -ForegroundColor Gray
    Write-Host "    (Install Inno Setup 6 if you wish to generate a single-file Setup.exe wizard)" -ForegroundColor Gray
}

# 4. Create Portable Distribution Zip Package
Write-Host "[4/4] Creating portable distribution package..." -ForegroundColor Yellow
$PortableStage = Join-Path $DistDir "wallpaper-engine-hotswap-v$AppVersion-portable"
if (Test-Path $PortableStage) {
    Remove-Item -Path $PortableStage -Recurse -Force
}
New-Item -ItemType Directory -Path $PortableStage -Force | Out-Null

Copy-Item -Path $ReleaseExe -Destination (Join-Path $PortableStage $ExeName)
Copy-Item -Path (Join-Path $ScriptDir "install.cmd") -Destination (Join-Path $PortableStage "install.cmd")
Copy-Item -Path (Join-Path $ScriptDir "uninstall.cmd") -Destination (Join-Path $PortableStage "uninstall.cmd")
Copy-Item -Path (Join-Path $ScriptDir "install.ps1") -Destination (Join-Path $PortableStage "install.ps1")
Copy-Item -Path (Join-Path $ScriptDir "uninstall.ps1") -Destination (Join-Path $PortableStage "uninstall.ps1")
Copy-Item -Path (Join-Path $RepoDir "config.example.toml") -Destination (Join-Path $PortableStage "config.example.toml")
Copy-Item -Path (Join-Path $RepoDir "README.md") -Destination (Join-Path $PortableStage "README.md")
Copy-Item -Path (Join-Path $RepoDir "LICENSE") -Destination (Join-Path $PortableStage "LICENSE")

$ZipPath = Join-Path $DistDir "wallpaper-engine-hotswap-v$AppVersion-windows-x64.zip"
if (Test-Path $ZipPath) {
    Remove-Item -Path $ZipPath -Force
}
Compress-Archive -Path "$PortableStage\*" -DestinationPath $ZipPath
Remove-Item -Path $PortableStage -Recurse -Force

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host "   Distribution packages generated successfully!" -ForegroundColor Green
Write-Host "   Output Directory: $DistDir" -ForegroundColor White
Get-ChildItem -Path $DistDir | ForEach-Object {
    Write-Host "   -> $($_.Name) ($([math]::Round($_.Length / 1MB, 2)) MB)" -ForegroundColor Cyan
}
Write-Host "============================================================" -ForegroundColor Green
