#!/usr/bin/env python3
"""Census the numbers `DOCTRINE_ENFORCEMENT.md` restates, and what anchors them.

`DOCTRINE_ENFORCEMENT.md` says of itself that it "is the human-readable mirror of
the registry". A mirror nothing derives drifts: `BOOK-FRONTIER` and
`INDEX-FRONTIER` both exist because a SECOND copy of a fact, derived by nothing,
drifted from the first — the latter for 39 of 60 commits. `SIGNOFF-REPAIR.11.16`
opened on a third instance of that shape inside the mirror itself, and this is
its producer.

    python3 -B scripts/census_mirror_numbers.py            # the census
    python3 -B scripts/census_mirror_numbers.py --json
    python3 -B scripts/census_mirror_numbers.py --calibrate [N]
    python3 -B scripts/census_mirror_numbers.py --self-test

⛔ THERE IS NO `--check`, AND THAT IS THE RESULT RATHER THAN AN OMISSION.
`SIGNOFF-REPAIR.11.16` measured three candidate gates over this population and
REJECTED all three. `--calibrate` reproduces the third one's number on demand so
the rejection can be FALSIFIED by a later reader instead of believed:

  A. a numeral must sit in a sentence carrying a freezing citation
     → fires on 68 of 103 (66%) of the standing population
  B. A, minus four mechanical "structural" exclusions (§section, ordinal/name,
     compound noun, code span)
     → still fires on 51 of 103 (50%)
  C. B, scoped to the staged diff — the `GAP-CLAIM-CENSUS` precedent
     → over 200 commits, 14 add a numeral to a rationale cell and it would have
       BLOCKED 10 of them (71%). `--calibrate` is this one. ⚠️ Scoping A the same
       way blocks 12 of 14 (86%); the four exclusions buy 2 commits, which is the
       measurement that says the structural filter is not the missing piece.

`SIGNOFF-REPAIR.11.9`'s gate was rejected at 114 of 131 (87%) and `.11.15`'s at
13 of 14 (93%), both for teaching bypass; `POSITIONAL-REF` shipped at 9.5%.
71% is the rejected shape, not the accepted one. ⛔ And a per-numeral allowlist
is rejected by this repository's own measured statement, in `VISIBILITY-POLICY`'s
row: "an allowlist thirty entries long teaches bypass" — this population is 103.

⭐ WHAT THE MEASUREMENT FOUND INSTEAD, and it needs no rule: both numbers that had
actually drifted were a POPULATION SIZE the named instrument itself enumerates at
runtime (`SELF-TEST`'s 17-of-28, measured at 29-of-38; `FILE-TERMINATION`'s 642,
measured at 773). A number the instrument prints on every run cannot go stale, so
those two gained a `--census` flag and the mirror cites the command. The numbers
below are reported, never asserted — a later reader runs this against their own
tree and gets their own.

⛔ THE CLASSIFICATION THIS SCRIPT PRINTS IS MECHANICAL AND THE REAL ONE IS NOT.
"Frozen historical measurement" versus "claim about the current tree" is a
judgement over prose — `SIGNOFF-REPAIR.11.6`'s measured failure mode — and
`anchored`/`structural`/`bare` is only an approximation of it. A `bare` numeral
is a candidate for review, NOT a defect; every narrowing attempted above bought
accuracy with a growing list of prose special cases, which is the same failure
mode one level up.
"""

from __future__ import annotations

import collections
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MIRROR = ROOT / "DOCTRINE_ENFORCEMENT.md"

# A registry row: `| \`ID\` | <rationale> | <check> |`. Only these are the mirror;
# the prose around them is not restating a census.
ROW = re.compile(r"^\| `")

# A numeral, with digit-group separators (this corpus uses both "138,403" and
# "1 592"). The left guard keeps `REPAIR-0062`, `R-53-4` and `v1.2` out: a number
# welded to a word or an identifier is not a restated measurement.
NUM = re.compile(r"(?<![\w.:-])\d[\d,  ]*(?:\.\d+)?(?![\w])")

