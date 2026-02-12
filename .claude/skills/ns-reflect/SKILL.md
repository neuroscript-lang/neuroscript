---
name: reflect
description: Capture session learnings and update PROGRESS.md with what was accomplished, lessons learned, and NeuroScript limitations discovered.
user_invocable: true
---

# Session Reflection

Generate a structured reflection on recent work in this session. This helps future sessions benefit from past learnings.

## Steps

1. **Read recent commits** — Run `git log --oneline -10` and `git diff --stat HEAD~5..HEAD` to understand what was changed.

2. **Read pending reflection marker** — Check if `.claude/context/pending-reflection.md` exists and read it for session context.

3. **Read current PROGRESS.md** — Read `docs/PROGRESS.md` to understand the format and existing entries.

4. **Generate reflection** — Create a new session entry for PROGRESS.md with:
   - `## Session: YYYY-MM-DD — <brief title>`
   - `### What Was Done` — Bullet points of accomplishments
   - `### Files Changed` — Table of layers/files/changes (follow existing format)
   - `### Lessons Learned` — What broke, what was tricky, what to watch out for
   - `### NeuroScript Limitations` — Any language limitations discovered (if any)
   - `### Next Steps` — What should be done next

5. **Append to PROGRESS.md** — Add the new entry at the top of `docs/PROGRESS.md` (after the title).

6. **Append to session-reflections.md** — Add a condensed version to `.claude/context/session-reflections.md` (create if it doesn't exist). This file persists across sessions as accumulated wisdom. Format:
   ```
   ## YYYY-MM-DD: <title>
   - Key lesson 1
   - Key lesson 2
   ```

7. **Clean up** — Remove `.claude/context/pending-reflection.md` if it exists.

## Output Format

After completing the reflection, summarize what was captured and note any important learnings for future sessions.
