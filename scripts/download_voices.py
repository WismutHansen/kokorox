#!/usr/bin/env python3
"""
Voice downloader for Kokoros TTS
This script downloads voice files from Hugging Face model repository
"""

import os
import sys
import argparse
import json
import requests
from typing import Dict, List, Optional
from pathlib import Path
import shutil
import tqdm


# URLs for voice files
VOICES_INFO_URL = "https://huggingface.co/hexgrad/Kokoro-82M/raw/main/langs/langs.json"
VOICE_BASE_URL = "https://huggingface.co/hexgrad/Kokoro-82M/resolve/main/langs/{lang}/{voice}.npz"
DEFAULT_VOICES_BIN = "data/voices-v1.0.bin"  # Default binary file containing all voices
VOICES_DIR = "data/voices"  # Directory to store individual voice files


def download_file(url: str, output_path: str, desc: Optional[str] = None) -> None:
    """Download a file from a URL with progress bar."""
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


def get_available_voices() -> Dict:
    """Get information about available voices from Hugging Face."""
    try:
        response = requests.get(VOICES_INFO_URL)
        response.raise_for_status()
        return response.json()
    except requests.RequestException as e:
        print(f"Error fetching voice information: {e}")
        # Create a basic fallback structure if API fails
        return {
            "languages": {
                "en": {"name": "English", "voices": ["af_sarah", "af_nicole"]},
                "zh": {"name": "Chinese", "voices": ["zf_xiaoxiao"]},
                "ja": {"name": "Japanese", "voices": ["jf_alpha"]},
                "de": {"name": "German", "voices": ["bf_emma"]}
            }
        }


def download_voice(lang: str, voice: str, voices_dir: str) -> str:
    """Download a specific voice file."""
    output_path = os.path.join(voices_dir, lang, f"{voice}.npz")
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    voice_url = VOICE_BASE_URL.format(lang=lang, voice=voice)
    
    try:
        download_file(voice_url, output_path, f"Downloading {lang}/{voice}")
        return output_path
    except requests.RequestException as e:
        print(f"Error downloading voice {voice}: {e}")
        return ""


def download_all_voices() -> str:
    """Download the combined voices binary file."""
    os.makedirs("data", exist_ok=True)
    output_path = DEFAULT_VOICES_BIN
    
    try:
        download_file(
            "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/voices-v1.0.bin", 
            output_path,
            "Downloading all voices (combined file)"
        )
        return output_path
    except requests.RequestException as e:
        print(f"Error downloading voices: {e}")
        return ""


def download_voices_by_language(langs: List[str], voices_dir: str) -> List[str]:
    """Download all available voices for specified languages."""
    voices_info = get_available_voices()
    downloaded_voices = []
    
    for lang in langs:
        if lang in voices_info["languages"]:
            lang_info = voices_info["languages"][lang]
            print(f"\nDownloading {lang_info['name']} voices:")
            
            for voice in lang_info["voices"]:
                voice_path = download_voice(lang, voice, voices_dir)
                if voice_path:
                    downloaded_voices.append(voice_path)
        else:
            print(f"Language '{lang}' not found in available voices")
    
    return downloaded_voices


def list_available_languages() -> None:
    """List all available languages and their voices."""
    voices_info = get_available_voices()
    
    print("\nAvailable languages and voices:")
    print("==============================")
    
    for lang_code, lang_info in voices_info["languages"].items():
        print(f"\n{lang_info['name']} ({lang_code}):")
        for voice in lang_info["voices"]:
            print(f"  - {voice}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Download voice files for Kokoros TTS")
    
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--all", action="store_true", help="Download the combined voices file (default approach)")
    group.add_argument("--lang", nargs="+", help="Download voices for specific languages (e.g., en zh ja)")
    group.add_argument("--list", action="store_true", help="List available languages and voices")
    
    parser.add_argument("--output-dir", default=VOICES_DIR, help="Directory to store individual voice files")
    
    args = parser.parse_args()
    
    if args.list:
        list_available_languages()
        return
    
    if args.all:
        voices_path = download_all_voices()
        if voices_path:
            print(f"\nAll voices downloaded to {voices_path}")
            print("You can now use Kokoros with the default voice file")
    
    if args.lang:
        voices = download_voices_by_language(args.lang, args.output_dir)
        if voices:
            print(f"\n{len(voices)} voices downloaded to {args.output_dir}")
            print("To use these voices, update the --data parameter in koko command")


if __name__ == "__main__":
    main()