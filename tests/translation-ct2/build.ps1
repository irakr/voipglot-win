# Build script for translation-ct2 test
# This script downloads and sets up the NLLB-200 model for CTranslate2

# Stop on any error
$ErrorActionPreference = "Stop"

# Function to check if a command exists
function Test-Command {
    param (
        [string]$Name
    )
    return $null -ne (Get-Command -Name $Name -ErrorAction SilentlyContinue)
}

# Check for required build tools
Write-Host "Checking build requirements..."
$cmake_version = (cmake --version | Select-Object -First 1)
Write-Host "Full CMake version info:"
cmake --version
if (-not $cmake_version) {
    Write-Host "Error: CMake is not installed"
    Write-Host "Please install CMake from: https://cmake.org/download/"
    exit 1
}
Write-Host "Found CMake version: $cmake_version"

# Check for Visual Studio Build Tools
function Test-VSBuildTools {
    # Check for MSBuild
    $msbuild = Get-Command "MSBuild.exe" -ErrorAction SilentlyContinue
    if (-not $msbuild) {
        return $false
    }

    # Check for cl.exe (C++ compiler)
    $vcvarsall = @(
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvarsall.bat",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvarsall.bat"
    )

    foreach ($path in $vcvarsall) {
        if (Test-Path $path) {
            return $true
        }
    }
    
    return $false
}

$vsFound = Test-VSBuildTools

if (-not $vsFound) {
    Write-Host "Warning: Visual Studio Build Tools might not be installed"
    Write-Host "If the build fails, please install Visual Studio Build Tools 2022"
    Write-Host "Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    Write-Host "Make sure to select 'Desktop development with C++' during installation"
    Write-Host "Press any key to continue or Ctrl+C to cancel..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
}

# Function to parse Python version
function Get-PythonVersion {
    try {
        $versionString = python -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" 2>&1
        return [version]$versionString
    } catch {
        Write-Host "Error getting Python version: $_"
        return $null
    }
}

# Check if Python is installed and version is compatible
$minVersion = [version]"3.8"
$maxVersion = [version]"3.12"  # Current max tested version
$pythonVersion = Get-PythonVersion

if ($null -eq $pythonVersion) {
    Write-Host "Error: Python is not installed or not in PATH"
    Write-Host "Please install Python 3.8 or later from https://www.python.org/downloads/"
    exit 1
}

Write-Host "Found Python version: $pythonVersion"

if ($pythonVersion -lt $minVersion) {
    Write-Host "Error: Python version $pythonVersion is too old. Minimum required version is $minVersion"
    Write-Host "Please upgrade Python or install a newer version from https://www.python.org/downloads/"
    exit 1
}

if ($pythonVersion -gt $maxVersion) {
    Write-Host "Warning: Python version $pythonVersion is newer than the maximum tested version $maxVersion"
    Write-Host "The script may still work, but if you encounter issues, consider using Python $maxVersion"
    # Continue execution, just warning
}

# Check if pip is installed and install if needed
try {
    $pipVersion = python -m pip --version 2>&1
    Write-Host "Found pip: $pipVersion"
} catch {
    Write-Host "Installing pip..."
    try {
        Invoke-WebRequest -Uri "https://bootstrap.pypa.io/get-pip.py" -OutFile "get-pip.py"
        python get-pip.py
        Remove-Item "get-pip.py"
    } catch {
        Write-Host "Error installing pip. Please install pip manually."
        exit 1
    }
}

# Create models directory if it doesn't exist
$modelsDir = Join-Path $PWD "models"
if (-not (Test-Path $modelsDir)) {
    New-Item -ItemType Directory -Path $modelsDir
}

# Install required Python packages
Write-Host "Installing required Python packages..."
$requiredPackages = @(
    "ctranslate2",
    "transformers",
    "torch",
    "huggingface_hub",
    "protobuf",  # Added for compatibility
    "hf_xet"     # Added for better download performance
)

