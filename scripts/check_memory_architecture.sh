#!/usr/bin/env bash
# scripts/check_memory_architecture.sh — MEMORY-ARCH invariants (MEMORY_ARCHITECTURE.md).
#
# Structural, harness-agnostic checks that the durable-memory layers exist and hold
# their shape. Cheap, deterministic, no network.
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"
errs=0
fail(){ echo "MEMORY-ARCH: $1" >&2; errs=$((errs+1)); }

# Layer A — the bounded resume pointer must exist and stay bounded by BOTH caps.
#
# ⛔ A LINE CAP ALONE DOES NOT BOUND THIS FILE, and the failure is measured, not hypothetical.
# On a real project running this spine, MEMORY.md sat at 60 lines — PASSING, exactly at its
# line cap — while carrying 138,403 BYTES: 2,306 bytes per line, with one line of 18,816
# bytes. A file this standard calls a "bounded resume pointer" was a 138 KB document and this
# check was green the whole time. The block headed "Current state (OVERWRITE this block each
# update — do not append)" had accumulated 18 sessions and 81.3% of the file.
#   ⇒ Line and byte caps are COMPLEMENTS: neither wrapped prose nor very long lines can bypass
#     the budget. See README_POLICY.md, which states the same rule for README.md.
#
# The line cap is deliberately TIGHTENED from an earlier 120: MEMORY_ARCHITECTURE.md §6 and
# the pointer template both say "≤ ~50 lines", so 120 was more than twice as loose as the rule
# it policed. 7168 bytes ≈ 143 B/line at 50 lines — the shape of a real pointer file — so the
# two caps bind on the same kind of document rather than one shadowing the other.
# ⛔ Never raise a cap to fit the content: demote it to docs/tasks/ (B) or docs/decisions/ (C).
CAP="${MEMORY_POINTER_LINE_CAP:-50}"
BYTE_CAP="${MEMORY_POINTER_BYTE_CAP:-7168}"
[ -f MEMORY.md ] || fail "MEMORY.md (layer-A resume pointer) is missing"
if [ -f MEMORY.md ]; then
  lines=$(wc -l < MEMORY.md | tr -d ' ')
  bytes=$(wc -c < MEMORY.md | tr -d ' ')
  [ "$lines" -le "$CAP" ] || fail "MEMORY.md has $lines lines (> cap $CAP) — it is a bounded pointer, not a log; demote content to docs/tasks/ (B) or docs/decisions/ (C)"
  [ "$bytes" -le "$BYTE_CAP" ] || fail "MEMORY.md is $bytes bytes (> cap $BYTE_CAP) — long lines bypass the line cap; demote content, do NOT raise the cap"
fi

# Layer C — decision records live one-per-file under docs/decisions/ with an INDEX.
[ -d docs/decisions ] || fail "docs/decisions/ (layer-C decision records) is missing"
[ -f docs/decisions/INDEX.md ] || fail "docs/decisions/INDEX.md is missing"
# Every decision record (excluding INDEX/TEMPLATE) must be listed in the index.
if [ -d docs/decisions ]; then
  for f in docs/decisions/*.md; do
    b="$(basename "$f")"
    case "$b" in INDEX.md|TEMPLATE.md) continue;; esac
    grep -q "$b" docs/decisions/INDEX.md || fail "decision record $b is not listed in docs/decisions/INDEX.md"
  done
fi

# Layer B — the task-tree system exists.
[ -f docs/TASK_TREE.md ] || fail "docs/TASK_TREE.md (task-tree index) is missing"
[ -d docs/tasks ] || fail "docs/tasks/ (task-tree leaves) is missing"

# Tool-neutral bootstrap entrypoints exist.
[ -f README.md ] || fail "README.md (tool-neutral entrypoint) is missing"
[ -f CLAUDE.md ] || fail "CLAUDE.md (agent bootstrap) is missing"

[ "$errs" -eq 0 ] || exit 1
exit 0
