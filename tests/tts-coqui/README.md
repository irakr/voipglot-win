
# TTS-Coqui Test Application

This is a proof-of-concept application demonstrating real-time Text-to-Speech (TTS) using Coqui TTS. The application accepts text input from the user via console and plays the synthesized speech through the system's default audio output device.

## Prerequisites

1. Rust toolchain (2021 edition or later)
2. Coqui TTS library and its dependencies
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

## Usage

1. Run the application
2. The program will initialize the TTS engine and connect to your default audio output device
3. Type text at the prompt and press Enter
4. The application will convert your text to speech and play it through the speakers
5. Press Ctrl+C to exit

## Features

- Real-time text-to-speech synthesis
- Uses system's default audio output device
- Supports listing available audio devices
- Configurable audio settings
- Error handling and logging

## Troubleshooting

1. No audio output:
   - Check if your audio device is properly connected and set as default
   - Use `--list-devices` to verify available audio devices
   - Check system volume settings

2. TTS initialization fails:
   - Ensure Coqui TTS is properly installed
   - Check if required language models are available

## Notes

- The application uses the default Coqui TTS voice and en-US language
- Audio configuration is automatically detected based on your system's capabilities 