@echo off
setlocal EnableExtensions
REM =====================================================================
REM  TSideload first-time installer for Windows
REM  IMPORTANT: keep this file ASCII-only (cmd.exe mangles other encodings)
REM =====================================================================

set "APP=TSideload"
set "BIN_DIR=%LOCALAPPDATA%\TSideload\bin"
set "EXE=%BIN_DIR%\tsd.exe"
REM TODO(P1): CI publishes this asset on every release. Until then, build
REM tsd from source:  cd apps\desktop  ^&^& cargo build --release -p tsd
set "DL_URL=https://github.com/goddie9x/TSideload/releases/latest/download/tsd-x86_64-pc-windows-msvc.zip"

echo.
echo  === %APP% first-time setup ===
echo.

REM ---- 1. Apple mobile-device driver -----------------------------------
echo [1/4] Checking Apple mobile-device driver...
set "HAS_APPLE=0"
if exist "%ProgramFiles%\Apple Devices\AppleDevices.exe" set "HAS_APPLE=1"
if exist "%ProgramFiles(x86)%\iTunes\iTunes.exe" set "HAS_APPLE=1"
if exist "%ProgramFiles%\iTunes\iTunes.exe" set "HAS_APPLE=1"

if "%HAS_APPLE%"=="1" (
    echo       OK: Apple Devices / iTunes found.
) else (
    echo       NOT FOUND.
    echo.
    echo       TSideload needs the Apple mobile-device driver. Install
    echo       "Apple Devices" (or iTunes) FROM APPLE'S WEBSITE:
    echo         https://www.apple.com/apple-devices/
    echo         https://www.apple.com/itunes/
    echo       The Microsoft Store versions do NOT work with third-party tools.
    echo.
    choice /M "Open the Apple Devices download page now"
    if not errorlevel 2 start "" "https://www.apple.com/apple-devices/"
    echo       Re-run this script after installing, then continue.
    pause
    exit /b 1
)

REM ---- 2. Download tsd --------------------------------------------------
echo [2/4] Downloading tsd...
if not exist "%BIN_DIR%" mkdir "%BIN_DIR%" 2>nul
if exist "%EXE%" (
    echo       tsd already present at %EXE% - skipping download.
) else (
    echo       Downloading %DL_URL%
    powershell -NoProfile -Command "try { Invoke-WebRequest -Uri '%DL_URL%' -OutFile '%TEMP%\tsd.zip' } catch { exit 1 }"
    if errorlevel 1 (
        echo       Download failed. The release asset may not exist yet ^(P1^).
        echo       Build from source instead:
        echo         cd apps\desktop ^&^& cargo build --release -p tsd
        echo       then copy target\release\tsd.exe to %BIN_DIR%
        pause
        exit /b 1
    )
    powershell -NoProfile -Command "Expand-Archive -Path '%TEMP%\tsd.zip' -DestinationPath '%BIN_DIR%' -Force"
    del "%TEMP%\tsd.zip" 2>nul
    echo       Installed to %EXE%
)

REM ---- 3. PATH -----------------------------------------------------------
echo [3/4] Adding to user PATH...
powershell -NoProfile -Command "$p=[Environment]::GetEnvironmentVariable('Path','User'); if ($p -notlike '*TSideload\bin*') { [Environment]::SetEnvironmentVariable('Path', $p + ';%BIN_DIR%', 'User'); Write-Host '      Added.' } else { Write-Host '      Already on PATH.' }"
set "PATH=%BIN_DIR%;%PATH%"

REM ---- 4. Smoke test ------------------------------------------------------
echo [4/4] Smoke test - connect your iPhone via USB now.
echo       On the iPhone, tap "Trust" when asked.
echo.
pause
"%EXE%" devices
echo.
echo  Done. Try:  tsd install C:\path\to\app.ipa --apple-id you@example.com
echo  (Open a NEW terminal so the updated PATH applies.)
pause
endlocal
