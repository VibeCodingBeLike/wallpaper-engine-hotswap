@echo off
setlocal
cd /d "%~dp0"
echo ========================================================
echo   Wallpaper Engine Hotswap - Uninstaller
echo ========================================================
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0uninstall.ps1" %*
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [!] Uninstallation encountered an issue.
    pause
) else (
    echo.
    pause
)
