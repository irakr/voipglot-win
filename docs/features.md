# Features

## Core Components (Successfully Tested)

### ✅ Speech-to-Text (STT) - VOSK Implementation
- **Real-time audio capture** from physical microphone devices
- **VOSK-based speech recognition** for offline, high-accuracy transcription
- **Live transcription output** with partial results support
- **Automated environment setup** and model download
- **Multi-device support** with automatic device detection
- **Configurable audio parameters** (16kHz mono, required by VOSK)

### ✅ Translation - CTranslate2 with NLLB-200
- **Offline translation** using CTranslate2 framework
- **NLLB-200 model support** for 200+ languages
- **CPU and GPU acceleration** options
- **Configurable translation parameters** (batch size, beam size, threads)
- **Low-latency processing** optimized for real-time use
- **Automatic model download and conversion**

### ✅ Text-to-Speech (TTS) - Coqui TTS
- **Real-time speech synthesis** using Coqui TTS
- **System audio output integration** with device detection
- **Configurable audio settings** and device selection
- **Natural-sounding voice generation**
- **Cross-platform audio support** via CPAL

## Pipeline Architecture

```
[Real Microphone] → [VOSK STT] → [CTranslate2 Translation] → [Coqui TTS] → [Virtual Microphone (VB-CABLE)] → [Target Applications]
```

## High-Level Flow

```
[Real Mic] → [VoipGlot] → [STT] → [Translation] → [TTS] → [Virtual Mic] → [Game/Discord/Zoom]
```

## Technical Features

- **Real-time Audio Processing**: Low-latency audio capture and playback
- **Offline AI Processing**: All core components work without internet connection
- **Cross-Platform Audio**: Built with CPAL for reliable audio handling
- **Configurable**: Easy configuration via TOML files and environment variables
- **Multi-language Support**: 200+ languages via NLLB-200 model
- **Automated Setup**: Build scripts handle all dependencies and model downloads
- **Device Detection**: Automatic audio device discovery and configuration
- **Error Handling**: Comprehensive error handling and logging throughout the pipeline

## Performance Characteristics

- **STT Latency**: Real-time transcription with VOSK
- **Translation Speed**: Optimized CTranslate2 processing
- **TTS Quality**: Natural-sounding speech synthesis
- **Audio Quality**: High-fidelity audio processing pipeline
- **Resource Usage**: Efficient CPU/GPU utilization 