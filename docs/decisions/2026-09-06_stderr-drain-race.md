# 2026-09-06_stderr-drain-race.md

## Context

`PHASE-1-MAINT-2` (tracked since the `.1.3.1` verification as an unreproduced
one-off `codex_adapter` failure under parallel load): during the `.1.5.3`
offline verification it reproduced with the failing test captured —
`nonzero_exit_produces_failed_known_with_the_stderr_tail` failed with an EMPTY
stderr tail (`codex exited with exit status: 2; stderr tail: `) where the stub's
"simulated provider error" belongs.

## Decision

- **The EOF path must await the stderr drain before snapshotting it.** The
  failure is a real race, not a port/filesystem collision: `drain_stderr` is a
  spawned task, and the EOF path read the buffer right after `child.wait()` —
  the task may not have consumed the pipe's tail yet. Load widens the window; it
  never creates it.
- **The drain returns its JoinHandle.** `drain_stderr` now returns
  `(Arc<Mutex<String>>, JoinHandle<()>)`; the EOF path awaits the handle
  (bounded at 5 s — a grandchild that inherited stderr could keep the pipe open)
  and only THEN snapshots the tail. Fixed in `codex.rs` AND its `claude.rs`
  mirror (the `.1.4` copy carried the identical race).

## Consequences

- The `FailedKnown` reason now carries the provider's stderr tail
  deterministically; the 10× adapter-suite loop (both suites) and the full
  offline workspace are green. `PHASE-1-MAINT-2` closed.

answers:

- **A flake that reproduces once is a bug with evidence, not a mystery.** The
  first failure (`.1.3.1`) had no test name; the `.1.5.3` run captured it —
  `--nocapture` was never the fix, the missing NAME was. Every future defect
  leaf records the failing test name from the first report.
- **Spawned consumers need a join point before their buffer is read.** Any
  "spawn a drainer, read its buffer at EOF" supervisor pattern has this race by
  construction; the join (bounded) is the fix, not a longer sleep.
- **Mirrors inherit defects.** The `claude.rs` copy of the `.4.2` pattern had
  the identical race; the fix lands in both, and future mirrors must diff
  against the fixed template, not the original.
