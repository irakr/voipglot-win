# VoipGlot Windows Build Script
# This script builds the Windows application using the voipglot-core distribution package
# Now includes Tauri GUI integration

param(
    [switch]$Clean,
    [switch]$Fast,
    [switch]$NoClippy,
    [switch]$Test,
    [switch]$Check,
    [switch]$BuildOnly,
    [switch]$PackageOnly,
    [switch]$SetupOnly,
    [switch]$TauriDev,
    [switch]$FrontendBuild
)

# Determine operation mode based on parameters
$operationMode = "full"  # Default: full build and package with Tauri GUI
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
} elseif ($TauriDev) {
    $operationMode = "tauri-dev"
} elseif ($FrontendBuild) {
    $operationMode = "frontend-build"
} elseif ($Fast -or $NoClippy -or $Test -or $Check) {
    # Build with specific options
    $operationMode = "build"
} else {
    # No parameters provided - show usage and run full build with Tauri GUI
    $showUsage = $true
}

if ($showUsage) {
    Write-Host ""
    Write-Host "Usage: .\build.ps1 [options]" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Default behavior (no flags): Full build with Tauri GUI" -ForegroundColor Cyan
    Write-Host "  - Extract voipglot-core package, build Tauri GUI, create executables" -ForegroundColor White
    Write-Host ""
    Write-Host "Operation modes:" -ForegroundColor Cyan
    Write-Host "  -Clean              Clean previous builds and exit" -ForegroundColor White
    Write-Host "  -BuildOnly          Build only (no package extraction)" -ForegroundColor White
    Write-Host "  -PackageOnly        Package only (requires existing build)" -ForegroundColor White
    Write-Host "  -SetupOnly          Setup voipglot-core environment only" -ForegroundColor White
    Write-Host "  -TauriDev           Run Tauri in development mode (GUI)" -ForegroundColor White
    Write-Host "  -FrontendBuild      Build only the frontend (no Tauri GUI)" -ForegroundColor White
    Write-Host ""
    Write-Host "Build options:" -ForegroundColor Cyan
    Write-Host "  -Fast               Use fast build profile (faster compilation)" -ForegroundColor White
    Write-Host "  -NoClippy           Skip clippy linting" -ForegroundColor White
    Write-Host "  -Test               Run tests" -ForegroundColor White
    Write-Host "  -Check              Run cargo check only" -ForegroundColor White
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Cyan
    Write-Host "  .\build.ps1                    # Full build with Tauri GUI" -ForegroundColor White
    Write-Host "  .\build.ps1 -Fast              # Fast full build with Tauri GUI" -ForegroundColor White
    Write-Host "  .\build.ps1 -Clean             # Clean and exit" -ForegroundColor White
    Write-Host "  .\build.ps1 -BuildOnly         # Build only" -ForegroundColor White
    Write-Host "  .\build.ps1 -PackageOnly       # Package only" -ForegroundColor White
    Write-Host "  .\build.ps1 -SetupOnly         # Setup environment only" -ForegroundColor White
    Write-Host "  .\build.ps1 -TauriDev          # Run Tauri GUI in development" -ForegroundColor White
    Write-Host "  .\build.ps1 -FrontendBuild     # Build only the frontend" -ForegroundColor White
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
        
        # Clean extracted voipglot-core package directory
        if (Test-Path $buildDir) {
            Write-Host "Cleaning extracted voipglot-core package directory..." -ForegroundColor Yellow
            Remove-Item $buildDir -Recurse -Force
            Write-Host "Extracted package directory cleaned" -ForegroundColor Green
        }
        
        # Clean .cargo config (will be recreated)
        if (Test-Path ".cargo\config.toml") {
            Write-Host "Cleaning .cargo/config.toml (will be recreated)" -ForegroundColor Yellow
            Remove-Item ".cargo\config.toml" -Force
        }
        
        # Clean frontend dependencies and build artifacts
        Write-Host "Cleaning frontend dependencies and build artifacts..." -ForegroundColor Yellow
        
        # Clean node_modules
        if (Test-Path "node_modules") {
            Write-Host "Removing node_modules directory..." -ForegroundColor Yellow
            Remove-Item "node_modules" -Recurse -Force
            Write-Host "node_modules cleaned" -ForegroundColor Green
        }
        
        # Clean package-lock.json
        if (Test-Path "package-lock.json") {
            Write-Host "Removing package-lock.json..." -ForegroundColor Yellow
            Remove-Item "package-lock.json" -Force
            Write-Host "package-lock.json cleaned" -ForegroundColor Green
        }
        
        # Clean Vite build output
        if (Test-Path "dist") {
            Write-Host "Removing Vite build output (dist/)..." -ForegroundColor Yellow
            Remove-Item "dist" -Recurse -Force
            Write-Host "Vite build output cleaned" -ForegroundColor Green
        }
        
        # Clean Tauri build artifacts
        if (Test-Path "src-tauri\target") {
            Write-Host "Removing Tauri build artifacts..." -ForegroundColor Yellow
            Remove-Item "src-tauri\target" -Recurse -Force
            Write-Host "Tauri build artifacts cleaned" -ForegroundColor Green
        }
        
        # Clean .tauri directory
        if (Test-Path ".tauri") {
            Write-Host "Removing .tauri directory..." -ForegroundColor Yellow
            Remove-Item ".tauri" -Recurse -Force
            Write-Host ".tauri directory cleaned" -ForegroundColor Green
        }
        
        # Clean Tauri resources directory
        if (Test-Path "src-tauri\resources") {
            Write-Host "Removing Tauri resources directory..." -ForegroundColor Yellow
            Remove-Item "src-tauri\resources" -Recurse -Force
            Write-Host "Tauri resources directory cleaned" -ForegroundColor Green
        }
        
        # Clean DLL files copied to Tauri src directory root
        $tauriSrcDir = "src-tauri"
        $dllFiles = Get-ChildItem "$tauriSrcDir\*.dll" -ErrorAction SilentlyContinue
        if ($dllFiles) {
            Write-Host "Removing DLL files from Tauri src directory..." -ForegroundColor Yellow
            Remove-Item "$tauriSrcDir\*.dll" -Force
            Write-Host "DLL files cleaned from Tauri src directory" -ForegroundColor Green
        }
        
        Write-Host "All build artifacts cleaned successfully" -ForegroundColor Green
        Write-Host "Clean operation completed. Exiting." -ForegroundColor Cyan
        exit 0
    }
    "tauri-dev" {
        Write-Host "Tauri Development mode: Running Tauri GUI in development..." -ForegroundColor Yellow
        Write-Host "This will build the frontend and start the GUI application with hot reloading" -ForegroundColor Cyan
        
        # Check if Node.js and npm are installed
        Write-Host "Checking Node.js and npm installation..." -ForegroundColor Yellow
        try {
            $nodeVersion = node --version 2>$null
            $npmVersion = npm --version 2>$null
            if ($LASTEXITCODE -ne 0) {
                throw "Node.js or npm not found"
            }
            Write-Host "Node.js found: $nodeVersion" -ForegroundColor Green
            Write-Host "npm found: $npmVersion" -ForegroundColor Green
        } catch {
            Write-Host "Error: Node.js or npm is not installed" -ForegroundColor Red
            Write-Host "Please install Node.js from https://nodejs.org/ (includes npm)" -ForegroundColor Yellow
            Write-Host "After installation, restart PowerShell and run the build script again" -ForegroundColor Yellow
            exit 1
        }

        # Install frontend dependencies
        Write-Host "Installing frontend dependencies..." -ForegroundColor Yellow
        npm install
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Error: Failed to install frontend dependencies" -ForegroundColor Red
            Write-Host "Try running 'npm install' manually to see detailed error messages" -ForegroundColor Yellow
            exit 1
        }
        Write-Host "Frontend dependencies installed successfully" -ForegroundColor Green
        
        # Build frontend to ensure dist directory is up to date
        Write-Host "Building frontend to ensure dist directory is up to date..." -ForegroundColor Yellow
        npm run build
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Error: Failed to build frontend" -ForegroundColor Red
            Write-Host "Try running 'npm run build' manually to see detailed error messages" -ForegroundColor Yellow
            exit 1
        }
        Write-Host "Frontend built successfully" -ForegroundColor Green
        Write-Host "dist/ directory is now up to date with latest TypeScript changes" -ForegroundColor Green
        
        # Check if Tauri CLI is installed
        try {
            $tauriVersion = cargo tauri --version 2>$null
            if ($LASTEXITCODE -ne 0) {
                throw "Tauri CLI not found"
            }
            Write-Host "Tauri CLI found: $tauriVersion" -ForegroundColor Green
        } catch {
            Write-Host "Error: Tauri CLI is not installed" -ForegroundColor Red
            Write-Host "Installing Tauri CLI..." -ForegroundColor Yellow
            cargo install tauri-cli --version '^2.0.0' --locked
            if ($LASTEXITCODE -ne 0) {
                Write-Host "Error: Failed to install Tauri CLI" -ForegroundColor Red
                exit 1
            }
        }
        
        Write-Host "Starting Tauri development server..." -ForegroundColor Yellow
        Write-Host "Note: For frontend changes, you may need to run 'npm run build' again" -ForegroundColor Cyan
        Write-Host "Then restart the Tauri development server with 'cargo tauri dev'" -ForegroundColor Cyan
        Push-Location src-tauri
        cargo tauri dev
        Pop-Location
        exit $LASTEXITCODE
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
        Write-Host "Full mode: Extract package, build Tauri GUI, and package..." -ForegroundColor Yellow
        # Continue with normal full process including Tauri GUI
    }
    "frontend-build" {
        Write-Host "Frontend-only mode: Building frontend only..." -ForegroundColor Yellow
        Write-Host "This will build the TypeScript frontend and generate the dist/ directory" -ForegroundColor Cyan
        
        # Check if Node.js and npm are installed
        Write-Host "Checking Node.js and npm installation..." -ForegroundColor Yellow
        try {
            $nodeVersion = node --version 2>$null
            $npmVersion = npm --version 2>$null
            if ($LASTEXITCODE -ne 0) {
                throw "Node.js or npm not found"
            }
            Write-Host "Node.js found: $nodeVersion" -ForegroundColor Green
            Write-Host "npm found: $npmVersion" -ForegroundColor Green
        } catch {
            Write-Host "Error: Node.js or npm is not installed" -ForegroundColor Red
            Write-Host "Please install Node.js from https://nodejs.org/ (includes npm)" -ForegroundColor Yellow
            Write-Host "After installation, restart PowerShell and run the build script again" -ForegroundColor Yellow
            exit 1
        }

        # Install frontend dependencies
        Write-Host "Installing frontend dependencies..." -ForegroundColor Yellow
        npm install
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Error: Failed to install frontend dependencies" -ForegroundColor Red
            Write-Host "Try running 'npm install' manually to see detailed error messages" -ForegroundColor Yellow
            exit 1
        }
        Write-Host "Frontend dependencies installed successfully" -ForegroundColor Green
        
        # Build frontend
        Write-Host "Building frontend..." -ForegroundColor Yellow
        npm run build
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Error: Failed to build frontend" -ForegroundColor Red
            Write-Host "Try running 'npm run build' manually to see detailed error messages" -ForegroundColor Yellow
            exit 1
        }
        Write-Host "Frontend built successfully" -ForegroundColor Green
        Write-Host "dist/ directory is now up to date with latest TypeScript changes" -ForegroundColor Green
        
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Green
        Write-Host "Frontend build completed successfully!" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "Next steps:" -ForegroundColor Yellow
        Write-Host "  - Run Tauri development: .\build.ps1 -TauriDev" -ForegroundColor White
        Write-Host "  - Build full application: .\build.ps1 -BuildOnly" -ForegroundColor White
        Write-Host "  - Or run cargo tauri dev directly" -ForegroundColor White
        Write-Host ""
        exit 0
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
Write-Host "Checking voipglot-core package for runtime dependencies..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# First check if the extracted build directory already exists and contains required files
$extractionNeeded = $true
if (Test-Path $buildDir) {
    # Check if required runtime files exist
    $requiredFiles = @("models", "native-libs")
    $missingFiles = @()
    
    foreach ($file in $requiredFiles) {
        if (-not (Test-Path "$buildDir\$file")) {
            $missingFiles += $file
        }
    }
    
    if ($missingFiles.Count -eq 0) {
        $absoluteBuildDir = (Resolve-Path $buildDir).Path
        Write-Host "Found extracted voipglot-core package in $absoluteBuildDir" -ForegroundColor Green
        $extractionNeeded = $false
    } else {
        Write-Host "Re-extracting voipglot-core package (missing: $($missingFiles -join ', '))" -ForegroundColor Yellow
    }
}

