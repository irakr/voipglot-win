#!/bin/bash

# VoipGlot Windows Build Script (Shell)
echo "========================================"
echo "VoipGlot Windows Build Script"
echo "========================================"

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Error: Rust is not installed or not in PATH"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Rust is installed: $(rustc --version)"
echo "Checking toolchain..."

# Check if the required target is installed
if ! rustup target list --installed | grep -q "x86_64-pc-windows-msvc"; then
    echo "Installing Windows target..."
    rustup target add x86_64-pc-windows-msvc
    if [ $? -ne 0 ]; then
        echo "Error: Failed to install Windows target"
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
echo "Building VoipGlot for Windows..."
echo "========================================"

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Run clippy for code quality checks
echo "Running clippy..."
cargo clippy --release
if [ $? -ne 0 ]; then
    echo "Warning: Clippy found issues, but continuing with build..."
fi

# Build the release version
echo "Building release version..."
cargo build --release --target x86_64-pc-windows-msvc
if [ $? -ne 0 ]; then
    echo "Error: Build failed!"
    exit 1
fi

echo ""
echo "========================================"
echo "Build completed successfully!"
echo "========================================"
echo ""
echo "Executable location: target/x86_64-pc-windows-msvc/release/voipglot-win.exe"
echo ""
echo "To run the application:"
echo "  target/x86_64-pc-windows-msvc/release/voipglot-win.exe"
echo ""
echo "Remember to:"
echo "1. Install VB-CABLE Virtual Audio Device"
echo "2. Set up your API keys in environment variables"
echo "3. Configure config.toml if needed"
echo "" 