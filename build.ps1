# VoipGlot Windows Build Script
# This script builds the Windows application using the voipglot-core library

param(
    [switch]$Clean,
    [switch]$Release,
    [switch]$Fast,
    [switch]$NoClippy
)

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

# Check if the required target is installed
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

# Check if required components are installed
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
Write-Host "Checking voipglot-core dependency..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Check if voipglot-core exists
if (-not (Test-Path "../voipglot-core")) {
    Write-Host "Error: voipglot-core library not found at ../voipglot-core" -ForegroundColor Red
    Write-Host "Please ensure voipglot-core is available in the parent directory" -ForegroundColor Yellow
    exit 1
}

Write-Host "voipglot-core library found" -ForegroundColor Green

Write-Host "" 
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building VoipGlot Windows..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Build options
$cleanBuild = $false
$runClippy = $true
$fastBuild = $false

if ($Clean) {
    $cleanBuild = $true
    Write-Host "Cleaning previous builds (-Clean flag specified)..." -ForegroundColor Yellow
    cargo clean
}

if ($NoClippy) {
    $runClippy = $false
    Write-Host "Skipping clippy (-NoClippy flag specified)..." -ForegroundColor Yellow
}

if ($Fast) {
    $fastBuild = $true
    Write-Host "Using fast build profile (-Fast flag specified)..." -ForegroundColor Yellow
    Write-Host "  - Disabled LTO for faster compilation" -ForegroundColor Gray
    Write-Host "  - Using parallel compilation (16 codegen units)" -ForegroundColor Gray
    Write-Host "  - Reduced optimization level (still good performance)" -ForegroundColor Gray
}

# Build the application
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
    Write-Host "Try running with -Clean flag if you suspect cached issues:" -ForegroundColor Yellow
    Write-Host "  .\build.ps1 -Clean" -ForegroundColor White
    Write-Host "Or try fast build for quicker iteration:" -ForegroundColor Yellow
    Write-Host "  .\build.ps1 -Fast" -ForegroundColor White
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
    $targetDir = "target\x86_64-pc-windows-msvc\fast-release"
    Write-Host "Executable location: $targetDir\voipglot-win.exe" -ForegroundColor Cyan
} else {
    $targetDir = "target\x86_64-pc-windows-msvc\release"
    Write-Host "Executable location: $targetDir\voipglot-win.exe" -ForegroundColor Cyan
}

# Copy configuration file to target directory (preserve existing config)
if (Test-Path "config.toml") {
    Copy-Item "config.toml" $targetDir -Force
    Write-Host "Configuration file copied to target directory" -ForegroundColor Green
} else {
    Write-Host "Warning: config.toml not found. Please create one before running the application." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "VoipGlot Windows Application:" -ForegroundColor Green
Write-Host "=============================" -ForegroundColor Green
Write-Host "✅ Built successfully using voipglot-core library" -ForegroundColor White
Write-Host "✅ Audio processing and translation handled by core library" -ForegroundColor White
Write-Host "✅ Windows-specific optimizations applied" -ForegroundColor White

Write-Host ""
Write-Host "To run the application:" -ForegroundColor Yellow
Write-Host "  $targetDir\voipglot-win.exe" -ForegroundColor White

Write-Host ""
Write-Host "Build optimization tips:" -ForegroundColor Yellow
Write-Host "- Fast development builds: .\build.ps1 -Fast" -ForegroundColor White
Write-Host "- Skip clippy for speed: .\build.ps1 -Fast -NoClippy" -ForegroundColor White
Write-Host "- Production builds: .\build.ps1 (default, optimized)" -ForegroundColor White
Write-Host "- Clean when needed: .\build.ps1 -Clean" -ForegroundColor White
Write-Host "- Dependencies are cached for faster subsequent builds" -ForegroundColor White

Write-Host ""
Write-Host "Configuration:" -ForegroundColor Yellow
Write-Host "==============" -ForegroundColor Yellow
if (Test-Path "config.toml") {
    Write-Host "✅ config.toml found and preserved" -ForegroundColor Green
    Write-Host "Your configuration file was not modified by the build script" -ForegroundColor White
} else {
    Write-Host "❌ config.toml not found" -ForegroundColor Red
    Write-Host "Please create a config.toml file with the correct structure" -ForegroundColor White
}

Write-Host ""
Write-Host "Remember to:" -ForegroundColor Yellow
Write-Host "1. Models are managed by voipglot-core library" -ForegroundColor White
Write-Host "2. Install VB-CABLE Virtual Audio Device if needed" -ForegroundColor White
Write-Host "3. Your config.toml file is preserved and not overwritten" -ForegroundColor White
Write-Host "" 