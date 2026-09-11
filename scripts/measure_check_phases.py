#!/usr/bin/env python3
"""Measure `make check`'s phases directly instead of inferring them from cargo.

`SIGNOFF-REPAIR.11.4.3.1.2.15`. The full checkpoint's `02-check` command took
3,922 seconds while cargo's own summaries accounted for 813s of compilation and
127s of test execution, leaving about 2,982s that nothing reports. Cargo's
`Finished ... in Ns` line times ONE build graph and stops there: everything after
it — test-binary startup, the browser launcher's setup, rustdoc's doctest pass —
is invisible to the log. A duration nobody can predict is a gate people route
around, so the phases get their own clock here.

This runs the exact commands `make check` runs, in the same order, each with its
own wall clock, and additionally splits the single `cargo test --all` into the
sub-phases that command hides: the test-target compile, the non-doc run, and the
doc run. Every phase writes its own log beside a JSON receipt under a
repository-derived directory (§13: no /tmp, no home cache).

    python3 -B scripts/project_env.py python3 -B scripts/measure_check_phases.py

The receipt names `accounted_seconds` and `unaccounted_seconds` against the
phases' own total, so the next reader compares like with like rather than
re-deriving the arithmetic. Nothing here changes, weakens or skips a gate: it
only times the commands the gate already runs.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from project_env import ROOT, local_directory, project_environment, toolchain_directory


# The phases, in the order `make check` runs them. `make check` is
#   cargo fmt --all -- --check
#   cargo clippy --all-targets --all-features -- -D warnings
#   make test  ->  cargo build --workspace --bins --locked
#                  scripts/ci_browser.py -- cargo test --all --locked
# The final `cargo test --all` is split three ways here. `--no-run` compiles
# every test target; the non-doc run then finds that compile cached, so the two
# durations do not double-count the same work. The doc phase is last for the
# same reason it is last in cargo's own output.
PHASES: list[tuple[str, list[str], bool]] = [
    ("fmt", ["cargo", "fmt", "--all", "--", "--check"], False),
    (
        "clippy",
        ["cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"],
        False,
    ),
    ("build-bins", ["cargo", "build", "--workspace", "--bins", "--locked"], False),
    ("test-compile", ["cargo", "test", "--all", "--locked", "--no-run"], True),
    (
        "test-run",
        ["cargo", "test", "--all", "--locked", "--lib", "--bins", "--tests"],
        True,
    ),
    ("test-doc", ["cargo", "test", "--all", "--locked", "--doc"], True),
]


def browser_launcher(root: Path) -> list[str]:
    """The launcher `make test` wraps its cargo test in.

    Its setup — verifying and, on a cold tree, downloading a 191 MB archive —
    happens inside the timed phase exactly as it does inside `make check`.
    """
    return [sys.executable, "-B", str(root / "scripts" / "ci_browser.py"), "--"]


def run_phase(
    name: str,
    command: list[str],
    directory: Path,
    environment: dict[str, str],
) -> dict[str, object]:
    log = directory / f"{name}.log"
    started = time.monotonic()
    with log.open("wb") as handle:
        completed = subprocess.run(
            command,
            cwd=ROOT,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=handle,
            stderr=subprocess.STDOUT,
            check=False,
        )
    elapsed = time.monotonic() - started
    record = {
        "phase": name,
        "command": command,
        "returncode": completed.returncode,
        "seconds": round(elapsed, 3),
        "log": str(log.relative_to(ROOT)),
    }
    print(
        f"{name}: rc={completed.returncode} seconds={elapsed:.1f} "
        f"log={record['log']}",
        flush=True,
    )
    return record


def cargo_reported_seconds(log: Path) -> float | None:
    """The compile time cargo itself claims for this phase, for comparison.

    Returns None when the phase printed no `Finished ... in` line at all, which
    is itself a finding: a phase with no self-report is a phase whose cost is
    invisible to the checkpoint log.
    """
    import re

    total = None
    pattern = re.compile(r"Finished .* in (?:(\d+)m )?([\d.]+)s")
    for line in log.read_text(errors="replace").splitlines():
        found = pattern.search(line)
        if found:
            minutes = float(found.group(1) or 0)
            total = (total or 0.0) + minutes * 60 + float(found.group(2))
    return total


def harness_reported_seconds(log: Path) -> tuple[int, float]:
    """The seconds the test harnesses themselves report, and how many blocks."""
    import re

    pattern = re.compile(r"finished in ([\d.]+)s")
    values = [float(v) for v in pattern.findall(log.read_text(errors="replace"))]
    return len(values), round(sum(values), 2)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--only",
        action="append",
        choices=[name for name, _, _ in PHASES],
        help="measure only these phases (repeatable); default is every phase",
    )
    arguments = parser.parse_args()

    ambient = dict(os.environ)
    toolchain = toolchain_directory(ROOT, ambient)
    environment = project_environment(ROOT, ambient, toolchain)
    # A checkpoint runs with no ambient database target; keep that precondition.
    environment.pop("DATABASE_URL", None)

    parent = local_directory(ROOT, "target/check-phases")
    parent.mkdir(parents=True, exist_ok=True)
    directory = parent / f"run-{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}"
    directory.mkdir(mode=0o700)

    selected = [p for p in PHASES if not arguments.only or p[0] in arguments.only]
    launcher = browser_launcher(ROOT)
    records: list[dict[str, object]] = []
    stopped = None
    for name, command, needs_browser in selected:
        full = (launcher + command) if needs_browser else command
        record = run_phase(name, full, directory, environment)
        log = ROOT / str(record["log"])
        record["cargo_reported_seconds"] = cargo_reported_seconds(log)
        blocks, harness = harness_reported_seconds(log)
        record["harness_blocks"] = blocks
        record["harness_reported_seconds"] = harness
        records.append(record)
        if record["returncode"] != 0:
            stopped = name
            break

    measured = round(sum(float(r["seconds"]) for r in records), 3)
    accounted = round(
        sum(
            float(r["cargo_reported_seconds"] or 0.0)
            + float(r["harness_reported_seconds"])
            for r in records
        ),
        3,
    )
    receipt = {
        "phases": records,
        "measured_seconds": measured,
        "accounted_seconds": accounted,
        "unaccounted_seconds": round(measured - accounted, 3),
        "stopped_at": stopped,
        "workspace": str(directory.relative_to(ROOT)),
    }
    (directory / "phases.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2), flush=True)
    return 1 if stopped else 0


if __name__ == "__main__":
    sys.exit(main())
