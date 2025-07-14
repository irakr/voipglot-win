
# TTS-Coqui Test Application

This is a proof-of-concept application demonstrating real-time Text-to-Speech (TTS) using the Coqui TTS Rust bindings. The application accepts text input from the user via console and plays the synthesized speech through a configurable audio output device.

## Prerequisites

1. Rust toolchain (2021 edition or later)
2. Coqui TTS models (optional, will use default if not specified)
3. Working audio output device

## Building the Application

```bash
cargo build
```

## Running the Application

1. Basic usage:
```bash
cargo run
```

2. List available audio devices:
```bash
cargo run -- --list-devices
```

## Configuration

The application uses `config.toml` for configuration:

```toml
[audio]
input_device = ""  # Input device name (optional)
output_device = ""  # Output device name (leave empty for default)
sample_rate = 22050  # Sample rate in Hz
channels = 1  # Number of audio channels
buffer_size = 2048  # Audio buffer size

[tts]
model_path = ""  # Path to Coqui TTS model (optional)
voice_speed = 1.0  # Voice speed multiplier
voice_pitch = 1.0  # Voice pitch multiplier
enable_gpu = false  # Enable GPU acceleration

[logging]
level = "info"  # Log level
```

## Usage

1. Run the application
2. The program will initialize the Coqui TTS synthesizer and connect to your configured audio output device
3. Type text at the prompt and press Enter
4. The application will convert your text to speech and play it through the selected audio device
5. Press Ctrl+C to exit

## Features

- Real-time text-to-speech synthesis using coqui-tts crate
- Configurable audio input/output devices
- Direct access to audio buffers for custom routing
- Configurable sample rates and audio settings
- GPU acceleration support (optional)
- Error handling and logging

## Audio Device Selection

The application supports selecting specific audio devices:

1. Use `--list-devices` to see available output devices
2. Set the `output_device` in `config.toml` to use a specific device
3. Leave `output_device` empty to use the system default

## Troubleshooting

1. No audio output:
   - Check if your audio device is properly connected
   - Use `--list-devices` to verify available audio devices
   - Check system volume settings
   - Verify the device name in config.toml matches exactly

2. TTS initialization fails:
   - Ensure coqui-tts crate is properly installed
   - Check if required language models are available
   - Verify model_path in config.toml if using custom models

3. Audio buffer issues:
   - Check sample rate compatibility with your audio device
   - Adjust buffer_size in config.toml if experiencing audio glitches

## Technical Details

- Uses the `coqui-tts` crate for speech synthesis
- Implements custom audio buffer streaming using CPAL
- Supports real-time audio routing to any output device
- Provides direct access to synthesized audio samples
- Configurable audio format and device selection

## Notes

- The application uses the default Coqui TTS voice and en-US language
- Audio configuration is automatically detected based on your system's capabilities
- GPU acceleration can be enabled for faster synthesis (requires CUDA support) 