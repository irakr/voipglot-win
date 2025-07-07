# Features

- **Real-time Audio Processing**: Low-latency audio capture and playback
- **Local AI Processing**: Whisper-rs for STT, MarianMT for translation, tts crate for TTS
- **Cross-Platform Audio**: Built with CPAL for reliable audio handling
- **Configurable**: Easy configuration via TOML files and environment variables
- **Multiple Languages**: Support for 10+ languages
- **Fully Offline**: No internet or API keys required

## High-Level Flow

```
[Real Mic] → [VoipGlot] → [STT] → [Translation] → [TTS] → [Virtual Mic] → [Game/Discord/Zoom]
``` 