#!/usr/bin/env python3
"""Snapshot a stalled child WHILE it is stuck (`SIGNOFF-REPAIR.11.26.1`).

⭐ WHY THIS EXISTS. `SIGNOFF-REPAIR.11.26` has localized its failure precisely —
the `download` phase, 8 of 8, ~15 s to start a small local child that normally
costs ~200 ms — and has run out of levers. Three were produced deliberately and
none reproduced it: repeated `cargo build`s writing new executables, a fresh
191 MB signed browser bundle (~2,000 new Mach-O files), and a co-tenant driving
the 1-minute load average to 17.37. Two explanations were WITHDRAWN under
measurement — *the machine was busy*, *the policy daemon is busy*. The leaf's own
narrowing says the next move is not a fourth lever: it is to sample the child
while it is stuck.

⛔ AND THE EVIDENCE CANNOT BE COLLECTED AFTER THE FACT. `run_pg_tests.run_command`
kills the process group in its `finally`, so by the time `subprocess.TimeoutExpired`
reaches the caller the child is already gone. The snapshot has to fire from a
watchdog BEFORE the deadline, which is why this is a module and not a procedure.

⚠️ AND THE STALLED THING IS THE GRANDCHILD. The test runs `python -B <fixture>`,
and the `download` phase is that fixture spawning a stub `curl` — a freshly
written 0o700 script in a fresh directory. A stack of the direct child would show
a process waiting on its own child, so the tree is walked and the DEEPEST
descendant is sampled.

⛔ THE PAGING QUANTITY IS A RATE, NOT A TOTAL, and that is the whole point of
taking two readings. `.11.26` measured `vm.swapusage` at an identical
5,719 / 7,168 MB during a failing run AND during a run that passed in 21.3 s —
swap *used* is a high-water mark and cannot discriminate. `vm_stat`'s `Pageins`,
`Swapins` and friends are cumulative counters, so the discriminating quantity is
their difference over a measured interval. That is what `paging_rate` returns.

⚠️ A LIMIT OF THIS INSTRUMENT, RECORDED BEFORE IT IS USED. `sample` is itself an
`exec`. If the state genuinely makes starting any new process cost seconds, the
sampler may be slow or may stall too — and `/usr/bin/sample` is a long-resident
system binary rather than a freshly written one, which is exactly the distinction
the leaf's surviving candidate turns on. The asymmetry is informative either way:
a sampler that runs fast while a fresh child hangs is evidence FOR the
first-run-evaluation shape; a sampler that also hangs refutes it. Every capture
therefore records its own elapsed time.

⛔ THIS DIAGNOSES; IT NEVER DECIDES. Nothing here raises a bound, retries, or
changes what the suite asserts. `.11.26` measured 15,000 ms at 8.4x the worst of
240 normal observations, so the bound is not the defect and picking another
number is `SIGNOFF-REPAIR.11.6`'s prohibition.

Usage:
    python3 -B scripts/stall_snapshot.py --self-test
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

# The counters worth a rate. Cumulative since boot, so only their DIFFERENCE
# over a measured interval means anything.
PAGING_COUNTERS = ("Pageins", "Pageouts", "Swapins", "Swapouts",
                   "Compressions", "Decompressions")

PS_FIELDS = ("pid", "ppid", "state", "pcpu", "etime", "command")


def _run(argv: list[str], timeout: float) -> str:
    """A diagnostic subprocess. Never raises — a snapshot that fails is a
    snapshot that says so, not one that replaces the failure it was capturing."""
    try:
        done = subprocess.run(argv, capture_output=True, text=True, timeout=timeout)
        return done.stdout
    except (OSError, subprocess.SubprocessError) as error:
        return f"<{type(error).__name__}: {error}>"


def parse_ps(text: str) -> list[dict[str, str]]:
    """`ps -ax -o pid,ppid,state,pcpu,etime,command` into rows.

    ⚠️ Split on whitespace exactly `len(PS_FIELDS) - 1` times, so a command
    containing spaces stays whole. The fixture this instrument exists for runs
    `python -B -c '…' 'argument with spaces'`.
    """
    rows: list[dict[str, str]] = []
    for line in text.splitlines()[1:]:
        parts = line.split(None, len(PS_FIELDS) - 1)
        if len(parts) < len(PS_FIELDS):
            continue
        row = dict(zip(PS_FIELDS, parts))
        if not row["pid"].isdigit() or not row["ppid"].isdigit():
            continue
        rows.append(row)
    return rows


def process_tree(rows: list[dict[str, str]], root: int) -> list[dict[str, str]]:
    """`root` and every descendant, breadth-first, with a `depth` on each.

    ⛔ Visited-set guarded. A ppid table read from a live system can contain a
    pid that has been reused, which is a cycle; an unguarded walk would hang
    inside the instrument that exists to diagnose a hang.
    """
    children: dict[int, list[dict[str, str]]] = {}
    for row in rows:
        children.setdefault(int(row["ppid"]), []).append(row)
    found: list[dict[str, str]] = []
    seen: set[int] = set()
    frontier = [(row, 0) for row in rows if int(row["pid"]) == root]
    while frontier:
        row, depth = frontier.pop(0)
        pid = int(row["pid"])
        if pid in seen:
            continue
        seen.add(pid)
        found.append({**row, "depth": str(depth)})
        for child in children.get(pid, []):
            if int(child["pid"]) not in seen:
                frontier.append((child, depth + 1))
    return found


def deepest(tree: list[dict[str, str]]) -> int | None:
    """The pid furthest from the root — the one actually doing the work.

    Ties break on the LAST such row, which is the most recently listed and so
    the most recently started of equals.
    """
    if not tree:
        return None
    best = max(tree, key=lambda row: int(row["depth"]))
    candidates = [row for row in tree if row["depth"] == best["depth"]]
    return int(candidates[-1]["pid"])


def parse_vm_stat(text: str) -> dict[str, int]:
    counters: dict[str, int] = {}
    for line in text.splitlines():
        name, _, value = line.partition(":")
        name = name.strip()
        if name in PAGING_COUNTERS:
            digits = value.strip().rstrip(".")
            if digits.isdigit():
                counters[name] = int(digits)
    return counters


def paging_rate(first: dict[str, int], second: dict[str, int],
                seconds: float) -> dict[str, float]:
    """Per-second rates from two cumulative readings.

    ⛔ Returns the DIFFERENCE over the interval, never the totals. `.11.26`
    measured an identical swap total during a failing run and a passing one; a
    high-water mark cannot discriminate and a rate can.
    """
    if seconds <= 0:
        return {}
    return {name: round((second[name] - first[name]) / seconds, 1)
            for name in sorted(set(first) & set(second))}


def capture(pid: int, out_dir: Path, *, label: str = "stall",
            interval: float = 1.0, sample_seconds: int = 2,
            runner=_run, clock=time.monotonic) -> dict:
    """Everything worth having about a stalled `pid`, while it is still stalled."""
    began = clock()
    out_dir.mkdir(parents=True, exist_ok=True)
    stamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
    record: dict = {"label": label, "pid": pid, "captured_at": stamp}

    before = parse_vm_stat(runner(["vm_stat"], 10))
    ps_text = runner(["ps", "-ax", "-o", ",".join(PS_FIELDS)], 15)
    tree = process_tree(parse_ps(ps_text), pid)
    record["tree"] = tree
    record["tree_size"] = len(tree)

    target = deepest(tree)
    record["sampled_pid"] = target
    if target is not None:
        stack_path = out_dir / f"{label}-{stamp}-{target}.sample.txt"
        sample_began = clock()
        stack = runner(["/usr/bin/sample", str(target), str(sample_seconds),
                        "-mayDie"], sample_seconds + 30)
        # ⚠️ The sampler's OWN cost is the asymmetry that makes it evidence: a
        # fast sampler beside a hung fresh child supports the first-run
        # evaluation shape; a slow one refutes it.
        record["sample_elapsed_ms"] = round((clock() - sample_began) * 1000)
        stack_path.write_text(stack, encoding="utf-8")
        record["sample_path"] = str(stack_path)

    remaining = interval - (clock() - began)
    if remaining > 0:
        time.sleep(remaining)
    elapsed = clock() - began
    after = parse_vm_stat(runner(["vm_stat"], 10))
    record["paging_rate_per_s"] = paging_rate(before, after, elapsed)
    record["paging_interval_s"] = round(elapsed, 3)
    record["capture_elapsed_ms"] = round(elapsed * 1000)
    return record


def watchdog(pid_holder, out_dir: Path, after_seconds: float, **kwargs):
    """A timer that snapshots `pid_holder()` if it is still running.

    Returns a `(timer, records)` pair; the caller cancels the timer on the happy
    path, so a child that finishes normally costs one cancelled timer and no
    subprocesses at all.
    """
    import threading
    records: list[dict] = []

    def fire():
        pid = pid_holder()
        if pid is None:
            return
        try:
            os.kill(pid, 0)
        except OSError:
            return  # already gone; nothing to sample
        try:
            records.append(capture(pid, out_dir, **kwargs))
        except Exception as error:  # a diagnostic must never mask its subject
            records.append({"pid": pid, "capture_failed": repr(error)})

    timer = threading.Timer(after_seconds, fire)
    timer.daemon = True
    return timer, records


SELF_TEST_PS = """  PID  PPID STAT  %CPU     ELAPSED COMM
    1     0 Ss     0.0 15-21:39:49 /sbin/launchd
  100     1 S      0.0       01:02 /usr/bin/python3 -B runner.py
  200   100 S      0.0       00:31 /usr/bin/python3 -B fixture.py
  300   200 U      0.0       00:15 /tmp/fresh dir/curl --disable
  400     1 S      0.0       09:09 /usr/sbin/unrelated
