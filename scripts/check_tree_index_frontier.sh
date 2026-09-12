#!/usr/bin/env bash
# scripts/check_tree_index_frontier.sh — INDEX-FRONTIER doctrine.
#
# docs/TASK_TREE.md's Active Task Trees table carries a Frontier column, which is
# a SECOND copy of a fact each tree already owns. Nothing derived it and nothing
# checked it, so it drifted: the SIGNOFF-REPAIR row named .7.3.3.2.2, a leaf
# closed by REPAIR-0060 on 2026-09-11, for roughly thirty-five commits.
#
# BOOK-FRONTIER fixed this exact shape for the BOOK and was simply not extended
# to the project's own index. This is that extension.
#
# ⛔ The rule is DELIBERATELY narrower than "the cell must equal row 1", because
# the census says that rule is wrong. Measured over all fourteen rows:
#
#   - 8 trees are complete and write `| — |` instead of a numbered row 1. Their
#     index cell names the NEXT TREE's first leaf ("next executable work is
#     PHASE-1.1"), which is deliberately not a leaf of that tree at all.
#   - PHASE-8's row 1 is `SIGNOFF-REPAIR.3.3`, a cross-tree PREREQUISITE, and its
#     index cell says "corrective prerequisite SIGNOFF-REPAIR; then .5.3" — an
#     accurate summary of rows 1 AND 2. Equality would flag it, falsely.
#   - 2 rows name no leaf at all, which claims nothing.
#   - Exactly 1 row — the active tree whose row 1 is its OWN leaf — is the shape
#     where equality is the right invariant. It is also the only row that moves.
#
# So: an ACTIVE tree whose Current Frontier row 1 names a leaf OF THAT TREE must
# be named by its index cell. Everything else is not a claim this can check, and
# pretending otherwise would teach bypass.
#
# ⛔ A GENERATOR was the leaf's provisional preference and the census rejected it:
# regenerating the column would destroy accurate curated prose in 13 of 14 rows.
#
# Modes:
#   scripts/check_tree_index_frontier.sh              the tree as it stands
#   scripts/check_tree_index_frontier.sh --self-test  the predicate, both ways
#   scripts/check_tree_index_frontier.sh --against SHA  re-run over a past commit
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

HEADLINE="INDEX-FRONTIER"
INDEX="docs/TASK_TREE.md"

# Row 1 of a tree's Current Frontier table, when it is a NUMBERED row. A
# completed tree writes `| — |` and is correctly skipped here.
frontier_row1() {
  awk '
    /^## Current Frontier/ { inside = 1; next }
    inside && /^## / { exit }
    inside && /^\| *1 *\|/ {
      if (match($0, /`[A-Z][A-Z0-9-]*\.[0-9][0-9.]*`/)) {
        print substr($0, RSTART + 1, RLENGTH - 2); exit
      }
    }
  '
}

# The leaf an index cell names, normalised: a cell may write the full
# `TREE.1.2.3` or the dot-prefixed shorthand `.1.2.3`.
cell_leaf() {
  local tree="$1" cell="$2" token
  token="$(printf '%s' "$cell" | grep -oE '`(([A-Z][A-Z0-9-]*)?\.[0-9][0-9.]*)`' | head -1 | tr -d '`')"
  [ -n "$token" ] || return 0
  case "$token" in
    .*) printf '%s%s\n' "$tree" "$token" ;;
    *)  printf '%s\n' "$token" ;;
  esac
}

if [ "${1:-}" = "--self-test" ]; then
  fails=0
  row1_probe='## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `DEMO.4.5` | `pending` | the row under test |
| 2 | `DEMO.9` | `pending` | not row one |

## Next'
  got="$(printf '%s\n' "$row1_probe" | frontier_row1)"
  [ "$got" = "DEMO.4.5" ] || { echo "SELF-TEST: row-1 extraction returned '$got'" >&2; fails=$((fails+1)); }

  complete_probe='## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | — | — | **Tree complete** — next executable work is `OTHER.1` |

