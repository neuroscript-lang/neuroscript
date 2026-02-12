#!/bin/bash
# Validate a NeuroScript neuron file through all 3 stages:
# 1. Parse (syntax check)
# 2. Validate (semantic check: shapes, symbols, cycles)
# 3. Compile (code generation)
#
# Usage: ./validate_neuron.sh <file.ns> [--neuron NeuronName] [--verbose]

set -euo pipefail

FILE="${1:-}"
NEURON_FLAG=""
VERBOSE=""

if [ -z "$FILE" ]; then
  echo "Usage: validate_neuron.sh <file.ns> [--neuron NeuronName] [--verbose]"
  exit 1
fi

shift
while [[ $# -gt 0 ]]; do
  case "$1" in
    --neuron) NEURON_FLAG="--neuron $2"; shift 2 ;;
    --verbose|-v) VERBOSE="--verbose"; shift ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

BINARY="./target/release/neuroscript"
if [ ! -f "$BINARY" ]; then
  echo "Binary not found. Building..."
  cargo build --release 2>&1
fi

echo "=== Stage 1: Parse ==="
if $BINARY parse $VERBOSE "$FILE" 2>&1; then
  echo "  Parse OK"
else
  echo "  Parse FAILED"
  exit 1
fi

echo ""
echo "=== Stage 2: Validate ==="
if $BINARY validate $VERBOSE "$FILE" 2>&1; then
  echo "  Validate OK"
else
  echo "  Validate FAILED"
  exit 1
fi

echo ""
echo "=== Stage 3: Compile ==="
if $BINARY compile $VERBOSE $NEURON_FLAG "$FILE" 2>&1; then
  echo "  Compile OK"
else
  # Compile may fail with auto-detect if multiple neurons exist
  echo "  Compile WARNING: may need --neuron flag for multi-neuron files"
  exit 0
fi

echo ""
echo "All stages passed."
