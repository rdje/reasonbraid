#!/usr/bin/env python3
"""RUNTIME-ROOT — a storage base that reaches generated project data derives the
repository root at RUNTIME, never from a compile-time macro and never from an
ambient environment variable (`SIGNOFF-REPAIR.11.2.1.2.3`).

⭐ §12 IS WHY, AND IT IS A STANDING POLICY RATHER THAN A PREFERENCE: the
repository root may be moved, even onto another filesystem, without the project
noticing. `env!("CARGO_MANIFEST_DIR")` is expanded by rustc, so a base built
from it names the checkout the artifact was BUILT in. Measured in the artifact
rather than argued from the macro: before `.11.2.1.2.1` the `journal_cli` test
binary carried the checkout's absolute path as a storage base, while `file!()`
in a sibling binary was RELATIVE — so those strings were storage, not debug
information.

🔴 THE AMBIENT HALF WAS WORSE, BECAUSE IT READ AS THE CAREFUL OPTION. Seventeen
sites reached for a cargo-provided temporary directory and fell back to the
compile-time root, which looks like handling the general case. That variable is
UNSET in this project's runs — measured three ways, two of them free of the
instrument that could have perturbed the result — so the fallback was the only
path ever taken, and a source comment stated the opposite as fact.

⛔ THE RULE IS ABOUT WHERE THE BASE LANDS, NOT ABOUT THE MACRO. A compile-time
manifest dir is CORRECT for a path that travels with its crate, and the census
found 13 such uses: three `schema/` goldens, a `bench/v1` corpus and eight reads
of `../../migrations`. A pattern matching the macro itself would condemn all
thirteen and teach bypass, which is the failure mode `.11.2.2` avoided by ruling
on the ARGUMENT rather than the call.

⭐ CALIBRATED OVER 42 COMMITS SAMPLED EVENLY ACROSS THE HISTORY. It would have
fired on 40 of them, on 24 rising to 46 sites, and fires on 0 today. Zero today
over a real historical population is the shape this repository registers
(`REASON-CODE-DOC`); the shape it has rejected three times is a rule that fires
on most of a correct population.

🔴 DECIDABILITY WAS THE OPEN LEG, AND THE CORPUS SETTLED IT. The first
calibration returned one UNRESOLVED at fifteen consecutive commits: a base BOUND
to a local and joined in a LATER statement, which a statement-local classifier
cannot follow. Following that one name to the end of the statement that uses it
takes UNRESOLVED to 0 at all 42. That is why this gate parses instead of
matching, and why an unclassifiable base FAILS CLOSED — zero today and zero
across the sample, so the strictness costs nothing, while an UNRESOLVED that
merely passed would be the silent evasion the rule exists to stop.

⛔ A COMMENT IS NOT A BREACH. An earlier census in this family flagged the very
file whose header explains the rule, which is the SELF-TEST family exactly
(`.11.2.2`). Comment lines are skipped here, and the repaired sites carry
comments naming the macro for the same reason.

Usage:
    python3 -B scripts/check_runtime_root.py             # the whole tree
    python3 -B scripts/check_runtime_root.py --calibrate  # over git history
    python3 -B scripts/check_runtime_root.py --self-test

Self-test: scripts/check_runtime_root.py --self-test
"""

from __future__ import annotations

import argparse
import pathlib
import re
import subprocess
import sys

# The compile-time root, and the ambient base that fell back to it.
COMPILE = re.compile(r'env!\s*\(\s*"CARGO_MANIFEST_DIR"\s*\)'
                     r'|concat!\s*\(\s*env!\s*\(\s*"CARGO_MANIFEST_DIR"')
