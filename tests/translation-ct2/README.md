# Translation Test with CTranslate2

This is a test application for the translation component of VoipGlot using CTranslate2 with the NLLB-200 model.

## Overview

This test demonstrates:
- Loading and initializing the CTranslate2 translator
- Performing text translation using the NLLB-200 model
- Configuration management
- Basic error handling and logging

## Prerequisites

### Python Requirements
- **Python Version**: 3.8 - 3.12 (tested and confirmed working with Python 3.11)
  - Earlier versions may not support all required packages
  - Later versions may work but are untested
  - Download Python from: https://www.python.org/downloads/
- **Required Python Packages** (automatically installed by build script):
  - ctranslate2
  - transformers
  - torch
  - huggingface_hub
  - protobuf
  - hf_xet (optional, for better download performance)

### Other Requirements
- Rust toolchain (2021 edition or later)
- PowerShell (for running build scripts)
- CMake (for building CTranslate2)
- Git (for cloning repositories)
- ~5GB free disk space for model files

## Requirements

### CMake
- **Required Version**: 3.28.3
- **Download URL**: https://cmake.org/files/v3.28/cmake-3.28.3-windows-x86_64.msi
- **Installation Instructions**:
  1. Download the Windows x64 Installer from the URL above
  2. Run the installer with administrative privileges
  3. During installation, select "Add CMake to the system PATH for all users"
  4. Complete the installation and restart your Developer PowerShell

**Note**: Other CMake versions may cause compatibility issues with the build process. We specifically recommend version 3.28.3 as it has been tested with all project dependencies.

## Setup

1. **Verify Python Installation**:
   ```powershell
   python --version  # Should show Python 3.8 - 3.12
   ```

2. **Run the build script**:
   ```powershell
   .\build.ps1
   ```
   This will:
   - Verify Python version compatibility
   - Install required Python packages
   - Download and convert the NLLB-200 model
   - Build the Rust project

3. The script will create a `models` directory containing the converted NLLB-200 model.

## Configuration

Edit `config.toml` to customize:
- Model path
- Source and target languages
- Translation parameters
- Device selection (CPU/GPU)
- Logging level

### Language Codes

NLLB-200 uses specific language codes. Common ones include:
- `eng_Latn`: English
- `spa_Latn`: Spanish
- `fra_Latn`: French
- `deu_Latn`: German
- `ita_Latn`: Italian
- `por_Latn`: Portuguese
- `rus_Cyrl`: Russian
- `cmn_Hans`: Chinese (Simplified)
- `jpn_Jpan`: Japanese

See the [NLLB-200 documentation](https://github.com/facebookresearch/flores/blob/main/flores200/README.md#languages-in-flores-200) for a complete list.

## Running the Test

1. Make sure the model is downloaded and converted (run `build.ps1` if not)
2. Configure settings in `config.toml`
3. Run the test:
   ```bash
   cargo run --release
   ```

## Integration with VoipGlot

This component fits into the VoipGlot pipeline:
```
Audio Input -> VOSK (STT) -> CTranslate2 (Translation) -> Coqui (TTS) -> Audio Output
```

The test demonstrates the translation step, converting text from one language to another with low latency.

## Performance Considerations

- CPU vs GPU: Configure in `config.toml` based on your hardware
- Batch size: Adjust for optimal throughput
- Thread count: Set based on your CPU cores
- Beam size: Higher values may improve quality but increase latency

## Troubleshooting

1. **Python Version Issues**:
   - If you see package compatibility errors, verify Python version (3.8 - 3.12)
   - If using Python outside this range:
     - For older versions: Upgrade Python
     - For newer versions: Consider installing a supported version

2. **Model Download Issues**:
   - Check internet connection
   - Verify Python packages are installed
   - Check disk space (need ~5GB free)
   - If seeing path errors, try running PowerShell as administrator

3. **Build Failures**:
   - Ensure CMake is installed
   - Check Rust toolchain is up to date
   - Verify all dependencies are installed

4. **Runtime Errors**:
   - Check model path in config.toml
   - Verify language codes are correct
   - Check system resources (memory/GPU)

## Common Issues and Solutions

1. **Path-related Errors**:
   - Use PowerShell for running scripts
   - Avoid spaces in installation paths
   - Run as administrator if seeing permission errors

2. **Package Installation Errors**:
   - Try updating pip: `python -m pip install --upgrade pip`
   - If a package fails, try installing it manually
   - Check Python version compatibility

3. **Model Conversion Errors**:
   - Ensure adequate disk space
   - Check write permissions in the models directory
   - Try running PowerShell as administrator 