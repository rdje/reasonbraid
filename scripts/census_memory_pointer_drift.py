#!/usr/bin/env python3
"""Census how often `MEMORY.md`'s derivable current-state fields have been WRONG.

`MEMORY_ARCHITECTURE.md` §6 states a preference rather than a rule: *"**Prefer
derived over hand-written** — a small script can regenerate the current-state
block from `git log` + each tree's frontier row, so it cannot drift."* Today
every field is hand-written. `SIGNOFF-REPAIR.11.4.2.4` owes the measurement that
preference has to be decided on, and `SIGNOFF-REPAIR.11.6` forbids proposing the
generator before the population is counted.

    python3 -B scripts/census_memory_pointer_drift.py             # the census
    python3 -B scripts/census_memory_pointer_drift.py --json      # machine-readable
    python3 -B scripts/census_memory_pointer_drift.py --self-test # its own controls

⛔ **THE DENOMINATOR IS THE TRAP, AND IT IS THIS INSTRUMENT'S WHOLE DIFFICULTY.**
A field that reads *"derive on read with `git log -1 --oneline`"* carries no
value, so it cannot be wrong — counting it as correct would divide the defects
by the wrong population and report a drift rate the history does not have. Every
field is therefore classified `absent` / `derived` / `carried` FIRST, and a rate
is published only over the `carried` versions
(`docs/knowledge/calibrate-over-the-history-that-contains-the-instance.md`).

⛔ **THE READING OF `latest_commit` IS MEASURED, NOT ASSUMED.** A version writing
a bare work-unit id could mean *the commit I am part of* or *the commit before
me*, and picking one by taste manufactures whichever drift rate the choice
implies. Both readings are counted and both are printed; the convention is
whichever the corpus overwhelmingly follows, and disagreements are counted
against THAT.

⛔ **THE FRONTIER PARSER IS IMPORTED FROM THE GATE THAT OWNS IT**
(`scripts/check_frontier_status.py`). `census_frontier_duplicates.py` records
why in as many words: a second parser reading one table is the defect class this
lane exists for, so there is exactly one definition of *row 1*.

⚠️ **AN ERA-1 `**Active tree:**` BULLET IS CURATED PROSE AND IS OFTEN NOT
PARSEABLE**, e.g. *"`BEDROCK-MAINTENANCE` — `.1` done, `.2.1` done …; frontier
`.2` = the transfer loop"*. Reading a leaf out of that by guesswork is exactly
the segmentation over-count `SIGNOFF-REPAIR.11.4.2.1`'s instrument shipped twice
before it was right. A bullet whose shape this instrument declines to read is
reported as `unparsed` — a population to classify, never silently scored as
agreeing (`SIGNOFF-REPAIR.11.4.5.2`).

The ahead-of-origin count has no recoverable ground truth: `origin/main`'s past
positions are not in the repository. It is therefore checked by CONSISTENCY
between consecutive carrying versions — the stated increase must equal the real
number of commits between them. A decrease is a push, not a defect. A stated
increase of ZERO across a real gap is the unambiguous stale signature, and it is
the one `SIGNOFF-REPAIR.11.4.2.4` predicted would be non-zero.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

# ⛔ Imported, never restated — see the module docstring.
from check_frontier_status import frontier_rows  # noqa: E402

POINTER = "MEMORY.md"

# The block heading has had two spellings: bare, and with the overwrite reminder
# `SIGNOFF-REPAIR.11.4.2.3` restored from the §6 template. Both are the block.
BLOCK_HEADING = re.compile(r"^##\s+current state\b", re.IGNORECASE)

# A bullet key in either era's spelling: `- **Latest commit:** …` (484 versions)
# or the §6 template's `- latest_commit: …` (161 versions).
BULLET = re.compile(r"^-\s+(?:\*\*(?P<bold>[^*]+?):\*\*|(?P<plain>[a-z_]+):)\s*(?P<val>.*)$")

CANONICAL = {
    "latest commit": "latest_commit",
    "latest_commit": "latest_commit",
    "active tree": "active_work_unit",
    "active_work_unit": "active_work_unit",
    "in-flight uncommitted work": "in_flight_uncommitted",
    "in_flight_uncommitted": "in_flight_uncommitted",
    "next action": "next_action",
    "next_action": "next_action",
}

# A work-unit id as COMMIT.md shapes it: `REASONBRAID-DOC-0089`, `REPAIR-0058`,
# `BEDROCK-MAINTENANCE-0010`. Anchored to a backtick span so prose cannot match.
WORK_UNIT = re.compile(r"`([A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*-\d{4})`")
SUBJECT_WORK_UNIT = re.compile(r"^([A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*-\d{4})\b")
SHA = re.compile(r"`([0-9a-f]{7,40})`")
# A derivation instruction rather than a value.
DERIVES_COMMIT = re.compile(r"git\s+log\s+-1|git\s+rev-parse", re.IGNORECASE)
DERIVES_AHEAD = re.compile(r"git\s+rev-list\s+--count", re.IGNORECASE)

# An ahead-of-origin count, in every spelling the corpus uses. Ordered: the
# explicit `ahead of origin: N` first, so a line carrying both shapes is read
# by its most specific one.
AHEAD = [
    re.compile(r"ahead of origin:\s*\**\s*(\d+)", re.IGNORECASE),
    re.compile(r"(\d+)\s+commits ahead", re.IGNORECASE),
    re.compile(r"\bahead\s+(\d+)\b", re.IGNORECASE),
]

TREE_NAME = re.compile(r"`([A-Z][A-Z0-9-]*)`")
LEAF_TOKEN = re.compile(r"`((?:[A-Z][A-Z0-9-]*)?\.[0-9][0-9.]*)`")
STATUS_TOKEN = re.compile(r"`(pending|active|in_progress|blocked|done|proposed)`", re.IGNORECASE)


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout


def block_bullets(text: str) -> dict[str, str]:
    """The current-state block's bullets, keyed canonically.

    A bullet's value runs to the NEXT bullet, not to the next newline: era-1
    wrote `**Active tree:**` across six wrapped lines, and a line-at-a-time
    reader sees only its first fragment. That is the same under-read that made
    `census_memory_warnings.py` report 2 warnings where 8 stood.
    """
    lines = text.split("\n")
    try:
        start = next(i for i, l in enumerate(lines) if BLOCK_HEADING.match(l))
    except StopIteration:
        return {}
    out: dict[str, str] = {}
    key: str | None = None
    buf: list[str] = []

    def flush() -> None:
        if key is not None:
            out.setdefault(key, " ".join(buf).strip())

    for line in lines[start + 1 :]:
        if line.startswith("## "):
            break
        m = BULLET.match(line)
        if m:
            flush()
            raw = (m.group("bold") or m.group("plain")).strip().lower()
            key = CANONICAL.get(raw)
            buf = [m.group("val")] if key else []
            if key is None:
                key = None
            continue
        if key is not None and line.startswith("  "):
            buf.append(line.strip())
            continue
        if line.startswith("- "):  # an unkeyed bullet ends the one before it
            flush()
            key, buf = None, []
    flush()
    return out


def block_text(text: str) -> str:
    """The whole current-state block, for fields that are not bullet-keyed.

    The ahead-of-origin count has lived in `latest_commit`, in `**Push cadence:**`
    and in `**PNT:**` across the corpus, so it is searched block-wide rather than
    under a key that would miss two thirds of it.
    """
    lines = text.split("\n")
    try:
        start = next(i for i, l in enumerate(lines) if BLOCK_HEADING.match(l))
    except StopIteration:
        return ""
    out = []
    for line in lines[start + 1 :]:
        if line.startswith("## "):
            break
        out.append(line)
    return "\n".join(out)


def classify_latest(value: str) -> tuple[str, str | None, str | None]:
    """(kind, sha, work_unit) — `carried`, `derived` or `absent`.

    🔴 **THIS FUNCTION'S FIRST CUT REPORTED 227 WRONG SHAs AND THE HISTORY HAS
    25.** It gave a carried token precedence over a derivation instruction, so it
    read `derive with `git log -1 --oneline`; review baseline `9c2d2ba`` as a
    claim that the latest commit was `9c2d2ba` — 217 versions of a bullet whose
    actual claim is *derive it*, with a REVIEW BASELINE cited beside it as a
    different fact. The 227 described the instrument, not the pointer
    (`docs/knowledge/an-instruments-zero-describes-its-reach.md`, the same rule
    from the other end: a large number is a statement about the instrument until
    something proves otherwise).

    ⛔ The repair is ORDER, not a keyword ban, and it was measured before it was
    written: of the 217 bullets carrying both a hint and a token, **217 put the
    hint first and 0 put a token first**, and every one of the 217 SHAs is
    preceded by the literal words `review baseline`. So a derivation instruction
    standing BEFORE every token is the bullet's claim; a token before it is a
    carried value and still checkable.
    """
    if not value:
        return "absent", None, None
    sha = SHA.search(value)
    unit = WORK_UNIT.search(value)
    derive = DERIVES_COMMIT.search(value)
    if sha or unit:
        first_token = min(m.start() for m in (sha, unit) if m)
        if derive and derive.start() < first_token:
            return "derived", None, None
        # 🔴 A THIRD REACH DEFECT, same family as the two above. A long era-2
        # bullet cites a SHA inside the LESSON it carries — `0e47b27` in a note
        # about a 601-commit-old commit — and reading both tokens scored that
        # prose as a latest-commit claim 601 commits stale. Measured before the
        # rule was written: of the 15 bullets carrying both kinds of token, **15
        # put the work-unit id first and 0 put the SHA first**, so the FIRST
        # identifier is the claim and anything after it is a different fact.
        if sha and unit:
            if unit.start() < sha.start():
                return "carried", None, unit.group(1)
            return "carried", sha.group(1), None
        return "carried", (sha.group(1) if sha else None), (unit.group(1) if unit else None)
    if derive:
        return "derived", None, None
    return "unparsed", None, None


def same_work_unit(a: str | None, b: str | None) -> bool:
    """Whether two work-unit ids name one commit, ignoring the project prefix.

    ⚠️ The corpus writes both `REPAIR-0219` and `REASONBRAID-REPAIR-0219` for the
    same unit. Comparing them literally reports a SPELLING as drift, which is the
    over-count `SIGNOFF-REPAIR.11.4.2.1`'s instrument shipped twice; the area and
    number are the identity, and the prefix is decoration.
    """
    if not a or not b:
        return False
    return a == b or a.endswith(b) or b.endswith(a)


def classify_ahead(text: str) -> tuple[str, int | None]:
    if not text:
        return "absent", None
    for pattern in AHEAD:
        m = pattern.search(text)
        if m:
            return "carried", int(m.group(1))
    if DERIVES_AHEAD.search(text):
        return "derived", None
    return "absent", None


def classify_frontier(value: str) -> tuple[str, str | None, str | None, str | None]:
    """(kind, tree, leaf, status) from an `Active tree` / `active_work_unit` bullet.

    ⛔ A leaf is read ONLY from a backticked token that follows the word
    `frontier`. Era-1 bullets list several done leaves before naming the frontier
    one, so the first leaf token in the bullet is usually the WRONG answer —
    taking it would be a silent misread, not a parse failure.
    """
    if not value:
        return "absent", None, None, None
    tree_m = TREE_NAME.search(value)
    tree = tree_m.group(1) if tree_m else None
    lowered = value.lower()
    at = lowered.find("frontier")
    if at < 0:
        return "unparsed", tree, None, None
    tail = value[at:]
    leaf_m = LEAF_TOKEN.search(tail)
    if not leaf_m:
        return "unparsed", tree, None, None
    status_m = STATUS_TOKEN.search(tail[leaf_m.end() : leaf_m.end() + 40])
    return "carried", tree, leaf_m.group(1), (status_m.group(1).lower() if status_m else None)


TREE_TRAILING_NUMBER = re.compile(r"-\d+$")


def normalise_leaf(tree: str | None, leaf: str) -> list[str]:
    """Every reading of a dot-shorthand leaf under `tree`, most literal first.

    🔴 **A SECOND INSTRUMENT DEFECT, and it manufactured 34 disagreements.** The
    literal concatenation `check_tree_index_frontier.sh` uses is right for
    `SIGNOFF-REPAIR` + `.11.4.2`, and WRONG for a tree whose own name ends in a
    number: `PHASE-1` + `.1.1.1` yields `PHASE-1.1.1.1`, a leaf that does not
    exist, where the tree's row 1 says `PHASE-1.1.1`. The pointer was writing the
    full id minus the `PHASE-` prefix, so the tree's trailing `1` is the leaf
    path's FIRST component, not part of the tree name.

    ⛔ Both readings are returned and a match on EITHER is agreement, with the
    second-reading population reported separately — the corpus is genuinely
    ambiguous here and scoring it by one reading invents whichever answer that
    reading implies. Confirmed semantically on `e8adf87`, where the pointer's
    `.1.1.1` and row 1's `PHASE-1.1.1` carry the same description (the
    aggregate/event/outbox library).
    """
    if not tree:
        return [leaf]
    if not leaf.startswith("."):
        return [leaf]
    readings = [f"{tree}{leaf}"]
    m = TREE_TRAILING_NUMBER.search(tree)
    # ⛔ ONLY when the shorthand's first segment IS that trailing number. Without
    # the guard `PHASE-8` + `.5.3` also yields `PHASE-5.3` — another tree's leaf
    # — and a reading that can match anything is a checker that cannot refuse.
    if m and leaf[1:].split(".")[0] == m.group(0)[1:]:
        readings.append(TREE_TRAILING_NUMBER.sub("-", tree) + leaf[1:])
    return readings


def row_one(text: str) -> tuple[str, str] | None:
    """(leaf, status column) of the frontier's numbered row 1, or None.

    `1a`/`1b` are DELIBERATELY not row 1: the corpus uses the suffixed orders for
    parallel lanes beneath a single row 1, and `check_tree_index_frontier.sh`
    already treats the bare `1` as the tree's own claim.
    """
    for _line, order, leaf, status in frontier_rows(text):
        if order.strip() == "1":
            return leaf, status
    return None


def versions() -> list[dict]:
    """Every commit that touched `MEMORY.md`, oldest first, with its subject."""
    raw = git("log", "--reverse", "--format=%H%x1f%P%x1f%s", "--", POINTER)
    out = []
    for line in raw.splitlines():
        if not line.strip():
            continue
        sha, parents, subject = line.split("\x1f", 2)
        out.append({"sha": sha, "parent": parents.split()[0] if parents.split() else None,
                    "subject": subject})
    return out


def blob(sha: str, path: str) -> str:
    r = subprocess.run(["git", "show", f"{sha}:{path}"], cwd=ROOT,
                       capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else ""


def census() -> dict:
    vs = versions()
    # ⛔ THE TRUTH TABLE COVERS EVERY COMMIT, NOT ONLY THE POINTER'S OWN.
    # Keying subjects off the MEMORY.md-touching commits alone made a parent that
    # did NOT touch the pointer look like it had no work-unit id, so versions
    # correctly naming their parent were scored as naming neither. A census whose
    # ground truth is drawn from the same population as its subject cannot see
    # past that population — `docs/CLAIM_VERIFICATION.md` §2's row about tests and
    # implementation written from one spec, in the shape of a lookup table.
    subject_unit = {}
    for line in git("log", "--format=%H\x1f%s").splitlines():
        if not line.strip():
            continue
        sha, subject = line.split("\x1f", 1)
        m = SUBJECT_WORK_UNIT.match(subject)
        subject_unit[sha] = m.group(1) if m else None

    latest = {"absent": 0, "derived": 0, "unparsed": 0, "carried": 0}
    latest_sha_ok = latest_sha_bad = 0
    unit_matches_own = unit_matches_parent = unit_matches_neither = 0
    sha_bad_examples: list[dict] = []
    unit_bad_examples: list[dict] = []

    frontier = {"absent": 0, "unparsed": 0, "carried": 0}
    frontier_agree = frontier_disagree = frontier_no_row = frontier_second_reading = 0
    frontier_status_agree = frontier_status_disagree = 0
    frontier_bad_examples: list[dict] = []

    ahead_kind = {"absent": 0, "derived": 0, "carried": 0}
    ahead_points: list[tuple[str, int]] = []

    for v in vs:
        text = blob(v["sha"], POINTER)
        bullets = block_bullets(text)
        whole = block_text(text)

        kind, sha, unit = classify_latest(bullets.get("latest_commit", ""))
        latest[kind] += 1
        if kind == "carried":
            if sha is not None:
                parent = v["parent"]
                if parent and parent.startswith(sha):
                    latest_sha_ok += 1
                else:
                    latest_sha_bad += 1
                    if len(sha_bad_examples) < 8:
                        sha_bad_examples.append(
                            {"commit": v["sha"][:7], "stated": sha,
                             "parent": (parent or "")[:7], "subject": v["subject"][:70]})
            if unit is not None:
                own = subject_unit.get(v["sha"])
                par = subject_unit.get(v["parent"]) if v["parent"] else None
                if same_work_unit(unit, own):
                    unit_matches_own += 1
                elif same_work_unit(unit, par):
                    unit_matches_parent += 1
                else:
                    unit_matches_neither += 1
                    if len(unit_bad_examples) < 8:
                        unit_bad_examples.append(
                            {"commit": v["sha"][:7], "stated": unit,
                             "own": own, "parent": par, "subject": v["subject"][:70]})

        fkind, tree, leaf, status = classify_frontier(bullets.get("active_work_unit", ""))
        frontier[fkind if fkind in frontier else "unparsed"] += 1
        if fkind == "carried" and tree:
            tree_text = blob(v["sha"], f"docs/tasks/{tree}.md")
            row = row_one(tree_text) if tree_text else None
            if row is None:
                frontier_no_row += 1
            else:
                readings = normalise_leaf(tree, leaf or "")
                if readings and readings[0] == row[0]:
                    frontier_agree += 1
                elif row[0] in readings:
                    frontier_agree += 1
                    frontier_second_reading += 1
                else:
                    frontier_disagree += 1
                    if len(frontier_bad_examples) < 8:
                        frontier_bad_examples.append(
                            {"commit": v["sha"][:7], "stated": " or ".join(readings),
                             "row1": row[0], "subject": v["subject"][:70]})
                if status is not None:
                    if status == row[1].strip("`").lower():
                        frontier_status_agree += 1
                    else:
                        frontier_status_disagree += 1

        akind, value = classify_ahead(whole)
        ahead_kind[akind] += 1
        if akind == "carried" and value is not None:
            ahead_points.append((v["sha"], value))

    transitions = []
    for (sha_a, a), (sha_b, b) in zip(ahead_points, ahead_points[1:]):
        real = int(git("rev-list", "--count", f"{sha_a}..{sha_b}").strip())
        stated = b - a
        if stated < 0:
            verdict = "push"
        elif stated == real:
            verdict = "consistent"
        elif stated == 0 and real > 0:
            verdict = "stale"
        else:
            verdict = "inconsistent"
        transitions.append({"from": sha_a[:7], "to": sha_b[:7],
                            "stated_delta": stated, "real_delta": real, "verdict": verdict})

    return {
        "versions": len(vs),
        "latest_commit": {
            "kinds": latest,
            "sha_agrees_with_parent": latest_sha_ok,
            "sha_disagrees": latest_sha_bad,
            "sha_examples": sha_bad_examples,
            "work_unit_is_own_commit": unit_matches_own,
            "work_unit_is_parent_commit": unit_matches_parent,
            "work_unit_is_neither": unit_matches_neither,
            "work_unit_examples": unit_bad_examples,
        },
        "frontier": {
            "kinds": frontier,
            "agrees_with_row1": frontier_agree,
            "disagrees_with_row1": frontier_disagree,
            "tree_has_no_row1": frontier_no_row,
            "agreed_only_on_the_second_reading": frontier_second_reading,
            "status_agrees": frontier_status_agree,
            "status_disagrees": frontier_status_disagree,
            "examples": frontier_bad_examples,
        },
        "ahead": {
            "kinds": ahead_kind,
            "transitions": len(transitions),
            "stale": sum(1 for t in transitions if t["verdict"] == "stale"),
            "inconsistent": sum(1 for t in transitions if t["verdict"] == "inconsistent"),
            "consistent": sum(1 for t in transitions if t["verdict"] == "consistent"),
            "push": sum(1 for t in transitions if t["verdict"] == "push"),
            "worst": sorted((t for t in transitions if t["verdict"] == "stale"),
                            key=lambda t: -t["real_delta"])[:8],
        },
    }


def report(data: dict) -> None:
    print(f"MEMORY.md current-state drift census — {data['versions']} versions "
          f"(every commit that touched the pointer)\n")

    lc = data["latest_commit"]
    k = lc["kinds"]
    print("latest_commit")
    print(f"  carried a value: {k['carried']}   said 'derive on read': {k['derived']}   "
          f"absent: {k['absent']}   unreadable: {k['unparsed']}")
    print(f"  as an abbreviated SHA — agrees with the commit's parent: {lc['sha_agrees_with_parent']}"
          f"   disagrees: {lc['sha_disagrees']}")
    print(f"  as a work-unit id — names its OWN commit: {lc['work_unit_is_own_commit']}"
          f"   names the PARENT: {lc['work_unit_is_parent_commit']}"
          f"   neither: {lc['work_unit_is_neither']}")
    print("  ⚠️ the two readings are printed rather than chosen: the convention is whichever")
    print("     the corpus follows, and only the residue is a defect.")
    for e in lc["sha_examples"]:
        print(f"    SHA mismatch at {e['commit']}: stated {e['stated']}, parent {e['parent']}")
    for e in lc["work_unit_examples"]:
        print(f"    id mismatch at {e['commit']}: stated {e['stated']}, own {e['own']}, "
              f"parent {e['parent']}")

    fr = data["frontier"]
    k = fr["kinds"]
    print("\nfrontier leaf (active_work_unit / Active tree)")
    print(f"  named a leaf this instrument will read: {k['carried']}   "
          f"prose it declines to read: {k['unparsed']}   absent: {k['absent']}")
    print(f"  agrees with the tree's own frontier row 1: {fr['agrees_with_row1']}"
          f"   disagrees: {fr['disagrees_with_row1']}"
          f"   tree had no numbered row 1: {fr['tree_has_no_row1']}")
    print(f"  of those agreements, {fr['agreed_only_on_the_second_reading']} matched only under the "
          f"second reading of the dot-shorthand (a tree whose name ends in a number)")
    print(f"  status column — agrees: {fr['status_agrees']}   disagrees: {fr['status_disagrees']}")
    for e in fr["examples"]:
        print(f"    {e['commit']}: pointer said {e['stated']}, row 1 was {e['row1']}")

    ah = data["ahead"]
    k = ah["kinds"]
    print("\nahead-of-origin count")
    print(f"  carried a number: {k['carried']}   said how to derive it: {k['derived']}   "
          f"absent: {k['absent']}")
    print(f"  transitions between consecutive carrying versions: {ah['transitions']}")
    print(f"    consistent: {ah['consistent']}   STALE (carried unchanged over real commits): "
          f"{ah['stale']}   otherwise inconsistent: {ah['inconsistent']}   a push reset it: {ah['push']}")
    for t in ah["worst"]:
        print(f"    stale {t['from']}→{t['to']}: stated +{t['stated_delta']}, "
              f"real +{t['real_delta']}")
    print("\n⛔ No ground truth exists for this field — `origin/main`'s past positions are not in")
    print("   the repository — so it is checked by CONSISTENCY, and a decrease is a push.")


def self_test() -> int:
    failures: list[str] = []

    ran = 0

    def check(label: str, cond: bool) -> None:
        # ⛔ The control COUNT is derived, never typed. A hand-carried total
        # guarded by a comment is the stale-constant shape `docs/CLAIM_VERIFICATION.md`
        # names in its own founding table, and a self-test that miscounts itself
        # is an instrument making a false claim in its passing message.
        nonlocal ran
        ran += 1
        if not cond:
            failures.append(label)

    # 1-2. Both spellings of the block heading are the block, and a bullet's
    #      value runs across wrapped lines rather than stopping at the newline.
    era1 = (
        "# MEMORY\n\n## Current state\n\n"
        "- **Active tree:** `DEMO` — `.1` **done**, `.2` **done**; frontier\n"
        "  `.3` (`pending`)\n"
        "- **Latest commit:** derive on read with `git log -1 --oneline`.\n"
        "- **Next action:** do the thing.\n\n## Next\n"
    )
    b = block_bullets(era1)
    check("era-1 bullets were not keyed canonically", set(b) >= {"active_work_unit", "latest_commit"})
    check("a wrapped bullet was truncated at its first line", "`.3`" in b["active_work_unit"])

    era2 = (
        "## Current state (OVERWRITE this block each update — do not append)\n"
        "- latest_commit: `abc1234` — \"X\" (ahead of origin: 153; push at ~300)\n"
        "- active_work_unit: `DEMO` → frontier leaf: `.9.1` (`pending`)\n"
    )
    b2 = block_bullets(era2)
    check("the OVERWRITE-suffixed heading was not recognised as the block",
          set(b2) >= {"latest_commit", "active_work_unit"})

    # 3-5. latest_commit classification, both ways.
    check("a derivation instruction was scored as a carried value",
          classify_latest("derive on read with `git log -1 --oneline`.")[0] == "derived")
    check("an abbreviated SHA was not read as carried",
          classify_latest('`abc1234` — "X"')[:2] == ("carried", "abc1234"))
    check("a work-unit id was not read as carried",
          classify_latest("`REASONBRAID-DOC-0089` (leaf `.13.4.6.3`)")[2] == "REASONBRAID-DOC-0089")
    check("a SHA cited inside the bullet's prose outranked the id that opens it",
          classify_latest("`REASONBRAID-REPAIR-0301` (leaf `.1`): the lesson cites `0e47b27`")
          == ("carried", None, "REASONBRAID-REPAIR-0301"))
    check("a SHA-first bullet lost its SHA",
          classify_latest('`abc1234` — "X (`REPAIR-0001`)"')[1] == "abc1234")
    check("the project prefix was scored as a different work unit",
          same_work_unit("REPAIR-0219", "REASONBRAID-REPAIR-0219")
          and not same_work_unit("REPAIR-0219", "REASONBRAID-REPAIR-0220"))
    check("a value carrying BOTH an id and a derive hint lost its value",
          classify_latest("`REPAIR-0058`; re-derive with `git log -1`")[0] == "carried")

    # 6-8. The frontier bullet: the leaf must come from AFTER the word
    #      `frontier`, or an era-1 bullet listing done leaves first is misread.
    kind, tree, leaf, status = classify_frontier(
        "`DEMO` — `.1` **done**, `.2` **done**; frontier `.3` (`pending`)")
    check("the frontier leaf was taken from before the word 'frontier'",
          (kind, tree, leaf, status) == ("carried", "DEMO", ".3", "pending"))
    check("a prose bullet naming no frontier was scored rather than reported",
          classify_frontier("`DEMO` — the tree is being reshaped")[0] == "unparsed")
    check("the dot shorthand did not normalise against its tree",
          normalise_leaf("DEMO", ".3") == ["DEMO.3"] and normalise_leaf("DEMO", "OTHER.3") == ["OTHER.3"])
    check("a tree whose name ends in a number offered only the literal reading",
          normalise_leaf("PHASE-1", ".1.1.1") == ["PHASE-1.1.1.1", "PHASE-1.1.1"])
    check("a shorthand whose first segment is NOT the tree's number gained a second reading",
          normalise_leaf("PHASE-8", ".5.3") == ["PHASE-8.5.3"])
    check("a derive hint standing BEFORE the token did not win",
          classify_latest("derive with `git log -1 --oneline`; review baseline `9c2d2ba`.")[0]
          == "derived")

    # 9-10. Row 1 comes from the imported parser, and a blank line ends the
    #       table (`SIGNOFF-REPAIR.13.4.6.2` — this exact severance shipped).
    tree_text = (
        "## Current Frontier\n\n| Order | Leaf | Status | Why |\n| --- | --- | --- | --- |\n"
        "| 1 | `DEMO.3` | `pending` | the row under test |\n"
        "| 1a | `DEMO.9` | `pending` | a parallel lane, not row 1 |\n\n## Next\n"
    )
    check("row 1 was not extracted by the imported parser", row_one(tree_text) == ("DEMO.3", "pending"))
    severed = tree_text.replace("| --- | --- | --- | --- |\n", "| --- | --- | --- | --- |\n\n")
    check("rows were read across a blank line, which GFM treats as the table's end",
          row_one(severed) is None)
    check("a suffixed order was accepted as row 1",
          row_one(tree_text.replace("| 1 | `DEMO.3`", "| 1b | `DEMO.3`")) is None)

    # 11-13. The ahead count: every spelling the corpus uses, and the
    #        derivation instruction that is NOT a value.
    check("`ahead of origin: N` was not read",
          classify_ahead("latest: `x` (ahead of origin: 153; push at ~300)") == ("carried", 153))
    check("`N commits ahead` was not read",
          classify_ahead("source was 310 commits ahead of confirmed remote") == ("carried", 310))
    check("a rev-list instruction was scored as a carried count",
          classify_ahead("derive with `git rev-list --count origin/main..HEAD`") == ("derived", None))
    check("an emphasised count was not read",
          classify_ahead("**133 commits ahead** of `origin/main`") == ("carried", 133))

    # 14. The real pointer still parses — the control that fails if the file is
    #     reshaped again and this instrument is not told (`SIGNOFF-REPAIR.11.20.1`:
    #     the last census died exactly this way, silently, reporting a small number).
    live = (ROOT / POINTER).read_text(encoding="utf-8")
    live_bullets = block_bullets(live)
    check("the live MEMORY.md yielded no keyed bullets — the template has been reshaped "
          "and this instrument has not been told",
          {"latest_commit", "active_work_unit", "next_action"} <= set(live_bullets))

    for f in failures:
        print(f"SELF-TEST: {f}", file=sys.stderr)
    if failures:
        return 1
    print(f"census_memory_pointer_drift --self-test: {ran} controls pass — both block headings, a "
          "wrapped bullet, carried-vs-derived both ways for the commit and the ahead count, the "
          "frontier leaf read from after the word 'frontier' and refused as prose otherwise, row 1 "
          "from the imported parser with a severed table yielding none, and the live pointer parsing")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    data = census()
    if "--json" in argv:
        print(json.dumps(data, indent=2))
    else:
        report(data)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
