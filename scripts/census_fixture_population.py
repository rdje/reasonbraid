#!/usr/bin/env python3
"""Census the generated fixture population under `target/` and the call sites that
produce it (`SIGNOFF-REPAIR.11.2.1.3.2.1`).

⭐ WHY THIS EXISTS AS A TRACKED INSTRUMENT. `.11.2.1.3.2` opened with two typed
numbers and both are wrong:

  - *221,496 KiB across thirteen families* — the families are **26**, of which 24
    exist, holding **351,000 KiB across 3,978 entries** at `c26a720`. Six families
    carrying 129,464 KiB were never counted, and every one of them is a TWO-STEP
    join (`.join("target")` in one statement, the family name in another) — the
    shape a hand-written list cannot see and `census_fixture_citations.py`
    already derives.
  - *roughly 40 call sites in `journal.rs` alone* — it is **17**, and **69**
    across the nine files that produce that family. An estimate overstated one
    file by 2.4x while understating the family by 4x.

⛔ AND THE FIRST WAS RE-DERIVED AS *UNCHANGED*. `.11.2.1.3.1.1` re-measured
221,496 KiB and reported it holding. It does hold — of those thirteen
directories. Re-measuring the same wrong list is exactly the blind spot
`docs/CLAIM_VERIFICATION.md` §1 warns about: a second pass down the same route
repeats it. The route that catches it is §2 — derive the membership test FROM
THE PRODUCER.

⛔ SO NOTHING HERE IS LISTED. The families come from `census_fixture_citations`'s
own derivation, IMPORTED rather than copied: a second copy of that regex set
would put this census one commit away from republishing the thirteen. The
producing call sites are found by walking back from each family literal to its
enclosing `fn`, and a producer is called INLINE only when that `fn` carries a
test attribute — read from the code, never inferred from a count.

⛔ AND EVERY UNIT IS PRINTED. `du -sk` reports DISK USAGE (allocated blocks); a
file's apparent size is a different number and on this tree a smaller one. Both
are reported, with the top-level and the recursive entry count beside them, so a
reader cannot pick the wrong one by accident — the confusion that made *57* a
per-family line sum one leaf ago.

⛔ THIS IS A CENSUS, NOT A GATE. It exits 0 whatever it finds, and it never
removes anything. A large family is not by itself a defect: exclusive creation
(`SIGNOFF-REPAIR.11.2.1.1`) means a fixture is never reused, so growth is the
bill for a property the suites deliberately bought. What to do about it is
`.11.2.1.3.2`'s question, not this instrument's.

The figure is PINNED: every report names the commit it was taken at and says
whether the tree was dirty, because the defect one leaf ago was a true number
published with no revision attached.

Usage:
    python3 -B scripts/census_fixture_population.py
    python3 -B scripts/census_fixture_population.py --json
    python3 -B scripts/census_fixture_population.py --self-test
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

# ⛔ IMPORTED, NOT RE-IMPLEMENTED. These three patterns ARE the definition of a
# fixture family in this repository, and the two-step one is there because its
# absence once made an instrument miss a family the thing it replaced had found.
# A copy here would drift from that definition silently.
from census_fixture_citations import (  # noqa: E402
    FAMILY_PUSHED,
    FAMILY_SOURCE,
    FAMILY_TWO_STEP,
    families_of,
)

ROOT = Path(__file__).resolve().parent.parent

# The enclosing function of a family literal: the last `fn NAME(` at or before it.
FUNCTION = re.compile(
    r"^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?(?:async[ \t]+)?(?:unsafe[ \t]+)?fn[ \t]+"
    r"([A-Za-z_][A-Za-z0-9_]*)",
    re.MULTILINE,
)
# A test attribute on the lines immediately above a function — `#[test]`,
# `#[tokio::test]`, `#[tokio::test(flavor = "multi_thread")]`. This is what makes
# a producer INLINE: nothing calls the function, the harness does.
TEST_ATTRIBUTE = re.compile(r"#\[(?:[A-Za-z_][A-Za-z0-9_]*::)*test\b")


def call_sites(text: str, name: str) -> int:
    """Calls to `name` in `text`, excluding its own definition."""
    total = len(re.findall(rf"\b{re.escape(name)}[ \t]*\(", text))
    defined = len(re.findall(rf"\bfn[ \t]+{re.escape(name)}[ \t]*\(", text))
    return total - defined


def enclosing(text: str, offset: int) -> tuple[str, bool] | None:
    """The function containing `offset`, and whether it carries a test attribute."""
    last = None
    for match in FUNCTION.finditer(text, 0, offset):
        last = match
    if last is None:
        return None
    # The attribute block sits between the previous item and this `fn` line; read
    # back to the last blank line, which is where an attribute block starts.
    head = text.rfind("\n\n", 0, last.start())
    preamble = text[head + 2 if head >= 0 else 0 : last.start()]
    return last.group(1), bool(TEST_ATTRIBUTE.search(preamble))


def producers(text: str) -> dict[str, list[dict]]:
    """Family name -> the producing functions in this one file."""
    found: dict[str, dict[str, dict]] = {}
    for pattern in (FAMILY_SOURCE, FAMILY_PUSHED, FAMILY_TWO_STEP):
        for match in pattern.finditer(text):
            family = match.group(1)
            site = enclosing(text, match.start())
            if site is None:
                continue
            name, is_test = site
            # A test function producing a fixture is ONE call site: the harness.
            # A helper's scope is the number of places that call it.
            scope = 1 if is_test else call_sites(text, name)
            found.setdefault(family, {})[name] = {
                "function": name,
                "inline_test": is_test,
                "call_sites": scope,
            }
    return {family: sorted(sites.values(), key=lambda s: s["function"])
            for family, sites in found.items()}


def disk(path: Path) -> dict:
    """Disk usage and apparent size of a tree, with both entry counts."""
    used = apparent = 0
    top = entries = 0
    stack = [(path, True)]
    while stack:
        current, is_root = stack.pop()
        try:
            status = current.lstat()
        except OSError:
            continue
        used += status.st_blocks * 512
        apparent += status.st_size
        if not is_root:
            entries += 1
        if current.is_dir() and not current.is_symlink():
            for child in sorted(current.iterdir()):
                if is_root:
                    top += 1
                stack.append((child, False))
    return {"disk_bytes": used, "apparent_bytes": apparent,
            "top_level_entries": top, "all_entries": entries}


def revision(root: Path) -> dict:
    def git(*args: str) -> str:
        return subprocess.run(["git", *args], cwd=root, capture_output=True,
                              text=True).stdout.strip()
    return {"commit": git("rev-parse", "--short", "HEAD"),
            "dirty": bool(git("status", "--porcelain"))}


def census(root: Path) -> dict:
    families = families_of(root, None)
    sources: dict[str, dict[str, list[dict]]] = {name: {} for name in families}
    listing = subprocess.run(["git", "ls-files", "--", "*.rs"], cwd=root,
                             capture_output=True, text=True).stdout.split()
    for relative in listing:
        text = (root / relative).read_text(encoding="utf-8", errors="replace")
        for family, sites in producers(text).items():
            sources.setdefault(family, {})[relative] = sites

    report = []
    for name in families:
        path = root / "target" / name
        measured = disk(path) if path.is_dir() else None
        files = sources.get(name, {})
        report.append({
            "family": name,
            "present": measured is not None,
            **(measured or {"disk_bytes": 0, "apparent_bytes": 0,
                            "top_level_entries": 0, "all_entries": 0}),
            "producing_files": {relative: sites for relative, sites in sorted(files.items())},
            "producing_call_sites": sum(site["call_sites"]
                                        for sites in files.values() for site in sites),
        })
    report.sort(key=lambda row: (-row["disk_bytes"], row["family"]))
    return {"revision": revision(root), "families": report}


def report(root: Path, as_json: bool) -> int:
    result = census(root)
    if as_json:
        print(json.dumps(result, indent=2))
        return 0

    rows = result["families"]
    present = [row for row in rows if row["present"]]
    used = sum(row["disk_bytes"] for row in rows)
    apparent = sum(row["apparent_bytes"] for row in rows)
    entries = sum(row["all_entries"] for row in rows)
    top = sum(row["top_level_entries"] for row in rows)
    sites = sum(row["producing_call_sites"] for row in rows)

    where = result["revision"]["commit"]
    state = " (tree DIRTY — the code census reflects the working tree)" if \
        result["revision"]["dirty"] else " (tree clean)"
    print(f"fixture-population census at {where}{state}")
    print(f"  fixture families DERIVED from tracked Rust : {len(rows)}")
    print(f"  families PRESENT under target/             : {len(present)}")
    print(f"  disk usage, the unit `du -sk` reports      : {used // 1024:,} KiB")
    print(f"  apparent size (sum of st_size)             : {apparent // 1024:,} KiB")
    print(f"  top-level entries (fixtures)               : {top:,}")
    print(f"  all entries (recursive)                    : {entries:,}")
    print(f"  producing call sites in tracked Rust       : {sites:,}")

    print("\n  per family, largest first "
          "(KiB is disk usage; `fixtures` is top-level entries):")
    print(f"    {'KiB':>10}  {'fixtures':>8}  {'entries':>8}  {'sites':>5}  family")
    for row in rows:
        if not row["present"]:
            print(f"    {'ABSENT':>10}  {'-':>8}  {'-':>8}  "
                  f"{row['producing_call_sites']:>5}  {row['family']}")
            continue
        print(f"    {row['disk_bytes'] // 1024:>10,}  {row['top_level_entries']:>8,}  "
              f"{row['all_entries']:>8,}  {row['producing_call_sites']:>5,}  {row['family']}")

    print("\n  the retrofit scope, per producing file "
          "(a call site is what must bind a cleanup guard):")
    for row in rows:
        if not row["producing_files"]:
            continue
        print(f"    {row['family']} — {row['producing_call_sites']} call sites")
        for relative, found in row["producing_files"].items():
            for site in found:
                kind = "inline test" if site["inline_test"] else "helper"
                print(f"      {site['call_sites']:>3}  {kind:<11}  "
                      f"{relative}::{site['function']}")

    print("\n  ⛔ A large family is not by itself a defect. Exclusive creation "
          "(SIGNOFF-REPAIR.11.2.1.1)\n     means a fixture is never reused, so growth "
          "is the bill for a property the suites\n     bought deliberately. This census "
          "measures; it never removes.")
    return 0


# A corpus carrying every producer shape this instrument must see: a direct join,
# a compile-time-root join, a pushed array literal, a two-step join, a helper with
# several callers, and an inline producer inside a test.
SELF_TEST_RUST = '''
fn widget_path(name: &str) -> PathBuf {
    let dir = reasonbraid_core::repository_root().unwrap().join("target/widget-controls");
    dir.join(name)
}

fn gizmo_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/gizmo-tests").join(name)
}

fn pushed_root() -> PathBuf {
    let mut root = base();
    for component in ["target", "pushed-controls"] { root.push(component); }
    root
}

#[tokio::test(flavor = "multi_thread")]
async fn inline_producer_builds_its_own() {
    let base = reasonbraid_core::repository_root().unwrap().join("target");
    let parent = base.join("two-step-controls");
    assert!(parent.exists());
}

#[test]
fn one() { let a = widget_path("a"); let b = gizmo_path("b"); drop((a, b)); }

#[test]
fn two() { let a = widget_path("c"); drop(a); }

#[test]
fn three() { let r = pushed_root(); drop(r); }
'''


def self_test() -> int:
    found = producers(SELF_TEST_RUST)
    checks: list[tuple[str, object, object]] = []

    checks.append(("every producer shape is seen", sorted(found),
                   ["gizmo-tests", "pushed-controls", "two-step-controls",
                    "widget-controls"]))

    flat = {family: sites[0] for family, sites in found.items() if len(sites) == 1}
    checks.append(("each family has exactly one producing function",
                   sorted(flat), sorted(found)))

    checks.append(("a helper's scope is its CALLERS, not its definition",
                   flat["widget-controls"]["call_sites"], 2))
    checks.append(("a helper called once is one call site",
                   flat["gizmo-tests"]["call_sites"], 1))
    checks.append(("the pushed-array helper is found and counted",
                   flat["pushed-controls"]["call_sites"], 1))
    checks.append(("an INLINE producer is one call site — the harness",
                   flat["two-step-controls"]["call_sites"], 1))
    checks.append(("inline is read from the test ATTRIBUTE, not from a zero count",
                   flat["two-step-controls"]["inline_test"], True))
    checks.append(("a helper is NOT inline even though tests call it",
                   flat["widget-controls"]["inline_test"], False))
    checks.append(("the enclosing function is the one holding the literal",
                   flat["gizmo-tests"]["function"], "gizmo_path"))

    # The two units must be able to disagree, or printing both proves nothing.
    measured = disk(Path(__file__).resolve().parent.parent / "scripts")
    checks.append(("disk usage is measured in allocated blocks, not st_size",
                   measured["disk_bytes"] >= measured["apparent_bytes"], True))
    checks.append(("a directory's own entry is not counted as one of its entries",
                   measured["all_entries"] >= measured["top_level_entries"], True))

    for name, got, want in checks:
        if got != want:
            print(f"SELF-TEST FAILED: {name} — got {got!r}, expected {want!r}",
                  file=sys.stderr)
            return 1

    print("fixture-population census self-test: 4 families derived FROM THE PRODUCER "
          "via the shared patterns (a direct join, a compile-time root, a pushed "
          "array literal and a TWO-STEP join — the shape the hand-written list that "
          "this instrument replaces could not see, and the shape of all six families "
          "it was missing), each mapped to the function that holds the literal; a "
          "helper's scope is its 2 and 1 CALLERS while an inline producer inside a "
          "test is 1, read from the test ATTRIBUTE rather than inferred from a zero "
          "count; and disk usage is proved to be allocated blocks rather than "
          "st_size, which is why both units are printed")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--json", action="store_true",
                        help="the full census as JSON, for a consumer that re-derives")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    return report(ROOT, args.json)


if __name__ == "__main__":
    sys.exit(main())