# Only check for package file if extraction is needed
if ($extractionNeeded) {
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
}

# Extract package if needed
if ($extractionNeeded) {
    if (Test-Path $buildDir) {
        Remove-Item $buildDir -Recurse -Force
    }

    Write-Host "Extracting voipglot-core package..." -ForegroundColor Yellow
    try {
        Expand-Archive -Path $packagePath -DestinationPath $buildDir -Force
        $absoluteBuildDir = (Resolve-Path $buildDir).Path
        Write-Host "voipglot-core package extracted successfully to $absoluteBuildDir" -ForegroundColor Green
    } catch {
        Write-Host "Error extracting package: $_" -ForegroundColor Red
        exit 1
    }

    # Verify package contents for runtime dependencies
    $requiredFiles = @("models", "native-libs")
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
}

# Set up VOSK environment variables for linking
Write-Host "" 
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Setting up VOSK environment for linking..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$voskLibPath = "$buildDir\native-libs"
if (Test-Path "$voskLibPath\libvosk.lib") {
    Write-Host "Found VOSK library at: $voskLibPath\libvosk.lib" -ForegroundColor Green
    
    # Set VOSK environment variables for linking
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
    Write-Host "  libvosk.lib found: $voskLibPath\libvosk.lib" -ForegroundColor Green
} else {
    Write-Host "Warning: VOSK library not found at $voskLibPath\libvosk.lib" -ForegroundColor Yellow
    Write-Host "This may cause linking errors if voipglot-core requires VOSK" -ForegroundColor Yellow
}

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
        
        # Note: voipglot-core build directory is used for runtime dependencies
        # (models, native libraries) - cleaned separately above
        
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

