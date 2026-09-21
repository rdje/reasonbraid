#!/usr/bin/env python3
"""Census the tracked references that point INTO generated fixture directories
(`SIGNOFF-REPAIR.11.2.1.3.1.1`).

⭐ WHY THIS EXISTS AS A TRACKED INSTRUMENT. `.11.2.1.3.1` published *57
references, six of them naming an individual entry, all six absent* — and
measured it with a shell loop typed into a terminal. The six held exactly when
re-derived. The 57 did not: it was correct at `69374f6` and is **85** a few
commits later, moved by the very commits that published it, and it carried no
revision. `docs/CLAIM_VERIFICATION.md` §3 leg 3 names that failure precisely —
*a number nothing re-derives goes stale silently*, and *replacing a wrong
unwatched number with a right unwatched number is not a fix*.

⛔ THE FAMILIES ARE DERIVED FROM THE PRODUCERS, never listed here. §2 of that
standard: *derive classifiers, shape lists and membership tests from the code
that emits the thing, never from a description of it.* A new fixture family
joins this census the moment a test creates one — a hand-kept list would be a
second copy that drifts, which is the defect this file was written about.

⛔ AND THE CORPUS IS PART OF THE POPULATION. This census once read tracked RUST
only, which is a scope typed into a docstring rather than derived, and it
therefore reported **15.9%** of the bytes: the largest fixture families here are
created by PYTHON, and `target/pg-tests` alone is 3.64x the whole population it
was publishing (`SIGNOFF-REPAIR.11.2.1.3.2.1.1`). Each corpus is now read with
its OWN language's patterns — see `CORPORA` — and never with another's.

⛔ AND THE UNIT IS PRINTED, not implied. *57* was a per-family line sum, which
double-counts a line naming two families; the distinct-line count happened to
equal it at that commit and does not in general. All four units are reported
together so a reader cannot pick the wrong one by accident.

⭐ WHAT THE CENSUS IS FOR. A reference into `target/` dangles by construction —
the directory is gitignored and regenerated. The question worth asking is not
how many references exist but whether any names an INDIVIDUAL entry, because
that is the citation `SIGNOFF-REPAIR.11.4.2.9` refused to break when it declined
to rotate a tree carrying 4,092 of them. Every individual is printed with
whether it still exists.

⛔ THIS IS A CENSUS, NOT A GATE. It exits 0 whatever it finds. An absent cited
fixture is not by itself a defect — the durable record is the tracked summary
beside it, and `docs/evidence/2026-09-07_benchmark-codex-run.md` shows the
honest form: name the artifact AND say where durability actually lives.

Usage:
    python3 -B scripts/census_fixture_citations.py
    python3 -B scripts/census_fixture_citations.py --at <commit>
    python3 -B scripts/census_fixture_citations.py --self-test
"""

from __future__ import annotations

import argparse
import pathlib
import re
import subprocess
import sys

# A fixture family is a first path component under `target/` that tracked Rust
# joins onto a repository root. Derived, never listed.
FAMILY_SOURCE = re.compile(r'\.join\(\s*"(?:\.\./\.\./)?target/([A-Za-z0-9_][A-Za-z0-9_.-]*)')
# A push-built family: `for component in ["target", "browser-lifetime-controls"]`.
FAMILY_PUSHED = re.compile(r'\[\s*"target"\s*,\s*"([A-Za-z0-9_][A-Za-z0-9_.-]*)"')
# ⭐ A TWO-STEP family, and the corpus is what insisted on it: a base joined with
# exactly "target" in one statement, and the family name in another —
# `let base = …join("target"); let parent = base.join("conformance-stubs");`.
# The first version of this instrument knew only the single-step form and
# therefore missed `conformance-stubs`, which is a family whose citation the
# hand-written census DID find. An instrument that misses what the thing it
# replaces caught is not ready to replace it.
FAMILY_TWO_STEP = re.compile(
    r'\.join\(\s*"target"\s*\)(?:.|\n){0,800}?\.join\(\s*"([A-Za-z0-9_][A-Za-z0-9_.-]*)"')
