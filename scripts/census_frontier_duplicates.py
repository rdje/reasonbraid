#!/usr/bin/env python3
"""FRONTIER-DUPLICATE — one UNFINISHED leaf must not hold two frontier rows
(`SIGNOFF-REPAIR.11.22.1`).

⛔ A DUPLICATE ROW IS NOT A DEFECT IN GENERAL, and getting that wrong would
condemn most of the corpus. `.11.22`'s own census recorded *66 rows naming 49
distinct leaves — 13 leaves carry 2 to 4 rows each*, because a leaf legitimately
gets an OPENING row and, later, a closing `done` row beside it. The defect is
narrower:

    two rows for one leaf that has NOT finished.

That is the shape that invites a reader to work a leaf twice, or to read the
second row as a different leaf.

⛔ WHY `FRONTIER-STATUS` CANNOT SEE IT, stated so nobody re-checks that gate.
It relates each row's status COLUMN to its leaf's own `Status:` line. When one
leaf holds two `pending` rows and the leaf says `pending`, every row AGREES —
the check does exactly its job, and the defect is in a property it never asks
about. The same is true of `INDEX-FRONTIER` (two files agreeing is what it
asks), `TASK-STATUS` (never reads a table) and `TABLE-ARITY-RATCHET` (cell
counts, never cell meaning).

⭐ THE MECHANISM IS NOW OBSERVED RATHER THAN INFERRED, three times, and the
third and fourth were produced during this leaf's own session: a leaf closes,
row 1 is vacated, the next leaf is PROMOTED to row 1 — and nobody checks whether
it already had a row further down. `make gate` stays green throughout, because
both rows agree with the leaf.

⚠️ A DEFERRING ROW IS LEGAL AND IS NOT COUNTED. `| 8 | X (see row 1a20) | — |`
carries `—` in its status column: it points at another row rather than
asserting a status, which is the opposite of the confusion this rule exists to
prevent.

⛔ THIS IS THE CALIBRATION, NOT THE GATE. The rule ships as rule 3 of
`FRONTIER-STATUS` (`scripts/check_frontier_status.py`), which is where
`SIGNOFF-REPAIR.11.22.1` decided it belongs — the two rules read one table and
a second gate would be a second parser. This instrument imports that rule and
answers the question the leaf's acceptance asks: **over the whole history, how
many commits would it have blocked, and were they real?**

THE ANSWER, 2026-09-20 at `20def51`: **38 of 672** commits touching a tracked
tree, across **12 distinct leaves**; `.11.14.3.10` carried the shape for **24
consecutive commits**. Seven were classified by hand and all seven are real —
a leaf promoted toward row 1 while an older row survived, with the two
descriptions often verbatim. No false positive was found.

    python3 -B scripts/census_frontier_duplicates.py --calibrate
    python3 -B scripts/census_frontier_duplicates.py --self-test
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

# ⛔ The row and status vocabularies are IMPORTED from the gate they extend,
# never restated. Two parsers reading one table is how the table and its
# checker drift, which is the class of defect this whole lane is about.
from check_frontier_status import HEADING, duplicate_rows, frontier_rows  # noqa: E402


def duplicates(text: str) -> list[tuple[str, list[tuple[int, str]]]]:
    """The gate's own rule, applied to one tree's text.

    ⛔ `duplicate_rows` is IMPORTED from `check_frontier_status.py`, never
    restated here. A calibration that measures a copy of the rule measures the
    copy, and the whole subject of this lane is a table and its checker
    drifting apart.
    """
    return duplicate_rows(frontier_rows(text))


def tracked_markdown() -> list[str]:
    """Every tracked `.md` under `docs/tasks/`, at HEAD.

    ⚠️ `docs/tasks/*.md` is a git PATHSPEC, and git's `*` crosses `/` — so this
    returns **70** files, of which **54** are evidence documents under
    `docs/tasks/artifacts/`. They define no leaves and carry no frontier table,
    so they contribute nothing; but a count of 70 reported as *trees* would be
    the recursive-glob overstatement `.11.24.1.6.1` recorded, in a second
    instrument. `tracked_trees` below is the honest population.
    """
    return subprocess.run(
        ["git", "ls-files", "docs/tasks/*.md"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split()


def tracked_trees() -> list[Path]:
    """The files that actually carry a `## Current Frontier` table."""
    out = []
    for rel in tracked_markdown():
        path = ROOT / rel
        if path.is_file() and HEADING in path.read_text(encoding="utf-8", errors="replace"):
            out.append(path)
    return out


def calibrate() -> int:
    """Every commit that touched a tracked tree, evaluated with THIS rule.

    ⛔ The WHOLE history, not a window (`SIGNOFF-REPAIR.11.22.1`'s acceptance).
    A threshold calibrated over the recent past is calibrated over the period
    the author remembers.
    """
    paths = tracked_markdown()
    commits = subprocess.run(
        ["git", "log", "--format=%H", "--", *paths],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout.split()
    print(
        f"calibrating over {len(commits)} commits touching {len(paths)} tracked "
        f"docs/tasks markdown files ({len(tracked_trees())} of them carry a frontier "
        f"table at HEAD)"
    )

    fired = 0
    instances: dict[str, list[str]] = {}
    for sha in commits:
        listing = subprocess.run(
            ["git", "ls-tree", "-r", "--name-only", sha, "docs/tasks/"],
            cwd=ROOT, capture_output=True, text=True,
        ).stdout.split()
        hit = False
        for path in listing:
            # ⛔ Top-level only, and it is a SPEED decision with a correctness
            # argument behind it: `docs/tasks/artifacts/**` defines no leaves
            # and carries no frontier table, so reading 54 extra blobs per
            # commit changes no finding and costs the whole walk. Verified:
            # the unscoped run over all 70 returns the same 38 commits.
            if not path.endswith(".md") or path.count("/") != 2:
                continue
            blob = subprocess.run(
                ["git", "show", f"{sha}:{path}"],
                cwd=ROOT, capture_output=True, text=True,
            )
            if blob.returncode != 0:
                continue
            for leaf, rows in duplicates(blob.stdout):
                hit = True
                instances.setdefault(leaf, []).append(sha[:7])
        if hit:
            fired += 1

    print(f"commits the rule would have BLOCKED: {fired} of {len(commits)}")
    print(f"distinct leaves implicated            : {len(instances)}")
    for leaf, shas in sorted(instances.items(), key=lambda kv: -len(kv[1])):
        print(f"  {leaf}: {len(shas)} commits, first {shas[-1]}, last {shas[0]}")
    return 0


def self_test() -> int:
    failures: list[str] = []

    def tree(rows: str) -> str:
        return (
            f"{HEADING}\n\n| Order | Leaf | Status | Why next |\n"
            f"| --- | --- | --- | --- |\n{rows}\n"
        )

    def case(label: str, rows: str, want_leaf: str | None) -> None:
        got = duplicates(tree(rows))
        if want_leaf is None:
            if got:
                failures.append(f"{label}: expected nothing, got {got}")
        elif not any(leaf == want_leaf for leaf, _ in got):
            failures.append(f"{label}: expected {want_leaf}, got {got}")

    case(
        "two pending rows for one leaf",
        "| 1 | `SIGNOFF-REPAIR.1.1` | `pending` | a |\n"
        "| 5 | `SIGNOFF-REPAIR.1.1` | `pending` | b |",
        "SIGNOFF-REPAIR.1.1",
    )
    case(
        "an opening row beside a closing one is LEGAL",
        "| 1 | `SIGNOFF-REPAIR.1.1` | `pending` | a |\n"
        "| 5 | `SIGNOFF-REPAIR.1.1` | `done` | closed |",
        None,
    )
    case(
        "two DONE rows are legal too — the table keeps history",
        "| 1 | `SIGNOFF-REPAIR.1.1` | `done` | a |\n"
        "| 5 | `SIGNOFF-REPAIR.1.1` | `done` | b |",
        None,
    )
    case(
        "a DEFERRING row does not count",
        "| 1 | `SIGNOFF-REPAIR.1.1` | `pending` | a |\n"
        "| 8 | `SIGNOFF-REPAIR.1.1` | — | see row 1 |",
        None,
    )
    case(
        "`active` beside `pending` is the SAME leaf still unfinished twice",
        "| 1 | `SIGNOFF-REPAIR.1.1` | `active` | a |\n"
        "| 5 | `SIGNOFF-REPAIR.1.1` | `pending` | b |",
        "SIGNOFF-REPAIR.1.1",
    )
    case(
        "different leaves, one row each",
        "| 1 | `SIGNOFF-REPAIR.1.1` | `pending` | a |\n"
        "| 2 | `SIGNOFF-REPAIR.1.2` | `pending` | b |",
        None,
    )
    case("no table at all", "", None)

    # ⛔ THE REAL-FILE PROBE. A parser that returns nothing for every input
    # passes every case above; this asserts the imported parser still finds
    # the corpus's rows at all, so a silent parse failure cannot read as
    # "no duplicates".
    tree_text = (ROOT / "docs/tasks/SIGNOFF-REPAIR.md").read_text(encoding="utf-8")
    rows = frontier_rows(tree_text)
    if len(rows) < 20:
        failures.append(
            f"the imported parser found {len(rows)} rows in the real frontier table; "
            "a parse that collapses reads as 'no duplicates'"
        )

    if failures:
        for f in failures:
            print(f"SELF-TEST: {f}", file=sys.stderr)
        print(f"FRONTIER-DUPLICATE: --self-test FAILED ({len(failures)})", file=sys.stderr)
        return 1
    print(f"FRONTIER-DUPLICATE: --self-test ok (7 audit cases, 1 real-file parse of {len(rows)} rows)")
    return 0


if __name__ == "__main__":
    arg = sys.argv[1] if len(sys.argv) > 1 else "--calibrate"
    if arg == "--self-test":
        sys.exit(self_test())
    sys.exit(calibrate())