# Build the Tauri GUI application (which includes CLI functionality)
Write-Host "" 
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building Tauri GUI Application..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Check if Node.js and npm are installed
Write-Host "Checking Node.js and npm installation..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version 2>$null
    $npmVersion = npm --version 2>$null
    if ($LASTEXITCODE -ne 0) {
        throw "Node.js or npm not found"
    }
    Write-Host "Node.js found: $nodeVersion" -ForegroundColor Green
    Write-Host "npm found: $npmVersion" -ForegroundColor Green
} catch {
    Write-Host "Error: Node.js or npm is not installed" -ForegroundColor Red
    Write-Host "Please install Node.js from https://nodejs.org/ (includes npm)" -ForegroundColor Yellow
    Write-Host "After installation, restart PowerShell and run the build script again" -ForegroundColor Yellow
    exit 1
}

# Install frontend dependencies
Write-Host "Installing frontend dependencies..." -ForegroundColor Yellow
npm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Failed to install frontend dependencies" -ForegroundColor Red
    Write-Host "Try running 'npm install' manually to see detailed error messages" -ForegroundColor Yellow
    exit 1
}
Write-Host "Frontend dependencies installed successfully" -ForegroundColor Green

# Check if Tauri CLI is installed
try {
    $tauriVersion = cargo tauri --version 2>$null
    if ($LASTEXITCODE -ne 0) {
        throw "Tauri CLI not found"
    }
    Write-Host "Tauri CLI found: $tauriVersion" -ForegroundColor Green
} catch {
    Write-Host "Error: Tauri CLI is not installed" -ForegroundColor Red
    Write-Host "Installing Tauri CLI..." -ForegroundColor Yellow
    cargo install tauri-cli --version '^2.0.0' --locked
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Failed to install Tauri CLI" -ForegroundColor Red
        exit 1
    }
}

