# VoipGlot Windows

Real-time audio translation for Windows gaming and VOIP applications using the VoipGlot Core library. Features both a modern GUI interface and command-line interface.

## Features

- **Modern GUI Interface**: Beautiful Tauri-based desktop application with real-time audio visualization
- **Command-Line Interface**: Traditional CLI for automation and scripting
- **Cross-platform Core**: Uses voipglot-core library for audio processing and translation
- **Speech-to-Text (STT)**: Real-time speech recognition using VOSK
- **Translation**: Text translation using CTranslate2 with NLLB-200 model
- **Text-to-Speech (TTS)**: Speech synthesis using Coqui TTS
- **Offline Processing**: All AI processing happens locally, no internet required
- **Real-time Pipeline**: Low-latency audio processing pipeline
- **Multi-language Support**: Support for 200+ languages via NLLB-200
- **Windows Optimized**: Windows-specific audio optimizations and integration
- **Audio Device Management**: Easy selection of input/output audio devices
- **Real-time Audio Visualization**: Live microphone frequency display

## Architecture

```
Microphone → voipglot-core → Audio Output
                ↓
        [STT → Translation → TTS]
```

The Windows application provides both a modern GUI interface and CLI wrapper around the voipglot-core library, which handles all the AI processing and audio pipeline management.

## Prerequisites

- Windows 10/11
- Rust 1.82.0 or later
- Visual Studio Build Tools (for native dependencies)
- Microphone input device
- Audio output device
- voipglot-core distribution package (ZIP file)

## Installation

### 1. Clone the Repository

```powershell
git clone <repository-url>
cd voipglot-win
```

### 2. Install Rust

If you don't have Rust installed, download and install it from [https://rustup.rs/](https://rustup.rs/)

### 3. Install Visual Studio Build Tools

Download and install Visual Studio Build Tools from Microsoft. Make sure to include:
- MSVC v143 build tools
- Windows 10/11 SDK
- CMake tools

### 4. Download voipglot-core Package

Download the voipglot-core distribution package (ZIP file) from the releases page or build it yourself using the voipglot-core project.

The package contains:
- Pre-built voipglot-core library
- Native dependencies (VOSK, etc.)
- AI models for speech recognition and translation
- Setup scripts for environment configuration

## Building

### Using the Build Script (Recommended)

1. **Set the package path environment variable:**
   ```powershell
   $env:LIBVOIPGLOT_CORE_PKG = "path/to/voipglot-core-release.zip"
   ```

2. **Run the build script:**
   ```powershell
   # Open Developer PowerShell for VS
   .\build.ps1
   ```

The build script will automatically:
- Extract the voipglot-core package
- Set up all required environment variables
- Link against the pre-built library and native dependencies
- Copy models and native libraries to the target directory
- Build both CLI and GUI versions

### Build Modes

```powershell
# Full build (CLI + GUI) - Default
.\build.ps1

# GUI development mode (with hot reload)
.\build.ps1 -TauriDev

# CLI only build
.\build.ps1 -CliOnly

# GUI only build
.\build.ps1 -TauriOnly

# Fast development build
.\build.ps1 -Fast

# Clean build
.\build.ps1 -Clean
```

### Manual Build

```powershell
# Build CLI release version
cargo build --release --target x86_64-pc-windows-msvc

# Build GUI release version
cargo tauri build --project src-tauri

# Build GUI development version
cargo tauri dev --project src-tauri

# Build fast development version
cargo build --profile fast-release --target x86_64-pc-windows-msvc
```

## Usage

### GUI Interface (Recommended)

The modern GUI provides an intuitive interface for real-time audio translation:

1. **Launch the application** - The GUI will open automatically
2. **Configure audio devices** - Select your input and output devices
3. **Choose languages** - Set source and target languages
4. **Start processing** - Click the microphone button to begin real-time translation

**Features:**
- Real-time audio frequency visualization
- Easy audio device selection
- Language selection with 200+ supported languages
- Modern glassmorphism design
- Settings and help access

