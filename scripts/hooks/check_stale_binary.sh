#!/bin/bash
# PreToolUse hook: Warn when running neuroscript CLI with a stale binary
# Prevents confusing results from a binary that doesn't reflect recent code changes.
#
# Input: JSON on stdin from Claude Code (PreToolUse event)
# Output: Warning as stdout if binary is stale (injected as Claude context)

set -o pipefail

INPUT=$(cat)

# Extract command from Bash tool input
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)

# Only check when running the neuroscript binary
[[ "$COMMAND" != *"neuroscript"* ]] && exit 0
# Skip if it's a cargo command (building, testing, etc.)
[[ "$COMMAND" == *"cargo"* ]] && exit 0

BINARY="./target/release/neuroscript"

# If binary doesn't exist at all, warn
if [ ! -f "$BINARY" ]; then
  echo "WARNING: neuroscript binary does not exist. Run 'cargo build --release' first."
  exit 0
fi

# Compare binary mtime against newest .rs file in src/
# macOS stat: -f %m gives epoch seconds
BINARY_MTIME=$(stat -f %m "$BINARY" 2>/dev/null || echo 0)

# Find the most recently modified .rs file
NEWEST_RS=$(find src/ -name '*.rs' -newer "$BINARY" -print -quit 2>/dev/null)

if [ -n "$NEWEST_RS" ]; then
  echo "WARNING: neuroscript binary is stale ($(basename "$NEWEST_RS") modified after last build). Consider running 'cargo build --release' before using the CLI."
fi

exit 0
