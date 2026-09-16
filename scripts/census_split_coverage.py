#!/usr/bin/env python3
"""Census the leaves that declare a SPLIT, and what they enumerate.

`SIGNOFF-REPAIR.11.15` owns the question this answers: two leaves have now
drawn a split from their own goal line and dropped a mechanism that goal line
names — `.3.4` dropped two (found by tranche 4b) and `.7.2` dropped one (found
by tranche 4c, three commits after the split). The obvious rule is "a split must
cover its goal line". This instrument measures whether any mechanical form of
that rule is sound BEFORE one is proposed, which is `SIGNOFF-REPAIR.11.6`'s
standing requirement and the reason two neighbouring gates were rejected.

⛔ It does NOT judge whether a split is complete. Counting the mechanisms in a
goal line is a judgement over prose — `.11.6` measured exactly that and shipped
a method statement instead of a gate. What this counts is STRUCTURAL: which
split-declaring leaves carry an explicit mechanism-to-child mapping, and which
carry only a prose count.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

TREE = pathlib.Path("docs/tasks/SIGNOFF-REPAIR.md")

# A leaf heading: three or more hashes, then the leaf id. ⛔ The tree id is NOT
# hardcoded — a probe that can only be written in the real tree's vocabulary is
# a self-test whose fixture is the thing under test (`SELF-TEST`'s own lesson).
HEADING = re.compile(r"^(#{3,6})\s+([A-Z][A-Z0-9-]*\.[0-9][0-9.]*)\b")
# The sentence shapes this tree uses when a leaf declares it has split itself.
SPLIT_DECL = re.compile(
    r"censused and split|censused and SPLIT|censused, split|SPLIT below|split below"
    r"|DECOMPOSED into|decomposed into",
)
# A prose count of the speaker's own products: "five children", "four mechanisms".
WORD_NUMBERS = {
    "one": 1, "two": 2, "three": 3, "four": 4, "five": 5,
    "six": 6, "seven": 7, "eight": 8, "nine": 9, "ten": 10,
}
COUNT_CLAIM = re.compile(
    r"\b(one|two|three|four|five|six|seven|eight|nine|ten|\d+)\s+(children|mechanisms)\b"
)
# A mapping table: a GFM header row whose first cell names a mechanism and whose
# other cells name what carries it. The shape `.3.4`'s closure produced.
# ⛔ The leading `\s*` is not decoration. The FIRST version of this pattern
# anchored at `^\|` and reported 0 of 14 — while `grep -c` over the same file
# returned 1, because `.3.4`'s table is INDENTED two spaces as a list
# continuation. The instrument's first number was checked against one obtained a
# different way and lost (`TOOLBOX.md`; `SIGNOFF-REPAIR.11.8`'s lesson).
MAPPING_HEADER = re.compile(
    r"^\s*\|\s*(mechanism|the mechanism|goal[- ]line|clause)[^|]*\|.*\|", re.IGNORECASE
)


def leaves(text: str) -> list[tuple[str, int, list[str]]]:
    """(leaf id, heading depth, its OWN lines) — a parent never swallows a child.

    The parent/child scoping bug `SIGNOFF-REPAIR.11.9`'s instrument had to fix is
    the reason this stops at the NEXT heading of any depth rather than the next
    heading of the same depth.
    """
    lines = text.splitlines()
    marks: list[tuple[int, str, int]] = []
    for i, line in enumerate(lines):
        m = HEADING.match(line)
        if m:
            marks.append((i, m.group(2), len(m.group(1))))
    out = []
    for n, (start, leaf_id, depth) in enumerate(marks):
        end = marks[n + 1][0] if n + 1 < len(marks) else len(lines)
        out.append((leaf_id, depth, lines[start:end]))
    return out


def census(text: str) -> dict:
    all_ids = {leaf_id for leaf_id, _, _ in leaves(text)}
    rows = []
    for leaf_id, _depth, body in leaves(text):
        joined = "\n".join(body)
        if not SPLIT_DECL.search(joined):
            continue
        children = sorted(
            other
            for other in all_ids
            if other.startswith(leaf_id + ".")
            and other[len(leaf_id) + 1 :].count(".") == 0
        )
        claims = [
            (WORD_NUMBERS.get(a.lower(), a if not a.isdigit() else int(a)), b)
            for a, b in COUNT_CLAIM.findall(joined)
        ]
        rows.append(
            {
                "leaf": leaf_id,
                "children": len(children),
                "has_mapping_table": any(MAPPING_HEADER.match(l) for l in body),
                "count_claims": claims,
            }
        )
    return {"splits": rows, "leaves": len(all_ids)}


def self_test() -> int:
    probe = """### DEMO.1 — a lane

