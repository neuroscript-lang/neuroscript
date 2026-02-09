#!/bin/bash
# log-response-time.sh — Stop hook
# Logs when Claude finishes responding. Outputs a systemMessage
# so the timestamp appears in the transcript.

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')
STOP_ACTIVE=$(echo "$INPUT" | jq -r '.stop_hook_active // false')

# Don't re-trigger if we're already in a stop-hook continuation
if [ "$STOP_ACTIVE" = "true" ]; then
  exit 0
fi

NOW_UTC=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
NOW_LOCAL=$(date +"%Y-%m-%d %H:%M:%S %Z")

# Log to file for audit trail
LOG_DIR="${HOME}/.claude/logs"
mkdir -p "$LOG_DIR"
echo "${NOW_LOCAL} | session=${SESSION_ID} | response_completed" >> "${LOG_DIR}/datetime-hook.log"

# No need to block stopping — just log and allow
exit 0
