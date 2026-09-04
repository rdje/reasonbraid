#!/usr/bin/env bash
# scripts/check_docpaths.sh — DOCPATH doctrine.
#
# Tracked .md files must not embed checkout-specific absolute paths (they break for
# every other clone and leak local usernames). Repo-internal references must be
# repo-root-relative. Only STAGED .md files are checked (fast, pre-commit friendly).
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

# Absolute paths that point into a user home / checkout are the smell.
pattern='(/Users/[^ ]+|/home/[^ ]+)'
errs=0
while IFS= read -r f; do
  [ -n "$f" ] || continue
  case "$f" in *.md) ;; *) continue;; esac
  [ -f "$f" ] || continue
  if hits="$(grep -nE "$pattern" "$f" 2>/dev/null)"; then
    echo "DOCPATH: $f contains checkout-specific absolute path(s):" >&2
    printf '%s\n' "$hits" | sed 's/^/    /' >&2
    errs=$((errs+1))
  fi
done < <(git diff --cached --name-only --diff-filter=ACM)

[ "$errs" -eq 0 ] || exit 1
exit 0
