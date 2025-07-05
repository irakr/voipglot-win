# Usage

## Basic Usage

```powershell
# Run with default settings (English to Spanish)
./target/x86_64-pc-windows-msvc/fast-release/voipglot-win.exe

# Or, for production build:
./target/x86_64-pc-windows-msvc/release/voipglot-win.exe

# Run with custom languages
./target/x86_64-pc-windows-msvc/fast-release/voipglot-win.exe --source-lang en --target-lang fr

# Run with custom config file
./target/x86_64-pc-windows-msvc/fast-release/voipglot-win.exe --config my-config.toml

# Enable debug logging
./target/x86_64-pc-windows-msvc/fast-release/voipglot-win.exe --debug
```

## Build Optimization Tips

- **Fast development builds:** `./build-windows.ps1 --fast`
- **Skip clippy for speed:** `./build-windows.ps1 --fast --no-clippy`
- **Production builds:** `./build-windows.ps1` (default, optimized)
- **Clean when needed:** `./build-windows.ps1 --clean`
- **Dependencies are cached** for faster subsequent builds

**Build speed comparison:**
- Fast build: ~2-3x faster, slightly larger binary
- Release build: Slowest, smallest and fastest binary

## Performance Optimization

### For Gaming
- Use `chunk_duration_ms = 500` for lower latency
- Enable `noise_reduction = true`
- Use Whisper for offline STT to reduce network dependency

### For VOIP Applications
- Use `chunk_duration_ms = 1000` for better quality
- Enable `echo_cancellation = true`
- Use Azure or Google for cloud-based processing 