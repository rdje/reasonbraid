# Keyed CLI bootstrap and explicit recovery

Owner: SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.2. Product baseline: b8dd415.
Status: matched baseline consumed; implementation and qualification in progress.

## Matched dispatch baseline

`python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --test state_writers bootstrap_dispatch_has_a_durable_matching_pending_request -- --nocapture --test-threads=1`
returns rc=101, 0/1 passed (build 1m06s; execution 0.91s). At the actual owned
HTTP gate the local state remains version one, recovery is absent and the
request has no bootstrap_request_id, despite the process lock being held.
The request completes successfully and the child/server results are consumed
before asserting the missing recovery metadata. The desired assertion fails
on its absent pending record; this is the missing pre-dispatch intent, not a
lock failure. Unique writer fixtures are absent before product edits.

The fixture echoes a key only when actually supplied; it does not synthesize
one for the old client. The HTTP gate observes dispatch, not authoritative
PostgreSQL commit. Real-server compatibility and broader server interruption
qualification retain their separately named checks/owners.

- target/cli-bootstrap-controls/flow-baseline.log: 1030 bytes, SHA-256 6cff874e4b9fc7b7f58b7d896d2478b4e6dce044b6856f1731f2ee439f8b32e7.
- target/cli-bootstrap-controls/flow-baseline.exit: 4 bytes, SHA-256 39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c.

## Candidate controls and exact observed output window

`python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --lib --test bootstrap_state --test state_publication --test state_writers -- --nocapture --test-threads=1`
returns rc=0: 11 library, four recovery-schema, four publication and eleven real
writer controls pass (build 16.84s; execution 2.00s / 1.54s / 0.29s / 34.00s).
All results are consumed. The strict response control includes 37 malformed
original-byte cases and two valid historical-source/replayed controls. The schema
retains 53 malformed-record preservation cases. The matched HTTP gate now sees
version two, saved recovery, an actual matching request key and a held lock.

Four successive failed HTTP outcomes (500, duplicate key, mismatched source and
missing grant) preserve exact pending bytes and old maps. All five dispatches,
including successful recovery, carry the same key and original ignored actions.
Explicit completion recovery restores its historical mapping without HTTP,
preserves historical replayed, and reports recovery_source=local_receipt. A
normal subsequent invocation has a distinct key. Both writer overlap orders,
independent lock refusal, named/explicit actor controls and process-loss release
remain green; interrupted bootstrap recovers its pending request before another
writer. Invalid resume modes and changed endpoint/name refuse without effects.

The owned stdout experiment uses a 98,304-byte name. After principal/receipt
publication and pending cleanup, the real rb process remains alive while its
unread pipe holds 65,536 bytes at both observations separated by 200ms. The exact
child is killed/reaped; the reader receives only 65,536 bytes, not a complete JSON
result. Explicit resume returns the original recorded key/principal/name with
local_receipt and no additional HTTP (total requests=1). This proves recoverable
output uncertainty under measured unread-output backpressure, not user receipt,
physical power-cut survival or an authoritative database commit in this fixture.
The later fixture refinement bounds the post-kill reap and stream reads too;
its focused final rerun and real-server checks are recorded separately below.

`python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-cli --all-targets -- -D warnings`
initially returns rc=0 in 4m08s, result consumed. It precedes the additional live
restoration control and final stream-bound refinement, so final lint remains
required. Product behavior is unchanged by subsequent conditional-import cleanup.

An observed pre-test publication wait ends normally and all four tests pass.
The attempted one-second sample returns rc=255 because the exact process already
exited; it supplies no stack or causal evidence. A later guarded compiler sample
finds no unique live match and takes no sample. Neither attempt changes host
settings, signatures, attributes or any unrelated process; no OS root-cause claim
is made. Existing underlying host investigation remains .11.2.

## Owned capacity follow-up

A source-level serialization model of the current pretty JSON snapshot uses one
large existing thread subject and a three-byte bootstrap name. Against the
8,388,608-byte limit, pending is 8388544 bytes (64 bytes spare), while
principal plus completed receipt with pending retained is 8389340 bytes
(732 bytes over). The Python model preserves the current ASCII field
layout; it is not represented as an actual Rust codec or HTTP runtime result.
The real-client reproduction and pre-dispatch completion-capacity fix are owned
by .3.3.4.3.3.3.3.2.3, ahead of deadline integration. Retaining a key alone does
not prove the current snapshot can hold its outcome. Evidence is
target/cli-bootstrap-controls/flow-capacity-source.json.

