@echo off
setlocal
cd /d "%~dp0"
echo ========================================================
echo   Wallpaper Engine Hotswap - Installer
echo ========================================================
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" %*
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [!] Installation encountered an issue.
    pause
) else (
    echo.
    pause
)
