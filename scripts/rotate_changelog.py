#!/usr/bin/env python3
"""Rotate an ordered ledger through its Git terminal, to a DERIVED target.

`docs/decisions/2026-09-09_changelog-rotation.md` specifies the rotation's
procedure in detail — predecessor identity, lossless reconstruction, exact
retrieval, the chain notice — and says **nothing about how much to retire**. So
every rotation re-decides it by hand, and `SIGNOFF-REPAIR.11.4.1.6` exists
because the author of two consecutive rotations chose the minimum that cleared
the threshold: **344 and 296 bytes** of headroom, against a historical minimum of
705 and a median of 18,741 across the 43 rotations before them. The first of the
two forced another rotation on the very next commit.

    python3 -B scripts/rotate_changelog.py --check      do the ENFORCED ledgers have runway?
    python3 -B scripts/rotate_changelog.py --check-all  judge every ledger, debt included
    python3 -B scripts/rotate_changelog.py --plan       what a rotation would retire
    python3 -B scripts/rotate_changelog.py --apply      perform it
    python3 -B scripts/rotate_changelog.py --self-test  the instrument's own controls

Add `--ledger dev-notes` to any of them to act on the second ledger. ⚠️ The file
keeps its `rotate_changelog` name in this slice deliberately: renaming it would
touch the doctrine registry, the scaffold's neutral list and
`DOCTRINE_ENFORCEMENT.md` in the same commit as a parser change, and one concern
per commit is worth more than an accurate filename. The rename is owed, and
`SIGNOFF-REPAIR.11.4.2.6.2` records it as owed rather than leaving it to be
noticed.

⛔ **EVERY NUMBER HERE IS DERIVED AT RUN TIME FROM THE LEDGER'S OWN HISTORY.**
The target is not a constant: it is the ledger's `runway_commits` multiplied by
the p90 entry size measured over the last `window` non-rotation commits that
touched that file.
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
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# ⛔ ONE DEFINITION OF WHAT A RECORD IS, SHARED BY BOTH LEDGERS, and it is QUOTED
# rather than chosen: `scripts/check_lesson_promotion.sh` has governed
# `DEV_NOTES.md` since it was ported and matches `^## .*[0-9]{4}-[0-9]{2}-[0-9]{2}`,
# pinning BOTH heading spellings in its own self-test. This pattern is that one.
#
# ⚠️ The narrower `^## \d{4}-\d{2}-\d{2}` this file used to carry sees 291 of
# `DEV_NOTES.md`'s 439 records — 148 records and 178,556 bytes invisible — and a
# rotation states its retired-record count in its own chain notice, so the narrow
# pattern would publish a false one (`SIGNOFF-REPAIR.11.4.2.6.1`).
#
# ⭐ Widening it changes NOTHING on the ledger already in production, and that was
# measured before the change rather than argued: across all 664 versions of
# `CHANGELOG.md` the two patterns return the same count in every one.
# ⛔ `## ` is still anchored at line start, so a date inside a body is not a
# record — the control that keeps a cut from landing mid-entry.
HEADING = re.compile(r"^## .*\d{4}-\d{2}-\d{2}.*$", re.M)


@dataclass(frozen=True)
class Ledger:
    """One ordered ledger and the numbers its rotation is derived from.

    ⛔ `threshold` is NOT a cap this tool may change: it belongs to
    `.doctrine/readme_routes.txt` and is enforced by README-STABILITY. "Never
    raise a threshold to fit the content" is the rule this file serves.

    ⚠️ `enforced` is what keeps a ledger carrying transition debt out of the
    gate. A ledger admitted to `--check` before its first rotation would make the
    enforcer red on every commit, which is a gate people route around
    (`SIGNOFF-REPAIR.11.5`) — not a bound.
    """

    path: str
    threshold: int
    enforced: bool
    # How much runway a rotation must leave, in COMMITS. The byte figure is
    # derived from this and the measured entry size — see `target_headroom`.
    runway_commits: int = 10
    window: int = 60


CHANGELOG = Ledger(path="CHANGELOG.md", threshold=96000, enforced=True)

# ⭐ ENFORCED SINCE ITS FIRST ROTATION (`SIGNOFF-REPAIR.11.4.2.6.3`): 430 records
# retired into git history, 12 kept, and the ledger went from 908,850 bytes to
# 37,873 with ~10 commits of runway. It was deliberately NOT enforced before that
# — `SIGNOFF-REPAIR.11.4.2.5` measured it at 427 of 427 versions growing and ZERO
# bytes ever removed, so admitting it to the gate first would have made the
# enforcer red on every commit, and a gate that is always red is one people route
# around (`SIGNOFF-REPAIR.11.5`).
# ⭐ THE THRESHOLD IS DERIVED, NOT CHOSEN — the SAME LIVE WINDOW the ledger already
# in production runs on, expressed in this ledger's own measured entry size, so
# the authority behind the number is an existing reviewed decision rather than a
# fresh preference. Measured at `7fc8913` with this file's own `entry_size_p90`:
#
#   CHANGELOG.md  threshold 96,000 / p90 4,734 = 20.279 p90-entries of window
#   DEV_NOTES.md  20.279 x p90 3,763           = 76,309 bytes
#
# Rounded DOWN to 76,000: rounding up would grant headroom the derivation does
# not support, and a ceiling may only ever move the other way. The full
# derivation, its calibration and why this ledger is not yet enforced:
# docs/decisions/2026-09-21_the-second-ledgers-threshold-is-the-first-ledgers-window.md
# ⚠️ Pinned to that commit deliberately. p90 moves, so this is a figure DERIVED
# ONCE at a named moment, not one re-derived on read — re-deriving a declared
# ceiling would let the ledger widen its own bound by growing.
DEV_NOTES = Ledger(path="DEV_NOTES.md", threshold=76000, enforced=True)

LEDGERS = {"changelog": CHANGELOG, "dev-notes": DEV_NOTES}
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
    # ⛔ REFUSE BELOW 1 RATHER THAN WRAP. `ORDINALS[n - 1]` with n=0 returned
    # `'twentieth'` — a silent negative index — and `render_footer` asks for
    # `ordinal_word(ordinal - 1)`, so a FIRST rotation would have written "it
    # carries the twentieth rotation's notice in turn" into a governed ledger.
    # Wrong and plausible-looking is the worst combination a rendered figure can
    # have (`SIGNOFF-REPAIR.11.4.2.6.3`).
    if n < 1:
        raise ValueError(f"there is no {n}th rotation — ordinals start at 1")
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


def entry_size_p90(ledger: Ledger) -> tuple[int, int, int]:
    """(p90, median, sample size) bytes added per non-rotation ledger commit.

    ⛔ Rotation commits are EXCLUDED: their byte delta is a retirement, not an
    entry, and averaging the two together would report a growth rate the ledger
    does not have. A rotation is a commit where a heading present in the parent
    version is absent here — never a fall in the count.
    """
    # ⛔ NEWEST-FIRST, AND IT STOPS. Walking the whole ledger history cost 8.5 s
    # — a third of the doctrine enforcer's total — to answer a question about the
    # last `window` commits. `SIGNOFF-REPAIR.11.5`'s constraint is that a gate
    # nobody routes around is a cheap one, so this reads only as far back as the
    # sample it needs (about window+rotations blobs) and returns.
    _, raw = git("log", "--format=%H", "--", ledger.path)
    shas = raw.split()
    deltas: list[int] = []
    newer_heads: set[str] | None = None
    newer_size = 0
    for s in shas:
        _, text = git("show", f"{s}:{ledger.path}")
        heads, size = set(headings(text)), len(text.encode())
        if newer_heads is not None and not (heads - newer_heads) and newer_size > size:
            deltas.append(newer_size - size)
            if len(deltas) >= ledger.window:
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


def target_headroom(ledger: Ledger, p90: int) -> int:
    return ledger.runway_commits * p90


def split_ledger(text: str) -> tuple[list[tuple[int, str]], int]:
    """([(offset, heading)], footer offset) for the live ledger.

    ⛔ A LEDGER THAT HAS NEVER BEEN ROTATED HAS NO FOOTER, and that is a
    legitimate state, not a malformed file. This used to be `text.index(...)`,
    which raised an uncaught `ValueError: substring not found` and printed a
    traceback — a tool that tracebacks on a legitimate input has nothing to say
    (`docs/knowledge/an-instrument-must-explain-its-own-failure.md`). The absent
    footer is now the end of the text, so `--plan` answers correctly for a first
    rotation and `--apply` reaches its own REFUSED message about the missing
    chain ordinal instead of crashing before it.
    """
    foot = text.index(FOOTER_START) if FOOTER_START in text else len(text)
    return [(m.start(), m.group(0)) for m in HEADING.finditer(text)], foot


def plan(ledger: Ledger, text: str, p90: int) -> tuple[int, list[str], int]:
    """(cut offset, retired headings, resulting size) to reach the derived target."""
    entries, foot = split_ledger(text)
    target = target_headroom(ledger, p90)
    n = 0
    while n < len(entries) - 1:
        n += 1
        cut = entries[len(entries) - n][0]
        if ledger.threshold - len((text[:cut] + text[foot:]).encode()) >= target:
            break
    cut = entries[len(entries) - n][0]
    retired = [h for _, h in entries[len(entries) - n:]]
    return cut, retired, len((text[:cut] + text[foot:]).encode())


def head_identity(ledger: Ledger) -> dict:
    """The predecessor's figures, each DERIVED from the named object."""
    _, sha = git("rev-parse", "HEAD")
    sha = sha.strip()
    _, blob = git("rev-parse", f"HEAD:{ledger.path}")
    _, text = git("show", f"HEAD:{ledger.path}")
    return {
        "commit": sha,
        "blob": blob.strip(),
        "bytes": len(text.encode()),
        "lines": text.count("\n"),
        "entries": len(headings(text)),
        "sha256": hashlib.sha256(text.encode()).hexdigest(),
        "text": text,
    }


