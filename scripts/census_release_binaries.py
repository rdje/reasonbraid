#!/usr/bin/env python3
"""Census: every binary the workspace builds, against the release ledger.

`SIGNOFF-REPAIR.6.8.1`. ADR-027 makes the release manifest **the single
verification unit** — *binaries verify THROUGH it, never individually*. So a
binary the manifest does not name is one the five-rung ladder cannot verify at
all, and `make release` was naming **four** while `cargo build --release --bins`,
the line immediately above it, built **ten**.

🔴 **Two of the six omitted are the acquisition workers `rb-server` SPAWNS**, and
`extraction::worker_path` resolves them from `current_exe().parent()` — the
release directory itself. An executable sitting beside the server that no
manifest names is a code-execution path inside the deployment.

    python3 -B scripts/census_release_binaries.py            # the census
    python3 -B scripts/census_release_binaries.py --check    # the gate
    python3 -B scripts/census_release_binaries.py --release-flags   # what `make release` consumes
    python3 -B scripts/census_release_binaries.py --json
    python3 -B scripts/census_release_binaries.py --self-test

⭐ **THE LEDGER IS THE ONLY LIST, and that is the repair rather than a tidy-up.**
`.doctrine/release_binaries.tsv` carries one row per binary with its disposition
and its reason; `make release` derives its `--bin` flags from the `release` rows;
this census refuses any workspace binary the ledger does not cover. There is no
second copy to drift — which is what let a hardcoded four sit against a growing
workspace without anything noticing.

⛔ **THE WORKSPACE IS READ WITHOUT CARGO'S RESOLVER.** `cargo metadata` would do
it, but it resolves the dependency graph and costs seconds in a pre-commit gate;
the binary targets are a property of the manifests and the filesystem alone.
⚠️ Both routes were run against each other while this was written and returned
the same ten names — `docs/CLAIM_VERIFICATION.md` leg 1, before the number was
published.

⛔ **IT DOES NOT READ A GENERATED MANIFEST.** One exists only after a release
build, so a check that consulted it would silently pass whenever it was absent —
the shape `SIGNOFF-REPAIR.7.1.2.2.1` measured, where twenty suites were dead for
a migration's worth of commits while the gate stayed green. The join is against
the Makefile's own flags, which are always there.
"""

from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = Path(".doctrine/release_binaries.tsv")
MAKEFILE = Path("Makefile")
CRATES = Path("crates")
DISPOSITIONS = ("release", "tool")


def workspace_binaries(root: Path) -> dict[str, str]:
    """Every binary target, from the manifests and the filesystem.

    Cargo's three ways of declaring one, all of them handled: an explicit
    `[[bin]]` table, the implicit `src/main.rs`, and the implicit `src/bin/*.rs`.
    ⛔ A `[[bin]]` grep alone misses the second — `reasonbraid-browse` and
    `reasonbraid-extract` declare no table, which is exactly how an earlier count
    of this population came back as six.
    """
    found: dict[str, str] = {}
    for manifest in sorted((root / CRATES).glob("*/Cargo.toml")):
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        package = data.get("package", {}).get("name")
        if not package:
            continue
        src = manifest.parent / "src"
        explicit = data.get("bin", [])
        for entry in explicit:
            found[entry["name"]] = package
        if (src / "main.rs").exists() and not any(
            e.get("path", "").endswith("main.rs") for e in explicit
        ):
            found.setdefault(package, package)
        bin_dir = src / "bin"
        if bin_dir.is_dir():
            for file in sorted(bin_dir.glob("*.rs")):
                found.setdefault(file.stem, package)
    return found


def ledger_rows(text: str) -> dict[str, tuple[str, str]]:
    rows: dict[str, tuple[str, str]] = {}
    for line in text.splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) < 3:
            continue
        name, disposition, reason = parts[0].strip(), parts[1].strip(), parts[2].strip()
        rows[name] = (disposition, reason)
    return rows


def release_flags(ledger: dict[str, tuple[str, str]]) -> str:
    """The `--bin` flags, emitted for the Makefile to consume.

    ⭐ **The Makefile asks this script rather than parsing the ledger itself.**
    The first attempt had `make` run its own `awk` over the TSV, which is a
    second parser of one file — and it failed immediately for a reason worth
    keeping: `make` strips `#` as a comment **inside `$(shell …)`**, so the awk
    program's own `!/^#/` truncated the call. One parser, asked by name, has
    neither problem.
    """
    return " ".join(f"--bin {n}" for n, (d, _) in sorted(ledger.items()) if d == "release")