AMBIENT = re.compile(r'var_os\s*\(\s*"CARGO_TARGET_TMPDIR"\s*\)')
# What the base is joined with: `.join("…")`, or `concat!(env!(…), "/…")`.
JOIN = re.compile(r'\.join\(\s*"([^"]+)"|,\s*"(/[^"]+)"\s*\)')
BIND = re.compile(r"let\s+(?:mut\s+)?([a-z_][a-z0-9_]*)\s*=")
# ⭐ A third shape the corpus insisted on: the destination is not joined at all,
# it is PUSHED, component by component, out of a `for` over an array literal —
# `for component in ["target", "browser-production-controls"] { root.push(…) }`.
# A follower that only knows `.join("…")` reports that as unresolved.
ARRAY = re.compile(r"for\s+\w+\s+in\s*\[([^\]]*)\]")
LITERAL = re.compile(r'"([^"]+)"')
# Generated project data. Anything else a base is joined with travels with the
# crate, which is what makes the compile-time form correct for it.
SCRATCH = "target"
# ⛔ A join that is nothing but `..` components names the REPOSITORY ROOT, not
# a destination: `env!("CARGO_MANIFEST_DIR").join("../..")` is the root, and
# what lands under it is decided by a LATER statement. Calling that SOURCE is a
# FALSE NEGATIVE, and this gate's own --calibrate is what found it — it reported
# 42 breaches at the pre-repair commit where the census counted 46, and the four
# missing were exactly this shape (`SIGNOFF-REPAIR.11.2.1.2.3`).
TRAVERSAL = re.compile(r"^(?:\.\.?/?)+$")

BREACH, SOURCE, UNRESOLVED = "scratch", "source", "unresolved"


def determining(joins: list[str]) -> list[str]:
    """The joins that actually say where the base LANDS."""
    return [join for join in joins if not TRAVERSAL.match(join.rstrip("/"))]


def classify(lines: list[str], index: int) -> tuple[str, list[str]]:
    """Classify the base on `lines[index]` (0-based) by where it lands."""
    # The statement ends at its first `;`, or — for a Rust TAIL expression,
    # which has none — at the line closing its block, or at a blank line.
    statement: list[str] = []
    for offset, line in enumerate(lines[index:index + 6]):
        if offset and (not line.strip() or line.strip() in ("}", "};")):
            break
        statement.append(line)
        if ";" in line:
            break

    joins = [a or b for a, b in JOIN.findall("\n".join(statement))]

    # A base can be BOUND and joined in a later statement — either because the
    # statement joins nothing at all, or because everything it joins is pure
    # traversal to the repository root. Follow the one name to the end of the
    # statement that uses it — bounded, and no dataflow.
    if not determining(joins):
        bound = BIND.search("\n".join(statement))
        if bound:
            # ⭐ A FOURTH shape, and the corpus insisted on this one too: the
            # root is RE-BOUND to an alias — `let mut parent = root;` — and the
            # pushes go onto the alias. Tracking only the original name reads
            # `publisher.rs` as unresolved and `browse/tests/support/mod.rs` as
            # SOURCE, which is a false negative rather than a fail-closed one.
            names, using, pending = {bound.group(1)}, False, []
            for line in lines[index + 1:index + 41]:
                if line.startswith("fn ") or line.startswith("}"):
                    break
                alias = re.match(r"\s*let\s+(?:mut\s+)?([a-z_][a-z0-9_]*)\s*="
                                 r"\s*([a-z_][a-z0-9_]*)\s*;", line)
                if alias and alias.group(2) in names:
                    names.add(alias.group(1))
                    continue
                # A `for X in ["a", "b"]` immediately above a push onto one of
                # these names supplies the components it is about to receive.
                array = ARRAY.search(line)
                if array:
                    pending = LITERAL.findall(array.group(1))
                if any(re.search(rf"\b{re.escape(n)}\b\s*\.\s*push\s*\(", line)
                       for n in names):
                    joins += LITERAL.findall(line) or pending
                    continue
                # The name is written once and the joins hang off it on
                # continuation lines, so read THROUGH the statement rather
                # than testing each line for the name.
                if using or any(re.search(rf"\b{re.escape(n)}\b", line)
                                for n in names):
                    using = True
                    joins += [a or b for a, b in JOIN.findall(line)]
                    if ";" in line or line.rstrip().endswith(")"):
                        using = False
                        if determining(joins):
                            break

    decided = determining(joins)
    if not decided:
        return UNRESOLVED, joins
    if any(SCRATCH in join for join in decided):
        return BREACH, joins
    return SOURCE, joins


def scan(text: str) -> list[tuple[int, str, str, list[str]]]:
    """Every storage base in one Rust source, with its verdict."""
    out = []
    lines = text.splitlines()
    for index, line in enumerate(lines):
        # ⛔ A comment naming the macro is DOCUMENTATION of this rule, not a
        # breach of it. The repaired sites all carry one.
        if line.lstrip().startswith("//"):
            continue
        ambient, compile_time = AMBIENT.search(line), COMPILE.search(line)
        if not (ambient or compile_time):
            continue
        verdict, joins = classify(lines, index)
        if ambient:
            # An ambient base is a breach wherever it lands: §13 says derive
            # the path from the current root, never accept one handed in.
            verdict = BREACH
        out.append((index + 1, "ambient" if ambient else "compile", verdict, joins))
    return out


