#!/usr/bin/env python3
"""Census the RELATIVE leaf references (`` `.2.3` ``) in the task trees.

`SIGNOFF-REPAIR.11.24.1.6`. A task tree cites its own leaves by a bare suffix —
`` `.11.24.1` `` inside `SIGNOFF-REPAIR.md` means `SIGNOFF-REPAIR.11.24.1`. That
is convenient and it is ambiguous the moment the reference means a leaf in a
DIFFERENT tree, because the trees share a numbering shape: `PHASE-8.4.4` writes
*the `.2.3` distribution-channel deferral* and means `PHASE-7.2.3`, while
`PHASE-8.2.3` exists, is `done`, and is the A2A facade.

⛔ A RESOLVER CANNOT NOTICE THAT, and it is the whole reason this instrument
exists. The wrong target EXISTS, so same-tree resolution returns a real leaf with
a real status; only a human reading the surrounding words catches it. What CAN be
measured mechanically is how many references are exposed to the mistake — a
reference whose suffix also names a leaf in another tree is one a reader has to
disambiguate from prose.

    python3 -B scripts/census_relative_leaf_refs.py            # the classified census
    python3 -B scripts/census_relative_leaf_refs.py --check     # the ratchet gate
    python3 -B scripts/census_relative_leaf_refs.py --json
    python3 -B scripts/census_relative_leaf_refs.py --detail <kind>
    python3 -B scripts/census_relative_leaf_refs.py --at <rev>
    python3 -B scripts/census_relative_leaf_refs.py --calibrate [N]
    python3 -B scripts/census_relative_leaf_refs.py --self-test

⭐ THE CLASSES ARE ABOUT RESOLVABILITY, NEVER ABOUT CORRECTNESS. This cannot
tell a correct cross-tree reference from an incorrect one; nothing can, short of
reading the sentence. The population it produces is the input to that reading —
`docs/CLAIM_VERIFICATION.md` leg 2: a search returning N hits gives a population,
not a count of defects.

  * `home`       — the suffix resolves in its own tree and in NO other. Reading
                   it as a same-tree reference is the only available reading, so
                   there is nothing to get wrong.
  * `shared`     — the suffix resolves in its own tree AND in at least one
                   other. The same-tree reading is a convention, not a fact, and
                   this is the class the known instance lives in.
  * `foreign`    — the suffix does NOT resolve in its own tree but does resolve
                   elsewhere. The reference cannot mean what it literally says,
                   so a reader is forced to look outward. ⭐ Not a defect: this
                   is the SAFE failure, because it cannot silently succeed.
  * `dangling`   — the suffix resolves nowhere. Either a typo or a leaf that was
                   renumbered; it is reported, not graded.

⛔ IT READS THE TREES ONLY. A `.N` in prose outside `docs/tasks/` has no
containing tree to resolve against, so the ambiguity this measures does not
arise there — and including those files would have produced a population whose
majority class was "not a leaf reference at all".

⛔ TWO ID FORMATS, AND BOTH ARE PARSED rather than one being assumed. The
`SIGNOFF-REPAIR` tree writes leaves as Markdown headings (`##### TREE.1.2 — …`);
every other tree writes them as `- ID: \\`TREE.1.2\\`` list items. Measured at the
time of writing: 426 heading-shaped and 280 ID-shaped. An instrument that knew
only one format would report the other tree's every reference as `dangling`,
which is how a census confirms whatever its parser happens to see.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TREES = "docs/tasks"

# A leaf id: an upper-case tree name, then one or more dotted numbers.
HEADING_ID = re.compile(r"^#{2,6}\s+([A-Z][A-Z0-9-]*(?:\.\d+)+)\s")
LIST_ID = re.compile(r"^\s*-?\s*ID:\s*`([A-Z][A-Z0-9-]*(?:\.\d+)+)`")
# A relative reference: a backticked bare suffix, `.2.3` / `.11.24.1.3.2`.
# ⛔ Backticks are REQUIRED. Without them `§12.6` and `0.1.0` and every ordinary
# decimal in prose parse as references, which is a population of noise.
REL_REF = re.compile(r"`(\.(?:\d+)(?:\.\d+)*)`")


def tracked_trees() -> list[str]:
    """The TREE files, and nothing else.

    ⛔ Two exclusions, both measured rather than assumed, and the first one is
    the instrument's own founding false positive. `git ls-files docs/tasks/*.md`
    matches RECURSIVELY, so the first run swept `docs/tasks/artifacts/` — 24
    evidence documents that define no leaves at all — and every relative
    reference in them came back `foreign`, because there was no home tree for it
    to resolve against. That is not a finding about the corpus; it is the
    instrument reporting the shape of its own glob. An artifact is written in
    the context of the leaf that produced it and has no tree of its own, so the
    ambiguity this measures does not arise there.

    Second: a depth-1 file that defines NO leaf is not a tree either
    (`TEMPLATE.md`). Its references have nothing to resolve against for the same
    reason, so it is excluded by the same rule rather than by name.
    """
    out = subprocess.run(
        ["git", "ls-files", f"{TREES}/*.md"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    depth_one = [
        p
        for p in out.stdout.splitlines()
        if p and str(Path(p).parent) == TREES
    ]
    return [p for p in depth_one if leaf_ids((ROOT / p).read_text(encoding="utf-8"))]


def tree_name(path: str) -> str:
    return Path(path).stem


def leaf_ids(text: str) -> set[str]:
    """Every leaf id a tree file DEFINES, in either of the two formats."""
    found: set[str] = set()
    for line in text.splitlines():
        m = HEADING_ID.match(line) or LIST_ID.match(line)
        if m:
            found.add(m.group(1))
    return found


def suffix_of(leaf_id: str) -> str:
    """`PHASE-8.4.4` -> `.4.4` — the SUFFIX dialect's form of a reference."""
    return leaf_id[leaf_id.index(".") :]