# ⭐ A family named to the SHARED GUARD rather than joined at the call site:
# `Fixture::create("journal-tests", name)`. `SIGNOFF-REPAIR.11.2.1.3.2.2` moved
# the `target/` join into `reasonbraid_core::fixture`, so fifty call sites stopped
# carrying a joinable literal in one commit — and this census would have reported
# the family as gone while 2,375 of its fixtures sat on disk. An instrument that
# derives from the producer is correct only while it knows every shape the
# producer has, and the producer just grew one.
FAMILY_GUARD = re.compile(r'Fixture::create\(\s*"([A-Za-z0-9_][A-Za-z0-9_.-]*)"')
# ⭐ THE PYTHON PRODUCER, and it holds the LARGEST families in this repository.
# `project_env.local_directory(root, "target/<family>")` is the one idiom every
# script creates through — it refuses symlinks and cross-volume escapes before
# creating, which is the §13 gate — so all ten Python families come out of it.
# ⛔ `SIGNOFF-REPAIR.11.2.1.3.2.1.1`: this census published *the families are read
# out of the `.join("target/…")` literals in tracked Rust* and therefore reported
# **15.9%** of the bytes. `target/pg-tests` alone is 3.64x the whole population it
# was reporting. Deriving membership perfectly from the wrong CORPUS is the same
# error as a hand-written list, wearing the instrument's authority.
# ⚠️ Only a FIRST path component is a family; `local_directory` is also called with
# two-level paths (`target/doctrine_scratch/task-acceptance`), and counting the
# child as a family would double-count its parent's bytes.
FAMILY_PYTHON = re.compile(
    r'local_directory\(\s*[A-Za-z_][A-Za-z0-9_.]*\s*,\s*"target/([A-Za-z0-9_][A-Za-z0-9_.-]*)')

# ⛔ EACH CORPUS IS MATCHED BY ITS OWN LANGUAGE'S PATTERNS AND ONLY ITS OWN, and
# that is a defence rather than tidiness. This file and `census_fixture_population.py`
# carry `target/widget-controls`, `target/gizmo-tests` and `target/pushed-controls`
# inside their own self-test corpora — Rust-shaped literals living in Python source.
# Running the Rust patterns over `.py` would invent three families out of the
# censuses' own text, which is the self-reference failure `check_self_tests.sh`
# recursed on and this family of instruments has already hit once, on a doc comment.
# ⭐ THE SHELL PRODUCER, a THIRD corpus — and its absence is why
# `SIGNOFF-REPAIR.11.2.1.3.2.1.2` exists. `.11.2.1.3.2.1.1` widened this census
# from Rust to Rust+Python and published the result as *the population*;
# `scripts/demo_two_host.sh:91` writes `$ROOT/target/demo/$RUN_ID` — 106,000 KiB
# of it — and shell was still unread. Widening a corpus by one language and
# calling it complete is the same defect one turn later.
FAMILY_SHELL = re.compile(
    r'(?:\$\{?(?:ROOT|REPO|root|repo)\}?"?/|(?<![\w./])")target/([A-Za-z0-9_][A-Za-z0-9_.-]*)')

CORPORA = {
    "*.rs": (FAMILY_SOURCE, FAMILY_PUSHED, FAMILY_TWO_STEP, FAMILY_GUARD),
    "*.py": (FAMILY_PYTHON,),
    "*.sh": (FAMILY_SHELL,),
}

# ⛔ CARGO'S OWN DIRECTORIES ARE NOT FIXTURE FAMILIES, and the boundary is drawn
# from CARGO'S published layout rather than from a list of ours. The shell
# corpus is what forced this to be explicit: scripts legitimately reference
# `$ROOT/target/debug/rb-server`, so a pattern matching `target/<name>` sees
# `debug` — and `target/debug` is 64 GiB of build cache. Without this the census
# reported 148,813,880 KiB.
#
# ⭐ The first half is DERIVED and needs no name at all: a cargo profile
# directory contains `.fingerprint`. The second half names the three fixed
# directories the Cargo Book's target-directory layout defines, which is another
# tool's contract rather than an inventory of this project's producers.
CARGO_LAYOUT = ("doc", "package", "tmp")