### Command Line Interface

```powershell
# Run CLI with default configuration
.\target\release\voipglot-win.exe

# Run CLI with custom configuration
.\target\release\voipglot-win.exe -c my_config.toml

# Run CLI with debug logging
.\target\release\voipglot-win.exe --debug

# List available audio devices
.\target\release\voipglot-win.exe --list-devices
```

### Command Line Options

- `-c, --config <path>`: Configuration file path (default: config.toml)
- `--debug`: Enable debug logging
- `--list-devices`: List available audio input/output devices
- `--source-lang <lang>`: Source language code (e.g., "en", "fr", "de")
- `--target-lang <lang>`: Target language code (e.g., "en", "fr", "de")
- `--sample-rate <rate>`: Audio sample rate in Hz (default: 16000)
- `--channels <count>`: Audio channels (1 for mono, 2 for stereo, default: 1)
- `--buffer-size <size>`: Audio buffer size in samples (default: 1024)
- `--latency-ms <ms>`: Target latency in milliseconds (default: 10)
- `--silence-threshold <value>`: Silence threshold for voice detection (default: 0.01)
- `--chunk-duration-ms <ms>`: Audio chunk duration in milliseconds (default: 1000)

## Configuration

The application uses `config.toml` for configuration. The configuration structure matches the voipglot-core library:

```toml
[audio.input]
input_device = ""
sample_rate = 16000
channels = 1
buffer_size = 1024
latency_ms = 50
vb_cable_device = "CABLE Input (VB-Audio Virtual Cable)"

[audio.output]
output_device = ""
sample_rate = 48000
channels = 2
buffer_size = 2048
latency_ms = 100

[processing]
chunk_duration_ms = 100
silence_threshold = 0.01
noise_reduction = true
echo_cancellation = true
enable_feedback_prevention = true
tts_silence_buffer_ms = 50
tts_queue_size = 3

[stt]
provider = "vosk"
model_path = "../voipglot-core/models/vosk-model-en-in-0.5"
sample_rate = 16000.0
enable_partial_results = true

[translation]
provider = "ct2"
model_path = "../voipglot-core/models/nllb-200-ct2"
source_language = "eng_Latn"
target_language = "eng_Latn"
num_threads = 4
device = "cpu"
max_batch_size = 32
beam_size = 4

[tts]
provider = "coqui"
model_path = "tts_models/en/ljspeech/fast_pitch"
voice_speed = 1.0
voice_pitch = 1.0
enable_gpu = false
synthesis_timeout_secs = 5
```

## What voipglot-win Does NOT Handle

Since voipglot-win uses the voipglot-core library, it does not need to worry about:

### ❌ **AI Model Management**
- **Model files**: All AI models (VOSK, CTranslate2, Coqui TTS) are managed by voipglot-core
- **Model downloads**: Model acquisition and setup is handled by voipglot-core
- **Model updates**: Model versioning and updates are managed centrally

### ❌ **AI Library Dependencies**
- **Audio processing libraries**: cpal, symphonia, dasp are handled by voipglot-core
- **STT libraries**: VOSK bindings and integration are in voipglot-core
- **Translation libraries**: CTranslate2 integration is in voipglot-core
- **TTS libraries**: Coqui TTS bindings are in voipglot-core

### ❌ **Core Pipeline Logic**
- **Audio pipeline**: Audio capture, processing, and playback logic is in voipglot-core
- **Translation pipeline**: STT → Translation → TTS flow is managed by voipglot-core
- **Error handling**: Core error handling and recovery is in voipglot-core

## What voipglot-win DOES Handle

### ✅ **Windows-Specific Integration**
- **Platform-specific audio device management**
- **Windows user experience and interface**
- **Windows-specific configuration and settings**
- **Integration with Windows gaming/VOIP applications**

### ✅ **Application-Level Features**
- **Modern GUI interface with Tauri**
- **Command-line interface and argument parsing**
- **Configuration file management**
- **Logging and debugging for Windows environment**
- **User interaction and feedback**
- **Real-time audio visualization**

