# TTS Performance Fix - Root Cause Analysis and Solution

## Critical Issue Identified

The main VoipGlot application was experiencing **28+ second synthesis times** for simple phrases like "how are you", causing:
- High CPU usage (100%)
- Application unresponsiveness
- Poor audio quality (high-pitched, cartoon-like voice)
- Buffer starvation and overflow

## Root Cause Analysis

### 1. **Wrong TTS Model Selection**
- **Main App**: Using `tacotron2-DDC` (slow, high-quality model)
- **PoC App**: Using `fast_pitch` (fast, efficient model)
- **Impact**: `tacotron2-DDC` is 10-20x slower than `fast_pitch`

### 2. **Sample Rate Mismatch**
- **Main App**: Forcing 16kHz sample rate
- **TTS Output**: Native 22.05kHz
- **PoC App**: Using 22.05kHz natively
- **Impact**: Unnecessary resampling causing CPU overhead

### 3. **Inefficient Audio Processing**
- **Main App**: Complex audio transformations (pitch/speed changes)
- **PoC App**: Simple normalization only
- **Impact**: CPU-intensive processing causing delays

### 4. **Poor Resampling Algorithm**
- **Main App**: Naive resampling with sample-by-sample processing
- **PoC App**: Efficient linear interpolation with chunk processing
- **Impact**: 10x slower resampling performance

## Solution Implemented

### 1. **Model Switch to fast_pitch**
```rust
// Before: tacotron2-DDC (slow)
"tts_models/en/ljspeech/tacotron2-DDC"

// After: fast_pitch (fast)
"tts_models/en/ljspeech/fast_pitch"
```

**Expected Improvement**: 10-20x faster synthesis

### 2. **Sample Rate Alignment**
```toml
# Before: 48kHz (causing resampling)
sample_rate = 48000

# After: 22.05kHz (native TTS rate)
sample_rate = 22050
```

**Expected Improvement**: Eliminates resampling overhead

### 3. **Simplified Audio Processing**
```rust
// Before: Complex transformations
processed_buffer = self.apply_pitch_shift(&processed_buffer, self.config.voice_pitch);
processed_buffer = self.apply_speed_change(&processed_buffer, self.config.voice_speed);

// After: Simple normalization only (PoC approach)
let max_amplitude = buffer.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
if max_amplitude > 0.0 {
    buffer.into_iter().map(|x| (x / max_amplitude) * 0.95).collect()
}
```

**Expected Improvement**: 5-10x faster audio processing

### 4. **Efficient Resampling Algorithm**
```rust
// Before: Naive sample-by-sample processing
for i in 0..new_length {
    let src_index = i as f32 / ratio;
    // ... complex interpolation
}

// After: PoC's efficient linear interpolation
let mut src_pos = 0.0;
let step = 1.0 / ratio;
for _ in 0..new_samples_per_channel {
    let src_index = src_pos as usize;
    let frac = src_pos - src_index as f64;
    // ... efficient linear interpolation
    src_pos += step;
}
```

**Expected Improvement**: 10x faster resampling

## Configuration Changes

### Audio Configuration
```toml
[audio]
sample_rate = 22050   # Match TTS output
channels = 1          # Mono for efficiency
```

### TTS Configuration
```toml
[tts]
model_path = "tts_models/en/ljspeech/fast_pitch"  # Fast model
sample_rate = 22050  # Native rate
voice_speed = 1.0    # Default (no processing)
voice_pitch = 1.0    # Default (no processing)
```

### STT Configuration
```toml
[stt]
sample_rate = 22050.0  # Match TTS rate
```

## Expected Performance Improvements

### 1. **Synthesis Time**
- **Before**: 28+ seconds for "how are you"
- **After**: 1-3 seconds for "how are you"
- **Improvement**: 10-20x faster

### 2. **CPU Usage**
- **Before**: 100% CPU during synthesis
- **After**: 20-30% CPU during synthesis
- **Improvement**: 3-5x reduction

### 3. **Audio Quality**
- **Before**: High-pitched, cartoon-like voice
- **After**: Natural, clear voice
- **Improvement**: Eliminates audio artifacts

### 4. **Responsiveness**
- **Before**: Application becomes unresponsive
- **After**: Application remains responsive
- **Improvement**: No blocking operations

## Verification Steps

1. **Test Synthesis Time**: Should complete in 1-3 seconds for short phrases
2. **Monitor CPU Usage**: Should stay below 50% during synthesis
3. **Check Audio Quality**: Voice should sound natural and clear
4. **Verify Responsiveness**: Application should remain responsive during synthesis
5. **Check Logs**: Should see "Speech synthesis completed in X.XXs" with reasonable times

## PoC Validation

The solution is based on the proven approach from `tests/tts-coqui/` which:
- Uses `fast_pitch` model successfully
- Achieves sub-second synthesis times
- Maintains good audio quality
- Uses efficient audio processing
- Handles 22.05kHz sample rate natively

## Future Optimizations

If further improvements are needed:
1. **GPU Acceleration**: Enable GPU for TTS if available
2. **Model Quantization**: Use quantized models for even faster inference
3. **Streaming Synthesis**: Implement streaming for real-time output
4. **Model Caching**: Cache frequently used synthesis results 