# Whole CLI writer exclusion

Owner: SIGNOFF-REPAIR.3.3.4.3.3.3.3.1.2. Product baseline: 8b2a8d9.
Status: whole-writer integration qualified; nineteen selected controls, final all-target CLI strict lint and book checks pass. Pending request/recovery remains separate.

## Boundary under qualification

StateFile::save now locks and atomically replaces a complete snapshot, but
run_enroll and run_thread_create perform HTTP before separate load/modify/save.
A busy local save can therefore return an error after dispatching an effect.
main.rs also resolves a named create actor before the writer acquires its lock.
The current leaf owns one held store across fresh selection, HTTP and publication.
Existing caller-supplied PrincipalRef behavior remains a distinct explicit input.

The baseline starts the real rb binary twice against an owned plaintext loopback
HTTP fixture while independent Python acknowledges flock on the same state.lock.
It measures actual request count for both verbs, as well as final status and exact
unchanged state bytes. A CLI error alone is insufficient evidence of pre-dispatch
refusal. Mock responses are complete enough for the current client; this fixture
measures HTTP dispatch, not authoritative server commit or bootstrap recovery.

Command: `python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --test state_writers -- --nocapture --test-threads=1`.
All fixture data/logs are under target/cli-writer-controls on the repository
volume. Installed Python/Rust/OS metadata are necessary read-only dependencies.
The process helper bounds captured output, records child PIDs and kills/waits
on its deadline. Normal completion consumes child results, holder kill/wait and
HTTP shutdown. Unique fixture cleanup follows shutdown; fallback drops preserve
bounded process lifetime during a failed control. No existing fixed live-fixture
residue is deleted by this new baseline.

Following controls qualify both writer orders, process loss and fresh merged
state. The owned real-server cli_end_to_end suite remains the compatibility gate;
its two fixed delete-at-start paths and unbounded process lifetime are owned for
specific fixture repair before the run. Pending request/reply/restart protocol
remains the following .3.2/.3.3 children, never inferred from local exclusion.

Baseline returns rc=101, 0/1 passed (build 7.70s, execution 5.45s). Both real
CLI processes exit unsuccessfully with the expected local lock error, and the
state bytes remain identical, but the fixture observes two HTTP requests rather
than zero. Child PIDs 60795 and 60952 finish and are consumed; the holder is
killed/waited and HTTP shutdown is joined before the desired assertion fails.
This proves the existing error is too late to exclude dispatch. Product changes
begin only after consuming that result.

The first candidate's library binary remained before test output at PID 67626
(parent Cargo 66863), observed age 49s/CPU 0.00s. A one-second sample contains
803 stacks at _dyld_start, footprint 96 KiB. This proves a loader-start wait in
that sampled interval, not a CLI lock deadlock or a native trust-store call.
Sample: target/cli-writer-controls/library-start.sample. The underlying host
cause remains the existing .11.2 owner; no host settings change or speed claim.
The verification result still must be consumed; sampling is not completion.

## Bounded HTTP follow-up

Source-level risk: ApiClient::new constructs Reqwest Client::new, and pinned
0.12.28 async_impl/client.rs sets connect_timeout/read_timeout/timeout to None
(lines 299/313/314). Holding a local store across an unbounded HTTP wait can
retain exclusion until cancellation. The pending-request child .3.2 now owns a
stalled-response reproduction, explicit request limits and durable key retention
on timeout. No runtime reproduction or completed timeout repair is claimed here.
This is a tracked repair, not a reason to bypass a live writer's OS lock.

## Initial repaired controls and live compatibility

The candidate returns rc=0: eight library, four state-publication and four
writer controls (build 24.22s; execution 0.96s/0.14s/2.07s). The original held-lock
control now observes zero HTTP requests. Both overlap orders observe the first
writer's OS lock while its HTTP response is withheld, refuse the second with
exactly one pending request, then preserve both principal/thread maps after
an explicit later operation (two total requests). Both killed-writer cases
observe held then released exclusion with exact original state bytes and a
successful different operation. No mock server-commit/replay claim is made.
The named-actor control verifies actual header identity and explicit tenant
selection without changing the stored principal's tenant; it does not claim a
runtime reproduction of the former pre-lock actor-selection race.

All-target CLI strict lint returns rc=0 in 20.85s. Both results are consumed.
The final pass adds a public resolved-principal entrypoint control: busy refusal
before HTTP, usage-error release and deliberate explicit identity preservation.
The final test/lint results are consumed below.

`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py cli_end_to_end`
returns rc=0, both real-server controls pass (build 7.48s, execution 13.52s).
The cluster target/pg-tests/run-hhhksw26, database rb_test_070d95e77a353ec6f0823601,
port 57671 is stopped and removed by the runner; result consumed. The full flow
and typed denials use the actual rb binary/API/PostgreSQL, retaining the existing
behavior checks. Fixture roots are unique, child output is bounded to 1 MiB per
stream with a 90s completion deadline and kill/wait recovery, HTTP server tasks
are aborted/joined and pools closed before normal fixture removal. No old fixed
workspace is deleted. The final residue census finds the cluster and every unique writer/e2e/storage fixture absent.

## Final qualification

`python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --lib --test state_publication --test state_writers -- --nocapture --test-threads=1`
returns rc=0, 8+4+5 controls (build 6.05s; execution 1.37s/0.13s/24.87s).
Four writer controls exercise the actual rb process; the fifth preserves the
public resolved-principal API, busy refusal, usage-error release and explicit
identity. Together with the two live PostgreSQL/CLI compatibility controls,
nineteen selected controls pass. Each result is consumed.

Final `python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-cli --all-targets -- -D warnings`
returns rc=0 in 11.67s; result consumed. Formatting/diff and book checks pass.
The initial rendered-marker check caught mdBook smart punctuation converting a
plain --as flag to an en-dash; inline code preserves literal --as/--tenant flags.
Fourteen final rendered contract markers, the chapter link and one parsed JSON
example pass. No full CI or push. The final escalated process census returns rc=0,
handoff: OK; result consumed, with only three inherited-CWD session processes
and zero repository handles. No project-owned background result remains.

Retained evidence SHA-256 (target/cli-writer-controls):

- baseline.log: `7d140ed95437034fb4b623f537a77641a690b75ea001ec6d2c436eb1003ded1b` (1212 bytes).
- baseline.exit: `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c` (4 bytes).
- candidate.log: `d638409e144f1dd2efe19de63fffa30694016b5fbffdb82ffc1676555cae7186` (4044 bytes).
- candidate.exit: `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` (2 bytes).
- lint.log: `b71843ee66d243798d4f52f9e2398a3d2ef78b0c23418bc10f9fb14df56fdfdf` (176 bytes).
- lint.exit: `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` (2 bytes).
- live-cli.log: `6f39f43d702be978db5f1255b304949792d7a4e446e2aa471e236bc2b3e3b589` (715 bytes).
- live-cli.exit: `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` (2 bytes).
- final.log: `b9954b8c351a83765b04174bd1d4c19f07bd252b45480f324af5a4f9b62c2d7c` (4130 bytes).
- final.exit: `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` (2 bytes).
- final-lint.log: `c14fe81d97bb34196f1dffdfb67d3a064fbb82c15c8e2c6a3e89caadb4cb3547` (176 bytes).
- final-lint.exit: `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` (2 bytes).
- library-start.sample: `aebb4bb83cb35a41fd39058f01cc513937ac87f0a52f7cd05887d580007eabee` (1013 bytes).
