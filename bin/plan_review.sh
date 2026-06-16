#!/usr/bin/env bash
# Auto-review hook: when a plan file under plans/ is written, spawn a second
# headless Claude to critique and adjust the plan in place. Fires at most once
# per plan (idempotent via marker + loop-guarded via PLAN_REVIEW env var).
set -euo pipefail

LOG=/tmp/plan_review.log

# 1. Loop guard: the reviewer runs with PLAN_REVIEW=1, so its own Edits to the
#    plan can't re-trigger this hook. Env inherits down the process tree.
[ -n "${PLAN_REVIEW:-}" ] && exit 0

# 2. Extract the written file path from the PostToolUse JSON on stdin.
input=$(cat)
file=$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty')
[ -z "$file" ] && exit 0

# 3. Only act on plans/*.md
case "$file" in
  */plans/*.md|plans/*.md) ;;
  *) exit 0 ;;
esac
[ -f "$file" ] || exit 0

# 4. Idempotency: skip anything already reviewed.
grep -q '<!-- auto-reviewed -->' "$file" && exit 0

# 5. Fire the reviewer asynchronously so the interactive session isn't blocked.
PLAN_REVIEW=1 nohup claude -p "Review the implementation plan at $file as a critical senior engineer. Judge it against the project's CLAUDE.md philosophy: simplicity over cleverness, no quick fixes, idiomatic Rust, strict separation of world logic from rendering, per-module constants.rs. Adjust the plan IN PLACE with Edit to fix flaws, tighten scope, correct bad assumptions, and surface missed edge cases. Keep the author's intent. When finished, append this exact line as the last line of the file: <!-- auto-reviewed -->" \
  --allowedTools "Read,Edit" \
  >>"$LOG" 2>&1 &

exit 0