def is_cargo_owned(root: pathlib.Path, name: str) -> bool:
    return name in CARGO_LAYOUT or (root / "target" / name / ".fingerprint").is_dir()

# ⛔ AN INSTRUMENT IS NOT A PRODUCER, and this file is the proof: adding the
# Python corpus made the census read ITSELF and report `python-tests` and
# `python-nested` — the names in its own self-test text — as live families, 37
# becoming 39. That is the failure `scripts/check_self_tests.sh` recursed on,
# arriving here for the third time in this instrument family.
#
# ⭐ DERIVED, NOT LISTED. An instrument declares itself by IMPORTING these
# patterns, so the import is the test. A future census that reuses them is
# excluded the moment it is written, and one that does not reuse them is a
# producer like any other and stays in scope. Listing filenames here would be a
# second copy of exactly the kind this file exists to argue against.
INSTRUMENT = re.compile(r"^\s*(?:from|import)\s+census_fixture_citations\b", re.M)
SELF = "scripts/census_fixture_citations.py"
# A reference in prose, with whatever path follows the family name.
def reference(families: list[str]) -> re.Pattern[str]:
    alt = "|".join(re.escape(name) for name in families)
    return re.compile(rf"target/({alt})((?:/[A-Za-z0-9_.-]+)*)")

# A trailing sentence period is punctuation, not a path component.
def clean(path: str) -> str:
    return path.rstrip(".")

# A name ending in `-` is a PREFIX standing for many entries; `pub-N` and
# friends are placeholders. Neither cites an individual.
PLACEHOLDER = re.compile(r"(-|/[A-Za-z]+-[A-Z])$")


def git(args: list[str], root: pathlib.Path) -> str:
    return subprocess.run(["git", *args], cwd=root, capture_output=True,
                          text=True).stdout


def families_of(root: pathlib.Path, at: str | None) -> list[str]:
    names: set[str] = set()
    for glob, patterns in CORPORA.items():
        suffix = glob.lstrip("*")
        listing = (git(["ls-tree", "-r", "--name-only", at], root).split()
                   if at else git(["ls-files", "--", glob], root).split())
        for relative in (path for path in listing if path.endswith(suffix)):
            text = (git(["show", f"{at}:{relative}"], root) if at
                    else (root / relative).read_text(encoding="utf-8", errors="replace"))
            if relative == SELF or INSTRUMENT.search(text):
                continue  # an instrument's corpus is not a producer's code
            for pattern in patterns:
                names.update(pattern.findall(text))
    return sorted(n for n in names if not is_cargo_owned(root, n))


def census(root: pathlib.Path, at: str | None) -> dict:
    families = families_of(root, at)
    if not families:
        return {"families": [], "lines": 0, "occurrences": 0, "individuals": []}
    pattern = reference(families)

    listing = (git(["ls-tree", "-r", "--name-only", at], root).split()
               if at else git(["ls-files", "--", "*.md"], root).split())
    lines = occurrences = 0
    individuals: set[str] = set()
    per_family: dict[str, int] = {name: 0 for name in families}

    for relative in (path for path in listing if path.endswith(".md")):
        text = (git(["show", f"{at}:{relative}"], root) if at
                else (root / relative).read_text(encoding="utf-8", errors="replace"))
        for line in text.splitlines():
            found = pattern.findall(line)
            if not found:
                continue
            lines += 1
            occurrences += len(found)
            for family, tail in found:
                per_family[family] += 1
                if tail and not PLACEHOLDER.search(clean(tail)):
                    individuals.add(clean(f"target/{family}{tail}"))
    return {"families": families, "lines": lines, "occurrences": occurrences,
            "per_family": per_family, "individuals": sorted(individuals)}


