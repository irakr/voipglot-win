# VOSK STT Test - Proof of Concept

This is a basic Rust project that demonstrates real-time Speech-to-Text (STT) using the VOSK framework. The application captures audio from the physical microphone device (Microphone Array) and performs real-time transcription, displaying the transcribed text in the terminal as you speak.

## Features

- Real-time audio capture from physical microphone device (Microphone Array)
- VOSK-based speech recognition for real-time transcription
- Live transcription output to terminal as you speak
- Simple and lightweight implementation
- Automated environment setup and model download

## Prerequisites

1. **VOSK Library**: You need to install the VOSK library. The Rust crate requires the native VOSK library to be available.

   **Windows Installation (Recommended):**
   - Download VOSK from GitHub releases: https://github.com/alphacep/vosk-api/releases
   - Extract to `C:\vosk` (this is the default path the script expects)
   - Ensure `libvosk.lib` and `libvosk.dll` are present in the VOSK directory

   **Alternative Installation:**
   - Install to any custom path and specify with `-VoskPath` parameter

2. **Rust Toolchain**: Ensure you have Rust installed (version 1.70+ recommended)

## Quick Start

### Option 1: Automated Setup (Recommended)

1. **Run the build script with automatic setup**:
   ```powershell
   cd voipglot-win/tests/tts-vosk
   .\build.ps1 -SetupEnv -DownloadModel
   ```

   This will:
   - Set up all required environment variables
   - Download and extract the VOSK model
   - Build the project
   - Run the application

### Option 2: Manual Setup

1. **Download VOSK Model** (if not using automated setup):
   ```powershell
   # Create VOSK models directory
   mkdir C:\vosk\models
   
   # Download small English model
   Invoke-WebRequest -Uri "https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip" -OutFile "vosk-model-small-en-us-0.15.zip"
   
   # Extract to VOSK models directory
   Expand-Archive -Path "vosk-model-small-en-us-0.15.zip" -DestinationPath "C:\vosk\models"
   Remove-Item "vosk-model-small-en-us-0.15.zip"
   ```

2. **Build the project**:
   ```powershell
   .\build.ps1 -SetupEnv
   ```

## Build Script Usage

The enhanced build script provides several options with smart defaults:

```powershell
# Basic build with default settings (assumes C:\vosk and C:\vosk\models)
.\build.ps1

# Setup environment and download model to default location
.\build.ps1 -SetupEnv -DownloadModel

# Build with custom VOSK path
.\build.ps1 -VoskPath "D:\custom\vosk"

# Full setup with custom paths
.\build.ps1 -VoskPath "C:\vosk" -ModelPath "C:\vosk\models\vosk-model-small-en-us-0.15" -SetupEnv

# Clean build
.\build.ps1 -Clean -SetupEnv

# Download model only to default location
.\build.ps1 -DownloadModel
```

### Build Script Parameters

- `-VoskPath <path>`: Specify custom VOSK installation path
- `-ModelPath <path>`: Set VOSK model path
- `-SetupEnv`: Automatically set up environment variables
- `-DownloadModel`: Download and extract VOSK model
- `-Clean`: Clean build artifacts before building

## Usage

### List Available Audio Devices

To see all available microphone devices and their capabilities:

```powershell
cargo run --release -- --list-devices
```

This will show:
- All available input devices with their indices
- Supported sample rates and channel configurations for each device
- The default input device that will be used
- The index of the default device
- Detailed capabilities of the default device

### Run Speech-to-Text

1. **Run the application**:
   ```powershell
   cargo run --release
   ```

2. **The application will show**:
   - All available input devices (including your Microphone Array)
   - Which device is being used (with index)
   - Device capabilities and supported configurations
   - The selected audio configuration (with fallback warnings if needed)

3. **Speak into your physical microphone** and watch the real-time transcription appear in the terminal as you speak.

4. **Exit the application** by pressing `Ctrl+C`.

## Configuration

The application uses the following default settings:
- **Sample Rate**: 16kHz (required by VOSK)
- **Channels**: 1 (mono)
- **Input Device**: Default system microphone (Microphone Array)
- **VOSK Library Path**: `C:\vosk` (default)
- **Model Path**: `C:\vosk\models\vosk-model-small-en-us-0.15` (default, or set via `VOSK_MODEL_PATH`)

## Environment Variables

The build script automatically sets these environment variables:

- `LIBRARY_PATH`: Path to VOSK library files (for linking)
- `VOSK_LIB_PATH`: Explicit VOSK library path
- `INCLUDE_PATH`: Path to VOSK header files
- `VOSK_MODEL_PATH`: Path to VOSK model directory
- `PATH`: Includes VOSK directory for runtime DLL loading

## Troubleshooting

### No Input Device Found
- Ensure your microphone is connected and set as the default input device
- Check Windows audio settings

### VOSK Library Not Found
- Ensure VOSK library is installed and in your PATH
- Use `.\build.ps1 -SetupEnv` to automatically configure environment variables
- Check that `libvosk.lib` and `libvosk.dll` exist in your VOSK directory

### VOSK Model Loading Failed
- Use `.\build.ps1 -DownloadModel` to automatically download the model
- Verify the model path is correct
- Ensure the model files are properly extracted
- Check file permissions

### Audio Stream Errors
- Try running as administrator
- Check if another application is using the microphone
- Verify audio drivers are up to date
- The application now automatically finds compatible audio configurations
- If you see warnings about using non-16kHz sample rates, VOSK may not work optimally

### Build Errors
- Run `.\build.ps1 -Clean -SetupEnv` to clean and rebuild
- Ensure all environment variables are set correctly
- Check that VOSK files are in the expected location

## Dependencies

- `cpal`: Cross-platform audio I/O
- `vosk`: VOSK speech recognition bindings
- `anyhow`: Error handling
- `env_logger`: Simple logging
- `log`: Logging framework

## Notes

- This is a proof of concept implementation
- Performance may vary depending on your hardware and the VOSK model used
- The application uses simple channel communication for simplicity
- Partial results are shown with `[Partial]` prefix for debugging purposes
- The build script handles all environment setup automatically 