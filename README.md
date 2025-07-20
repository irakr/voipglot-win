# VoipGlot Windows

Real-time audio translation for Windows gaming and VOIP applications using offline AI models.

## Features

- **Speech-to-Text (STT)**: Real-time speech recognition using VOSK
- **Translation**: Text translation using CTranslate2 with NLLB-200 model
- **Text-to-Speech (TTS)**: Speech synthesis using Coqui TTS
- **Offline Processing**: All AI processing happens locally, no internet required
- **Real-time Pipeline**: Low-latency audio processing pipeline
- **Multi-language Support**: Support for 200+ languages via NLLB-200

## Architecture

```
Microphone → STT (VOSK) → Translation (CTranslate2) → TTS (Coqui) → Audio Output
```

## Prerequisites

- Windows 10/11
- Rust 1.82.0 or later
- Visual Studio Build Tools (for native dependencies)
- Microphone input device
- Audio output device

## Installation

### 1. Clone the Repository

```powershell
git clone <repository-url>
cd voipglot-win
```

### 2. Install Rust

If you don't have Rust installed, download and install it from [https://rustup.rs/](https://rustup.rs/)

### 3. Install Visual Studio Build Tools

Download and install Visual Studio Build Tools from Microsoft. Make sure to include:
- MSVC v143 build tools
- Windows 10/11 SDK
- CMake tools

### 4. Download AI Models

The build script can automatically download and manage AI models for you:

#### Automatic Download (Recommended)
```powershell
# Download required models only (VOSK for STT)
.\build.ps1 -DownloadModels

# Download all models (VOSK, CTranslate2, Coqui TTS)
.\build.ps1 -ForceDownload
```

#### Manual Download (Alternative)
If you prefer to download models manually:

##### VOSK Model (Required for STT)
1. Download a VOSK model from [https://alphacephei.com/vosk/models](https://alphacephei.com/vosk/models)
2. Extract to `models/vosk-model-small-en-us-0.15`

##### CTranslate2 Model (Optional for Translation)
1. Download NLLB-200 model from [https://huggingface.co/facebook/nllb-200-3.3B](https://huggingface.co/facebook/nllb-200-3.3B)
2. Extract to `models/nllb-200-ct2`

##### Coqui TTS Model (Optional for TTS)
1. Download Coqui TTS model from [https://huggingface.co/coqui/XTTS-v2](https://huggingface.co/coqui/XTTS-v2)
2. Extract to `models/coqui-tts-model`

## Building

### Using the Build Script (Recommended)

```powershell
# Open Developer PowerShell for VS
.\build.ps1
```

For fast development builds:
```powershell
.\build.ps1 -Fast
```

For clean build:
```powershell
.\build.ps1 -Clean
```

For downloading required models:
```powershell
.\build.ps1 -DownloadModels
```

For downloading all missing models (including optional ones):
```powershell
.\build.ps1 -ForceDownload
```

Skip clippy for faster builds:
```powershell
.\build.ps1 -Fast -NoClippy
```

Combine options:
```powershell
.\build.ps1 -Fast -DownloadModels
```

### Manual Build

```powershell
# Open Developer PowerShell for VS
cargo build --release
```

## Configuration

Edit `config.toml` to configure the application:

```toml
[audio]
sample_rate = 16000
channels = 1
buffer_size = 1024
latency_ms = 50
silence_threshold = 0.01

[stt]
vosk_model_path = "models/vosk-model-en-us-0.22"

[translation]
ct2_model_path = "models/nllb-200-3.3B"
source_language = "eng_Latn"
target_language = "spa_Latn"

[tts]
coqui_model_path = "models/coqui-tts-model"
speaker_id = 0
```

## Usage

### Basic Usage

```powershell
# Run the application
.\target\release\voipglot-win.exe
```

### Command Line Options

```powershell
# List available audio devices
.\target\release\voipglot-win.exe --list-devices

# Enable debug logging
.\target\release\voipglot-win.exe --debug

# Specify custom configuration file
.\target\release\voipglot-win.exe --config my-config.toml

# Override language settings
.\target\release\voipglot-win.exe --source-lang eng_Latn --target-lang fra_Latn
```

### Testing STT Only

For testing speech recognition without translation and TTS:

1. Make sure you have a VOSK model downloaded
2. Run the application
3. Speak into your microphone
4. Transcribed text will appear in the console and log file

## Current Status

### ✅ Implemented and Tested
- **Audio Pipeline**: Complete audio capture, processing, and playback infrastructure
- **STT Module**: VOSK speech recognition with real-time audio capture
- **Translation Module**: CTranslate2 integration with NLLB-200 model
- **TTS Module**: Coqui TTS integration for speech synthesis
- **Configuration System**: Flexible configuration management
- **Logging System**: Comprehensive logging with file and console output

### 🚧 In Progress
- **Performance Optimization**: Fine-tuning the complete pipeline for lower latency
- **Language Support**: Testing and validation of additional language pairs
- **Virtual Microphone**: Integration with VB-CABLE for application output

### 📋 Next Steps
1. Optimize end-to-end latency in the translation pipeline
2. Add support for more VOSK language models
3. Improve audio preprocessing and noise reduction
4. Enhance virtual microphone integration
5. Add user interface for language selection and settings

## Troubleshooting

### Build Issues
- Ensure you're using Developer PowerShell for VS
- Make sure Visual Studio Build Tools are installed
- Check that Rust is up to date: `rustup update`

### Audio Issues
- Use `--list-devices` to see available audio devices
- Check microphone permissions in Windows settings
- Ensure microphone is set as default input device

### Model Issues
- Verify model paths in config.toml
- Check that models are properly extracted
- Ensure sufficient disk space for models

## Development

### Project Structure
```
voipglot-win/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error handling
│   ├── audio/               # Audio processing modules
│   │   ├── mod.rs
│   │   ├── capture.rs       # Audio capture
│   │   ├── playback.rs      # Audio playback
│   │   └── processing.rs    # Audio processing
│   └── translation/         # AI translation modules
│       ├── mod.rs
│       ├── stt.rs           # Speech-to-text (VOSK)
│       ├── translator_api.rs # Text translation (CTranslate2)
│       └── tts.rs           # Text-to-speech (Coqui TTS)
├── tests/                   # Individual component tests
│   ├── stt-vosk/           # ✅ VOSK speech recognition
│   ├── translation-ct2/     # ✅ CTranslate2 translation
│   └── tts-coqui/          # ✅ Coqui TTS synthesis
├── models/                  # AI model directory
├── config.toml              # Configuration file
├── build.ps1                # Build script
└── README.md               # This file
```

### Testing Individual Components
Each component has been tested individually in the `tests/` directory:
- `tests/stt-vosk/`: VOSK speech recognition
- `tests/translation-ct2/`: CTranslate2 translation
- `tests/tts-coqui/`: Coqui TTS synthesis

## License

[Add your license information here]

## Contributing

[Add contribution guidelines here] 