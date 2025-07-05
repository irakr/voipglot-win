@echo off
echo ========================================
echo VoipGlot Windows Build Script
echo ========================================

REM Check if Rust is installed
rustc --version >nul 2>&1
if errorlevel 1 (
    echo Error: Rust is not installed or not in PATH
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo Rust is installed. Checking toolchain...

REM Check if the required target is installed
rustup target list --installed | findstr "x86_64-pc-windows-msvc" >nul
if errorlevel 1 (
    echo Installing Windows target...
    rustup target add x86_64-pc-windows-msvc
    if errorlevel 1 (
        echo Error: Failed to install Windows target
        pause
        exit /b 1
    )
)

REM Check if required components are installed
rustup component list --installed | findstr "rustfmt" >nul
if errorlevel 1 (
    echo Installing rustfmt...
    rustup component add rustfmt
)

rustup component list --installed | findstr "clippy" >nul
if errorlevel 1 (
    echo Installing clippy...
    rustup component add clippy
)

echo.
echo ========================================
echo Building VoipGlot for Windows...
echo ========================================

REM Clean previous builds
echo Cleaning previous builds...
cargo clean

REM Run clippy for code quality checks
echo Running clippy...
cargo clippy --release
if errorlevel 1 (
    echo Warning: Clippy found issues, but continuing with build...
)

REM Build the release version
echo Building release version...
cargo build --release --target x86_64-pc-windows-msvc
if errorlevel 1 (
    echo Error: Build failed!
    pause
    exit /b 1
)

echo.
echo ========================================
echo Build completed successfully!
echo ========================================
echo.
echo Executable location: target\x86_64-pc-windows-msvc\release\voipglot-win.exe
echo.
echo To run the application:
echo   target\x86_64-pc-windows-msvc\release\voipglot-win.exe
echo.
echo Remember to:
echo 1. Install VB-CABLE Virtual Audio Device
echo 2. Set up your API keys in environment variables
echo 3. Configure config.toml if needed
echo.