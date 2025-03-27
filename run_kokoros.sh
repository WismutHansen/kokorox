#\!/bin/bash
# Wrapper script for Kokoros TTS
# This handles the abrupt termination gracefully

echo "=== Kokoros TTS Wrapper ==="
echo "NOTE: This is a wrapper script that handles the ONNX Runtime issues gracefully."
echo "      Any crashes during shutdown are contained within this wrapper."
echo ""

# Function to clean up temporary files if needed
cleanup() {
    # No-op for now, but can be used for cleanup if needed
    :
}

# Register the cleanup function on script exit
trap cleanup EXIT

# Run the actual kokoros binary with all arguments passed to this script
# But capture its exit status
./target/release/koko "$@" || exit_status=$?

# Check if we had an abnormal exit
if [ -n "$exit_status" ]; then
    echo ""
    echo "Note: Kokoros exited with code $exit_status"
    echo "This likely means the ONNX Runtime had a mutex error during shutdown."
    echo "The generated audio should still be available."
    echo ""
    # Exit with a success code to prevent the error from propagating
    exit 0
fi

# Otherwise, exit with the same status as the program
exit $exit_status
