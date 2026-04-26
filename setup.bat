@echo off
echo ================================
echo    Newtron Dev Setup
echo ================================

:: Check Node
node -v >nul 2>&1
IF %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Node.js not found. Install it from https://nodejs.org then rerun this.
    pause
    exit /b 1
)
echo [OK] Node.js found

:: Check Rust
cargo -v >nul 2>&1
IF %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Rust not found. Install it from https://rustup.rs then rerun this.
    pause
    exit /b 1
)
echo [OK] Rust found

:: Check Tauri CLI
cargo tauri -V >nul 2>&1
IF %ERRORLEVEL% NEQ 0 (
    echo [INSTALLING] Tauri CLI...
    cargo install tauri-cli
)
echo [OK] Tauri CLI found

:: Install/update Node dependencies
echo [INSTALLING] Node dependencies...
npm install

:: Run the app
echo ================================
echo    Launching Newtron...
echo ================================
npm run tauri dev
