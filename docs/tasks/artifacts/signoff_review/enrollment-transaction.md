# Complete development enrollment transaction

Owner: `SIGNOFF-REPAIR.3.3.4.3.3.2`. Baseline product implementation: `1dfc84d`.

Final status: complete local enrollment ordering/rollback qualification. All 97
selected controls (96 live / one pure), final focused strict lint and rendered
book checks pass. Every result/shutdown is consumed; all three owned clusters are
absent. Client bootstrap recovery remains the explicitly tracked next child.
Historical captures below retain the matched baselines and qualification stages.

## Source contract and implementation boundary

The complete handler/models/dev constructors, authority insertion/error helpers,
quota defaults and migrations 0006/0007 were read before product edits (the prior
prerequisite artifact also records that source read). The current handler parses
kind and tenant before replay, returns replay before action parsing, and starts
its raw transaction after replay. Existing-boundary lookup opens and commits a
separate shared guarded transaction. Grant, identity, quota and enrollment writes
then happen in the unguarded raw transaction. A fresh human bootstrap has no outer
guard. Its host timestamp is sampled before those waits.

Own the whole local transaction: exclusive guard before replay; same-context
active-boundary lookup and grant insertion; current database time for a new
boundary after guard/replay and for parent liveness immediately before grant
INSERT. Every staged identity, authority, quota, enrollment and first-use anchor
row must roll back on a typed pre-commit error. Commit failures retain the
unconfirmed phase through the existing HTTP mapper. The new private rollback
entrypoint is qualified in docs/tasks/artifacts/signoff_review/typed-rollback-errors.md.
No public API/schema expansion is needed. Card import's full transaction remains
`.3.3.4.11`; caller/issuer authentication remains `.3.5`.

Preserve current request/replay behavior: human actions are ignored for the nine
explicit dev admin actions; role defaults are contribution plus invitation
response; replay returns the original principal and omits boundary/grant IDs.
Frozen replay creates nothing and remains allowed. Missing tenant for roles,
unsupported kind and malformed tenant remain invalid. A no-active-boundary error
precedes role action validation. New human requests without tenant_id create
separate tenants, even if their names match. Role issuer code generates a new
HumanPrincipalId; correct the inaccurate role-ID comment without changing that
dev-trusted behavior. Standalone authority namespaces need no identity, but an
enrollment still requires the actual tenant identity FK.

## Matched baseline controls

The new suite tests eight cases/composites with unique disposable fixtures:

1. An external exclusive guard blocks replay before action validation; another
   tenant can progress; exact replay JSON and original tenant rows are preserved.
2. Enrollment paused inside its grant INSERT holds the tenant guard against real
   boundary revocation through its later identity writes/commit.
3. Real revocation waiting at the parent row already holds its guard; enrollment
   waits and then refuses without creating a role.
4. A parent expires during an observed guard wait; fresh database-time evaluation
   refuses, with the full eight-table snapshot unchanged.
5. Concurrent same-tenant/kind/name enrollment queues on the first enrollment's
   guard and returns one new identity/grant/quota plus one honest replay.
6. Eight late failure cases: new/existing tenant × identity, principal quota,
   enrollment INSERT and deferred commit fault. Full table snapshots must match,
   restored fixtures must recover and the commit fault preserves its HTTP phase.
7. Invalid actions, structural ceiling, expired/future/frozen/unknown parent
   contexts with first-use anchors leave full snapshots unchanged. Role defaults,
   ignored human action input, frozen replay and initial validation stay explicit.
8. A standalone authority namespace cannot silently create an identity: FK
   refusal leaves all rows unchanged; explicitly supplying the missing identity
   in the fixture permits recovery.

Waits require actual pg_stat_activity/pg_blocking_pids dependencies, never elapsed
sleep alone. The tests release held transactions, consume both request jobs and
restore owned fault DDL before outcome assertions. Snapshots contain full sorted
JSON rows from tenants, enrollment_boundaries, authority_grants, human_principals,
agent_roles, usage_quotas, enrollments and tenant_authority_guards. Tests remove
only their exact unique fixture anchors before concurrency, to model first use.

The initial test source had an incorrectly quoted inline JSON SQL literal; the
formatter/compiler rejected it before runtime. It was replaced with a bound JSON
parameter, and the corrected focused strict Clippy check passes, rc=0 (8.01s):
`python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-server --test enrollment_transaction -- -D warnings`.
Evidence: target/enrollment-transaction-controls/baseline-lint.log / .exit.