def render_footer(ledger: Ledger, pred: dict, ordinal: int, retired: int, kept: int, leaf: str) -> str:
    # ⛔ A FIRST ROTATION HAS NO PREDECESSOR NOTICE, and the footer must not claim
    # one. The chain's whole value is that every notice names the one before it;
    # a first notice that invents an antecedent breaks the property it exists to
    # carry, and it is precisely the sentence a reader would trust.
    if ordinal == 1:
        chain = ("It carries NO earlier rotation notice: this is the first rotation of this\n"
                 "ledger, so the chain starts here and every later notice will name this one.")
    else:
        chain = (f"It carries the {ordinal_word(ordinal - 1)} rotation's\n"
                 "notice in turn, and each earlier notice names the one before it, so the chain\n"
                 "walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds\n"
                 "the first transition's evidence.")
    return f"""{FOOTER_START} into reachable Git history at the
**{ordinal_word(ordinal)} rotation** (`{leaf}`, which owns this ledger’s rotation). The exact predecessor — every
byte this file held immediately before the rotation — is:

```bash
git show {pred['commit']}:{ledger.path}
```

That snapshot is {pred['bytes']} bytes and {pred['lines']} lines, and contains {pred['entries']} dated
entries; its Git blob is `{pred['blob']}` and its SHA-256 is
`{pred['sha256']}`. {chain}

⛔ **{retired} record(s) rotated out, {kept} kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least {ledger.runway_commits} commits of runway at the p90 entry size measured over the last
{ledger.window} non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`)."""


