# Whisper STT Integration Guide

## Current Status

The VoipGlot application now has a foundation for local Whisper STT integration. The current implementation:

1. ✅ **Audio Detection**: Detects when speech is present using energy-based analysis
2. ✅ **Model Management**: Checks for Whisper model availability and provides download instructions
3. ✅ **Audio Preprocessing**: Converts stereo to mono and normalizes audio
4. ⏳ **Whisper Inference**: Placeholder for actual Whisper transcription

## What's Working Now

- Audio capture and processing pipeline
- Speech detection (energy-based)
- Model file validation
- Audio preprocessing for Whisper input format

## What Needs to be Completed

### Option 1: Use whisper-rs crate (Recommended)

1. **Add dependency** to `Cargo.toml`:
```toml
whisper-rs = "0.12.0"
```

2. **Download Whisper model**:
   - Download `ggml-base.bin` from: https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
   - Place it at: `~/.voipglot/whisper/ggml-base.bin` (or set `WHISPER_MODEL_PATH` environment variable)

3. **Complete the implementation** in `src/translation/stt.rs`:
   - Replace the fallback transcription with actual Whisper inference
   - Use the `whisper-rs` crate to load the model and perform transcription

### Option 2: Use whisper.cpp directly

1. **Build whisper.cpp**:
```bash
git clone https://github.com/ggerganov/whisper.cpp.git
cd whisper.cpp
make
```

2. **Download model**:
```bash
bash ./models/download-ggml-model.sh base
```

3. **Integrate with Rust** using FFI or command-line interface

## Testing the Current Implementation

1. **Build and run** the application:
```powershell
.\build-windows.ps1
.\target\release\voipglot-win.exe
```

2. **Check logs** for:
   - "Whisper model found, but not yet integrated" - Model is available
   - "Speech detected (RMS: X), but Whisper not yet integrated" - Audio is being detected

3. **Expected behavior**:
   - Audio will be captured and processed
   - Speech will be detected (energy-based)
   - No actual transcription will be produced (returns empty string)
   - Translation pipeline will skip empty transcriptions

## Next Steps

1. **Choose integration method** (whisper-rs or whisper.cpp)
2. **Download the Whisper model**
3. **Complete the transcription implementation**
4. **Test with real speech input**

## Troubleshooting

### Model not found
- Check the model path in logs
- Download the model manually
- Set `WHISPER_MODEL_PATH` environment variable

### Audio not detected
- Check microphone settings
- Verify audio input device configuration
- Adjust silence threshold in `config.toml`

### Build errors
- Ensure all dependencies are installed
- Use Visual Studio Developer PowerShell
- Check Rust toolchain version

## Performance Notes

- **ggml-base.bin**: ~142MB, good balance of speed and accuracy
- **ggml-small.bin**: ~244MB, better accuracy, slower
- **ggml-medium.bin**: ~769MB, best accuracy, slowest

For real-time applications, `ggml-base.bin` is recommended. 