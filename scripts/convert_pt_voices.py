#!/usr/bin/env python3
"""
Voice converter for Kokoros TTS

This script converts PyTorch .pt voice files from huggingface.co/hexgrad/Kokoro-82M
to the format used by Kokoros (ONNX version)
"""

import os
import sys
import argparse
import json
import requests
import torch
import numpy as np
from pathlib import Path
import tqdm
import time

# URLs for Hugging Face API
HF_API_FILES = "https://huggingface.co/api/models/hexgrad/Kokoro-82M/tree/langs"
HF_RAW_FILE_URL = "https://huggingface.co/hexgrad/Kokoro-82M/resolve/main/{path}"

# Local storage
DOWNLOAD_DIR = "data/pt_voices"
OUTPUT_DIR = "data/voices"
COMBINED_OUTPUT = "data/voices-custom.bin"

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


def list_available_languages():
    """List all available languages on Hugging Face."""
    try:
        response = requests.get(HF_API_FILES)
        response.raise_for_status()
        
        # Extract language directories
        languages = []
        for item in response.json():
            if item["type"] == "directory":
                lang_code = item["path"].split("/")[-1]
                languages.append(lang_code)
                
        return sorted(languages)
    except Exception as e:
        print(f"Error fetching languages: {e}")
        return []


def list_voices_for_language(lang_code):
    """List all voices available for a specific language."""
    try:
        response = requests.get(f"{HF_API_FILES}/{lang_code}")
        response.raise_for_status()
        
        # Extract voice files
        voices = []
        for item in response.json():
            if item["type"] == "file" and item["path"].endswith(".pt"):
                voice_name = os.path.basename(item["path"]).replace(".pt", "")
                voices.append(voice_name)
                
        return sorted(voices)
    except Exception as e:
        print(f"Error fetching voices for {lang_code}: {e}")
        return []


def download_pt_voice(lang_code, voice_name):
    """Download a PyTorch voice file from Hugging Face."""
    voice_path = f"langs/{lang_code}/{voice_name}.pt"
    output_path = f"{DOWNLOAD_DIR}/{lang_code}/{voice_name}.pt"
    
    # Create directory if it doesn't exist
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    # Check if file already exists
    if os.path.exists(output_path):
        print(f"Voice file already exists at {output_path}")
        return output_path
    
    # Download the file
    url = HF_RAW_FILE_URL.format(path=voice_path)
    try:
        download_file(url, output_path, f"Downloading {lang_code}/{voice_name}")
        print(f"Downloaded voice file to {output_path}")
        return output_path
    except Exception as e:
        print(f"Error downloading voice {voice_name}: {e}")
        return None


def convert_pt_to_npz(pt_file_path, lang_code, voice_name):
    """Convert a PyTorch voice file to NPZ format."""
    output_path = f"{OUTPUT_DIR}/{lang_code}/{voice_name}.npz"
    
    # Create directory if it doesn't exist
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    try:
        # Load the PyTorch state dict
        state_dict = torch.load(pt_file_path, map_location=torch.device('cpu'))
        
        # Extract the spkr_emb tensor
        if 'spkr_emb' in state_dict:
            # Convert to numpy and reshape to the expected format
            style_tensor = state_dict['spkr_emb'].detach().numpy()
            
            # Reshape to the format used by Kokoros
            # The format used by the combined binaries is 511x1x256
            # Where each item is a tensor of size 1x256
            # Create a 511x1x256 tensor filled with zeros
            reshaped_tensor = np.zeros((511, 1, 256), dtype=np.float32)
            
            # Fill each index with the same style vector
            # This assumes the voice embedding is a 1D tensor of size 256
            for i in range(511):
                reshaped_tensor[i, 0, :] = style_tensor.flatten()[:256]
            
            # Save as npz
            np.savez(output_path, **{voice_name: reshaped_tensor})
            print(f"Converted to NPZ format: {output_path}")
            return output_path
        else:
            print(f"Error: 'spkr_emb' not found in {pt_file_path}")
            return None
    except Exception as e:
        print(f"Error converting {pt_file_path}: {e}")
        return None