"""

SELF_TEST_VM_A = """Mach Virtual Memory Statistics: (page size of 16384 bytes)
Pages free:                                     4200.
Pageins:                                         100.
Pageouts:                                         10.
Swapins:                                           5.
Swapouts:                                          7.
Compressions:                                     20.
Decompressions:                                   30.
"""

SELF_TEST_VM_B = SELF_TEST_VM_A.replace("Pageins:                                         100.",
                                        "Pageins:                                         400.")


def self_test() -> int:
    failures: list[str] = []

    rows = parse_ps(SELF_TEST_PS)
    if len(rows) != 5:
        failures.append(f"ps rows miscounted: {len(rows)}")
    # ⭐ The load-bearing parse case: a command containing SPACES must stay one
    # field. The fixture this exists for runs `… 'argument with spaces'`, and a
    # naive split would truncate the command and mis-key nothing visibly.
    spaced = [r for r in rows if r["pid"] == "300"]
    if not spaced or spaced[0]["command"] != "/tmp/fresh dir/curl --disable":
        failures.append(f"a command containing spaces was split: {spaced}")

    tree = process_tree(rows, 100)
    if [r["pid"] for r in tree] != ["100", "200", "300"]:
        failures.append(f"the tree is not the descendants of 100: {tree}")
    if any(r["pid"] == "400" for r in tree):
        failures.append("an unrelated process entered the tree")
    # ⛔ The GRANDCHILD is the point: sampling the direct child would show a
    # process waiting on its own child, not the stall.
    if deepest(tree) != 300:
        failures.append(f"the deepest descendant is not the grandchild: {deepest(tree)}")

    # A ppid cycle must not hang the instrument that diagnoses hangs.
    cyclic = parse_ps("""  PID  PPID STAT  %CPU     ELAPSED COMM
   10    11 S      0.0       00:01 a
   11    10 S      0.0       00:01 b
