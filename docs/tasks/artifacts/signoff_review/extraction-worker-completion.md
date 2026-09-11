# Direct extraction worker completion

Owner: `SIGNOFF-REPAIR.7.3.3.2.2`; REPAIR-0060. Predecessor:
`97fe885ce17ea32565df744b6308993e0577091d` (REPAIR-0059, clean tree).
Raw evidence: `target/extraction-completion-controls`.

## Preserved baseline identities

The originals were hashed before the first edit
(`target/extraction-completion-controls/original-identities.json`), from a clean
tree at `97fe885`:

| Preserved original | Bytes | SHA-256 |
| --- | --- | --- |
| `crates/reasonbraid-server/src/extraction.rs` | 9,196 | `c0128310ccc4d4537d6ac0773367714a4d6196322489943fdd59d9dedfa419c4` |
| `crates/reasonbraid-server/src/api.rs` | 284,707 | `228eb342846a792e59b2be81e2c8eef3c28cd7d70cd5e1d604735c17b6b51452` |
| `crates/reasonbraid-extract/src/main.rs` | 32,031 | `3dfb3e5c2916489b17434cb19a23e7124e08c1f0ab7927371d457adcc6247a14` |
| `crates/reasonbraid-server/Cargo.toml` | 7,416 | `165f2e00e6ccab1110978755440d8a21f4c360de3ebffdcafda58834ba879bbc` |
| `Cargo.lock` | 127,550 | `eaada411d35d870dbc12cac6ed8cafa24b9c9c2f7df837f04187e181df90cc81` |
| `target/debug/reasonbraid-extract` (the real worker) | 11,523,336 | `85e9cefefe3691e413732adeeacd83209708b0dc86c341c2d7a92175d094b358` |

## The reproduction is the permanent control, run on unchanged production

The eight process-fact controls in
`crates/reasonbraid-server/tests/extraction_completion.rs` drive the REAL
`run_extraction` against controlled repository-local `/bin/sh` workers selected
through the documented `R2_WORKER_BIN` override, then ask the operating system
whether the exact direct child this process created is still in the process
table. A reaped child has no entry; a killed-but-unreaped child is still a
zombie entry, so one observation covers stop AND reap.

Against the unchanged 9,196-byte `extraction.rs`
(`target/extraction-completion-controls/baseline.json`):

```text
python3 -B scripts/project_env.py cargo test --locked --offline \
  -p reasonbraid-server --test extraction_completion -- --nocapture
FAILED. 7 passed; 1 failed; 0 ignored; finished in 46.10s
an_early_request_failure_never_abandons_a_live_worker:
  pid 39486 was still in the process table after the spawner returned:
  the early failure abandoned its direct worker
```

The controlled worker records its own identity, closes its stdin and then waits
on an explicit release marker with a hard 30-second deadline. A media type of
8 MiB forces the actual write path: the spawner blocks past the pipe buffer and
the closed reader turns that write into a real `EPIPE`. No clock was mocked, no
parser changed, and no HTTP or database path was executed.

Seven controls already passed before the repair — absent worker, unspawnable
worker, successful exchange, named refusal, nonzero exit, unparseable reply and
the tripped time budget. The failure is therefore pinpointed, not general.

## Root cause

`run_extraction` propagated the request-write failure and the `try_wait`
failure with `?` while its `std::process::Child` was still owned by the frame.
Dropping a `Child` in Rust neither waits nor signals, so the direct worker kept
running with nobody waiting for it. The timeout path did call `kill` then
`wait`, but discarded both results and used an unbounded `wait`, so a worker
that ignored the signal could hold the caller open indefinitely and a failed
kill was indistinguishable from a confirmed one. Two `expect` calls on the
piped handles could also panic inside the control plane.

## The repair

`crates/reasonbraid-server/src/extraction.rs` only (+393/−22 lines; api.rs
unchanged, 284,707 bytes, same SHA-256 as above):

- `exchange` performs the request/response with the already-spawned child and
  returns on failure; it never owns the stop.
- Every return path from `run_extraction_reporting` passes through ONE bounded
  `stop_and_reap`. Phase one gives a worker whose stdin is already closed a
  500 ms grace to finish on its own; phase two asks the OS to terminate it and
  waits out a 5-second total budget. A tripped time budget passes a zero grace,
  so the killing budget stays the quarantine's enforcement.
- `WorkerCompletion` records what was actually observed: `NeverStarted`,
  `Consumed { success, status }`, or `Unconfirmed { pid, detail }`. A failed
  stop request is recorded in the evidence and the wait continues; it never
  becomes a claimed termination.
- `ExtractionError` keeps its exact five variants and their classification.
  Only the `TimedOut` message changed, dropping the words "(the worker was
  killed)" — that outcome is now the separate completion evidence.
- The two `expect` calls on the piped handles became typed `RequestFailed`
  refusals.
- `run_extraction` keeps its established signature and delegates, so the R2 API
  caller is untouched; `run_extraction_reporting` is the completion-bearing
  entrypoint that `SIGNOFF-REPAIR.7.3.3.3` must migrate onto before it can make
  input cleanup depend on the reader having finished.

## Verified after the repair

| Control set | Command | Result |
| --- | --- | --- |
| 16 process/evidence controls | `cargo test -p reasonbraid-server --test extraction_completion` | 16 passed, 5.02s |
| 6 spawner controls (4 injected, 1 zero-grace, 1 real worker) | `cargo test -p reasonbraid-server --lib extraction` | 6 passed, 0.04s |
| 81 server library tests | `env -u DATABASE_URL … --lib` | 81 passed, 10.79s |
| 12 adjacent extractor tests | `cargo test -p reasonbraid-extract --all-targets` | 12 passed |
| strict lint | `cargo clippy -p reasonbraid-server --all-targets --locked --offline -- -D warnings` | rc=0, 1m57s |
| format | `cargo fmt --all -- --check` | rc=0 |

The four `Unconfirmed` controls are SYNTHETIC injection against the private
`DirectChild` seam: no cooperative worker reproduces an unkillable process or an
unreadable status on demand, so that classification is proved rather than
claimed. They are explicitly not native reproductions.

## Residue and honest limits

- The retained failed baseline fixture
  `target/extraction-completion-controls/fixtures/abandoned-39410-0` is
  preserved. Pid 39486 left the process table after its release marker, and a
  literal `ps` census for `extraction-completion-controls` is empty: no control
  worker survived either run.
- This is evidence about ONE process — the direct child. Descendants that child
  may have started are untouched by it. Synchronous pipes, total deadlines,
  descendant containment and aggregate retained storage remain
  `SIGNOFF-REPAIR.7.3.4`; the spawner still drains stdout only after the child
  exits, so a worker writing past the pipe buffer still deadlocks into the time
  budget. That is unchanged and separately owned.
- Exclusive same-volume inputs, source/response digest binding and
  cleanup-only-with-confirmed-completion remain `SIGNOFF-REPAIR.7.3.3.3`. The
  R2 API still names its input under `std::env::temp_dir()`, still uses the
  PID/`subsec_nanos` name and still removes that path unconditionally.
- The adjacent in-module spawner test still writes its own input under
  `std::env::temp_dir()`. That fixture's input ownership is `.7.3.3.3`'s wiring
  step, not this leaf's.
- The browse worker's own `BrowseError::TimedOut` message still asserts "(the
  worker was killed)". `SIGNOFF-REPAIR.7.3.1` owns that parent transport and
  termination surface; it was not touched here.
- No HTTP handler, database write, full CI run or public push is claimed by
  this leaf.
