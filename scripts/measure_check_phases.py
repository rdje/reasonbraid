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
import resource
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
    # `SIGNOFF-REPAIR.9.2.1.1.1`: the CPU a phase actually consumes, recorded
    # here rather than left to a reader's division. REPAIR-0190 published one
    # phase's utilisation ratio beside another's and called them the same
    # number; they were 9 % and 207 %, and the arm reporting 207 % was spending
    # almost all of it in the KERNEL. A wall clock alone cannot tell a process
    # that is blocked from one that is busy in a syscall, and that distinction
    # is the whole difference between "this volume is slow" and "this work is
    # expensive". `RUSAGE_CHILDREN` accumulates, so each phase takes a delta.
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
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
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    user = after.ru_utime - before.ru_utime
    system = after.ru_stime - before.ru_stime
    record = {
        "phase": name,
        "command": command,
        "returncode": completed.returncode,
        "seconds": round(elapsed, 3),
        "user_seconds": round(user, 3),
        "system_seconds": round(system, 3),
        # Percent of ONE core. Above 100 means several cores were busy; well
        # below 100 means the phase spent its wall clock waiting rather than
        # computing. Reported, never inferred.
        "cpu_percent": cpu_percent(user, system, elapsed),
        "log": str(log.relative_to(ROOT)),
    }
    print(
        f"{name}: rc={completed.returncode} seconds={elapsed:.1f} "
        f"user={user:.1f} sys={system:.1f} cpu={record['cpu_percent']}% "
        f"log={record['log']}",
        flush=True,
    )
    return record


def cpu_percent(user: float, system: float, elapsed: float) -> float | None:
    """CPU consumed as a percentage of ONE core, or None when unmeasurable.

    A phase that took no measurable wall time has no meaningful ratio, and
    returning 0.0 for it would read as "this phase was idle" — the exact
    misreading this field exists to prevent.
    """
    if elapsed <= 0:
        return None
    return round((user + system) / elapsed * 100, 1)


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


def self_test() -> int:
    """Two-sided controls over the utilisation ratio (`.9.2.1.1.1`).

    ⛔ These are the arms that would have stopped REPAIR-0190 publishing a
    wrong number. It measured two real phases, divided both by hand, and
    reported ONE ratio for both — 9 %, which was the idle arm's. The busy arm
    was 207 %, and 198 % for a third. The arms below are those exact
    measurements, so a change that makes the two indistinguishable fails here.
    """
    failures = 0

    # The real measurements REPAIR-0190 took, with the answers it should have
    # published. `--tests` and clippy are BUSY across several cores; the
    # lib-only phase is genuinely idle. One instrument, three phases, and the
    # spread between them is the finding.
    for label, user, system, elapsed, want in [
        ("cargo check --tests", 46.515, 282.784, 159.014, 207.1),
        ("cargo check (lib only)", 5.836, 11.282, 181.718, 9.4),
        ("cargo clippy --all-targets", 47.093, 267.743, 158.723, 198.4),
    ]:
        got = cpu_percent(user, system, elapsed)
        if got != want:
            print(f"SELF-TEST: {label} -> {got}, want {want}", file=sys.stderr)
            failures += 1

    # The busy and the idle arm must not be confusable. Stated as its own
    # control rather than left implicit in the three above, because "they
    # agreed" is precisely the claim that failed.
    busy = cpu_percent(46.515, 282.784, 159.014)
    idle = cpu_percent(5.836, 11.282, 181.718)
    if busy is None or idle is None or busy < idle * 10:
        print(
            f"SELF-TEST: a busy phase ({busy}%) must be an order of magnitude "
            f"above an idle one ({idle}%)",
            file=sys.stderr,
        )
        failures += 1

    # Kernel time counts. A phase burning its wall clock in syscalls is BUSY,
    # and dropping system time would report it as idle — the opposite reading.
    if cpu_percent(0.0, 120.0, 60.0) != 200.0:
        print("SELF-TEST: system time must count toward utilisation", file=sys.stderr)
        failures += 1

    # An unmeasurable ratio is None, never 0.0: zero reads as "idle", which is
    # a claim, and this instrument has none to make about a phase it could not
    # time.
    for elapsed in (0.0, -1.0):
        if cpu_percent(1.0, 1.0, elapsed) is not None:
            print(f"SELF-TEST: elapsed={elapsed} must yield None", file=sys.stderr)
            failures += 1

    if failures:
        print(f"measure_check_phases --self-test: {failures} control(s) FAILED", file=sys.stderr)
        return 1
    print("measure_check_phases --self-test: 8 controls pass")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--only",
        action="append",
        choices=[name for name, _, _ in PHASES],
        help="measure only these phases (repeatable); default is every phase",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run this instrument's own controls and exit",
    )
    arguments = parser.parse_args()

    if arguments.self_test:
        return self_test()

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