TRAILING_NUMBER = re.compile(r"^(.*)-(\d+)$")


def readings(tree: str, ref: str) -> list[str]:
    """Every leaf id a relative reference could be naming, in its own tree.

    🔴 **TWO DIALECTS COEXIST, and this function is where that was discovered
    rather than assumed.** The instrument's first version implemented one —
    `TREE` + `.a.b` — and reported `PHASE-1` as 114 foreign and 80 dangling out
    of 252 references, which is not a corpus with a few slips in it. Read: that
    tree writes `` `.1.6.1` `` and means `PHASE-1.6.1`, repeating its own phase
    number, while `PHASE-8` writes `` `.4.4` `` and means `PHASE-8.4.4`. Both
    are coherent; they are different.

    * `suffix`       — `TREE` + the reference. The dominant dialect.
    * `phase-repeat` — for a tree ending in `-N`, a reference whose first
      component is that same `N` may be the full id with the word `PHASE`
      dropped: `.1.6.1` in `PHASE-1` is `PHASE-1` + `.6.1`.

    A reference is reported against every reading that resolves, so a census
    that found only one dialect would have said so, and one that finds a
    reference satisfying BOTH reports it as genuinely ambiguous inside a single
    tree rather than quietly preferring one.
    """
    candidates = [("suffix", tree + ref)]
    match = TRAILING_NUMBER.match(tree)
    if match:
        head, number = match.groups()
        parts = ref.split(".")  # ['', 'a', 'b', ...]
        if len(parts) > 2 and parts[1] == number:
            candidates.append(("phase-repeat", f"{head}-{number}." + ".".join(parts[2:])))
    return candidates


def classify(paths: list[str], read) -> list[dict]:
    """The census. `read(path) -> str` is injected so the self-test needs no files."""
    defined: dict[str, set[str]] = {}
    for path in paths:
        defined[tree_name(path)] = leaf_ids(read(path))
    # suffix -> the trees that define a leaf with it
    by_suffix: dict[str, set[str]] = {}
    for tree, ids in defined.items():
        for leaf in ids:
            by_suffix.setdefault(suffix_of(leaf), set()).add(tree)

    rows: list[dict] = []
    for path in paths:
        tree = tree_name(path)
        for number, line in enumerate(read(path).splitlines(), start=1):
            for ref in REL_REF.findall(line):
                # Which readings of this reference name a leaf THIS tree defines.
                resolved = [
                    dialect
                    for dialect, leaf in readings(tree, ref)
                    if leaf in defined.get(tree, set())
                ]
                homes = by_suffix.get(ref, set())
                elsewhere = sorted(homes - {tree})
                if len(resolved) > 1:
                    kind = "internally-ambiguous"
                elif resolved and elsewhere:
                    kind = "shared"
                elif resolved:
                    kind = "home"
                elif elsewhere:
                    kind = "foreign"
                else:
                    kind = "dangling"
                rows.append(
                    {
                        "file": path,
                        "line": number,
                        "tree": tree,
                        "ref": ref,
                        "kind": kind,
                        "dialects": resolved,
                        "also_in": elsewhere,
                    }
                )
    return rows


