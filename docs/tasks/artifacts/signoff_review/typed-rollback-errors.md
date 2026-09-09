# Typed errors abort guarded work

Owner: `SIGNOFF-REPAIR.3.3.4.3.3.1`. Baseline product implementation: `e5a3e54`.

Final status: qualified. All 89 selected controls (88 live / one pure), focused
strict lint and rendered book checks pass. Every result/shutdown is consumed;
both owned clusters are absent. Historical captures below preserve the baseline
and staged qualification evidence.

## Contract and baseline

The fixed guard callback commits successful values, including a deliberately
returned domain outcome. Preserve that contract: committed refusal/audit records
can be intentional. Enrollment needs a different contract: its typed validation
or policy error must abort preceding identity, quota and authority writes.
A new private typed-error entrypoint will share the same connection lease, guard
order, limits and commit-phase handling. Its error type must implement
From<GuardError>; transaction errors must not lose their phase or SQLx cause.
Use the existing validated Limits type, with production defaults for standalone
grant creation. This permits focused short-deadline controls without adding a
configurable BEGIN or weakening production ceilings.

The existing standalone create_grant adapter wraps MissingBoundary, Refused and
BoundaryNotLive as successful callback values. On first use it therefore commits
a coordination anchor despite returning an error to its caller. A deferred
constraint fault on that anchor can replace an established domain refusal with a
commit error. Two matched controls cover all three refusal classes: six cases
compare missing/existing anchors and three inject a deferred anchor failure.
Each case checks no grant was inserted and successful recovery. Fault DDL is
removed before result assertions; only exact unique fixture anchors are removed
before the operation, with no concurrent user of those fixture namespaces.

Baseline command:
`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py authority_issuance`.
Owned cluster: `target/pg-tests/run-qion7lwg`.
Outer evidence: `target/typed-rollback-controls/baseline.log` and `.exit`.
Result and shutdown pending. No product change before consuming this baseline.

## Enrollment source contract for the following child

The complete handler, models and development constructors in
`crates/reasonbraid-server/src/api.rs` were re-read along with migrations 0006,
0007 and both default quota insertion helpers. Kind validation and tenant parsing
precede replay. Replay uses (tenant, kind, name), precedes action parsing, returns
the original principal and omits boundary/grant IDs. Human action input is ignored
in favor of the nine development admin actions. Role defaults include contribution
and invitation response. A new human with no tenant creates a new tenant each
call; this is not global name idempotency.

The existing handler starts its raw transaction after replay and reads an existing
boundary via a separate guarded service. Its later grant/identity/quota/enrollment
writes therefore have no encompassing tenant guard. New-tenant identity/quota/
boundary writes are already in the same raw transaction. Both identity tables
have tenant FKs and per-tenant name uniqueness; enrollments has its own
(tenant, kind, name) unique key. Standalone authority namespaces have no required
identity row. Preserve these distinctions when integrating the complete handler.
The role issuer is a newly generated HumanPrincipalId, despite its old comment
claiming a role-principal stand-in; human issuance uses that human's own ID.
Caller/issuer admission policy remains separately owned by `.3.5`.

## Consumed baseline result

Baseline returned rc=101: 12 existing controls passed and both matched controls
failed (41.32s execution; 9.48s build). All three new-anchor refusals left one
anchor and no grant. Each corresponding existing-anchor control preserved its
anchor. All three deferred faults replaced the intended domain refusal with
Transaction(Commit(Database(P0001))). Every case recovered after its fixture was
restored. Runner result and shutdown consumed; exact stopped cluster awaits
census/removal. Product edits began only after consuming that result.

## Baseline artifact cleanup

Exact stopped cluster removed after consumed shutdown, no live old postmaster or owned server, no postmaster.pid, no symlinks and same-volume census (1610 files / 51346676 bytes; device 16777244). Residue census: target/pg-tests/run-qion7lwg absent. The complete command log is byte-for-byte contained in the retained outer baseline log.