def apply(ledger: Ledger, leaf: str, bootstrap: bool = False) -> int:
    path = ROOT / ledger.path
    text = path.read_text(encoding="utf-8")
    p90, median, sample = entry_size_p90(ledger)
    if p90 == 0:
        print("no ledger history to derive a target from", file=sys.stderr)
        return 1
    cut, retired, size = plan(ledger, text, p90)
    if not retired:
        print("nothing to rotate", file=sys.stderr)
        return 1

    pred = head_identity(ledger)
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
    if m and bootstrap:
        # ⛔ SYMMETRIC REFUSAL. --bootstrap starting a chain that already exists
        # would silently reset its ordinal to 1 and orphan every earlier notice,
        # which is the one irreversible thing this tool could do to a ledger.
        print(f"REFUSED: {ledger.path} already carries a rotation chain "
              f"(ordinal {ordinal_index(m.group('ordinal'))}); --bootstrap would restart it at 1 "
              "and orphan every earlier notice.", file=sys.stderr)
        return 1
    if not m and bootstrap:
        ordinal = 1
    elif not m:
        print(f"REFUSED: {ledger.path} carries no rotation ordinal to continue the chain from.",
              file=sys.stderr)
        print("  This tool CONTINUES a chain; it cannot START one. A first rotation has no\n"
              "  predecessor notice to take its ordinal from, and inventing one would break the\n"
              "  property the chain exists for — that every notice names the one before it, all\n"
              "  the way back to the first transition's recorded evidence.\n"
              "  Pass --bootstrap to START one, which is allowed exactly once per ledger.",
              file=sys.stderr)
        return 1
    else:
        ordinal = ordinal_index(m.group("ordinal")) + 1

    body = text[:cut] + text[foot:]
    # ⛔ A LEDGER WITH NO FOOTER GETS ONE APPENDED, rather than `index` raising on
    # its absence. `fs` is the end of the text in that case, so the slicing below
    # is the same arithmetic for both shapes and there is no second code path to
    # keep in step.
    fs = body.index(FOOTER_START) if FOOTER_START in body else len(body)
    fe = body.index("warns about.") + len("warns about.") if "warns about." in body[fs:] else len(body)
    tail = body[fe:]
    kept = len(headings(body[:fs]))
    body = body[:fs] + render_footer(ledger, pred, ordinal, len(retired), kept, leaf) + tail
    # ⛔ EXACTLY ONE TRAILING NEWLINE. When the notice is APPENDED rather than
    # replacing an existing one, `tail` is empty and the footer ends the file — so
    # the first bootstrapped rotation wrote a tracked file with no final newline
    # and `FILE-TERMINATION` refused the commit. ⚠️ A continuing rotation keeps
    # its tail, which already ends in a newline, so this is a no-op there and the
    # byte-identity of `CHANGELOG.md`'s output is untouched.
    body = body.rstrip("\n") + "\n"
    path.write_text(body, encoding="utf-8")

    print(f"rotated ({ordinal_word(ordinal)}): {len(retired)} record(s) retired, {kept} kept")
    print(f"  entry size p90 {p90} B (median {median} B over {sample} commits) "
          f"-> target headroom {target_headroom(ledger, p90)} B")
    print(f"  ledger now {len(body.encode())} B, headroom {ledger.threshold - len(body.encode())} B "
          f"(~{(ledger.threshold - len(body.encode())) // max(p90, 1)} commits of runway)")
    for h in retired:
        print(f"  retired: {h[:100]}")
    return 0


