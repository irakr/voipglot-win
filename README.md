# VoipGlot Windows

Real-time audio translation for Windows gaming and VOIP applications using the VoipGlot Core library.

## Features

- **Cross-platform Core**: Uses voipglot-core library for audio processing and translation
- **Speech-to-Text (STT)**: Real-time speech recognition using VOSK
- **Translation**: Text translation using CTranslate2 with NLLB-200 model
- **Text-to-Speech (TTS)**: Speech synthesis using Coqui TTS
- **Offline Processing**: All AI processing happens locally, no internet required
- **Real-time Pipeline**: Low-latency audio processing pipeline
- **Multi-language Support**: Support for 200+ languages via NLLB-200
- **Windows Optimized**: Windows-specific audio optimizations and integration

## Architecture

```
Microphone → voipglot-core → Audio Output
                ↓
        [STT → Translation → TTS]
```

The Windows application is a thin wrapper around the voipglot-core library, which handles all the AI processing and audio pipeline management.

## Prerequisites

- Windows 10/11
- Rust 1.82.0 or later
- Visual Studio Build Tools (for native dependencies)
- Microphone input device
- Audio output device
- voipglot-core library (in parent directory)

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

### 4. Ensure voipglot-core is Available

The Windows application depends on the voipglot-core library. Make sure it's available in the parent directory:

```
VoipGlot/
├── voipglot-core/     # Core library (required)
└── voipglot-win/      # Windows application
```

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

### Manual Build

```powershell
# Build release version
cargo build --release --target x86_64-pc-windows-msvc

# Build fast development version
cargo build --profile fast-release --target x86_64-pc-windows-msvc
```

## Configuration

The application uses `config.toml` for configuration. The configuration structure matches the voipglot-core library:

```toml
[audio.input]
input_device = ""
sample_rate = 16000
channels = 1
buffer_size = 1024
latency_ms = 50
vb_cable_device = "CABLE Input (VB-Audio Virtual Cable)"

[audio.output]
output_device = ""
sample_rate = 48000
channels = 2
buffer_size = 2048
latency_ms = 100

[processing]
chunk_duration_ms = 100
silence_threshold = 0.01
noise_reduction = true
echo_cancellation = true
enable_feedback_prevention = true
tts_silence_buffer_ms = 50
tts_queue_size = 3

[stt]
provider = "vosk"
model_path = "../voipglot-core/models/vosk-model-en-in-0.5"
sample_rate = 16000.0
enable_partial_results = true

[translation]
provider = "ct2"
model_path = "../voipglot-core/models/nllb-200-ct2"
source_language = "eng_Latn"
target_language = "eng_Latn"
num_threads = 4
device = "cpu"
max_batch_size = 32
beam_size = 4

[tts]
provider = "coqui"
model_path = "tts_models/en/ljspeech/fast_pitch"
voice_speed = 1.0
voice_pitch = 1.0
enable_gpu = false
synthesis_timeout_secs = 5
```

## Usage

### Basic Usage

```powershell
# Run with default configuration
.\target\x86_64-pc-windows-msvc\release\voipglot-win.exe

# Run with custom configuration
.\target\x86_64-pc-windows-msvc\release\voipglot-win.exe -c my_config.toml

# Run with debug logging
.\target\x86_64-pc-windows-msvc\release\voipglot-win.exe --debug

# List available audio devices
.\target\x86_64-pc-windows-msvc\release\voipglot-win.exe --list-devices
```

### Command Line Options

- `-c, --config <path>`: Configuration file path (default: config.toml)
- `--debug`: Enable debug logging
- `--list-devices`: List available audio input/output devices
- `--source-lang <lang>`: Source language code (e.g., "en", "fr", "de")
- `--target-lang <lang>`: Target language code (e.g., "en", "fr", "de")
- `--sample-rate <rate>`: Audio sample rate in Hz (default: 16000)
- `--channels <count>`: Audio channels (1 for mono, 2 for stereo, default: 1)
- `--buffer-size <size>`: Audio buffer size in samples (default: 1024)
- `--latency-ms <ms>`: Target latency in milliseconds (default: 10)
- `--silence-threshold <value>`: Silence threshold for voice detection (default: 0.01)
- `--chunk-duration-ms <ms>`: Audio chunk duration in milliseconds (default: 1000)

## What voipglot-win Does NOT Handle

Since voipglot-win uses the voipglot-core library, it does not need to worry about:

### ❌ **AI Model Management**
- **Model files**: All AI models (VOSK, CTranslate2, Coqui TTS) are managed by voipglot-core
- **Model downloads**: Model acquisition and setup is handled by voipglot-core
- **Model updates**: Model versioning and updates are managed centrally

### ❌ **AI Library Dependencies**
- **Audio processing libraries**: cpal, symphonia, dasp are handled by voipglot-core
- **STT libraries**: VOSK bindings and integration are in voipglot-core
- **Translation libraries**: CTranslate2 integration is in voipglot-core
- **TTS libraries**: Coqui TTS bindings are in voipglot-core

### ❌ **Core Pipeline Logic**
- **Audio pipeline**: Audio capture, processing, and playback logic is in voipglot-core
- **Translation pipeline**: STT → Translation → TTS flow is managed by voipglot-core
- **Error handling**: Core error handling and recovery is in voipglot-core

## What voipglot-win DOES Handle

### ✅ **Windows-Specific Integration**
- **Platform-specific audio device management**
- **Windows user experience and interface**
- **Windows-specific configuration and settings**
- **Integration with Windows gaming/VOIP applications**

### ✅ **Application-Level Features**
- **Command-line interface and argument parsing**
- **Configuration file management**
- **Logging and debugging for Windows environment**
- **User interaction and feedback**

## Model Management

Models are managed by the voipglot-core library. The Windows application doesn't handle model downloads directly. Models should be available in the paths specified in the configuration file.

### Required Models (Managed by voipglot-core)

- **VOSK Model**: For speech recognition (specified in `[stt].model_path`)
- **CTranslate2 Model**: For translation (specified in `[translation].model_path`)
- **Coqui TTS Model**: For speech synthesis (specified in `[tts].model_path`)

## Troubleshooting

### Common Issues

1. **voipglot-core not found**: Ensure the voipglot-core library is in the parent directory
2. **Audio device issues**: Use `--list-devices` to see available devices
3. **Model not found**: Check that model paths in config.toml are correct
4. **Build errors**: Try `.\build.ps1 -Clean` to clean and rebuild

### Logs

The application creates logs in `voipglot-win.log` in the current directory. Enable debug logging with `--debug` for more detailed information.

## Development

### Project Structure

```
voipglot-win/
├── src/
│   └── main.rs              # Main application entry point
├── config.toml              # Configuration file
├── build.ps1                # Build script
├── Cargo.toml               # Rust dependencies
└── README.md                # This file
```

### Adding Features

Since this application uses voipglot-core, new features should be implemented in the core library rather than here. This application focuses on Windows-specific integration and user experience.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 