# VoipGlot Windows Audio Translation

VoipGlot is a real-time audio translation application for Windows gaming and VOIP applications. It now uses only a local translation engine (MarianMT) for privacy, speed, and offline capability.

## Features
- Real-time audio capture and playback
- Local speech-to-text (Whisper via whisper-rs)
- Local text translation (MarianMT)
- Local text-to-speech (tts crate)
- No internet or API keys required
- Works fully offline

## System Requirements

- **Windows 10/11 (x64)**
- **Visual Studio 2022 with C++ workload** OR **Microsoft C++ Build Tools**
- **Rust 1.70+** (installed via rustup)
- **8GB+ RAM** (16GB+ recommended for PyTorch builds)
- **10GB+ free disk space**
- **VB-CABLE Virtual Audio Device** (for audio routing)

## Installation & Setup

### 1. Install Prerequisites

#### A. Install Rust
```powershell
# Download and run rustup-init.exe from https://rustup.rs/
# Or run this in PowerShell:
winget install Rustlang.Rust.MSVC
```

**⚠️ Important: Use rustup installation for full compatibility**

The VoipGlot build scripts require `rustup` (Rust's toolchain manager) for managing targets and components. There are two ways to install Rust:

**Option 1: Official rustup installer (Recommended)**
- **Download from:** [https://rustup.rs/](https://rustup.rs/)
- **Provides:** `rustup`, `cargo`, `rustfmt`, `clippy`, and full toolchain management
- **PATH:** Adds `%USERPROFILE%\.cargo\bin` to your PATH
- **Compatibility:** Full compatibility with all Rust projects and build scripts

**Option 2: winget installation (Limited compatibility)**
- **Command:** `winget install Rustlang.Rust.MSVC`
- **Provides:** Standalone Rust compiler only
- **PATH:** Adds `C:\Program Files\Rust stable MSVC 1.88\bin` to your PATH
- **Limitations:** No `rustup`, `cargo`, or component management
- **Issues:** Build scripts will fail with "rustup not recognized" errors

**If you installed via winget and encounter issues:**
1. Uninstall winget Rust:
   ```powershell
   winget uninstall "Rustlang.Rust.MSVC"
   ```
2. Install via rustup:
   - Download `rustup-init.exe` from [https://rustup.rs/](https://rustup.rs/)
   - Run the installer and follow the prompts
   - Restart your terminal after installation

**Verify installation:**
```powershell
rustup --version
cargo --version
```

#### B. Install Visual Studio 2022 with C++ Support
**Option 1: Full Visual Studio 2022**
1. Download [Visual Studio 2022 Community](https://visualstudio.microsoft.com/vs/community/) (free)
2. During installation, select **"Desktop development with C++"** workload
3. Ensure these components are included:
   - MSVC v143 - VS 2022 C++ x64/x86 build tools
   - Windows 10/11 SDK
   - CMake tools for Visual Studio

**Option 2: Standalone C++ Build Tools**
1. Download [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Select **"C++ build tools"** workload during installation

#### C. Install VB-CABLE Virtual Audio Device
1. Download from [VB-CABLE Virtual Audio Device](https://vb-audio.com/Cable/)
2. Install and restart your system
3. This creates virtual audio devices for routing audio between applications

### 2. Install PyTorch 2.0.0 for Local Features

**Required for core functionality:** PyTorch 2.0.0 is essential for local translation capabilities.

**PyTorch Package Used:**
- **Version:** PyTorch 2.0.0 CPU (LibTorch)
- **Package:** `libtorch-win-shared-with-deps-2.0.0+cpu.zip`
- **Download URL:** https://download.pytorch.org/libtorch/cpu/libtorch-win-shared-with-deps-2.0.0%2Bcpu.zip

**Installation Steps:**
1. Download the PyTorch package from the URL above
2. Extract the contents to `C:\libtorch`
3. Verify installation by checking that `C:\libtorch\lib\torch_cpu.lib` exists
4. The build script will automatically detect and use this installation

**Note:** This version is compatible with MSVC 2022 and provides the necessary libraries for local translation features. VoipGlot's core functionality depends on this PyTorch installation.

### 3. Whisper Model for STT

**Automatic download:** VoipGlot uses whisper-rs for local speech recognition and will automatically download the required Whisper model on first run.

**Whisper Model:**
- **Model:** `ggml-base.bin` (Whisper base model)
- **Size:** ~1GB
- **Location:** `%USERPROFILE%\.voipglot\whisper\ggml-base.bin`

**Note:** The application will automatically download and install the Whisper model when you first run it. No manual download required.

## Building the Application

### ⚠️ Important: Use Visual Studio Developer PowerShell

**The MSVC compiler (`cl.exe`) is only available in Visual Studio Developer environments.** You must use one of these terminals:

#### Option 1: Developer PowerShell for VS 2022 (Recommended)
1. Open **"Developer PowerShell for VS 2022"** from Start Menu
2. Navigate to your project directory
3. Run the build commands

#### Option 2: Developer Command Prompt for VS 2022
1. Open **"Developer Command Prompt for VS 2022"** from Start Menu
2. Navigate to your project directory
3. Run: `powershell -ExecutionPolicy Bypass -File .\build-windows.ps1 --fast`

### Build Options

| Command | Description |
|---------|-------------|
| `.\build-windows.ps1` | Standard optimized build |
| `.\build-windows.ps1 --fast` | Fast build (2-3x faster, slightly larger binary) |
| `.\build-windows.ps1 --clean` | Clean previous builds |
| `.\build-windows.ps1 --no-clippy` | Skip code linting |
| `.\build-windows.ps1 --no-pytorch` | Build without PyTorch (API-only) |
| `.\build-windows.ps1 --force-pytorch` | Force PyTorch build |

### Build Profiles

**Fast Build** (`--fast`):
- 2-3x faster compilation
- Parallel compilation (16 codegen units)
- Reduced optimization level
- Larger binary size
- Perfect for development

**Release Build** (default):
- Maximum optimization
- Link-time optimization (LTO)
- Smallest and fastest binary
- Slower compilation
- Best for production

## Configuration

1. **Configure** your `config.toml`:
   ```toml
   [audio]
   input_device = ""      # Leave empty for auto-detection
   output_device = ""     # Leave empty for auto-detection
   sample_rate = 48000
   channels = 2
   buffer_size = 1024
   latency_ms = 50

   [translation]
   source_language = "en"
   target_language = "es"

   [processing]
   chunk_duration_ms = 1000
   silence_threshold = 0.01
   noise_reduction = true
   echo_cancellation = true
   ```

2. **Set up API keys** (if using cloud services):
   ```powershell
   $env:OPENAI_API_KEY = "your-openai-key"
   $env:GOOGLE_TRANSLATE_API_KEY = "your-google-key"
   ```

## Running the Application

```powershell
# Standard release build
.\target\x86_64-pc-windows-msvc\release\voipglot-win.exe

# Fast release build
.\target\x86_64-pc-windows-msvc\fast-release\voipglot-win.exe

# With debug logging to file (captures all output and removes ANSI color codes)
.\target\x86_64-pc-windows-msvc\release\voipglot-win.exe --passthrough --debug *>&1 | ForEach-Object { $_ -replace '\x1b\[[0-9;]*[a-zA-Z]', '' } | Tee-Object -FilePath voipglot.log
```

**First run** will download MarianMT models for your language pair. Subsequent runs are fully offline.

## Troubleshooting

### Common Issues

**1. 'cl.exe' not found**
- **This is normal in regular PowerShell!** Use Visual Studio Developer PowerShell instead
- Install Visual Studio 2022 with 'Desktop development with C++' workload
- Or install standalone C++ Build Tools
- **Always use "Developer PowerShell for VS 2022" for building**

**2. PowerShell script execution policy error ('running scripts is disabled on this system')**
- By default, Windows restricts running PowerShell scripts for security reasons.
- If you see an error like:
  > File ... cannot be loaded because running scripts is disabled on this system.
- **Solution:** Open PowerShell and run:
  ```powershell
  Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
  ```
- This allows local scripts to run while keeping downloaded scripts restricted unless signed.
- You only need to do this once per user account.

**3. 'rustup' not recognized after installing Rust via winget**
- If you installed Rust using `winget`, it should include `rustup`, but sometimes the PATH is not updated immediately.
- If you see errors like:
  > rustup : The term 'rustup' is not recognized as the name of a cmdlet, function, script file, or operable program.
- **Solution:**
  1. Close and reopen your terminal (or log out and back in) to reload your environment variables.
  2. If the problem persists, manually add Rust's bin directory to your PATH:
     - Open System Properties → Environment Variables
     - Under "User variables" or "System variables", find `Path` and click Edit
     - Add: `C:\Users\<YourUsername>\.cargo\bin`
     - Click OK and restart your terminal
  3. Verify by running:
     ```powershell
     rustup --version
     cargo --version
     ```
  4. If you still have issues, try reinstalling Rust using the official installer from https://rustup.rs/

**4. PyTorch compilation errors**
- PyTorch 1.12.1 has known compatibility issues with newer MSVC
- Try: `./build-windows.ps1 --no-pytorch` (API-only build)
- Or upgrade to PyTorch 1.13.1+ for better compatibility

**5. Build takes too long**
- Use: `./build-windows.ps1 --fast` (2-3x faster)
- Skip clippy: `./build-windows.ps1 --fast --no-clippy`

**6. Dependency conflicts**
- Try: `./build-windows.ps1 --clean`
- Clear cargo cache: `cargo clean`

**7. Permission errors**
- Run PowerShell as Administrator
- Check antivirus isn't blocking the build

### Using Visual Studio Developer Environments

**The MSVC compiler is only available in Visual Studio Developer environments.** Regular PowerShell will not work.

#### Developer PowerShell for VS 2022 (Recommended)
```powershell
# 1. Open "Developer PowerShell for VS 2022" from Start Menu
# 2. Navigate to project directory
cd D:\Irak\Voipglot\voipglot-win

# 3. Run build commands
.\build-windows.ps1 --fast
```

#### Developer Command Prompt for VS 2022
```cmd
# 1. Open "Developer Command Prompt for VS 2022" from Start Menu
# 2. Navigate to project directory
cd D:\Irak\Voipglot\voipglot-win

# 3. Run build script
powershell -ExecutionPolicy Bypass -File .\build-windows.ps1 --fast
```

**Note**: The `cl.exe` compiler is intentionally not added to the system PATH to avoid conflicts with other build tools. Visual Studio provides these specialized environments for C++ development.

### Build Features

| Feature | With PyTorch | Without PyTorch |
|---------|-------------|-----------------|
| Local Translation | ✓ | ✗ |
| Local STT | ✓ | ✗ |
| Local TTS | ✓ | ✗ |
| API Translation | ✓ | ✓ |
| Audio Capture/Playback | ✓ | ✓ |

## Documentation
- See `docs/local-translation.md` for details on the local translation engine.
- See `docs/configuration.md` for configuration options.

## Supported Languages
- English, Spanish, French, German, Italian, Portuguese, Russian, Japanese, Korean, Chinese, Arabic, Hindi (see docs for full list)

## License
MIT 