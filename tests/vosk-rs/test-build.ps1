# Test script to verify Vosk-rs build
# This script tests the build process without running the full application

Write-Host "Testing Vosk-rs Build Process" -ForegroundColor Green
Write-Host "=============================" -ForegroundColor Green

# Check if we're in the right directory
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "Error: Not in the vosk-rs directory. Please run this from voipglot-win\tests\vosk-rs" -ForegroundColor Red
    exit 1
}

# Check if we're in Developer PowerShell
if (-not $env:VSINSTALLDIR) {
    Write-Host "Warning: This script should be run in a Developer PowerShell environment" -ForegroundColor Yellow
    Write-Host "Please open 'Developer PowerShell for VS' and run this script again" -ForegroundColor Yellow
    exit 1
}

# Check if Rust is available
try {
    $rustVersion = rustc --version
    Write-Host "Rust version: $rustVersion" -ForegroundColor Green
}
catch {
    Write-Host "Error: Rust is not installed or not in PATH" -ForegroundColor Red
    exit 1
}

# Check if Cargo is available
try {
    $cargoVersion = cargo --version
    Write-Host "Cargo version: $cargoVersion" -ForegroundColor Green
}
catch {
    Write-Host "Error: Cargo is not installed or not in PATH" -ForegroundColor Red
    exit 1
}

# Test if we can build the workspace
Write-Host "`nTesting workspace build..." -ForegroundColor Blue
try {
    cargo check --workspace
    Write-Host "Workspace check passed!" -ForegroundColor Green
}
catch {
    Write-Host "Error: Workspace check failed" -ForegroundColor Red
    exit 1
}

# Check if Vosk libraries are present
$voskLibDir = "vosk-libs\lib"
if (Test-Path "$voskLibDir\libvosk.dll") {
    Write-Host "Vosk libraries found" -ForegroundColor Green
} else {
    Write-Host "Vosk libraries not found. Run build.ps1 first to download them." -ForegroundColor Yellow
}

# Test if we can build the microphone example (without linking)
Write-Host "`nTesting microphone example compilation..." -ForegroundColor Blue
try {
    # Set environment variables for the test
    if (Test-Path $voskLibDir) {
        $env:LIBRARY_PATH = "$(Get-Location)\$voskLibDir"
        $env:LD_LIBRARY_PATH = "$(Get-Location)\$voskLibDir"
    }
    
    cargo check --example microphone
    Write-Host "Microphone example compilation test passed!" -ForegroundColor Green
}
catch {
    Write-Host "Error: Microphone example compilation failed" -ForegroundColor Red
    Write-Host "This might be due to missing Vosk libraries. Run build.ps1 first." -ForegroundColor Yellow
    exit 1
}

Write-Host "`nAll tests passed! The build environment is ready." -ForegroundColor Green
Write-Host "You can now run the full build with: .\build.ps1" -ForegroundColor Cyan 