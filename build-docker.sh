#!/bin/bash

# VoipGlot Docker Cross-Compilation Script
echo "========================================"
echo "VoipGlot Docker Cross-Compilation"
echo "========================================"

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed or not in PATH"
    echo "Please install Docker Desktop for macOS"
    exit 1
fi

echo "Docker is installed: $(docker --version)"

# Build the Docker image
echo "Building Docker image for cross-compilation..."
docker build -f Dockerfile.cross -t voipglot-cross .

if [ $? -ne 0 ]; then
    echo "Error: Docker build failed!"
    exit 1
fi

# Run the container to extract the binary
echo "Extracting Windows binary..."
docker run --rm -v $(pwd):/app voipglot-cross cp /app/target/x86_64-pc-windows-msvc/release/voipglot-win.exe /app/

if [ $? -ne 0 ]; then
    echo "Error: Failed to extract binary!"
    exit 1
fi

echo ""
echo "========================================"
echo "Docker cross-compilation completed!"
echo "========================================"
echo ""
echo "Windows executable location:"
echo "  voipglot-win.exe"
echo ""
echo "File size: $(ls -lh voipglot-win.exe | awk '{print $5}')"
echo ""
echo "To test the Windows binary:"
echo "1. Transfer to a Windows machine"
echo "2. Install VB-CABLE Virtual Audio Device on Windows"
echo "3. Set up API keys in environment variables"
echo "4. Run: voipglot-win.exe"
echo ""