## Final source, real-server and lifecycle evidence

`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py cli_end_to_end`
returns rc=0, all three live controls pass (build 3m55s; execution 25.21s).
The new real-client control reads the actual stored outcome independently,
restores its original pending request into a separate owned store and obtains
the same server IDs with replayed=true. Explicit original-store recovery returns
the historical local receipt with its original replayed=false. Database counts
remain one tenant/request after both recoveries, then become two after deliberate
fresh invocation; all generated identifiers differ and the first outcome is
unchanged. Existing whole-flow and typed-denial controls remain green. This is
an explicit restoration test, not actual response/commit interruption. The
runner stops/removes target/pg-tests/run-86ov3v3c; result/shutdown consumed.

`python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --test state_writers losing_the_process_with_unread_output_recovers_the_completed_receipt -- --nocapture --test-threads=1`
returns rc=0, 1/1 pass on the final bounded-reap/stream fixture (build 3m06s
including build-lock waiting; execution 1.21s). The same measured 65,536-byte
unread/partial window, live process and single HTTP request pass. Result consumed.

`python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-cli --all-targets -- -D warnings`
finally returns rc=0 in 38.74s; result consumed. Together thirty non-database plus
three live controls pass, with the final output fixture requalified separately.
Formatting and diff checks pass. Unique writer/e2e/record/state/unit/sync-probe
fixtures and the owned PostgreSQL cluster are absent. No full CI or push.
Final book checks and escalated handoff census remain below.

Retained evidence (all paths under target/cli-bootstrap-controls):

- flow-baseline.log: 1030 bytes; SHA-256 `6cff874e4b9fc7b7f58b7d896d2478b4e6dce044b6856f1731f2ee439f8b32e7`.
- flow-baseline.exit: 4 bytes; SHA-256 `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c`.
- flow-candidate.log: 6788 bytes; SHA-256 `0c58c03e7c39d579d4d35117eb744c9126f2d363a8d97b814cf258eec879cefe`.
- flow-candidate.exit: 2 bytes; SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.
- flow-lint.log: 176 bytes; SHA-256 `555e2ea44cc728fa14b2d3d51cf13d67a3dff94a9f1a20bafedd20493e913424`.
- flow-lint.exit: 2 bytes; SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.
- flow-live.log: 957 bytes; SHA-256 `9a18a612600e12ec5e2ad1ad1fff29f71b5419391ea1acc33d82eafe5e020a7e`.
- flow-live.exit: 2 bytes; SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.
- flow-output-final.log: 722 bytes; SHA-256 `0b6cc1592f0ad1778fd48704a1011b54d06c65575beb45a697f2310c96861e0c`.
- flow-output-final.exit: 2 bytes; SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.
- flow-lint-final.log: 176 bytes; SHA-256 `991fa4acd4cb9eed43419143849e5a72cc2f87045473326f4a910f7ea29208fa`.
- flow-lint-final.exit: 2 bytes; SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.
- flow-capacity-source.json: 249 bytes; SHA-256 `b890d40dbfeb19251fbfe59f45340a2b7e5094883f2b0455e36ce225706f7fae`.

Final mdBook build returns rc=0. Twelve rendered flow/limit markers, parent chapter
link and three parsed JSON examples with request/outcome/source bindings and
independent illustrative identifiers pass. Obsolete CLI-gap text is absent.
README remains 52 lines/2054 bytes; LIVE_STATUS category values are unchanged.
The next owner is .3.3.4.3.3.3.3.2.3.1 for capacity, followed by transport bounds.

The final escalated `python3 -B scripts/project_env.py bash scripts/check_no_background_jobs.sh`
census returns `handoff: OK`, rc=0. Result consumed before REPAIR-0031 staging.

Capacity follow-up: REPAIR-0032 reproduces and repairs the actual one-byte-over
CLI case, with an exact-fit positive and preserved existing-pending refusal.
See docs/tasks/artifacts/signoff_review/bootstrap-capacity.md; the earlier source
model above remains its historical observation rather than the final proof.