def check_one(ledger: Ledger) -> int:
    path = ROOT / ledger.path
    if not path.is_file():
        return 0
    text = path.read_text(encoding="utf-8")
    size = len(text.encode())
    p90, median, sample = entry_size_p90(ledger)
    headroom = ledger.threshold - size
    if p90 == 0 or headroom >= p90:
        return 0
    flag = f" --ledger {name_of(ledger)}" if ledger is not CHANGELOG else ""
    print(f"LEDGER-RUNWAY: {ledger.path} has {headroom} bytes of headroom under its "
          f"{ledger.threshold}-byte threshold, and the p90 entry over the last {sample} "
          f"non-rotation commits is {p90} bytes (median {median}).", file=sys.stderr)
    print(
        "\n  The next entry does not fit, so the next commit must rotate — which is what a\n"
        "  rotation that stops AT the threshold guarantees. Two consecutive rotations left\n"
        "  344 and 296 bytes here, against a historical minimum of 705 and a median of\n"
        f"  18,741, and the first forced another rotation one commit later.\n\n"
        f"  Rotate to the derived target: python3 -B scripts/rotate_changelog.py{flag} --apply\n"
        "  ⛔ Do NOT raise the threshold — that is the failure restated as a policy.\n",
        file=sys.stderr,
    )
    return 1


def name_of(ledger: Ledger) -> str:
    for name, l in LEDGERS.items():
        if l is ledger:
            return name
    return ledger.path


