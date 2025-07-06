# PyTorch Setup Guide for VoipGlot

This guide will help you set up PyTorch for local Text-to-Speech (TTS) and translation capabilities in VoipGlot.

## Prerequisites

- Windows 10/11
- Rust toolchain (already installed)
- Visual Studio Build Tools (already installed)
- At least 2GB free disk space

## Step 1: Download PyTorch 1.12.1

**IMPORTANT**: You must use PyTorch 1.12.1 specifically. Other versions are incompatible.

1. Go to: https://pytorch.org/get-started/previous-versions/
2. Find **PyTorch 1.12.1** in the version history
3. Download: `libtorch-win-shared-with-deps-1.12.1+cpu.zip`
4. Extract the ZIP file to `C:\libtorch`

**File structure should look like:**
```
C:\libtorch\
├── include\
├── lib\
├── share\
└── version.txt
```

## Step 2: Verify Installation

Run the verification script:
```powershell
.\build-with-pytorch.ps1 --clean
```

This script will:
- Check if PyTorch is installed correctly
- Set up environment variables automatically
- Build VoipGlot with local TTS and translation

## Step 3: Build with Local Features

### Option A: Use the PyTorch Build Script (Recommended)
```powershell
# Clean build with PyTorch support
.\build-with-pytorch.ps1 --clean

# Fast build for development
.\build-with-pytorch.ps1 --fast

# Build without clippy for speed
.\build-with-pytorch.ps1 --fast --no-clippy
```

### Option B: Manual Build
```powershell
# Set environment variables
$env:LIBTORCH="C:\libtorch"
$env:CXXFLAGS="/std:c++17"

# Build
.\build-windows.ps1 --clean
```

## Step 4: Run the Application

```powershell
target\x86_64-pc-windows-msvc\release\voipglot-win.exe
```

## Features Available

With PyTorch 1.12.1 installed, your VoipGlot application will support:

- ✅ **Local Text-to-Speech (TTS)** - Generate speech from text locally
- ✅ **Local Translation** - Translate text between languages locally
- ✅ **Speech-to-Text (STT)** - Convert speech to text (via API)
- ✅ **Audio Processing** - Capture, process, and playback audio

## Troubleshooting

### Build Errors

**Error: "Cannot find a libtorch install"**
- Make sure PyTorch is extracted to `C:\libtorch`
- Check that the `include` and `lib` folders exist

**Error: "C++17 standard required"**
- The build script automatically sets `CXXFLAGS="/std:c++17"`
- Make sure you're using the `build-with-pytorch.ps1` script

**Error: API compatibility issues**
- Ensure you're using PyTorch 1.12.1, not 1.13.x or newer
- Delete `C:\libtorch` and reinstall the correct version

### Runtime Errors

**Error: "Text-to-Speech not available"**
- PyTorch is not installed or not in the correct location
- Run `.\build-with-pytorch.ps1 --clean` to rebuild

**Error: "Local translation not available"**
- Same as above - PyTorch installation issue

## Performance Notes

- **First run**: Models will be downloaded automatically (~500MB)
- **Memory usage**: ~2GB RAM for local TTS and translation
- **CPU usage**: Moderate during translation, higher during TTS
- **Disk space**: ~1GB for models and PyTorch libraries

## Alternative: API-Only Mode

If you prefer to use API-based services instead of local processing:

1. Comment out PyTorch dependencies in `Cargo.toml`:
```toml
# rust-bert = "=0.21.0"
# tokenizers = "=0.15.0"
# tch = "=0.13.0"
```

2. Use the regular build script:
```powershell
.\build-windows.ps1 --clean
```

This will disable local TTS and translation but keep API-based features working.

## Support

If you encounter issues:

1. Check this troubleshooting guide
2. Verify PyTorch version: `Get-Content C:\libtorch\version.txt`
3. Try a clean build: `.\build-with-pytorch.ps1 --clean`
4. Check available disk space and RAM 