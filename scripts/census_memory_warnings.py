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

⛔ IT READS THE WHOLE `## Current state` BLOCK, NOT ONE BULLET
(`SIGNOFF-REPAIR.11.20.1`). It was written when every standing warning lived
inside the `Next action` bullet, and `MEMORY_ARCHITECTURE.md` §6's template —
which `SIGNOFF-REPAIR.11.4.2.3` conformed the file to — puts the named facts in
separate `key:` bullets, so warnings migrated into top-level bullets of their
own. Measured before the model was changed: the instrument reported **2**
warnings while **8** stood in the block, and **1,801 of 3,809 bytes (47%)** sat
in bullets it could not see. ⭐ Its headline was not wrong so much as answering a
question the template had stopped asking, which is worse — a reader at the byte
cap runs it, sees a small number, and concludes the file is mostly next-action.

⭐ THE BULLET NAMES COME FROM THE TEMPLATE, NOT FROM THIS FILE'S CONTENTS. A
model fitted to today's `MEMORY.md` is stale the next time the file is reshaped
— which is exactly how this instrument died the first time. `template_keys`
parses §6's fenced template, so the named-fact bullets are whatever the standard
says they are; every OTHER bullet in the block is warning text in its own right.
⚠️ An unkeyed bullet is NOT a defect — `SIGNOFF-REPAIR.11.20` deliberately moved
the standing lessons into pointer bullets of their own. It only has to be SEEN.

⭐ WEIGHT IS REPORTED BESIDE THE COUNT, because weight is what the cap is about.
A census that answers "how many" to a reader who is about to evict something for
bytes is answering the wrong question.

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


CURRENT_STATE = "## Current state"
ARCHITECTURE = "MEMORY_ARCHITECTURE.md"


def template_keys(architecture: str) -> tuple[str, ...]:
    """The named-fact bullets `MEMORY_ARCHITECTURE.md` §6's template defines.

    ⛔ Parsed from the STANDARD, never from `MEMORY.md`. A model derived from the
    file it measures agrees with that file by construction and says nothing —
    and re-deriving it from today's contents is how this census went stale the
    first time (`SIGNOFF-REPAIR.11.20`).
    """
    keys: list[str] = []
    in_fence = in_block = False
    for line in architecture.splitlines():
        if line.startswith("```"):
            if in_fence and keys:
                break
            in_fence, in_block = not in_fence, False
            continue
        if not in_fence:
            continue
        if line.startswith(CURRENT_STATE):
            in_block = True
            continue
        if in_block:
            m = re.match(r"- ([a-z_]+):", line)
            if m:
                keys.append(m.group(1))
            elif line.strip():
                in_block = False
    if not keys:
        raise SystemExit(
            f"census: {ARCHITECTURE} has no '{CURRENT_STATE}' template block to read the "
            "resume pointer's bullet names from"
        )
    return tuple(keys)


def state_bullets(memory: str) -> list[str]:
    """Every bullet in `MEMORY.md`'s Current state block, in order."""
    lines = memory.splitlines()
    for i, line in enumerate(lines):
        if line.startswith(CURRENT_STATE):
            break
    else:
        raise SystemExit(f"census: MEMORY.md has no '{CURRENT_STATE}' block")
    bullets = []
    for line in lines[i + 1:]:
        if line.startswith("#"):
            break
        if line.startswith("- "):
            bullets.append(line)
    return bullets


def bullet_body(bullet: str, keys: tuple[str, ...]) -> tuple[str, str]:
    """-> (the bullet's name, the text a warning can live in).

    A bullet the template names contributes the text AFTER its key; any other
    bullet is warning text in its own right and contributes all of it.
    """
    text = bullet[2:]
    for key in keys:
        if text.startswith(f"{key}:"):
            return key, text[len(key) + 1:].strip()
    return "(unkeyed)", text.strip()


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


def collect_warnings(memory: str, keys: tuple[str, ...]) -> list[tuple[str, str]]:
    """Every standing warning in the Current state block, with its bullet."""
    found: list[tuple[str, str]] = []
    for bullet in state_bullets(memory):
        name, body = bullet_body(bullet, keys)
        for text in segment(body):
            found.append((name, text))
    return found


def classify(warnings: list[str], headings: set[str], corpus: str,
             bullets: list[str] | None = None) -> list[dict]:
    rows = []
    for i, text in enumerate(warnings):
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
                "bullet": bullets[i] if bullets else None,
                "bytes": len(text.encode()),
                "cited": cited,
                "resolving": resolving,
                "phrase": phrase,
                "text": text,
            }
        )
    return rows


BYTE_CAP = 7168      # `MEMORY_POINTER_BYTE_CAP`'s default in MEMORY_ARCHITECTURE.md §9


