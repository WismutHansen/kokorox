#!/bin/bash
# Wrapper script to handle segfaults from ONNX runtime
#
# This script is a workaround for segmentation faults that can occur
# when the ONNX Runtime library is cleaning up its resources at
# program exit. The script runs koko in a subshell to isolate any
# potential crashes.

# Display a helpful message
echo "Running Kokoros TTS Engine with crash protection wrapper..."
echo "This wrapper helps prevent segmentation faults during program exit."
echo ""

# Determine if we should use debug or release build
if [ -f "./target/release/koko" ]; then
    KOKO_BIN="./target/release/koko"
    echo "Using release build."
elif [ -f "./target/debug/koko" ]; then
    KOKO_BIN="./target/debug/koko"
    echo "Using debug build."
else
    echo "Error: Could not find koko executable in target/release or target/debug."
    echo "Please build the project first with 'cargo build' or 'cargo build --release'."
    exit 1
fi

echo ""

# Capture all command line arguments to pass to the actual program
ARGS="$@"

# Run the program in a subshell
($KOKO_BIN $ARGS)
RESULT=$?

# Regardless of how the program exits, immediately exit with the same code
# This prevents any segfault that might occur during cleanup
exit $RESULT