def makefile_derivation(text: str) -> tuple[bool, list[str]]:
    """Is the release target's binary list DERIVED, and does it hardcode any?

    ⛔ Comment lines are stripped first, and that is not tidiness: the first
    version of this function matched the explanatory comment directly above the
    recipe — which still names the superseded four — and reported the defect it
    had just repaired. An instrument that reads prose as code confirms whatever
    the prose says.
    """
    code = "\n".join(l for l in text.splitlines() if not l.lstrip().startswith("#"))
    derived = "RELEASE_BIN_FLAGS" in code and "census_release_binaries.py" in code
    hardcoded = re.findall(r"--bin\s+([A-Za-z0-9_-]+)", code)
    return derived, hardcoded


def breaches(
    binaries: dict[str, str],
    ledger: dict[str, tuple[str, str]],
    wiring: tuple[bool, list[str]],
) -> list[str]:
    """Three joins. Each one is a way the copies can disagree."""
    out: list[str] = []
    # 1. The workspace against the ledger — a new binary is adjudicated or refused.
    for name in sorted(set(binaries) - set(ledger)):
        out.append(
            f"the workspace builds `{name}` ({binaries[name]}) and the ledger does not "
            f"cover it — add a row with its disposition and the reason"
        )
    # 2. The ledger against the workspace — a row for a binary that no longer exists
    #    is a rule about nothing, and it would keep `make release` asking for a file
    #    that is never built.
    for name in sorted(set(ledger) - set(binaries)):
        out.append(f"the ledger names `{name}` and the workspace builds no such binary")
    # 3. The vocabulary. A disposition outside the two is not a decision.
    for name, (disposition, reason) in sorted(ledger.items()):
        if disposition not in DISPOSITIONS:
            out.append(
                f"`{name}` has disposition `{disposition}`, outside {DISPOSITIONS}"
            )
        if not reason:
            out.append(f"`{name}` carries no reason — a disposition is a decision")
    # 4. The WIRING. The flags are DERIVED from this ledger, so the Makefile and
    #    the ledger cannot disagree about which binaries are signed — there is no
    #    second list to drift. What can still go wrong is somebody replacing the
    #    derivation with a list again, which is the exact regression this leaf
    #    repaired, so that is what is checked.
    derived, hardcoded = wiring
    if not derived:
        out.append(
            "the release target does not derive its --bin flags from this ledger "
            "(expected RELEASE_BIN_FLAGS, obtained from census_release_binaries.py)"
        )
    if hardcoded:
        out.append(
            f"the release target hardcodes --bin flags {sorted(set(hardcoded))} — a second "
            f"list is how four binaries were signed while ten were built"
        )
    return out


def report(binaries, ledger, wiring) -> int:
    print("workspace binaries, against the release ledger")
    print()
    print(f"  {'binary':<24} {'package':<26} disposition")
    for name in sorted(binaries):
        disposition = ledger.get(name, ("UNCOVERED", ""))[0]
        print(f"  {name:<24} {binaries[name]:<26} {disposition}")
    released = [n for n, (d, _) in ledger.items() if d == "release"]
    print()
    print(f"  {len(binaries)} binary target(s); {len(released)} marked `release`; "
          f"{len(binaries) - len(released)} deliberately not distributed")
    derived, hardcoded = wiring
    print(f"  the release target derives its flags: {'yes' if derived else 'NO'}"
          f"{'' if not hardcoded else f'; hardcoded {sorted(set(hardcoded))}'}")
    print(f"  emitted flags: {release_flags(ledger)}")
    found = breaches(binaries, ledger, wiring)
    print()
    if found:
        for line in found:
            print(f"  BREACH  {line}")
    else:
        print("  the ledger, the workspace and the release target agree")
    return 0