foreach ($package in $requiredPackages) {
    Write-Host "Installing $package..."
    try {
        if ($package -eq "torch") {
            # Install PyTorch with specific CUDA version if needed
            python -m pip install --upgrade torch --index-url https://download.pytorch.org/whl/cu118
        } else {
            python -m pip install --upgrade $package
        }
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to install $package"
        }
    } catch {
        Write-Host "Error installing $package. Error: $_"
        if ($package -eq "hf_xet") {
            Write-Host "Continuing without hf_xet (optional package)"
            continue
        }
        exit 1
    }
}

# Set environment variable to disable symlink warning
$env:HF_HUB_DISABLE_SYMLINKS_WARNING = "1"

# Download and convert NLLB-200 model if not already present
$modelDir = Join-Path $modelsDir "nllb-200-ct2"
$modelDirPosix = $modelDir.Replace("\", "/")

if (-not (Test-Path $modelDir)) {
    Write-Host "Converting NLLB-200 model to CTranslate2 format..."
    
    # Create model directory first
    New-Item -ItemType Directory -Path $modelDir -Force
    
    # Convert model using Python script
    $pythonScript = @"
from ctranslate2.converters import TransformersConverter
from transformers import AutoTokenizer, AutoModelForSeq2SeqLM
import os
import torch
import shutil

def convert_model():
    print("CUDA available:", torch.cuda.is_available())
    if torch.cuda.is_available():
        print("CUDA device:", torch.cuda.get_device_name(0))
    
    print("Downloading NLLB-200 model...")
    model_name = 'facebook/nllb-200-distilled-600M'
    
    print("Loading tokenizer...")
    tokenizer = AutoTokenizer.from_pretrained(model_name)
    
    print("Loading model...")
    model = AutoModelForSeq2SeqLM.from_pretrained(model_name)
    
    print("Converting model to CTranslate2 format...")
    converter = TransformersConverter(model_name)
    
    # Use a temporary directory for conversion
    temp_dir = '$modelDirPosix.tmp'
    if os.path.exists(temp_dir):
        shutil.rmtree(temp_dir)
    os.makedirs(temp_dir)
    
    try:
        converter.convert(temp_dir, force=True)
        print("Saving tokenizer...")
        tokenizer.save_pretrained(temp_dir)
        
        # Move files from temp directory to final location
        for item in os.listdir(temp_dir):
            s = os.path.join(temp_dir, item)
            d = os.path.join('$modelDirPosix', item)
            if os.path.isfile(s):
                shutil.copy2(s, d)
            else:
                if os.path.exists(d):
                    shutil.rmtree(d)
                shutil.copytree(s, d)
        
        print("Conversion completed successfully!")
    finally:
        if os.path.exists(temp_dir):
            shutil.rmtree(temp_dir)

if __name__ == '__main__':
    convert_model()
"@

    # Save and run the Python script
    $pythonScript | Out-File -FilePath "convert_model.py" -Encoding utf8
    try {
        python convert_model.py
        if ($LASTEXITCODE -ne 0) {
            throw "Model conversion failed"
        }
    } catch {
        Write-Host "Error during model conversion: $_"
        exit 1
    } finally {
        Remove-Item -Force "convert_model.py" -ErrorAction SilentlyContinue
    }
    
    Write-Host "Model conversion completed"
}

# Before building the Rust project, set required environment variables
Write-Host "Setting up build environment..."
$env:RUSTFLAGS = "-C target-feature=+crt-static"

# Build the Rust project
Write-Host "Building Rust project..."
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo build failed"
    }
} catch {
    Write-Host "Error building Rust project: $_"
    Write-Host "`nTroubleshooting steps:"
    Write-Host "1. Make sure CMake is properly installed and in PATH"
    Write-Host "2. Install Visual Studio Build Tools 2022 with 'Desktop development with C++'"
    Write-Host "3. Try running the build script from a new PowerShell session"
    Write-Host "4. If issues persist, try running from Developer PowerShell for VS 2022"
    exit 1
}

Write-Host "Build completed successfully!" 