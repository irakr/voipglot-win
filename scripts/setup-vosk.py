#!/usr/bin/env python3
import sys
import os
import argparse
import shutil
import urllib.request
import zipfile
from urllib.parse import urlparse

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

def get_model_name_from_url(url):
    """Extract the model name from the URL."""
    parsed_url = urlparse(url)
    filename = os.path.basename(parsed_url.path)
    
    # Remove the .zip extension if present
    if filename.endswith('.zip'):
        model_name = filename[:-4]
    else:
        model_name = filename
    
    return model_name

def main():
    parser = argparse.ArgumentParser(description="Download and extract a VOSK model.")
    parser.add_argument("--url", required=True, help="VOSK model URL")
    parser.add_argument("--output-dir", help="Output directory where the model will be extracted (default: models/ in current directory)")
    args = parser.parse_args()

    # Determine output directory
    if args.output_dir:
        # Use the specified directory and append the model name
        model_name = get_model_name_from_url(args.url)
        output_dir = os.path.join(args.output_dir, model_name)
    else:
        # Default behavior: use models/ directory in current directory
        model_name = get_model_name_from_url(args.url)
        output_dir = f"models/{model_name}"
    
    print(f"Output directory: {output_dir}")
    
    if os.path.exists(output_dir):
        print(f"Model directory already exists: {output_dir}")
        print("Delete it first if you want to re-download.")
        sys.exit(0)

    os.makedirs(os.path.dirname(output_dir), exist_ok=True)
    download_and_extract(args.url, output_dir)

if __name__ == "__main__":
    main() 