# What FREEZES a numeral: a citation pinning it to a moment. A leaf id, a work
# unit, a short sha, or the word "upstream" (this spine is vendored, and an
# upstream measurement is a fact about another tree).
ANCHOR = re.compile(r"SIGNOFF-REPAIR\.\d|REPAIR-\d{3,}|\b[0-9a-f]{7}\b|upstream", re.I)

# Sentence boundary. Deliberately crude and deliberately deterministic: the point
# is a reproducible partition, not natural-language parsing.
SENT = re.compile(r"(?<=[.!?])\s+")

# The four mechanical "structural" exclusions measured in `SIGNOFF-REPAIR.11.16`.
# Each says "this numeral is not a measurement at all".
ORDINAL = re.compile(r"\b(row|level|Phase|tranche|layer)\s$")
COMPOUND = re.compile(r"-(layer|line|digit)\b")


def split_row(line: str) -> tuple[str, str] | None:
    """Return (id, rationale) for a registry row, or None.

    ⛔ Splits on UNESCAPED pipes only. `TABLE-ARITY-RATCHET`'s row contains a
    literal `\\| — \\|` inside a code span, and a naive `line.split("|")` tears
    it into phantom cells — the same escaping rule that gate had backwards on
    its first implementation.
    """
    if not ROW.match(line):
        return None
    cells, buf, i = [], [], 0
    while i < len(line):
        c = line[i]
        if c == "\\" and i + 1 < len(line):
            buf.append(line[i : i + 2])
            i += 2
            continue
        if c == "|":
            cells.append("".join(buf))
            buf = []
            i += 1
            continue
        buf.append(c)
        i += 1
    cells.append("".join(buf))
    # cells[0] is the empty string before the leading pipe; the last is after the
    # trailing one. A registry row is | ID | rationale | check |.
    if len(cells) < 5:
        return None
    return cells[1].strip(), cells[2]


def sentence_spans(text: str) -> list[tuple[int, int]]:
    spans, start = [], 0
    for m in SENT.finditer(text):
        spans.append((start, m.start()))
        start = m.end()
    spans.append((start, len(text)))
    return spans


def structural(body: str, start: int, end: int) -> str | None:
    """Why this numeral is not a measurement, or None."""
    before = body[max(0, start - 12) : start]
    after = body[end : end + 14]
    if before.endswith("§"):
        return "section number"
    if ORDINAL.search(before):
        return "ordinal or name"
    if COMPOUND.match(after):
        return "compound noun"
    if body[:start].count("`") % 2 == 1:
        return "code span"
    return None


def classify(text: str) -> list[dict]:
    """Every numeral in every rationale cell, with its mechanical class."""
    out = []
    for line in text.splitlines():
        row = split_row(line)
        if row is None:
            continue
        ident, body = row
        spans = sentence_spans(body)
        for m in NUM.finditer(body):
            span = next(s for s in spans if s[0] <= m.start() < s[1])
            why = structural(body, m.start(), m.end())
            if ANCHOR.search(body[span[0] : span[1]]):
                cls, reason = "anchored", "its sentence cites a leaf, a commit or upstream"
            elif why:
                cls, reason = "structural", why
            else:
                cls, reason = "bare", "no freezing citation in its sentence"
            out.append(
                {
                    "doctrine": ident,
                    "value": m.group().strip(),
                    "class": cls,
                    "why": reason,
                    "context": body[max(0, m.start() - 55) : m.end() + 55].strip(),
                }
            )
    return out


def bare_counts(text: str) -> collections.Counter:
    """Multiset of bare numerals per row — the unit `--calibrate` differences."""
    c = collections.Counter()
    for item in classify(text):
        if item["class"] == "bare":
            c[(item["doctrine"], item["value"])] += 1
    return c


