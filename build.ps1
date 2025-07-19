# VoipGlot Windows Build Script
# This script builds the main VoipGlot application using the proven integrated approach

param(
    [switch]$Clean,
    [switch]$Release,
    [switch]$DownloadModels,
    [switch]$ForceDownload,
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
Write-Host "Checking required models..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Create models directory if it doesn't exist
if (-not (Test-Path "models")) {
    Write-Host "Creating models directory..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path "models" | Out-Null
}

$voskPath = "models/vosk-model-small-en-us-0.15"
$ct2Path = "models/nllb-200-ct2"
$coquiPath = "tts_models/en/ljspeech/tacotron2-DDC"
$allModelsPresent = $true

# Check VOSK model
if (-not (Test-Path $voskPath)) {
    Write-Host "VOSK model not found at: $voskPath" -ForegroundColor Red
    if ($DownloadModels -or $ForceDownload) {
        Write-Host "Setting up VOSK model using Python script..." -ForegroundColor Yellow
        try {
            python scripts/setup-vosk.py
            if ($LASTEXITCODE -eq 0) {
                Write-Host "VOSK model setup completed successfully" -ForegroundColor Green
                $allModelsPresent = $true
            } else {
                Write-Host "Error setting up VOSK model" -ForegroundColor Red
                $allModelsPresent = $false
            }
        }
        catch {
            Write-Host "Error running VOSK setup script: $_" -ForegroundColor Red
            Write-Host "Please download manually from: https://alphacephei.com/vosk/models/" -ForegroundColor Yellow
            Write-Host "Extract to: models/vosk-model-small-en-us-0.15/" -ForegroundColor Yellow
            $allModelsPresent = $false
        }
    } else {
        Write-Host "Please download from: https://alphacephei.com/vosk/models/" -ForegroundColor Yellow
        Write-Host "Extract to: models/vosk-model-small-en-us-0.15/" -ForegroundColor Yellow
        $allModelsPresent = $false
    }
} else {
    Write-Host "VOSK model found: $voskPath" -ForegroundColor Green
}

# Check CT2 model
if (-not (Test-Path $ct2Path)) {
    Write-Host "CT2 model not found at: $ct2Path" -ForegroundColor Red
    if ($DownloadModels -or $ForceDownload) {
        Write-Host "Setting up CT2 model using Python script..." -ForegroundColor Yellow
        try {
            python scripts/setup-ct2.py
            if ($LASTEXITCODE -eq 0) {
                Write-Host "CT2 model setup completed successfully" -ForegroundColor Green
                $allModelsPresent = $true
            } else {
                Write-Host "Error setting up CT2 model" -ForegroundColor Red
                $allModelsPresent = $false
            }
        }
        catch {
            Write-Host "Error running CT2 setup script: $_" -ForegroundColor Red
            Write-Host "Please download the NLLB-200 CT2 model manually:" -ForegroundColor Yellow
            Write-Host "  1. Visit: https://huggingface.co/facebook/nllb-200-distilled-600M" -ForegroundColor White
            Write-Host "  2. Convert to CT2 format if needed (see CTranslate2 docs)" -ForegroundColor White
            Write-Host "  3. Extract to: models/nllb-200-ct2/" -ForegroundColor White
            $allModelsPresent = $false
        }
    } else {
        Write-Host "Please download the NLLB-200 CT2 model manually:" -ForegroundColor Yellow
        Write-Host "  1. Visit: https://huggingface.co/facebook/nllb-200-distilled-600M" -ForegroundColor White
        Write-Host "  2. Convert to CT2 format if needed (see CTranslate2 docs)" -ForegroundColor White
        Write-Host "  3. Extract to: models/nllb-200-ct2/" -ForegroundColor White
        $allModelsPresent = $false
    }
} else {
    Write-Host "CT2 model found: $ct2Path" -ForegroundColor Green
}

# Note: Coqui TTS models are downloaded automatically at runtime, not required at build time
Write-Host "Coqui TTS model: Will be downloaded automatically on first use" -ForegroundColor Green
Write-Host "  - Model path: $coquiPath" -ForegroundColor Gray
Write-Host "  - No build-time setup required" -ForegroundColor Gray

if (-not $allModelsPresent) {
    Write-Host "" 
    Write-Host "Critical: One or more required models are missing. Build cannot continue." -ForegroundColor Red
    if ($DownloadModels -or $ForceDownload) {
        Write-Host "Use -DownloadModels flag to attempt automatic download of missing models." -ForegroundColor Yellow
    }
    exit 1
}

Write-Host "" 
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Setting up VOSK environment..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Setup VOSK environment variables (similar to test build script)
$voskLibPath = "C:\vosk"
if (Test-Path $voskLibPath) {
    Write-Host "Setting VOSK environment variables..." -ForegroundColor Yellow
    $env:LIBRARY_PATH = $voskLibPath
    $env:VOSK_LIB_PATH = $voskLibPath
    $env:INCLUDE_PATH = $voskLibPath
    
    # Add VOSK to PATH if not already there
    if ($env:PATH -notlike "*$voskLibPath*") {
        $env:PATH += ";$voskLibPath"
    }
    
    Write-Host "VOSK environment variables set:" -ForegroundColor Green
    Write-Host "  LIBRARY_PATH: $env:LIBRARY_PATH" -ForegroundColor Cyan
    Write-Host "  VOSK_LIB_PATH: $env:VOSK_LIB_PATH" -ForegroundColor Cyan
    Write-Host "  INCLUDE_PATH: $env:INCLUDE_PATH" -ForegroundColor Cyan
    
    # Verify libvosk.lib exists
    if (Test-Path "$voskLibPath\libvosk.lib") {
        Write-Host "  libvosk.lib found: $voskLibPath\libvosk.lib" -ForegroundColor Green
    } else {
        Write-Host "  Warning: libvosk.lib not found at $voskLibPath\libvosk.lib" -ForegroundColor Yellow
        Write-Host "  Please ensure VOSK is properly installed at: $voskLibPath" -ForegroundColor Yellow
    }
} else {
    Write-Host "Warning: VOSK directory not found at: $voskLibPath" -ForegroundColor Yellow
    Write-Host "Please ensure VOSK is installed or the path is correct" -ForegroundColor Yellow
}

# Set Rust flags for static runtime (like CT2 test build script)
Write-Host "Setting Rust flags for static runtime..." -ForegroundColor Yellow
$env:RUSTFLAGS = "-C target-feature=+crt-static"
Write-Host "RUSTFLAGS set to: $env:RUSTFLAGS" -ForegroundColor Green

Write-Host "" 
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building VoipGlot..." -ForegroundColor Cyan
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
Write-Host "Available Models:" -ForegroundColor Green
Write-Host "================" -ForegroundColor Green
if (Test-Path $voskPath) {
    Write-Host "✅ VOSK Speech Recognition" -ForegroundColor White
}
if (Test-Path $ct2Path) {
    Write-Host "✅ CTranslate2 Translation" -ForegroundColor White
}
Write-Host "✅ Coqui TTS (auto-download)" -ForegroundColor White

Write-Host ""
Write-Host "To run the application:" -ForegroundColor Yellow
Write-Host "  $targetDir\voipglot-win.exe" -ForegroundColor White

Write-Host ""
Write-Host "Build optimization tips:" -ForegroundColor Yellow
Write-Host "- Fast development builds: .\build.ps1 -Fast" -ForegroundColor White
Write-Host "- Skip clippy for speed: .\build.ps1 -Fast -NoClippy" -ForegroundColor White
Write-Host "- Production builds: .\build.ps1 (default, optimized)" -ForegroundColor White
Write-Host "- Clean when needed: .\build.ps1 -Clean" -ForegroundColor White
Write-Host "- Download models: .\build.ps1 -DownloadModels" -ForegroundColor White
Write-Host "- Force download all: .\build.ps1 -ForceDownload" -ForegroundColor White
Write-Host "- Dependencies are cached for faster subsequent builds" -ForegroundColor White

Write-Host ""
Write-Host "STT Testing:" -ForegroundColor Yellow
Write-Host "============" -ForegroundColor Yellow
if (Test-Path $voskPath) {
    Write-Host "✅ STT ready: VOSK model is available" -ForegroundColor Green
    Write-Host "Speak into your microphone to test speech recognition" -ForegroundColor White
} else {
    Write-Host "❌ STT not ready: VOSK model missing" -ForegroundColor Red
}

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
Write-Host "1. Models are automatically managed by the build script" -ForegroundColor White
Write-Host "2. Install VB-CABLE Virtual Audio Device if needed" -ForegroundColor White
Write-Host "3. Your config.toml file is preserved and not overwritten" -ForegroundColor White
Write-Host "" 