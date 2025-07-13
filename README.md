# VoipGlot Windows - Real-time Audio Translation

VoipGlot Windows is a real-time audio translation application designed for gaming and VOIP applications. It provides offline speech-to-text, translation, and text-to-speech capabilities using local models.

## Features

- **Offline Processing**: All components work locally without requiring internet connectivity
- **Real-time Translation**: Low-latency audio processing pipeline
- **VB Cable Support**: Designed to work with VB-Audio Virtual Cable for seamless integration
- **Multiple Language Support**: Supports 200+ languages through NLLB-200 model
- **Configurable Audio**: Flexible audio device configuration

## Architecture

The application implements the following pipeline:

```
(Microphone) → [VoipGlot] → [VOSK STT] → [CT2 Translation] → [Custom TTS] → (VB Cable Output)
```

### Components

1. **VOSK Speech-to-Text**: Offline speech recognition using VOSK models
2. **CTranslate2 Translation**: Fast neural machine translation using NLLB-200
3. **Custom Text-to-Speech**: Simple audio synthesis (placeholder for future TTS engines)
4. **Audio Processing**: Real-time audio capture and playback with VB Cable support

## Prerequisites

### Required Software

1. **VB-Audio Virtual Cable**: Download and install from [VB-Audio](https://vb-audio.com/Cable/)
2. **Visual Studio 2022**: For building the Rust application
3. **Rust Toolchain**: Latest stable Rust version

### Required Models

1. **VOSK Model**: `vosk-model-small-en-us-0.15` (automatically downloaded)
2. **CT2 Model**: NLLB-200 model converted to CT2 format (manual download required)

## Installation

### 1. Clone the Repository

```bash
git clone <repository-url>
cd voipglot-win
```

### 2. Install VB-Audio Virtual Cable

1. Download VB-Audio Virtual Cable from [VB-Audio](https://vb-audio.com/Cable/)
2. Install the application
3. Verify installation by checking Windows audio devices

### 3. Build the Application

#### Using the Integrated Build Script (Recommended)

```powershell
# Open Developer PowerShell and run:
.\build-integrated.ps1 -DownloadModels -Release
```

#### Manual Build

```powershell
# Download VOSK model
mkdir models
# Download vosk-model-small-en-us-0.15.zip and extract to models/

# Download CT2 model
# Download NLLB-200 CT2 model and extract to models/nllb-200-ct2/

# Build the application
cargo build --release
```

### 4. Configure Audio Devices

1. Set your microphone as the default input device
2. Set VB Cable Input as the default output device in your VOIP application
3. Configure VoipGlot to output to VB Cable Output

## Configuration

The application uses `config.toml` for configuration:

```toml
[audio]
input_device = ""  # Leave empty for default microphone
output_device = ""  # Leave empty for default output
sample_rate = 16000
channels = 1
buffer_size = 1024
vb_cable_device = "CABLE Input (VB-Audio Virtual Cable)"

[stt]
provider = "vosk"
model_path = "models/vosk-model-small-en-us-0.15"
sample_rate = 16000.0
enable_partial_results = true

[translation]
provider = "ct2"
model_path = "models/nllb-200-ct2"
source_language = "en"
target_language = "es"
num_threads = 4
device = "cpu"
max_batch_size = 32
beam_size = 4

[tts]
provider = "custom"
sample_rate = 22050
channels = 1
voice_speed = 1.0
voice_pitch = 1.0

[processing]
chunk_duration_ms = 1000
silence_threshold = 0.01
noise_reduction = true
echo_cancellation = true

[logging]
level = "info"
format = "simple"
log_file = "voipglot.log"
```

## Usage

### Basic Usage

```bash
# List available audio devices
voipglot-win.exe --list-devices

# Run with default settings (English to Spanish)
voipglot-win.exe

# Run with debug logging
voipglot-win.exe --debug

# Run with custom languages
voipglot-win.exe --source-lang en --target-lang fr
```

### Command Line Options

- `--config <file>`: Specify configuration file (default: config.toml)
- `--debug`: Enable debug logging
- `--list-devices`: List available audio devices
- `--source-lang <lang>`: Source language code (default: en)
- `--target-lang <lang>`: Target language code (default: es)

### Supported Languages

The application supports 200+ languages through the NLLB-200 model. Common language codes:

- `en`: English
- `es`: Spanish
- `fr`: French
- `de`: German
- `it`: Italian
- `pt`: Portuguese
- `ru`: Russian
- `ja`: Japanese
- `ko`: Korean
- `zh`: Chinese

## Troubleshooting

### Common Issues

1. **VB Cable Device Not Found**
   - Ensure VB-Audio Virtual Cable is properly installed
   - Check Windows audio devices in Sound settings
   - Restart the application after installation

2. **Models Not Found**
   - Verify model paths in config.toml
   - Use `--list-devices` to check audio device names
   - Download missing models using the build script

3. **Audio Quality Issues**
   - Adjust `silence_threshold` in config.toml
   - Check audio device sample rates
   - Ensure proper audio device configuration

4. **High Latency**
   - Reduce `chunk_duration_ms` in config.toml
   - Increase `buffer_size` for better performance
   - Use release build for better performance

### Debug Information

Enable debug logging to get detailed information:

```bash
voipglot-win.exe --debug
```

Check the log file (`voipglot.log`) for detailed error messages and processing information.

## Development

### Project Structure

```
voipglot-win/
├── src/
│   ├── audio/          # Audio capture and playback
│   ├── translation/    # STT, translation, and TTS components
│   ├── config.rs       # Configuration management
│   ├── error.rs        # Error handling
│   └── main.rs         # Main application entry point
├── tests/              # Individual component tests
├── models/             # Model files (VOSK, CT2)
├── config.toml         # Configuration file
└── build-integrated.ps1 # Build script
```

### Building Individual Components

Each component can be tested individually in the `tests/` directory:

```bash
# Test VOSK STT
cd tests/stt-vosk
cargo run

# Test CT2 Translation
cd tests/translation-ct2
cargo run

# Test TTS
cd tests/tts-coqui
cargo run
```

### Adding New Features

1. **New TTS Engine**: Implement in `src/translation/tts.rs`
2. **New STT Engine**: Implement in `src/translation/stt.rs`
3. **New Translation Engine**: Implement in `src/translation/translator_api.rs`
4. **Audio Processing**: Extend `src/audio/processing.rs`

## Performance

### Optimization Tips

1. **Use Release Build**: Always use `--release` for production
2. **Adjust Thread Count**: Set appropriate `num_threads` for your CPU
3. **Optimize Buffer Sizes**: Balance latency vs. stability
4. **Use GPU**: Set `device = "cuda"` if you have a compatible GPU

### Expected Performance

- **Latency**: 1-3 seconds end-to-end (depending on audio chunk size)
- **CPU Usage**: 10-30% on modern CPUs
- **Memory Usage**: 2-4 GB (mainly for models)
- **Accuracy**: Varies by language and audio quality

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [VOSK](https://alphacephei.com/vosk/) for offline speech recognition
- [CTranslate2](https://github.com/OpenNMT/CTranslate2) for fast translation
- [VB-Audio](https://vb-audio.com/) for virtual audio cable
- [NLLB-200](https://ai.meta.com/research/no-language-left-behind/) for multilingual translation 