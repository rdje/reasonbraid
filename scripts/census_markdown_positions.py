#!/usr/bin/env python3
"""Census the positional references whose TARGET is tracked Markdown (`FILE.md:123`).

`SIGNOFF-REPAIR.11.4.2.6.5.1`. `scripts/census_positional_refs.py` censuses
positional references to SOURCE files and reports `unresolved=0` across 597
occurrences — a true statement about a population that does not contain Markdown
targets. A stale one sat in a tracked decision record the whole time:
`LIVE_STATUS.md:1719` was a table separator at its citing record's own commit and
is 928 lines off today.

⛔ **THIS INSTRUMENT REGISTERS NO GATE, AND THE DECLINE IS ON THE MEASUREMENT.**
A ratchet on the class *targets a prepend-only file* was priced over the 30 most
recent commits touching tracked Markdown: it would have fired on exactly **1**,
and that one is the commit that REPORTED the defect, whose references are
deliberate mentions of a pointer it is naming as broken. ⛔ **A count of mentions
cannot separate a use from a mention** — the same trap `scripts/check_self_tests.sh`
records in its own header, where registering a check put its flag's literal text
into the file discovery greps. A rule whose only firing in thirty commits is
against the report of the defect is a rule that punishes reporting.

⚠️ THE PREPEND-ONLY TEST IS HEADER-AGNOSTIC, and two earlier formulations were
not. `b.endswith(a)` returns 0 for every file here, because a fixed `# Title`
header means a prepended entry lands after it; stripping the title still returns 0
for `LIVE_STATUS.md`, whose insertion point is below a preamble AND a section
heading. What works is the longest common SUFFIX in lines, which asks the question
directly: was the old version preserved with something inserted above it?

    python3 -B scripts/census_markdown_positions.py                  the census
    python3 -B scripts/census_markdown_positions.py --measure-prepend re-derive the registry
    python3 -B scripts/census_markdown_positions.py --self-test      the instrument's controls
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REGISTRY = ".doctrine/prepend_only_documents.txt"

# Anchored exactly as `census_positional_refs.py` anchors its own, so the two
# instruments disagree about the TARGET SUFFIX and nothing else.
REF_RE = re.compile(r"(?<![A-Za-z0-9_./-])([A-Za-z_0-9./-]+\.md):(\d+)")

# The documents whose own contract makes them a live view rather than a record.
# Derived from `scripts/check_lockstep_claim.sh`'s LIVE_DOCS plus the book, which
# is the surface the director reads.
LIVE_EXTRA = {"ROADMAP.md", "TOOLBOX.md", "DOCTRINE_ENFORCEMENT.md", "COMMIT.md",
              "KNOWLEDGE_MAP.md"}


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True).stdout


def tracked_markdown() -> list[str]:
    return git("ls-files", "*.md").split()


def live_docs() -> set[str]:
    text = (ROOT / "scripts/check_lockstep_claim.sh").read_text()
    m = re.search(r"^LIVE_DOCS=\(([^)]*)\)", text, re.MULTILINE)
    return set(m.group(1).split() if m else []) | LIVE_EXTRA


def prepend_only() -> dict[str, str]:
    rows: dict[str, str] = {}
    path = ROOT / REGISTRY
    if not path.is_file():
        return rows
    for line in path.read_text().splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.split("\t")
        rows[parts[0]] = parts[1] if len(parts) > 1 else ""
    return rows


# ── pure verdicts ─────────────────────────────────────────────────────────────

def resolve(ref: str, tracked: list[str]) -> list[str]:
    """Every tracked file a reader could reach from what is WRITTEN.

    Suffix matching on a path boundary, the same question
    `census_positional_refs.py` asks: not *did the author mean this one* but
    *how many can a reader reach*. `README.md` reaches four files here.
    """
    return [p for p in tracked if p == ref or p.endswith("/" + ref)]


def resolution(ref: str, line: int, tracked: list[str], length_of) -> str:
    hits = resolve(ref, tracked)
    if not hits:
        return "unresolved"
    if len(hits) > 1:
        return "ambiguous"
    if line > length_of(hits[0]):
        return "past-end-of-file"
    return "in-range"


def longest_common_suffix(a: list[str], b: list[str]) -> int:
    n = 0
    while n < len(a) and n < len(b) and a[len(a) - 1 - n] == b[len(b) - 1 - n]:
        n += 1
    return n


def inserted_above(older: list[str], newer: list[str]) -> bool:
    """Was the older version preserved with content inserted ABOVE it?

    ⛔ Header-agnostic by construction. The two formulations this replaced each
    returned 0 for files that are plainly prepend-only, because both assumed they
    knew where the insertion point was.
    """
    if len(newer) <= len(older):
        return False
    return longest_common_suffix(older, newer) >= len(older) * 0.95


# ── modes ─────────────────────────────────────────────────────────────────────

def measure_prepend(window: int = 25) -> int:
    tracked = tracked_markdown()
    print(f"prepend-only measurement over the last {window} versions of each candidate")
    print(f"{'document':<44} {'pairs':>6} {'inserted-above':>15}  share")
    for path in tracked:
        revs = git("log", "--format=%H", "--", path).split()[:window]
        if len(revs) < 3:
            continue
        versions = []
        for rev in reversed(revs):
            p = subprocess.run(["git", "show", f"{rev}:{path}"],
                               cwd=ROOT, capture_output=True, text=True)
            if p.returncode == 0:
                versions.append(p.stdout.splitlines())
        pairs = above = 0
        for older, newer in zip(versions, versions[1:]):
            if len(newer) <= len(older):
                continue
            pairs += 1
            above += inserted_above(older, newer)
        if pairs and above / pairs >= 0.8:
            print(f"{path:<44} {pairs:>6} {above:>15}  {above / pairs:.0%}  PREPEND-ONLY")
    return 0


def census() -> int:
    tracked = tracked_markdown()
    live = live_docs()
    pre = prepend_only()
    lengths: dict[str, int] = {}

    def length_of(p: str) -> int:
        if p not in lengths:
            lengths[p] = len((ROOT / p).read_text(errors="replace").splitlines())
        return lengths[p]

    rows = []
    for path in tracked:
        text = (ROOT / path).read_text(errors="replace")
        for n, line in enumerate(text.splitlines(), 1):
            for m in REF_RE.finditer(line):
                target, target_line = m.group(1), int(m.group(2))
                rows.append({
                    "citer": path, "citer_line": n,
                    "target": target, "target_line": target_line,
                    "resolution": resolution(target, target_line, tracked, length_of),
                    "in_live": path in live or path.startswith("docs/book/"),
                    # ⛔ MATCH THE RESOLVED PATH, NEVER THE BASENAME. The registry
                    # gained `docs/tasks/artifacts/signoff_review/INDEX.md`, and a
                    # basename key would then have marked `docs/adr/INDEX.md` —
                    # a different file, not prepend-only — as broken by
                    # construction (`docs/knowledge/a-key-too-loose-returns-the-wrong-instance.md`).
                    "prepend_target": bool(
                        (hits := resolve(target, tracked)) and len(hits) == 1
                        and hits[0] in pre),
                })

    counts: dict[str, int] = {}
    for r in rows:
        counts[r["resolution"]] = counts.get(r["resolution"], 0) + 1
    targets = {r["target"] for r in rows}
    print(f"positional references with a MARKDOWN target, in tracked Markdown: "
          f"{len(rows)} occurrence(s), {len(targets)} distinct target(s)")
    for kind in ("in-range", "ambiguous", "past-end-of-file", "unresolved"):
        print(f"  {kind:<18} {counts.get(kind, 0)}")
    live_rows = [r for r in rows if r["in_live"]]
    pre_rows = [r for r in rows if r["prepend_target"]]
    both = [r for r in rows if r["in_live"] and r["prepend_target"]]
    print(f"  {'in a live doc or the book':<18} {len(live_rows)}")
    print(f"  {'target is prepend-only':<18} {len(pre_rows)}   "
          f"(broken by construction — the next entry invalidates them)")
    print(f"  {'BOTH':<18} {len(both)}")
    print()
    for r in rows:
        if r["resolution"] != "in-range" or (r["in_live"] and r["prepend_target"]):
            flag = r["resolution"] if r["resolution"] != "in-range" else "live+prepend-only"
            print(f"  {r['citer']}:{r['citer_line']} -> {r['target']}:{r['target_line']}  [{flag}]")
    return 0


# ── self-test ─────────────────────────────────────────────────────────────────

def self_test() -> int:
    controls = 0
    fails: list[str] = []

    def check(name: str, got, want) -> None:
        nonlocal controls
        controls += 1
        if got != want:
            fails.append(f"{name}: got {got!r} want {want!r}")

    check("a markdown target is extracted",
          REF_RE.findall("see `LIVE_STATUS.md:1719` for it"), [("LIVE_STATUS.md", "1719")])
    check("a SOURCE target is not this instrument's business",
          REF_RE.findall("see api.rs:149"), [])
    check("a bare filename with no line is not a reference",
          REF_RE.findall("see LIVE_STATUS.md for it"), [])
    check("a pathed markdown target is extracted",
          REF_RE.findall("docs/book/src/cli.md:12"), [("docs/book/src/cli.md", "12")])
    # ⛔ The left boundary: a longer path must not be truncated to its tail, which
    # is the defect `census_positional_refs.py` records having shipped once.
    check("a hyphenated path survives",
          REF_RE.findall("docs/tasks/artifacts/signoff_review/census-1.md:342"),
          [("docs/tasks/artifacts/signoff_review/census-1.md", "342")])

    tracked = ["README.md", "docs/README.md", "LIVE_STATUS.md"]
    check("an exact path resolves to one", resolve("LIVE_STATUS.md", tracked), ["LIVE_STATUS.md"])
    check("a bare basename reaching two files is ambiguous",
          len(resolve("README.md", tracked)), 2)
    check("an unknown name reaches nothing", resolve("nope.md", tracked), [])

    lengths = {"LIVE_STATUS.md": 10, "README.md": 5, "docs/README.md": 5}
    check("in range", resolution("LIVE_STATUS.md", 10, tracked, lengths.get), "in-range")
    check("past the end", resolution("LIVE_STATUS.md", 11, tracked, lengths.get),
          "past-end-of-file")
    check("ambiguous outranks the line check",
          resolution("README.md", 1, tracked, lengths.get), "ambiguous")
    check("unresolved", resolution("nope.md", 1, tracked, lengths.get), "unresolved")

    # The prepend test, in both directions and — critically — with a header, which
    # is what defeated the two formulations this one replaced.
    # ⛔ THE FIXTURE IS SIZED TO THE RULE. A four-line fixture cannot exercise a
    # "95% of the older version's lines" threshold: a two-line header is half of
    # it, and the first version of this arm failed for that reason alone rather
    # than because the code was wrong. On the real files a header is two lines in
    # thousands, so the rule holds at 90–100%; the control has to look like that.
    # Second instrument in two commits to need a fixture at realistic scale —
    # `docs/knowledge/a-self-test-cannot-be-tidier-than-the-real-input.md`.
    body = [f"line {i}" for i in range(60)]
    old = ["# T", ""] + body
    prepended = ["# T", "", "## new", "new body"] + body
    appended = old + ["## c", "body c"]
    check("a prepend under a header is detected", inserted_above(old, prepended), True)
    check("an append is not a prepend", inserted_above(old, appended), False)
    check("no growth is not a prepend", inserted_above(old, old), False)
    check("a rewrite is not a prepend",
          inserted_above(old, ["# T", "", "## z", "different", "and more"]), False)
    # ⛔ THE PREPEND KEY IS THE RESOLVED PATH, NOT THE BASENAME. Two tracked
    # INDEX.md files exist and only one is prepend-only.
    two_index = ["docs/adr/INDEX.md", "docs/tasks/artifacts/signoff_review/INDEX.md"]
    check("an ambiguous basename must not resolve to one file",
          len(resolve("INDEX.md", two_index)), 2)
    check("the exact path resolves to exactly its own file",
          resolve("docs/adr/INDEX.md", two_index), ["docs/adr/INDEX.md"])

    check("suffix length", longest_common_suffix(["a", "b", "c"], ["x", "b", "c"]), 2)
    check("no common suffix", longest_common_suffix(["a"], ["b"]), 0)

    if fails:
        for f in fails:
            print(f"census_markdown_positions self-test MISSED: {f}", file=sys.stderr)
        return 1
    print(f"census_markdown_positions --self-test: {controls} controls pass")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    if "--measure-prepend" in argv:
        return measure_prepend()
    return census()


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