## Model Management

Models are managed by the voipglot-core library. The Windows application doesn't handle model downloads directly. Models should be available in the paths specified in the configuration file.

### Required Models (Managed by voipglot-core)

- **VOSK Model**: For speech recognition (specified in `[stt].model_path`)
- **CTranslate2 Model**: For translation (specified in `[translation].model_path`)
- **Coqui TTS Model**: For speech synthesis (specified in `[tts].model_path`)

## Troubleshooting

### Debugging and Logs

The application provides comprehensive logging for both frontend and backend components to help with debugging.

#### Frontend Logs (TypeScript/UI)

**Development Mode (Recommended for Debugging):**
```powershell
cd voipglot-win
.\build.ps1 -TauriDev
```

This command:
- **Automatically builds the frontend** (npm run build)
- **Generates fresh dist/ directory** with latest TypeScript changes
- **Automatically opens Developer Tools** with console visible
- **Shows real-time TypeScript logs** in the browser console
- **Enables hot reload** for frontend development
- **Displays all UI interactions** and Tauri command calls

**For Frontend-Only Changes:**
```powershell
cd voipglot-win
.\build.ps1 -FrontendBuild
```

This will:
- **Build only the frontend** (npm run build)
- **Update the dist/ directory** with latest changes
- **Skip Tauri backend** (faster for frontend-only work)

**Manual Development (Alternative):**
```powershell
cd voipglot-win
cargo tauri dev
```

**Note**: If you make changes to TypeScript files, you need to rebuild the frontend first:
```powershell
npm run build
# Then restart cargo tauri dev
```

#### Backend Logs (Rust/Backend)

**Log File Location:**
- **File**: `voipglot-win.log` in the application directory
- **Format**: Structured logging with timestamps and log levels
- **Content**: All Rust backend operations, Tauri commands, and error conditions

**Viewing Backend Logs:**
```powershell
# View the entire log file
Get-Content voipglot-win.log

# Follow the log in real-time (like tail -f)
Get-Content voipglot-win.log -Wait

# View last 50 lines
Get-Content voipglot-win.log -Tail 50

# Search for specific errors
Get-Content voipglot-win.log | Select-String "ERROR"
```

**What You'll See in Backend Logs:**
```
2024-01-15T10:30:45.124Z INFO  voipglot_win::lib::commands > Test connection command called
2024-01-15T10:30:45.125Z INFO  voipglot_win::lib::commands > Getting audio devices...
2024-01-15T10:30:45.126Z INFO  voipglot_win::lib::commands > Found 3 input devices and 2 output devices
2024-01-15T10:30:45.127Z INFO  voipglot_win::lib::commands > Starting audio processing with config: input=Microphone, output=Speakers, source=en, target=es
2024-01-15T10:30:45.128Z INFO  voipglot_win::lib::commands > Configuration updated successfully
2024-01-15T10:30:45.129Z INFO  voipglot_win::lib::commands > Creating VoipGlot pipeline...
2024-01-15T10:30:45.130Z INFO  voipglot_win::lib::commands > Pipeline created successfully
```

#### Combined Debugging Workflow

For comprehensive debugging, use both logging systems:

1. **Start development mode**: `.\build.ps1 -TauriDev`
2. **Open browser console** (automatically opened)
3. **Monitor both consoles**:
   - **Frontend console**: UI interactions, Tauri calls
   - **Backend log file**: Rust operations, pipeline status
4. **Test UI interactions** and watch both logs simultaneously

**For Frontend Changes During Development:**
1. **Make changes** to TypeScript files in `src/`
2. **Rebuild frontend**: `.\build.ps1 -FrontendBuild`
3. **Restart Tauri**: `cargo tauri dev` (or use `.\build.ps1 -TauriDev` again)

#### Production Debugging

For production builds, backend logs are always available in `voipglot-win.log`. Frontend debugging requires development mode.

### Common Issues