def report(root: pathlib.Path, at: str | None) -> int:
    result = census(root, at)
    where = at or "the working tree"
    print(f"fixture-citation census at {where}")
    print(f"  fixture families DERIVED from the producers: {len(result['families'])}")
    print(f"  corpora read (each by ITS OWN patterns)    : "
          f"{', '.join(CORPORA)}")
    print(f"  tracked Markdown lines carrying a reference: {result['lines']}")
    print(f"  references (OCCURRENCES, not lines)        : {result['occurrences']}")
    print(f"  per-family line sum (double-counts a line naming two families): "
          f"{sum(result.get('per_family', {}).values())}")
    print(f"  references naming an INDIVIDUAL entry      : {len(result['individuals'])}")
    if result["individuals"]:
        print("\n  each individual, and whether it still exists:")
        for path in result["individuals"]:
            exists = (root / path).exists() if at is None else None
            state = "PRESENT" if exists else "ABSENT" if exists is False else "(not checked: --at)"
            print(f"    {state:<8} {path}")
        print("\n  ⛔ An absent citation is not by itself a defect: target/ is gitignored\n"
              "     and regenerated. What matters is whether the record naming it says so.")
    return 0


SELF_TEST_RUST = '''
    let dir = reasonbraid_core::repository_root().unwrap().join("target/widget-controls");
    let other = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/gizmo-tests");
    for component in ["target", "pushed-controls"] { root.push(component); }
    let base = reasonbraid_core::repository_root().unwrap().join("target");
    let parent = base.join("two-step-controls");
    let fixture = Fixture::create("guarded-controls", name).unwrap();
'''
SELF_TEST_MD = [
    "the family alone: target/widget-controls holds the fixtures",
    "an individual: target/gizmo-tests/case-01a0 is the one that failed",
    "a PREFIX is not an individual: target/widget-controls/case- names many",
    "a placeholder is not an individual: target/gizmo-tests/run-N",
    "two families on ONE line: target/widget-controls and target/gizmo-tests",
    "a trailing period is punctuation: target/pushed-controls/inventory.json.",
]


SELF_TEST_PYTHON = '''
    parent = project_env.local_directory(root, "target/python-tests")
    nested = local_directory(ROOT, "target/python-nested/one-case")
    read_only = ROOT / "target" / "debug" / "some-binary"
'''

SELF_TEST_SHELL = '''
WORK="$ROOT/target/shell-demo/$RUN_ID"
OTHER="${REPO}/target/shell-braced/case"
mkdir -p "$WORK"
BIN="$ROOT/target/debug/rb-server"
'''


