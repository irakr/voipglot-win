# Coqui TTS Test Run Script
# This script ensures the Python environment is properly set up and runs the app

Write-Host "Coqui TTS Test Run Script" -ForegroundColor Green
Write-Host "=========================" -ForegroundColor Green

# Check if Python and TTS are available
Write-Host "Checking Python environment..." -ForegroundColor Yellow
try {
    python -c "import TTS" | Out-Null
    Write-Host "TTS module found." -ForegroundColor Green
} catch {
    Write-Host "TTS module not found. Please run .\build.ps1 first to set up dependencies." -ForegroundColor Red
    exit 1
}

# Get Python executable path
$pythonExe = python -c "import sys; print(sys.executable)" 2>$null
if ($pythonExe) {
    $env:PYTHON_EXECUTABLE = $pythonExe
    Write-Host "Set PYTHON_EXECUTABLE to: $pythonExe" -ForegroundColor Green
}

# Get Python site-packages path
$sitePackages = python -c "import site; print(site.getsitepackages()[0])" 2>$null
if ($sitePackages) {
    $env:PYTHONPATH = $sitePackages
    Write-Host "Set PYTHONPATH to: $sitePackages" -ForegroundColor Green
}

# Also try user site-packages
$userSitePackages = python -c "import site; print(site.getusersitepackages())" 2>$null
if ($userSitePackages -and (Test-Path $userSitePackages)) {
    if ($env:PYTHONPATH) {
        $env:PYTHONPATH = "$env:PYTHONPATH;$userSitePackages"
    } else {
        $env:PYTHONPATH = $userSitePackages
    }
    Write-Host "Added user site-packages to PYTHONPATH: $userSitePackages" -ForegroundColor Green
}

# Set additional PyO3 environment variables
$env:PYO3_PYTHON = $pythonExe
Write-Host "Set PYO3_PYTHON to: $pythonExe" -ForegroundColor Green

# Run the application
Write-Host "Starting Coqui TTS test application..." -ForegroundColor Yellow
cargo run 