@echo off
setlocal
cd /d "%~dp0"
echo ========================================================
echo   Wallpaper Engine Hotswap - Package Generator
echo ========================================================
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0generate_installer.ps1" %*
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [!] Package generation encountered an issue.
    pause
) else (
    echo.
    pause
)
