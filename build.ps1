# VoipGlot Windows Build Script
# This script builds the Windows application using the voipglot-core distribution package

param(
    [switch]$Clean,
    [switch]$Fast,
    [switch]$NoClippy,
    [switch]$Test,
    [switch]$Check,
    [switch]$BuildOnly,
    [switch]$PackageOnly,
    [switch]$SetupOnly
)

# Determine operation mode based on parameters
$operationMode = "full"  # Default: full build and package
$showUsage = $false

# Check for specific operation modes
if ($Clean) {
    $operationMode = "clean"
} elseif ($BuildOnly) {
    $operationMode = "build"
} elseif ($PackageOnly) {
    $operationMode = "package"
} elseif ($SetupOnly) {
    $operationMode = "setup"
} elseif ($Fast -or $NoClippy -or $Test -or $Check) {
    # Build with specific options
    $operationMode = "build"
} else {
    # No parameters provided - show usage and run full build
    $showUsage = $true
}

if ($showUsage) {
    Write-Host ""
    Write-Host "Usage: .\build.ps1 [options]" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Default behavior (no flags): Full build and package" -ForegroundColor Cyan
    Write-Host "  - Extract voipglot-core package, build, and create executable" -ForegroundColor White
    Write-Host ""
    Write-Host "Operation modes:" -ForegroundColor Cyan
    Write-Host "  -Clean              Clean previous builds and exit" -ForegroundColor White
    Write-Host "  -BuildOnly          Build only (no package extraction)" -ForegroundColor White
    Write-Host "  -PackageOnly        Package only (requires existing build)" -ForegroundColor White
    Write-Host "  -SetupOnly          Setup voipglot-core environment only" -ForegroundColor White
    Write-Host ""
    Write-Host "Build options:" -ForegroundColor Cyan
    Write-Host "  -Fast               Use fast build profile (faster compilation)" -ForegroundColor White
    Write-Host "  -NoClippy           Skip clippy linting" -ForegroundColor White
    Write-Host "  -Test               Run tests" -ForegroundColor White
    Write-Host "  -Check              Run cargo check only" -ForegroundColor White
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Cyan
    Write-Host "  .\build.ps1                    # Full build and package" -ForegroundColor White
    Write-Host "  .\build.ps1 -Fast              # Fast full build and package" -ForegroundColor White
    Write-Host "  .\build.ps1 -Clean             # Clean and exit" -ForegroundColor White
    Write-Host "  .\build.ps1 -BuildOnly         # Build only" -ForegroundColor White
    Write-Host "  .\build.ps1 -PackageOnly       # Package only" -ForegroundColor White
    Write-Host "  .\build.ps1 -SetupOnly         # Setup environment only" -ForegroundColor White
    Write-Host ""
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "VoipGlot Windows Build Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Define build directory for runtime dependencies
$buildDir = "build-voipglot-core"

# Handle different operation modes
Write-Host "" 
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Operation Mode: $operationMode" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

switch ($operationMode) {
    "clean" {
        Write-Host "Clean mode: Cleaning build artifacts and exiting..." -ForegroundColor Yellow
        
        Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
        
        # Clean Cargo build artifacts
        Write-Host "Cleaning Cargo build artifacts..." -ForegroundColor Yellow
        cargo clean
        
        # Note: No longer using voipglot-core build directory
        # Using Cargo dependency instead
        
        # Clean .cargo config (will be recreated)
        if (Test-Path ".cargo\config.toml") {
            Write-Host "Cleaning .cargo/config.toml (will be recreated)" -ForegroundColor Yellow
            Remove-Item ".cargo\config.toml" -Force
        }
        
        Write-Host "All build artifacts cleaned successfully" -ForegroundColor Green
        Write-Host "Clean operation completed. Exiting." -ForegroundColor Cyan
        exit 0
    }
    "setup" {
        Write-Host "Setup mode: Setting up voipglot-core environment only..." -ForegroundColor Yellow
        # Continue with package extraction and setup
    }
    "build" {
        Write-Host "Build mode: Building application only..." -ForegroundColor Yellow
        # Using Cargo dependency - no need to check for extracted package
    }
    "package" {
        Write-Host "Package mode: Creating executable package only..." -ForegroundColor Yellow
        # Skip to package creation
    }
    "full" {
        Write-Host "Full mode: Extract package, build, and package..." -ForegroundColor Yellow
        # Continue with normal full process
    }
}

# Note: Using voipglot-core as Cargo dependency for compilation
# But still need to extract package for runtime dependencies (models, native libs)
Write-Host "Using voipglot-core as Cargo dependency" -ForegroundColor Green

# Set Rust flags for static runtime to match voipglot-core
Write-Host "Setting Rust flags for static runtime..." -ForegroundColor Yellow
$env:RUSTFLAGS = "-C target-feature=+crt-static"
Write-Host "RUSTFLAGS set to: $env:RUSTFLAGS" -ForegroundColor Green

# Extract voipglot-core package for runtime dependencies (models, native libs)
Write-Host "" 
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Extracting voipglot-core package for runtime dependencies..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Check for voipglot-core package
if (-not $env:LIBVOIPGLOT_CORE_PKG) {
    Write-Host "Error: LIBVOIPGLOT_CORE_PKG environment variable not set" -ForegroundColor Red
    Write-Host "Please set the path to the voipglot-core package ZIP file:" -ForegroundColor Yellow
    Write-Host "  `$env:LIBVOIPGLOT_CORE_PKG = 'path/to/voipglot-core-release.zip'" -ForegroundColor White
    Write-Host "Then run: .\build.ps1" -ForegroundColor White
    exit 1
}

$packagePath = $env:LIBVOIPGLOT_CORE_PKG
if (-not (Test-Path $packagePath)) {
    Write-Host "Error: Package file not found at: $packagePath" -ForegroundColor Red
    Write-Host "Please check the LIBVOIPGLOT_CORE_PKG environment variable" -ForegroundColor Yellow
    exit 1
}

Write-Host "Found package: $packagePath" -ForegroundColor Green

# Extract package to build directory
if (Test-Path $buildDir) {
    Write-Host "Removing existing build directory..." -ForegroundColor Yellow
    Remove-Item $buildDir -Recurse -Force
}

Write-Host "Extracting package to: $buildDir" -ForegroundColor Yellow
try {
    Expand-Archive -Path $packagePath -DestinationPath $buildDir -Force
    Write-Host "Package extracted successfully" -ForegroundColor Green
} catch {
    Write-Host "Error extracting package: $_" -ForegroundColor Red
    exit 1
}

# Verify package contents for runtime dependencies
$requiredFiles = @(
    "models",
    "native-libs"
)

$missingFiles = @()
foreach ($file in $requiredFiles) {
    if (-not (Test-Path "$buildDir\$file")) {
        $missingFiles += $file
    }
}

if ($missingFiles.Count -gt 0) {
    Write-Host "Error: Package is missing required runtime files:" -ForegroundColor Red
    foreach ($file in $missingFiles) {
        Write-Host "  - $file" -ForegroundColor Red
    }
    Write-Host "Please ensure you have a valid voipglot-core distribution package" -ForegroundColor Yellow
    exit 1
}

Write-Host "Runtime dependencies verified" -ForegroundColor Green

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
Write-Host "Setting up voipglot-core linking..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Note: No longer creating .cargo directory for static linking
# Using voipglot-core as Cargo dependency instead

# Note: Using voipglot-core as a Cargo dependency instead of static linking
# The dependency is declared in Cargo.toml: voipglot-core = { path = "../voipglot-core" }
Write-Host "Using voipglot-core as Cargo dependency" -ForegroundColor Green

# Skip build for package-only mode
if ($operationMode -eq "package") {
    Write-Host "" 
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "Skipping build (package-only mode)..." -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "Proceeding directly to package creation..." -ForegroundColor Yellow
} else {
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
        
        # Clean Cargo build artifacts
        cargo clean
        
        # Note: No longer using voipglot-core build directory
        # Using Cargo dependency instead
        
        # Clean .cargo config (will be recreated)
        if (Test-Path ".cargo\config.toml") {
            Write-Host "Cleaning .cargo/config.toml (will be recreated)" -ForegroundColor Yellow
            Remove-Item ".cargo\config.toml" -Force
        }
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
Write-Host "[OK] Built successfully using voipglot-core library" -ForegroundColor White
Write-Host "[OK] Audio processing and translation handled by core library" -ForegroundColor White
Write-Host "[OK] Windows-specific optimizations applied" -ForegroundColor White

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
    Write-Host "[OK] config.toml found and preserved" -ForegroundColor Green
    Write-Host "Your configuration file was not modified by the build script" -ForegroundColor White
} else {
    Write-Host "[ERROR] config.toml not found" -ForegroundColor Red
    Write-Host "Please create a config.toml file with the correct structure" -ForegroundColor White
}

Write-Host ""
Write-Host "Remember to:" -ForegroundColor Yellow
Write-Host "1. Models are managed by voipglot-core library" -ForegroundColor White
Write-Host "2. Install VB-CABLE Virtual Audio Device if needed" -ForegroundColor White
Write-Host "3. Your config.toml file is preserved and not overwritten" -ForegroundColor White
Write-Host "4. Using voipglot-core as Cargo dependency" -ForegroundColor White
Write-Host ""

# Copy runtime dependencies to target directory
Write-Host "Copying runtime dependencies to target directory..." -ForegroundColor Yellow

# Copy models to target directory for runtime access
Write-Host "Copying models to target directory..." -ForegroundColor Yellow
if (Test-Path "$buildDir\models") {
    Copy-Item "$buildDir\models" $targetDir -Recurse -Force
    Write-Host "Models copied to target directory" -ForegroundColor Green
} else {
    Write-Host "Warning: Models directory not found in package" -ForegroundColor Yellow
}

# Copy native libraries to target directory for runtime
Write-Host "Copying native libraries to target directory..." -ForegroundColor Yellow
$targetNativeLibs = "$targetDir\native-libs"
if (-not (Test-Path $targetNativeLibs)) {
    New-Item -ItemType Directory -Path $targetNativeLibs -Force | Out-Null
}
Copy-Item "$buildDir\native-libs\*" $targetNativeLibs -Force
Write-Host "Native libraries copied to target directory" -ForegroundColor Green

Write-Host "Runtime dependencies copied successfully" -ForegroundColor Green
Write-Host ""

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "[OK] voipglot-win built successfully using voipglot-core library" -ForegroundColor Green
Write-Host "[OK] All dependencies resolved by Cargo" -ForegroundColor Green
Write-Host "[OK] Runtime dependencies extracted and copied" -ForegroundColor Green
Write-Host "[OK] Ready for runtime execution" -ForegroundColor Green
Write-Host ""
} 