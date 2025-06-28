# VoipGlot Windows

Real-time audio translation for Windows gaming and VOIP applications.

## Overview

VoipGlot Windows captures audio from your real microphone, translates it in real-time, and outputs the translated audio to a virtual microphone that other applications can use as input.

### High-Level Flow

```
[Real Mic] → [VoipGlot] → [STT] → [Translation] → [TTS] → [Virtual Mic] → [Game/Discord/Zoom]
```

## Features

- **Real-time Audio Processing**: Low-latency audio capture and playback
- **Multiple AI Providers**: Support for Whisper, Azure, Google, DeepL, and ElevenLabs
- **Cross-Platform Audio**: Built with CPAL for reliable audio handling
- **Configurable**: Easy configuration via TOML files and environment variables
- **Multiple Languages**: Support for 10+ languages

## Prerequisites

### Windows Requirements
- Windows 10/11 (64-bit)
- Rust 1.70+ with Cargo
- Visual Studio Build Tools 2019 or later
- VB-CABLE Virtual Audio Device (for virtual microphone output)

### Audio Setup
1. Install [VB-CABLE Virtual Audio Device](https://vb-audio.com/Cable/)
2. Set VB-CABLE as your default output device in VoipGlot
3. Configure your target application (game, Discord, etc.) to use VB-CABLE as input

## Installation

### 1. Clone the Repository
```bash
git clone <repository-url>
cd voipglot-win
```

### 2. Install Dependencies
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Windows build tools
rustup target add x86_64-pc-windows-msvc
```

### 3. Configure API Keys
Set the following environment variables:

```bash
# For DeepL translation
export DEEPL_API_KEY="your-deepl-api-key"

# For Azure Speech Services
export AZURE_SPEECH_KEY="your-azure-speech-key"
export AZURE_REGION="eastus"

# For ElevenLabs TTS
export ELEVENLABS_API_KEY="your-elevenlabs-api-key"

# For Google APIs
export GOOGLE_API_KEY="your-google-api-key"
```

### 4. Build the Application
```bash
cargo build --release
```

## Usage

### Basic Usage
```bash
# Run with default settings (English to Spanish)
cargo run --release

# Run with custom languages
cargo run --release -- --source-lang en --target-lang fr

# Run with custom config file
cargo run --release -- --config my-config.toml

# Enable debug logging
cargo run --release -- --debug
```

### Command Line Options
- `--config <file>`: Configuration file path (default: config.toml)
- `--debug`: Enable debug logging
- `--source-lang <lang>`: Source language code (default: en)
- `--target-lang <lang>`: Target language code (default: es)

### Configuration
Edit `config.toml` to customize:
- Audio devices and settings
- Translation providers
- Processing parameters
- API endpoints

## Supported Languages

### Speech-to-Text
- English (en)
- Spanish (es)
- French (fr)
- German (de)
- Italian (it)
- Portuguese (pt)
- Russian (ru)
- Japanese (ja)
- Korean (ko)
- Chinese (zh)

### Translation
All STT languages plus:
- Arabic (ar)
- Hindi (hi)

## AI Providers

### Speech-to-Text
- **Whisper**: Offline processing, high accuracy
- **Azure Speech Services**: Cloud-based, real-time
- **Google Speech-to-Text**: Cloud-based, wide language support

### Translation
- **DeepL**: High-quality translations
- **Google Translate**: Wide language support
- **Azure Translator**: Microsoft's translation service

### Text-to-Speech
- **Azure Speech Services**: Natural-sounding voices
- **ElevenLabs**: High-quality, customizable voices
- **Google Text-to-Speech**: Wide language support

## Architecture

### Core Components
- **AudioManager**: Orchestrates audio capture and playback
- **AudioCapture**: Handles real microphone input
- **AudioPlayback**: Outputs to virtual microphone
- **AudioProcessor**: Manages the translation pipeline
- **Translator**: Coordinates STT, translation, and TTS

### Audio Pipeline
1. **Capture**: Real-time audio from microphone
2. **Preprocessing**: Noise reduction, silence detection
3. **STT**: Convert speech to text
4. **Translation**: Translate text to target language
5. **TTS**: Convert translated text to speech
6. **Playback**: Output to virtual microphone

## Development

### Project Structure
```
voipglot-win/
├── src/
│   ├── main.rs              # Application entry point
│   ├── error.rs             # Error handling
│   ├── config.rs            # Configuration management
│   ├── audio/               # Audio processing modules
│   │   ├── mod.rs
│   │   ├── capture.rs       # Audio capture
│   │   ├── playback.rs      # Audio playback
│   │   └── processing.rs    # Audio processing pipeline
│   └── translation/         # AI translation modules
│       ├── mod.rs
│       ├── stt.rs           # Speech-to-text
│       ├── translator_api.rs # Text translation
│       └── tts.rs           # Text-to-speech
├── Cargo.toml               # Rust dependencies
├── config.toml              # Configuration file
└── README.md               # This file
```

### Building for Development
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check for issues
cargo clippy
```

## Troubleshooting

### Common Issues

#### Audio Device Not Found
- Ensure your microphone is properly connected and recognized by Windows
- Check Windows Sound settings
- Try running with `--debug` to see available devices

#### High Latency
- Reduce `chunk_duration_ms` in config.toml
- Use lower latency audio devices
- Enable hardware acceleration if available

#### Translation Errors
- Verify API keys are correctly set
- Check internet connectivity
- Ensure language codes are supported by your chosen provider

#### Virtual Microphone Not Working
- Verify VB-CABLE is properly installed
- Check that VB-CABLE is selected as output device
- Restart target applications after changing audio settings

### Debug Mode
Run with `--debug` flag to get detailed logging:
```bash
cargo run --release -- --debug
```

## Performance Optimization

### For Gaming
- Use `chunk_duration_ms = 500` for lower latency
- Enable `noise_reduction = true`
- Use Whisper for offline STT to reduce network dependency

### For VOIP Applications
- Use `chunk_duration_ms = 1000` for better quality
- Enable `echo_cancellation = true`
- Use Azure or Google for cloud-based processing

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and questions:
1. Check the troubleshooting section
2. Search existing issues
3. Create a new issue with detailed information

## Roadmap

- [ ] GUI interface
- [ ] Real-time voice cloning
- [ ] Offline translation models
- [ ] Multi-language simultaneous translation
- [ ] Custom voice training
- [ ] Plugin system for additional providers 