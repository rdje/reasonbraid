#!/usr/bin/env bash
# scripts/check_task_tree_ownership.sh — TASK-TREE-OWNERSHIP doctrine.
#
# Binding rule: no code change lands unless a task-tree leaf owns it. This check is
# a heuristic backstop, not a proof: when a commit stages source/build files, it
# requires that the SAME commit also touches a task-tree file under docs/tasks/
# (evidence the change was routed through a leaf). The real ownership is enforced by
# COMMIT.md discipline + review; this catches the obvious "code with no tree" slip.
#
# Tune CODE_GLOBS for your project. Set SPINE_ALLOW_UNOWNED=1 to bypass for a
# deliberate tree-less doc/scaffold commit (use sparingly, and say why in the message).
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

[ "${SPINE_ALLOW_UNOWNED:-0}" = "1" ] && exit 0

staged="$(git diff --cached --name-only --diff-filter=ACM)"
[ -n "$staged" ] || exit 0

# What counts as a "code change" for this project. Extend as needed.
code_changed=0
tree_touched=0
while IFS= read -r f; do
  [ -n "$f" ] || continue
  case "$f" in
    docs/tasks/*) tree_touched=1 ;;
    *.rs|crates/*|src/*|build.rs|Cargo.toml|Cargo.lock|*/Cargo.toml) code_changed=1 ;;
  esac
done <<< "$staged"

if [ "$code_changed" = "1" ] && [ "$tree_touched" = "0" ]; then
  echo "TASK-TREE-OWNERSHIP: code files are staged but no docs/tasks/ leaf was updated in this commit." >&2
  echo "  Create/extend a task-tree leaf that owns this change, or set SPINE_ALLOW_UNOWNED=1 for a deliberate exception." >&2
  exit 1
fi
exit 0
