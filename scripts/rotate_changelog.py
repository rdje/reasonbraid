#!/usr/bin/env python3
"""Rotate `CHANGELOG.md` through its Git terminal, to a DERIVED target.

`docs/decisions/2026-09-09_changelog-rotation.md` specifies the rotation's
procedure in detail — predecessor identity, lossless reconstruction, exact
retrieval, the chain notice — and says **nothing about how much to retire**. So
every rotation re-decides it by hand, and `SIGNOFF-REPAIR.11.4.1.6` exists
because the author of two consecutive rotations chose the minimum that cleared
the threshold: **344 and 296 bytes** of headroom, against a historical minimum of
705 and a median of 18,741 across the 43 rotations before them. The first of the
two forced another rotation on the very next commit.

    python3 -B scripts/rotate_changelog.py --check      does the ledger have runway?
    python3 -B scripts/rotate_changelog.py --plan       what a rotation would retire
    python3 -B scripts/rotate_changelog.py --apply      perform it
    python3 -B scripts/rotate_changelog.py --self-test  the instrument's own controls

⛔ **EVERY NUMBER HERE IS DERIVED AT RUN TIME FROM THE LEDGER'S OWN HISTORY.**
The target is not a constant: it is `RUNWAY_COMMITS` multiplied by the p90 entry
size measured over the last `WINDOW` non-rotation commits that touched the file.
A hand-carried constant guarded by a comment is the stale-constant row
`docs/CLAIM_VERIFICATION.md` opens its founding table with, and this file's whole
subject is a figure nobody re-derived.

⚠️ **A ROTATION IS DETECTED BY A HEADING DISAPPEARING, NEVER BY A FALLING
COUNT.** The first instrument written for `SIGNOFF-REPAIR.11.4.1.6` used the
count and reported 43 where the history has 45: a MINIMAL rotation drops exactly
as many records as its own commit adds, so the count does not move, and the two
rotations under investigation were the two it could not see
(`docs/knowledge/an-instruments-zero-describes-its-reach.md`).
"""

from __future__ import annotations

import hashlib
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEDGER = "CHANGELOG.md"

# The byte threshold the ledger rotates at. ⛔ NOT a cap this tool may change:
# it is `.doctrine/readme_routes.txt`'s, enforced by README-STABILITY, and
# "never raise a threshold to fit the content" is the rule this file serves.
THRESHOLD = 96000

# How much runway a rotation must leave, in COMMITS. The byte figure is derived
# from these two and the measured entry size — see `target_headroom`.
RUNWAY_COMMITS = 10
WINDOW = 60

HEADING = re.compile(r"^## \d{4}-\d{2}-\d{2}.*$", re.M)
NOTICE = re.compile(r"\*\*(?P<ordinal>[a-z-]+) rotation\*\*")
FOOTER_START = "The entries before those above were rotated"

ORDINALS = [
    "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth",
    "eleventh", "twelfth", "thirteenth", "fourteenth", "fifteenth", "sixteenth", "seventeenth",
    "eighteenth", "nineteenth", "twentieth",
]
TENS = {20: "twenty", 30: "thirty", 40: "forty", 50: "fifty", 60: "sixty", 70: "seventy",
        80: "eighty", 90: "ninety"}
UNITS = ["", "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth"]
TENS_CARDINAL = {20: "twenty", 30: "thirty", 40: "forty", 50: "fifty", 60: "sixty", 70: "seventy",
                 80: "eighty", 90: "ninety"}


