#!/usr/bin/env python3
"""POINTER-CURRENCY — `MEMORY.md`'s two mechanical fields must name the present.

`MEMORY_ARCHITECTURE.md` §6 prefers a DERIVED resume pointer to a hand-written
one. `SIGNOFF-REPAIR.11.4.2.4` measured the preference over all **645** commits
that have touched the file and **declined the generator** — the risk lands on
`next_action`, a judgement line a regenerating script destroys, and on 222
curated-prose bullets. What it left owed is this: a check, which writes nothing.

⛔ **CALIBRATED, NOT ASSERTED** (`DOCTRINE_ENFORCEMENT.md`; `SIGNOFF-REPAIR.11.9`'s
gate was rejected at 114-of-131). `scripts/census_memory_pointer_drift.py`
reproduces the calibration over the whole history: **17** instances, each
classified by hand — 7 in `latest_commit` (of which **one named a hash that is
in no object in this repository**: `ba6e77e` at `69b6374`, whose quoted subject
sits at `c8020be`, that commit's own parent) and 10 in the frontier leaf.

⛔ **IT MUST NAME WHICH COPY MOVED, AND THAT IS NOT A COURTESY.** The census
measured the disagreement as SYMMETRIC: in **5 of the 10** frontier instances the
TREE's row 1 was the copy that had not advanced — a parent leaf had just been
decomposed and the pointer correctly named the first child. A gate that says
"the pointer is stale" would be wrong half the time, and a gate that is wrong
half the time teaches bypass.

⚠️ **THE DOT-SHORTHAND IS AMBIGUOUS UNDER A TREE WHOSE NAME ENDS IN A NUMBER**
and both readings are accepted. Reading it one way cost the census 34 false
disagreements before that was found.

⛔ The row-1 parser and the pointer parser are IMPORTED, never restated — from
`check_frontier_status.py` and `census_memory_pointer_drift.py` respectively.
Two parsers reading one document is the defect class this whole lane is about.

    scripts/check_pointer_currency.py              the pointer as it will be committed
    scripts/check_pointer_currency.py --self-test  the predicate, both ways
    scripts/check_pointer_currency.py --against SHA  re-run over a past commit
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

from census_memory_pointer_drift import (  # noqa: E402
    POINTER,
    block_bullets,
    classify_frontier,
    classify_latest,
    normalise_leaf,
    row_one,
    same_work_unit,
)

HEADLINE = "POINTER-CURRENCY"


def git(*args: str, check: bool = False) -> tuple[int, str]:
    r = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    if check and r.returncode != 0:
        raise RuntimeError(r.stderr)
    return r.returncode, r.stdout


def staged_or_worktree(path: str) -> str:
    """The bytes this commit will carry for `path`.

    At pre-commit the INDEX is the commit, so it wins; a file that is not staged
    falls back to the working tree so the check is still useful when run by hand.

    🔴 **AND THAT PREFERENCE SILENTLY DEFEATED THIS GATE'S OWN FALSIFICATION.**
    Three deliberate breaks were written into the working tree and the check
    returned rc=0 on all three — correctly, because `MEMORY.md` was unmodified in
    the INDEX and that is what it read. A gate reading different bytes from the
    ones in front of you is indistinguishable from a gate with nothing to say,
    which is the failure mode this project has now met in four instruments. The
    preference stays (the index IS the commit); what changes is that the
    divergence is ANNOUNCED, so a hand-run can never be misread as a verdict on
    what is on screen.
    """
    rc, out = git("show", f":{path}")
    if rc == 0:
        p = ROOT / path
        if p.is_file() and p.read_text(encoding="utf-8", errors="replace") != out:
            print(
                f"{HEADLINE}: note — {path} differs between the index and the working tree; "
                f"this check reads the INDEX, because that is what a commit carries. "
                f"`git add {path}` to check what you are looking at.",
                file=sys.stderr,
            )
        return out
    p = ROOT / path
    return p.read_text(encoding="utf-8", errors="replace") if p.is_file() else ""


def head_work_unit() -> tuple[str | None, str | None]:
    """(HEAD's sha, HEAD's subject work-unit id) — (None, None) before the first commit."""
    rc, out = git("log", "-1", "--format=%H\x1f%s")
    if rc != 0 or not out.strip():
        return None, None
    sha, subject = out.strip().split("\x1f", 1)
    from census_memory_pointer_drift import SUBJECT_WORK_UNIT

    m = SUBJECT_WORK_UNIT.match(subject)
    return sha, (m.group(1) if m else None)


def work_unit_already_committed(unit: str, history_from: str = "HEAD") -> str | None:
    """The sha of a commit REACHABLE FROM `history_from` whose subject carries `unit`.

    🔴 **`history_from` EXISTS BECAUSE ITS ABSENCE MADE THIS GATE'S CALIBRATION
    REPORT 151 WHERE THE HISTORY HAS 17.** Walking from today's `HEAD` while
    judging a commit from months ago lets the walk see that commit — and every
    commit after it — so a pointer that CORRECTLY named the commit it was part of
    looked like one naming an already-used id. Classified rather than argued: of
    the 140 flagged, **136 had the very commit under test as the owner** and 0
    had an owner in its future. At real pre-commit time `HEAD` is the parent and
    the commit being made does not exist yet, so the rule was right and the
    harness was wrong — the fourth time in this session that a number described
    the instrument (`docs/knowledge/an-instruments-zero-describes-its-reach.md`).

    ⭐ This is the whole of rule 1's reach, and it is what the corpus supports.
    Of the 152 versions naming a work unit, **135 named the commit being created**
    (an id no commit carries yet) and **12 named the parent**. So a staged id is
    current when it is unused or is HEAD's; an id belonging to some OLDER commit
    is one the author did not update, which is 5 of the 7 measured defects.
    """
    rc, out = git("log", "--format=%H\x1f%s", history_from)
    if rc != 0:
        return None
    from census_memory_pointer_drift import SUBJECT_WORK_UNIT

    for line in out.splitlines():
        if not line.strip():
            continue
        sha, subject = line.split("\x1f", 1)
        m = SUBJECT_WORK_UNIT.match(subject)
        if m and same_work_unit(m.group(1), unit):
            return sha
    return None


def breaches(pointer_text: str, tree_text_for, head_sha, head_unit, *, resolve_sha, unit_owner) -> list[str]:
    """Every POINTER-CURRENCY breach in `pointer_text`. Pure, so the self-test drives it.

    `tree_text_for(name)` returns a tree's markdown; `resolve_sha(s)` returns the
    full sha an abbreviation names, or None when it names no object; `unit_owner(u)`
    returns the sha of a commit ALREADY carrying work unit `u`, or None. All three
    are injected so the predicate is pure and the self-test drives it directly.
    """
    out: list[str] = []
    bullets = block_bullets(pointer_text)
    if not bullets:
        return out  # no current-state block: MEMORY-ARCH owns that, not this gate

    # RULE 1 — latest_commit names the present.
    kind, sha, unit = classify_latest(bullets.get("latest_commit", ""))
    if kind == "carried" and head_sha is not None:
        if sha is not None:
            full = resolve_sha(sha)
            if full is None:
                out.append(
                    f"{POINTER} latest_commit names `{sha}`, which is NOT A VALID OBJECT in this "
                    f"repository. A hand-written hash that resolves to nothing has happened here "
                    f"before (`ba6e77e` at `69b6374`) and no reader noticed for the life of the commit."
                )
            elif full != head_sha:
                out.append(
                    f"{POINTER} latest_commit names `{sha}`, but HEAD is `{head_sha[:7]}`. "
                    f"The pointer names an older commit — re-derive it with `git log -1 --oneline`."
                )
        if unit is not None:
            if head_unit is not None and same_work_unit(unit, head_unit):
                pass  # names the parent: 12 of 152 versions do this, deliberately
            else:
                owner = unit_owner(unit)
                if owner is not None:
                    out.append(
                        f"{POINTER} latest_commit names work unit `{unit}`, which already belongs to "
                        f"commit `{owner[:7]}` — not to HEAD (`{head_unit}`) and not to the commit "
                        f"being made. The pointer was not updated when the leaf closed."
                    )

    # RULE 2 — the frontier leaf agrees with the tree that owns it, and the
    # message names WHICH copy moved rather than assuming the pointer is wrong.
    fkind, tree, leaf, status = classify_frontier(bullets.get("active_work_unit", ""))
    if fkind != "carried" or not tree:
        return out
    tree_text = tree_text_for(tree)
    if not tree_text:
        return out
    row = row_one(tree_text)
    if row is None:
        return out  # a completed tree writes a dash row: no claim to disagree with
    readings = normalise_leaf(tree, leaf or "")
    if row[0] not in readings:
        out.append(
            f"{POINTER} active_work_unit names frontier leaf `{readings[0]}`, and "
            f"`docs/tasks/{tree}.md` row 1 names `{row[0]}`. ⛔ These are two copies of one fact and "
            f"EITHER may be the stale one: in 5 of the 10 instances measured over this repository's "
            f"history the TREE's row 1 was the copy that had not advanced, because a parent leaf had "
            f"just been decomposed and the pointer correctly named the first child. Decide which "
            f"moved, then fix that one."
        )
    elif status is not None:
        declared = row[1].strip("`").lower()
        if declared and status != declared:
            out.append(
                f"{POINTER} calls frontier leaf `{row[0]}` `{status}`, and its row 1 status column "
                f"says `{declared}`. Same fact, two copies, one of them behind."
            )
    return out


def self_test() -> int:
    failures: list[str] = []
    ran = 0

    def check(label: str, cond: bool) -> None:
        # ⛔ Derived, never typed — a self-test that miscounts itself makes a
        # false claim in the line that says it passed.
        nonlocal ran
        ran += 1
        if not cond:
            failures.append(label)

    HEAD = "a" * 40
    tree_ok = (
        "## Current Frontier\n\n| Order | Leaf | Status | Why |\n| --- | --- | --- | --- |\n"
        "| 1 | `DEMO.3` | `pending` | the row under test |\n\n## Next\n"
    )
    trees = {"DEMO": tree_ok}

    def tree_for(name):
        return trees.get(name, "")

    OTHER = "b" * 40  # a real commit that is NOT HEAD — the stub must be able to
    #                    produce one, or control 4 can only ever take the
    #                    "names no object" branch and would pass for the wrong reason.

    def resolve(s):
        for full in (HEAD, OTHER):
            if full.startswith(s):
                return full
        return None

    COMMITTED = {"PROJ-AREA-0009": "c" * 40, "PROJ-AREA-0007": "d" * 40}

    def owner(u):
        for k, v in COMMITTED.items():
            if same_work_unit(k, u):
                return v
        return None

    def run(pointer, *, head=HEAD, unit="PROJ-AREA-0009"):
        return breaches(pointer, tree_for, head, unit, resolve_sha=resolve, unit_owner=owner)

    def pointer(latest, active):
        return (f"# M\n\n## Current state\n- latest_commit: {latest}\n"
                f"- active_work_unit: {active}\n- next_action: unchanged.\n")

    GOOD_ACTIVE = "`DEMO` → frontier leaf: `.3` (`pending`)"

    # 1-2. The clean case is silent, and a derive-on-read pointer makes no claim.
    check("a correct pointer was flagged",
          run(pointer("`PROJ-AREA-0010` (leaf `.3`)", GOOD_ACTIVE)) == [])
    check("a derive-on-read pointer was judged",
          run(pointer("derive on read with `git log -1 --oneline`.", GOOD_ACTIVE)) == [])

    # 3. A hash naming no object — the sharpest measured instance, driven.
    b = run(pointer("`beefbee` — \"X\"", GOOD_ACTIVE))
    check("a hash naming no object was not refused",
          len(b) == 1 and "NOT A VALID OBJECT" in b[0])

    # 4-5. A resolvable hash must be HEAD, and HEAD itself passes.
    check("a hash that is not HEAD was accepted",
          any("older commit" in m for m in run(pointer("`bbbbbbb` — \"X\"", GOOD_ACTIVE))))
    check("HEAD's own hash was refused",
          run(pointer(f"`{HEAD[:7]}` — \"X\"", GOOD_ACTIVE)) == [])

    # 6-7. A work-unit id: HEAD's own is legal (12 of 152 versions), and an id
    #      belonging to an older commit is the 5-of-7 defect shape.
    check("naming the parent work unit was refused",
          run(pointer("`PROJ-AREA-0009` (leaf `.3`)", GOOD_ACTIVE)) == [])
    check("the project prefix was treated as a different work unit",
          run(pointer("`AREA-0009` (leaf `.3`)", GOOD_ACTIVE)) == [])

    # 8. The frontier disagreement fires, and its message names BOTH copies —
    #    the rule the census's symmetry finding makes non-negotiable.
    b = run(pointer("`PROJ-AREA-0010`", "`DEMO` → frontier leaf: `.9` (`pending`)"))
    check("a frontier disagreement was not caught, or its message blamed one side",
          len(b) == 1 and "DEMO.9" in b[0] and "DEMO.3" in b[0] and "EITHER may be the stale one" in b[0])

    # 9. The status column is a second copy of one fact and is checked too.
    b = run(pointer("`PROJ-AREA-0010`", "`DEMO` → frontier leaf: `.3` (`active`)"))
    check("a status disagreement was not caught", len(b) == 1 and "active" in b[0])

    # 10-11. Both readings of the dot-shorthand are accepted — the ambiguity that
    #        cost the census 34 false disagreements before it was found.
    trees["PHASE-1"] = tree_ok.replace("`DEMO.3`", "`PHASE-1.1.1`")
    check("the second reading of the shorthand was refused",
          run(pointer("`PROJ-AREA-0010`", "`PHASE-1` → frontier leaf: `.1.1.1` (`pending`)")) == [])
    trees["PHASE-1"] = tree_ok.replace("`DEMO.3`", "`PHASE-1.1.1.1`")
    check("the literal reading of the shorthand was refused",
          run(pointer("`PROJ-AREA-0010`", "`PHASE-1` → frontier leaf: `.1.1.1` (`pending`)")) == [])
    del trees["PHASE-1"]

    # 12-14. The three silences, each asserted rather than assumed — a gate that
    #        cannot say why it said nothing is indistinguishable from a broken one.
    check("a completed tree's dash row produced a breach",
          run(pointer("`PROJ-AREA-0010`",
                      "`DONE` → frontier leaf: `.3` (`pending`)")) == [])
    trees["DASH"] = ("## Current Frontier\n\n| Order | Leaf | Status | Why |\n| --- | --- | --- | --- |\n"
                     "| — | — | — | **Tree complete** |\n\n## Next\n")
    check("a tree with no numbered row 1 produced a breach",
          run(pointer("`PROJ-AREA-0010`", "`DASH` → frontier leaf: `.3` (`pending`)")) == [])
    check("a prose active_work_unit bullet was judged rather than skipped",
          run(pointer("`PROJ-AREA-0010`", "`DEMO` — the tree is being reshaped")) == [])

    # 15. A file with no current-state block belongs to MEMORY-ARCH, not here.
    check("a pointer with no current-state block was judged",
          run("# M\n\nnothing here\n") == [])
    b = run(pointer("`PROJ-AREA-0007` (leaf `.3`)", GOOD_ACTIVE))
    check("an id belonging to an OLDER commit was not refused",
          len(b) == 1 and "already belongs to commit" in b[0])

    # 16. THE POSITIVE CONTROL ON THE REAL FILES: the imported parsers must still
    #     read today's pointer and today's tree. An absence proved only by a
    #     parser that found nothing is not an absence
    #     (`docs/knowledge/an-instruments-zero-describes-its-reach.md`).
    live = staged_or_worktree(POINTER)
    lb = block_bullets(live)
    k, _, _, _ = classify_frontier(lb.get("active_work_unit", ""))
    check("the live pointer no longer parses — this gate would pass by seeing nothing",
          {"latest_commit", "active_work_unit"} <= set(lb) and k == "carried")

    for f in failures:
        print(f"SELF-TEST: {f}", file=sys.stderr)
    if failures:
        return 1
    print(f"{HEADLINE} self-test: {ran} controls pass — a clean pointer and a derive-on-read one stay "
          "silent, a hash naming no object and a hash that is not HEAD are refused, a parent work unit "
          "is legal and an older one is not, both readings of the dot-shorthand are accepted, the "
          "frontier message names BOTH copies, and the three silences plus the live-file positive "
          "control are asserted rather than assumed")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()

    if len(argv) >= 2 and argv[0] == "--against":
        sha = argv[1]
        pointer_text = git("show", f"{sha}:{POINTER}")[1]

        def tree_for(name):
            return git("show", f"{sha}:docs/tasks/{name}.md")[1]

        rc, out = git("log", "-1", "--format=%H\x1f%s", f"{sha}^")
        if rc == 0 and out.strip():
            head_sha, subject = out.strip().split("\x1f", 1)
            from census_memory_pointer_drift import SUBJECT_WORK_UNIT

            m = SUBJECT_WORK_UNIT.match(subject)
            head_unit = m.group(1) if m else None
        else:
            head_sha, head_unit = None, None
    else:
        pointer_text = staged_or_worktree(POINTER)

        def tree_for(name):
            return staged_or_worktree(f"docs/tasks/{name}.md")

        head_sha, head_unit = head_work_unit()

    def resolve_sha(s: str) -> str | None:
        rc, out = git("rev-parse", "--verify", f"{s}^{{commit}}")
        return out.strip() if rc == 0 else None

    def unit_owner(u: str) -> str | None:
        # ⛔ Bound to the commit being built ON, never to today's HEAD — see
        # work_unit_already_committed's docstring for what that cost.
        return work_unit_already_committed(u, head_sha) if head_sha else None

    found = breaches(pointer_text, tree_for, head_sha, head_unit,
                     resolve_sha=resolve_sha, unit_owner=unit_owner)
    if not found:
        return 0
    print(f"{HEADLINE}: the resume pointer does not name the present:", file=sys.stderr)
    for m in found:
        print(f"    {m}", file=sys.stderr)
    print(
        "\n  MEMORY.md is layer A — what a fresh session reads first. A field that names an\n"
        "  older commit or a closed leaf sends that session to the wrong place, and the\n"
        "  measurement says it happens: 7 of 162 carried latest_commit values and 10 of 385\n"
        "  frontier claims were wrong across this repository's whole history\n"
        "  (python3 -B scripts/census_memory_pointer_drift.py).\n",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
