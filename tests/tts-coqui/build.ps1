# Coqui TTS Test Build Script (coqui-tts crate)
# =============================================
#
# This script builds the Rust test app and ensures Python+TTS dependencies are present.
#
# Usage:
#   .\build.ps1 [-Clean] [-Test] [-Release] [-SkipPythonSetup]
#
#   -SkipPythonSetup   Skip the Python/TTS dependency setup (for CI or advanced users)

param(
    [switch]$Clean,
    [switch]$Test,
    [switch]$Release,
    [switch]$SkipPythonSetup
)

# --- Python/Coqui TTS Python Dependency Setup ---
if (-not $SkipPythonSetup) {
    Write-Host "Checking for Python..." -ForegroundColor Yellow
    $python = Get-Command python -ErrorAction SilentlyContinue
    if (-not $python) {
        Write-Host "Python is not installed or not in PATH. Please install Python 3.7+ and re-run this script." -ForegroundColor Red
        exit 1
    }

    # Check if TTS is already available globally
    try {
        python -c "import TTS" | Out-Null
        Write-Host "TTS module found in global Python environment." -ForegroundColor Green
    } catch {
        Write-Host "TTS module not found in global Python. Installing..." -ForegroundColor Yellow
        
        # Install TTS globally (outside any venv)
        Write-Host "Installing TTS globally..." -ForegroundColor Yellow
        python -m pip install --user TTS
        
        # Verify installation
        try {
            python -c "import TTS"
            Write-Host "TTS module installed successfully in global Python." -ForegroundColor Green
        } catch {
            Write-Host "Failed to install TTS module globally. Please install manually: pip install TTS" -ForegroundColor Red
            exit 1
        }
    }

    # Create a virtual environment for development (optional)
    $venvPath = ".venv"
    if (-not (Test-Path $venvPath)) {
        Write-Host "Creating Python virtual environment for development..." -ForegroundColor Yellow
        python -m venv $venvPath
    }

    # Activate the virtual environment for build-time verification
    $activateScript = ".\.venv\Scripts\Activate.ps1"
    Write-Host "Activating Python virtual environment for verification..." -ForegroundColor Yellow
    & $activateScript

    # Upgrade pip in venv
    Write-Host "Upgrading pip in virtual environment..." -ForegroundColor Yellow
    python -m pip install --upgrade pip

    # Install TTS in venv as well (for consistency)
    Write-Host "Installing TTS in virtual environment..." -ForegroundColor Yellow
    python -m pip install TTS

    # Check that TTS is importable in venv
    try {
        python -c "import TTS"
        Write-Host "Python TTS module is available in virtual environment." -ForegroundColor Green
    } catch {
        Write-Host "Failed to import TTS Python module in virtual environment." -ForegroundColor Red
        exit 1
    }

    # Set environment variable to help Rust find Python
    $env:PYTHONPATH = "$env:USERPROFILE\AppData\Local\Programs\Python\Python*\Lib\site-packages"
    Write-Host "Set PYTHONPATH to help Rust find Python modules." -ForegroundColor Green
}

# Clean if requested
if ($Clean) {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Yellow
    if (Test-Path "target") {
        Remove-Item -Recurse -Force "target"
    }
    Write-Host "Clean completed" -ForegroundColor Green
}

# Check if config.toml exists, create if not
if (-not (Test-Path "config.toml")) {
    Write-Host "Creating default config.toml..." -ForegroundColor Yellow
    @"
# Coqui TTS Test Configuration

[audio]
input_device = ""
output_device = ""
sample_rate = 22050
channels = 1
buffer_size = 2048

[tts]
model_path = ""
voice_speed = 1.0
voice_pitch = 1.0
enable_gpu = false

[logging]
level = "info"
"@ | Out-File -FilePath "config.toml" -Encoding UTF8
    Write-Host "Default config.toml created" -ForegroundColor Green
}

# Build the project
Write-Host "Building project..." -ForegroundColor Yellow

try {
    if ($Test) {
        Write-Host "Running tests..." -ForegroundColor Yellow
        cargo test
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Tests passed!" -ForegroundColor Green
        } else {
            Write-Host "Tests failed!" -ForegroundColor Red
            exit 1
        }
    } elseif ($Release) {
        Write-Host "Building release version..." -ForegroundColor Yellow
        cargo build --release
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Release build completed successfully!" -ForegroundColor Green
            Write-Host "Run with: .\target\release\tts-coqui-test.exe" -ForegroundColor Cyan
        } else {
            Write-Host "Release build failed!" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "Building debug version..." -ForegroundColor Yellow
        cargo build
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Debug build completed successfully!" -ForegroundColor Green
            Write-Host "Run with: cargo run" -ForegroundColor Cyan
        } else {
            Write-Host "Debug build failed!" -ForegroundColor Red
            exit 1
        }
    }
}
catch {
    Write-Host "Build error: $_" -ForegroundColor Red
    exit 1
}

Write-Host "`nBuild complete!" -ForegroundColor Green
Write-Host "To run the application:" -ForegroundColor Cyan
if ($Release) {
    Write-Host "  .\target\release\tts-coqui-test.exe" -ForegroundColor White
} else {
    Write-Host "  cargo run" -ForegroundColor White
}
Write-Host "  cargo run -- --list-devices  # List available audio devices" -ForegroundColor White

Write-Host "`nConfiguration:" -ForegroundColor Yellow
Write-Host "  Edit config.toml to customize audio devices and TTS settings" -ForegroundColor Cyan 