# Explicit state-lock release

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.11.2`; REPAIR-0056. Predecessor:
`4c40abb5050a4e949ced48c7445d70bcd417ec1c`. Original diagnosis:
`docs/tasks/artifacts/signoff_review/state-writer-lock-lifetime.md`. Raw repair:
`target/state-writer-lock-controls/repair`.

## Root cause and bounded correction

Publication retained an ordinary File. File close alone cannot release flock
while an inherited/duplicated open-file description survives. The preceding
actual-API fork matrix proves retention across success, HTTP error and cancelled
future, with matched no-child and child-exit controls. It does not recover the
original full-checkpoint holder, whose fixture was deleted during panic cleanup.

A private StateLock now owns that File immediately after successful acquisition,
before later identity validation or directory synchronization can fail. Its Drop
explicitly unlocks the shared description, retries interrupted unlock syscalls,
and then lets File close. An unexpected OS unlock refusal retains File-close
fallback. Publication owns the guard across HTTP and every intermediate durable
snapshot. Nonblocking acquisition, CLOEXEC, directory/lock identity validation,
encoding, snapshot ordering and synchronization are unchanged. No public API,
wire format, dependency, manifest, server or database-schema change.

A source census finds two other successful direct Rust lock lifetimes in the CLI
tests: state_writers::lock_is_held and the explicit-principal test's held File.
Both now explicitly unlock before closing, so concurrent child creation cannot
turn a completed test probe into another false writer. All existing assertions
and default test concurrency remain unchanged. The independent Python lock
holders acquire after exec and spawn no descendants.

## Permanent regression and red baseline

One new private unit control exercises five paths: successful publication,
encoding failure, pre-replacement publication failure, discard and unwind. An
owned Python child receives a duplicate of the real lock File through a standard
stream and witnesses its exact device/inode. Its readiness/readers/termination
are bounded and consumed. The original guard remains exclusive while active,
including after a refused same-process contender. Each path then attempts a new
writer while the old child still holds its description. Consume the child before
asserting the desired result, and verify that its exit does not unlock a newly
acquired successor. Exact state bytes remain unchanged except after success.

The control runs first against byte-reconstructed unchanged production. It fails
0/1, recording all five retained outcomes, after all five children are released
and reaped (24.650542 seconds command; 0.36 seconds body). After repair the same
assertions pass (24.587245 command; 0.32 body). Only the private wrapped-file
access changes. This negative baseline falsifies the old close-only behavior;
active contention and successor exclusion prevent replacing it with no locking.

The initial complete selected run passes 32 distinct Rust tests under default
concurrency: twelve CLI library, four bootstrap-state, four publication and twelve
writer controls (114.113150 seconds command). The fixture companion then passes
all twelve writer controls again (46.595986 seconds). Final all-target/all-feature
CLI Clippy returns zero in 13.992059 seconds; final format returns zero in 0.500096.
The earlier focused/affected runs repeat the new regression; these are 45 final
successful Rust test executions, not 45 distinct tests. The red baseline is separate.

## Independent public-API raw-fork check

Preserve the original probe source/executable. Rebuild it against the selected
CLI with only one desired assertion changed: after-operation contention must be
absent even with an inherited child. All observation, descriptor witness, state
and cleanup code remains byte-identical. The build passes in 37.511292 seconds;
all six scenarios pass in 0.721499 seconds.

| Public API outcome | No child | Inherited child still alive | Snapshot |
| --- | --- | --- | --- |
| Success | lock available | lock available | new thread published |
| HTTP error | lock available | lock available | original bytes unchanged |
| Future cancellation | lock available | lock available | original bytes unchanged |

Every scenario first observes valid live-writer exclusion and later verifies the
parent descriptor has closed. All three inherited children exit zero and are
reaped; lock acquisition remains available after their exit. This probes actual
CLI behavior on the native macOS volume, independently of the private-unit
standard-stream transfer. It is not a Linux execution or a process-crash test.

## Reproduction and evidence boundaries

```bash
python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --lib writer_release_does_not_wait_for_an_inherited_descriptor -- --nocapture
python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --lib --test bootstrap_state --test state_publication --test state_writers -- --nocapture
python3 -B scripts/project_env.py cargo clippy -p reasonbraid-cli --all-targets --all-features --locked -- -D warnings
```

The durable regression is part of the existing CLI library target. Source
reconstruction confirms its baseline assertions and all existing writer assertions
are preserved, with 339 other non-Markdown files unchanged. Nineteen frozen
historical evidence records, original binaries and all old probe snapshots remain
unchanged. An initial source-verifier comparison omitted rustfmt's line break
before unwrap; correcting that expected string required no product change.

Explicit Drop cannot run after abrupt process death. A surviving inherited
reference can still retain a lock then; the broader CLI interruption/restart leaf
.3.3.4.3.3.3.3.3 owns that concrete boundary and safe recovery policy. Never unlink
state.lock to bypass contention, claim server rollback from a local outcome, or
silently terminate unrelated same-user processes. The original checkpoint holder
remains uncaptured. This bounded repair does not promise universal crash release,
a fully passing checkpoint, completed roadmap or zero remaining defects.

Final native process/source/book verification and per-leaf commit evidence are
recorded in the owning task tree. Resume the complete checkpoint only from the
committed repair, preserving both earlier failed checkpoints and their payloads.

Final verification returns zero: all 62 recorded checkpoint/diagnostic/repair
groups are absent, nineteen frozen evidence records and original binary/snapshot
copies are unchanged, and eight rendered markers pass. Book returns zero in
0.150580 seconds. README remains 52 lines/2017 bytes and LIVE_STATUS categories
are unchanged. All results are consumed before the per-leaf commit.