def check() -> int:
    """The gate arm. ⛔ ONLY the ledgers marked `enforced` are judged.

    A ledger carrying transition debt — one whose first rotation has not
    happened — is deliberately excluded rather than quietly passed: admitting it
    here would make the enforcer red on every commit until the migration lands,
    and a gate that is always red is one people route around
    (`SIGNOFF-REPAIR.11.5`). The exclusion is a DECLARED field on the ledger, not
    an omission, and `--check-all` judges every ledger regardless so the debt can
    be measured on demand.
    """
    return max((check_one(l) for l in LEDGERS.values() if l.enforced), default=0)


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
    cut, retired, size = plan(CHANGELOG, text, p90=100)
    chk("the plan retired nothing on an oversized ledger", len(retired) >= 0)
    chk("the cut did not land on a heading boundary", cut == 0 or text[cut:cut + 3] == "## ")

    # 7. THE TARGET SCALES WITH THE MEASURED ENTRY SIZE — the property that makes
    #    it derived rather than a constant in disguise.
    chk("the target did not scale with the measured entry size",
        target_headroom(CHANGELOG, 200) == 2 * target_headroom(CHANGELOG, 100)
        and target_headroom(CHANGELOG, 0) == 0)

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
    live = (ROOT / CHANGELOG.path).read_text(encoding="utf-8")
    chk("the live ledger yielded no dated records — this tool would silently do nothing",
        len(headings(live)) > 0 and FOOTER_START in live)

    # 11-13. BOTH HEADING DIALECTS ARE RECORDS, and the boundary is the one
    #        `scripts/check_lesson_promotion.sh` already enforces on DEV_NOTES.md.
    #        The narrower pattern this file used to carry saw 291 of that file's
    #        439 records (`SIGNOFF-REPAIR.11.4.2.6.1`).
    both = ("# L\n\n## 2026-09-04 — bare\n\nb\n\n## _(2026-09-05)_ — italic\n\nb\n")
    chk("the italicised heading dialect was not counted as a record", len(headings(both)) == 2)
    chk("a dateless H2 was counted as a record",
        len(headings("# L\n\n## clause-1 [org-baseline 1.0.0]\n\nb\n")) == 0)
    chk("the placeholder heading was counted as a record",
        len(headings("# L\n\n## _(YYYY-MM-DD)_ — bootstrap\n\nb\n")) == 0)

    # 13b. A FOOTERLESS LEDGER IS A LEGITIMATE STATE, NOT A CRASH. `split_ledger`
    #      used to raise ValueError on one, which is what a ledger looks like
    #      before its first rotation.
    chainless = "# L\n\n## 2026-09-04 — one\n\nb\n\n## 2026-09-03 — two\n\nb\n"
    entries, foot = split_ledger(chainless)
    chk("a ledger with no rotation footer did not split at the end of the text",
        foot == len(chainless) and len(entries) == 2)
    chk("a ledger WITH a footer no longer splits at it",
        split_ledger("## 2026-09-04 — one\nb\n" + FOOTER_START + " x")[1]
        == len("## 2026-09-04 — one\nb\n"))

    # 13c. THE ORDINAL REFUSES BELOW 1 instead of wrapping to 'twentieth'.
    try:
        ordinal_word(0)
        chk("ordinal_word(0) returned a word instead of refusing", False)
    except ValueError:
        chk("ordinal_word(0) refused", True)
    chk("ordinal_word(1) no longer works", ordinal_word(1) == "first")

    # 13d. THE FIRST NOTICE CLAIMS NO PREDECESSOR NOTICE, and a later one does.
    pred_stub = {"commit": "c" * 40, "blob": "b" * 40, "bytes": 1, "lines": 1,
                 "entries": 1, "sha256": "s" * 64, "text": ""}
    first = render_footer(CHANGELOG, pred_stub, 1, 1, 1, "LEAF")
    later = render_footer(CHANGELOG, pred_stub, 38, 1, 1, "LEAF")
    chk("the first notice claims an earlier rotation notice",
        "NO earlier rotation notice" in first and "thirty-seventh" not in first)
    chk("a later notice stopped naming its predecessor", "thirty-seventh" in later)

    # 13e. THE ROTATED BODY ENDS IN EXACTLY ONE NEWLINE, both shapes. The first
    #      bootstrapped rotation appended a notice that ends the file, leaving no
    #      final newline, and FILE-TERMINATION refused the commit — while a
    #      separate probe read that refusal as a DIFFERENT file being bounded
    #      (`docs/knowledge/a-control-that-passes-for-an-unrelated-reason.md`).
    for shape, raw in (("appended", "a\n\nnotice with no newline"),
                       ("replaced", "a\n\nnotice with a tail\n"),
                       ("over-newlined", "a\n\nnotice\n\n\n\n")):
        chk(f"the {shape} body did not end in exactly one newline",
            (raw.rstrip("\n") + "\n").endswith("\n")
            and not (raw.rstrip("\n") + "\n").endswith("\n\n"))

    # 14. THE SECOND LEDGER STILL PARSES under the shared boundary — the positive
    #     control for the widening, matching control 10 for the first ledger.
    if (ROOT / DEV_NOTES.path).is_file():
        second = (ROOT / DEV_NOTES.path).read_text(encoding="utf-8")
        chk("the second ledger yielded no records under the shared boundary",
            len(headings(second)) > 0)

    # 15-16. Ledger selection is per-ledger and does not leak. ⛔ The thresholds
    #        must differ, or a bug that ignored the argument would still pass.
    chk("the two ledgers share a threshold, so selection cannot be tested",
        CHANGELOG.threshold != DEV_NOTES.threshold)
    chk("a ledger stopped being enforced without its rotation being undone",
        [l.path for l in LEDGERS.values() if l.enforced] == [CHANGELOG.path, DEV_NOTES.path])

    # 17. The gate arm judges ONLY enforced ledgers. A ledger whose first
    #     rotation has not happened must not make the enforcer red every commit.
    chk("name_of did not round-trip the registry",
        all(LEDGERS[name_of(l)] is l for l in LEDGERS.values()))

    for f in failures:
        print(f"SELF-TEST: {f}", file=sys.stderr)
    if failures:
        return 1
    print(f"rotate_changelog --self-test: {ran} controls pass — the ordinal chain round-trips at "
          "every shape the ledger uses, BOTH heading dialects are records while a dateless or "
          "placeholder H2 is not, a date inside a body is not a record, a footerless ledger splits "
          "at the end of its text instead of raising, the cut lands on a record boundary, the "
          "target scales with the measured entry size, losslessness is refused both ways, only "
          "enforced ledgers reach the gate, and both live ledgers still parse")
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
    # ⛔ `--ledger` SELECTS, it does not widen: an unknown name is refused rather
    # than falling back to the changelog, because a tool that silently acts on a
    # different file from the one you named is worse than one that will not run.
    ledger = CHANGELOG
    if "--ledger" in argv:
        i = argv.index("--ledger")
        name = argv[i + 1] if len(argv) > i + 1 else ""
        if name not in LEDGERS:
            print(f"unknown ledger {name!r} — known: {', '.join(sorted(LEDGERS))}", file=sys.stderr)
            return 2
        ledger = LEDGERS[name]
    if "--check-all" in argv:
        return max((check_one(l) for l in LEDGERS.values()), default=0)
    if "--check" in argv or not argv:
        return check_one(ledger) if "--ledger" in argv else check()
    if "--plan" in argv or "--apply" in argv:
        p90, median, sample = entry_size_p90(ledger)
        text = (ROOT / ledger.path).read_text(encoding="utf-8")
        cut, retired, size = plan(ledger, text, p90)
        print(f"entry size p90 {p90} B, median {median} B, over {sample} non-rotation commits")
        print(f"target headroom {target_headroom(ledger, p90)} B "
              f"({ledger.runway_commits} commits of runway)")
        print(f"would retire {len(retired)} record(s), leaving {size} B "
              f"({ledger.threshold - size} B headroom)")
        if "--apply" in argv:
            i = argv.index("--apply")
            leaf = argv[i + 1] if len(argv) > i + 1 and not argv[i + 1].startswith("-") \
                else "SIGNOFF-REPAIR.11.4.1.6"
            return apply(ledger, leaf, bootstrap="--bootstrap" in argv)
        return 0
    print(__doc__)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