def calibrate(depth: int) -> dict:
    """Re-measure candidate C: the staged-diff-scoped rule, across history.

    `SIGNOFF-REPAIR.11.6`'s standing requirement — a rule is measured against its
    population before it is proposed — and `POSITIONAL-REF`'s method.
    """
    shas = subprocess.run(
        ["git", "rev-list", "--reverse", "-n", str(depth + 1), "HEAD"],
        capture_output=True,
        text=True,
        cwd=ROOT,
    ).stdout.split()

    def at(sha: str) -> str | None:
        r = subprocess.run(
            ["git", "show", f"{sha}:DOCTRINE_ENFORCEMENT.md"],
            capture_output=True,
            text=True,
            cwd=ROOT,
        )
        return r.stdout if r.returncode == 0 else None

    def all_counts(text: str) -> collections.Counter:
        """Every numeral in a rationale cell, bare or not — the denominator."""
        c = collections.Counter()
        for line in text.splitlines():
            row = split_row(line)
            if row:
                for m in NUM.finditer(row[1]):
                    c[(row[0], m.group().strip())] += 1
        return c

    blocked, blocked_at = 0, []
    adding, added = 0, 0
    prev = at(shas[0]) if shas else None
    for sha in shas[1:]:
        cur = at(sha)
        if cur is None or prev is None or cur == prev:
            prev = cur
            continue
        gained = all_counts(cur) - all_counts(prev)
        if gained:
            adding += 1
            added += sum(gained.values())
        new_bare = bare_counts(cur) - bare_counts(prev)
        if new_bare:
            blocked += 1
            rows = sorted({d for d, _ in new_bare})
            blocked_at.append({"commit": sha[:7], "rows": rows})
        prev = cur
    examined = max(len(shas) - 1, 0)
    return {
        "commits_examined": examined,
        "commits_adding_a_numeral": adding,
        "numerals_added": added,
        "commits_blocked": blocked,
        "blocked_pct_of_all": round(100 * blocked / examined, 1) if examined else 0.0,
        "blocked_pct_of_adding": round(100 * blocked / adding, 1) if adding else 0.0,
        "detail": blocked_at,
    }


def census() -> dict:
    items = classify(MIRROR.read_text())
    by_class = collections.Counter(i["class"] for i in items)
    rows = sorted({i["doctrine"] for i in items})
    bare = collections.defaultdict(list)
    for i in items:
        if i["class"] == "bare":
            bare[i["doctrine"]].append(i["value"])
    return {
        "population": len(items),
        "rows_carrying_a_numeral": len(rows),
        "anchored": by_class["anchored"],
        "structural": by_class["structural"],
        "bare": by_class["bare"],
        "bare_by_row": {k: v for k, v in sorted(bare.items())},
        "items": items,
    }