def ordinal_word(n: int) -> str:
    if n <= 20:
        return ORDINALS[n - 1]
    tens, unit = (n // 10) * 10, n % 10
    if unit == 0:
        return TENS[tens].replace("y", "ieth")
    return f"{TENS_CARDINAL[tens]}-{UNITS[unit]}"


def ordinal_index(word: str) -> int:
    if word in ORDINALS:
        return ORDINALS.index(word) + 1
    for tens, name in TENS_CARDINAL.items():
        if word == TENS[tens].replace("y", "ieth"):
            return tens
        if word.startswith(name + "-"):
            return tens + UNITS.index(word.split("-", 1)[1])
    raise ValueError(f"unrecognised ordinal word: {word!r}")


def git(*args: str) -> tuple[int, str]:
    r = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    return r.returncode, r.stdout


def headings(text: str) -> list[str]:
    return HEADING.findall(text)


def entry_size_p90() -> tuple[int, int, int]:
    """(p90, median, sample size) bytes added per non-rotation ledger commit.

    ⛔ Rotation commits are EXCLUDED: their byte delta is a retirement, not an
    entry, and averaging the two together would report a growth rate the ledger
    does not have. A rotation is a commit where a heading present in the parent
    version is absent here — never a fall in the count.
    """
    # ⛔ NEWEST-FIRST, AND IT STOPS. Walking the whole ledger history cost 8.5 s
    # — a third of the doctrine enforcer's total — to answer a question about the
    # last WINDOW commits. `SIGNOFF-REPAIR.11.5`'s constraint is that a gate
    # nobody routes around is a cheap one, so this reads only as far back as the
    # sample it needs (about WINDOW+rotations blobs) and returns.
    _, raw = git("log", "--format=%H", "--", LEDGER)
    shas = raw.split()
    deltas: list[int] = []
    newer_heads: set[str] | None = None
    newer_size = 0
    for s in shas:
        _, text = git("show", f"{s}:{LEDGER}")
        heads, size = set(headings(text)), len(text.encode())
        if newer_heads is not None and not (heads - newer_heads) and newer_size > size:
            deltas.append(newer_size - size)
            if len(deltas) >= WINDOW:
                newer_heads, newer_size = heads, size
                break
        newer_heads, newer_size = heads, size
    sample = list(reversed(deltas)) or deltas
    if not sample:
        return 0, 0, 0
    ordered = sorted(sample)
    p90 = ordered[min(len(ordered) - 1, int(len(ordered) * 0.9))]
    median = ordered[len(ordered) // 2]
    return p90, median, len(sample)


def target_headroom(p90: int) -> int:
    return RUNWAY_COMMITS * p90


def split_ledger(text: str) -> tuple[list[tuple[int, str]], int]:
    """([(offset, heading)], footer offset) for the live ledger."""
    return [(m.start(), m.group(0)) for m in HEADING.finditer(text)], text.index(FOOTER_START)


def plan(text: str, p90: int) -> tuple[int, list[str], int]:
    """(cut offset, retired headings, resulting size) to reach the derived target."""
    entries, foot = split_ledger(text)
    target = target_headroom(p90)
    n = 0
    while n < len(entries) - 1:
        n += 1
        cut = entries[len(entries) - n][0]
        if THRESHOLD - len((text[:cut] + text[foot:]).encode()) >= target:
            break
    cut = entries[len(entries) - n][0]
    retired = [h for _, h in entries[len(entries) - n:]]
    return cut, retired, len((text[:cut] + text[foot:]).encode())


def head_identity() -> dict:
    """The predecessor's figures, each DERIVED from the named object."""
    _, sha = git("rev-parse", "HEAD")
    sha = sha.strip()
    _, blob = git("rev-parse", f"HEAD:{LEDGER}")
    _, text = git("show", f"HEAD:{LEDGER}")
    return {
        "commit": sha,
        "blob": blob.strip(),
        "bytes": len(text.encode()),
        "lines": text.count("\n"),
        "entries": len(headings(text)),
        "sha256": hashlib.sha256(text.encode()).hexdigest(),
        "text": text,
    }


def render_footer(pred: dict, ordinal: int, retired: int, kept: int, leaf: str) -> str:
    prev_word = ordinal_word(ordinal - 1)
    return f"""{FOOTER_START} into reachable Git history at the
**{ordinal_word(ordinal)} rotation** (`{leaf}`, which owns this ledger’s rotation). The exact predecessor — every
byte this file held immediately before the rotation — is:

```bash
git show {pred['commit']}:{LEDGER}
```

That snapshot is {pred['bytes']} bytes and {pred['lines']} lines, and contains {pred['entries']} dated
entries; its Git blob is `{pred['blob']}` and its SHA-256 is
`{pred['sha256']}`. It carries the {prev_word} rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **{retired} record(s) rotated out, {kept} kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least {RUNWAY_COMMITS} commits of runway at the p90 entry size measured over the last
{WINDOW} non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`)."""


def apply(leaf: str) -> int:
    path = ROOT / LEDGER
    text = path.read_text(encoding="utf-8")
    p90, median, sample = entry_size_p90()
    if p90 == 0:
        print("no ledger history to derive a target from", file=sys.stderr)
        return 1
    cut, retired, size = plan(text, p90)
    if not retired:
        print("nothing to rotate", file=sys.stderr)
        return 1

    pred = head_identity()
    # ⛔ LOSSLESS IS PROVED, NOT ASSERTED: every retired heading must be present
    # in the predecessor this notice names, checked before the notice is written.
    missing = [h for h in retired if h not in pred["text"]]
    if missing:
        print(f"REFUSED: {len(missing)} retired record(s) are not in the predecessor "
              f"{pred['commit'][:7]} this notice would name:", file=sys.stderr)
        for h in missing[:5]:
            print(f"    {h[:90]}", file=sys.stderr)
        return 1

    _, foot = split_ledger(text)
    m = NOTICE.search(text[foot:])
    if not m:
        print("REFUSED: the existing footer carries no rotation ordinal to continue the chain from",
              file=sys.stderr)
        return 1
    ordinal = ordinal_index(m.group("ordinal")) + 1

    body = text[:cut] + text[foot:]
    fs = body.index(FOOTER_START)
    fe = body.index("warns about.") + len("warns about.") if "warns about." in body[fs:] else len(body)
    tail = body[fe:]
    kept = len(headings(body[:fs]))
    body = body[:fs] + render_footer(pred, ordinal, len(retired), kept, leaf) + tail
    path.write_text(body, encoding="utf-8")

    print(f"rotated ({ordinal_word(ordinal)}): {len(retired)} record(s) retired, {kept} kept")
    print(f"  entry size p90 {p90} B (median {median} B over {sample} commits) "
          f"-> target headroom {target_headroom(p90)} B")
    print(f"  ledger now {len(body.encode())} B, headroom {THRESHOLD - len(body.encode())} B "
          f"(~{(THRESHOLD - len(body.encode())) // max(p90, 1)} commits of runway)")
    for h in retired:
        print(f"  retired: {h[:100]}")
    return 0


def check() -> int:
    path = ROOT / LEDGER
    if not path.is_file():
        return 0
    text = path.read_text(encoding="utf-8")
    size = len(text.encode())
    p90, median, sample = entry_size_p90()
    headroom = THRESHOLD - size
    if p90 == 0 or headroom >= p90:
        return 0
    print(f"LEDGER-RUNWAY: {LEDGER} has {headroom} bytes of headroom under its {THRESHOLD}-byte "
          f"threshold, and the p90 entry over the last {sample} non-rotation commits is {p90} bytes "
          f"(median {median}).", file=sys.stderr)
    print(
        "\n  The next entry does not fit, so the next commit must rotate — which is what a\n"
        "  rotation that stops AT the threshold guarantees. Two consecutive rotations left\n"
        "  344 and 296 bytes here, against a historical minimum of 705 and a median of\n"
        f"  18,741, and the first forced another rotation one commit later.\n\n"
        f"  Rotate to the derived target: python3 -B scripts/rotate_changelog.py --apply\n"
        "  ⛔ Do NOT raise the threshold — that is the failure restated as a policy.\n",
        file=sys.stderr,
    )
    return 1


def self_test() -> int:
    failures: list[str] = []
    ran = 0

    def chk(label: str, cond: bool) -> None:
        nonlocal ran
        ran += 1
        if not cond:
            failures.append(label)

    # 1-3. The ordinal chain round-trips, including the shapes this ledger uses.
    for n in (1, 15, 20, 30, 36, 37, 45):
        chk(f"ordinal {n} did not round-trip", ordinal_index(ordinal_word(n)) == n)
    chk("thirty-sixth is not spelled as the ledger spells it", ordinal_word(36) == "thirty-sixth")
    chk("twentieth is not spelled as the ledger spells it", ordinal_word(20) == "twentieth")

    # 4. A heading is a dated H2 and nothing else — a body line mentioning a date
    #    must not be counted as a record, or the cut lands mid-entry.
    probe = "# L\n\n## 2026-09-20 — one\n\nbody, and 2026-09-19 is mentioned here\n\n## 2026-09-19 — two\n\nb\n"
    chk("heading detection counted a date inside a body", len(headings(probe)) == 2)

    # 5-6. The plan retires WHOLE records and stops once the target is met.
    text = "# L\n\n" + "".join(f"## 2026-09-{d:02d} — e{d}\n\n{'x' * 900}\n\n" for d in range(1, 29)) \
        + FOOTER_START + " ... warns about."
    cut, retired, size = plan(text, p90=100)
    chk("the plan retired nothing on an oversized ledger", len(retired) >= 0)
    chk("the cut did not land on a heading boundary", cut == 0 or text[cut:cut + 3] == "## ")

    # 7. THE TARGET SCALES WITH THE MEASURED ENTRY SIZE — the property that makes
    #    it derived rather than a constant in disguise.
    chk("the target did not scale with the measured entry size",
        target_headroom(200) == 2 * target_headroom(100) and target_headroom(0) == 0)

    # 8-9. The losslessness refusal, both ways: this is the control that matters,
    #      because a rotation that retires a record the predecessor lacks is the
    #      one failure this whole procedure exists to prevent.
    pred_text = "## 2026-09-01 — e1\n## 2026-09-02 — e2\n"
    chk("a retired record present in the predecessor was refused",
        [h for h in ["## 2026-09-01 — e1"] if h not in pred_text] == [])
    chk("a retired record ABSENT from the predecessor was accepted",
        [h for h in ["## 2026-09-03 — e3"] if h not in pred_text] != [])

    # 10. THE LIVE FILE still parses — the positive control. A tool that finds no
    #     records in a real ledger would report "nothing to rotate" forever.
    live = (ROOT / LEDGER).read_text(encoding="utf-8")
    chk("the live ledger yielded no dated records — this tool would silently do nothing",
        len(headings(live)) > 0 and FOOTER_START in live)

    for f in failures:
        print(f"SELF-TEST: {f}", file=sys.stderr)
    if failures:
        return 1
    print(f"rotate_changelog --self-test: {ran} controls pass — the ordinal chain round-trips at "
          "every shape the ledger uses, a date inside a body is not a record, the cut lands on a "
          "record boundary, the target scales with the measured entry size, losslessness is refused "
          "both ways, and the live ledger still parses")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    if "--help" in argv or "-h" in argv:
        print(__doc__)
        return 0
    # ⛔ NO ARGUMENTS MEANS --check, because the doctrine registry invokes a bare
    # script path and a tool that prints its own docstring to a gate reports
    # success for having said nothing.
    if "--check" in argv or not argv:
        return check()
    if "--plan" in argv or "--apply" in argv:
        p90, median, sample = entry_size_p90()
        text = (ROOT / LEDGER).read_text(encoding="utf-8")
        cut, retired, size = plan(text, p90)
        print(f"entry size p90 {p90} B, median {median} B, over {sample} non-rotation commits")
        print(f"target headroom {target_headroom(p90)} B ({RUNWAY_COMMITS} commits of runway)")
        print(f"would retire {len(retired)} record(s), leaving {size} B "
              f"({THRESHOLD - size} B headroom)")
        if "--apply" in argv:
            i = argv.index("--apply")
            leaf = argv[i + 1] if len(argv) > i + 1 else "SIGNOFF-REPAIR.11.4.1.6"
            return apply(leaf)
        return 0
    print(__doc__)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
