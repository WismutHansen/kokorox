#!/bin/bash
# All-in-one download script for Kokoros TTS

echo "=== Kokoros TTS Download Script ==="
echo "This script will download all required resources and build the project."
echo ""

# Step 1: Ensure Python requirements are installed
echo "Step 1: Installing Python requirements..."
pip install -r scripts/requirements.txt
if [ $? -ne 0 ]; then
    echo "Error installing Python requirements. Please check your Python installation."
    exit 1
fi
echo "Python requirements installed successfully."
echo ""

# Step 2: Download model and voices
echo "Step 2: Downloading model and voices..."
python scripts/download_voices.py --all
if [ $? -ne 0 ]; then
    echo "Error downloading resources. Please check your internet connection."
    exit 1
fi
echo ""

# Step 3: Build the project if requested
if [ "$1" == "--build" ]; then
    echo "Step 3: Building Kokoros (release mode)..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "Error building Kokoros. Please check your Rust installation."
        exit 1
    fi
    echo "Build completed successfully."
    echo ""
    echo "You can now run Kokoros with:"
    echo "./target/release/koko -h"
else
    echo "To build the project, run:"
    echo "cargo build --release"
    echo ""
    echo "Or run this script with the --build flag:"
    echo "./download_all.sh --build"
fi