def run(as_json: bool) -> int:
    memory = (ROOT / "MEMORY.md").read_text()
    keys = template_keys((ROOT / ARCHITECTURE).read_text())
    found = collect_warnings(memory, keys)
    bullets = [b for b, _ in found]
    rows = classify([w for _, w in found], leaf_headings(), durable_corpus(), bullets)
    uncited = [r for r in rows if r["verdict"] == "UNCITED"]
    total_bytes = len(memory.encode())
    warn_bytes = sum(r["bytes"] for r in rows)

    if as_json:
        print(json.dumps({"total": len(rows), "uncited": len(uncited),
                          "warning_bytes": warn_bytes, "file_bytes": total_bytes,
                          "byte_cap": BYTE_CAP, "template_keys": list(keys),
                          "rows": rows}, ensure_ascii=False, indent=2))
        return 0

    pct = round(100 * warn_bytes / total_bytes) if total_bytes else 0
    print(f"MEMORY.md standing warnings: {len(rows)}")
    print(f"  weight                       : {warn_bytes} of {total_bytes} bytes "
          f"({pct}% of the file; cap {BYTE_CAP}, headroom {BYTE_CAP - total_bytes})")
    print(f"  anchored to a task-tree leaf : {sum(1 for r in rows if r['verdict'] == 'anchored:leaf')}")
    print(f"  anchored to a method record  : {sum(1 for r in rows if r['verdict'] == 'anchored:method')}")
    print(f"  UNCITED (classify by hand)   : {len(uncited)}")
    print()
    print(f"  bullets read ({len(state_bullets(memory))} in the block; "
          f"template names {', '.join(keys)}):")
    per: dict[str, list[int]] = {}
    for r in rows:
        per.setdefault(r["bullet"], []).append(r["bytes"])
    for name, sizes in per.items():
        print(f"    {name:<24} {len(sizes)} warning(s), {sum(sizes)} bytes")
    print()
    for i, r in enumerate(rows, 1):
        mark = "?" if r["verdict"] == "UNCITED" else " "
        where = ",".join(r["resolving"]) if r["resolving"] else r["verdict"]
        print(f"{mark}{i:3d} [{where}] {r['bytes']:>5}B {r['bullet']:<22} {r['text'][:80]}")
    print()
    print("⚠️ UNCITED is a POPULATION, not a defect count. It means this instrument found")
    print("   neither a resolving leaf citation nor a verbatim phrase match — usually a")
    print("   findability cost (the substance is recorded; the pointer back is missing)")
    print("   rather than a loss of fact. Hand-classified twice, and the two runs do NOT")
    print("   agree, which is why the older sentence is gone: 2026-09-13, 13 of 13 were")
    print("   recorded somewhere durable (`SIGNOFF-REPAIR.11.4.2.1`); 2026-09-18, over the")
    print("   population this widened model can finally see, 3 of 4 were and ONE was not —")
    print("   an environment fact that existed only in this file. It is now")
    print("   `docs/decisions/2026-09-18_a-cargo-process-is-not-evidence-of-this-repo.md`")
    print("   (`SIGNOFF-REPAIR.11.20.1`).")
    print("⛔ So do not read UNCITED as 'already safe to evict'. Classify it; the class")
    print("   that costs a FACT rather than a pointer is rare, real, and was invisible to")
    print("   this census until it read the whole block.")
    return 0


