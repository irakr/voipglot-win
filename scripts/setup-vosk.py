#!/usr/bin/env python3
import sys
import os
import argparse
import shutil
import urllib.request
import zipfile

def download_and_extract(url, output_dir):
    zip_path = output_dir + ".zip"
    try:
        print(f"Downloading model from {url} ...")
        urllib.request.urlretrieve(url, zip_path)
        print(f"Extracting model to {output_dir} ...")
        with zipfile.ZipFile(zip_path, 'r') as zip_ref:
            zip_ref.extractall(os.path.dirname(output_dir))
        os.remove(zip_path)
        print("Model downloaded and extracted successfully!")
    except Exception as e:
        print(f"Error during download/extract: {e}", file=sys.stderr)
        if os.path.exists(zip_path):
            os.remove(zip_path)
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description="Download and extract a VOSK model.")
    parser.add_argument("--url", default="https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip", help="VOSK model URL (default: small English)")
    parser.add_argument("--output", default="models/vosk-model-small-en-us-0.15", help="Output directory (default: models/vosk-model-small-en-us-0.15)")
    args = parser.parse_args()

    output_dir = args.output
    if os.path.exists(output_dir):
        print(f"Model directory already exists: {output_dir}")
        print("Delete it first if you want to re-download.")
        sys.exit(0)

    os.makedirs(os.path.dirname(output_dir), exist_ok=True)
    download_and_extract(args.url, output_dir)

if __name__ == "__main__":
    main() 