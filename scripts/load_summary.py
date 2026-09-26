#!/usr/bin/env python3
"""The load harness's summary (SIGNOFF-REPAIR.11.3.4): what `scripts/load_harness.sh`
publishes, computed from its latencies file so the numbers re-derive from it.

    load_summary.py LATENCIES REQUESTED WORKERS_FAILED WALL_SECONDS

Each latencies line is `<http status> <seconds>`, one per request; curl writes
`000` for a request that never got an answer (a timeout, a refused connection).

⛔ The percentiles are over the COMMITTED requests only (status 200), and the
failures are counted beside them. The harness used to take p50/p95 over every
line, so a fast refusal pulled the published latency down.

The run PASSES only when the file holds exactly REQUESTED lines, every one a 200,
and every worker exited cleanly. Exit 0 = pass, 1 = a failed run (the summary is
still printed), 2 = unusable input.
"""

from __future__ import annotations

import math
import sys
from pathlib import Path


def nearest_rank(sorted_values: list[float], percent: int) -> float:
    """The nearest-rank percentile (1-based rank ceil(p/100 · n))."""
    rank = max(1, math.ceil(percent / 100 * len(sorted_values)))
    return sorted_values[rank - 1]


def summarize(lines: list[str], requested: int, workers_failed: int, wall: float) -> tuple[bool, list[str]]:
    if requested < 1:
        raise ValueError(f"{requested} requested commands measure nothing")
    committed: list[float] = []
    failed = 0
    for number, line in enumerate(lines, 1):
        fields = line.split()
        if len(fields) != 2:
            raise ValueError(f"latencies line {number} is not '<status> <seconds>': {line!r}")
        status, seconds = fields[0], float(fields[1])
        if status == "200":
            committed.append(seconds)
        else:
            failed += 1
    total = len(committed) + failed
    committed.sort()
    report = [f"load harness: {total} of {requested} requested commands ran in {wall:.3f}s"]
    if committed:
        report.append(
            f"  ingress→commit over the {len(committed)} committed: "
            f"p50 {nearest_rank(committed, 50):.4f}s  p95 {nearest_rank(committed, 95):.4f}s"
        )
    else:
        report.append("  ingress→commit: no committed request, so no percentile")
    report.append(f"  throughput: {len(committed) / max(wall, 1e-9):.1f} committed commands/s  "
                  f"failures: {failed}  workers failed: {workers_failed}")
    passed = True
    if total != requested:
        report.append(f"FAIL: {total} of {requested} commands ran")
        passed = False
    if failed:
        report.append(f"FAIL: {failed} commands did not commit with 200")
        passed = False
    if workers_failed:
        report.append(f"FAIL: {workers_failed} workers did not exit cleanly")
        passed = False
    if passed:
        report.append("PASS: every requested command committed (200) and the summary is recorded")
    return passed, report


def main(argv: list[str]) -> int:
    if len(argv) != 5:
        print(__doc__, file=sys.stderr)
        return 2
    try:
        lines = [l for l in Path(argv[1]).read_text(encoding="utf-8").splitlines() if l.strip()]
        passed, report = summarize(lines, int(argv[2]), int(argv[3]), float(argv[4]))
    except (OSError, ValueError) as error:
        print(f"load_summary: {error}", file=sys.stderr)
        return 2
    for line in report:
        print(line, file=sys.stdout if not line.startswith("FAIL") else sys.stderr)
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
