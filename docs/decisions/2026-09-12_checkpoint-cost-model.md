answers: why does the full checkpoint take an hour; where does the checkpoint's unaccounted time go; why is local `make check` far slower than the Linux runner; why is the first run of a freshly built binary slow on this machine; how much does one more test binary cost

# The checkpoint's cost is a per-executable constant, not a workspace property

- **Type:** `decision`
- **Date:** `2026-09-12`
- **Status:** accepted; its two open levers are closed in `docs/decisions/2026-09-12_checkpoint-gate-authority.md`
- **Owner:** leaf `SIGNOFF-REPAIR.11.4.3.1.2.15`
- **Evidence:** `docs/tasks/artifacts/signoff_review/checkpoint-wall-time.md`
- **Instrument:** `scripts/measure_check_phases.py`

## Context

The full pre-push checkpoint's `02-check` command took 3,922 s while cargo's own
summaries accounted for 813 s of compilation and 127 s of test execution. The
remaining 2,982 s (76 %) was reported by nothing, and `SIGNOFF-REPAIR.11.5` made
explaining it the gate on every new verification lane: a gate whose cost nobody
can predict is a gate people route around.

## Decision

**The unaccounted time is macOS first-execution validation of newly written
executables on the repository volume, at a fixed ~21.9 s each, and it is
recorded as a cost model rather than repaired in this repository's sources.**

Measured, same bytes, same minute: **21.9 s mean on `/Volumes/SSD` against 0.15 s
on the boot volume**, ~150×, spread under 0.8 s over four pairs. The cost is
**fixed, not proportional to size** (81.6 MB costs 21.25 s; 2.3 MB costs 21.6 s),
is paid **once per file identity** and cached (second execution: 0.00 s), and the
process is blocked throughout (`user 0.00 sys 0.00`). Only Apple's own
`XProtect`/`syspolicyd` stack is present, and `com.apple.provenance` is
kernel-managed, so there is no file-level lever.

The model that follows, and that a reader can apply before adding work:

    checkpoint test time ≈ (distinct newly-written executables actually run) × ~21.9 s
                           + the harnesses' own time

**Adding one integration-test file to this workspace costs about 22 seconds of
every future checkpoint, whatever that file tests.** That is the number this
decision exists to publish.

## What was refuted

- **"Nine rustdoc doctest-harness builds"**, the leaf's own leading candidate:
  the doc phase costs 310 s of a 3,839 s run — at most 8 %, and 207 s of that is
  compilation cargo already reports.
- **"Inherent to `--all-features` clippy followed by `cargo test --all` on a
  thrashed cache"**, the leaf's framing of the alternative: the same commands on
  the Linux runner, from a COLD checkout, take **444 s with 18.7 s (4.2 %)
  unaccounted**. The runner compiles 514 crates from cold in less time than this
  machine compiles 12 warm.
- **The browser launcher's per-invocation download and verify**, which is the one
  structurally untimed segment of the log: its `version.log` is written ~20 s
  after clippy ends, so its setup is ~20 s, not ~50 minutes.

## Consequences

- `scripts/measure_check_phases.py` is tracked, so the number is re-derivable by
  one command instead of inferred from cargo's summaries. It also splits the
  doc/non-doc phases that `cargo test --all` hides.
- **The checkpoint's cost is not evidence that the gate is too big.** The same
  gate costs 7.5 minutes on the runner. Throughput decisions should not remove
  coverage on the strength of a number that is 8.8× local overhead.
- **CI policy §16 is reinforced rather than changed.** Full CI before push, a
  selected set per commit — the expensive local run is largely redundant with a
  remote gate that already runs the same commands far more cheaply, and remote CI
  has been green since run 34652116508.
- **Two levers were left open here and are now CLOSED** in
  `docs/decisions/2026-09-12_checkpoint-gate-authority.md` — the macOS setting is
  rejected (it exempts the shell from validating exactly the untrusted-content
  workers this project builds, and it cannot be committed), and the volume move is
  rejected on measured capacity (the repository is 3.77 MiB but its build tree is
  185 GB against 249 GB free on the boot volume). The lever actually pulled was a
  third one: the remote run, already a verified superset, becomes the authoritative
  pre-push gate. For the record, the two originally named were:
  1. a macOS security setting (the system's Developer Tools privacy category is
     the documented category for locally built software) — untested, because
     changing a system security setting is not an autonomous act, and it must be
     measured before it is believed;
  2. the repository's volume — §13 requires project data to live on the
     repository's volume, and this measurement is the first evidence that the
     choice of volume carries a large, previously invisible cost.
- **No gate was weakened, skipped or reordered.** The phase split runs strictly
  more than `make check` does.
- Compile durations remain cache-dependent and are not part of this claim. The
  per-executable constant is, and it is cache-independent by construction.
