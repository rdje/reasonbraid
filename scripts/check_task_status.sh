#!/usr/bin/env bash
# scripts/check_task_status.sh — TASK-STATUS doctrine.
#
# A task-tree leaf may carry exactly ONE `- Status:` line. The convention that
# grew instead was to APPEND a closing status and leave the opening one where it
# was, so a section opened `pending` and closed `done` ended up asserting both.
# A careful reader takes the later line. A reader who stops at the first one —
# and any tool that takes the first match, which is the obvious way to write it —
# gets the opposite answer.
#
# That is not hypothetical. `SIGNOFF-REPAIR.7.4.1` opened `pending`, closed
# `done` with a real commit, and then sat at row 2 of the Current Frontier for
# seven further commits, because the row was written from the first line. Five
# sections carried the contradiction when this check was written.
#
# The remedy keeps every word: the opening line becomes `- Opened:`, which is
# what it always meant, and the single `- Status:` states what is true now.
#
# Scope is `docs/tasks/` — the work-memory layer whose statuses drive the
# frontier. A section runs from any ATX heading to the next; fenced code blocks
# are exempt, so a template or example inside a fence is not a breach.
#
# Self-test: scripts/check_task_status.sh --self-test
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

# Print "path:line:heading:count" for every section holding more than one status.
scan() {
  awk '
    function fence(l) { return (l ~ /^[ ]{0,3}(```+|~~~+)/) }
    function flush() {
      if (head != "" && n > 1) printf "%s:%d:%s:%d\n", FILENAME, hline, head, n
    }
    FNR == 1 { flush(); inside = 0; mark = ""; head = ""; n = 0; hline = 0 }
    {
      if (fence($0)) {
        m = ($0 ~ /^[ ]{0,3}```/) ? "`" : "~"
        if (!inside) { inside = 1; mark = m }
        else if (m == mark) { inside = 0 }
        next
      }
      if (inside) next
      if ($0 ~ /^#+[ \t]+[^ \t]/) { flush(); head = $0; hline = FNR; n = 0; next }
      if ($0 ~ /^- Status:/) n++
    }
    END { flush() }
  ' "$@"
}

if [ "${1:-}" = "--self-test" ]; then
  fails=0
  dir="target/doctrine_scratch"; mkdir -p "$dir"
  probe="$dir/task-status-selftest.md"
  cat > "$probe" <<'PROBE'
### TREE.1 — a leaf that contradicts itself

- Opened: `pending`; how it started.
- Status: `pending`; the stale opening line.
- Status: `done`; the real closing line.

### TREE.2 — a leaf with one status

- Opened: `pending`; how it started.
- Status: `done`; exactly one.

### TREE.3 — a leaf whose example lives in a fence

- Status: `active`.

```markdown
- Status: `pending`
- Status: `done`
```
PROBE
  got="$(scan "$probe")"
  if [ "$(printf '%s\n' "$got" | grep -c 'TREE.1')" != "1" ]; then
    echo "SELF-TEST: the contradicting section was not caught once" >&2; fails=$((fails+1))
  fi
  if printf '%s\n' "$got" | grep -q 'TREE.2'; then
    echo "SELF-TEST: a single-status section was wrongly reported" >&2; fails=$((fails+1))
  fi
  if printf '%s\n' "$got" | grep -q 'TREE.3'; then
    echo "SELF-TEST: statuses inside a fenced block were wrongly counted" >&2; fails=$((fails+1))
  fi
  if [ "$(printf '%s\n' "$got" | grep -c .)" != "1" ]; then
    echo "SELF-TEST: expected exactly one finding, got: $got" >&2; fails=$((fails+1))
  fi
  rm -f "$probe"
  [ "$fails" -eq 0 ] || exit 1
  echo "TASK-STATUS self-test: 1 contradicting section caught, a single status and a fenced example ignored"
  exit 0
fi

mapfile -t files < <(git ls-files 'docs/tasks/*.md' 2>/dev/null | grep -v '/TEMPLATE\.md$')
[ "${#files[@]}" -gt 0 ] || exit 0

hits="$(scan "${files[@]}")"
[ -n "$hits" ] || exit 0

echo "TASK-STATUS: a task-tree leaf may carry only one '- Status:' line." >&2
printf '%s\n' "$hits" | while IFS= read -r hit; do
  path="${hit%%:*}"; rest="${hit#*:}"
  line="${rest%%:*}"; rest="${rest#*:}"
  count="${rest##*:}"; head="${rest%:*}"
  printf '    %s:%s  (%s status lines)  %s\n' "$path" "$line" "$count" "$head" >&2
done
cat >&2 <<'GUIDE'

  A later status silently supersedes an earlier one, so the section asserts two
  answers and the reader picks by accident. The frontier is written from these
  lines: a leaf that closed while its first line still says `pending` stays on
  the frontier, which is exactly what happened for seven commits.

  Keep the provenance, drop the ambiguity: rename the opening line to
  `- Opened:` — which is what it always meant — and leave one `- Status:`
  stating what is true now.
GUIDE
exit 1