def self_test() -> int:
    fails = 0

    def check(name: str, got, want) -> None:
        nonlocal fails
        if got != want:
            print(f"MIRROR-NUMBERS self-test: {name}: got {got!r}, want {want!r}", file=sys.stderr)
            fails += 1

    def one(line: str) -> list[dict]:
        return classify(line)

    # 1 — a sentence citing a leaf freezes its numerals.
    r = one("| `X` | Measured at 7 sites (`SIGNOFF-REPAIR.11.6`). | `s.sh` |")
    check("anchored by a leaf id", [i["class"] for i in r], ["anchored"])

    # 2 — and one citing nothing does not. The negative half of arm 1: without
    # this, an implementation that called EVERYTHING anchored would pass.
    r = one("| `X` | Measured at 7 sites. | `s.sh` |")
    check("bare without a citation", [i["class"] for i in r], ["bare"])

    # 3 — a short sha freezes it too.
    r = one("| `X` | It drifted for 39 commits, starting at `1ebfebe`. | `s.sh` |")
    check("anchored by a sha", sorted({i["class"] for i in r}), ["anchored"])

    # 4 — a section number is not a measurement.
    r = one("| `X` | ROADMAP §9.8 publishes the model. | `s.sh` |")
    check("§ is structural", [i["class"] for i in r], ["structural"])

    # 5 — nor is an ordinal.
    r = one("| `X` | may not name a leaf row 1 does not. | `s.sh` |")
    check("ordinal is structural", [i["class"] for i in r], ["structural"])

    # 6 — nor a numeral inside a code span.
    r = one("| `X` | swept `migrations/0062` and missed one. | `s.sh` |")
    check("code span is structural", [i["class"] for i in r], ["structural"])

    # 7 — nor a compound noun.
    r = one("| `X` | the durable 4-layer invariants hold. | `s.sh` |")
    check("compound noun is structural", [i["class"] for i in r], ["structural"])

    # 8 — a number welded to an identifier is not a restated measurement at all.
    r = one("| `X` | forced the correction commit REPAIR-0062 here. | `s.sh` |")
    check("identifier is not a numeral", r, [])

    # 9 — AN ESCAPED PIPE MUST NOT TEAR THE ROW. `TABLE-ARITY-RATCHET` shipped
    # the opposite model and its own self-test asserted the false answer; this
    # arm fails against a naive `line.split("|")`, which would put the numeral
    # into a phantom cell and lose it.
    r = one(r"| `X` | 8 completed trees write `\| — \|` and point elsewhere. | `s.sh` |")
    check("escaped pipe keeps one cell", [i["value"] for i in r], ["8"])

    # 10 — prose outside the registry table is not the mirror.
    check("non-row ignored", classify("Measured: 29 such lines in the tree.\n"), [])

    # 11 — the real corpus is reachable and non-trivial, or this census is
    # vacuous the way `check_self_tests.sh`'s discovery arm guards against.
    real = census()
    if real["population"] < 20:
        print(
            f"MIRROR-NUMBERS self-test: the mirror yielded only {real['population']} numerals"
            " — expected the real population",
            file=sys.stderr,
        )
        fails += 1

    # 12 — every numeral lands in exactly one class.
    check(
        "classes partition the population",
        real["anchored"] + real["structural"] + real["bare"],
        real["population"],
    )

    if fails:
        return 1
    print(
        "MIRROR-NUMBERS self-test: 12 arms — citation/sha anchoring and its negative,"
        " four structural exclusions, an identifier, an escaped pipe, non-row prose,"
        f" and the live corpus ({real['population']} numerals) partitioned"
    )
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()

    if "--calibrate" in argv:
        i = argv.index("--calibrate")
        depth = int(argv[i + 1]) if len(argv) > i + 1 and argv[i + 1].isdigit() else 200
        c = calibrate(depth)
        if "--json" in argv:
            print(json.dumps(c, indent=2))
            return 0
        print(f"candidate C — the staged-diff-scoped rule, over {c['commits_examined']} commits")
        print(f"  commits adding a numeral to a rationale cell : {c['commits_adding_a_numeral']}"
              f"  ({c['numerals_added']} numerals)")
        print(f"  commits it would have BLOCKED               : {c['commits_blocked']}"
              f"  ({c['blocked_pct_of_all']}% of all, {c['blocked_pct_of_adding']}% of those adding)")
        print()
        print("  ⛔ REJECTED. `SIGNOFF-REPAIR.11.9`'s gate was rejected at 87% and `.11.15`'s at")
        print("     93%, both for teaching bypass; `POSITIONAL-REF` shipped at 9.5%.")
        for d in c["detail"][-10:]:
            print(f"     {d['commit']}  {', '.join(d['rows'])}")
        return 0

    c = census()
    if "--json" in argv:
        print(json.dumps(c, indent=2))
        return 0

    print(f"numerals in the registry's rationale cells : {c['population']}"
          f"  (across {c['rows_carrying_a_numeral']} rows)")
    print(f"  anchored — the sentence cites a moment   : {c['anchored']}")
    print(f"  structural — not a measurement at all    : {c['structural']}")
    print(f"  bare — neither                           : {c['bare']}")
    print()
    print("BARE numerals, by row. ⛔ A CANDIDATE FOR REVIEW, NOT A DEFECT COUNT: the")
    print("frozen/live distinction is a judgement over prose and this partition is a")
    print("mechanical approximation of it (`SIGNOFF-REPAIR.11.6`, `.11.16`).")
    for row, values in c["bare_by_row"].items():
        print(f"  {row:<26} {', '.join(values)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