def self_test() -> int:
    """Two-sided controls over the part that can be silently wrong."""
    failures: list[str] = []
    ran = 0

    # ⛔ THE TOTAL IS COUNTED, NOT WRITTEN DOWN. This printed a hardcoded `17`
    # beside its arms — correct on the day, and one added arm away from
    # publishing a false total. `TOOLBOX.md` names that exact shape: an
    # instrument's own banner is prose too.
    def check(name: str, got, want) -> None:
        nonlocal ran
        ran += 1
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

    # ---- `SIGNOFF-REPAIR.11.20.1`: the block, not the bullet ---------------
    # ⚠️ LABELLED, because two falsifications were needed and neither alone
    #    covers this section. Against the PRE-FIX model (`collect_warnings`
    #    returning the next-action bullet alone, restored in situ) exactly
    #    three arms go red by name — `unkeyed-bullet-counted`, `block-total`,
    #    `row-carries-bullet`. The rest pass both ways and are NOT thereby
    #    worthless: they exercise readers the pre-fix instrument did not have
    #    (`template_keys`, `bullet_body`, `state_bullets`), so there is no
    #    pre-fix behaviour for them to contradict.
    # ⛔ The negative arms have their own RED, against a DEGENERATE rule rather
    #    than the old one — `segment` returning every bullet whole. That fires
    #    `keyed-bullet-without-marker-is-silent` and `no-markers` by name, which
    #    is the control that stops "read the whole block" collapsing into
    #    "every bullet is a warning". A negative arm falsified only against the
    #    old code is a negative arm nothing has tested.
    KEYS = ("latest_commit", "next_action", "blockers")
    TEMPLATE = """intro prose
```markdown
# MEMORY — resume pointer

## How to resume
- Read `MEMORY_ARCHITECTURE.md`.

## Current state (OVERWRITE this block each update)
- latest_commit: `<hash>`
- next_action: <one concrete sentence>
- blockers: <none | what and who-owns>
```
trailing prose
"""
    # The bullet names are read from the TEMPLATE, which is the whole point:
    # a model derived from the measured file agrees with it by construction.
    check("template-keys", template_keys(TEMPLATE), KEYS)
    # …and the template's OWN prose bullets must not be mistaken for it.
    check("template-skips-how-to-resume", "Read" not in " ".join(template_keys(TEMPLATE)), True)

    MEM = """# MEMORY

## Current state (OVERWRITE this block each update)
- latest_commit: `abc1234` — a subject with no marker in it
- next_action: do the thing. ⛔ **and mind this.**
- ⚠️ **a standing lesson in a bullet of its own.**
- blockers: none
"""
    # 🔴 THE DEFECT, POSITIVE DIRECTION: a warning in its own top-level bullet
    #    is counted. Before this the census read `next_action` alone and
    #    reported 2 where the block held 8, missing 47% of the file by weight.
    found = collect_warnings(MEM, KEYS)
    check("unkeyed-bullet-counted", [b for b, _ in found].count("(unkeyed)"), 1)
    check("block-total", len(found), 2)
    # ⭐ THE NEGATIVE DIRECTION, and without it the rule degenerates into "every
    #    bullet is a warning": a template bullet carrying NO marker contributes
    #    nothing, however long it is.
    check("keyed-bullet-without-marker-is-silent",
          [b for b, _ in found].count("latest_commit"), 0)
    check("keyed-bullet-with-marker-contributes",
          [b for b, _ in found].count("next_action"), 1)
    # …and the key itself is never part of the warning text.
    check("key-stripped-from-body",
          all(not w.startswith("next_action") for _, w in found), True)
    # A bullet whose key the template does NOT name is warning text, not a fact.
    check("unknown-key-is-unkeyed",
          bullet_body("- surprise: ⛔ a warning.", KEYS)[0], "(unkeyed)")
    # Weight travels with the row, because weight is what the cap is about.
    rows = classify([w for _, w in found], set(), "", [b for b, _ in found])
    check("row-carries-bytes", all(r["bytes"] > 0 for r in rows) and bool(rows), True)
    # ⛔ INDEXED SAFELY, and that is not defensive clutter. Written as
    #    `rows[1]["bullet"]`, this arm raised IndexError against the pre-fix
    #    model and took the whole self-test down with it — a RED that names
    #    nothing, which is the one thing a falsification must not produce
    #    (`docs/knowledge/an-instrument-must-explain-its-own-failure.md`).
    check("row-carries-bullet", [r["bullet"] for r in rows][1:2], ["(unkeyed)"])
    # Both readers REFUSE rather than returning an empty census — the shape
    # `SIGNOFF-REPAIR.11.20` was killed by was a silent-on-paper failure.
    for name, fn, arg in (("state-bullets-refuses", state_bullets, "# MEMORY\n\nno block here\n"),
                          ("template-keys-refuses", template_keys, "no fenced template here")):
        try:
            fn(arg)
            check(name, "returned", "SystemExit")
        except SystemExit:
            check(name, "SystemExit", "SystemExit")

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
    # ⛔ It now covers all THREE readers, because the census gained two more
    #    ways to go blind: the template it derives its model from, and the
    #    block it reads. One live arm per thing that can silently stop matching.
    ran += 1
    try:
        memory = (ROOT / "MEMORY.md").read_text()
        warning_text(memory)
        keys = template_keys((ROOT / ARCHITECTURE).read_text())
        if not state_bullets(memory):
            failures.append("live-corpus: the real MEMORY.md's Current state block has no bullets")
        elif not collect_warnings(memory, keys):
            failures.append("live-corpus: the real MEMORY.md yields no warnings at all — "
                            "the segmenter or the block reader has stopped matching")
    except SystemExit as exc:
        failures.append(f"live-corpus: the real corpus is unreadable to this census ({exc})")

    if failures:
        for f in failures:
            print(f"SELF-TEST FAIL {f}", file=sys.stderr)
        return 1
    print(f"census_memory_warnings --self-test: {ran} controls pass")
    return 0


def main() -> int:
    args = sys.argv[1:]
    if "--self-test" in args:
        return self_test()
    return run("--json" in args)


if __name__ == "__main__":
    raise SystemExit(main())
