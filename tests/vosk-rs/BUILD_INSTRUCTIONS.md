# Vosk-rs Microphone Example Build Instructions

This document explains how to build and run the Vosk microphone example on Windows.

## Prerequisites

1. **Developer PowerShell for Visual Studio** - This is required for building Rust projects with native dependencies on Windows
2. **Rust toolchain** - Make sure you have Rust installed and up to date
3. **Vosk Speech Recognition Model** - You'll need to download a model for speech recognition

## Quick Start

### 1. Open Developer PowerShell

Open "Developer PowerShell for Visual Studio" (not regular PowerShell) and navigate to the vosk-rs directory:

```powershell
cd voipglot-win\tests\vosk-rs
```

### 2. Run the Build Script

The build script will automatically:
- Download Vosk dynamic libraries for Windows
- Set up the build environment
- Build the microphone example
- Copy necessary DLLs to the target directory

```powershell
.\build.ps1
```

### 3. Download a Vosk Model

Download a speech recognition model from [Vosk Models](https://alphacephei.com/vosk/models). For testing, you can use:

- **Small English model**: `vosk-model-small-en-us-0.15`
- **Large English model**: `vosk-model-en-us-0.22`

Extract the model to a directory, for example: `C:\vosk-models\en-us`

### 4. Run the Microphone Example

The microphone example provides clean, user-friendly output with speech recognition results:

```powershell
.\target\release\examples\microphone.exe "C:\vosk-models\en-us" 10
```

This will:
- Record audio from your microphone for 10 seconds
- Show partial results as you speak with clear formatting
- Display final results with confidence scores
- Provide a clean, readable output format

## Manual Build Process

If you prefer to build manually:

### 1. Download Vosk Libraries

Download `vosk-win64-0.3.45.zip` from [Vosk API Releases](https://github.com/alphacep/vosk-api/releases)

Extract to `vosk-libs\` directory with this structure:
```
vosk-libs/
├── lib/
│   ├── libvosk.dll
│   └── libvosk.lib
└── include/
    └── vosk_api.h
```

### 2. Set Environment Variables

```powershell
$env:LIBRARY_PATH = "$(Get-Location)\vosk-libs\lib"
$env:LD_LIBRARY_PATH = "$(Get-Location)\vosk-libs\lib"
```

### 3. Build the Project

```powershell
cargo build --release --example microphone
```

### 4. Copy DLLs for Runtime

```powershell
Copy-Item "vosk-libs\lib\*.dll" "target\release\" -Force
```

## Troubleshooting

### Build Errors

1. **"linker command failed"** - Make sure you're using Developer PowerShell, not regular PowerShell
2. **"cannot find -llibvosk"** - Check that the Vosk libraries are in the correct location
3. **"entry point not found"** - Ensure you're using the correct version of Vosk libraries

### Runtime Errors

1. **"libvosk.dll not found"** - Make sure the DLL is copied to the target directory or in your PATH
2. **"No input device connected"** - Check that your microphone is properly connected and set as default
3. **"Could not create the model"** - Verify the model path is correct and the model files are present

### Audio Issues

1. **No audio input** - Check Windows audio settings and microphone permissions
2. **Poor recognition** - Try a larger model or check microphone quality
3. **Sample rate issues** - The example automatically detects your microphone's sample rate

## Example Output

When running successfully, you should see output like:

```
🎤 Recording for 10 seconds... (speak now)
📝 Partial results will show as you speak:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔄 "hello"
🔄 "hello world"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎯 Final result:
✅ "hello world" (confidence: 0.95)
```

## Project Structure

```
vosk-rs/
├── build.ps1                    # Automated build script
├── BUILD_INSTRUCTIONS.md        # This file
├── crates/
│   ├── vosk/                    # High-level Rust bindings
│   │   ├── examples/
│   │   │   ├── microphone.rs    # Main microphone example (clean output)
│   │   │   └── microphone-original.rs  # Original verbose version (backup)
│   │   └── Cargo.toml
│   └── vosk-sys/                # Low-level FFI bindings
│       ├── build.rs             # Build script for linking
│       └── Cargo.toml
├── vosk-libs/                   # Vosk dynamic libraries (created by build script)
│   ├── lib/
│   │   ├── libvosk.dll
│   │   └── libvosk.lib
│   └── include/
└── target/                      # Build output
    └── release/
        └── examples/
            └── microphone.exe
``` 