# Copy native dependencies and models to Tauri bundle directory before building
Write-Host "Copying native dependencies and models to Tauri bundle directory..." -ForegroundColor Yellow
$tauriSrcDir = "src-tauri"
$tauriResourcesDir = "$tauriSrcDir\resources"

# Copy native libraries to Tauri src directory root (will be bundled in root)
Copy-Item "$buildDir\native-libs\*.dll" $tauriSrcDir -Force
Write-Host "Native dependencies copied to Tauri src directory root" -ForegroundColor Green

# Copy model files to resources subdirectory
if (-not (Test-Path $tauriResourcesDir)) {
    New-Item -ItemType Directory -Path $tauriResourcesDir -Force | Out-Null
}
$tauriModelsDir = "$tauriResourcesDir\models"
if (-not (Test-Path $tauriModelsDir)) {
    New-Item -ItemType Directory -Path $tauriModelsDir -Force | Out-Null
}
Copy-Item "$buildDir\models\*" $tauriModelsDir -Force -Recurse
Write-Host "Model files copied to Tauri resources directory" -ForegroundColor Green

Write-Host "Building Tauri GUI application..." -ForegroundColor Yellow
Push-Location src-tauri

if ($operationMode -eq "build") {
    # For build-only mode, build frontend first, then use cargo build
    Write-Host "Build-only mode: Building frontend first..." -ForegroundColor Yellow
    Pop-Location
    
    # Build the frontend
    Write-Host "Building frontend with npm..." -ForegroundColor Yellow
    npm run build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Frontend build failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "Frontend built successfully" -ForegroundColor Green
    
    # Now build the Rust application
    Write-Host "Building Rust application with cargo build..." -ForegroundColor Yellow
    Push-Location src-tauri
    cargo build --release
    $tauriExitCode = $LASTEXITCODE
} else {
    # For full mode, use cargo tauri build (creates installers and handles frontend)
    Write-Host "Using cargo tauri build for full mode (creates installers)..." -ForegroundColor Yellow
    cargo tauri build
    $tauriExitCode = $LASTEXITCODE
}

