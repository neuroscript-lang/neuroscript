#!/bin/bash
# SessionStart hook: Inject project state context at session start
# Gives Claude immediate awareness of branch, build status, and recent work.
#
# Input: JSON on stdin from Claude Code (SessionStart event)
# Output: Project context as stdout (injected into Claude's context)

cd "$CLAUDE_PROJECT_DIR" 2>/dev/null || exit 0

echo "Project state:"

# Current branch
BRANCH=$(git branch --show-current 2>/dev/null)
[ -n "$BRANCH" ] && echo "  Branch: $BRANCH"

# Uncommitted changes summary
CHANGES=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
if [ "$CHANGES" -gt 0 ]; then
  STAGED=$(git diff --cached --stat 2>/dev/null | tail -1)
  echo "  Uncommitted changes: $CHANGES file(s)"
  [ -n "$STAGED" ] && echo "  Staged: $STAGED"
fi

# Build status
BINARY="./target/release/neuroscript"
if [ -f "$BINARY" ]; then
  STALE=$(find src/ -name '*.rs' -newer "$BINARY" -print -quit 2>/dev/null)
  if [ -n "$STALE" ]; then
    echo "  Build: STALE (source changed since last build)"
  else
    echo "  Build: up to date"
  fi
else
  echo "  Build: no binary (needs cargo build --release)"
fi

# Quick compile check (cargo check is much faster than cargo test --no-run)
CHECK_RESULT=$(cargo check --quiet 2>&1)
if [ $? -ne 0 ]; then
  echo "  Compile: ERRORS present"
else
  echo "  Compile: clean"
fi

# Recent commits (last 3)
echo "  Recent commits:"
git log --oneline -3 2>/dev/null | while read line; do
  echo "    $line"
done

exit 0
