# Cross-Compilation Guide for VoipGlot Windows

This guide explains how to build Windows binaries from macOS.

## Quick Start

### Method 1: Direct Cross-Compilation (Recommended)
```bash
./build-cross-macos.sh
```

### Method 2: Docker Cross-Compilation (Most Reliable)
```bash
./build-docker.sh
```

### Method 3: Manual Commands
```bash
# Install Windows target
rustup target add x86_64-pc-windows-msvc

# Build for Windows
cargo build --release --target x86_64-pc-windows-msvc
```

## Prerequisites

### For Direct Cross-Compilation:
- Rust 1.76.0+ (already configured in rust-toolchain.toml)
- macOS (the script checks for this)

### For Docker Cross-Compilation:
- Docker Desktop for macOS
- No Rust installation required on host

## Output

The Windows executable will be created at:
- Direct method: `target/x86_64-pc-windows-msvc/release/voipglot-win.exe`
- Docker method: `voipglot-win.exe` (in project root)

## Testing the Binary

1. Transfer the `.exe` file to a Windows machine
2. Install VB-CABLE Virtual Audio Device on Windows
3. Set up your API keys as environment variables:
   ```cmd
   set DEEPL_API_KEY=your-key
   set AZURE_SPEECH_KEY=your-key
   set AZURE_REGION=eastus
   ```
4. Run the executable: `voipglot-win.exe`

## Troubleshooting

### Build Fails with Direct Method
- Try the Docker method instead
- Check your internet connection
- Ensure you have sufficient disk space

### Docker Build Fails
- Make sure Docker Desktop is running
- Check Docker has sufficient resources allocated
- Try running `docker system prune` to free space

### Binary Doesn't Run on Windows
- Ensure Windows 10/11 (64-bit)
- Install Visual C++ Redistributable if needed
- Check that VB-CABLE is properly installed
- Verify API keys are correctly set

## Alternative Targets

If MSVC target fails, try GNU target:
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

## File Sizes

Typical file sizes:
- Release build: ~5-10 MB
- Debug build: ~50-100 MB

## Performance Notes

- Cross-compiled binaries may be slightly larger than native builds
- Performance should be identical to native Windows builds
- All optimizations are preserved in release builds
