# VOSK STT Test Build Script
# This script helps build and set up the VOSK test project
# Defaults: VOSK at C:\vosk, models at C:\vosk\models

param(
    [string]$ModelPath = "",
    [string]$VoskPath = "C:\vosk",
    [switch]$DownloadModel,
    [switch]$Clean,
    [switch]$SetupEnv
)

Write-Host "VOSK STT Test Build Script" -ForegroundColor Green
Write-Host "==========================" -ForegroundColor Green

# Setup environment variables for VOSK
function Setup-VoskEnvironment {
    Write-Host "Setting up VOSK environment variables..." -ForegroundColor Yellow
    
    # Default VOSK path if not provided
    if (-not $VoskPath) {
        $VoskPath = "C:\vosk"
    }
    
    # Check if VOSK directory exists
    if (!(Test-Path $VoskPath)) {
        Write-Host "Warning: VOSK directory not found at: $VoskPath" -ForegroundColor Yellow
        Write-Host "Please ensure VOSK is installed or specify path with -VoskPath" -ForegroundColor Yellow
        return $false
    }
    
    # Set environment variables for VOSK
    $env:LIBRARY_PATH = $VoskPath
    $env:VOSK_LIB_PATH = $VoskPath
    $env:INCLUDE_PATH = $VoskPath
    
    # Add VOSK to PATH if not already there
    if ($env:PATH -notlike "*$VoskPath*") {
        $env:PATH += ";$VoskPath"
    }
    
    Write-Host "Environment variables set:" -ForegroundColor Green
    Write-Host "  LIBRARY_PATH: $env:LIBRARY_PATH" -ForegroundColor Cyan
    Write-Host "  VOSK_LIB_PATH: $env:VOSK_LIB_PATH" -ForegroundColor Cyan
    Write-Host "  INCLUDE_PATH: $env:INCLUDE_PATH" -ForegroundColor Cyan
    Write-Host "  PATH includes VOSK: $($env:PATH -like "*$VoskPath*")" -ForegroundColor Cyan
    
    # Verify libvosk.lib exists
    if (Test-Path "$VoskPath\libvosk.lib") {
        Write-Host "  libvosk.lib found: $VoskPath\libvosk.lib" -ForegroundColor Green
    } else {
        Write-Host "  Warning: libvosk.lib not found at $VoskPath\libvosk.lib" -ForegroundColor Yellow
    }
    
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

# Setup environment if requested or if VoskPath is provided
if ($SetupEnv -or $VoskPath) {
    if (!(Setup-VoskEnvironment)) {
        Write-Host "Environment setup failed. Continuing anyway..." -ForegroundColor Yellow
    }
}

# Download model if requested
if ($DownloadModel) {
    Write-Host "Downloading VOSK model..." -ForegroundColor Yellow
    
    $modelUrl = "https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip"
    $modelZip = "vosk-model-small-en-us-0.15.zip"
    $modelDir = "vosk-model-small-en-us-0.15"
    $voskModelsPath = "C:\vosk\models"
    
    # Create VOSK models directory if it doesn't exist
    if (!(Test-Path $voskModelsPath)) {
        Write-Host "Creating VOSK models directory: $voskModelsPath" -ForegroundColor Yellow
        New-Item -ItemType Directory -Path $voskModelsPath -Force | Out-Null
    }
    
    # Download model
    try {
        Write-Host "Downloading model from: $modelUrl" -ForegroundColor Yellow
        Invoke-WebRequest -Uri $modelUrl -OutFile $modelZip
        Write-Host "Model downloaded successfully" -ForegroundColor Green
        
        # Extract model to VOSK models directory
        Write-Host "Extracting model to: $voskModelsPath" -ForegroundColor Yellow
        Expand-Archive -Path $modelZip -DestinationPath $voskModelsPath -Force
        Remove-Item $modelZip
        
        $finalModelPath = "$voskModelsPath\$modelDir"
        Write-Host "Model extracted to: $finalModelPath" -ForegroundColor Green
        
        # Set the model path for this session
        $env:VOSK_MODEL_PATH = $finalModelPath
        Write-Host "VOSK_MODEL_PATH set to: $finalModelPath" -ForegroundColor Green
    }
    catch {
        Write-Host "Failed to download model: $_" -ForegroundColor Red
        Write-Host "Please download manually from: $modelUrl" -ForegroundColor Yellow
        Write-Host "Extract to: $voskModelsPath" -ForegroundColor Yellow
    }
}

# Set model path with smart defaults
if ($ModelPath) {
    $env:VOSK_MODEL_PATH = $ModelPath
    Write-Host "VOSK_MODEL_PATH set to: $ModelPath" -ForegroundColor Green
} else {
    # Default model path: C:\vosk\models\vosk-model-small-en-us-0.15
    $defaultModelPath = "C:\vosk\models\vosk-model-small-en-us-0.15"
    
    if (Test-Path $defaultModelPath) {
        $env:VOSK_MODEL_PATH = $defaultModelPath
        Write-Host "VOSK_MODEL_PATH auto-detected: $defaultModelPath" -ForegroundColor Green
    } else {
        Write-Host "Warning: No VOSK model found at default location: $defaultModelPath" -ForegroundColor Yellow
        Write-Host "To download model to default location, run: .\build.ps1 -DownloadModel" -ForegroundColor Cyan
        Write-Host "To specify custom model path, run: .\build.ps1 -ModelPath 'path\to\model'" -ForegroundColor Cyan
    }
}

# Check for required VOSK files
function Test-VoskFiles {
    $voskPath = if ($VoskPath) { $VoskPath } else { "C:\vosk" }
    
    $requiredFiles = @("libvosk.lib", "libvosk.dll")
    $missingFiles = @()
    
    foreach ($file in $requiredFiles) {
        if (!(Test-Path "$voskPath\$file")) {
            $missingFiles += $file
        }
    }
    
    if ($missingFiles.Count -gt 0) {
        Write-Host "Warning: Missing VOSK files: $($missingFiles -join ', ')" -ForegroundColor Yellow
        Write-Host "Please ensure VOSK is properly installed at: $voskPath" -ForegroundColor Yellow
        return $false
    }
    
    Write-Host "VOSK files check passed" -ForegroundColor Green
    return $true
}

# Check VOSK files
Test-VoskFiles | Out-Null

# Build the project
Write-Host "Building project..." -ForegroundColor Yellow
Write-Host "Using VOSK path: $env:VOSK_LIB_PATH" -ForegroundColor Cyan
Write-Host "Library search path: $env:LIBRARY_PATH" -ForegroundColor Cyan

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
        Write-Host "Check that libvosk.lib is in the VOSK directory and environment variables are set correctly" -ForegroundColor Yellow
        exit 1
    }
}
catch {
    Write-Host "Build error: $_" -ForegroundColor Red
    Write-Host "Check that libvosk.lib is in the VOSK directory and environment variables are set correctly" -ForegroundColor Yellow
    exit 1
}

Write-Host "`nSetup complete!" -ForegroundColor Green
Write-Host "To run the application:" -ForegroundColor Cyan
Write-Host "  cargo run --release" -ForegroundColor White
Write-Host "`nEnvironment variables set for this session:" -ForegroundColor Yellow
Write-Host "  LIBRARY_PATH: $env:LIBRARY_PATH" -ForegroundColor Cyan
Write-Host "  VOSK_LIB_PATH: $env:VOSK_LIB_PATH" -ForegroundColor Cyan
if ($env:VOSK_MODEL_PATH) {
    Write-Host "  VOSK_MODEL_PATH: $env:VOSK_MODEL_PATH" -ForegroundColor Green
} else {
    Write-Host "  VOSK_MODEL_PATH: NOT SET" -ForegroundColor Red
    Write-Host "  To set model path, run: .\build.ps1 -ModelPath 'path\to\model'" -ForegroundColor Yellow
    Write-Host "  To download model, run: .\build.ps1 -DownloadModel" -ForegroundColor Yellow
} 