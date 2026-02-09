#!/bin/bash
# inject-datetime.sh — UserPromptSubmit hook
# Injects current datetime as additionalContext so Claude sees
# the timestamp of each user prompt.

INPUT=$(cat)
PROMPT=$(echo "$INPUT" | jq -r '.prompt // empty')

NOW_UTC=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
NOW_LOCAL=$(date +"%Y-%m-%d %H:%M:%S %Z")

jq -n \
  --arg ctx "[Prompt submitted at: $NOW_LOCAL (UTC: $NOW_UTC)]" \
  '{
    hookSpecificOutput: {
      hookEventName: "UserPromptSubmit",
      additionalContext: $ctx
    }
  }'