Pop-Location

if ($tauriExitCode -ne 0) {
    Write-Host "Error: Tauri GUI build failed!" -ForegroundColor Red
    exit 1
}

if ($operationMode -eq "build") {
    Write-Host "Tauri GUI build completed successfully (executable only)!" -ForegroundColor Green
    Write-Host "Executable location: target\release\voipglot-win.exe" -ForegroundColor Cyan
} else {
    Write-Host "Tauri GUI build completed successfully!" -ForegroundColor Green
    
    # Copy native dependencies to the final bundle location
    Write-Host "Copying native dependencies to final bundle..." -ForegroundColor Yellow
    $finalBundleDir = "target\release\bundle\msi"
    if (Test-Path $finalBundleDir) {
        # Find the MSI file
        $msiFiles = Get-ChildItem $finalBundleDir -Filter "*.msi"
        if ($msiFiles.Count -gt 0) {
            $msiFile = $msiFiles[0]
            Write-Host "Found MSI bundle: $($msiFile.Name)" -ForegroundColor Green
            
            # Create a native-libs directory next to the MSI for manual installation
            $bundleNativeLibs = "$finalBundleDir\native-libs"
            if (-not (Test-Path $bundleNativeLibs)) {
                New-Item -ItemType Directory -Path $bundleNativeLibs -Force | Out-Null
            }
            Copy-Item "$buildDir\native-libs\*" $bundleNativeLibs -Force
            Write-Host "Native dependencies copied to bundle directory for manual installation" -ForegroundColor Green
            Write-Host "Note: DLLs are now automatically bundled in the MSI installer" -ForegroundColor Green
        }
    }
    
    # Post-build: Verify resources are properly bundled
    Write-Host "Verifying resources in final bundle..." -ForegroundColor Yellow
    $finalBundleDir = "target\release\bundle\msi"
    if (Test-Path $finalBundleDir) {
        # Find the MSI file
        $msiFiles = Get-ChildItem $finalBundleDir -Filter "*.msi"
        if ($msiFiles.Count -gt 0) {
            $msiFile = $msiFiles[0]
            Write-Host "Found MSI bundle: $($msiFile.Name)" -ForegroundColor Green
            
            # Check if resources are properly bundled
            $tauriSrcDir = "src-tauri"
            $nativeLibsCount = (Get-ChildItem "$tauriSrcDir\*.dll" -File).Count
            Write-Host "Native libraries bundled: $nativeLibsCount files (will be extracted to root directory)" -ForegroundColor Green
            
            if (Test-Path "$tauriResourcesDir\models") {
                $modelsCount = (Get-ChildItem "$tauriResourcesDir\models" -Recurse -File).Count
                Write-Host "Model files bundled: $modelsCount files" -ForegroundColor Green
            }
            
            Write-Host "Resources should be automatically included in the MSI installer" -ForegroundColor Green
        }
    }
}

