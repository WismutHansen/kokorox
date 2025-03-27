#!/bin/bash
# Wrapper script to handle segfaults from ONNX runtime

# Capture all command line arguments to pass to the actual program
ARGS="$@"

# Run the program in a subshell
(./target/debug/koko $ARGS)

# Regardless of how the program exits, immediately exit with the same code
# This prevents any segfault that might occur during cleanup
exit $?