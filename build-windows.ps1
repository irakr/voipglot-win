#!/usr/bin/env pwsh

# VoipGlot Windows Build Script (PowerShell)
# Unified script for both regular and PyTorch builds

param(
    [switch]$Clean,
    [switch]$Fast,
    [switch]$NoClippy,
    [switch]$NoPyTorch,
    [switch]$ForcePyTorch
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

# Check if MSVC compiler is available
try {
    $clVersion = cl.exe 2>&1 | Select-String "Microsoft.*C/C\+\+.*Compiler"
    if ($LASTEXITCODE -ne 0 -or -not $clVersion) {
        throw "MSVC compiler not found"
    }
    Write-Host "MSVC compiler found: $($clVersion.Line.Trim())" -ForegroundColor Green
} catch {
    Write-Host "Error: Microsoft C++ compiler (cl.exe) not found in PATH" -ForegroundColor Red
    Write-Host "This is required for building native dependencies (PyTorch, audio libraries)" -ForegroundColor Yellow
    Write-Host "" -ForegroundColor Yellow
    Write-Host "Solutions:" -ForegroundColor Cyan
    Write-Host "1. Install Visual Studio 2022 with C++ workload:" -ForegroundColor Gray
    Write-Host "   - Open Visual Studio Installer" -ForegroundColor Gray
    Write-Host "   - Modify Visual Studio 2022" -ForegroundColor Gray
    Write-Host "   - Check 'Desktop development with C++' workload" -ForegroundColor Gray
    Write-Host "   - Install and restart terminal" -ForegroundColor Gray
    Write-Host "" -ForegroundColor Gray
    Write-Host "2. Or install standalone C++ Build Tools:" -ForegroundColor Gray
    Write-Host "   - Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Gray
    Write-Host "   - Select 'C++ build tools' workload" -ForegroundColor Gray
    Write-Host "" -ForegroundColor Gray
    Write-Host "3. Or use Visual Studio Developer Command Prompt:" -ForegroundColor Gray
    Write-Host "   - Open 'Developer Command Prompt for VS 2022'" -ForegroundColor Gray
    Write-Host "   - Navigate to project directory" -ForegroundColor Gray
    Write-Host "   - Run: powershell -ExecutionPolicy Bypass -File .\build-windows.ps1" -ForegroundColor Gray
    Write-Host "" -ForegroundColor Gray
    Write-Host "After installing, restart your terminal and try again." -ForegroundColor Yellow
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

# PyTorch detection and setup
$usePyTorch = $false
$libtorchPath = "C:\libtorch"

if ($ForcePyTorch) {
    $usePyTorch = $true
    Write-Host "PyTorch build forced by --force-pytorch flag" -ForegroundColor Yellow
} elseif (-not $NoPyTorch) {
    # Check if PyTorch is available
    if (Test-Path $libtorchPath) {
        $usePyTorch = $true
        Write-Host "PyTorch detected at $libtorchPath" -ForegroundColor Green
        
        # Check if PyTorch version is correct
        $versionFile = Join-Path $libtorchPath "version.txt"
        $buildVersionFile = Join-Path $libtorchPath "build-version"
        
        if (Test-Path $versionFile) {
            $version = Get-Content $versionFile
            Write-Host "PyTorch version: $version" -ForegroundColor Green
        } elseif (Test-Path $buildVersionFile) {
            $version = Get-Content $buildVersionFile
            Write-Host "PyTorch version: $version" -ForegroundColor Green
        } else {
            Write-Host "Warning: Could not determine PyTorch version" -ForegroundColor Yellow
            Write-Host "Make sure you have PyTorch 1.12.1+ installed" -ForegroundColor Yellow
        }

        # After detecting $version, set a compatibility note
        if ($version -match '^2\\.') {
            $compatNote = 'Note: Using compatibility flags for PyTorch 2.x + MSVC 2022'
        } else {
            $compatNote = 'Note: Using compatibility flags for PyTorch 1.12.1 + MSVC 2022'
        }
    } else {
        Write-Host "PyTorch not found at $libtorchPath" -ForegroundColor Yellow
        Write-Host "Building without PyTorch support (local translation/STT/TTS disabled)" -ForegroundColor Yellow
        Write-Host "To enable PyTorch support:" -ForegroundColor Cyan
        Write-Host "  1. Download PyTorch 1.12.1+ from https://pytorch.org/get-started/previous-versions/" -ForegroundColor Gray
        Write-Host "  2. Extract to C:\libtorch" -ForegroundColor Gray
        Write-Host "  3. Run this script again" -ForegroundColor Gray
        Write-Host "Or use --no-pytorch to skip this check" -ForegroundColor Gray
    }
}

# Set environment variables for PyTorch if needed
if ($usePyTorch) {
    $env:LIBTORCH = $libtorchPath
    # Set LIBTORCH_INCLUDE to parent directory so tch crate appends \include correctly
    $env:LIBTORCH_INCLUDE = $libtorchPath
    $env:LIBTORCH_LIB = Join-Path $libtorchPath "lib"
    
    # Add RUSTFLAGS for PyTorch libraries
    $env:RUSTFLAGS = "-L C:\libtorch\lib -l torch_cpu -l torch -l c10"
    
    # Add compiler flags for PyTorch compatibility with newer MSVC
    # Also add include paths to ensure all headers are found
    $env:CXXFLAGS = "/std:c++17 /EHsc /wd4067 /wd4805 /wd4624 /wd4996 /wd4530 /I`"$libtorchPath\include`" /I`"$libtorchPath\include\torch\csrc\api\include`""
    $env:DISTUTILS_USE_SDK = "1"
    
    Write-Host "Environment variables set for PyTorch:" -ForegroundColor Green
    Write-Host "  LIBTORCH = $env:LIBTORCH" -ForegroundColor Gray
    Write-Host "  LIBTORCH_INCLUDE = $env:LIBTORCH_INCLUDE" -ForegroundColor Gray
    Write-Host "  LIBTORCH_LIB = $env:LIBTORCH_LIB" -ForegroundColor Gray
    Write-Host "  CXXFLAGS = $env:CXXFLAGS" -ForegroundColor Gray
    Write-Host "  DISTUTILS_USE_SDK = $env:DISTUTILS_USE_SDK" -ForegroundColor Gray
    Write-Host "  $compatNote" -ForegroundColor Gray
    Write-Host "  Note: Include paths added to CXXFLAGS for proper header resolution" -ForegroundColor Gray
    
    # Check for compatibility issues
    Write-Host "Warning: PyTorch may have compatibility issues with newer MSVC" -ForegroundColor Yellow
    Write-Host "If build fails, consider:" -ForegroundColor Yellow
    Write-Host "  1. Using a newer PyTorch version" -ForegroundColor Gray
    Write-Host "  2. Building without PyTorch: .\build-windows.ps1 --no-pytorch" -ForegroundColor Gray
    Write-Host "  3. Using fast build for quicker iteration: .\build-windows.ps1 --fast" -ForegroundColor Gray
} else {
    # Clear any existing PyTorch environment variables
    if ($env:LIBTORCH) {
        Remove-Item Env:LIBTORCH
    }
    if ($env:LIBTORCH_INCLUDE) {
        Remove-Item Env:LIBTORCH_INCLUDE
    }
    if ($env:LIBTORCH_LIB) {
        Remove-Item Env:LIBTORCH_LIB
    }
    if ($env:CXXFLAGS) {
        Remove-Item Env:CXXFLAGS
    }
    if ($env:DISTUTILS_USE_SDK) {
        Remove-Item Env:DISTUTILS_USE_SDK
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building VoipGlot for Windows..." -ForegroundColor Cyan
if ($usePyTorch) {
    Write-Host "With PyTorch support (local translation/STT/TTS)" -ForegroundColor Green
} else {
    Write-Host "Without PyTorch support (API-based translation only)" -ForegroundColor Yellow
}
Write-Host "========================================" -ForegroundColor Cyan

# Check build options
$cleanBuild = $false
$runClippy = $true
$fastBuild = $false

if ($Clean) {
    $cleanBuild = $true
    Write-Host "Cleaning previous builds (--clean flag specified)..." -ForegroundColor Yellow
    cargo clean
}

if ($NoClippy) {
    $runClippy = $false
    Write-Host "Skipping clippy (--no-clippy flag specified)..." -ForegroundColor Yellow
}

if ($Fast) {
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
    if ($usePyTorch) {
        Write-Host "If PyTorch-related errors occur, try building without PyTorch:" -ForegroundColor Yellow
        Write-Host "  .\build-windows.ps1 --no-pytorch" -ForegroundColor White
    }
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
Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host ""

# Show executable location based on profile
if ($fastBuild) {
    Write-Host "Executable: target\x86_64-pc-windows-msvc\fast-release\voipglot-win.exe" -ForegroundColor Cyan
} else {
    Write-Host "Executable: target\x86_64-pc-windows-msvc\release\voipglot-win.exe" -ForegroundColor Cyan
}
Write-Host ""