""")
    if len(process_tree(cyclic, 10)) != 2:
        failures.append("a ppid cycle was not terminated")

    before, after = parse_vm_stat(SELF_TEST_VM_A), parse_vm_stat(SELF_TEST_VM_B)
    if len(before) != len(PAGING_COUNTERS):
        failures.append(f"not every paging counter parsed: {sorted(before)}")
    rate = paging_rate(before, after, 2.0)
    # ⛔ A RATE, not a total: 300 pageins over 2 s is 150/s, and the absolute
    # 400 must not appear anywhere. `.11.26` measured an identical swap TOTAL
    # during a failing and a passing run, which is why this distinction is the
    # instrument's reason to exist.
    if rate.get("Pageins") != 150.0:
        failures.append(f"the paging rate is not a per-second delta: {rate}")
    if rate.get("Pageouts") != 0.0:
        failures.append(f"an unchanged counter is not a zero rate: {rate}")
    if paging_rate(before, after, 0) != {}:
        failures.append("a zero interval produced a rate")

    # ⛔ A diagnostic that fails must SAY so rather than replace the failure it
    # was capturing, so the real runner swallows its own errors into the record.
    if not _run(["/nonexistent-tool-for-the-self-test"], 5).startswith("<"):
        failures.append("a failing diagnostic subprocess did not degrade into a note")

    calls: list[list[str]] = []

    def fake(argv, timeout):
        calls.append(argv)
        if argv[0] == "vm_stat":
            return SELF_TEST_VM_A if len(calls) < 3 else SELF_TEST_VM_B
        if argv[0] == "ps":
            return SELF_TEST_PS
        return "fake stack"

    import tempfile
    with tempfile.TemporaryDirectory(dir=Path(__file__).resolve().parent.parent / "target") as tmp:
        record = capture(100, Path(tmp), interval=0.0, sample_seconds=1, runner=fake)
        if record["sampled_pid"] != 300:
            failures.append(f"capture sampled the wrong pid: {record['sampled_pid']}")
        if record["tree_size"] != 3:
            failures.append(f"capture recorded the wrong tree: {record}")
        if not Path(record["sample_path"]).is_file():
            failures.append("the stack file was not written")
        if "sample_elapsed_ms" not in record:
            failures.append("the sampler's own cost was not recorded")
        if not any(c[0] == "/usr/bin/sample" for c in calls):
            failures.append(f"the sampler was never invoked: {calls}")

    # ── THE FIRING PATH, END TO END, ON A REAL STALLED CHILD ──
    # `SIGNOFF-REPAIR.11.26.1`'s acceptance: prove the snapshot fires on a
    # deliberately stalled child rather than waiting for the rare event. A real
    # `sleep` is spawned, a watchdog is armed well inside its lifetime, and the
    # record must name that pid. ⚠️ Whether `/usr/bin/sample` is PERMITTED here
    # is environmental, so the arm asserts the record and the tree — not the
    # stack's contents, which would make the control fail for an unrelated reason.
    with tempfile.TemporaryDirectory(dir=Path(__file__).resolve().parent.parent / "target") as tmp:
        stalled = subprocess.Popen(["/bin/sleep", "30"])
        try:
            timer, records = watchdog(lambda: stalled.pid, Path(tmp), 0.2,
                                      interval=0.2, sample_seconds=1)
            timer.start()
            timer.join(60)
            if len(records) != 1:
                failures.append(f"the watchdog did not fire on a stalled child: {records}")
            elif records[0].get("pid") != stalled.pid:
                failures.append(f"the snapshot named the wrong pid: {records[0]}")
            elif records[0].get("tree_size", 0) < 1:
                failures.append(f"the snapshot found no process tree: {records[0]}")
            elif "paging_rate_per_s" not in records[0]:
                failures.append(f"no paging rate was recorded: {records[0]}")
        finally:
            stalled.kill()
            stalled.wait()

        # ⭐ And the complement, which is what keeps an ordinary run free: a child
        # that has already exited produces NO capture and spawns no subprocess.
        gone = subprocess.Popen(["/bin/sleep", "0"])
        gone.wait()
        timer, records = watchdog(lambda: gone.pid, Path(tmp), 0.05)
        timer.start()
        timer.join(30)
        if records:
            failures.append(f"a finished child was still sampled: {records}")

    for failure in failures:
        print(f"SELF-TEST FAILED: {failure}", file=sys.stderr)
    if failures:
        return 1
    print(
        "stall_snapshot --self-test: a ps command containing SPACES is proved to "
        "stay one field, the tree is proved to be the descendants of the named pid "
        "and no unrelated process, the GRANDCHILD is proved to be what gets sampled "
        "(sampling the direct child would show a process waiting on its own child), "
        "a ppid cycle is proved to terminate rather than hang the instrument that "
        "diagnoses hangs, and the paging figure is proved to be a per-second DELTA "
        "rather than the cumulative total — the distinction `.11.26` needs, because "
        "an identical swap total was measured during both a failing and a passing run; "
        "and the firing path is proved END TO END on a real stalled child — a live "
        "`sleep` is snapshotted and named, while a child that has already exited is "
        "proved to produce no capture and spawn no subprocess at all"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--pid", type=int, help="snapshot this pid now")
    parser.add_argument("--out", default="target/stall-snapshots")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.pid is None:
        parser.error("give --pid or --self-test")
    root = Path(__file__).resolve().parent.parent
    print(json.dumps(capture(args.pid, root / args.out), indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
