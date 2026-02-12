#!/bin/bash
# SessionStart hook: Inject project state + context bundle references
# Gives Claude immediate awareness of branch, build status, recent work,
# and pointers to generated context files for deeper exploration.
#
# Input: JSON on stdin from Claude Code (SessionStart event)
# Output: Project context as stdout (injected into Claude's context)

cd "$CLAUDE_PROJECT_DIR" 2>/dev/null || exit 0

CONTEXT_DIR="$CLAUDE_PROJECT_DIR/.claude/context"
GEN_SCRIPT="$CLAUDE_PROJECT_DIR/scripts/generate-context.sh"

# Regenerate context if artifacts are stale (older than last commit)
if [ -f "$GEN_SCRIPT" ]; then
  NEEDS_REGEN=false

  # Check if context files exist at all
  if [ ! -f "$CONTEXT_DIR/ir-types-summary.md" ] || [ ! -f "$CONTEXT_DIR/recent-changes.md" ]; then
    NEEDS_REGEN=true
  else
    # Check if any context file is older than the last commit
    LAST_COMMIT_TIME=$(git log -1 --format=%ct 2>/dev/null || echo "0")
    for f in "$CONTEXT_DIR"/*.md; do
      if [ -f "$f" ]; then
        FILE_TIME=$(stat -f %m "$f" 2>/dev/null || echo "0")
        if [ "$FILE_TIME" -lt "$LAST_COMMIT_TIME" ]; then
          NEEDS_REGEN=true
          break
        fi
      fi
    done
  fi

  if [ "$NEEDS_REGEN" = true ]; then
    "$GEN_SCRIPT" all >/dev/null 2>&1 || true
  fi
fi

echo "Project state:"

# Current branch
BRANCH=$(git branch --show-current 2>/dev/null)
[ -n "$BRANCH" ] && echo "  Branch: $BRANCH"

# Uncommitted changes summary
CHANGES=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
if [ "$CHANGES" -gt 0 ]; then
  echo "  Uncommitted changes: $CHANGES file(s)"
fi

# Build status (fast — no cargo check)
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

# Recent commits (last 3)
echo "  Recent commits:"
git log --oneline -3 2>/dev/null | while read -r line; do
  echo "    $line"
done

# Context bundle references
echo ""
echo "Context files (.claude/context/):"
if [ -f "$CONTEXT_DIR/ir-types-summary.md" ]; then
  echo "  - ir-types-summary.md — Core IR data types (enums, structs, type aliases)"
fi
if [ -f "$CONTEXT_DIR/recent-changes.md" ]; then
  echo "  - recent-changes.md — Recent git activity and diff stats"
fi
if [ -f "$CONTEXT_DIR/source-index.md" ]; then
  echo "  - source-index.md — Function/type catalog with line numbers"
fi
if [ -f "$CONTEXT_DIR/call-graph.md" ]; then
  echo "  - call-graph.md — Caller/callee cross-references"
fi
if [ -f "$CONTEXT_DIR/project-status.md" ]; then
  echo "  - project-status.md — Phase, language gaps, stdlib progress"
fi
if [ -f "$CONTEXT_DIR/architecture.md" ]; then
  echo "  - architecture.md — Mermaid diagrams (pipeline, types, module deps)"
fi
if [ -f "$CONTEXT_DIR/session-reflections.md" ]; then
  echo "  - session-reflections.md — Accumulated session learnings"
fi
echo "  Read these files before exploring the codebase."

# Check for pending reflection from previous session
MARKER="$CONTEXT_DIR/pending-reflection.md"
if [ -f "$MARKER" ]; then
  COMMIT_COUNT=$(grep '^Commits made:' "$MARKER" 2>/dev/null | awk '{print $3}')
  echo ""
  echo "Previous session made $COMMIT_COUNT commit(s). Run /reflect to capture learnings."
fi

# Record session start time for the Stop hook
mkdir -p "$CONTEXT_DIR"
date -u +"%Y-%m-%dT%H:%M:%SZ" > "$CONTEXT_DIR/.session-start-time"

exit 0
