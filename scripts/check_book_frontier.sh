#!/usr/bin/env bash
# scripts/check_book_frontier.sh — BOOK-FRONTIER doctrine.
#
# The mdBook is the director's review surface, so it must not carry a frontier
# pointer that has quietly fallen behind the task tree. `docs/book/src/roadmap.md`
# named `SIGNOFF-REPAIR.3.3.4.3.3.3.3` as "the current frontier" seven committed
# leaves after that leaf closed — nothing checked it, because the page held a
# SECOND copy of a fact the tree already owns.
#
# The rule is therefore not "the book must be re-synchronized": it is "the book
# may not hold an unchecked copy". A book page may state the current frontier
# only if it names the SAME leaf as row 1 of the owning tree's Current Frontier
# table. Saying nothing passes — routing the reader to the maintained page is the
# intended shape.
#
# Self-test: scripts/check_book_frontier.sh --self-test
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

TREE="docs/tasks/SIGNOFF-REPAIR.md"
BOOK="docs/book/src"

# "the current frontier is `X`" / "current frontier: `X`" — the claim shapes a
# page would use. The leaf id is captured from the first backticked token after.
CLAIM='[Cc]urrent frontier'

frontier_leaf() {
  # Row 1 of the Current Frontier table: | 1 | `LEAF` | status | why |
  awk '
    /^## Current Frontier/ { inside = 1; next }
    inside && /^## / { exit }
    inside && /^\| *1 *\|/ {
      if (match($0, /`[A-Z][A-Z0-9-]*\.[0-9.]+`/)) {
        leaf = substr($0, RSTART + 1, RLENGTH - 2)
        print leaf
        exit
      }
    }
  ' "$1"
}

if [ "${1:-}" = "--self-test" ]; then
  fails=0
  probe="$(mktemp_dir="target/doctrine_scratch"; mkdir -p "$mktemp_dir"; echo "$mktemp_dir/frontier-selftest.md")"
  cat > "$probe" <<'TREE'
## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SIGNOFF-REPAIR.9.9.9` | `pending` | the self-test row |
| 2 | `SIGNOFF-REPAIR.1.1` | `pending` | not row one |

## Next section
TREE
  got="$(frontier_leaf "$probe")"
  if [ "$got" != "SIGNOFF-REPAIR.9.9.9" ]; then
    echo "SELF-TEST: row-1 extraction returned '$got'" >&2; fails=$((fails+1))
  fi
  for probe_line in \
    'The current frontier is `SIGNOFF-REPAIR.3.3.4`: durable CLI bootstrap.' \
    'Current frontier: `SIGNOFF-REPAIR.7.1`.'; do
    if ! printf '%s\n' "$probe_line" | grep -qE "$CLAIM"; then
      echo "SELF-TEST: the claim matcher missed: $probe_line" >&2; fails=$((fails+1))
    fi
  done
  if printf '%s\n' 'Roadmap and progress: the frozen plan and its gates.' | grep -qE "$CLAIM"; then
    echo "SELF-TEST: the claim matcher over-matched" >&2; fails=$((fails+1))
  fi
  rm -f "$probe"
  [ "$fails" -eq 0 ] || exit 1
  echo "BOOK-FRONTIER self-test: row-1 extraction and 2 claim shapes verified, 1 non-claim ignored"
  exit 0
fi

[ -f "$TREE" ] || exit 0   # No owning tree: nothing to disagree with.
[ -d "$BOOK" ] || exit 0

expected="$(frontier_leaf "$TREE")"
errs=0
while IFS= read -r hit; do
  [ -n "$hit" ] || continue
  path="${hit%%:*}"; rest="${hit#*:}"
  line="${rest%%:*}"; text="${rest#*:}"
  named="$(printf '%s' "$text" | grep -oE '`[A-Z][A-Z0-9-]*\.[0-9.]+`' | head -1 | tr -d '`')"
  [ -n "$named" ] || continue   # A pointer with no leaf id claims nothing stale.
  if [ "$named" != "$expected" ]; then
    if [ "$errs" -eq 0 ]; then
      echo "BOOK-FRONTIER: the book names a frontier the task tree does not:" >&2
      echo "  $TREE row 1 is '${expected:-<none>}'." >&2
    fi
    echo "    $path:$line names '$named'" >&2
    echo "  Either name the same leaf, or — better — drop the second copy and" >&2
    echo "  route the reader to the page that is maintained per leaf." >&2
    errs=$((errs+1))
  fi
done < <(grep -rnE "$CLAIM" "$BOOK" 2>/dev/null)

[ "$errs" -eq 0 ] || exit 1
exit 0
