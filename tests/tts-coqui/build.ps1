# Coqui TTS Test Build Script
# This script helps build and set up the Coqui TTS test project
# Defaults: Coqui TTS at C:\coqui-tts

param(
    [string]$ModelPath = "",
    [string]$CoquiPath = "C:\coqui-tts",
    [switch]$DownloadModel,
    [switch]$Clean,
    [switch]$SetupEnv
)

Write-Host "Coqui TTS Test Build Script" -ForegroundColor Green
Write-Host "==========================" -ForegroundColor Green

# Setup environment variables for Coqui TTS
function Setup-CoquiEnvironment {
    Write-Host "Setting up Coqui TTS environment variables..." -ForegroundColor Yellow
    
    # Default Coqui path if not provided
    if (-not $CoquiPath) {
        $CoquiPath = "C:\coqui-tts"
    }
    
    # Check if Coqui directory exists
    if (!(Test-Path $CoquiPath)) {
        Write-Host "Warning: Coqui TTS directory not found at: $CoquiPath" -ForegroundColor Yellow
        Write-Host "Creating directory: $CoquiPath" -ForegroundColor Yellow
        New-Item -ItemType Directory -Path $CoquiPath -Force | Out-Null
    }
    
    # Set environment variables for Coqui TTS
    $env:COQUI_TTS_PATH = $CoquiPath
    $env:COQUI_TTS_DATA_PATH = "$CoquiPath\models"
    
    # Add Coqui to PATH if not already there
    if ($env:PATH -notlike "*$CoquiPath*") {
        $env:PATH += ";$CoquiPath"
    }
    
    Write-Host "Environment variables set:" -ForegroundColor Green
    Write-Host "  COQUI_TTS_PATH: $env:COQUI_TTS_PATH" -ForegroundColor Cyan
    Write-Host "  COQUI_TTS_DATA_PATH: $env:COQUI_TTS_DATA_PATH" -ForegroundColor Cyan
    Write-Host "  PATH includes Coqui: $($env:PATH -like "*$CoquiPath*")" -ForegroundColor Cyan
    
    return $true
}

# Clean if requested
if ($Clean) {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Yellow
    if (Test-Path "target") {
        Remove-Item -Recurse -Force "target"
    }
    Write-Host "Clean completed" -ForegroundColor Green
}

# Setup environment if requested or if CoquiPath is provided
if ($SetupEnv -or $CoquiPath) {
    if (!(Setup-CoquiEnvironment)) {
        Write-Host "Environment setup failed. Continuing anyway..." -ForegroundColor Yellow
    }
}

# Download model if requested
if ($DownloadModel) {
    Write-Host "Setting up Python environment for Coqui TTS..." -ForegroundColor Yellow
    
    try {
        # Check if Python is installed
        python --version
        if ($LASTEXITCODE -ne 0) {
            throw "Python not found. Please install Python 3.7 or later."
        }
        
        # Create virtual environment if it doesn't exist
        if (!(Test-Path "$CoquiPath\venv")) {
            Write-Host "Creating Python virtual environment..." -ForegroundColor Yellow
            python -m venv "$CoquiPath\venv"
        }
        
        # Activate virtual environment
        Write-Host "Activating virtual environment..." -ForegroundColor Yellow
        & "$CoquiPath\venv\Scripts\Activate.ps1"
        
        # Install/Upgrade pip
        Write-Host "Upgrading pip..." -ForegroundColor Yellow
        python -m pip install --upgrade pip
        
        # Install Coqui TTS
        Write-Host "Installing Coqui TTS..." -ForegroundColor Yellow
        pip install TTS
        
        # Download default model
        Write-Host "Downloading default Coqui TTS model..." -ForegroundColor Yellow
        python -c "from TTS.utils.manage import ModelManager; ModelManager().download_model('tts_models/en/ljspeech/tacotron2-DDC')"
        
        Write-Host "Coqui TTS and model setup completed successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "Failed to set up Coqui TTS: $_" -ForegroundColor Red
        Write-Host "Please ensure Python 3.7+ is installed and accessible" -ForegroundColor Yellow
        exit 1
    }
}

# Check Python environment
function Test-PythonEnvironment {
    try {
        python --version
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Warning: Python not found" -ForegroundColor Yellow
            return $false
        }
        
        if (Test-Path "$CoquiPath\venv") {
            Write-Host "Python virtual environment found" -ForegroundColor Green
            return $true
        } else {
            Write-Host "Warning: Python virtual environment not found" -ForegroundColor Yellow
            Write-Host "Run with -DownloadModel to set up the environment" -ForegroundColor Cyan
            return $false
        }
    }
    catch {
        Write-Host "Warning: Python environment check failed: $_" -ForegroundColor Yellow
        return $false
    }
}

# Check Python environment
Test-PythonEnvironment | Out-Null

# Build the project
Write-Host "Building project..." -ForegroundColor Yellow
Write-Host "Using Coqui TTS path: $env:COQUI_TTS_PATH" -ForegroundColor Cyan

try {
    # Clean build to ensure fresh linking
    Write-Host "Running cargo clean..." -ForegroundColor Yellow
    cargo clean
    
    Write-Host "Building with cargo build --release..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Build completed successfully!" -ForegroundColor Green
        Write-Host "Run with: cargo run --release" -ForegroundColor Cyan
    } else {
        Write-Host "Build failed!" -ForegroundColor Red
        Write-Host "Check that Coqui TTS is properly installed and environment variables are set correctly" -ForegroundColor Yellow
        exit 1
    }
}
catch {
    Write-Host "Build error: $_" -ForegroundColor Red
    Write-Host "Check that Coqui TTS is properly installed and environment variables are set correctly" -ForegroundColor Yellow
    exit 1
}

Write-Host "`nSetup complete!" -ForegroundColor Green
Write-Host "To run the application:" -ForegroundColor Cyan
Write-Host "  cargo run --release" -ForegroundColor White
Write-Host "`nEnvironment variables set for this session:" -ForegroundColor Yellow
Write-Host "  COQUI_TTS_PATH: $env:COQUI_TTS_PATH" -ForegroundColor Cyan
Write-Host "  COQUI_TTS_DATA_PATH: $env:COQUI_TTS_DATA_PATH" -ForegroundColor Cyan

if (Test-Path "$CoquiPath\venv") {
    Write-Host "  Python virtual environment: FOUND" -ForegroundColor Green
} else {
    Write-Host "  Python virtual environment: NOT FOUND" -ForegroundColor Red
    Write-Host "  To set up Coqui TTS and download models, run: .\build.ps1 -DownloadModel" -ForegroundColor Yellow
} 