def tracked_sources(root: pathlib.Path) -> list[str]:
    listing = subprocess.run(["git", "ls-files", "-z", "--", "*.rs"],
                             cwd=root, capture_output=True, text=True, check=True)
    return [path for path in listing.stdout.split("\0") if path]


def survey(root: pathlib.Path) -> tuple[list[str], list[str], int, int]:
    breaches, unresolved, sources, total = [], [], 0, 0
    for relative in tracked_sources(root):
        try:
            text = (root / relative).read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for line, kind, verdict, joins in scan(text):
            total += 1
            where = " -> ".join(joins) if joins else "(no destination found)"
            if verdict == BREACH:
                breaches.append(f"{relative}:{line} — a {kind} storage base "
                                f"reaching generated data: {where}")
            elif verdict == UNRESOLVED:
                unresolved.append(f"{relative}:{line} — a {kind} base whose "
                                  f"destination could not be determined")
            else:
                sources += 1
    return breaches, unresolved, sources, total


SELF_TEST_CASES: list[tuple[str, str, str]] = [
    ("the ambient base, with its compile-time fallback", BREACH, '''
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
'''),
    ("a compile-time root joined straight onto target/", BREACH, '''
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/policy-publish-tests");
'''),
    ("a compile-time root BOUND, then joined in a later statement", BREACH, '''
fn target_tmp(name: &str) -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .join("../../target")
        .join(format!("rb-cli-e2e-{name}"))
}
'''),
    ("a tracked corpus that travels with the crate", SOURCE, '''
fn default_corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bench/v1")
}
'''),
    ("the migrations directory", SOURCE, '''
    let migrator =
        Migrator::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"))
            .await
            .unwrap();
'''),
    ("a concat! golden path", SOURCE, '''
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/schema");
'''),
    ("the repaired form, which names neither", None, '''
    let base = reasonbraid_core::repository_root()
        .expect("the tests run inside the repository")
        .join("target");
'''),
    ("a COMMENT naming the macro is documentation, not a breach", None, '''
    // The base follows the PROCESS, not the build: env!("CARGO_MANIFEST_DIR")
    // would bake one checkout's absolute path into the binary.
    let base = reasonbraid_core::repository_root().unwrap().join("target");
'''),
    ("a root reached by pure traversal, then joined with target later", BREACH, '''
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let base = root.join("target/cli-state-controls");
        std::fs::create_dir_all(&base).unwrap();
    }
'''),
    ("a root whose destination is PUSHED from a for-loop array literal", BREACH, '''
    fn root() -> PathBuf {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let device = std::fs::metadata(&root).unwrap().dev();
        for component in ["target", "browser-production-controls"] {
            root.push(component);
        }
        root
    }
'''),
    ("a root RE-BOUND to an alias, then pushed onto", BREACH, '''
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root exists");
        let device = std::fs::metadata(&root).unwrap().dev();
        let mut parent = root;
        for component in ["target", "publisher-tests"] {
            parent.push(component);
        }
    }
'''),
    ("a compile-time root with no destination at all fails CLOSED", UNRESOLVED, '''
fn opaque() -> PathBuf {
    let anchor = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    anchor
}
'''),
]