- Goal and acceptance: do A; do B.
- Status: `active`; censused and split below into two children.

  | Mechanism the goal line names | Carried by | Verdict |
  | --- | --- | --- |
  | do A | `.1` | done |

#### DEMO.1.1 — first

- Status: `done`.

#### DEMO.1.2 — second

- Status: `done`.

##### DEMO.1.2.1 — a grandchild

- Status: `done`.

### DEMO.2 — a lane that never split

- Status: `pending`.
"""
    misses = 0
    got = census(probe)
    rows = {r["leaf"]: r for r in got["splits"]}
    if set(rows) != {"DEMO.1"}:
        print(f"SELF-TEST: split detection returned {sorted(rows)}", file=sys.stderr)
        misses += 1
    if rows.get("DEMO.1", {}).get("children") != 2:
        print("SELF-TEST: a grandchild must not count as a child", file=sys.stderr)
        misses += 1
    if not rows.get("DEMO.1", {}).get("has_mapping_table"):
        # ⛔ The probe's table is INDENTED on purpose: that is the shape the
        # first version of this instrument could not see, and a probe written
        # in the easy shape would have gone on missing it.
        print("SELF-TEST: an INDENTED mapping table was not detected", file=sys.stderr)
        misses += 1
    flush = census(
        "### DEMO.4 — x\n\n- Status: `active`; censused and split below.\n\n"
        "| Mechanism | Carried by |\n| --- | --- |\n\n#### DEMO.4.1 — y\n"
    )
    if not flush["splits"][0]["has_mapping_table"]:
        print("SELF-TEST: a FLUSH mapping table was not detected", file=sys.stderr)
        misses += 1
    if rows.get("DEMO.1", {}).get("count_claims") != [(2, "children")]:
        print(
            f"SELF-TEST: count claim -> {rows.get('DEMO.1', {}).get('count_claims')}",
            file=sys.stderr,
        )
        misses += 1
    # A leaf whose section has no table must not inherit the previous leaf's.
    no_table = census(
        "### DEMO.3 — x\n\n- Status: `active`; censused and split below.\n\n#### DEMO.3.1 — y\n"
    )
    if no_table["splits"][0]["has_mapping_table"]:
        print("SELF-TEST: a table was inherited across sections", file=sys.stderr)
        misses += 1
    if misses:
        print(f"census_split_coverage: {misses} control(s) missed", file=sys.stderr)
        return 1
    print("census_split_coverage --self-test: 6 controls pass")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    if not TREE.exists():
        print(f"census_split_coverage: {TREE} is missing", file=sys.stderr)
        return 1
    got = census(TREE.read_text())
    rows = got["splits"]
    mapped = [r for r in rows if r["has_mapping_table"]]
    claimed = [r for r in rows if r["count_claims"]]
    print(f"leaves in the tree                    : {got['leaves']}")
    print(f"leaves that DECLARE a split           : {len(rows)}")
    print(f"  … carrying a mechanism->child table : {len(mapped)}")
    print(f"  … carrying only a prose count       : {len(claimed) - len(mapped)}")
    print(f"  … carrying neither                  : "
          f"{len(rows) - len({r['leaf'] for r in mapped} | {r['leaf'] for r in claimed})}")
    print()
    for r in sorted(rows, key=lambda r: r["leaf"]):
        table = "table" if r["has_mapping_table"] else "  -  "
        claims = ", ".join(f"{n} {w}" for n, w in r["count_claims"]) or "-"
        print(f"  {r['leaf']:<34} children={r['children']:<3} {table}  claims: {claims}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
