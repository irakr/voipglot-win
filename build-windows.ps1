# VoipGlot Windows Build Script (PowerShell)
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "VoipGlot Windows Build Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Check if Rust is installed
try {
    $rustVersion = rustc --version 2>$null
    if ($LASTEXITCODE -ne 0) {
        throw "Rust not found"
    }
    Write-Host "Rust is installed: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "Error: Rust is not installed or not in PATH" -ForegroundColor Red
    Write-Host "Please install Rust from https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host "Checking toolchain..." -ForegroundColor Yellow

# Check if the required target is installed (only if not already installed)
$installedTargets = rustup target list --installed 2>$null
if ($installedTargets -notcontains "x86_64-pc-windows-msvc") {
    Write-Host "Installing Windows target..." -ForegroundColor Yellow
    rustup target add x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Failed to install Windows target" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "Windows target already installed" -ForegroundColor Green
}

# Check if required components are installed (only if not already installed)
$installedComponents = rustup component list --installed 2>$null
if ($installedComponents -notcontains "rustfmt") {
    Write-Host "Installing rustfmt..." -ForegroundColor Yellow
    rustup component add rustfmt
} else {
    Write-Host "rustfmt already installed" -ForegroundColor Green
}

if ($installedComponents -notcontains "clippy") {
    Write-Host "Installing clippy..." -ForegroundColor Yellow
    rustup component add clippy
} else {
    Write-Host "clippy already installed" -ForegroundColor Green
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building VoipGlot for Windows..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Check build options
$cleanBuild = $false
$runClippy = $true
$fastBuild = $false

if ($args -contains "--clean") {
    $cleanBuild = $true
    Write-Host "Cleaning previous builds (--clean flag specified)..." -ForegroundColor Yellow
    cargo clean
}

if ($args -contains "--no-clippy") {
    $runClippy = $false
    Write-Host "Skipping clippy (--no-clippy flag specified)..." -ForegroundColor Yellow
}

if ($args -contains "--fast") {
    $fastBuild = $true
    Write-Host "Using fast build profile (--fast flag specified)..." -ForegroundColor Yellow
    Write-Host "  - Disabled LTO for faster compilation" -ForegroundColor Gray
    Write-Host "  - Using parallel compilation (16 codegen units)" -ForegroundColor Gray
    Write-Host "  - Reduced optimization level (still good performance)" -ForegroundColor Gray
}

# Build the release version first (this caches dependencies)
if ($fastBuild) {
    Write-Host "Building fast release version..." -ForegroundColor Yellow
    cargo build --profile fast-release --target x86_64-pc-windows-msvc
} else {
    Write-Host "Building optimized release version..." -ForegroundColor Yellow
    Write-Host "  - Using LTO for maximum performance (slower build)" -ForegroundColor Gray
    Write-Host "  - Single-threaded compilation for best optimization" -ForegroundColor Gray
    cargo build --release --target x86_64-pc-windows-msvc
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Build failed!" -ForegroundColor Red
    Write-Host "Try running with --clean flag if you suspect cached issues:" -ForegroundColor Yellow
    Write-Host "  .\build-windows.ps1 --clean" -ForegroundColor White
    Write-Host "Or try fast build for quicker iteration:" -ForegroundColor Yellow
    Write-Host "  .\build-windows.ps1 --fast" -ForegroundColor White
    exit 1
}

# Run clippy after successful build (uses cached dependencies)
if ($runClippy) {
    Write-Host "Running clippy (using cached dependencies)..." -ForegroundColor Yellow
    if ($fastBuild) {
        cargo clippy --profile fast-release --target x86_64-pc-windows-msvc
    } else {
        cargo clippy --release --target x86_64-pc-windows-msvc
    }
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Warning: Clippy found issues, but build completed successfully" -ForegroundColor Yellow
    } else {
        Write-Host "Clippy passed successfully" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# Show executable location based on profile
if ($fastBuild) {
    Write-Host "Executable location: target\x86_64-pc-windows-msvc\fast-release\voipglot-win.exe" -ForegroundColor Cyan
} else {
    Write-Host "Executable location: target\x86_64-pc-windows-msvc\release\voipglot-win.exe" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "To run the application:" -ForegroundColor Yellow
if ($fastBuild) {
    Write-Host "  target\x86_64-pc-windows-msvc\fast-release\voipglot-win.exe" -ForegroundColor White
} else {
    Write-Host "  target\x86_64-pc-windows-msvc\release\voipglot-win.exe" -ForegroundColor White
}

Write-Host ""
Write-Host "Build optimization tips:" -ForegroundColor Yellow
Write-Host "- Fast development builds: .\build-windows.ps1 --fast" -ForegroundColor White
Write-Host "- Skip clippy for speed: .\build-windows.ps1 --fast --no-clippy" -ForegroundColor White
Write-Host "- Production builds: .\build-windows.ps1 (default, optimized)" -ForegroundColor White
Write-Host "- Clean when needed: .\build-windows.ps1 --clean" -ForegroundColor White
Write-Host "- Dependencies are cached for faster subsequent builds" -ForegroundColor White
Write-Host ""
Write-Host "Build speed comparison:" -ForegroundColor Yellow
Write-Host "- Fast build: ~2-3x faster, slightly larger binary" -ForegroundColor White
Write-Host "- Release build: Slower, smallest and fastest binary" -ForegroundColor White
Write-Host ""
Write-Host "Remember to:" -ForegroundColor Yellow
Write-Host "1. Install VB-CABLE Virtual Audio Device" -ForegroundColor White
Write-Host "2. Set up your API keys in environment variables" -ForegroundColor White
Write-Host "3. Configure config.toml if needed" -ForegroundColor White
Write-Host ""