#!/bin/bash
# PostToolUse hook: Validate .ns files after edits
# Catches NeuroScript syntax/validation errors immediately.
#
# Input: JSON on stdin from Claude Code (PostToolUse event)
# Output: Validation errors as stdout (injected as Claude context)

set -o pipefail

INPUT=$(cat)

# Extract file path from tool input
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null)

# Only run for .ns files
[[ "$FILE_PATH" != *.ns ]] && exit 0

# Check if the binary exists
BINARY="./target/release/neuroscript"
if [ ! -f "$BINARY" ]; then
  echo "neuroscript binary not found — skipping .ns validation (run cargo build --release)"
  exit 0
fi

# Run validation
RESULT=$("$BINARY" validate "$FILE_PATH" 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
  ERRORS=$(echo "$RESULT" | head -20)
  echo "NeuroScript validation failed for $(basename "$FILE_PATH"):"
  echo "$ERRORS"
fi

# Always exit 0 — informational
exit 0