1. **voipglot-core not found**: Ensure the voipglot-core library is in the parent directory
2. **Audio device issues**: Use `--list-devices` to see available devices
3. **Model not found**: Check that model paths in config.toml are correct
4. **Build errors**: Try `.\build.ps1 -Clean` to clean and rebuild
5. **GUI not starting**: Ensure Tauri dependencies are installed with `cargo install tauri-cli --version '^2.0.0' --locked`
6. **libvosk.dll missing**: The MSI installer may not include native dependencies. Copy `native-libs` from the bundle directory to the installation directory. Alternative: Run the executable from the `target\release` directory where native dependencies are available.

## Development

### Project Structure

```
voipglot-win/
├── src-tauri/               # Tauri backend (Rust)
│   ├── src/
│   │   ├── main.rs          # Tauri application entry point
│   │   └── lib.rs           # Tauri backend library
│   ├── Cargo.toml           # Rust dependencies
│   ├── tauri.conf.json      # Tauri application configuration
│   └── build.rs             # Tauri build configuration
├── src/                     # Frontend source (HTML/TS/CSS/etc.)
│   ├── index.html           # Frontend entry point
│   ├── main.ts              # TypeScript frontend logic
│   └── styles.css           # Frontend styling
├── package.json             # Frontend dependencies
├── tsconfig.json            # TypeScript configuration
├── vite.config.ts           # Vite build configuration
├── icons/
├── config.toml
├── build.ps1
├── Cargo.toml               # Workspace configuration
└── README.md
```

**Note:**
- All frontend code (HTML, TS, CSS, etc.) is in the outermost `src/` directory.
- Tauri config (`tauri.conf.json`) has `"frontendDist": "../dist"` (points to Vite build output).
- Frontend uses TypeScript with Vite build system.
- Vite root is set to `src/` and builds to `dist/` (project root).
- In `index.html`, reference scripts as `main.ts` (Vite handles the build).

### Adding Features

Since this application uses voipglot-core, new AI features should be implemented in the core library rather than here. This application focuses on Windows-specific integration, user experience, and the GUI interface.

### GUI Development

The GUI is built with Tauri 2.0 using:
- **HTML/CSS/TypeScript** for the frontend (all code in `src/`)
- **Rust** for the backend with Tauri commands
- **Modern styling** with glassmorphism effects
- **Real-time audio visualization**

#### Development Workflow

**⚠️ IMPORTANT: Proper Development Method**

The application **must** be run using Tauri's development mode for full functionality:

```powershell
cd voipglot-win
cargo tauri dev
```

**❌ DO NOT** access `http://localhost:1420/` directly in a browser. This will:
- Run the frontend without the Tauri backend
- Show an error overlay explaining the proper usage
- Disable all audio processing and translation features

**✅ DO** use `cargo tauri dev` which:
- Opens the native Tauri window with full functionality
- Automatically opens Developer Tools for debugging
- Enables hot reload for frontend development
- Provides access to all Tauri backend features

#### Frontend Development

The frontend uses TypeScript with Vite for fast development:

```bash
# Install frontend dependencies
npm install

# Start development server (for Tauri integration)
cargo tauri dev

# Build for production
npm run build
```

The frontend structure follows modern web development practices with:
- **TypeScript** for type safety
- **Vite** for fast build times and hot reload
- **ES modules** for modern JavaScript features
- **Tauri APIs** for native desktop integration

#### Debugging

**Frontend Logs (TypeScript/UI):**
```powershell
cargo tauri dev
```
- **Automatically opens Developer Tools** with console visible
- **Shows real-time TypeScript logs** in the browser console
- **Enables hot reload** for frontend development
- **Displays all UI interactions** and Tauri command calls

**Backend Logs (Rust/Backend):**
- **File**: `voipglot-win.log` in the application directory
- **View with**: `Get-Content voipglot-win.log -Wait`

**Combined Debugging:**
1. Run `cargo tauri dev`
2. Monitor both frontend console and backend log file
3. Test UI interactions and watch both logs simultaneously

## License

This project is licensed under the MIT License - see the LICENSE file for details. 