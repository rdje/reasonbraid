#!/usr/bin/env python3
"""Census MEMORY.md's standing warnings against the durable memory layers.

`MEMORY.md` is the bounded layer-A resume pointer (`MEMORY_ARCHITECTURE.md`).
Its "Next action" bullet accumulates one standing warning per closing leaf and
sheds one whenever the byte cap is crossed, so the eviction order is whatever
the current author judges least costly — and a warning that leaves is not
recorded as having left.

This answers the question `SIGNOFF-REPAIR.3.4.3` posed and `.11.4.2.1` executed:
of the warnings standing in that bullet, how many are ALSO written down in a
layer that outlives the next eviction (their own task-tree leaf, a
`docs/knowledge/` record, or `TOOLBOX.md`)?

⛔ The measured answer, 2026-09-13, was **all of them** — 26 warnings, 0 existing
only in `MEMORY.md`. The worry the annotation was written against does not
reproduce, and this tool reports a POPULATION to classify rather than a defect
count (`SIGNOFF-REPAIR.11.4.5.2`: a search's N hits are a population, and
publishing N unclassified trades a false negative for a false positive).

What the census DID find is narrower: 13 of the 26 carry no pointer to where
their substance lives, so evicting one costs the next reader the path back, not
the fact. Run this before choosing what to evict: prefer shedding a warning
whose leaf the line names.

    python3 -B scripts/census_memory_warnings.py            # the census
    python3 -B scripts/census_memory_warnings.py --json      # machine-readable
    python3 -B scripts/census_memory_warnings.py --self-test # the instrument's own controls

⚠️ SEGMENTATION IS THE HARD PART, and it is why this is a tracked instrument
rather than a one-off grep. A warning legitimately contains internal markers:
"🔴 DELEGATION IS ONE HOP DEEP: ⛔ do not fix …" is ONE warning with two. A
naive split at every marker over-counts it as two. The rule below is therefore
explicit: a marker opens a NEW warning only when it begins the bullet or
follows a sentence terminator; a marker after ';', ':' or ',' continues the
warning it is inside. The `--self-test` fires on both shapes.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The markers this project writes its standing warnings with.
MARKERS = ("⛔", "⚠️", "🔴", "⭐")
_MARKER_ALT = "|".join(re.escape(m) for m in MARKERS)

# A marker opens a new warning when it starts the text or follows a sentence
# terminator. After ';', ':' or ',' it is a continuation of the same warning.
#
# ⚠️ The terminator may sit INSIDE markup — this project writes "**… clamp.**"
# and "*… in production.*" — so the closing `*`/`` ` `` characters are consumed
# between the terminator and the split. The first version of this regex did not,
# and UNDER-counted by merging three pairs of distinct warnings. Its own
# `--self-test` did not catch it, because the fixtures were written in the same
# idiom as the bug (`TOOLBOX.md`: a self-test written alongside the code shares
# its blind spots). What caught it was reading the instrument's output against
# the file it measured — a second route with no loyalty to the regex.
_SPLIT = re.compile(rf"(?<=[.!?])[*`\)]*\s+(?=(?:{_MARKER_ALT})\s)")
_FIRST = re.compile(rf"(?:{_MARKER_ALT})\s")

# Leaf ids as this project writes them in prose: `.4.2.3`, `.11.4.5.2`.
_LEAF = re.compile(r"`(\.[0-9]+(?:\.[0-9]+)*)`")
# The bolded phrase a warning leads with — derived from the warning itself, so
# the search key is the PRODUCER's words and not the reader's paraphrase.
_BOLD = re.compile(r"\*\*(.+?)\*\*", re.S)

# ⛔ THE BULLET HAS BEEN RENAMED ONCE AND WILL BE AGAIN, so both spellings are
# accepted rather than one (`SIGNOFF-REPAIR.11.20`). This instrument keyed on the
# ORIGINAL `- **Next action:**`; `SIGNOFF-REPAIR.11.4.2.3` conformed `MEMORY.md`
# to the `MEMORY_ARCHITECTURE.md` §6 template, which spells it `- next_action:`,
# and did not update the reader. From that commit this census REFUSED on every
# run — and nothing noticed, because nothing runs it and `SELF-TEST` only proves
# its own fixtures still pass. Its 16 controls were green throughout.
NEXT_ACTION_KEYS = ("- next_action:", "- **Next action:**")


def warning_text(memory: str) -> str:
    """The next-action bullet, whose body is the standing-warning list."""
    for line in memory.splitlines():
        for key in NEXT_ACTION_KEYS:
            if line.startswith(key):
                return line[len(key) :].strip()
    raise SystemExit(
        "census: MEMORY.md has no next-action bullet ("
        + " or ".join(repr(k) for k in NEXT_ACTION_KEYS)
        + ")"
    )


def segment(body: str) -> list[str]:
    """Split the bullet into warnings. Prose before the first marker is the
    resume pointer itself, not a warning, and is dropped."""
    first = _FIRST.search(body)
    if not first:
        return []
    return [p.strip() for p in _SPLIT.split(body[first.start() :]) if p.strip()]


def leaf_headings() -> set[str]:
    """Every leaf id that is an actual heading in a tracked task tree — the
    set a citation must resolve into to count as anchored."""
    found: set[str] = set()
    pattern = re.compile(r"^#{2,6}\s+([A-Z0-9-]+)((?:\.[0-9]+)+)\s")
    for tree in sorted((ROOT / "docs" / "tasks").glob("*.md")):
        for line in tree.read_text().splitlines():
            m = pattern.match(line)
            if m:
                found.add(m.group(2))
    return found


def durable_corpus() -> str:
    """The layers a warning can be anchored in besides its own leaf."""
    parts = [(ROOT / "TOOLBOX.md").read_text()]
    for record in sorted((ROOT / "docs" / "knowledge").glob("*.md")):
        parts.append(record.read_text())
    return "\n".join(parts)


def key_phrase(warning: str) -> str | None:
    """The warning's own leading bolded phrase, stripped of markup, as the
    search key. Derived from the producer (CLAIM_VERIFICATION leg 2)."""
    m = _BOLD.search(warning)
    if not m:
        return None
    phrase = re.sub(r"[`*]", "", m.group(1)).strip()
    return phrase or None


def classify(warnings: list[str], headings: set[str], corpus: str) -> list[dict]:
    rows = []
    for text in warnings:
        cited = sorted(set(_LEAF.findall(text)))
        resolving = [leaf for leaf in cited if leaf in headings]
        phrase = key_phrase(text)
        in_corpus = bool(phrase) and phrase in corpus
        if resolving:
            verdict = "anchored:leaf"
        elif in_corpus:
            verdict = "anchored:method"
        else:
            verdict = "UNCITED"
        rows.append(
            {
                "verdict": verdict,
                "cited": cited,
                "resolving": resolving,
                "phrase": phrase,
                "text": text,
            }
        )
    return rows


def run(as_json: bool) -> int:
    memory = (ROOT / "MEMORY.md").read_text()
    warnings = segment(warning_text(memory))
    rows = classify(warnings, leaf_headings(), durable_corpus())
    uncited = [r for r in rows if r["verdict"] == "UNCITED"]

    if as_json:
        print(json.dumps({"total": len(rows), "uncited": len(uncited), "rows": rows}, ensure_ascii=False, indent=2))
        return 0

    print(f"MEMORY.md standing warnings: {len(rows)}")
    print(f"  anchored to a task-tree leaf : {sum(1 for r in rows if r['verdict'] == 'anchored:leaf')}")
    print(f"  anchored to a method record  : {sum(1 for r in rows if r['verdict'] == 'anchored:method')}")
    print(f"  UNCITED (classify by hand)   : {len(uncited)}")
    print()
    for i, r in enumerate(rows, 1):
        mark = "?" if r["verdict"] == "UNCITED" else " "
        where = ",".join(r["resolving"]) if r["resolving"] else r["verdict"]
        print(f"{mark}{i:3d} [{where}] {r['text'][:110]}")
    print()
    print("⚠️ UNCITED is a POPULATION, not a defect count. It means this instrument found")
    print("   neither a resolving leaf citation nor a verbatim phrase match — NOT that the")
    print("   warning is undocumented. Classified by hand on 2026-09-13, all 13 of 13 were")
    print("   in fact recorded in a durable layer (`SIGNOFF-REPAIR.11.4.2.1`). What UNCITED")
    print("   actually marks is a warning that, once evicted, leaves the reader no pointer")
    print("   back to where its substance lives — a findability cost, not a loss of fact.")
    return 0


CONTROL_COUNT = 17


def self_test() -> int:
    """Two-sided controls over the part that can be silently wrong."""
    failures = []

    def check(name: str, got, want) -> None:
        if got != want:
            failures.append(f"{name}: got {got!r}, want {want!r}")

    # A marker after a sentence terminator OPENS a warning.
    check("sentence-split", len(segment("⛔ one thing. ⚠️ another thing.")), 2)
    # …and it still opens one when the terminator is INSIDE markup. This is the
    # shape the first instrument got wrong; it is a control because the failure
    # was silent and shrank the published number.
    check("terminator-inside-bold", len(segment("⛔ **one thing.** ⚠️ another.")), 2)
    check("terminator-inside-italic", len(segment("⛔ a *thing.* ⚠️ another.")), 2)
    check("terminator-inside-code", len(segment("⛔ a `thing.` ⚠️ another.")), 2)
    check("terminator-before-paren", len(segment("⛔ a thing.) ⚠️ another.")), 2)
    # ⛔ And markup WITHOUT a terminator still continues — the over-count guard
    # must survive the fix that removed the under-count.
    check("bold-without-terminator", len(segment("⛔ **a rule**; ⛔ its corollary.")), 1)
    # A marker after ':' or ';' CONTINUES one — the over-counting shape.
    check("colon-continues", len(segment("🔴 ONE HOP DEEP: ⛔ do not fix it.")), 1)
    check("semicolon-continues", len(segment("⛔ a rule (`.1.2`); ⛔ and its corollary.")), 1)
    # Prose before the first marker is the pointer, not a warning.
    check("preamble-dropped", len(segment("frontier row 1 is `.1.1`, then `.2`. ⛔ a warning.")), 1)
    check("no-markers", segment("frontier row 1 is `.1.1`."), [])
    # Citations are extracted, and only resolving ones anchor.
    rows = classify(["⛔ a rule (`.1.2`) and (`.9.9`)."], {".1.2"}, "")
    check("cited", rows[0]["cited"], [".1.2", ".9.9"])
    check("resolving", rows[0]["resolving"], [".1.2"])
    check("verdict-leaf", rows[0]["verdict"], "anchored:leaf")
    # No resolving citation, but the phrase is in a method record.
    rows = classify(["⚠️ **Rank a census by REACHABILITY** always."], set(), "… Rank a census by REACHABILITY …")
    check("verdict-method", rows[0]["verdict"], "anchored:method")
    # Neither: the class this census exists to find.
    rows = classify(["⚠️ **Some standing advice** with no citation."], set(), "")
    check("verdict-uncited", rows[0]["verdict"], "UNCITED")
    # A citation that resolves to NOTHING must not anchor — the silent-failure
    # shape, where a leaf id is renamed and the warning looks anchored anyway.
    rows = classify(["⛔ a rule (`.9.9`)."], {".1.2"}, "")
    check("dangling-citation", rows[0]["verdict"], "UNCITED")
    # The heading extractor reads real trees, and must find a known leaf.
    check("headings-find-known", ".4.2.3" in leaf_headings(), True)

    # ⭐ THE ARM THAT WOULD HAVE CAUGHT THIS INSTRUMENT DYING, and the reason it
    # is last: every control above is built from a FIXTURE, and a fixture written
    # beside the code shares its assumptions. This one reads the REAL `MEMORY.md`
    # and requires the key to still find its bullet. `SIGNOFF-REPAIR.11.4.2.3`
    # renamed that bullet while conforming the file to its template; this census
    # refused on every run from that commit, and the 16 fixture controls stayed
    # green throughout (`SIGNOFF-REPAIR.11.20`).
    # ⛔ It deliberately asserts almost nothing about the CONTENT — only that the
    # instrument can still locate what it is about. A control coupled to the live
    # file's wording would fail on every honest edit and be waived within a week.
    try:
        warning_text((ROOT / "MEMORY.md").read_text())
    except SystemExit as exc:
        failures.append(f"live-corpus: the real MEMORY.md is unreadable to this census ({exc})")

    if failures:
        for f in failures:
            print(f"SELF-TEST FAIL {f}", file=sys.stderr)
        return 1
    print(f"census_memory_warnings --self-test: {CONTROL_COUNT} controls pass")
    return 0


def main() -> int:
    args = sys.argv[1:]
    if "--self-test" in args:
        return self_test()
    return run("--json" in args)


if __name__ == "__main__":
    raise SystemExit(main())
