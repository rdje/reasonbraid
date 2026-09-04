#!/usr/bin/env bash
# TABLE-ARITY-RATCHET — a staged markdown file may not RAISE the number of table rows whose cell
# count disagrees with their own header. Exits NONZERO on a rise. Called by check_doctrines.sh.
#
# WHY THIS EXISTS (ported as a fresh minimal implementation by BEDROCK-MAINTENANCE.2.6; the
#   upstream instrument's self-test is bound to that project's shipped contract file). GFM's table
#   rule is per TABLE and it is two-sided and SILENT: a row with MORE cells than its own header has
#   the excess DISCARDED — not rendered, not warned about — and a row with FEWER is silently PADDED.
#   A dropped cell leaves no gap, so a reader cannot tell. Measured upstream on a clean tree where
#   every enforcer was green: 26 of 197 rows of a SHIPPED contract were losing their rightmost
#   cells, and 14 of 67 rows of the resume index — the four widest being the four most active trees.
#   The class breaks no build, fails no test, and leaves the rendered page looking fine.
#
# THE RATCHET: per staged .md file, count = rows whose cell count != their table's header count.
#   HEAD's count is the ceiling. A rise blocks and names the rows; a fall is promoted silently.
#   ⛔ Scoped to staged files, never the whole tree — a blocker over pre-existing defects teaches
#   bypass. `--all` reports the backlog, advisory.
#
# CELL COUNTING (the honest bound, stated): cells are split on pipes that are NOT escaped (`\|`)
#   and NOT inside an inline code span (`` ` `` runs matched by length, per CommonMark); a leading
#   and a trailing pipe are delimiters, not cells. A pipe inside a link or HTML is counted as a
#   separator, as GFM does. A code span left UNPAIRED (an odd run) swallows the rest of the row —
#   exactly the defect this rule catches, so it is counted as such rather than special-cased.
# CONTRACT: exit code is the verdict; explains on stderr; deterministic; read-only; fast.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

PY_SRC=$(cat <<'PY'
import re, sys
def cells(line):
    s = line.strip()
    if s.startswith("|"): s = s[1:]
    if s.endswith("|") and not s.endswith("\\|"): s = s[:-1]
    out, buf, i, n = [], "", 0, len(s)
    while i < n:
        c = s[i]
        if c == "\\" and i + 1 < n and s[i+1] == "|": buf += "|"; i += 2; continue
        if c == "`":
            m = re.match(r"`+", s[i:]); run = m.group(0); j = s.find(run, i + len(run))
            while j != -1 and (j + len(run) < n and s[j + len(run)] == "`"):  # a longer run is not our closer
                j = s.find(run, j + 1)
            if j == -1: buf += s[i:]; i = n; continue          # unpaired: swallows the row
            buf += s[i:j + len(run)]; i = j + len(run); continue
        if c == "|": out.append(buf); buf = ""; i += 1; continue
        buf += c; i += 1
    out.append(buf); return out
lines = sys.stdin.read().split("\n"); i = 0; fence = False
while i < len(lines):
    l = lines[i]
    if re.match(r"^\s*(```|~~~)", l): fence = not fence; i += 1; continue
    if not fence and l.lstrip().startswith("|") and i + 1 < len(lines) and re.match(r"^\s*\|?\s*:?-{3,}", lines[i+1]):
        want = len(cells(l)); i += 2
        while i < len(lines) and lines[i].lstrip().startswith("|"):
            have = len(cells(lines[i]))
            if have != want: print(f"{i+1}\t{have}\t{want}\t{lines[i].strip()[:120]}")
            i += 1
        continue
    i += 1
PY
)
arity_defects() { # stdin = markdown; stdout = "lineno<TAB>have<TAB>want<TAB>text" per defective row
  python3 -c "$PY_SRC"
}
count_defects() { arity_defects | grep -c . || true; }

if [ "${1:-}" = "--self-test" ]; then
  fails=0
  t() { local want="$1" name="$2"; local got; got="$(printf '%b' "$3" | count_defects)"; [ "$got" = "$want" ] && echo "  arm ok  $name ($got)" || { echo "TABLE-ARITY self-test: MISS $name want=$want got=$got" >&2; fails=1; }; }
  t 0 "a well-formed 3-column table" '| a | b | c |\n|---|---|---|\n| 1 | 2 | 3 |\n| x | y | z |\n'
  t 1 "a row with one cell too many" '| a | b |\n|---|---|\n| 1 | 2 | 3 |\n'
  t 1 "a row with one cell too few" '| a | b | c |\n|---|---|---|\n| 1 | 2 |\n'
  t 0 "a pipe inside a code span is not a separator" '| a | b |\n|---|---|\n| `x | y` | 2 |\n'
  t 0 "an escaped pipe is not a separator" '| a | b |\n|---|---|\n| x \\| y | 2 |\n'
  t 1 "an unpaired backtick swallows the rest of the row" '| a | b |\n|---|---|\n| `` | 2 |\n'
  t 0 "a table inside a code fence is not judged" '```\n| a | b |\n|---|---|\n| 1 |\n```\n'
  t 0 "a double-backtick span holding a single backtick" '| a | b |\n|---|---|\n| `` x ` y `` | 2 |\n'
  [ "$fails" = 0 ] && echo "TABLE-ARITY-RATCHET --self-test: 8/8 arms" || exit 1
  exit 0
fi

if [ "${1:-}" = "--all" ]; then
  total=0
  for f in $(git ls-files '*.md'); do n="$(count_defects < "$f")"; [ "$n" -gt 0 ] && { printf '  %-60s %s\n' "$f" "$n"; total=$((total + n)); }; done
  echo "TABLE-ARITY-RATCHET --all (ADVISORY): $total arity-defective row(s) in tracked markdown"
  exit 0
fi

staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null | grep -E '\.md$' || true)"
[ -n "$staged" ] || { echo "TABLE-ARITY-RATCHET: ok (no staged markdown)"; exit 0; }
fail=0
for f in $staged; do
  now="$(git show ":$f" 2>/dev/null | count_defects)"
  before="$(git show "HEAD:$f" 2>/dev/null | count_defects)"
  if [ "${now:-0}" -gt "${before:-0}" ]; then
    fail=1
    { echo "TABLE-ARITY-RATCHET: RISE — $f has $now row(s) whose cell count disagrees with their header (HEAD: $before):"
      git show ":$f" | arity_defects | while IFS=$'\t' read -r ln have want text; do echo "    line $ln: $have cell(s), header has $want: $text"; done
      echo "  GFM silently DROPS the extra cells or PADS the missing ones. Fix the row (an escaped \\| or a code span for a literal pipe)."; } >&2
  fi
done
[ "$fail" = 0 ] && echo "TABLE-ARITY-RATCHET: ok ($(printf '%s\n' "$staged" | wc -l | tr -d ' ') staged markdown file(s), no rise)"
[ "$fail" = 0 ] || exit 1
exit 0
