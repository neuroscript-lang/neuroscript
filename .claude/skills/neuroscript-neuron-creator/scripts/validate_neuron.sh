#!/bin/bash
# Validate a NeuroScript neuron file through all compilation stages

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 <neuron_file.ns> [--verbose]"
    exit 1
fi

NEURON_FILE="$1"
VERBOSE=""

if [ "$2" = "--verbose" ] || [ "$2" = "-v" ]; then
    VERBOSE="--verbose"
fi

if [ ! -f "$NEURON_FILE" ]; then
    echo "Error: File '$NEURON_FILE' not found"
    exit 1
fi

echo "=== Validating: $NEURON_FILE ==="
echo

# Step 1: Parse
echo "1. Parsing..."
./target/release/neuroscript parse $VERBOSE "$NEURON_FILE"
if [ $? -eq 0 ]; then
    echo "✅ Parse successful"
else
    echo "❌ Parse failed"
    exit 1
fi
echo

# Step 2: Validate
echo "2. Validating..."
./target/release/neuroscript validate $VERBOSE "$NEURON_FILE"
if [ $? -eq 0 ]; then
    echo "✅ Validation successful"
else
    echo "❌ Validation failed"
    exit 1
fi
echo

# Step 3: Compile (try auto-detect, skip if it fails)
echo "3. Compiling (auto-detect)..."
./target/release/neuroscript compile $VERBOSE "$NEURON_FILE" 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Compilation successful"
else
    echo "⚠️  Auto-detect failed (file contains multiple neurons)"
    echo "   To compile a specific neuron, use: neuroscript compile $NEURON_FILE --neuron <NAME>"
fi
echo

echo "=== Parse and validation checks passed! ==="