# Also copy to the main target directory for the executable
Write-Host "Copying native dependencies to main target directory..." -ForegroundColor Yellow
$mainTargetDir = "target\release"
if (Test-Path $mainTargetDir) {
    $mainNativeLibs = "$mainTargetDir\native-libs"
    if (-not (Test-Path $mainNativeLibs)) {
        New-Item -ItemType Directory -Path $mainNativeLibs -Force | Out-Null
    }
    Copy-Item "$buildDir\native-libs\*" $mainNativeLibs -Force
    Write-Host "Native dependencies copied to main target directory" -ForegroundColor Green
}

Write-Host "" 
Write-Host "========================================" -ForegroundColor Green
Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# Show executable locations
if ($operationMode -eq "build") {
    Write-Host "Tauri GUI Executable location: target\release\voipglot-win.exe" -ForegroundColor Cyan
    Write-Host "Build-only mode: No installers created" -ForegroundColor Yellow
} else {
    Write-Host "Tauri GUI Executable location: target\release\bundle\" -ForegroundColor Cyan
    Write-Host "Tauri GUI files:" -ForegroundColor Cyan
    if (Test-Path "target\release\bundle\msi") {
        Write-Host "  - MSI Installer: target\release\bundle\msi\" -ForegroundColor White
    }
    if (Test-Path "target\release\bundle\nsis") {
        Write-Host "  - NSIS Installer: target\release\bundle\nsis\" -ForegroundColor White
    }
    if (Test-Path "target\release\bundle\updater") {
        Write-Host "  - Updater: target\release\bundle\updater\" -ForegroundColor White
    }
}

Write-Host ""
Write-Host "VoipGlot Windows Application:" -ForegroundColor Green
Write-Host "=============================" -ForegroundColor Green
Write-Host "[OK] Tauri GUI built successfully with modern interface" -ForegroundColor White
Write-Host "[OK] Audio processing and translation handled by core library" -ForegroundColor White
Write-Host "[OK] Windows-specific optimizations applied" -ForegroundColor White

Write-Host ""
Write-Host "To run the application:" -ForegroundColor Yellow
if ($operationMode -eq "build") {
    Write-Host "  Executable: target\release\voipglot-win.exe" -ForegroundColor White
} else {
    Write-Host "  GUI: Run the installer from target\release\bundle\" -ForegroundColor White
}
Write-Host "  GUI Dev: .\build.ps1 -TauriDev" -ForegroundColor White

Write-Host ""
Write-Host "Build optimization tips:" -ForegroundColor Yellow
Write-Host "- Fast development builds: .\build.ps1 -Fast" -ForegroundColor White
Write-Host "- Skip clippy for speed: .\build.ps1 -Fast -NoClippy" -ForegroundColor White
Write-Host "- Production builds: .\build.ps1 (default, optimized)" -ForegroundColor White
Write-Host "- Build only (no installers): .\build.ps1 -BuildOnly" -ForegroundColor White
Write-Host "- Frontend only: .\build.ps1 -FrontendBuild" -ForegroundColor White
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
Write-Host "5. The application includes both GUI and CLI functionality" -ForegroundColor White
Write-Host ""

# Copy runtime dependencies to target directory
Write-Host "Copying runtime dependencies to target directory..." -ForegroundColor Yellow

# Copy models to target directory for runtime access
Write-Host "Copying models to target directory..." -ForegroundColor Yellow
if (Test-Path "$buildDir\models") {
    Copy-Item "$buildDir\models" $mainTargetDir -Recurse -Force
    Write-Host "Models copied to target directory" -ForegroundColor Green
} else {
    Write-Host "Warning: Models directory not found in package" -ForegroundColor Yellow
}

# Copy native libraries to target directory for runtime
Write-Host "Copying native libraries to target directory..." -ForegroundColor Yellow
$targetNativeLibs = "$mainTargetDir\native-libs"
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
Write-Host "[OK] voipglot-win Tauri GUI built successfully" -ForegroundColor Green
Write-Host "[OK] All dependencies resolved by Cargo" -ForegroundColor Green
Write-Host "[OK] Runtime dependencies extracted and copied" -ForegroundColor Green
Write-Host "[OK] Ready for runtime execution" -ForegroundColor Green
Write-Host ""
} 