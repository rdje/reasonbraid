#!/usr/bin/env python3
"""FRONTIER-STATUS — a frontier row's status column must agree with its leaf,
and row 1 must not name a finished leaf (`SIGNOFF-REPAIR.11.22`).

A Current Frontier table carries, per row, a leaf id and a status column. The
leaf carries its own status. Those are two copies of one fact, and nothing
related them — so they drifted, three times that the tree records:

  - `.7.4.1` sat at row 2 for SEVEN commits after it closed (`.11.15`);
  - REPAIR-0253 found FIVE rows saying `pending` whose leaves said `done`
    (`.11.18`, `.11.17`, `.11.14.3.7`, `.11.14.3.5`, `.11.4.7.2`);
  - two commits later, **row 1 itself** named `.7.4.5`, `done` since
    REPAIR-0216. Row 1 is the one row a fresh session resumes from.

⭐ THE MECHANISM, which is sharper than "nobody checked". The table gains a NEW
row when a leaf closes and the OPENING row is never removed, so one leaf ends up
with two rows that contradict each other. The census that opened this check
measured **66 rows naming 49 leaves — 13 leaves with 2 to 4 rows each, and 10
rows whose status column disagreed with its leaf.**

⛔ THREE GATES LOOK ADJACENT AND EVERY ONE CORRECTLY MISSES IT. `INDEX-FRONTIER`
compares `docs/TASK_TREE.md`'s cell to the tree's row 1 — both named the closed
leaf, and two files agreeing is exactly what it asks. `TASK-STATUS` requires one
`Status:` line per leaf and never reads a table. `TABLE-ARITY-RATCHET` reads cell
counts, never cell meaning. The defect lives in the seam between three checks
each doing its own job.

THE TWO RULES, and the second is the one that hurts:

  1. every row's status column equals its leaf's own status;
  2. row 1's leaf is not finished.

⚠️ A `done` ROW IS EXPLICITLY LEGAL. The table keeps closed leaves as recent
history — most of it is closed rows — so this is not "no finished leaves in the
table". It is that a row must not LIE about its leaf, and that the top of the
list must be work.

⚠️ A LEAF'S STATUS IS NOT ALWAYS IN A `Status:` LINE. Measured: 350 leaves, 28
carry none, and 27 of those state it in their `Opened:` bullet. Reading only the
first form would report 28 false disagreements, which is how a gate gets switched
off rather than fixed.

    python3 -B scripts/check_frontier_status.py
    python3 -B scripts/check_frontier_status.py --self-test
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HEADING = "## Current Frontier"
# A status a row may carry meaning "no longer work". `active` is a real third
# value in this corpus (`.9.3.4`), and it is NOT finished.
FINISHED = {"done", "closed", "superseded"}
# A cell that deliberately defers to another row rather than stating a status.
DEFERRED = {"—", "-", ""}

LEAF = r"SIGNOFF-REPAIR[\d.]*\d|[A-Z][A-Z0-9-]*[A-Z0-9](?:\.\d+)+"
ROW = re.compile(rf"^\|\s*([^|]+?)\s*\|\s*`({LEAF})`[^|]*\|\s*`?([\w-]*)`?[^|]*\|")


def leaf_status(text: str) -> dict[str, str | None]:
    """Each leaf's OWN status, from a `Status:` line or an `Opened:` bullet.

    The first heading wins: a leaf's section runs to the next heading, and the
    first status line inside it is the leaf's own rather than a child's.
    """
    heads = list(re.finditer(rf"^#{{3,6}} ({LEAF}) — ", text, re.M))
    found: dict[str, str | None] = {}
    for i, h in enumerate(heads):
        end = heads[i + 1].start() if i + 1 < len(heads) else len(text)
        body = text[h.end(): end]
        m = re.search(r"^- Status: `([\w-]+)`", body, re.M) or re.search(
            r"^- Opened: `([\w-]+)`", body, re.M
        )
        found.setdefault(h.group(1), m.group(1) if m else None)
    return found


def frontier_rows(text: str) -> list[tuple[int, str, str, str]]:
    """(line number, order, leaf, status column) for each data row.

    The table is the CONTIGUOUS run of pipe lines after the heading: a blank
    line TERMINATES a GFM table, and this very table once carried one, which
    silently rendered 45 rows as a paragraph (`.11.4.3.1.2.x`).
    """
    lines = text.split("\n")
    try:
        start = next(i for i, l in enumerate(lines) if l.rstrip() == HEADING)
    except StopIteration:
        return []
    i = start
    while i < len(lines) and not lines[i].startswith("|"):
        if lines[i].strip() and i > start:  # prose before any table: no table here
            return []
        i += 1
    rows = []
    for n in range(i + 2, len(lines)):  # skip header + delimiter
        if not lines[n].startswith("|"):
            break
        m = ROW.match(lines[n])
        if m:
            rows.append((n + 1, m.group(1), m.group(2), m.group(3).strip()))
    return rows


def breaches(text: str) -> list[str]:
    """Every way this file's frontier table disagrees with its own leaves."""
    own = leaf_status(text)
    rows = frontier_rows(text)
    out: list[str] = []
    for line, order, leaf, col in rows:
        if col in DEFERRED:
            continue
        actual = own.get(leaf)
        if actual is None:
            continue  # a cross-tree leaf this file does not define
        if col != actual:
            out.append(f"line {line}: row {order} says `{col}`, but {leaf} says `{actual}`")
    first = [r for r in rows if r[1] == "1"]
    if first:
        line, _, leaf, _ = first[0]
        actual = own.get(leaf)
        if actual in FINISHED:
            out.append(
                f"line {line}: ROW 1 names {leaf}, which is `{actual}` — "
                "row 1 is the leaf a fresh session resumes from"
            )
    return out