# ── the two-sided self-test (the SELF-TEST doctrine) ────────────────────────
def self_test() -> int:
    arms: list[tuple[str, bool]] = []
    bins = {"a": "pkg-a", "b": "pkg-b", "t": "pkg-t"}
    led = {"a": ("release", "r"), "b": ("release", "r"), "t": ("tool", "r")}
    ok = (True, [])

    # 1 the agreeing case is SILENT — without this, every arm below passes
    #   against a predicate that always breaches.
    arms.append(("an agreeing ledger reports nothing", breaches(bins, led, ok) == []))
    # 2 a NEW workspace binary the ledger does not cover is refused BY NAME
    got = breaches({**bins, "fresh": "pkg-f"}, led, ok)
    arms.append(
        ("an uncovered binary is refused by name", any("`fresh`" in b for b in got))
    )
    # 3 ... and a `tool` row satisfies it, so the gate demands a DECISION and not
    #   a release. Without this arm the gate would be "ship everything".
    got = breaches({**bins, "fresh": "pkg-f"}, {**led, "fresh": ("tool", "why")}, ok)
    arms.append(("marking it `tool` satisfies the gate", got == []))
    # 4 a ledger row for a binary that does not exist is refused
    got = breaches(bins, {**led, "ghost": ("release", "r")}, ok)
    arms.append(("a ledger row naming no binary is refused", any("ghost" in b for b in got)))
    # 5 ⛔ THE ORIGINAL DEFECT'S SHAPE: a hardcoded list in the release target.
    got = breaches(bins, led, (True, ["a", "b"]))
    arms.append(("a hardcoded --bin list is refused", any("hardcodes" in b for b in got)))
    # 6 ... and so is losing the derivation altogether, which is the other way
    #   the wiring can break and would otherwise pass silently.
    got = breaches(bins, led, (False, []))
    arms.append(("a lost derivation is refused", any("does not derive" in b for b in got)))
    # 7 a disposition outside the vocabulary is not a decision
    got = breaches(bins, {**led, "a": ("maybe", "r")}, ok)
    arms.append(("an unknown disposition is refused", any("maybe" in b for b in got)))
    # 8 an empty reason is refused — a disposition without one is a note
    got = breaches(bins, {**led, "t": ("tool", "")}, ok)
    arms.append(("a reason-less row is refused", any("no reason" in b for b in got)))
    # 8b ⛔ COMMENTS ARE NOT CODE. The first version of the wiring reader matched
    #    the explanatory comment above the recipe — which still names the
    #    superseded four — and reported the defect it had just repaired.
    commented = "# used to carry --bin rb --bin rb-server\nRELEASE_BIN_FLAGS := $(shell python3 census_release_binaries.py --release-flags)\n"
    arms.append(("a --bin flag inside a comment is not code", makefile_derivation(commented) == (True, [])))
    # 9 ⛔ THE PARSER ARM. `reasonbraid-browse` declares no `[[bin]]`; it is an
    #   implicit `src/main.rs`. A parser that read only `[[bin]]` tables returned
    #   SIX for this workspace, which is how the population was first miscounted.
    real = workspace_binaries(ROOT)
    arms.append(
        (
            "the implicit src/main.rs targets are found",
            {"reasonbraid-browse", "reasonbraid-extract"} <= set(real),
        )
    )
    # 10 ... and the implicit src/bin/*.rs ones, a different rule again.
    arms.append(("the implicit src/bin/ targets are found", {"rb-site", "rb-node"} <= set(real)))
    # 11 the emitted flags are the `release` rows and nothing else
    arms.append(
        (
            "the emitted flags are exactly the release rows",
            release_flags(led) == "--bin a --bin b",
        )
    )

    passed = sum(1 for _, ok in arms if ok)
    for label, ok in arms:
        print(f"  {'PASS' if ok else 'FAIL'}  {label}")
    print(f"census_release_binaries --self-test: {passed}/{len(arms)} controls pass")
    return 0 if passed == len(arms) else 1


def main(argv: list[str]) -> int:
    mode = argv[1] if len(argv) > 1 else ""
    if mode == "--self-test":
        return self_test()
    ledger = ledger_rows((ROOT / LEDGER).read_text(encoding="utf-8"))
    if mode == "--release-flags":
        # ⛔ stdout carries the flags and NOTHING else: `make` consumes this.
        print(release_flags(ledger))
        return 0
    binaries = workspace_binaries(ROOT)
    wiring = makefile_derivation((ROOT / MAKEFILE).read_text(encoding="utf-8"))
    if mode == "--json":
        print(json.dumps(
            {"binaries": binaries, "ledger": ledger,
             "release_flags": release_flags(ledger),
             "derived": wiring[0], "hardcoded": wiring[1]}, indent=2))
        return 0
    if mode == "--check":
        found = breaches(binaries, ledger, wiring)
        for line in found:
            print(f"RELEASE-BINARY: {line}", file=sys.stderr)
        if found:
            print(
                "  ADR-027 makes the release manifest the single verification unit, so a\n"
                "  binary it does not name cannot be verified at all. Adjudicate the binary\n"
                "  in .doctrine/release_binaries.tsv — `release` or `tool`, with its reason.\n"
                "  `make release` derives its --bin flags from that file.",
                file=sys.stderr,
            )
            return 1
        return 0
    return report(binaries, ledger, wiring)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