def self_test() -> int:
    checked = 0
    for name, expected, source in SELF_TEST_CASES:
        found = scan(source)
        if expected is None:
            if found:
                print(f"SELF-TEST FAILED: {name!r} should produce no finding, "
                      f"got {found}", file=sys.stderr)
                return 1
            checked += 1
            continue
        if len(found) < 1:
            print(f"SELF-TEST FAILED: {name!r} produced no finding at all",
                  file=sys.stderr)
            return 1
        verdicts = {verdict for _, _, verdict, _ in found}
        if expected not in verdicts:
            print(f"SELF-TEST FAILED: {name!r} expected {expected}, "
                  f"got {sorted(verdicts)}", file=sys.stderr)
            return 1
        checked += len(found)

    # ⭐ The POSITIVE CONTROL for the binding follower: without it, case 3 is
    # unresolved rather than a breach — which is how fifteen consecutive
    # commits in this repository's history looked to the first version.
    bound_case = SELF_TEST_CASES[2][2].splitlines()
    statement_only = [line for line in bound_case]
    index = next(i for i, line in enumerate(statement_only) if COMPILE.search(line))
    verdict, _ = classify(statement_only[:index + 1], index)
    if verdict != UNRESOLVED:
        print("SELF-TEST FAILED: the binding case must be UNRESOLVED without "
              "the lines that follow it, or the follower is not what resolves it",
              file=sys.stderr)
        return 1
    checked += 1

    print(f"RUNTIME-ROOT self-test: {checked} classifications verified — the "
          "ambient base, a direct compile-time scratch join, a base bound then "
          "joined later, a root reached by PURE TRAVERSAL and joined with "
          "target afterwards, a destination PUSHED from a for-loop array "
          "literal, a root RE-BOUND to an alias before being pushed onto — the "
          "three false negatives this gate's own --calibrate found, by "
          "disagreeing with the census that opened the leaf — three source "
          "reads that must NOT trip, the repaired form, a comment naming the "
          "macro, a fail-closed unresolved base, and the positive control "
          "proving the binding follower is what resolves the bound case")
    return 0


def calibrate(root: pathlib.Path, samples: int) -> int:
    count = int(subprocess.run(["git", "rev-list", "--count", "HEAD"], cwd=root,
                               capture_output=True, text=True, check=True).stdout)
    step = max(1, count // samples)
    revisions = subprocess.run(["git", "rev-list", "--reverse", "HEAD"], cwd=root,
                               capture_output=True, text=True, check=True).stdout.split()
    print(f"RUNTIME-ROOT calibration over {count} commits, every {step}th:")
    print(f"  {'commit':<10} {'date':<12} {'bases':>6} {'breach':>7} "
          f"{'source':>7} {'unres':>6}")
    fired = clean = unresolved_at = 0
    sampled = revisions[::step]
    for revision in sampled:
        blobs = subprocess.run(["git", "ls-tree", "-r", "--name-only", revision],
                               cwd=root, capture_output=True, text=True).stdout.split()
        breach = source = unres = total = 0
        for relative in (b for b in blobs if b.endswith(".rs")):
            text = subprocess.run(["git", "show", f"{revision}:{relative}"], cwd=root,
                                  capture_output=True, text=True).stdout
            for _, _, verdict, _ in scan(text):
                total += 1
                breach += verdict == BREACH
                source += verdict == SOURCE
                unres += verdict == UNRESOLVED
        date = subprocess.run(["git", "log", "-1", "--format=%ad", "--date=short",
                               revision], cwd=root, capture_output=True,
                              text=True).stdout.strip()
        print(f"  {revision[:9]:<10} {date:<12} {total:>6} {breach:>7} "
              f"{source:>7} {unres:>6}")
        fired += breach > 0
        clean += breach == 0
        unresolved_at += unres > 0
    print(f"\n  sampled {len(sampled)} commits — would FIRE on {fired}, "
          f"clean on {clean}, UNRESOLVED at {unresolved_at}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true",
                        help="verify the classifier against known shapes")
    parser.add_argument("--calibrate", action="store_true",
                        help="what this rule would have fired on, over git history")
    parser.add_argument("--samples", type=int, default=40,
                        help="how many commits --calibrate samples (default 40)")
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    root = pathlib.Path(subprocess.run(["git", "rev-parse", "--show-toplevel"],
                                       capture_output=True, text=True,
                                       check=True).stdout.strip())
    if args.calibrate:
        return calibrate(root, args.samples)

    breaches, unresolved, sources, total = survey(root)
    if breaches or unresolved:
        for finding in breaches:
            print(f"RUNTIME-ROOT: {finding}", file=sys.stderr)
        for finding in unresolved:
            print(f"RUNTIME-ROOT: {finding} — state the destination in the same "
                  f"statement, or derive the root at runtime", file=sys.stderr)
        print("\n  §12: the repository root may be moved, even onto another "
              "filesystem, without the project noticing. A compile-time macro "
              "names the checkout the artifact was BUILT in, and an ambient "
              "variable names whatever the caller handed in.\n"
              "  Use reasonbraid_core::repository_root(), which walks the "
              "current directory's ancestors.", file=sys.stderr)
        return 1

    print(f"{total} storage base(s) in tracked Rust derive the repository root "
          f"at runtime or read tracked source that travels with the crate "
          f"({sources} source read(s), 0 reaching generated data)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