def tracked_trees() -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", "docs/tasks/*.md"], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout.split()
    return [ROOT / p for p in out]


def self_test() -> int:
    failures: list[str] = []

    def check(label: str, text: str, *, fragment: str | None) -> None:
        got = breaches(text)
        if fragment is None:
            if got:
                failures.append(f"{label}: expected no breach, got {got}")
        elif not any(fragment in g for g in got):
            failures.append(f"{label}: expected a breach mentioning {fragment!r}, got {got}")

    def tree(rows: str, leaves: str) -> str:
        return (
            f"{leaves}\n\n{HEADING}\n\n| Order | Leaf | Status | Why next |\n"
            f"| --- | --- | --- | --- |\n{rows}\n"
        )

    ok = tree(
        "| 1 | `SIGNOFF-REPAIR.1.1` | `pending` | work |\n"
        "| 2 | `SIGNOFF-REPAIR.1.2` | `done` | history |",
        "#### SIGNOFF-REPAIR.1.1 — a\n\n- Status: `pending`.\n\n"
        "#### SIGNOFF-REPAIR.1.2 — b\n\n- Status: `done`.\n",
    )
    check("an agreeing table", ok, fragment=None)

    # ⭐ A `done` row is LEGAL. If this ever breaks, the rule has become
    # "no finished leaves in the table", which would condemn most of the corpus.
    if any("SIGNOFF-REPAIR.1.2" in b for b in breaches(ok)):
        failures.append("a done row kept as history was refused")

    check(
        "a stale pending row",
        tree(
            "| 1 | `SIGNOFF-REPAIR.1.1` | `pending` | work |\n"
            "| 2 | `SIGNOFF-REPAIR.1.2` | `pending` | stale |",
            "#### SIGNOFF-REPAIR.1.1 — a\n\n- Status: `pending`.\n\n"
            "#### SIGNOFF-REPAIR.1.2 — b\n\n- Status: `done`.\n",
        ),
        fragment="says `pending`, but SIGNOFF-REPAIR.1.2 says `done`",
    )
    check(
        "row 1 naming a closed leaf",
        tree(
            "| 1 | `SIGNOFF-REPAIR.1.2` | `done` | closed |",
            "#### SIGNOFF-REPAIR.1.2 — b\n\n- Status: `done`.\n",
        ),
        fragment="ROW 1 names SIGNOFF-REPAIR.1.2",
    )
    # The `Opened:` form: 27 leaves in this corpus carry no `Status:` line.
    check(
        "a leaf whose status is only in its Opened bullet",
        tree(
            "| 1 | `SIGNOFF-REPAIR.1.3` | `done` | wrong |",
            "#### SIGNOFF-REPAIR.1.3 — c\n\n- Opened: `pending` by somebody.\n",
        ),
        fragment="says `done`, but SIGNOFF-REPAIR.1.3 says `pending`",
    )
    check(
        "the Opened form agreeing",
        tree(
            "| 1 | `SIGNOFF-REPAIR.1.3` | `pending` | right |",
            "#### SIGNOFF-REPAIR.1.3 — c\n\n- Opened: `pending` by somebody.\n",
        ),
        fragment=None,
    )
    # `active` is a real third value and is NOT finished.
    check(
        "an active leaf at row 1",
        tree(
            "| 1 | `SIGNOFF-REPAIR.1.4` | `active` | in flight |",
            "#### SIGNOFF-REPAIR.1.4 — d\n\n- Status: `active`; a decision taken.\n",
        ),
        fragment=None,
    )
    check(
        "a deferred cell",
        tree(
            "| 1 | `SIGNOFF-REPAIR.1.1` | `pending` | work |\n"
            "| 2b | `SIGNOFF-REPAIR.1.2` | — | see row 1 |",
            "#### SIGNOFF-REPAIR.1.1 — a\n\n- Status: `pending`.\n\n"
            "#### SIGNOFF-REPAIR.1.2 — b\n\n- Status: `done`.\n",
        ),
        fragment=None,
    )
    # A blank line terminates a GFM table, and this corpus has been bitten by it.
    blank = ok.replace("| --- | --- | --- | --- |\n", "| --- | --- | --- | --- |\n\n")
    if frontier_rows(blank):
        failures.append("rows were read across a blank line, which GFM treats as the table's end")
    # And a tree with no frontier table at all is not a breach.
    check("a tree with no frontier table", "#### SIGNOFF-REPAIR.1.1 — a\n\n- Status: `done`.\n", fragment=None)

    for f in failures:
        print(f"SELF-TEST FAILED: {f}", file=sys.stderr)
    if failures:
        return 1
    print(
        "FRONTIER-STATUS self-test: an agreeing table passes, a done row kept as history is "
        "allowed, a stale pending row and a row-1 pointer at a closed leaf are each refused by "
        "name, both status forms read, `active` is not finished, a deferred cell is skipped, a "
        "blank line ends the table, and a tree with no table is silent"
    )
    return 0


def main() -> int:
    if "--self-test" in sys.argv[1:]:
        return self_test()
    found: list[str] = []
    for path in tracked_trees():
        text = path.read_text()
        for b in breaches(text):
            found.append(f"    {path.relative_to(ROOT)}:{b}")
    if not found:
        return 0
    print("FRONTIER-STATUS: a frontier row disagrees with the leaf it names.", file=sys.stderr)
    for f in found:
        print(f, file=sys.stderr)
    print(
        "\n  A row's status column is a SECOND copy of the leaf's own status. When a leaf\n"
        "  closes, update its row — or remove the opening row the closing row supersedes.\n"
        "  Row 1 is the leaf a fresh session resumes from, so it must be work, not history.\n"
        "  A `done` row kept as recent history is fine; a row that LIES about its leaf is not.\n"
        "  (SIGNOFF-REPAIR.11.22 — it had drifted three times, once at row 1 itself.)",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
