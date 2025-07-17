#!/usr/bin/env python3
"""
Coqui TTS Setup Script for VoipGlot
====================================

This script sets up the Python TTS dependencies required for Coqui TTS integration.
"""

import sys
import subprocess
import os
from pathlib import Path

def check_python_version():
    """Check if Python version is compatible."""
    if sys.version_info < (3, 7):
        print("Error: Python 3.7 or higher is required")
        print(f"Current version: {sys.version}")
        return False
    print(f"Python version: {sys.version}")
    return True

def install_tts():
    """Install TTS package."""
    print("Installing TTS package...")
    try:
        subprocess.check_call([sys.executable, "-m", "pip", "install", "--user", "TTS"])
        print("TTS package installed successfully")
        return True
    except subprocess.CalledProcessError as e:
        print(f"Error installing TTS: {e}")
        return False

def verify_tts_installation():
    """Verify TTS installation."""
    print("Verifying TTS installation...")
    try:
        import TTS
        print(f"TTS version: {TTS.__version__}")
        return True
    except ImportError as e:
        print(f"Error importing TTS: {e}")
        return False

def setup_environment():
    """Set up environment variables."""
    print("Setting up environment variables...")
    
    # Get user site-packages directory
    try:
        import site
        user_site = site.getusersitepackages()
        print(f"User site-packages: {user_site}")
        
        # Set PYTHONPATH environment variable
        os.environ['PYTHONPATH'] = user_site
        print(f"PYTHONPATH set to: {user_site}")
        
        return True
    except Exception as e:
        print(f"Error setting up environment: {e}")
        return False

def main():
    """Main setup function."""
    print("=" * 50)
    print("Coqui TTS Setup for VoipGlot")
    print("=" * 50)
    
    # Check Python version
    if not check_python_version():
        sys.exit(1)
    
    # Install TTS if not already installed
    if not verify_tts_installation():
        if not install_tts():
            sys.exit(1)
        if not verify_tts_installation():
            print("Failed to verify TTS installation after install")
            sys.exit(1)
    
    # Set up environment
    if not setup_environment():
        print("Warning: Environment setup failed, but TTS is installed")
    
    print("=" * 50)
    print("Coqui TTS setup completed successfully!")
    print("=" * 50)
    print("Note: TTS models will be downloaded automatically on first use.")
    print("The default model path is: tts_models/en/ljspeech/fast_pitch")
    
    return 0

if __name__ == "__main__":
    sys.exit(main()) 