Command log SHA-256: `2e10cabd544faaa680a696dd28d868c7228c857bc224bbbc65b2150057387b54`.
Outer baseline log SHA-256: `fdbc4a399c3943bfb9c587071c14b7f85d07a19a4b2565da2dbf6853cae65cd2`.

## Implemented qualification matrix

- A typed refusal after a guard-protected INSERT retains its exact reason and
  backend ID, discards that backend, and leaves zero probe/first-use anchor rows.
  The same tenant then succeeds through the typed entrypoint and commits its row
  and success value. Existing deliberate refusal-as-success-value persistence
  remains controlled by the unchanged fixed-callback test.
- The test error implements only From<GuardError>, deliberately not From<sqlx::Error>.
  Empty guards, scope mismatch, original division SQLSTATE 22012, swallowed-error
  health SQLSTATE 25P02, exhausted-pool and callback whole deadlines, and closed
  pool storage errors retain their typed transaction variants. Timed-out staged
  writes disappear before successful guard reacquisition.
- Existing deferred FK and COMMIT/PgSleep controls now call the typed entrypoint:
  the former retains Commit plus SQLSTATE 23503 with no effect; the latter observes
  actual commit wait, returns typed CommitDeadline and subsequently reads the
  original committed row. No retry or blanket rollback inference is permitted.
- Standalone issuance repeats the six missing/existing-anchor and three deferred
  fault cases against the repaired adapter, with all prior issuance/race/storage
  controls. Boundary creation still uses the fixed callback entrypoint; its
  deferred fault and the public grant commit-source control remain in this gate.

Focused strict lint passed, rc=0 (17.67s):
`python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-server --lib --test authority_transaction --test authority_issuance --test authority --test command_api --test cards -- -D warnings`.
Evidence: `target/typed-rollback-controls/lint.log` / `.exit`.

Final selected gate is running in `target/pg-tests/run-e6r2dkt0`:
`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py authority_transaction authority_issuance authority command_api cards migration_upgrade`.
Evidence: `target/typed-rollback-controls/final.log` / `.exit`.
The first suite passes all 16 controls (15 live / one pure), 4.53s execution.
Other suite results and shutdown remain pending at this capture.

## Final consumed qualification

The six-suite gate returned rc=0: 16 primitive (15 live / one pure), 14 issuance,
22 authority, 33 HTTP, one composite card and three upgrade controls pass: 89 in
total, 88 live / one pure. No ignored or filtered controls. Every result and
shutdown consumed; runner stopped and removed target/pg-tests/run-e6r2dkt0.
Both baseline and final cluster residues are absent. No full CI or push.

| Suite | Build | Test execution | Passed |
| --- | --- | --- | --- |
| authority_transaction | 21.00s | 4.53s | 16 |
| authority_issuance | 9.51s | 15.87s | 14 |
| authority | 9.01s | 0.26s | 22 |
| command_api | 5.87s | 15.71s | 33 |
| cards | 5.67s | 30.11s | 1 |
| migration_upgrade | 16.21s | 9.49s | 3 |

`cargo fmt --all --check`, `git diff --check` and
`python3 -B scripts/project_env.py mdbook build docs/book` pass. Five parsed
generated-authority-page markers verify typed refusal rollback, existing anchors,
error/value choice and the unconfirmed-commit wire contract. README remains
52 lines / 2,054 bytes; its objective/layout/commands are unchanged. LIVE_STATUS
categories are unchanged.

final.log SHA-256: `c86b6eb23f022d61853e00d5163c7ffd613ffd69bc129bae06e13c645514c979`.
lint.log SHA-256: `d2817da33cb7d757c4f6a6991250f687c6dbe774e18d5d164d758c893af72bda`.
baseline-cleanup.json SHA-256: `99ca280b98178d53c6b93214bbc2cf9013d50d1fde56f6de95e131e6e73562f7`.
