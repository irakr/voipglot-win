# Coqui TTS Integration

This document describes the integration of Coqui TTS into the VoipGlot Windows application.

## Overview

The Coqui TTS module from `tests/tts-coqui` has been successfully integrated into the main VoipGlot application, replacing the previous custom TTS implementation with a high-quality, offline text-to-speech engine.

## Changes Made

### 1. Dependencies

**Cargo.toml**:
- Added `coqui-tts = "0.2.0"` dependency
- Added `pyo3-build-config = "0.20"` build dependency

### 2. Configuration

**config.rs**:
- Extended `TtsConfig` struct with:
  - `model_path: String` - Path to Coqui TTS model
  - `enable_gpu: bool` - GPU acceleration flag
- Updated default configuration to use Coqui TTS

**config.toml**:
- Changed provider from `"custom"` to `"coqui"`
- Added `model_path = "tts_models/en/ljspeech/fast_pitch"`
- Added `enable_gpu = false`

### 3. Implementation

**src/translation/tts.rs**:
- Replaced entire implementation with Coqui TTS integration
- Added `Synthesizer` from `coqui-tts` crate
- Implemented proper error handling for TTS initialization
- Added text preprocessing for better performance
- Added audio normalization for consistent volume
- Maintained compatibility with existing TTS interface

### 4. Build System

**build-integrated.ps1**:
- Added Coqui TTS model path checking
- Added Python TTS dependency setup
- Integrated with existing model validation system

**scripts/setup-coqui.py**:
- New Python script for setting up Coqui TTS dependencies
- Handles Python version checking
- Installs TTS package
- Sets up environment variables

### 5. Documentation

**README.md**:
- Updated architecture diagram to show Coqui TTS
- Added Python 3.7+ requirement
- Updated configuration examples
- Added Coqui TTS model information

## Technical Details

### Model Management

- **Default Model**: `tts_models/en/ljspeech/fast_pitch`
- **Auto-download**: Models are downloaded automatically on first use
- **Model Location**: Stored in `tts_models/` directory (separate from other models)

### Performance Optimizations

- **Text Length Limiting**: Truncates text to 150 characters for better performance
- **Audio Normalization**: Normalizes audio levels for consistent volume
- **Memory Management**: Processes audio in separate scopes to free memory immediately
- **Error Handling**: Graceful fallback when TTS initialization fails

### Integration Points

The Coqui TTS integration maintains full compatibility with the existing VoipGlot pipeline:

1. **Translator Interface**: No changes to the main translator API
2. **Audio Pipeline**: Seamless integration with existing audio processing
3. **Configuration**: Backward compatible with existing config structure
4. **Error Handling**: Consistent error handling with other components

## Usage

### Basic Usage

The TTS integration works automatically with the existing VoipGlot pipeline:

```rust
// Create TTS instance
let tts_config = TtsConfig::default();
let tts = TextToSpeech::new(tts_config)?;

// Synthesize speech
let audio = tts.synthesize("Hello, world!").await?;
```

### Configuration

```toml
[tts]
provider = "coqui"
model_path = "tts_models/en/ljspeech/fast_pitch"
sample_rate = 22050
channels = 1
voice_speed = 1.0
voice_pitch = 1.0
enable_gpu = false
```

### Model Selection

Different Coqui TTS models can be used by changing the `model_path`:

- `tts_models/en/ljspeech/fast_pitch` - Fast, CPU-optimized model
- `tts_models/en/ljspeech/tacotron2-DDC` - Higher quality, slower model
- `tts_models/en/vctk/vits` - Multi-speaker model

## Troubleshooting

### Common Issues

1. **Python TTS Not Installed**
   - Run: `python scripts/setup-coqui.py`
   - Or manually: `pip install TTS`

2. **Model Download Issues**
   - Models are downloaded automatically on first use
   - Check internet connection
   - Verify Python TTS installation

3. **Performance Issues**
   - Reduce text length in configuration
   - Enable GPU if available
   - Use faster models (fast_pitch instead of tacotron2)

4. **Audio Quality Issues**
   - Adjust voice_speed and voice_pitch settings
   - Try different models
   - Check audio device configuration

### Debug Information

Enable debug logging to see TTS synthesis details:

```toml
[logging]
level = "debug"
```

## Future Enhancements

1. **Multi-speaker Support**: Integration with VCTK models
2. **GPU Acceleration**: Better CUDA support
3. **Voice Cloning**: Custom voice training
4. **Real-time Streaming**: Streaming TTS for lower latency
5. **Language-specific Models**: Better support for non-English languages

## Testing

The integration includes basic tests:

```bash
# Run TTS tests
cargo test tts

# Run all tests
cargo test
```

Tests verify:
- Configuration loading
- TTS initialization
- Default configuration values
- Error handling

## Dependencies

- **Python 3.7+**: Required for Coqui TTS
- **TTS Package**: Python TTS library
- **coqui-tts**: Rust bindings for Coqui TTS
- **pyo3-build-config**: Build configuration for Python integration 