#!/usr/bin/env python3
"""Refuse a qualification-review limitation that outlived its repair.

`docs/book/src/qualification-review.md` is the page a reader trusts for what is
still wrong. Each limitation row names the task-tree leaves that own it in its
last cell. When those leaves close, the row must say so, or it states a defect
the code no longer has. Nothing checked that, and at REPAIR-0528 one row had
been stale since REPAIR-0184 (`SIGNOFF-REPAIR.11.37`).

    python3 -B scripts/check_qualification_currency.py            # the census
    python3 -B scripts/check_qualification_currency.py --check    # the gate
    python3 -B scripts/check_qualification_currency.py --self-test

A row is CLOSED when any of its cells begins with ✅ (the page's convention:
limitation rows open their first cell with *✅ **Repaired***, the area table its
second with *✅ **Complete***). An OPEN row breaches when:

  · every owner it names resolves to a leaf whose Status is `done` — the repair
    landed and the row still reads as open; or
  · an owner it names resolves to no leaf at all — a reference to nothing.

Owners are the backticked relative ids in the LAST cell (`.9.3.3.7` means
`SIGNOFF-REPAIR.9.3.3.7`), and an en-dash range between two siblings
(`.5.1`–`.5.3`) is expanded to every sibling in between.

⚠️ WHAT THIS DOES NOT DO. It cannot tell whether a CLOSED row's text is still
true, and an open row owned by a live leaf may be stale in its wording; both need
reading. It proves the narrower thing that went wrong: a row left open after
every owner closed. A row with no owner id is not judged (the page's prose
tables carry none), and only this one page is read.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAGE = ROOT / "docs" / "book" / "src" / "qualification-review.md"
TREE = ROOT / "docs" / "tasks" / "SIGNOFF-REPAIR.md"

HEADING = re.compile(r"^#{2,6} SIGNOFF-REPAIR((?:\.\d+)+) — ")
ANY_HEADING = re.compile(r"^#{1,6} ")
STATUS = re.compile(r"^- Status: `(\w+)`")
OWNER = re.compile(r"`(\.\d+(?:\.\d+)*)`")
RANGE = re.compile(r"`(\.\d+(?:\.\d+)*)`\s*[–-]\s*`(\.\d+(?:\.\d+)*)`")


def leaf_statuses(tree: str) -> dict[str, str]:
    """Every leaf id to its FIRST Status value (TASK-STATUS guarantees one)."""
    statuses: dict[str, str] = {}
    current: str | None = None
    for line in tree.splitlines():
        heading = HEADING.match(line)
        if heading:
            current = heading.group(1)
            continue
        if ANY_HEADING.match(line):
            current = None
            continue
        status = STATUS.match(line)
        if current and status and current not in statuses:
            statuses[current] = status.group(1)
    return statuses


def cells(row: str) -> list[str]:
    """Split a table row on pipes outside backticks."""
    body = row.strip()
    if body.startswith("|"):
        body = body[1:]
    if body.endswith("|"):
        body = body[:-1]
    out, cell, ticked = [], [], False
    for ch in body:
        if ch == "`":
            ticked = not ticked
        if ch == "|" and not ticked:
            out.append("".join(cell).strip())
            cell = []
        else:
            cell.append(ch)
    out.append("".join(cell).strip())
    return out


def owners(cell: str) -> list[str]:
    """The owner ids in a cell, with a sibling range expanded."""
    found: list[str] = []
    for start, end in RANGE.findall(cell):
        head_s, _, last_s = start.rpartition(".")
        head_e, _, last_e = end.rpartition(".")
        if head_s == head_e and last_s.isdigit() and last_e.isdigit():
            found.extend(f"{head_s}.{n}" for n in range(int(last_s), int(last_e) + 1))
    for owner in OWNER.findall(cell):
        if owner not in found:
            found.append(owner)
    return found


def judge(page: str, statuses: dict[str, str]) -> list[tuple[int, str, str]]:
    """(line, first cell, why) for every breaching row."""
    breaches = []
    for number, line in enumerate(page.splitlines(), 1):
        if not line.lstrip().startswith("|"):
            continue
        row = cells(line)
        if len(row) < 2 or all(re.fullmatch(r":?-{3,}:?", c) for c in row if c):
            continue
        if any(c.startswith("✅") for c in row):
            continue
        named = owners(row[-1])
        if not named:
            continue
        unknown = [o for o in named if o not in statuses]
        if unknown:
            breaches.append((number, row[0], f"names no such leaf: {', '.join(unknown)}"))
            continue
        if all(statuses[o] == "done" for o in named):
            breaches.append((number, row[0], f"every owner is done ({', '.join(named)}) and the row still reads open"))
    return breaches


def run(check: bool) -> int:
    statuses = leaf_statuses(TREE.read_text(encoding="utf-8"))
    breaches = judge(PAGE.read_text(encoding="utf-8"), statuses)
    if not breaches:
        print("QUALIFICATION-CURRENCY: OK — no open limitation outlived its owners")
        return 0
    stream = sys.stderr if check else sys.stdout
    print(f"QUALIFICATION-CURRENCY: {len(breaches)} limitation row(s) outlived their repair "
          f"({PAGE.relative_to(ROOT)})", file=stream)
    for number, first, why in breaches:
        print(f"  line {number}: {first[:90]} — {why}", file=stream)
    print("  Read the row against its owners' records and the code; open its first cell with "
          "✅ **Repaired** and say what it does now, or state the residual and the leaf that "
          "still holds it.", file=stream)
    return 1 if check else 0


def self_test() -> int:
    tree = "\n".join([
        "## SIGNOFF-REPAIR.5 — parent",
        "- Status: `active`",
        "#### SIGNOFF-REPAIR.5.1 — a",
        "- Status: `done` — REPAIR-1.",
        "#### SIGNOFF-REPAIR.5.2 — b",
        "",
        "- Status: `done` — REPAIR-2.",
        "#### SIGNOFF-REPAIR.5.3 — c",
        "- Status: `pending` — deferred.",
        "##### SIGNOFF-REPAIR.5.3.1 — d",
        "- Opened by something.",
        "- Status: `done`.",
        "- Status: `pending` — a second line never wins.",
    ])
    statuses = leaf_statuses(tree)
    failures: list[str] = []

    def expect(label: str, got: object, want: object) -> None:
        if got != want:
            failures.append(f"{label}: got {got!r}, want {want!r}")

    expect("statuses", statuses, {".5": "active", ".5.1": "done", ".5.2": "done",
                                  ".5.3": "pending", ".5.3.1": "done"})
    expect("cells keep a backticked pipe", cells("| a `x|y` b | c |"), ["a `x|y` b", "c"])
    expect("range expands", owners("`.5.1`–`.5.3`"), [".5.1", ".5.2", ".5.3"])
    expect("list", owners("`.5.1`, `.5.3.1`"), [".5.1", ".5.3.1"])
    page = "\n".join([
        "| Limitation | Effect | Owner |",                                    # 1 header: no owner id
        "| --- | --- | --- |",                                                # 2 delimiter
        "| stale | still says broken | `.5.1`, `.5.2` |",                      # 3 BREACH: all done
        "| ✅ **Repaired** (`.5.1`): fixed | now fine | `.5.1` |",            # 4 closed
        "| open | residual | `.5.1`, `.5.3` |",                               # 5 one owner pending
        "| ghost | names nothing | `.5.9` |",                                 # 6 BREACH: unknown
        "| Area | ✅ **Complete**. all of it | `.5.1`–`.5.2` |",              # 7 closed in cell 2
        "| Area | incomplete | `.5.1`–`.5.3` |",                              # 8 range has a pending
        "| prose | no owner at all | - |",                                    # 9 not judged
        "| deep | a done grandchild alone | `.5.3.1` |",                      # 10 BREACH
    ])
    got = [(n, why.split(" ")[0]) for n, _, why in judge(page, statuses)]
    expect("verdicts", got, [(3, "every"), (6, "names"), (10, "every")])
    for failure in failures:
        print(f"  FAIL {failure}")
    print(f"self-test: {'FAILED' if failures else 'ok'} ({len(failures)} failure(s))")
    return 1 if failures else 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    return run(check="--check" in argv)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
