#!/bin/bash
# PostToolUse hook: Run cargo check after .rs file edits
# Catches compilation errors immediately instead of letting them cascade.
#
# Input: JSON on stdin from Claude Code (PostToolUse event)
# Output: Compilation errors as stdout (injected as Claude context)

set -o pipefail

INPUT=$(cat)

# Extract file path from tool input
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null)

# Only run for .rs files
[[ "$FILE_PATH" != *.rs ]] && exit 0

# Run cargo check (incremental, usually fast)
RESULT=$(cargo check --quiet 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
  # Output errors so Claude sees them as context and can fix immediately
  ERRORS=$(echo "$RESULT" | head -30)
  echo "cargo check failed after editing $(basename "$FILE_PATH"):"
  echo "$ERRORS"
fi

# Always exit 0 — this is informational, not blocking
exit 0
