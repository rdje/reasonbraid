#!/usr/bin/env python3
"""Census: every `pub` item defined in reasonbraid-adapter, and whether any
NON-TEST caller outside its own defining file reaches it.

SIGNOFF-REPAIR.13.1.1's acceptance asks for this over the whole crate rather
than over the two items with known answers, because scoping a census to the
cases you already understand is how a census confirms what you assumed.

A `pub` item with no non-test caller is not necessarily a defect — it may be a
library surface, or a control that is ahead of its call site. What the census
produces is the POPULATION; classifying it is the leaf's job, never this
script's.

  python3 scripts/census_adapter_public_api.py            # the census
  python3 scripts/census_adapter_public_api.py --self-test # the predicate, both ways
"""
from __future__ import annotations
import re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATE = ROOT / "crates" / "reasonbraid-adapter" / "src"

DEF = re.compile(
    r"^\s*pub(?:\([^)]*\))?\s+(?:async\s+)?(?:const\s+)?"
    r"(?:fn|struct|enum|trait|type|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)",
    re.M,
)


def strip_test_blocks(src: str) -> str:
    """Remove `#[cfg(test)] mod ... { ... }` by BRACE MATCHING.

    ⛔ Not by searching for the next `pub`/`}` at column 0: a terminator that
    depends on how the next item happens to be spelled is not a delimiter, which
    is the defect `.13.1.2` had to repair in its own parser.
    """
    out, i = [], 0
    while True:
        m = re.search(r"#\[cfg\(test\)\]", src[i:])
        if not m:
            out.append(src[i:])
            return "".join(out)
        start = i + m.start()
        out.append(src[i:start])
        brace = src.find("{", start)
        if brace == -1:
            return "".join(out)
        depth, j = 0, brace
        while j < len(src):
            if src[j] == "{":
                depth += 1
            elif src[j] == "}":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        i = j + 1


def tracked_rust() -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", "crates/*.rs", "crates/**/*.rs"],
        cwd=ROOT, capture_output=True, text=True, check=True).stdout.split()
    return [ROOT / p for p in out if p.endswith(".rs")]


def census() -> list[tuple[str, str, int]]:
    items: dict[str, Path] = {}
    for f in sorted(CRATE.rglob("*.rs")):
        for name in DEF.findall(strip_test_blocks(f.read_text())):
            items.setdefault(name, f)

    bodies: list[tuple[Path, str]] = []
    for f in tracked_rust():
        try:
            bodies.append((f, strip_test_blocks(f.read_text())))
        except (OSError, UnicodeDecodeError):
            continue

    rows = []
    for name, home in sorted(items.items()):
        word = re.compile(rf"\b{re.escape(name)}\b")
        callers = 0
        for f, body in bodies:
            if f == home:
                continue                      # the definition's own file
            if f.parts[-2] == "tests" or "/tests/" in str(f):
                continue                      # integration tests are not production
            callers += len(word.findall(body))
        rows.append((name, str(home.relative_to(ROOT)), callers))
    return rows


def self_test() -> int:
    fails = 0
    probe = """
pub fn reached() {}
pub fn unreached() {}
#[cfg(test)]
mod t {
    fn inner() { if true { unreached(); } }
    fn again() { unreached(); }
}
pub struct Kept;
"""
    stripped = strip_test_blocks(probe)
    # ⛔ Assert on the CALL SITES (`name();`), not on the bare name: the probe's
    # own definition line `pub fn unreached() {}` contains `unreached()`, so a
    # substring check on the name passes/fails for the wrong reason. The first
    # version of this assertion did exactly that and reported a working stripper
    # as broken — the mirror of `.13.1.2`, where a weak assertion reported a
    # broken parser as working.
    if "unreached();" in stripped:
        print("SELF-TEST: a test-block CALL survived stripping", file=sys.stderr); fails += 1
    if stripped.count("pub fn unreached") != 1:
        print("SELF-TEST: the definition outside the test block was not kept", file=sys.stderr); fails += 1
    if "pub struct Kept" not in stripped:
        print("SELF-TEST: brace matching consumed the item AFTER the test block", file=sys.stderr); fails += 1
    if "pub fn reached" not in stripped:
        print("SELF-TEST: stripping ate code before the test block", file=sys.stderr); fails += 1
    names = set(DEF.findall(stripped))
    if names != {"reached", "unreached", "Kept"}:
        print(f"SELF-TEST: definition regex found {sorted(names)}", file=sys.stderr); fails += 1
    # the real file, asserted EXACTLY rather than "parsed something"
    real = {n for n, _, _ in census()}
    for expected in ("verify_ladder", "capabilities_within", "AllowedCapabilities"):
        if expected not in real:
            print(f"SELF-TEST: real-file parse missed {expected}", file=sys.stderr); fails += 1
    if fails == 0:
        print(f"adapter-public-api self-test: 4 predicate cases, {len(real)} real items parsed")
    return fails


def main() -> int:
    if "--self-test" in sys.argv:
        return 1 if self_test() else 0
    rows = census()
    zero = [r for r in rows if r[2] == 0]
    print(f"{len(rows)} pub items in reasonbraid-adapter; {len(zero)} with NO non-test caller outside their own file\n")
    for name, home, n in rows:
        mark = "  <-- none" if n == 0 else ""
        print(f"  {n:5d}  {name:<28} {home}{mark}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
