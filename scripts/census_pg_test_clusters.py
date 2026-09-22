#!/usr/bin/env python3
"""Census — and, on explicit request, retire — the disposable PostgreSQL clusters
the test runner retains after a failure (`SIGNOFF-REPAIR.11.4.3.1.7`).

`scripts/run_pg_tests.py` removes its workspace on success and RETAINS it on
failure, so `target/pg-tests/run-*` accumulates one cluster per failed run. That
retention is deliberate — it is the diagnostic evidence — and it is also why the
directory grows without bound until somebody looks.

This is the instrument, not a remembered procedure. `docs/CLAIM_VERIFICATION.md`
leg 3: a disposition whose producer is untracked cannot be re-taken, and the
periodic review in `CLAUDE.md` §8 happens more than once.

    python3 -B scripts/project_env.py python3 -B scripts/census_pg_test_clusters.py
    python3 -B scripts/project_env.py python3 -B scripts/census_pg_test_clusters.py --retire --confirm

⛔ Retirement requires EVERY one of these to hold for a cluster, checked at the
moment of deletion rather than read off this census:

  - its receipt says `state: stopped` — the runner verified shutdown. A
    `shutdown-unverified` receipt is never retired, because a process may still
    own those bytes;
  - no live process holds its `postmaster.pid`;
  - the directory is a real directory (not a symlink) on the REPOSITORY's own
    volume, so nothing outside the repository's storage is ever touched;
  - no TRACKED file names it. A retained cluster a task tree cites by name is
    evidence, and evidence is not disposable;
  - it is older than `--min-age-hours` (default 1), so a run that is still being
    read is left alone;
  - it has NOT already been reduced to its evidence. `SIGNOFF-REPAIR.7.3.2.1`'s
    `scripts/census_retained_fixtures.py` applies the other disposition — keep
    the receipt, drop the reproducible PGDATA payload — and leaves a
    `retired.json` behind. A reduced cluster is receipts and configuration with
    the bulk already gone, so removing it here would destroy the very evidence
    that reduction was performed to preserve, and recover almost nothing.

A cluster failing any check is REPORTED and kept. The exit code is the verdict:
0 when the requested operation completed, 1 when something was refused.

⛔⛔ AND A CLUSTER OUTSIDE `target/pg-tests` USED TO BE INVISIBLE, NOT KEPT
(`SIGNOFF-REPAIR.11.4.3.1.9.1`). This census iterated that one directory, so
three retained clusters at the top of `target/` — `pg-iter.5Im1bN`,
`pg-iter.DiTCe8`, `pg-ephemeral.5nSEfx` — were never judged and never mentioned.
⭐ The difference matters because silence reads exactly like a clean verdict:
`.11.4.3.1.9` printed `194 cluster(s) remain` and was telling the truth about
`target/pg-tests` alone (`docs/knowledge/an-instruments-zero-describes-its-reach.md`).

⛔ THE STRAY SCAN IS DERIVED FROM WHAT `initdb` WRITES, never from a list of
directory-name prefixes. A PostgreSQL data directory contains `PG_VERSION`, so a
cluster is any directory holding `data/PG_VERSION` — an oracle this instrument
did not build. A prefix list would be a second copy of the producers' spellings
and would drift exactly as `SCAFFOLD-COVERAGE`'s `NEUTRAL` list did; it would
also have missed `pg-iter.*`, whose spelling NO tracked producer has ever
written (`git log -S 'pg-iter' -- .` returns documentation commits only).

⚠️ A STRAY IS ANNOUNCED AND NEVER AUTO-RETIRED, and that is the honest verdict
rather than a timidity. Retirement requires a receipt proving the postmaster was
stopped; a stray has no `runner.json`, because the producer that wrote it did
not write receipts, so the guard cannot prove the bytes are unowned. ⛔ Removing
it would be substituting a hand-run subset of these checks for the guard, which
is the improvisation this instrument exists to replace. The census reports it,
its size and its citation count, and a human decides.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLUSTERS = ROOT / "target" / "pg-tests"
PREFIX = "run-"
# Written by scripts/census_retained_fixtures.py when it reduces a cluster to
# its evidence. Its presence means the bulk is already gone.
REDUCED = "retired.json"
# What `initdb` writes into every PostgreSQL data directory. This is the stray
# scan's whole population rule: a cluster is a directory holding `data/PG_VERSION`.
DATA_DIR = "data"
PG_MARKER = "PG_VERSION"
# How far below `base` a stray is looked for. Depth 2 covers a cluster left at
# the top of `target/` and one left one level inside another directory, and it
# is BOUNDED deliberately: an unbounded walk would descend `target/debug`, which
# is 69 GB and hundreds of thousands of files, to find nothing.
STRAY_DEPTH = 2


def is_cluster_dir(path: Path) -> bool:
    """A directory is a PostgreSQL cluster when it holds `data/PG_VERSION`."""
    try:
        return (path / DATA_DIR / PG_MARKER).is_file()
    except OSError:
        return False


def stray_clusters(base: Path, modelled: Path, depth: int = STRAY_DEPTH) -> list[Path]:
    """Every cluster directory under `base` that `modelled` does not contain.

    Bounded by `depth` and pruned at `modelled`, so the scan costs a handful of
    directory listings rather than a walk of the build tree.
    """
    found: list[Path] = []
    frontier = [(base, 0)]
    while frontier:
        directory, level = frontier.pop()
        try:
            children = sorted(p for p in directory.iterdir() if p.is_dir() and not p.is_symlink())
        except OSError:
            continue
        for child in children:
            if child == modelled:
                continue  # the modelled population is censused in full elsewhere
            if child.name == DATA_DIR:
                continue  # a cluster's own payload, never a cluster itself
            if is_cluster_dir(child):
                found.append(child)
                continue  # do not descend into a cluster we have already named
            if level + 1 < depth:
                frontier.append((child, level + 1))
    return sorted(found)


class Cluster:
    def __init__(self, path: Path) -> None:
        self.path = path
        self.name = path.name
        self.receipt: dict = {}
        self.receipt_error: str | None = None
        receipt = path / "runner.json"
        if receipt.is_file():
            try:
                self.receipt = json.loads(receipt.read_text())
            except (OSError, ValueError) as error:  # a truncated receipt is a fact
                self.receipt_error = str(error)
        self.files = 0
        self.bytes = 0
        for dirpath, _dirs, names in os.walk(path):
            for name in names:
                try:
                    stat = os.lstat(os.path.join(dirpath, name))
                except OSError:
                    continue
                self.files += 1
                self.bytes += stat.st_size
        self.age_hours = (time.time() - path.stat().st_mtime) / 3600.0

    @property
    def state(self) -> str:
        return str(self.receipt.get("state", "unknown"))

    @property
    def command(self) -> str:
        return str(self.receipt.get("command", ""))

    @property
    def exit_code(self):
        return self.receipt.get("command_exit")

    def live_postmaster(self) -> int | None:
        """The pid in `postmaster.pid` when that process is still alive."""
        pid_file = self.path / "data" / "postmaster.pid"
        if not pid_file.is_file():
            return None
        try:
            pid = int(pid_file.read_text().splitlines()[0])
        except (OSError, ValueError, IndexError):
            return None
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return None
        except PermissionError:
            return pid  # alive and owned by somebody else: never ours to remove
        return pid


def tracked_references(name: str) -> int:
    """How many TRACKED files name this cluster. A cited cluster is evidence."""
    result = subprocess.run(
        ["git", "grep", "-c", "--", name],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode not in (0, 1):
        raise SystemExit(f"git grep failed for {name}: {result.stderr.strip()}")
    return len([line for line in result.stdout.splitlines() if line.strip()])


def refusals(cluster: Cluster, repo_device: int, min_age_hours: float) -> list[str]:
    reasons: list[str] = []
    if cluster.receipt_error is not None:
        reasons.append(f"unreadable receipt ({cluster.receipt_error})")
    if cluster.state != "stopped":
        reasons.append(f"receipt state is {cluster.state!r}, not 'stopped'")
    pid = cluster.live_postmaster()
    if pid is not None:
        reasons.append(f"postmaster pid {pid} is alive")
    stat = os.lstat(cluster.path)
    if not os.path.isdir(cluster.path) or os.path.islink(cluster.path):
        reasons.append("not a real directory")
    elif stat.st_dev != repo_device:
        reasons.append(f"device {stat.st_dev} is not the repository's {repo_device}")
    if cluster.age_hours < min_age_hours:
        reasons.append(f"age {cluster.age_hours:.2f}h is under the {min_age_hours}h floor")
    references = tracked_references(cluster.name)
    if references:
        reasons.append(f"named by {references} tracked file(s) — it is cited evidence")
    if (cluster.path / REDUCED).is_file():
        reasons.append(
            f"already reduced to its evidence ({REDUCED}) — removing it would destroy "
            "the receipts that reduction preserved"
        )
    return reasons


def self_test() -> int:
    """Fire every refusal on purpose. A guard never seen refuse is not known to work."""
    import shutil
    import tempfile

    repo_device = os.stat(ROOT).st_dev
    scratch = ROOT / "target" / "pg-cluster-census-selftest"
    scratch.mkdir(parents=True, exist_ok=True)
    failures: list[str] = []

    def build(name: str, receipt: dict, *, pid: int | None = None) -> Cluster:
        path = Path(tempfile.mkdtemp(prefix=f"{name}-", dir=scratch))
        (path / "runner.json").write_text(json.dumps(receipt))
        if pid is not None:
            (path / "data").mkdir()
            (path / "data" / "postmaster.pid").write_text(f"{pid}\n")
        # Age it past the floor so only the property under test can refuse.
        old = time.time() - 3600 * 5
        os.utime(path, (old, old))
        return Cluster(path)

    def expect(label: str, cluster: Cluster, *, fragment: str | None, min_age: float = 1.0) -> None:
        reasons = refusals(cluster, repo_device, min_age)
        if fragment is None:
            if reasons:
                failures.append(f"{label}: expected no refusal, got {reasons}")
        elif not any(fragment in reason for reason in reasons):
            failures.append(f"{label}: expected a refusal mentioning {fragment!r}, got {reasons}")

    try:
        expect("a stopped, aged, uncited cluster", build("clean", {"state": "stopped"}), fragment=None)
        expect(
            "an unverified shutdown",
            build("unverified", {"state": "shutdown-unverified"}),
            fragment="not 'stopped'",
        )
        expect("a missing receipt", build("noreceipt", {}), fragment="not 'stopped'")
        expect(
            "a live postmaster",
            build("live", {"state": "stopped"}, pid=os.getpid()),
            fragment="is alive",
        )
        expect(
            "a young cluster",
            build("young", {"state": "stopped"}),
            fragment="under the",
            min_age=1e9,
        )
        reduced = build("reduced", {"state": "stopped"})
        (reduced.path / REDUCED).write_text('{"rule": "keep the receipt"}')
        # Re-age: writing the marker refreshed the mtime, and an arm should be
        # refused by the property under test and nothing else.
        old_time = time.time() - 3600 * 5
        os.utime(reduced.path, (old_time, old_time))
        expect("an already-reduced cluster", Cluster(reduced.path), fragment="already reduced")

        broken = build("broken", {"state": "stopped"})
        (broken.path / "runner.json").write_text("{not json")
        expect("an unreadable receipt", Cluster(broken.path), fragment="unreadable receipt")

        # The citation guard, in BOTH directions, against the real tracked tree.
        #
        # `SIGNOFF-REPAIR.11.4.3.1.7.1`: the absent-name probe is GENERATED, never
        # written as a literal. It used to be the literal
        # "run-selftest-no-such-cluster-name", which this very file contains and
        # `git grep` searches — so the arm passed exactly once, while the file was
        # still untracked, and failed from the instant it was committed. A probe
        # written down here cannot be absent from here. A fresh uuid4 can.
        absent = f"run-selftest-{uuid.uuid4().hex}"
        if tracked_references("run_pg_tests.py") == 0:
            failures.append("citation guard: a tracked filename reported zero references")
        if tracked_references(absent) != 0:
            failures.append(f"citation guard: the absent name {absent} reported references")
        # ---- `SIGNOFF-REPAIR.11.4.3.1.9.1`: the stray scan, in BOTH directions ----
        # 🔴 The founding defect: a cluster outside the modelled directory was not
        #   kept, it was INVISIBLE, and this census's totals read as if they covered
        #   `target/`. The arms below are on a synthetic base, so they judge the
        #   SCAN rather than whatever happens to be on this machine's disk today.
        base = Path(tempfile.mkdtemp(prefix="straybase-", dir=scratch))
        modelled = base / "pg-tests"
        (modelled / "run-inside" / DATA_DIR).mkdir(parents=True)
        (modelled / "run-inside" / DATA_DIR / PG_MARKER).write_text("16\n")
        outside = base / "pg-ephemeral.AAAAAA"
        (outside / DATA_DIR).mkdir(parents=True)
        (outside / DATA_DIR / PG_MARKER).write_text("16\n")
        nested = base / "scratchdir" / "pg-iter.BBBBBB"
        (nested / DATA_DIR).mkdir(parents=True)
        (nested / DATA_DIR / PG_MARKER).write_text("16\n")
        # ⭐ NEGATIVE: a directory with no marker is not a cluster, whatever it is
        #   called. Without this the scan degenerates into "any directory".
        decoy = base / "pg-ephemeral.NOTACLUSTER"
        (decoy / DATA_DIR).mkdir(parents=True)

        seen = {p.name for p in stray_clusters(base, modelled)}
        if "pg-ephemeral.AAAAAA" not in seen:
            failures.append("stray scan: a cluster beside the modelled directory must be announced")
        if "pg-iter.BBBBBB" not in seen:
            failures.append("stray scan: a cluster one level deeper must be announced")
        if "run-inside" in seen:
            failures.append("stray scan: the modelled population must be pruned, not double-reported")
        if "pg-ephemeral.NOTACLUSTER" in seen:
            failures.append("stray scan: a directory with no data/PG_VERSION is not a cluster")
        # ⭐ NEGATIVE: the depth bound is real. An unbounded walk would descend the
        #   build tree, so a cluster below the bound is deliberately NOT found — the
        #   arm pins the limit instead of leaving it to be discovered.
        deep = base / "a" / "b" / "pg-iter.CCCCCC"
        (deep / DATA_DIR).mkdir(parents=True)
        (deep / DATA_DIR / PG_MARKER).write_text("16\n")
        if "pg-iter.CCCCCC" in {p.name for p in stray_clusters(base, modelled)}:
            failures.append("stray scan: the depth bound is not being applied")
        if "pg-iter.CCCCCC" not in {p.name for p in stray_clusters(base, modelled, depth=3)}:
            failures.append("stray scan: a deeper bound must reach a deeper cluster")
    finally:
        shutil.rmtree(scratch, ignore_errors=True)

    for failure in failures:
        print(f"SELF-TEST FAILED: {failure}", file=sys.stderr)
    if failures:
        return 1
    print(
        "self-test: 7 refusal arms — including a cluster already reduced to its evidence by "
        "scripts/census_retained_fixtures.py — the citation guard's two directions, and 6 stray-scan "
        "arms (announced beside and below the modelled directory, the modelled population pruned, a "
        "marker-less directory refused, and the depth bound pinned in both directions) all fire"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="fire every refusal against synthetic clusters and exit",
    )
    parser.add_argument(
        "--retire",
        action="store_true",
        help="remove every cluster that passes all checks (requires --confirm)",
    )
    parser.add_argument(
        "--confirm",
        action="store_true",
        help="perform the removal; without it --retire only reports what it would do",
    )
    parser.add_argument(
        "--min-age-hours",
        type=float,
        default=1.0,
        help="leave clusters younger than this alone (default 1)",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    if not CLUSTERS.is_dir():
        print(f"pg-test clusters: {CLUSTERS.relative_to(ROOT)} does not exist — nothing retained")
        return 0

    repo_device = os.stat(ROOT).st_dev
    clusters = sorted(
        (Cluster(path) for path in CLUSTERS.iterdir() if path.is_dir() and path.name.startswith(PREFIX)),
        key=lambda c: c.name,
    )
    if not clusters:
        print("pg-test clusters: none retained")
        return 0

    print(f"{'cluster':<20} {'state':<20} {'exit':>5} {'files':>7} {'bytes':>12} {'age_h':>7}  suite")
    total = 0
    verdicts: list[tuple[Cluster, list[str]]] = []
    for cluster in clusters:
        total += cluster.bytes
        suite = cluster.command.split()[-1] if cluster.command else "?"
        print(
            f"{cluster.name:<20} {cluster.state:<20} {str(cluster.exit_code):>5} "
            f"{cluster.files:>7} {cluster.bytes:>12} {cluster.age_hours:>7.2f}  {suite}"
        )
        verdicts.append((cluster, refusals(cluster, repo_device, args.min_age_hours)))
    print(f"\n{len(clusters)} cluster(s), {total} bytes")

    retirable = [c for c, reasons in verdicts if not reasons]
    kept = [(c, reasons) for c, reasons in verdicts if reasons]
    for cluster, reasons in kept:
        print(f"KEEP {cluster.name}: " + "; ".join(reasons))
    print(f"\nretirable: {len(retirable)}; kept: {len(kept)}")

    # ⛔ Announce every cluster OUTSIDE the modelled directory. Until
    # `SIGNOFF-REPAIR.11.4.3.1.9.1` these were not kept — they were invisible,
    # and this census's own totals read as though they covered `target/`.
    strays = stray_clusters(ROOT / "target", CLUSTERS)
    if strays:
        print(f"\nSTRAY: {len(strays)} cluster(s) outside {CLUSTERS.relative_to(ROOT)} — announced, never auto-retired")
        for path in strays:
            stray = Cluster(path)
            cites = tracked_references(path.name)
            print(
                f"  {path.relative_to(ROOT)}  {stray.bytes} bytes  "
                f"{stray.age_hours:.1f}h  tracked citations {cites}  "
                f"receipt {'present' if (path / 'runner.json').is_file() else 'ABSENT'}"
            )
        print(
            "  ⛔ No receipt means no proof the postmaster stopped, so the guard cannot\n"
            "     judge these and does not remove them. Reported for a human decision."
        )
    else:
        print(f"\nSTRAY: none — every cluster under target/ is inside {CLUSTERS.relative_to(ROOT)}")

    if not args.retire:
        return 0
    if not args.confirm:
        print("--retire without --confirm: nothing was removed")
        return 0

    removed_bytes = 0
    for cluster in retirable:
        # Re-check immediately before removing: this census may be minutes old.
        again = refusals(cluster, repo_device, args.min_age_hours)
        if again:
            print(f"KEEP {cluster.name} (changed since the census): " + "; ".join(again))
            continue
        subprocess.run(["rm", "-rf", "--", str(cluster.path)], check=True)
        removed_bytes += cluster.bytes
        print(f"removed {cluster.path.relative_to(ROOT)} ({cluster.bytes} bytes)")

    residue = [p.name for p in CLUSTERS.iterdir() if p.name.startswith(PREFIX)]
    expected = sorted(c.name for c, reasons in verdicts if reasons)
    if sorted(residue) != expected:
        print(
            "RESIDUE CENSUS FAILED: expected "
            f"{expected} to remain, found {sorted(residue)}",
            file=sys.stderr,
        )
        return 1
    print(f"\nresidue verified: {len(residue)} cluster(s) remain, {removed_bytes} bytes removed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
