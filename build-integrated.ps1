# VoipGlot Integrated Build Script (PowerShell)
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "VoipGlot Integrated Build Script" -ForegroundColor Cyan
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

# Check if models should be downloaded
$downloadModels = $false
if ($args -contains "-DownloadModels") {
    $downloadModels = $true
    Write-Host "Download models flag detected. Will attempt to download missing models." -ForegroundColor Yellow
}

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
    if ($downloadModels) {
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
    if ($downloadModels) {
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
    if ($downloadModels) {
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
Write-Host "Building VoipGlot (integrated)..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Build options
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
    Write-Host "  .\build-integrated.ps1 --clean" -ForegroundColor White
    Write-Host "Or try fast build for quicker iteration:" -ForegroundColor Yellow
    Write-Host "  .\build-integrated.ps1 --fast" -ForegroundColor White
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
Write-Host "- Fast development builds: .\build-integrated.ps1 --fast" -ForegroundColor White
Write-Host "- Skip clippy for speed: .\build-integrated.ps1 --fast --no-clippy" -ForegroundColor White
Write-Host "- Production builds: .\build-integrated.ps1 (default, optimized)" -ForegroundColor White
Write-Host "- Clean when needed: .\build-integrated.ps1 --clean" -ForegroundColor White
Write-Host "- Dependencies are cached for faster subsequent builds" -ForegroundColor White
Write-Host ""
Write-Host "Remember to:" -ForegroundColor Yellow
Write-Host "1. Download and extract required models to the models/ directory" -ForegroundColor White
Write-Host "2. Install VB-CABLE Virtual Audio Device if needed" -ForegroundColor White
Write-Host "3. Configure config.toml if needed" -ForegroundColor White
Write-Host ""
