#!/usr/bin/env bash
# scripts/check_heading_depth.sh — HEADING-DEPTH doctrine.
#
# A Markdown ATX heading stops at level 6. `#######` is not a level-7 heading —
# it is a PARAGRAPH that happens to start with hashes, in CommonMark and in GFM
# alike. A document that encodes hierarchy in heading depth therefore runs out
# of levels silently: it keeps typing hashes, the file keeps looking nested, and
# every heading-aware reader stops seeing the structure.
#
# Measured here, and this is why the check exists: the active task tree carried
# 29 such lines, at levels 7 through 11. Their content was attributed to the
# nearest REAL heading above them, which meant the tree's deepest leaves — the
# ones under active work — were invisible as sections to anything that parses
# headings, including this repository's own censuses. The first census written
# against that file reported one section holding 18 status lines; it held one,
# and had swallowed five children.
#
# The remedy is not more hashes. A leaf id (`TREE.3.3.4.3.3.3.3.2.3.2`) already
# carries the depth exactly, and carries it further than six levels ever could,
# so the heading level is free to cap at 6 and lose nothing.
#
# Fenced code blocks are exempt: a fence may legitimately contain hashes, and
# shell comments inside one are not headings.
#
# Self-test: scripts/check_heading_depth.sh --self-test
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

# Print "path:line:text" for every over-deep heading outside a fenced block.
scan() {
  awk '
    function fence(l) { return (l ~ /^[ ]{0,3}(```+|~~~+)/) }
    FNR == 1 { inside = 0; mark = "" }
    {
      if (fence($0)) {
        m = ($0 ~ /^[ ]{0,3}```/) ? "`" : "~"
        if (!inside) { inside = 1; mark = m }
        else if (m == mark) { inside = 0 }
        next
      }
      if (!inside && $0 ~ /^#{7,}[ \t]+[^ \t]/) printf "%s:%d:%s\n", FILENAME, FNR, $0
    }
  ' "$@"
}

if [ "${1:-}" = "--self-test" ]; then
  fails=0
  dir="target/doctrine_scratch"; mkdir -p "$dir"
  probe="$dir/heading-depth-selftest.md"
  cat > "$probe" <<'PROBE'
# Level one
###### Level six is the deepest real heading
####### Level seven is a paragraph, not a heading
########## Level ten is worse

```bash
####### a fence may hold hashes: this is a comment, not a heading
```

~~~
######## a tilde fence too
~~~

#Not a heading at all (no space)
PROBE
  got="$(scan "$probe" | wc -l | tr -d ' ')"
  if [ "$got" != "2" ]; then
    echo "SELF-TEST: expected exactly 2 over-deep headings, found $got" >&2; fails=$((fails+1))
  fi
  if ! scan "$probe" | grep -q 'Level seven'; then
    echo "SELF-TEST: the level-7 heading was not detected" >&2; fails=$((fails+1))
  fi
  if scan "$probe" | grep -q 'a fence may hold hashes'; then
    echo "SELF-TEST: a fenced code line was wrongly reported" >&2; fails=$((fails+1))
  fi
  if scan "$probe" | grep -q 'tilde fence'; then
    echo "SELF-TEST: a tilde-fenced code line was wrongly reported" >&2; fails=$((fails+1))
  fi
  if scan "$probe" | grep -q 'Level six'; then
    echo "SELF-TEST: a legal level-6 heading was wrongly reported" >&2; fails=$((fails+1))
  fi
  rm -f "$probe"
  [ "$fails" -eq 0 ] || exit 1
  echo "HEADING-DEPTH self-test: 2 over-deep headings caught, level 6 and both fence styles ignored"
  exit 0
fi

mapfile -t files < <(git ls-files '*.md' 2>/dev/null)
[ "${#files[@]}" -gt 0 ] || exit 0

hits="$(scan "${files[@]}")"
[ -n "$hits" ] || exit 0

echo "HEADING-DEPTH: a Markdown heading cannot go deeper than level 6." >&2
printf '%s\n' "$hits" | sed 's/^/    /' >&2
cat >&2 <<'GUIDE'

  Seven or more hashes is a PARAGRAPH, not a heading, so these lines are not
  sections: their content belongs to the nearest real heading above them, and
  every heading-aware reader — renderers, censuses, this repository's own
  generators — will attribute it there.

  Cap the heading at `######` and let the identifier carry the depth. A leaf id
  already states its position exactly, and states it deeper than six levels can.
GUIDE
exit 1
