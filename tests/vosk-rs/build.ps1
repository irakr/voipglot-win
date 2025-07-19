# Vosk-rs Build Script for Windows
# This script downloads Vosk libraries and builds the microphone example

param(
    [switch]$Clean
)

Write-Host "Vosk-rs Build Script for Windows" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green

# Clean build artifacts if requested
if ($Clean) {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Blue
    if (Test-Path "target") {
        Remove-Item -Recurse -Force "target"
        Write-Host "Removed target directory" -ForegroundColor Green
    }
    if (Test-Path "Cargo.lock") {
        Remove-Item -Force "Cargo.lock"
        Write-Host "Removed Cargo.lock" -ForegroundColor Green
    }
    Write-Host "Clean completed" -ForegroundColor Green
    Write-Host ""
}

# Check if we're in a Developer PowerShell
if (-not $env:VSINSTALLDIR) {
    Write-Host "Warning: This script should be run in a Developer PowerShell environment" -ForegroundColor Yellow
    Write-Host "However, we'll try to continue with the current environment..." -ForegroundColor Yellow
    Write-Host "If you encounter build errors, please use 'Developer PowerShell for VS'" -ForegroundColor Yellow
}

# Create directories
$voskDir = "vosk-libs"
$libDir = "$voskDir\lib"
$includeDir = "$voskDir\include"

if (-not (Test-Path $voskDir)) {
    New-Item -ItemType Directory -Path $voskDir | Out-Null
    New-Item -ItemType Directory -Path $libDir | Out-Null
    New-Item -ItemType Directory -Path $includeDir | Out-Null
}

# Download Vosk libraries if not present
$voskVersion = "0.3.45"
$voskUrl = "https://github.com/alphacep/vosk-api/releases/download/v$voskVersion/vosk-win64-$voskVersion.zip"
$voskZip = "$voskDir\vosk-win64-$voskVersion.zip"

if (-not (Test-Path "$libDir\libvosk.dll")) {
    Write-Host "Downloading Vosk libraries..." -ForegroundColor Blue
    try {
        Invoke-WebRequest -Uri $voskUrl -OutFile $voskZip
        Write-Host "Extracting Vosk libraries..." -ForegroundColor Blue
        Expand-Archive -Path $voskZip -DestinationPath $voskDir -Force
        
        # Move files to correct locations
        if (Test-Path "$voskDir\vosk-win64-$voskVersion") {
            Copy-Item "$voskDir\vosk-win64-$voskVersion\*.dll" $libDir -Force
            Copy-Item "$voskDir\vosk-win64-$voskVersion\*.lib" $libDir -Force
            if (Test-Path "$voskDir\vosk-win64-$voskVersion\include") {
                Copy-Item "$voskDir\vosk-win64-$voskVersion\include\*" $includeDir -Force -Recurse
            }
        }
        Write-Host "Vosk libraries downloaded and extracted successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "Error downloading Vosk libraries: $_" -ForegroundColor Red
        exit 1
    }
}
else {
    Write-Host "Vosk libraries already present" -ForegroundColor Green
}

# Set environment variables for the build
$env:LIBRARY_PATH = "$(Get-Location)\$libDir"
$env:LD_LIBRARY_PATH = "$(Get-Location)\$libDir"

Write-Host "Building vosk-rs workspace..." -ForegroundColor Blue
try {
    # Build the workspace
    cargo build --release
    
    # Build the microphone example
    Write-Host "Building microphone example..." -ForegroundColor Blue
    cargo build --release --example microphone
    
    Write-Host "Build completed successfully!" -ForegroundColor Green
    
    # Copy DLL to target directory for runtime
    $targetDir = "target\release"
    Copy-Item "$libDir\*.dll" $targetDir -Force
    
    Write-Host "`nAvailable examples:" -ForegroundColor Green
    Write-Host "- microphone.exe: Clean speech recognition with user-friendly output" -ForegroundColor White
    
    Write-Host "`nUsage:" -ForegroundColor Yellow
    Write-Host "======" -ForegroundColor Yellow
    Write-Host "Build: .\build.ps1" -ForegroundColor Cyan
    Write-Host "Clean build: .\build.ps1 -Clean" -ForegroundColor Cyan
    
    Write-Host "`nSetup Instructions:" -ForegroundColor Yellow
    Write-Host "==================" -ForegroundColor Yellow
    Write-Host "1. Download a Vosk model from: https://alphacephei.com/vosk/models" -ForegroundColor White
    Write-Host "2. Extract the model to a directory (e.g., C:\vosk-models\en-us)" -ForegroundColor White
    Write-Host "3. Run the microphone example:" -ForegroundColor White
    Write-Host "   .\target\release\examples\microphone.exe <model-path> <duration>" -ForegroundColor Cyan
    Write-Host "   Example: .\target\release\examples\microphone.exe C:\vosk-models\en-us 10" -ForegroundColor Cyan
}
catch {
    Write-Host "Build failed: $_" -ForegroundColor Red
    exit 1
} 