def self_test() -> int:
    families = sorted(set(FAMILY_SOURCE.findall(SELF_TEST_RUST))
                      | set(FAMILY_PUSHED.findall(SELF_TEST_RUST))
                      | set(FAMILY_TWO_STEP.findall(SELF_TEST_RUST))
                      | set(FAMILY_GUARD.findall(SELF_TEST_RUST)))
    expected = ["gizmo-tests", "guarded-controls", "pushed-controls",
                "two-step-controls", "widget-controls"]

    # ⭐ The Python producer: a first path component only, and a path merely READ
    # beneath `target/` is not a family — which is why `target/debug`, at 64 GiB,
    # needs no deny-list.
    python = sorted(set(FAMILY_PYTHON.findall(SELF_TEST_PYTHON)))
    if python != ["python-nested", "python-tests"]:
        print(f"SELF-TEST FAILED: the Python producer derives {python}, expected "
              f"['python-nested', 'python-tests'] — a first component only, and "
              f"nothing for a path that is merely read", file=sys.stderr)
        return 1

    # ⛔ THE SELF-REFERENCE DEFENCE, asserted rather than trusted: this module's
    # own Rust corpus is Python source, and the Rust patterns must never be run
    # over it. If they were, this file would invent its own self-test families.
    shell = sorted(set(FAMILY_SHELL.findall(SELF_TEST_SHELL)))
    if shell != ["debug", "shell-braced", "shell-demo"]:
        print(f"SELF-TEST FAILED: the shell producer derives {shell}, expected "
              f"['debug', 'shell-braced', 'shell-demo'] — a root-anchored path, a "
              f"braced one, and `debug` proving this pattern does NOT judge what "
              f"cargo owns (the filesystem reconciliation does that)",
              file=sys.stderr)
        return 1

    leaked = set()
    for pattern in CORPORA["*.rs"]:
        leaked.update(pattern.findall(SELF_TEST_PYTHON))
    for pattern in CORPORA["*.py"]:
        leaked.update(pattern.findall(SELF_TEST_RUST))
    if leaked:
        print(f"SELF-TEST FAILED: a corpus was matched by another language's "
              f"patterns and produced {sorted(leaked)}", file=sys.stderr)
        return 1

    # ⛔ THE SELF-REFERENCE DEFENCE CHECKED AGAINST THE REAL TREE, not a model of
    # it. The corpus-isolation assertion above would have passed while this
    # census read ITS OWN self-test text and published `python-tests` as a live
    # family. Only running the real derivation catches that, so it is what runs.
    root = pathlib.Path(subprocess.run(["git", "rev-parse", "--show-toplevel"],
                                       capture_output=True, text=True,
                                       check=True).stdout.strip())
    live = set(families_of(root, None))
    invented = live & ({"python-tests", "python-nested"} | set(expected))
    if invented:
        print(f"SELF-TEST FAILED: the live census invented {sorted(invented)} out "
              f"of an instrument's own self-test text — an instrument declares "
              f"itself by importing these patterns and must be skipped",
              file=sys.stderr)
        return 1
    if families != expected:
        print(f"SELF-TEST FAILED: families derived from the producer = {families}, "
              f"expected {expected}", file=sys.stderr)
        return 1

    pattern = reference(families)
    lines = occurrences = 0
    individuals: set[str] = set()
    for line in SELF_TEST_MD:
        found = pattern.findall(line)
        if not found:
            continue
        lines += 1
        occurrences += len(found)
        for family, tail in found:
            if tail and not PLACEHOLDER.search(clean(tail)):
                individuals.add(clean(f"target/{family}{tail}"))

    checks = [
        ("every fixture line is seen", lines, 6),
        ("the two-family line counts TWICE as occurrences and ONCE as a line",
         occurrences, 7),
        ("a prefix and a placeholder are NOT individuals", len(individuals), 2),
    ]
    for name, got, want in checks:
        if got != want:
            print(f"SELF-TEST FAILED: {name} — got {got}, expected {want}",
                  file=sys.stderr)
            return 1
    if individuals != {"target/gizmo-tests/case-01a0",
                       "target/pushed-controls/inventory.json"}:
        print(f"SELF-TEST FAILED: the individuals are {sorted(individuals)}",
              file=sys.stderr)
        return 1

    print("fixture-citation census self-test: 5 families derived FROM THE PRODUCER "
          "(one joined directly, one via a compile-time root, one pushed from an "
          "array literal, one joined in TWO STEPS — the shape whose absence made "
          "this instrument's first version miss a family the hand-written census had "
          "found — and one named to the SHARED GUARD, which is how fifty call sites "
          "stopped carrying a joinable literal in a single commit), 2 more from the "
          "PYTHON producer that this census read NOTHING of until it was found "
          "reporting 15.9% of the bytes — a first path component only, and nothing "
          "for a path merely read beneath target/, which is why target/debug needs "
          "no deny-list — with each corpus proved to be matched by its own "
          "language's patterns and only its own, so this file cannot invent "
          "families out of its own self-test text, 6 reference lines, 7 "
          "occurrences — so the two-family line is proved to count once as a line "
          "and twice as an occurrence, which is the unit confusion this instrument "
          "exists to end — and 2 individuals, a prefix and a placeholder correctly "
          "excluded, a trailing sentence period correctly stripped")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--at", help="census a commit instead of the working tree")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    root = pathlib.Path(subprocess.run(["git", "rev-parse", "--show-toplevel"],
                                       capture_output=True, text=True,
                                       check=True).stdout.strip())
    return report(root, args.at)


if __name__ == "__main__":
    sys.exit(main())