Baseline command:
`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py enrollment_transaction`.
Owned cluster: target/pg-tests/run-0oyn4_3f.
Evidence: target/enrollment-transaction-controls/baseline.log / .exit.
Result and shutdown pending. No product code changes before consuming the matched
baseline. Only the new test suite and runner registration changed at this stage.

## Consumed initial baseline

run-0oyn4_3f returned rc=101: two controls passed / six failed (33.36s
execution, 17.35s build). Revocation-first fencing and standalone identity FK
refusal/recovery passed. Concurrent same-name enrollment returned 200 plus a
500 for agent_roles_tenant_id_name_key; the second request did not wait on the
first enrollment's tenant guard. Queued-expiry enrollment returned 200; enrollment
paused in grant INSERT let revocation finish and the parent become revoked before
enrollment returned 200. Replay bypassed the held guard. Invalid/structural/frozen/
unknown policy cases changed the eight-table snapshot; expired/future parents
issued new roles. The first existing-tenant identity fault changed the snapshot;
other later faults rolled back, but both deferred commit cases returned ordinary
dependency_unavailable instead of the commit phase. Every late-fault recovery
returned 200. Result and shutdown consumed; exact stopped cluster retained for
census/removal.

Before product edits, refine snapshot diagnostics to name changed tables and row
counts, then repeat the baseline. This distinguishes the coordination residue
from provisional identity/grant writes with direct runtime evidence instead of
inferring which row changed from source alone. All baseline controls remain.

## Initial baseline cleanup

Consumed stopped run-0oyn4_3f removed after exact process, postmaster.pid, no-symlink and same-volume verification: 1611 files / 51589185 bytes, device 16777244. Residue absent; the entire command log is preserved byte-for-byte in the outer baseline log.

Command SHA-256: `21ff8db8d5d329a42cd1fb353b799121eec70936392f0cce5e742bb8c1d60cad`.
Outer log SHA-256: `2b9b2c59232c03d76125aa5f819cb7ffcba180974afbc174283d303426aa2129`.

The diagnostic helper's first strict lint rejected bool::then inside filter_map; it now uses filter followed by map, with no lint waiver. The repeated baseline includes precise table/row-count deltas.

Detailed baseline: target/pg-tests/run-gco46rqd, running through the same eight
controls against unchanged product code. Log/exit: target/enrollment-transaction-controls/baseline-detail.log / .exit. Corrected diagnostic strict lint passes
rc=0 (16.06s), baseline-detail-lint-final.log / .exit. Result/shutdown pending.
CLI source comparison confirms omitted --actions stays absent in the HTTP body;
it therefore receives both current role defaults. Correct the CLI book's obsolete
single-action default within this leaf when documenting the complete enrollment
contract. No CLI implementation change is needed.

## Consumed detailed baseline / product edit boundary

run-gco46rqd returned rc=101: two pass / the same six fail (19.30s execution,
11.51s build). Snapshot diagnostics directly identify only tenant_authority_guards
(+1 row) for invalid-action, structural, frozen, unknown-tenant and the initial
existing-tenant identity-storage refusals. Expired/future parent cases add one
grant, role, principal quota, enrollment and new anchor. Queued expiry, whose
fixture anchor already exists before the snapshot, adds one grant, role, quota
and enrollment. Other later storage snapshots stay unchanged; both commit faults
still lose their phase. Every result and shutdown is consumed. The stopped
detailed cluster awaits exact cleanup. Product edits start after this capture.

## Detailed baseline cleanup

Consumed stopped run-gco46rqd removed after exact process, postmaster.pid, no-symlink and same-volume verification: 1611 files / 51426548 bytes, device 16777244. Residue absent; the complete command log remains byte-for-byte in the outer detailed baseline log. Both baseline clusters are absent.

Command SHA-256: `4747910eda9e1bef81a50113af8b061537fbd8195c788b547ee897679b60d49a`.
Outer log SHA-256: `cda70ddf3ebd0c40d6da5a00277ad45b98465bcdd29526074e5acb97d349a091`.

## Initial implementation qualification

