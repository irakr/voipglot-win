#!/usr/bin/env python3
import sys
import os
import argparse
import shutil
import subprocess

# Check Python version
if sys.version_info < (3, 8):
    print("Error: Python 3.8 or newer is required.", file=sys.stderr)
    sys.exit(1)

# Check for required packages
REQUIRED_PACKAGES = [
    "ctranslate2",
    "transformers",
    "torch",
    "huggingface_hub",
    "sentencepiece"
]
missing = []
for pkg in REQUIRED_PACKAGES:
    try:
        __import__(pkg)
    except ImportError:
        missing.append(pkg)
if missing:
    print(f"Error: Missing required packages: {', '.join(missing)}", file=sys.stderr)
    print("Please install them with: python -m pip install " + ' '.join(missing), file=sys.stderr)
    sys.exit(1)

import torch
from transformers import AutoTokenizer, AutoModelForSeq2SeqLM
from ctranslate2.converters import TransformersConverter

def main():
    parser = argparse.ArgumentParser(description="Download and convert a translation model to CTranslate2 format.")
    parser.add_argument("--model", default="Helsinki-NLP/opus-mt-en-es", help="HuggingFace model name (default: Helsinki-NLP/opus-mt-en-es)")
    parser.add_argument("--output", default="models/nllb-200-ct2", help="Output directory (default: models/nllb-200-ct2)")
    args = parser.parse_args()

    model_name = args.model
    output_dir = args.output
    temp_dir = output_dir + ".tmp"

    print(f"Using model: {model_name}")
    print(f"Output directory: {output_dir}")

    if os.path.exists(output_dir):
        print(f"Model directory already exists: {output_dir}")
        print("Delete it first if you want to re-download and convert.")
        sys.exit(0)

    if os.path.exists(temp_dir):
        shutil.rmtree(temp_dir)
    os.makedirs(temp_dir, exist_ok=True)

    try:
        print("Downloading model and tokenizer from HuggingFace...")
        tokenizer = AutoTokenizer.from_pretrained(model_name)
        model = AutoModelForSeq2SeqLM.from_pretrained(model_name)

        print("Converting model to CTranslate2 format...")
        converter = TransformersConverter(model_name)
        converter.convert(temp_dir, force=True)

        print("Saving tokenizer...")
        tokenizer.save_pretrained(temp_dir)

        print(f"Moving converted files to {output_dir}...")
        shutil.move(temp_dir, output_dir)
        print("Model conversion completed successfully!")
    except Exception as e:
        print(f"Error during model download/conversion: {e}", file=sys.stderr)
        if os.path.exists(temp_dir):
            shutil.rmtree(temp_dir)
        sys.exit(1)

if __name__ == "__main__":
    main() 