def summarise(rows: list[dict]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in rows:
        counts[row["kind"]] = counts.get(row["kind"], 0) + 1
    return counts


def report(rows: list[dict]) -> int:
    counts = summarise(rows)
    print("relative leaf references in docs/tasks/ — by resolvability")
    print()
    for kind in ("home", "shared", "internally-ambiguous", "foreign", "dangling"):
        print(f"  {kind:<10} {counts.get(kind, 0):>5}")
    print(f"  {'total':<10} {len(rows):>5}")
    print()
    per_tree: dict[str, dict[str, int]] = {}
    for row in rows:
        per_tree.setdefault(row["tree"], {})
        per_tree[row["tree"]][row["kind"]] = (
            per_tree[row["tree"]].get(row["kind"], 0) + 1
        )
    print("  by tree (shared / foreign / dangling / total)")
    for tree in sorted(per_tree):
        c = per_tree[tree]
        total = sum(c.values())
        print(
            f"    {tree:<18} {c.get('shared', 0):>4} {c.get('foreign', 0):>4}"
            f" {c.get('dangling', 0):>4} {total:>6}"
        )
    print()
    # ⭐ THE DIALECT TABLE. The kind counts alone cannot say WHY a tree's
    # references fail to resolve; this can, and it is what turned 194 apparent
    # defects in PHASE-1 into one convention nobody wrote down.
    dialect_counts: dict[str, dict[str, int]] = {}
    for row in rows:
        seen = dialect_counts.setdefault(row["tree"], {})
        key = "+".join(row["dialects"]) if row["dialects"] else "unresolved"
        seen[key] = seen.get(key, 0) + 1
    print("  which reading resolves, by tree")
    for tree in sorted(dialect_counts):
        parts = ", ".join(
            f"{k} {v}" for k, v in sorted(dialect_counts[tree].items(), key=lambda kv: -kv[1])
        )
        print(f"    {tree:<18} {parts}")
    print()
    print("  --detail <kind> lists the rows; --json emits everything.")
    return 0


def detail(rows: list[dict], kind: str) -> int:
    chosen = [r for r in rows if r["kind"] == kind]
    if not chosen:
        print(f"no rows of kind {kind!r}")
        return 0
    for row in chosen:
        also = ", ".join(row["also_in"]) or "-"
        print(f"{row['file']}:{row['line']}  {row['ref']:<14} also in: {also}")
    print(f"\n{len(chosen)} row(s) of kind {kind!r}")
    return 0


# ── the two-sided self-test (the SELF-TEST doctrine) ────────────────────────
SELF_TEST_FILES = {
    "docs/tasks/ALPHA.md": (
        "## ALPHA.1 — the first\n"
        "## ALPHA.2.3 — the shared suffix\n"
        "## ALPHA.9 — the lonely one\n"
        "see `.2.3` and `.9` and `.7.7`\n"
    ),
    "docs/tasks/BETA.md": (
        "- ID: `BETA.2.3`\n"
        "- ID: `BETA.4`\n"
        "the `.2.3` deferral, and the `.9` one\n"
    ),
}


def self_test() -> int:
    rows = classify(sorted(SELF_TEST_FILES), lambda p: SELF_TEST_FILES[p])
    got = {(r["tree"], r["ref"]): r for r in rows}
    arms: list[tuple[str, bool]] = []

    # 1 ⭐ THE KNOWN INSTANCE'S SHAPE: a suffix defined in BOTH trees is `shared`
    #   in each of them — the reading is a convention, not a fact.
    arms.append(
        (
            "a suffix defined in two trees is `shared` from both sides",
            got[("ALPHA", ".2.3")]["kind"] == "shared"
            and got[("BETA", ".2.3")]["kind"] == "shared",
        )
    )
    # 2 and it NAMES the other tree, so the reader is told where to look
    arms.append(
        (
            "a shared row names the other tree",
            got[("ALPHA", ".2.3")]["also_in"] == ["BETA"],
        )
    )
    # 3 THE NEGATIVE SIDE: a suffix only its own tree defines is `home`
    arms.append(("a suffix unique to its tree is `home`", got[("ALPHA", ".9")]["kind"] == "home"))
    # 4 the SAME suffix, referenced from the tree that does NOT define it, is
    #   `foreign` — the SAFE failure, because it cannot silently succeed
    arms.append(
        ("the same suffix from a tree that lacks it is `foreign`", got[("BETA", ".9")]["kind"] == "foreign")
    )
    # 5 a suffix no tree defines is `dangling`, its own class rather than an ambiguity
    arms.append(("a suffix nothing defines is `dangling`", got[("ALPHA", ".7.7")]["kind"] == "dangling"))
    # 6 ⛔ BOTH ID FORMATS PARSE. `BETA` uses `- ID:` list items; if only headings
    #   were parsed, `.2.3` would be `foreign` from ALPHA and `dangling` from BETA.
    arms.append(("the `- ID:` format defines leaves too", "BETA.4" in leaf_ids(SELF_TEST_FILES["docs/tasks/BETA.md"])))
    # 7 ⛔ AND THE BACKTICKS ARE LOAD-BEARING: an unbacktick'd decimal in prose
    #   is not a reference. Without this the population is mostly section numbers.
    plain = classify(["x"], lambda _p: "## X.1 — t\nROADMAP 12.6 and version 0.1.0 and .2.3\n")
    arms.append(("an unbackticked decimal is not a reference", plain == []))
    # 8 and the same line WITH backticks is one, so arm 7 is not passing because
    #   the parser is simply broken
    ticked = classify(["x"], lambda _p: "## X.1 — t\nsee `.2.3`\n")
    arms.append(("the same line backticked IS a reference", len(ticked) == 1))

    # 9 ⛔ THE RATCHET FIRES on a rise in a watched class ...
    arms.append(
        (
            "the ratchet fires on a rise",
            [k for k, _, _ in ratchet({"dangling": 1}, {"dangling": 2})] == ["dangling"],
        )
    )
    # 10 ... and NOT on an equal count or a fall — the negative side, without
    #    which arm 9 passes against a predicate that fires on everything.
    arms.append(
        (
            "and not on an equal count or a fall",
            ratchet({"dangling": 2}, {"dangling": 2}) == []
            and ratchet({"dangling": 2}, {"dangling": 1}) == [],
        )
    )
    # 11 ⛔ `foreign` is DECLINED, priced at 3 rises in 30 commits against 1 —
    #    so a rise in it must NOT fire, and that is a decision worth guarding.
    arms.append(
        ("a rise in the declined `foreign` class does not fire", ratchet({"foreign": 1}, {"foreign": 9}) == []),
    )

    passed = sum(1 for _, ok in arms if ok)
    for label, ok in arms:
        print(f"  {'PASS' if ok else 'FAIL'}  {label}")
    print(f"census_relative_leaf_refs --self-test: {passed}/{len(arms)} controls pass")
    return 0 if passed == len(arms) else 1


def at_revision(rev: str) -> list[dict]:
    """The same census over a past revision, read from git rather than the tree."""
    listing = subprocess.run(
        ["git", "ls-tree", "-r", "--name-only", rev, "--", TREES],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.splitlines()
    depth_one = [p for p in listing if p.endswith(".md") and str(Path(p).parent) == TREES]
    cache: dict[str, str] = {}

    def read(path: str) -> str:
        if path not in cache:
            cache[path] = subprocess.run(
                ["git", "show", f"{rev}:{path}"],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=True,
            ).stdout
        return cache[path]

    trees = [p for p in depth_one if leaf_ids(read(p))]
    return classify(trees, read)


def calibrate(count: int) -> int:
    """Price a RATCHET over real history: how often would it have fired?

    ⛔ The bar is `SIGNOFF-REPAIR.11.5`'s: a gate people route around is a gate
    that lies. A ratchet that fires on ordinary work is one somebody will
    disable, so the number that decides it is *how many of the last N commits
    RAISED a class* — not how large the class is today.
    """
    revs = subprocess.run(
        ["git", "log", f"-{count + 1}", "--format=%h", "--", TREES],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split()
    revs.reverse()  # oldest first
    watched = ("dangling", "internally-ambiguous", "foreign")
    previous: dict[str, int] | None = None
    rises: dict[str, int] = {k: 0 for k in watched}
    print(f"calibrating over {len(revs)} revisions touching {TREES} (oldest first)")
    print(f"  {'rev':<10} " + " ".join(f"{k:>21}" for k in watched))
    for rev in revs:
        counts = summarise(at_revision(rev))
        marks = []
        for kind in watched:
            now = counts.get(kind, 0)
            was = previous.get(kind, 0) if previous else now
            if previous and now > was:
                rises[kind] += 1
                marks.append(f"{now:>18} ⬆{now - was:<2}")
            else:
                marks.append(f"{now:>21}")
        print(f"  {rev:<10} " + " ".join(marks))
        previous = counts
    print()
    span = max(len(revs) - 1, 1)
    for kind in watched:
        print(f"  {kind:<22} rose in {rises[kind]:>3} of {span} commits")
    return 0


WATCHED = ("dangling", "internally-ambiguous")


def ratchet(head: dict[str, int], now: dict[str, int]) -> list[tuple[str, int, int]]:
    """The predicate, separated from the git reads so it can be self-tested.

    ⛔ A FALL IS NOT A BREACH and never becomes the new floor by accident: this
    reports rises only, and the baseline is recomputed from `HEAD` every run
    rather than stored — a stored number is a second copy, and this repository
    has measured what happens to those.
    """
    return [
        (kind, head.get(kind, 0), now.get(kind, 0))
        for kind in WATCHED
        if now.get(kind, 0) > head.get(kind, 0)
    ]


def check() -> int:
    """The RATCHET: neither unresolvable class may RISE against `HEAD`.

    ⛔ **A ratchet rather than a floor, and the number is why.** 76 `dangling`
    and 126 `internally-ambiguous` references exist today; demanding zero would
    be a gate red on arrival, which is a gate somebody turns off
    (`SIGNOFF-REPAIR.11.5`). Calibrated over the 30 commits touching
    `docs/tasks/` before it was written: `dangling` rose in **1**,
    `internally-ambiguous` in **0**. A gate that fires once in thirty commits,
    each time on a genuinely unfollowable new reference, is affordable.

    ⛔ **`foreign` is DELIBERATELY NOT RATCHETED, and the number is the reason
    rather than a preference.** It rose in **3 of 30** — ten times the rate —
    because a cross-tree reference is ordinary, intended work. It is also the
    SAFE class: it cannot silently resolve to the wrong leaf, because it does
    not resolve in its own tree at all. Gating the safe class at ten times the
    cost is the trade this declines.

    ⚠️ **AND IT DOES NOT CATCH THE INSTANCE THAT OPENED THE LEAF**, said plainly
    rather than implied. `PHASE-8.4.4`'s `.2.3` is `shared` — it resolves in its
    own tree to a real leaf with a real status, and 2178 references are in that
    class. Nothing mechanical can separate a correct shared reference from an
    incorrect one; only the surrounding words can. The repair for that instance
    is the convention in `docs/TASK_TREE_README.md` and the reference itself.
    """
    head = summarise(at_revision("HEAD"))
    paths = tracked_trees()
    now = summarise(classify(paths, lambda p: (ROOT / p).read_text(encoding="utf-8")))
    breaches = ratchet(head, now)
    for kind, was, is_now in breaches:
        print(
            f"RELATIVE-LEAF-REF: RISE — `{kind}` relative leaf references "
            f"{was} -> {is_now} (+{is_now - was})",
            file=sys.stderr,
        )
    if breaches:
        print(
            "  A relative reference like `.2.3` resolves against its OWN tree. One that\n"
            "  resolves NOWHERE (`dangling`) or BOTH ways (`internally-ambiguous`) is one a\n"
            "  reader cannot follow. Write a reference to another tree IN FULL —\n"
            "  `PHASE-7.2.3` — as docs/TASK_TREE_README.md states.\n"
            "  List them: python3 -B scripts/census_relative_leaf_refs.py --detail dangling",
            file=sys.stderr,
        )
        return 1
    return 0


def main(argv: list[str]) -> int:
    mode = argv[1] if len(argv) > 1 else ""
    if mode == "--self-test":
        return self_test()
    paths = tracked_trees()
    rows = classify(paths, lambda p: (ROOT / p).read_text(encoding="utf-8"))
    if mode == "--json":
        print(json.dumps({"rows": rows, "counts": summarise(rows)}, indent=2))
        return 0
    if mode == "--check":
        return check()
    if mode == "--calibrate":
        return calibrate(int(argv[2]) if len(argv) > 2 else 40)
    if mode == "--at":
        if len(argv) < 3:
            print("--at needs a revision", file=sys.stderr)
            return 2
        rows = at_revision(argv[2])
        print(json.dumps(summarise(rows), indent=2))
        return 0
    if mode == "--detail":
        if len(argv) < 3:
            print("--detail needs a kind: home | shared | foreign | dangling", file=sys.stderr)
            return 2
        return detail(rows, argv[2])
    return report(rows)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