Focused strict lint passed, rc=0 (35.35s):
`python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-server --lib --test enrollment_transaction --test authority_transaction --test authority_issuance --test authority --test command_api --test cards -- -D warnings`.
Evidence: target/enrollment-transaction-controls/lint.log / .exit.

The seven-suite selected gate is running in target/pg-tests/run-v6frkjiv:
`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py enrollment_transaction authority_transaction authority_issuance authority command_api cards migration_upgrade`.
All eight enrollment controls pass (17.75s execution). Remaining suite results
and shutdown pending; evidence: target/enrollment-transaction-controls/final.log
/ .exit. The book now describes full transaction/replay/time/error behavior and
examples, and the CLI chapter states both existing role default actions. Source
request/response doc comments are corrected to match those defaults and describe
replay in terms of no new identity/grant/enrollment; these final comment-only
edits require a final focused lint check, not another behavioral implementation.

Final selected strict lint after the API request/response comment corrections
passed, rc=0, 1m 06s including Cargo lock wait. Same command/scope as initial lint;
evidence target/enrollment-transaction-controls/final-lint.log / .exit. Result
consumed. Final book build and nine parsed generated authority/CLI markers pass,
covering ordering/replay, defaults, rollback, issuer scope and commit uncertainty.
The live selected gate remains in flight; no further Rust changes are pending.

## Tracked client recovery follow-up

A source audit of api.rs route registration, EnrollRequest, enroll/enroll_in_guard
and ControlApiError plus CLI ApiClient::enroll/run_enroll identifies a remaining
new-bootstrap recovery gap: absent tenant_id generates a new server-side ID;
commit_outcome_unconfirmed returns no generated ID; the route has no bootstrap
request key or outcome lookup; the CLI persists IDs only after a success response.
The primitive's actual later-commit control establishes why an unconfirmed error
cannot be treated as rollback. Matched enrollment response-loss/commit-timeout
reproduction and the protocol/CLI repair belong to new pending child
SIGNOFF-REPAIR.3.3.4.3.3.3. This child is selected after the current leaf is
committed, before coverage reconciliation. Keep intentionally new tenants
distinct; do not invent global-name idempotency. Current tests qualify complete
local enrollment ordering/rollback and truthful commit phase, not an end-to-end
bootstrap recovery protocol. The book names the limitation immediately.

## Final consumed qualification

The seven-suite gate returned rc=0: 97 controls, 96 live / one pure, none ignored
or filtered. Runner shutdown/result consumed, with target/pg-tests/run-v6frkjiv
stopped and removed. Both matched baseline clusters are also absent. No full CI
or push. Source search confirms enrollment uses the same-context helpers and
only card import still calls create_grant_unordered_in_tx and the standalone
active-boundary loader (api.rs); broader writer coverage remains `.3.3.4.3.4`.

| Suite | Build | Test execution | Passed |
| --- | --- | --- | --- |
| enrollment_transaction | 31.42s | 17.75s | 8 |
| authority_transaction | 8.46s | 4.51s | 16 |
| authority_issuance | 40.52s | 28.34s | 14 |
| authority | 17.50s | 0.27s | 22 |
| command_api | 16.08s | 22.80s | 33 |
| cards | 8.97s | 30.12s | 1 |
| migration_upgrade | 18.28s | 30.45s | 3 |

Final focused strict lint returned rc=0, 1m 06s including Cargo lock wait.
`cargo fmt --all --check`, `git diff --check`, `mdbook build docs/book` through
project_env.py and nine parsed authority/CLI contract markers pass. The book
explicitly names bootstrap recovery as remaining work; it does not promise
recovery from an unconfirmed outcome without the generated tenant ID. README
objective/layout/commands remain unchanged (52 lines / 2,054 bytes); LIVE_STATUS
categories unchanged.

final.log SHA-256: `f5566424ddab49cdf38dd28f48251dd6a1a218202711200302355782540c406a`.
final-lint.log SHA-256: `cebca20c947f7c853d4ce9504d86befcd92863ae788e7993b5fabd7115c7d6a3`.
baseline-cleanup.json SHA-256: `acd9646c7676a97b4d4b83785e5cc24268977291e80cc22ed6c1ebd83c12c0b6`.
baseline-detail-cleanup.json SHA-256: `cd9b2341f9f2d3d70e6a59ea4789b2424d1cd83296bf9d839aa7f845f374454e`.
