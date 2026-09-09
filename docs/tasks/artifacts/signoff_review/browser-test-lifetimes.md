# Browser verification origin and worker lifetimes

- Owner: `SIGNOFF-REPAIR.11.4.3.1.5.1`; REPAIR-0038.
- Baseline: 1df463d. Production crates/reasonbraid-browse/src/main.rs is unchanged in this child.
- Status: eight final controls, strict focused lint and independent process/source/residue checks pass; all results consumed. Original failure evidence remains retained.
- Raw evidence: target/browser-lifetime-controls.

## Source and installed inputs

The original browser test drops its origin JoinHandle, reads worker stdout without
a byte/time limit, discards stderr and waits without a bound. Its no-browser guard
also skips the budget test even though budget admission precedes browser launch.
The new child is test-side containment; production worker profile, launch/render/
shutdown ownership remains .5.2 and combined qualification .5.3.

Read-only inventory finds installed Chrome 152.0.7977.83 on the OS volume. The
preexisting on-volume worker is 35,890,096 bytes with SHA-256
6304017f988ea0e7cb9765893e9848b72ac6db3ece9261dd879eb41182b75dd5. No
.project-data/tmp/chromiumoxide-runner or target/browser-tests directory existed.
Inventory metadata is retained in inventory.json; this is availability evidence,
not successful browser execution. Installed Chrome/Rust/Python and OS inspection
are necessary read-only dependencies; owned fixtures and outputs stay on volume.

