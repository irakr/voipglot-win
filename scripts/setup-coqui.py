#!/usr/bin/env python3
"""
Coqui TTS Setup Script for VoipGlot
====================================

This script sets up the Python TTS dependencies and downloads TTS models locally.
"""

import sys
import subprocess
import os
import shutil
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

def download_tts_model(model_name, local_models_dir):
    """Download and cache TTS model locally."""
    print(f"Downloading TTS model: {model_name}")
    
    # Create local models directory structure
    model_path = Path(local_models_dir) / model_name
    model_path.mkdir(parents=True, exist_ok=True)
    
    try:
        from TTS.api import TTS
        
        # Initialize TTS with the model - this will download it to default cache
        print(f"Initializing TTS model {model_name}...")
        tts = TTS(model_name)
        
        # Get the model's cache location
        print("Model downloaded to cache successfully")
        
        # Find the cached model files
        import os
        cache_dirs = [
            os.path.expanduser("~/.local/share/tts"),  # Linux
            os.path.expanduser("~/Library/Application Support/tts"),  # Mac
            os.path.expanduser("~/AppData/Local/tts"),  # Windows
        ]
        
        cached_model_path = None
        for cache_dir in cache_dirs:
            potential_path = Path(cache_dir) / model_name.replace("/", "--")
            if potential_path.exists():
                cached_model_path = potential_path
                break
        
        if cached_model_path and cached_model_path.exists():
            print(f"Found cached model at: {cached_model_path}")
            
            # Copy cached files to local models directory
            if model_path.exists():
                shutil.rmtree(model_path)
            shutil.copytree(cached_model_path, model_path)
            
            print(f"Model copied to local directory: {model_path}")
            return True
        else:
            print("Warning: Could not locate cached model files")
            print("Model will be downloaded at runtime from cache")
            return True
            
    except Exception as e:
        print(f"Error downloading model {model_name}: {e}")
        return False

def setup_environment():
    """Set up environment variables."""
    print("Setting up environment variables...")
    
    # Get user site-packages directory
    try:
        import site
        user_site = site.getusersitepackages()
        print(f"User site-packages: {user_site}")
        
        # Add to Python path if needed
        if user_site not in sys.path:
            sys.path.append(user_site)
            
    except Exception as e:
        print(f"Warning: Could not set up user site-packages: {e}")

def main():
    """Main setup function."""
    print("=== Coqui TTS Setup ===")
    
    # Check Python version
    if not check_python_version():
        return 1
    
    # Install TTS
    if not install_tts():
        return 1
    
    # Verify installation
    if not verify_tts_installation():
        return 1
    
    # Set up environment
    setup_environment()
    
    # Download default TTS model
    models_dir = Path("models")
    models_dir.mkdir(exist_ok=True)
    
    # Define models to download
    default_models = [
        "tts_models/en/ljspeech/fast_pitch",  # Fast English model
        # Add more models here if needed
    ]
    
    print("\n=== Downloading TTS Models ===")
    for model_name in default_models:
        if not download_tts_model(model_name, models_dir):
            print(f"Warning: Failed to download {model_name}")
            print("Model will be downloaded at runtime")
    
    print("\n=== Coqui TTS Setup Complete ===")
    print("TTS models are now cached locally for offline use")
    return 0

if __name__ == "__main__":
    sys.exit(main()) 