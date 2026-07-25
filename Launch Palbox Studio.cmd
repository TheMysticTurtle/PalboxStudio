@echo off
setlocal
cd /d "%~dp0"
title Palbox Studio

where npm.cmd >nul 2>&1
if errorlevel 1 goto missing_node

where cargo.exe >nul 2>&1
if errorlevel 1 goto missing_rust

if not exist "node_modules\@tauri-apps\cli\package.json" goto missing_root_dependencies
if not exist "ui\node_modules\vite\package.json" goto missing_ui_dependencies
if not exist "data\palbox-reference.db" goto missing_reference_database

if /i "%~1"=="--check" (
  echo Palbox Studio launcher check passed.
  exit /b 0
)

echo Starting Palbox Studio...
echo Keep this window open while using the development build.
echo.
call npm.cmd run tauri -- dev
set "launcher_exit=%errorlevel%"

if not "%launcher_exit%"=="0" (
  echo.
  echo Palbox Studio stopped with exit code %launcher_exit%.
  pause
)
endlocal & exit /b %launcher_exit%

:missing_node
echo Palbox Studio needs Node.js and npm on PATH.
echo Install Node.js, then reopen this launcher.
goto launcher_error

:missing_rust
echo Palbox Studio needs the Rust toolchain and cargo on PATH.
echo Install Rust with the stable MSVC toolchain, then reopen this launcher.
goto launcher_error

:missing_root_dependencies
echo Palbox Studio's root Node dependencies are missing.
echo Run: npm install
goto launcher_error

:missing_ui_dependencies
echo Palbox Studio's UI dependencies are missing.
echo Run: npm --prefix ui install
goto launcher_error

:missing_reference_database
echo Palbox Studio's reference database is missing:
echo   data\palbox-reference.db
goto launcher_error

:launcher_error
echo.
pause
exit /b 1
