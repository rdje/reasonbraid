#!/usr/bin/env python3
"""Census — and, on explicit request, REDUCE — the fixtures the test suites retain
after a failure (`SIGNOFF-REPAIR.7.3.2.1`).

Two suites retain a workspace when a run fails, deliberately and correctly: the
retained directory IS the diagnostic evidence.

  - `crates/reasonbraid-browse/tests/support/mod.rs` keeps a fixture under
    `target/browser-lifetime-controls/case-*` whenever the control failed or the
    browser group outlived it;
  - `scripts/run_pg_tests.py` removes its cluster under `target/pg-tests/run-*`
    only on SUCCESS (its single `shutil.rmtree`), and prints
    `failure evidence retained in …` otherwise.

What neither has is a RETIREMENT rule, so both grow without bound until somebody
looks. Measured when this leaf opened: 12 browser fixtures at 515,424 KiB, and
12 pg clusters at 599 MiB.

⭐ THE RULE, one for both populations: KEEP THE RECEIPT, DROP THE REPRODUCIBLE
PAYLOAD — `SIGNOFF-REPAIR.11.4.8`'s shape for `target/ci-browser/`. It is not a
guess about which bytes matter; it is measured, and the measurement is why the
payload is defined as a list of DIRECTORIES rather than "everything but the logs":

    fixture case-01a0a6ee…  55,600 KiB total
      .project-data/browser/run-…/profile   55,372 KiB   Chrome profile storage,
                                                         36.7 MiB of it a single
                                                         optimization-guide
                                                         model.tflite
      .project-data/browser/run-…/cache        200 KiB   Chrome cache
      worker.json, stderr.log, stdout.log       12 KiB   the receipts
      .project-data/browser/run-…/owner.json,
                             completion.json     8 KiB   ⚠️ the WORKER's own
                                                         receipts, and they live
                                                         INSIDE the payload
                                                         directory. Dropping
                                                         `.project-data` wholesale
                                                         would have destroyed the
                                                         browser_group and
                                                         cleanup_confirmed record.

⛔ DELETION IS NOT THE REPAIR and this instrument never performs one. A retained
fixture is what makes a failed browser control diagnosable; `Fixture::finish`
already refuses to delete a fixture whose browser group outlived it. The question
this answers is what a retained fixture must KEEP — not whether to keep one. A
reduced fixture is left in place, carrying every log, receipt and configuration
file it had, plus a `retired.json` naming exactly what was dropped.

⛔ NOTHING HERE EVER SIGNALS A PROCESS. `SIGNOFF-REPAIR.7.3.2` owns the rule and
states it directly: "identify each process before cleanup, never signal a
historical numeric PID solely from a stale receipt". Every liveness probe below
is `signal 0` — an existence question, not an instruction. A receipt's numeric id
may have been recycled by an unrelated process months later, so the probe is
allowed to say only "this id is in use", never "this is that process".

⚠️ WHICH DIRECTION THAT ERRS IN, stated because a scoping defect always errs in
one (`docs/knowledge/a-scoping-defect-errs-in-one-direction.md`): an id that is
in use REFUSES. A recycled id therefore costs a kept fixture, never a wrongly
dropped payload. Refusal is the safe direction and the cost of being wrong is
disk, not evidence.

    python3 -B scripts/project_env.py python3 -B scripts/census_retained_fixtures.py
    python3 -B scripts/project_env.py python3 -B scripts/census_retained_fixtures.py --retire --confirm
    python3 -B scripts/census_retained_fixtures.py --population browser
    python3 -B scripts/census_retained_fixtures.py --self-test

Reduction requires EVERY one of these to hold, re-checked at the moment of
removal rather than read off the census:

  - the receipt parses. A fixture this cannot describe is not one it will touch;
  - a pg cluster's receipt says `state: stopped` — the runner verified shutdown;
  - no id the fixture records is in use: the browser process GROUPS named by
    every `browser ownership:` line and every `owner.json`, the worker/postgres/
    command pids in the receipt, and a pg `postmaster.pid`;
  - the directory is a real directory (not a symlink) on the REPOSITORY's own
    volume, so nothing outside the repository's storage is ever touched;
  - no TRACKED file names it. A fixture a task tree cites by name is evidence;
  - it is older than `--min-age-hours` (default 1), so a run still being read is
    left alone.

A fixture failing any check is REPORTED and kept whole. The exit code is the
verdict: 0 when the requested operation completed, 1 when something was refused.

⚠️ The neighbouring instrument `scripts/census_pg_test_clusters.py` REMOVES a pg
cluster outright — a different disposition, owned by `SIGNOFF-REPAIR.11.4.3.1.7`.
It is guarded against undoing this one: a cluster carrying `retired.json` has
already been reduced to its evidence, and that instrument refuses it.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RETIRED = "retired.json"
RULE = "keep the receipt, drop the reproducible payload"
LEAF = "SIGNOFF-REPAIR.7.3.2.1"


class Population:
    """One retained-fixture family: where it lives and which bytes reproduce.

    `payload` entries are globs relative to the fixture. Only DIRECTORIES that
    match are dropped — a file that matches, or any path that matches nothing, is
    kept. The default is therefore retention, which is the direction a mistake in
    this list should err in.
    """

    def __init__(
        self,
        name: str,
        root: str,
        prefix: str,
        receipt: str,
        payload: list[str],
        require_state: str | None = None,
        state_keys: tuple[str, ...] = ("state",),
    ) -> None:
        self.name = name
        self.root = ROOT / root
        self.prefix = prefix
        self.receipt = receipt
        self.payload = payload
        self.require_state = require_state
        self.state_keys = state_keys

    def describe(self, receipt: dict) -> str:
        """The one receipt field worth a census column for this population."""
        for key in self.state_keys:
            if key in receipt:
                return f"{key}={receipt[key]}"
        return "unknown"


POPULATIONS = {
    "browser": Population(
        name="browser",
        root="target/browser-lifetime-controls",
        prefix="case-",
        receipt="worker.json",
        # The Chrome profile and cache, per browser run, plus the fixture's own
        # XDG directories. `owner.json`, `completion.json` and `crashes/` sit
        # beside `profile/` and are NOT listed: they are the worker's receipts
        # and any real crash dump, which is exactly what a reader still acts on.
        payload=[
            ".project-data/browser/*/profile",
            ".project-data/browser/*/cache",
            "cache",
            "config",
            "data",
            "state",
            "tmp",
        ],
        state_keys=("group_cleanup_confirmed", "cleanup_error"),
    ),
    "pg": Population(
        name="pg",
        root="target/pg-tests",
        prefix="run-",
        receipt="runner.json",
        # Every DIRECTORY under PGDATA is cluster storage that `initdb` and a
        # re-run reproduce. The plain files at its root are configuration —
        # `postgresql.conf`, `pg_hba.conf`, `PG_VERSION`, `postmaster.opts` —
        # and say what the failing cluster actually ran with, so they stay. The
        # failure output a reader reads (`postgres.log`, `command-N.log`,
        # `runner.json`) is above PGDATA and is never in scope here.
        payload=["data/*"],
        require_state="stopped",
    ),
}


def id_in_use(kind: str, ident: int) -> bool:
    """Is this pid / process-group id in use RIGHT NOW? Signal 0 only.

    EPERM means the id exists and belongs to somebody else — presence, and the
    most important case to get right, because it is the one we must not touch.
    """
    if ident <= 1:
        return False
    try:
        if kind == "group":
            os.killpg(ident, 0)
        else:
            os.kill(ident, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    except OSError:
        return True  # an unreadable answer is not an absence
    return True


class Fixture:
    def __init__(self, path: Path, population: Population) -> None:
        self.path = path
        self.population = population
        self.name = path.name
        self.receipt: dict = {}
        self.receipt_error: str | None = None
        receipt = path / population.receipt
        if receipt.is_file():
            try:
                self.receipt = json.loads(receipt.read_text())
            except (OSError, ValueError) as error:  # a truncated receipt is a fact
                self.receipt_error = str(error)
        else:
            self.receipt_error = f"no {population.receipt}"
        self.files, self.bytes = tree_size(path)
        self.payload_paths = self.resolve_payload()
        self.payload_files = 0
        self.payload_bytes = 0
        for target in self.payload_paths:
            files, size = tree_size(target)
            self.payload_files += files
            self.payload_bytes += size
        self.age_hours = (time.time() - path.stat().st_mtime) / 3600.0

    @property
    def retired(self) -> bool:
        return (self.path / RETIRED).is_file()

    @property
    def state(self) -> str:
        return str(self.receipt.get("state", "unknown"))

    @property
    def described(self) -> str:
        return self.population.describe(self.receipt)

    def resolve_payload(self) -> list[Path]:
        """Every DIRECTORY matching a payload glob, within the fixture."""
        found: list[Path] = []
        for pattern in self.population.payload:
            for candidate in sorted(self.path.glob(pattern)):
                # Never follow a link out of the fixture, and never leave it.
                if candidate.is_symlink() or not candidate.is_dir():
                    continue
                if not str(candidate.resolve()).startswith(str(self.path.resolve()) + os.sep):
                    continue
                found.append(candidate)
        return found

    def recorded_ids(self) -> list[tuple[str, int, str]]:
        """(kind, id, where) for every process identity this fixture records."""
        ids: list[tuple[str, int, str]] = []

        def add(kind: str, raw, where: str) -> None:
            try:
                ident = int(raw)
            except (TypeError, ValueError):
                return
            ids.append((kind, ident, where))

        for key in ("pid", "postgres_pid", "command_pid"):
            if key in self.receipt:
                add("pid", self.receipt[key], f"{self.population.receipt}:{key}")

        stderr = self.path / "stderr.log"
        if stderr.is_file():
            try:
                text = stderr.read_text(errors="replace")
            except OSError:
                text = ""
            for line in text.splitlines():
                marker = "browser ownership: "
                if marker not in line:
                    continue
                try:
                    receipt = json.loads(line.split(marker, 1)[1])
                except ValueError:
                    continue
                add("group", receipt.get("browser_group"), "stderr.log ownership receipt")

        for owner in sorted(self.path.glob(".project-data/browser/*/owner.json")):
            try:
                receipt = json.loads(owner.read_text())
            except (OSError, ValueError):
                continue
            add("group", receipt.get("browser_group"), f"{owner.name} in {owner.parent.name}")

        postmaster = self.path / "data" / "postmaster.pid"
        if postmaster.is_file():
            try:
                add("pid", postmaster.read_text().splitlines()[0], "data/postmaster.pid")
            except (OSError, IndexError):
                pass

        return ids


def tree_size(path: Path) -> tuple[int, int]:
    files = 0
    total = 0
    if not path.exists():
        return (0, 0)
    for dirpath, _dirs, names in os.walk(path):
        for name in names:
            try:
                stat = os.lstat(os.path.join(dirpath, name))
            except OSError:
                continue
            files += 1
            total += stat.st_size
    return (files, total)


def tracked_references(name: str) -> int:
    """How many TRACKED files name this fixture. A cited fixture is evidence.

    `-F`: a fixture name is a literal, and a name carrying a regex metacharacter
    must not quietly widen the search.
    """
    result = subprocess.run(
        ["git", "grep", "-c", "-F", "--", name],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode not in (0, 1):
        raise SystemExit(f"git grep failed for {name}: {result.stderr.strip()}")
    return len([line for line in result.stdout.splitlines() if line.strip()])


def refusals(fixture: Fixture, repo_device: int, min_age_hours: float) -> list[str]:
    reasons: list[str] = []
    if fixture.receipt_error is not None:
        reasons.append(f"unreadable receipt ({fixture.receipt_error})")
    want = fixture.population.require_state
    if want is not None and fixture.state != want:
        reasons.append(f"receipt state is {fixture.state!r}, not {want!r}")
    for kind, ident, where in fixture.recorded_ids():
        if id_in_use(kind, ident):
            reasons.append(f"{kind} {ident} ({where}) is in use")
    stat = os.lstat(fixture.path)
    if not fixture.path.is_dir() or fixture.path.is_symlink():
        reasons.append("not a real directory")
    elif stat.st_dev != repo_device:
        reasons.append(f"device {stat.st_dev} is not the repository's {repo_device}")
    if fixture.age_hours < min_age_hours:
        reasons.append(f"age {fixture.age_hours:.2f}h is under the {min_age_hours}h floor")
    references = tracked_references(fixture.name)
    if references:
        reasons.append(f"named by {references} tracked file(s) — it is cited evidence")
    return reasons


def reduce_fixture(fixture: Fixture, after_removal=None) -> tuple[int, int]:
    """Drop the payload directories, keep everything else, leave a receipt.

    Returns (files, bytes) dropped. The kept tree is measured before and after
    and any difference is an error: this must remove the payload and NOTHING else.

    `after_removal` is a SEAM, and it exists because falsifying this file found
    the guard below to be vacuous: the reduction is correct, so `after != before`
    never fired, and deleting the whole comparison left every arm green. A guard
    that cannot be observed refusing is not known to work
    (`docs/knowledge/a-control-that-passes-for-an-unrelated-reason.md`). The
    self-test passes a callback that damages a kept file on purpose; production
    passes None and the seam costs one `if`.
    """
    before = {}
    for dirpath, _dirs, names in os.walk(fixture.path):
        for name in names:
            full = Path(dirpath) / name
            if any(full == p or p in full.parents for p in fixture.payload_paths):
                continue
            try:
                before[str(full.relative_to(fixture.path))] = os.lstat(full).st_size
            except OSError:
                continue

    dropped = []
    for target in fixture.payload_paths:
        files, size = tree_size(target)
        shutil.rmtree(target)
        if target.exists():
            raise SystemExit(f"payload removal not confirmed: {target}")
        dropped.append(
            {
                "path": str(target.relative_to(fixture.path)),
                "files": files,
                "bytes": size,
            }
        )

    if after_removal is not None:
        after_removal(fixture.path)

    kept_files, kept_bytes = tree_size(fixture.path)
    (fixture.path / RETIRED).write_text(
        json.dumps(
            {
                "rule": RULE,
                "instrument": "scripts/census_retained_fixtures.py",
                "leaf": LEAF,
                "population": fixture.population.name,
                "retired_utc": datetime.now(timezone.utc).isoformat(timespec="seconds"),
                "dropped": dropped,
                "dropped_files": sum(d["files"] for d in dropped),
                "dropped_bytes": sum(d["bytes"] for d in dropped),
                "kept_files": kept_files,
                "kept_bytes": kept_bytes,
                "reproducible_by": (
                    "re-running the control that produced this fixture; the dropped "
                    "trees are browser profile/cache or PGDATA cluster storage, which "
                    "no reader acts on. Every log, receipt and configuration file this "
                    "fixture carried is still here."
                ),
            },
            indent=2,
            sort_keys=True,
        )
        + "\n"
    )

    # The kept tree must be untouched, byte for byte, name for name.
    after = {}
    for dirpath, _dirs, names in os.walk(fixture.path):
        for name in names:
            full = Path(dirpath) / name
            relative = str(full.relative_to(fixture.path))
            if relative == RETIRED:
                continue
            try:
                after[relative] = os.lstat(full).st_size
            except OSError:
                continue
    if after != before:
        lost = sorted(set(before) - set(after))
        changed = sorted(k for k in set(before) & set(after) if before[k] != after[k])
        raise SystemExit(
            f"REDUCTION ALTERED KEPT EVIDENCE in {fixture.name}: "
            f"lost={lost} changed={changed}"
        )
    return (sum(d["files"] for d in dropped), sum(d["bytes"] for d in dropped))


def retire_all(
    fixtures: list[Fixture], repo_device: int, min_age_hours: float
) -> tuple[int, int]:
    """Reduce each fixture, RE-CHECKING every refusal immediately before doing it.

    Extracted from `main` so the re-check has a control. Inline, it had none: a
    mutant replacing it with `again = []` left the whole self-test green, because
    nothing drove this loop. A census is minutes old by the time a human types
    `--confirm`, and a fixture can acquire a live browser group in between.

    Returns (fixtures reduced, bytes dropped).
    """
    reduced = 0
    dropped_bytes = 0
    for fixture in fixtures:
        again = refusals(fixture, repo_device, min_age_hours)
        if again:
            print(f"KEEP {fixture.name} (changed since the census): " + "; ".join(again))
            continue
        files, size = reduce_fixture(fixture)
        dropped_bytes += size
        reduced += 1
        print(
            f"reduced {fixture.path.relative_to(ROOT)}: dropped {files} file(s), "
            f"{size} bytes; evidence and {RETIRED} remain"
        )
    return (reduced, dropped_bytes)


def self_test() -> int:
    """Fire every refusal on purpose, then reduce a synthetic fixture and prove
    what survived. A guard never seen refuse is not known to work."""
    import tempfile

    repo_device = os.stat(ROOT).st_dev
    scratch = ROOT / "target" / "retained-fixture-census-selftest"
    scratch.mkdir(parents=True, exist_ok=True)
    failures: list[str] = []
    browser = POPULATIONS["browser"]
    pg = POPULATIONS["pg"]

    def build(
        name: str,
        population: Population,
        receipt: dict,
        *,
        group: int | None = None,
        payload_bytes: int = 4096,
        age_seconds: float = 3600 * 5,
    ) -> Fixture:
        path = Path(tempfile.mkdtemp(prefix=f"{name}-", dir=scratch))
        (path / population.receipt).write_text(json.dumps(receipt))
        if population.name == "browser":
            run = path / ".project-data" / "browser" / "run-selftest"
            (run / "profile" / "deep").mkdir(parents=True)
            (run / "profile" / "deep" / "model.bin").write_bytes(b"x" * payload_bytes)
            (run / "crashes").mkdir()
            (run / "crashes" / "dump.txt").write_text("a real crash dump is evidence\n")
            (run / "owner.json").write_text(json.dumps({"browser_group": group or 0}))
            (run / "completion.json").write_text(json.dumps({"cleanup_confirmed": False}))
            (path / "stdout.log").write_text('{"error":"kept"}\n')
            if group is not None:
                (path / "stderr.log").write_text(
                    "browser ownership: " + json.dumps({"browser_group": group}) + "\n"
                )
        else:
            (path / "data" / "base").mkdir(parents=True)
            (path / "data" / "base" / "1").write_bytes(b"x" * payload_bytes)
            (path / "data" / "postgresql.conf").write_text("shared_buffers = 1MB\n")
            (path / "postgres.log").write_text("FATAL: the failure a reader reads\n")
        old = time.time() - age_seconds
        os.utime(path, (old, old))
        return Fixture(path, population)

    def expect(label: str, fixture: Fixture, *, fragment: str | None, min_age: float = 1.0) -> None:
        reasons = refusals(fixture, repo_device, min_age)
        if fragment is None:
            if reasons:
                failures.append(f"{label}: expected no refusal, got {reasons}")
        elif not any(fragment in reason for reason in reasons):
            failures.append(f"{label}: expected a refusal mentioning {fragment!r}, got {reasons}")

    try:
        expect("a clean, aged, uncited browser fixture", build("clean", browser, {"pid": 1}), fragment=None)
        expect("a clean, aged, uncited pg cluster", build("cleanpg", pg, {"state": "stopped"}), fragment=None)
        bare = Path(tempfile.mkdtemp(prefix="noreceipt-", dir=scratch))
        expect("a missing receipt", Fixture(bare, browser), fragment="unreadable receipt")
        broken = build("broken", browser, {"pid": 1})
        (broken.path / browser.receipt).write_text("{not json")
        expect("an unreadable receipt", Fixture(broken.path, browser), fragment="unreadable receipt")
        expect(
            "a pg cluster whose shutdown was never verified",
            build("unverified", pg, {"state": "shutdown-unverified"}),
            fragment="not 'stopped'",
        )
        # Our own process group and pid are the one liveness we can assert.
        expect(
            "a live browser group",
            build("livegroup", browser, {"pid": 1}, group=os.getpgrp()),
            fragment=f"group {os.getpgrp()}",
        )
        expect(
            "a live worker pid",
            build("livepid", browser, {"pid": os.getpid()}),
            fragment=f"pid {os.getpid()}",
        )
        expect(
            "a live postmaster",
            build("livepm", pg, {"state": "stopped", "postgres_pid": os.getpid()}),
            fragment=f"pid {os.getpid()}",
        )
        # ⛔ EPERM IS THE CASE THAT MATTERS AND OUR OWN PIDS CANNOT REACH IT.
        # The arms above probe `os.getpid()`/`os.getpgrp()`, which we own, so
        # `signal 0` SUCCEEDS and the PermissionError branch never runs. Flipping
        # that branch to `return False` — which would let this instrument reduce a
        # fixture whose browser group is alive under another uid — passed every
        # arm. Found by mutating this file, not by reading it.
        real_kill = os.kill
        try:
            os.kill = lambda pid, sig: (_ for _ in ()).throw(PermissionError())
            if not id_in_use("pid", 424242):
                failures.append("EPERM: a live process owned by somebody else read as absent")
            os.kill = lambda pid, sig: (_ for _ in ()).throw(ProcessLookupError())
            if id_in_use("pid", 424242):
                failures.append("ESRCH: a genuinely absent pid read as present")
        finally:
            os.kill = real_kill
        # And the same rule against the real process table, when the machine
        # offers a process this user cannot signal. Skipped loudly, never silently.
        foreign = subprocess.run(
            ["ps", "-axo", "pid=,uid="], capture_output=True, text=True, check=False
        )
        candidates = [
            int(line.split()[0])
            for line in foreign.stdout.splitlines()
            if len(line.split()) == 2 and line.split()[1] == "0" and int(line.split()[0]) > 1
        ]
        if os.geteuid() == 0 or not candidates:
            print("self-test NOTE: no unsignalable process available; EPERM checked by injection only")
        elif not id_in_use("pid", candidates[0]):
            failures.append(f"EPERM: live root-owned pid {candidates[0]} read as absent")

        expect(
            "a young fixture",
            build("young", browser, {"pid": 1}),
            fragment="under the",
            min_age=1e9,
        )

        # The citation guard, in BOTH directions, against the real tracked tree.
        #
        # `SIGNOFF-REPAIR.11.4.3.1.7.1`: the absent-name probe is GENERATED, never
        # written as a literal. Its predecessor was a literal this very file
        # contains and `git grep` searches, so the arm passed exactly once — while
        # the file was still untracked — and failed from the instant it was
        # committed. A probe written down here cannot be absent from here.
        absent = f"case-selftest-{uuid.uuid4().hex}"
        if tracked_references("run_pg_tests.py") == 0:
            failures.append("citation guard: a tracked filename reported zero references")
        if tracked_references(absent) != 0:
            failures.append(f"citation guard: the absent name {absent} reported references")

        # ── The reduction itself: the evidence survives, the payload does not ──
        subject = build("reduce", browser, {"pid": 1, "stderr_bytes": 7}, payload_bytes=100_000)
        keepers = {
            "worker.json",
            "stdout.log",
            ".project-data/browser/run-selftest/owner.json",
            ".project-data/browser/run-selftest/completion.json",
            ".project-data/browser/run-selftest/crashes/dump.txt",
        }
        before = {k: (subject.path / k).read_bytes() for k in keepers}
        dropped_files, dropped_bytes = reduce_fixture(subject)
        for name, content in before.items():
            target = subject.path / name
            if not target.is_file():
                failures.append(f"reduction: {name} was evidence and it is gone")
            elif target.read_bytes() != content:
                failures.append(f"reduction: {name} changed")
        if (subject.path / ".project-data/browser/run-selftest/profile").exists():
            failures.append("reduction: the profile payload survived")
        if dropped_bytes < 100_000:
            failures.append(f"reduction: dropped only {dropped_bytes} bytes of a 100,000-byte payload")
        if not (subject.path / RETIRED).is_file():
            failures.append("reduction: no retired.json receipt was written")
        else:
            written = json.loads((subject.path / RETIRED).read_text())
            if written.get("dropped_files") != dropped_files:
                failures.append("reduction: the receipt disagrees with what was dropped")
        again = Fixture(subject.path, browser)
        if not again.retired:
            failures.append("reduction: a reduced fixture does not report itself retired")
        if again.payload_paths:
            failures.append("reduction: a reduced fixture still resolves payload to drop")

        # The kept-tree guard, seen refusing. Without this the whole comparison
        # could be deleted and every arm stayed green (see `reduce_fixture`).
        damaged = build("damaged", browser, {"pid": 1}, payload_bytes=8192)
        try:
            reduce_fixture(damaged, after_removal=lambda root: (root / "stdout.log").unlink())
        except SystemExit as raised:
            if "ALTERED KEPT EVIDENCE" not in str(raised):
                failures.append(f"kept-tree guard raised the wrong thing: {raised}")
        else:
            failures.append("kept-tree guard: a destroyed keeper was not detected")

        # A pg cluster keeps its configuration and its failure log.
        pgsubject = build("reducepg", pg, {"state": "stopped"}, payload_bytes=50_000)
        reduce_fixture(pgsubject)
        for kept in ("postgres.log", "data/postgresql.conf"):
            if not (pgsubject.path / kept).is_file():
                failures.append(f"pg reduction: {kept} was evidence and it is gone")
        if (pgsubject.path / "data" / "base").exists():
            failures.append("pg reduction: PGDATA cluster storage survived")

        # ── The re-check: a fixture that goes LIVE between census and --confirm ──
        # The census below finds nothing to refuse. The fixture then acquires a
        # live browser group, exactly as a real one does when a human takes a
        # minute to type `--confirm`. Retirement must notice and keep it whole.
        racer = build("racer", browser, {"pid": 1}, payload_bytes=70_000)
        if refusals(racer, repo_device, 1.0):
            failures.append("re-check: the racer was refused before the race even started")
        (racer.path / "stderr.log").write_text(
            "browser ownership: " + json.dumps({"browser_group": os.getpgrp()}) + "\n"
        )
        racer_payload = racer.path / ".project-data/browser/run-selftest/profile/deep/model.bin"
        raced, _ = retire_all([racer], repo_device, 1.0)
        if raced != 0:
            failures.append("re-check: a fixture that went live between census and retire was reduced")
        if not racer_payload.is_file() or len(racer_payload.read_bytes()) != 70_000:
            failures.append("re-check: the raced fixture's payload was damaged")

        # ── And a LIVE fixture is untouched by the same pass ──
        live = build("untouched", browser, {"pid": 1}, group=os.getpgrp(), payload_bytes=50_000)
        live_payload = live.path / ".project-data/browser/run-selftest/profile/deep/model.bin"
        digest_before = live_payload.read_bytes()
        for fixture in (live,):
            if not refusals(fixture, repo_device, 1.0):
                failures.append("the live fixture was not refused, so 'untouched' proves nothing")
        if not live_payload.is_file() or live_payload.read_bytes() != digest_before:
            failures.append("a refused fixture's payload was altered")
    finally:
        shutil.rmtree(scratch, ignore_errors=True)

    for failure in failures:
        print(f"SELF-TEST FAILED: {failure}", file=sys.stderr)
    if failures:
        return 1
    print(
        "self-test: 9 refusal arms, both liveness branches (EPERM injected and, where the "
        "machine allows, a real unsignalable pid), the citation guard's two directions, a "
        "browser and a pg reduction that kept every receipt and dropped every payload byte, "
        "the kept-tree guard seen refusing a destroyed keeper, a fixture that went live "
        "between census and retirement left whole, and a live fixture untouched"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="fire every refusal against synthetic fixtures, reduce two, and exit",
    )
    parser.add_argument(
        "--population",
        choices=sorted(POPULATIONS),
        action="append",
        help="census only this population (repeatable; default: all)",
    )
    parser.add_argument(
        "--retire",
        action="store_true",
        help="reduce every fixture that passes all checks (requires --confirm)",
    )
    parser.add_argument(
        "--confirm",
        action="store_true",
        help="perform the reduction; without it --retire only reports what it would do",
    )
    parser.add_argument(
        "--min-age-hours",
        type=float,
        default=1.0,
        help="leave fixtures younger than this alone (default 1)",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    repo_device = os.stat(ROOT).st_dev
    chosen = args.population or sorted(POPULATIONS)
    reducible: list[Fixture] = []
    grand_bytes = 0
    grand_payload = 0

    for key in chosen:
        population = POPULATIONS[key]
        relative = population.root.relative_to(ROOT)
        if not population.root.is_dir():
            print(f"{key}: {relative} does not exist — nothing retained\n")
            continue
        fixtures = sorted(
            (
                Fixture(path, population)
                for path in population.root.iterdir()
                if path.is_dir() and path.name.startswith(population.prefix)
            ),
            key=lambda f: f.name,
        )
        if not fixtures:
            print(f"{key}: none retained under {relative}\n")
            continue

        print(f"{key}: {relative}")
        print(
            f"  {'fixture':<40} {'files':>7} {'bytes':>12} {'payload':>12} "
            f"{'age_h':>7}  state"
        )
        verdicts: list[tuple[Fixture, list[str]]] = []
        total = 0
        payload_total = 0
        for fixture in fixtures:
            total += fixture.bytes
            payload_total += fixture.payload_bytes
            state = "RETIRED" if fixture.retired else fixture.described
            print(
                f"  {fixture.name:<40} {fixture.files:>7} {fixture.bytes:>12} "
                f"{fixture.payload_bytes:>12} {fixture.age_hours:>7.2f}  {state}"
            )
            if fixture.retired or not fixture.payload_bytes:
                continue
            verdicts.append((fixture, refusals(fixture, repo_device, args.min_age_hours)))
        grand_bytes += total
        grand_payload += payload_total
        inert = len(fixtures) - len(verdicts)
        print(
            f"  {len(fixtures)} fixture(s), {total} bytes, {payload_total} of them payload"
            + (f"; {inert} already reduced or holding none" if inert else "")
        )
        for fixture, reasons in verdicts:
            if reasons:
                print(f"  KEEP {fixture.name}: " + "; ".join(reasons))
        ready = [f for f, reasons in verdicts if not reasons]
        reducible.extend(ready)
        print(f"  reducible: {len(ready)}; kept whole: {len(verdicts) - len(ready)}\n")

    recoverable = sum(f.payload_bytes for f in reducible)
    print(
        f"total {grand_bytes} bytes retained, {grand_payload} of them reproducible payload; "
        f"{len(reducible)} fixture(s) reducible now, recovering {recoverable} bytes"
    )

    if not args.retire:
        return 0
    if not args.confirm:
        print("--retire without --confirm: nothing was reduced")
        return 0

    reduced, dropped_bytes = retire_all(reducible, repo_device, args.min_age_hours)

    # Residue census: every fixture still stands, and every reduced one carries
    # its receipt with no payload left.
    residue_failures = []
    for fixture in reducible:
        if not fixture.path.is_dir():
            residue_failures.append(f"{fixture.name} no longer exists — this instrument never deletes")
            continue
        fresh = Fixture(fixture.path, fixture.population)
        if fresh.retired and fresh.payload_paths:
            residue_failures.append(f"{fixture.name} is marked retired but still holds payload")
    if residue_failures:
        for failure in residue_failures:
            print(f"RESIDUE CENSUS FAILED: {failure}", file=sys.stderr)
        return 1
    print(f"\nresidue verified: {reduced} fixture(s) reduced, {dropped_bytes} bytes dropped, 0 deleted")
    return 0


if __name__ == "__main__":
    sys.exit(main())
