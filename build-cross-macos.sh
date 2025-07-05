#!/bin/bash

# VoipGlot Cross-Compilation Script (macOS to Windows)
echo "========================================"
echo "VoipGlot Cross-Compilation (macOS → Windows)"
echo "========================================"

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "Error: This script is designed for macOS"
    echo "Current OS: $OSTYPE"
    exit 1
fi

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Error: Rust is not installed or not in PATH"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Rust is installed: $(rustc --version)"
echo "Current platform: $(rustc --print target-list | grep host)"

# Check if the Windows target is installed
if ! rustup target list --installed | grep -q "x86_64-pc-windows-msvc"; then
    echo "Installing Windows target..."
    rustup target add x86_64-pc-windows-msvc
    if [ $? -ne 0 ]; then
        echo "Error: Failed to install Windows target"
        echo "This might require additional setup. See troubleshooting below."
        exit 1
    fi
else
    echo "Windows target already installed"
fi

# Check if required components are installed
if ! rustup component list --installed | grep -q "rustfmt"; then
    echo "Installing rustfmt..."
    rustup component add rustfmt
fi

if ! rustup component list --installed | grep -q "clippy"; then
    echo "Installing clippy..."
    rustup component add clippy
fi

echo ""
echo "========================================"
echo "Cross-compiling VoipGlot for Windows..."
echo "========================================"

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Run clippy for code quality checks (on host target)
echo "Running clippy..."
cargo clippy --release
if [ $? -ne 0 ]; then
    echo "Warning: Clippy found issues, but continuing with build..."
fi

# Build the release version for Windows
echo "Building release version for Windows..."
cargo build --release --target x86_64-pc-windows-msvc
if [ $? -ne 0 ]; then
    echo ""
    echo "========================================"
    echo "Build failed! Common issues and solutions:"
    echo "========================================"
    echo ""
    echo "1. Missing Windows SDK:"
    echo "   - Install Visual Studio Build Tools for Windows"
    echo "   - Or use cross-compilation tools (see below)"
    echo ""
    echo "2. Alternative: Use cross-compilation tools:"
    echo "   brew install FiloSottile/musl-cross/musl-cross"
    echo "   rustup target add x86_64-pc-windows-gnu"
    echo "   cargo build --release --target x86_64-pc-windows-gnu"
    echo ""
    echo "3. Use Docker for cross-compilation:"
    echo "   docker run --rm -v $(pwd):/app -w /app rust:latest"
    echo "   rustup target add x86_64-pc-windows-msvc"
    echo "   cargo build --release --target x86_64-pc-windows-msvc"
    echo ""
    exit 1
fi

echo ""
echo "========================================"
echo "Cross-compilation completed successfully!"
echo "========================================"
echo ""
echo "Windows executable location:"
echo "  target/x86_64-pc-windows-msvc/release/voipglot-win.exe"
echo ""
echo "File size: $(ls -lh target/x86_64-pc-windows-msvc/release/voipglot-win.exe | awk '{print $5}')"
echo ""
echo "To test the Windows binary:"
echo "1. Transfer to a Windows machine"
echo "2. Install VB-CABLE Virtual Audio Device on Windows"
echo "3. Set up API keys in environment variables"
echo "4. Run: voipglot-win.exe"
echo ""
echo "Note: The binary requires Windows 10/11 and the following dependencies:"
echo "- Visual C++ Redistributable (usually pre-installed)"
echo "- VB-CABLE Virtual Audio Device"
echo "- Internet connection for API calls"
echo ""
