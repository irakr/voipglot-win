# Features

- **Real-time Audio Processing**: Low-latency audio capture and playback
- **Multiple AI Providers**: Support for Whisper, Azure, Google, DeepL, and ElevenLabs
- **Cross-Platform Audio**: Built with CPAL for reliable audio handling
- **Configurable**: Easy configuration via TOML files and environment variables
- **Multiple Languages**: Support for 10+ languages

## High-Level Flow

```
[Real Mic] → [VoipGlot] → [STT] → [Translation] → [TTS] → [Virtual Mic] → [Game/Discord/Zoom]
``` 