The pinned chromiumoxide 0.9.1 source defaults its profile to
TMPDIR/chromiumoxide-runner. The official
[Chromium user-data documentation](https://raw.githubusercontent.com/chromium/chromium/main/docs/user_data_dir.md)
explains custom profile/cache derivation on macOS and Linux: with the chosen test
profile outside standard home/config application directories, its cache follows
that profile. The harness sets each worker's TMPDIR/TMP/TEMP and XDG/Chrome config,
cache, data and state variables to its private fixture. It never repurposes HOME
or touches an interactive browser profile. The initial official googlesource fetch
timed out; the official GitHub source mirror supplied the documentation.

Pinned axum 0.8.9 with_graceful_shutdown stops accepting, drops the listener and
awaits all connection receiver lifetimes before completion. Consuming a successful
serve task therefore qualifies origin shutdown. A timed-out/aborted origin does
not count as a confirmed graceful shutdown and retains its fixture.

## Exact old-helper probe

Extract run_worker unchanged from 1df463d into a generated standalone Rust probe,
with the same read/write/process imports. Compile it against an existing local
serde_json library and point CARGO_BIN_EXE_reasonbraid-browse at an exclusively
owned synthetic Python worker. The synthetic worker consumes input then emits
exactly 3 MiB of stdout; no browser or actual external origin is started.

The first compiled-probe invocation hit its fifteen-second external timeout.
Its execution phase was not measured, so do not label it a loader wait or a
confirmed helper deadlock. subprocess.run consumed/killed the direct probe; the
follow-up exact-path process census found neither probe nor synthetic worker.
The initial comm-based census self-matched the inspection shell because macOS
truncated comm; its row and corrected zero-result census remain preserved.

Retry the unchanged binary under run_pg_tests.run_command with an explicit
120-second process-group supervisor. It returns rc=0 and reports exactly 3,145,728
captured bytes; its process group is absent after consumed cleanup. This reproduces
the old helper's lack of a 2 MiB stream bound. baseline-root.json records the
exclusive source/library location; baseline-retry-process.json retains binary hash
and supervisor identity; baseline-verification.json records the actual result and
the original timeout's limits. No timeout phase or OS root cause is inferred.

## Implemented and qualified harness contract

Each test exclusively creates a private UUIDv7 fixture under validated on-volume
parent directories. It owns one command and preserves its directory identity.
The worker has a new process group at spawn. Input writes, both output streams and
the process wait share one deadline; close the actual stdin writer to deliver EOF.
Each captured stream is limited to 2 MiB plus one detection byte. A normal browser
command gets forty-five seconds; synthetic controls use explicit shorter budgets.

After every result, consume the direct child and require its whole group absent.
Observable remaining groups receive a TERM request, then a KILL request after a
five-second grace interval; reaping and final group absence each have five-second
bounds. An initially uncertain group observation can take another five seconds.
Permission refusal never counts as absence; details follow below. stdout.log,
stderr.log and worker.json preserve bounded diagnostic data and cleanup status.
If cleanup is unconfirmed, retain the fixture. A test-side forced cleanup is
reported explicitly and cannot establish the production worker's shutdown claim.
An emergency Drop only requests kill and makes no reaping claim. Terminal receipts
are also printed into the aggregate log; the final group_stop_requested field
names a requested action, not proof that its signal caused the group to disappear.

Rendering retains its visible absent-browser skip, while an explicitly configured
invalid browser is exercised as a failure. Budget admission always runs, without
requiring browser availability. Existing render/network/digest/refusal assertions
run inside a caught test future so origin shutdown is consumed before assertion
failure is rethrown. Successful cases delete only their checked owned fixture;
failed or unconfirmed cases retain it. Added controls exercise worker timeout,
stdout/stderr overflow, a live stalled descendant and listener shutdown. The final eight-test suite and strict focused lint pass; exact evidence follows.

Cargo.lock adds only reasonbraid-browse edges to already locked Rustix and UUID;
Tokio process/I/O/sync features are explicit test requirements. No package version
or production worker behavior is changed by this test harness.

## Darwin teardown finding and correction

The initial six-control suite passes in 7.01s. Its real render reports a remaining
worker group after output and exit; the supervisor requests cleanup. The next
receipt-enabled run passes five controls but refuses the render cleanup: worker
PID 38697 exited zero with 458 stdout bytes, yet signal-zero group inspection
returns EPERM. The harness preserves case-01a08785-b3c6-71c1-9d01-257f1e9181d8
with group_cleanup_confirmed=false. A later real census finds no members/profile
processes and group inspection returns ESRCH. Keep final-tests.log/.exit and the
original fixture; no persistent permission boundary or exact Chrome member state
can be inferred retrospectively.

A controlled native probe on Darwin 25.6.0 reproduces a relevant mechanism with
one same-user child deliberately left unreaped: PID 77123, UID 501, state Z. The
direct-PID probe succeeds while its group probe returns EPERM. After wait consumes
exit zero, group inspection returns ESRCH. A second owned probe verifies that
TERM and KILL group requests also return EPERM while that child is a zombie, then
reaping establishes ESRCH. Both probes are consumed and their groups absent;
darwin-zombie-group.json and darwin-zombie-signals.json preserve observations.

The inspected [official XNU group-signal implementation](https://raw.githubusercontent.com/apple-oss-distributions/xnu/main/bsd/kern/kern_sig.c)
filters SZOMB entries during group iteration and selects EPERM when no eligible
member is counted in POSIX mode. This published-source mechanism matches the
controlled native result; it does not establish which member state caused the
original Chrome observation or identify the running kernel with that source revision.

The helper previously failed immediately on an observation that could resolve
during teardown. Reap the direct child first, then allow bounded EPERM observation
retries. A signal-request refusal also requires subsequent strict observation;
it never qualifies cleanup. Only actual absence completes the cleanup contract.
Persistent denial remains an error and retains data. Two deterministic controls
verify that a transient denial needs another observation and a persistent denial
fails at its deadline. The first correction build caught an incomplete local
variable rename (E0425), before running any tests; preserve that log and correct
the identifier. No runtime or permission assertion was weakened.

## Final executed evidence

- `python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-browse --test browser_roundtrip -- --nocapture`: eight controls pass in 7.02s, zero failed/ignored/skipped, rc=0. group-final-tests.log/.exit retain all six terminal worker receipts. Real rendering/network/refusal, both 3 MiB stream refusals at 2 MiB + 1 captured byte, stalled worker/descendant cleanup, origin listener closure and both group-observation controls pass.
- `python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-browse --test browser_roundtrip --all-features -- -D warnings`: final strict lint passes in 8.86s, rc=0. group-final-clippy.log/.exit retain the result. Workspace format passes; no production worker source changed.
- Final receipts show all six groups confirmed absent and all six successful fixture directories removed. The real render's receipt records worker exit zero, 458 output bytes and group_stop_requested=true; budget refusal has 88 bytes and no stop request. This precisely establishes supervisor assistance, not an indefinitely leaked process or proof that a signal caused disappearance. Production ownership remains .5.2.
- Independent native signal-zero checks find all six groups absent; a current-UID command census finds no completed-case profile processes. Exactly the original failed case remains. final-process-verification.json preserves the scope and result; it is not a universal cross-user process census.
- Parsed lockfile comparison shows only the two already-locked test dependency edges, with no package/version changes. Production main.rs and README are byte-identical to 1df463d. final-verification.json retains final source hashes, all six receipts and exact scope. make book and ten rendered contract markers pass, rc=0; staged doctrines run in the commit hook.

All compile/test/lint/probe results are consumed. Preserve the initial timeout,
first cleanup refusal, corrected-build error and successful final evidence. Finish
production profile/process/task ownership .5.2 and combined qualification .5.3
before compiler-artifact disposition .6 and the full checkpoint .2. No full CI,
project security gate, remote execution or push is claimed by this child.
