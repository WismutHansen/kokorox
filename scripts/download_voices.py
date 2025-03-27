#!/usr/bin/env python3
"""
Voice and model downloader for Kokoros TTS
This script downloads the required files from the official sources
"""

import os
import sys
import argparse
import json
import requests
from pathlib import Path
import tqdm
import time

# URLs for resources
RESOURCES = {
    "model": {
        "url": "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/kokoro-v1.0.onnx",
        "path": "checkpoints/kokoro-v1.0.onnx",
        "desc": "Kokoro model file (ONNX format)"
    },
    "voices": {
        "url": "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/voices-v1.0.bin",
        "path": "data/voices-v1.0.bin",
        "desc": "Voices data file (contains all voices)"
    },
    "segmentation": {
        "url": "https://raw.githubusercontent.com/thewh1teagle/kokoro-onnx/main/examples/en-sent.bin",
        "path": "data/en-sent.bin",
        "desc": "English sentence segmentation model"
    }
}

# Information about supported languages (for documentation purposes)
SUPPORTED_LANGUAGES = {
    "en": "English",
    "zh": "Chinese",
    "ja": "Japanese",
    "de": "German",
    "fr": "French",
    "es": "Spanish",
    "pt": "Portuguese",
    "ru": "Russian",
    "ko": "Korean",
    "ar": "Arabic",
    "hi": "Hindi"
}

def download_file(url, output_path, desc=None):
    """Download a file with progress tracking."""
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    response = requests.get(url, stream=True)
    response.raise_for_status()
    
    total_size = int(response.headers.get('content-length', 0))
    block_size = 1024  # 1 Kibibyte
    
    desc = desc or f"Downloading {os.path.basename(output_path)}"
    
    with open(output_path, 'wb') as file, tqdm.tqdm(
            desc=desc,
            total=total_size,
            unit='iB',
            unit_scale=True,
            unit_divisor=1024,
    ) as bar:
        for data in response.iter_content(block_size):
            size = file.write(data)
            bar.update(size)


def download_resource(resource_key):
    """Download a specific resource."""
    resource = RESOURCES[resource_key]
    output_path = resource["path"]
    
    # Create directory if it doesn't exist
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    # Check if file already exists
    if os.path.exists(output_path):
        file_size = os.path.getsize(output_path)
        if file_size > 0:
            print(f"{resource['desc']} already exists at {output_path} ({file_size} bytes)")
            return True
    
    # Download the file
    try:
        print(f"Downloading {resource['desc']}...")
        download_file(resource["url"], output_path, f"Downloading {resource['desc']}")
        print(f"Downloaded {resource['desc']} to {output_path}")
        return True
    except Exception as e:
        print(f"Error downloading {resource_key}: {e}")
        return False


def list_supported_languages():
    """List all supported languages."""
    print("\nKokoros supports the following languages:")
    print("=" * 40)
    
    for code, name in SUPPORTED_LANGUAGES.items():
        print(f"  {name} ({code})")
    
    print("\nTo use these languages:")
    print("1. Manual selection: --lan <language-code>")
    print("   Example: ./target/release/koko text \"Hello\" --lan en")
    print("   Example: ./target/release/koko text \"你好\" --lan zh")
    print("\n2. Automatic detection: --auto-detect or -a")
    print("   Example: ./target/release/koko -a text \"Hello\"")


def main():
    parser = argparse.ArgumentParser(description="Download resources for Kokoros TTS")
    
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--all", action="store_true", help="Download all required resources (model, voices, etc.)")
    group.add_argument("--model", action="store_true", help="Download only the model file")
    group.add_argument("--voices", action="store_true", help="Download only the voices data file")
    group.add_argument("--list-languages", action="store_true", help="List supported languages")
    
    args = parser.parse_args()
    
    # Default to downloading all if no specific option is provided
    if not (args.all or args.model or args.voices or args.list_languages):
        args.all = True
    
    if args.list_languages:
        list_supported_languages()
        return
    
    # Create directories
    os.makedirs("data", exist_ok=True)
    os.makedirs("checkpoints", exist_ok=True)
    
    if args.all:
        print("Downloading all required resources for Kokoros TTS...")
        for resource_key in RESOURCES:
            download_resource(resource_key)
    else:
        if args.model:
            download_resource("model")
        if args.voices:
            download_resource("voices")
    
    print("\nDownload complete!")
    print("You can now build and run Kokoros:")
    print("  cargo build --release")
    print("  ./target/release/koko -h")


if __name__ == "__main__":
    main()