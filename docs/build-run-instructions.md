# VoipGlot Build, Run, and Configuration Instructions

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Building the Application](#building-the-application)
3. [Running the Application](#running-the-application)
4. [Language Configuration](#language-configuration)
5. [Audio Device Setup](#audio-device-setup)
6. [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Software
1. **Rust Toolchain** (Latest stable)
   ```powershell
   # Install Rust from https://rustup.rs/
   # Or run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Visual Studio 2022** (with C++ build tools)
   - Download from Microsoft Visual Studio
   - Install "Desktop development with C++" workload

3. **VB-Audio Virtual Cable**
   - Download from: https://vb-audio.com/Cable/
   - Install and restart your computer

4. **Python 3.7+** (for Coqui TTS)
   - Download from https://python.org
   - Ensure `python` is in your PATH

### Required Models
- **VOSK Model**: `vosk-model-small-en-us-0.15` (auto-downloaded)
- **CT2 Model**: NLLB-200 CT2 format (manual download required)
- **Coqui TTS Model**: `tts_models/en/ljspeech/fast_pitch` (auto-downloaded and cached)

## Building the Application

### Method 1: Build Script (Recommended)

```powershell
# Open Developer PowerShell as Administrator
# Navigate to voipglot-win directory
cd voipglot-win

# Build with automatic model download
.\build.ps1 -DownloadModels -Release

# Or build without downloading models (if you have them)
.\build.ps1 -Release
```

### Method 2: Manual Build

```powershell
# 1. Download VOSK model
mkdir models
# Download vosk-model-small-en-us-0.15.zip from https://alphacephei.com/vosk/models/
# Extract to models/vosk-model-small-en-us-0.15/

# 2. Download CT2 model
# Download NLLB-200 CT2 model and extract to models/nllb-200-ct2/

# 3. Setup Coqui TTS
python scripts/setup-coqui.py

# 4. Build the application
cargo build --release
```

### Build Options

```powershell
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, faster runtime)
cargo build --release

# Build with specific features
cargo build --release --features "full"

# Clean and rebuild
cargo clean
cargo build --release
```

## Running the Application

### Basic Usage

```powershell
# List available audio devices
.\target\release\voipglot-win.exe --list-devices

# Run with default settings (English to Spanish)
.\target\release\voipglot-win.exe

# Run with debug logging
.\target\release\voipglot-win.exe --debug

# Run with custom configuration file
.\target\release\voipglot-win.exe --config my-config.toml
```

### Command Line Options

```powershell
# Available options
.\target\release\voipglot-win.exe --help

# Language configuration
.\target\release\voipglot-win.exe --source-lang en --target-lang fr

# Debug mode with custom config
.\target\release\voipglot-win.exe --debug --config debug-config.toml --source-lang en --target-lang de
```

### Command Line Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `--config <file>` | Configuration file path | `config.toml` |
| `--debug` | Enable debug logging | `false` |
| `--list-devices` | List available audio devices | - |
| `--source-lang <lang>` | Source language code | `en` |
| `--target-lang <lang>` | Target language code | `es` |

## Language Configuration

### Supported Languages

The application supports **200+ languages** through the NLLB-200 model. Here are the most common ones:

#### Speech-to-Text (VOSK) Languages
- **English**: `en` (vosk-model-small-en-us-0.15)
- **Spanish**: `es` (vosk-model-small-es-0.42)
- **French**: `fr` (vosk-model-small-fr-0.22)
- **German**: `de` (vosk-model-small-de-0.15)
- **Italian**: `it` (vosk-model-small-it-0.22)
- **Portuguese**: `pt` (vosk-model-small-pt-0.3)
- **Russian**: `ru` (vosk-model-small-ru-0.22)
- **Japanese**: `ja` (vosk-model-small-ja-0.22)
- **Korean**: `ko` (vosk-model-small-ko-0.22)
- **Chinese**: `zh` (vosk-model-small-cn-0.22)

#### Translation Languages (NLLB-200)
All 200+ languages supported by NLLB-200, including:
- **Arabic**: `ar`
- **Hindi**: `hi`
- **Dutch**: `nl`
- **Polish**: `pl`
- **Turkish**: `tr`
- **Swedish**: `sv`
- **Danish**: `da`
- **Norwegian**: `no`
- **Finnish**: `fi`
- **Czech**: `cs`
- **Slovak**: `sk`
- **Hungarian**: `hu`
- **Romanian**: `ro`
- **Bulgarian**: `bg`
- **Croatian**: `hr`
- **Serbian**: `sr`
- **Slovenian**: `sl`
- **Estonian**: `et`
- **Latvian**: `lv`
- **Lithuanian**: `lt`

#### Text-to-Speech (Coqui TTS) Languages
- **English**: `en` (tts_models/en/ljspeech/fast_pitch)
- **Spanish**: `es` (tts_models/es/css10/vits)
- **French**: `fr` (tts_models/fr/css10/vits)
- **German**: `de` (tts_models/de/css10/vits)
- **Italian**: `it` (tts_models/it/css10/vits)
- **Portuguese**: `pt` (tts_models/pt/css10/vits)
- **Russian**: `ru` (tts_models/ru/css10/vits)
- **Japanese**: `ja` (tts_models/ja/css10/vits)
- **Korean**: `ko` (tts_models/ko/css10/vits)
- **Chinese**: `zh` (tts_models/zh/css10/vits)

### Language Configuration Methods

#### Method 1: Command Line (Temporary)

```powershell
# English to French
.\target\release\voipglot-win.exe --source-lang en --target-lang fr

# Spanish to German
.\target\release\voipglot-win.exe --source-lang es --target-lang de

# Japanese to English
.\target\release\voipglot-win.exe --source-lang ja --target-lang en
```

#### Method 2: Configuration File (Permanent)

Edit `config.toml`:

```toml
[translation]
source_language = "en"  # Change this
target_language = "fr"  # Change this
```

#### Method 3: Environment Variables

```powershell
# Set environment variables
$env:VOIPGLOT_SOURCE_LANG = "en"
$env:VOIPGLOT_TARGET_LANG = "fr"

# Run application
.\target\release\voipglot-win.exe
```

### Language-Specific Model Configuration

#### VOSK Model Configuration

For different STT languages, update the model path in `config.toml`:

```toml
[stt]
model_path = "models/vosk-model-small-fr-0.22"  # For French
# or
model_path = "models/vosk-model-small-de-0.15"  # For German
```

#### TTS Model Configuration

The TTS automatically switches models based on target language, but you can override:

```toml
[tts]
model_path = "tts_models/fr/css10/vits"  # Force French TTS model
```

### Advanced Language Configuration

#### Custom Language Pairs

Create a custom configuration file for specific language pairs:

```toml
# french-to-german.toml
[audio]
input_device = ""
output_device = "CABLE Input (VB-Audio Virtual Cable)"
sample_rate = 16000
channels = 1

[stt]
provider = "vosk"
model_path = "models/vosk-model-small-fr-0.22"
sample_rate = 16000.0
enable_partial_results = true

[translation]
provider = "ct2"
model_path = "models/nllb-200-ct2"
source_language = "fr"
target_language = "de"
num_threads = 4
device = "cpu"

[tts]
provider = "coqui"
model_path = "tts_models/de/css10/vits"
sample_rate = 22050
channels = 1
voice_speed = 1.0
voice_pitch = 1.0
enable_gpu = false
```

Run with custom config:
```powershell
.\target\release\voipglot-win.exe --config french-to-german.toml
```

## Audio Device Setup

### 1. VB-Audio Virtual Cable Setup

1. **Install VB-Audio Virtual Cable**
   - Download from https://vb-audio.com/Cable/
   - Install and restart your computer

2. **Verify Installation**
   ```powershell
   .\target\release\voipglot-win.exe --list-devices
   ```
   Look for:
   - `CABLE Input (VB-Audio Virtual Cable)` (Input device)
   - `CABLE Output (VB-Audio Virtual Cable)` (Output device)

### 2. Audio Device Configuration

#### Windows Audio Settings
1. Right-click speaker icon → "Open Sound settings"
2. Under "Input", select your microphone
3. Under "Output", select "CABLE Input (VB-Audio Virtual Cable)"

#### Application Configuration
Edit `config.toml`:

```toml
[audio]
input_device = "Your Microphone Name"  # Leave empty for default
output_device = "CABLE Input (VB-Audio Virtual Cable)"
sample_rate = 16000
channels = 1
buffer_size = 2048
latency_ms = 100
vb_cable_device = "CABLE Input (VB-Audio Virtual Cable)"
```

### 3. VOIP Application Setup

Configure your VOIP application (Discord, Teams, etc.):
- **Input Device**: Your real microphone
- **Output Device**: `CABLE Input (VB-Audio Virtual Cable)`

## Troubleshooting

### Common Issues

#### 1. VB Cable Device Not Found
```powershell
# Check if VB Cable is installed
.\target\release\voipglot-win.exe --list-devices

# If not found, reinstall VB-Audio Virtual Cable
# Restart computer after installation
```

#### 2. Models Not Found
```powershell
# Check model paths
ls models/
ls tts_models/

# Download missing models
.\build.ps1 -DownloadModels -Release
```

#### 3. Audio Quality Issues
Edit `config.toml`:
```toml
[processing]
chunk_duration_ms = 500   # Reduce for lower latency
silence_threshold = 0.01  # Adjust for better speech detection
noise_reduction = true
echo_cancellation = true
```

#### 4. High Latency
```toml
[audio]
buffer_size = 1024  # Reduce buffer size
latency_ms = 50     # Reduce latency

[processing]
chunk_duration_ms = 300  # Reduce chunk duration
```

#### 5. Translation Quality Issues
```toml
[translation]
num_threads = 8        # Increase for better performance
max_batch_size = 64    # Increase batch size
beam_size = 8          # Increase beam size for better quality
```

#### 6. TTS Voice Issues
```toml
[tts]
voice_speed = 0.9      # Slow down speech
voice_pitch = 1.1      # Adjust pitch
enable_gpu = true      # Enable GPU if available
```

### Debug Mode

```powershell
# Run with debug logging
.\target\release\voipglot-win.exe --debug

# Check log file
Get-Content voipglot.log -Tail 50
```

### Performance Optimization

#### For Better Performance:
```toml
[audio]
buffer_size = 1024
latency_ms = 50

[processing]
chunk_duration_ms = 300
silence_threshold = 0.02

[translation]
num_threads = 8
max_batch_size = 64
device = "cuda"  # If GPU available

[tts]
enable_gpu = true  # If GPU available
```

#### For Better Quality:
```toml
[translation]
beam_size = 8
max_batch_size = 32

[processing]
chunk_duration_ms = 1000
silence_threshold = 0.005
```

## Quick Start Guide

### 1. First Time Setup
```powershell
# 1. Install prerequisites (Rust, Visual Studio, VB Cable)
# 2. Clone repository
git clone <repository-url>
cd voipglot-win

# 3. Build with models
.\build.ps1 -DownloadModels -Release

# 4. List devices
.\target\release\voipglot-win.exe --list-devices

# 5. Run with default settings
.\target\release\voipglot-win.exe
```

### 2. Change Languages
```powershell
# English to French
.\target\release\voipglot-win.exe --source-lang en --target-lang fr

# Spanish to German
.\target\release\voipglot-win.exe --source-lang es --target-lang de
```

### 3. Custom Configuration
```powershell
# Create custom config
Copy-Item config.toml my-config.toml
# Edit my-config.toml with your preferences

# Run with custom config
.\target\release\voipglot-win.exe --config my-config.toml
```

## Support

For additional help:
1. Check the log file: `voipglot.log`
2. Run with debug mode: `--debug`
3. Verify audio devices: `--list-devices`
4. Check model installation
5. Review configuration file syntax 