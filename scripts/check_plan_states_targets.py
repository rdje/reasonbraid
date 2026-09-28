#!/usr/bin/env python3
"""PLAN-STATES-TARGETS — the frozen plan names no task-tree leaf
(`SIGNOFF-REPAIR.11.43`).

`ROADMAP.md` and `KICKOFF.md` state TARGETS: v0.4.1 is the frozen execution
baseline, and it admits factual errata only. Current progress lives in derived
places: `LIVE_STATUS.md`, the book's qualification review, and
`scripts/census_open_leaves.py`. A plan that names a task-tree leaf is restating
progress, and a restated status goes false as soon as the leaf moves.

That is measured, not supposed. On 2026-09-28 `ROADMAP.md` opened with a
hand-kept narrative of thirteen leaf references, which still said *"bounded HTTP
waits and restart qualification follow"* two days after both had landed and
their tree had closed. The book's own roadmap page had already refused to
restate the frontier for that reason, so the two documents disagreed on the rule
itself. The director's rule (2026-09-28): the book is users' only window, and
the roadmap, the code and the book move together.

    python3 -B scripts/check_plan_states_targets.py              # the gate
    python3 -B scripts/check_plan_states_targets.py --self-test

A leaf reference is a tree prefix with a numbered path (`SIGNOFF-REPAIR.3.3`,
`PARTICIPATION.4`, `PHASE-8.5.3`) or a relative leaf path (`.11.4.3.1`) after
whitespace, a parenthesis or a backtick. Section numbers (`§20.3`) and versions
(`0.4.1`, `v0.4.1`) do not match, because nothing but a leaf path begins with a
bare dot. The files are read from the INDEX, so the gate judges what the commit
carries rather than the working tree.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PLANS = ("ROADMAP.md", "KICKOFF.md")
LEAF = re.compile(
    r"(SIGNOFF-REPAIR|PARTICIPATION|PHASE-[0-9]+)\.[0-9]|(^|[\s(`])\.[0-9]+(\.[0-9]+)+"
)


def staged(path: str) -> str:
    """The index's copy, falling back to the working tree for an untracked file."""
    shown = subprocess.run(
        ["git", "show", f":{path}"], cwd=ROOT, capture_output=True, text=True
    )
    if shown.returncode == 0:
        return shown.stdout
    return (ROOT / path).read_text(encoding="utf-8")


def breaches(name: str, text: str) -> list[str]:
    return [
        f"{name}:{number}: {line.strip()[:120]}"
        for number, line in enumerate(text.split("\n"), 1)
        if LEAF.search(line)
    ]


def self_test() -> int:
    refused = [
        "Tenant guard primitives are qualified under `SIGNOFF-REPAIR.3.3.4.2`;",
        "grant error classification is qualified under `.3.3.4.3.1` with 56 controls",
        "The scheduled pre-push checkpoint .11.4.3.1 now has a census",
        "resumes after `PHASE-8.5.3`",
        "owned by PARTICIPATION.4",
        "(.7.2.6) closed",
    ]
    admitted = [
        "Roadmap v0.4.1 is the execution baseline.",
        "see §20.3 and §12.9",
        "Document version: 0.4.1",
        "docs/tasks/SIGNOFF-REPAIR.md owns the corrective programme",
        "the G6/G7 Internet gates",
        "a 1.5 s wait",
    ]
    for line in refused:
        assert breaches("t", line), f"not refused: {line!r}"
    for line in admitted:
        assert not breaches("t", line), f"refused: {line!r}"
    print(f"check_plan_states_targets: self-test OK ({len(refused)} refused, {len(admitted)} admitted)")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    found = [b for name in PLANS for b in breaches(name, staged(name))]
    if found:
        for breach in found:
            print(f"PLAN-STATES-TARGETS: {breach}", file=sys.stderr)
        print(
            f"PLAN-STATES-TARGETS: {len(found)} task-leaf reference(s) in the frozen plan. It states "
            "targets; progress lives in LIVE_STATUS.md, the book's qualification review and "
            "scripts/census_open_leaves.py.",
            file=sys.stderr,
        )
        return 1
    print(f"PLAN-STATES-TARGETS: OK — {', '.join(PLANS)} name no task-tree leaf")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