## Next'
  got="$(printf '%s\n' "$complete_probe" | frontier_row1)"
  [ -z "$got" ] || { echo "SELF-TEST: a completed tree produced a row 1: '$got'" >&2; fails=$((fails+1)); }

  [ "$(cell_leaf DEMO '`.4.5` — the shorthand')" = "DEMO.4.5" ] || {
    echo "SELF-TEST: the dot-shorthand did not normalise" >&2; fails=$((fails+1)); }
  [ "$(cell_leaf DEMO '`DEMO.4.5` — the full form')" = "DEMO.4.5" ] || {
    echo "SELF-TEST: the full form did not survive" >&2; fails=$((fails+1)); }
  [ "$(cell_leaf DEMO 'index only — no leaf named here')" = "" ] || {
    echo "SELF-TEST: a prose-only cell produced a leaf" >&2; fails=$((fails+1)); }
  [ "$(cell_leaf PHASE-8 'corrective prerequisite `SIGNOFF-REPAIR`; then `.5.3` — the store-and-forward')" = "PHASE-8.5.3" ] || {
    echo "SELF-TEST: the prerequisite cell did not resolve to its own shorthand" >&2; fails=$((fails+1)); }

  [ "$fails" -eq 0 ] || exit 1
  echo "$HEADLINE self-test: numbered row 1 extracted, a completed tree's dash row correctly yields none, 4 cell shapes normalised"
  exit 0
fi

if [ "${1:-}" = "--against" ]; then
  sha="${2:?--against needs a commit}"
  index_text() { git show "$sha:$INDEX" 2>/dev/null; }
  tree_text()  { git show "$sha:docs/tasks/$1" 2>/dev/null; }
else
  index_text() { cat "$INDEX" 2>/dev/null; }
  tree_text()  { cat "docs/tasks/$1" 2>/dev/null; }
fi

index="$(index_text)"
[ -n "$index" ] || exit 0

errs=0; checked=0
while IFS='|' read -r _ treecell statuscell frontiercell _; do
  tree="$(printf '%s' "$treecell" | grep -oE '`[A-Z][A-Z0-9-]*`' | head -1 | tr -d '`')"
  [ -n "$tree" ] || continue
  status="$(printf '%s' "$statuscell" | tr -d ' `')"
  [ "$status" = "active" ] || continue
  path="$(printf '%s' "$treecell" | grep -oE 'tasks/[A-Za-z0-9_.-]+\.md' | head -1)"
  [ -n "$path" ] || continue
  row1="$(tree_text "${path#tasks/}" | frontier_row1)"
  [ -n "$row1" ] || continue                 # no numbered row 1: nothing to agree with
  case "$row1" in "$tree".*) ;; *) continue ;; esac   # a cross-tree prerequisite
  checked=$((checked + 1))
  named="$(cell_leaf "$tree" "$frontiercell")"
  [ -n "$named" ] || continue                # the cell claims no leaf
  if [ "$named" != "$row1" ]; then
    if [ "$errs" -eq 0 ]; then
      echo "$HEADLINE: the tree index names a frontier its own tree does not:" >&2
    fi
    echo "    $INDEX row for \`$tree\` names '$named'; docs/$path row 1 is '$row1'" >&2
    errs=$((errs + 1))
  fi
done <<EOF
$(printf '%s\n' "$index" | grep -E '^\| \[`[A-Z]')
EOF

if [ "$errs" -ne 0 ]; then
  cat >&2 <<'EXPLAIN'

  The index's Frontier column is a SECOND copy of a fact the tree owns, and this
  is what happens to second copies: the SIGNOFF-REPAIR row named a leaf that had
  closed roughly thirty-five commits earlier.

  Update the index row to name the tree's own row 1. If the tree's frontier is
  genuinely elsewhere — a cross-tree prerequisite, say — put THAT leaf in row 1
  of the tree's own table, which is where a reader looks first.
EXPLAIN
  exit 1
fi
exit 0
