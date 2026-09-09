# Bootstrap recovery qualification and contract

Owner: `SIGNOFF-REPAIR.3.3.4.3.3.3.1`. Product baseline: `01bd473`.

Final runtime status: 25 selected controls (24 live / one pure) and focused strict
lint pass. All results/shutdown are consumed and the owned cluster is absent.
The recovery protocol below is selected, not implemented in this leaf.

## Source census

Re-read the complete EnrollRequest/EnrollResponse, router and enrollment handler,
ControlApiError mapping, CLI Config/StateFile load/save, ApiClient parsing/enroll,
run_enroll, both local-state save call sites (enrollment and thread creation),
main argument dispatch and core ID/envelope conventions. Files:
crates/reasonbraid-server/src/api.rs, authority/transaction.rs,
crates/reasonbraid-cli/src/lib.rs and main.rs,
crates/reasonbraid-core/src/id.rs and envelope.rs, server tx.rs idempotency helpers,
and migrations 0001/0006/0007. The CLI live harness/Cargo manifest was also reviewed.

The server has POST /v1/enrollments, no bootstrap outcome lookup, no request key
on EnrollRequest and no generated IDs on the commit-uncertain error. The CLI
persists identities only after a 2xx response. StateFile::save writes state.json
in place; both enrollment and thread creation load/modify/save it. Bootstrap
recovery therefore needs a durable pending request plus safe state publication,
not only a new request field. The CLI's ordinary command-envelope key is fresh
per invocation and does not cover enrollment. Core RequestId and TenantId are
separate prefixed ID families; a client request ID must not be rebranded into a
tenant ID. Server tenant allocation remains authoritative.

## Matched live uncertainty control

The owned enrollment suite adds one compatibility control for no-key bootstrap.
A fixed BEFORE INSERT enrollment trigger sleeps six seconds. A separate deferred
AFTER INSERT constraint trigger sleeps 9.5 seconds. Each statement stays below
the ten-second ceiling; together they cross the fifteen-second total limit while
COMMIT is executing. The test observes actual COMMIT/PgSleep through PostgreSQL,
keeps ownership of its request handle and consumes it, polls authoritative
committed readback, restores both faults before assertions and then repeats the
no-key request. No elapsed sleep alone establishes commit or rollback.

Observed backend: 95611. Response: HTTP 500 commit_outcome_unconfirmed with the
existing safe message and no generated IDs. The original bootstrap subsequently
exists as tenant ten_01a085bd-b81a-7bd0-b2ef-74d706cc8847 and human
hpr_01a085bd-b81b-7d02-b501-ed259aeb8c1c. Exactly one original row was read back.
Repeating the same name without a key returns a distinct tenant/principal with
replayed=false. The test proves two distinct tenants; each has one tenant,
boundary, grant, human, enrollment and anchor plus two quotas. It does not infer
that a failed acknowledgment is a rollback receipt, and it does not call the
preserved intentionally-new no-key semantics an idempotent operation.

Command:
`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py enrollment_transaction authority_transaction`.
Owned cluster: target/pg-tests/run-x9haoeec. Result rc=0; nine enrollment controls
pass (11.07s build / 33.30s execution), sixteen primitive controls pass (16.32s
build / 4.52s execution). None ignored/filtered. Runner stopped and removed its
cluster; result and shutdown consumed, residue absent. Outer evidence:
target/bootstrap-recovery-controls/qualification.log / .exit.

Focused strict lint:
`python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-server --test enrollment_transaction -- -D warnings`.
Result rc=0, 14.99s; target/bootstrap-recovery-controls/lint.log / .exit.
This leaf changes the test and durable contract only, not production behavior.

## Selected next implementation

See docs/decisions/2026-09-09_bootstrap-recovery.md. The server protocol and durable
CLI recovery have separate owned implementation children. An optional canonical
client RequestId binds one new-human bootstrap and its immutable successful
outcome. The server continues generating the tenant ID. Same-key/same-request
recovery returns original IDs; a conflicting request refuses; no-key requests
remain intentionally distinct. Guarded routing may abort a provisional attempt
and reacquire the recorded tenant once, within one total operation budget. It
never decodes a foreign outcome before guarding its tenant. Name equality is
request binding, not a global tenant identity.

qualification.log SHA-256: `e5a16e45d724bb06790dce2e8d6fd135f7839bdc373b1a7ea1857d065a661c33`.
lint.log SHA-256: `8807f2b70031a0dbd9e91b34a2c4c1674e6ff2badcd855f68c61fe53169da6f1`.

Final format/diff checks, mdBook build and seven parsed authority/CLI markers
pass, distinguishing observed uncertainty from the unimplemented recovery field
and client behavior. The first marker check needed HTML inline-tag punctuation
normalization and correction of its expected article; source/rendered book text
was intact. README remains 52 lines / 2,054 bytes; LIVE_STATUS categories unchanged.
All verification results/shutdown consumed; no background result remains before
the commit workflow.