def combine_npz_to_bin(output_path=COMBINED_OUTPUT):
    """Combine all NPZ files into a single binary file."""
    voices_dict = {}
    
    # Find all NPZ files
    for root, dirs, files in os.walk(OUTPUT_DIR):
        for file in files:
            if file.endswith('.npz'):
                file_path = os.path.join(root, file)
                voice_name = os.path.splitext(file)[0]
                
                # Extract relative path from OUTPUT_DIR to get the voice name
                rel_path = os.path.relpath(file_path, OUTPUT_DIR)
                parts = rel_path.split(os.path.sep)
                if len(parts) > 1:
                    # Include language code in voice name
                    voice_name = f"{parts[0]}_{parts[1].replace('.npz', '')}"
                
                try:
                    # Load the NPZ file
                    data = np.load(file_path)
                    
                    # Get the first (and only) array in the file
                    array_name = list(data.keys())[0]
                    voices_dict[voice_name] = data[array_name]
                except Exception as e:
                    print(f"Error loading {file_path}: {e}")
    
    if not voices_dict:
        print("No voices found to combine.")
        return False
    
    # Create directory if it doesn't exist
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    # Save to a single NPZ file
    try:
        np.savez(output_path, **voices_dict)
        print(f"Combined {len(voices_dict)} voices into {output_path}")
        print(f"Voices included: {list(voices_dict.keys())}")
        return True
    except Exception as e:
        print(f"Error combining voices: {e}")
        return False


def main():
    parser = argparse.ArgumentParser(description="Convert PyTorch voice files to Kokoros format")
    
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--list-languages", action="store_true", help="List available languages")
    group.add_argument("--list-voices", metavar="LANG", help="List available voices for a language")
    group.add_argument("--download", nargs=2, metavar=("LANG", "VOICE"), help="Download a specific voice")
    group.add_argument("--download-all", action="store_true", help="Download all available voices")
    group.add_argument("--convert", nargs=2, metavar=("LANG", "VOICE"), help="Convert a downloaded PT voice to NPZ")
    group.add_argument("--convert-all", action="store_true", help="Convert all downloaded PT voices to NPZ")
    group.add_argument("--combine", action="store_true", help="Combine all NPZ voices into a single binary file")
    group.add_argument("--all", action="store_true", help="Download, convert, and combine all voices")
    
    args = parser.parse_args()
    
    # Create directories
    os.makedirs(DOWNLOAD_DIR, exist_ok=True)
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    
    if args.list_languages:
        languages = list_available_languages()
        print("\nAvailable languages:")
        for lang in languages:
            print(f"- {lang}")
        return
    
    elif args.list_voices:
        voices = list_voices_for_language(args.list_voices)
        print(f"\nVoices available for {args.list_voices}:")
        for voice in voices:
            print(f"- {voice}")
        return
    
    elif args.download:
        lang, voice = args.download
        download_pt_voice(lang, voice)
        return
    
    elif args.download_all:
        languages = list_available_languages()
        for lang in languages:
            voices = list_voices_for_language(lang)
            for voice in voices:
                download_pt_voice(lang, voice)
        return
    
    elif args.convert:
        lang, voice = args.convert
        pt_file = f"{DOWNLOAD_DIR}/{lang}/{voice}.pt"
        if os.path.exists(pt_file):
            convert_pt_to_npz(pt_file, lang, voice)
        else:
            print(f"Error: {pt_file} not found. Download it first.")
        return
    
    elif args.convert_all:
        for root, dirs, files in os.walk(DOWNLOAD_DIR):
            for file in files:
                if file.endswith('.pt'):
                    # Extract language and voice from the path
                    rel_path = os.path.relpath(root, DOWNLOAD_DIR)
                    lang = rel_path
                    voice = os.path.splitext(file)[0]
                    
                    # Convert the file
                    pt_file = os.path.join(root, file)
                    convert_pt_to_npz(pt_file, lang, voice)
        return
    
    elif args.combine:
        combine_npz_to_bin()
        return
    
    elif args.all:
        print("=== Performing complete workflow: download, convert, and combine ===")
        
        # Step 1: Download all voices
        print("\nStep 1: Downloading all voices...")
        languages = list_available_languages()
        for lang in languages:
            voices = list_voices_for_language(lang)
            for voice in voices:
                download_pt_voice(lang, voice)
        
        # Step 2: Convert all voices
        print("\nStep 2: Converting all voices...")
        for root, dirs, files in os.walk(DOWNLOAD_DIR):
            for file in files:
                if file.endswith('.pt'):
                    # Extract language and voice from the path
                    rel_path = os.path.relpath(root, DOWNLOAD_DIR)
                    lang = rel_path
                    voice = os.path.splitext(file)[0]
                    
                    # Convert the file
                    pt_file = os.path.join(root, file)
                    convert_pt_to_npz(pt_file, lang, voice)
        
        # Step 3: Combine all voices
        print("\nStep 3: Combining all voices...")
        combine_npz_to_bin()
        
        print("\nWorkflow complete!")
        print(f"The combined voice file is available at: {COMBINED_OUTPUT}")
        print("You can use it with Kokoros by specifying:")
        print(f"./target/release/koko -d {COMBINED_OUTPUT} text \"Your text here\"")
        return
    
    else:
        parser.print_help()


if __name__ == "__main__":
    main()