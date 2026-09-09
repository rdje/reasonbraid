---
answers:
  - How will tenant authority revocation be ordered against protected local effects?
  - Why does tenant authority need a dedicated guard instead of locking the identity row?
  - How will admission evidence differ from the final administrative mutation outcome?
  - How are tenant transaction deadlines and cancellation before BEGIN acknowledgment handled?
  - How does an incremental build detect newly added embedded migrations?
  - How do standalone authority writers preserve tenant ordering and commit uncertainty?
  - When is the parent boundary checked for live grant issuance?
  - How can a typed domain refusal roll back earlier guarded writes?
  - How does complete enrollment preserve replay while ordering issuance and revocation?
---
# Keep tenant authority stable through the protected local transaction

- Owner: `SIGNOFF-REPAIR.3.3.4`; census/design child `.1`, foundation child `.2`, standalone service integration `.3.1`–`.3.2`.
- Status: primitive migration/transaction owner qualified in `.2`; five standalone authority services qualified under `.3.3.4.3.2` with 85 selected controls, strict lint and book checks; all results/shutdown consumed and three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls, focused strict lint and rendered book checks, with all results/shutdown consumed and both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls, final focused strict lint and rendered book checks; all results/shutdown consumed and three owned clusters absent. Bootstrap response-loss recovery remains `.3.3.4.3.3.3`; other application/final-effect paths retain their subsequent children.
- Source census: `docs/tasks/artifacts/signoff_review/tenant-authority-paths.md` at `1ba6184`.
- Preserves: actual-parent selection, valid frozen-tenant inspection, exact receipt readback and the `.3.1` foreign-target/no-op corrections.

## Coordination belongs to the transaction

Use a dedicated `tenant_authority_guards` table keyed by the complete tenant ID.
Its rows coordinate authority use; they do not create identity, enroll callers or
grant permissions. Backfill the known authority/identity namespaces and create a
missing anchor transactionally. Keep anchors stable while the application can use
the namespace; they are persisted coordination data, not disposable artifacts.

Migration 0004's authority tables have no tenant-identity foreign key. Standalone
authority APIs and their tests store boundaries/grants/denials without a `tenants`
row. Requiring that identity row as a lock anchor would change their contract.
A dedicated full-key row preserves it without reducing IDs to a collision-prone
lock key. Concurrent first use and rollback need explicit live controls.

A private type owns the SQL transaction, admitted tenant key(s), lock mode and
deadline. Guarded operations must validate their authorization and actual effect
tenant against this context. A bare pooled executor or a detached guard token is
not proof that a lock still protects the transaction executing the effect.

Ordinary authority use acquires a shared row lock; boundary/grant issuance,
status changes and epoch changes acquire an exclusive row lock before reading
authority. Use `FOR SHARE` and `FOR NO KEY UPDATE` respectively. PostgreSQL's
documented row-lock matrix permits concurrent shared holders and excludes them
from an exclusive holder; locks normally last until transaction completion.
This supports the design; application integration still needs contention evidence.
([PostgreSQL 16 row locks](https://www.postgresql.org/docs/16/explicit-locking.html#LOCKING-ROWS))

## Ordering and time

1. Declare every required tenant guard and its strongest needed mode before
   taking transactional domain locks. Multiple tenants use a deterministic sorted
   full-key order. A lock on another tenant is coordination, never authority to
   mutate that tenant. Do not upgrade a shared guard partway through a transaction.
2. Acquire the guard before idempotency, lease, grant/boundary target, aggregate,
   budget, inbox and other domain locks. Node event ingestion therefore changes
   at its outer transaction, before the existing lease lock, not only inside its
   result callback. Bootstrap inserts and newly created anchors stay in the same
   transaction as their authority facts.
3. Use explicit READ COMMITTED. After a guard wait, later authority queries see
   committed changes; sample database `clock_timestamp()` at the live evaluation
   after all earlier waits, including idempotency waits. A timestamp captured at
   pool acquisition or guard construction must not silently become the later
   decision time. PostgreSQL documents statement snapshots at this isolation level.
   ([PostgreSQL 16 READ COMMITTED](https://www.postgresql.org/docs/16/transaction-iso.html#XACT-READ-COMMITTED))
4. Select and record the actual authority sources, perform the protected local
   operation, record its required outcome and commit while retaining the guard.
   Authority change ordered first fences the later new operation; an operation
   ordered first completes before the competing authority change.

The guard does not freeze the wall clock. Validity is evaluated at the recorded
admission time; no statement claims that all response bytes arrive before expiry.
The standalone API's explicit evaluation-time parameter is a separate compatibility
contract. Live HTTP/MCP/domain effect entrypoints choose current database time;
they must not inherit an arbitrary historical time from a caller.

Bound lock waits, statements and the whole guarded operation. The implementation
must distinguish those limits: per-statement timeout alone is not a total
transaction deadline. No network/provider, user-input or filesystem workflow wait
belongs inside the guarded local transaction. Cancellation/rollback and commit
acknowledgment uncertainty need explicit refusal semantics; never fabricate a
receipt or automatically retry an operation whose commit outcome is unknown.
The foundation child implements ceilings of 5 seconds for lock waits, 10 seconds
per statement and 15 seconds for the whole operation, including pool acquisition
and commit. Internal policies may shorten these values in whole milliseconds,
with lock <= statement <= total, but cannot enlarge or disable them. One to eight
predeclared entries bound acquisition; duplicate keys take the strongest mode.

### Connection ownership begins before BEGIN acknowledgment

The matched delayed-BEGIN control reproduced SQLx 0.8.6 returning the same backend
still idle in transaction after setup cancellation. An outer connection lease now
exists before that await; only acknowledged successful commit permits pooling.
Errors, deadlines and cancellation discard the connection. Healthy commit retains
reuse, with local timeout settings reset. A pre-commit health query rejects a
callback that swallowed a SQL error rather than accepting aborted COMMIT as success.

Connection closure/rollback finishes asynchronously. It cannot prove an already
sent COMMIT failed: the deferred-commit control observes COMMIT/PgSleep, receives
CommitDeadline, then finds the original committed row through readback. The runner
therefore distinguishes pre-commit storage/deadline and unconfirmed commit outcomes
and never returns the callback's success value after a failed acknowledgment.
The fixed delayed-BEGIN injection is test-only; production BEGIN is not configurable.
Matched results, exact failed-cluster cleanup and per-control limits are preserved
in `docs/tasks/artifacts/signoff_review/tenant-guard-qualification.md`.

## Standalone authority writer integration

Boundary/grant creation and both tenant-bound status services take the exclusive
guard before authority policy or target-row access, keeping it through commit.
The active-boundary lookup uses the shared mode. Status services retain exact
expected-tenant predicates and target-row locking, and couple status changes to
the same transaction's epoch update. Malformed stored status is an error that
preserves the original row and epoch; it is not an inferred transitionable state.

The grant service reads full parent policy only inside the guarded candidate
tenant. If absent, a minimal foreign-ID existence probe may identify the existing
tenant-binding refusal. That probe must not decode/evaluate foreign policy or
mutate foreign state. A held foreign guard must not impede that binding refusal.
Standalone absent-parent/structural/time refusals now leave the callback as typed
errors, rolling back any provisional anchor as well as preventing grant creation.
Pre-existing anchors remain stable. The matched baseline in `.3.3.4.3.3.1` showed
that the previous successful-result adapter could commit a new anchor and let a
deferred anchor fault replace the intended policy refusal. The generic error
entrypoint shares the original lease, bounded Limits, guard order and commit
logic; E: From<GuardError> carries infrastructure failures without losing SQL
sources or the commit phase. It does not require E: From<sqlx::Error>.

Use an error when earlier provisional work must roll back. A successful callback
value still commits its local effects; it can deliberately represent a recorded
authorization denial. Preserve this choice at each call site, rather than hiding
domain policy in a synthetic SQL error or permitting transaction-control SQL in
callbacks. SQL errors must also leave as errors so the pre-commit health check
does not replace the original cause. Complete enrollment integration uses this
rollback support under `.3.3.4.3.3.2`; its dedicated qualification is separate from
the standalone repair.
Evidence: `docs/tasks/artifacts/signoff_review/typed-rollback-errors.md`.

Structural checks, including parent status and all existing ceilings, run before
the additional live-time check. Sample database time after acquiring the guard
and reading the own-tenant parent, immediately before INSERT. The actual parent
must be live then. A scheduled grant remains supported under a currently live
parent. This is the evaluation instant, not a promise that the clock stops during
commit or that authority remains live at delivery; later use checks both windows.

Public `AuthorityTransactionError` retains ordinary storage, pre-commit deadline
and unconfirmed commit distinctions. `create_boundary` returns it; the public
grant error retains original SQLx storage sources and adds transaction and
parent-not-live outcomes. These are documented Rust API changes. HTTP consumers
map Commit/CommitDeadline to safe 500 `commit_outcome_unconfirmed`, advising state
inspection before retry; ordinary storage keeps `dependency_unavailable`.
Controlled deferred faults may prove rollback for those fixtures, while the
primitive's control for a commit acknowledgment timeout demonstrates a commit can still finish.
No automatic retry, success receipt or blanket rollback claim follows an error.

Complete development enrollment now holds one exclusive guard before replay,
then uses same-context boundary/grant helpers and scoped connections for every
identity, quota and enrollment write. Kind/tenant validation remains before the
guard; replay remains before action parsing and authority lookup. Same-name
concurrency therefore resolves at replay after the first commit, rather than
racing into an identity unique violation. Typed errors abort all prior writes.
New bootstrap time comes from the database after guard/replay; parent liveness
is checked immediately before grant insertion. A commit error retains the
unconfirmed HTTP phase. Dev-trusted human/role issuer semantics are unchanged;
the old role-issuer comment and CLI default-action documentation are corrected.
This qualifies the complete local transaction, not a bootstrap recovery protocol:
an absent tenant_id still generates a server-side ID, and an unconfirmed error
does not give the client that ID or a stable request key. The CLI only persists
IDs after a successful response. New child `.3.3.4.3.3.3` owns matched response-loss
reproduction and protocol/CLI recovery without conflating tenant-scoped names
with request identity. Operator reconciliation may be needed in the meantime.
Evidence: `docs/tasks/artifacts/signoff_review/enrollment-transaction.md`.

Card import still uses its explicit unordered insertion bridge after a guarded
boundary lookup; complete migration belongs to `.3.3.4.11`. Node-certificate
epoch mutation remains `.3.3.4.10`.
The status services do not yet join HTTP caller admission, submitted reason and
final effect evidence; that remains `.3.3.4.8`. Matched baseline, wait-graph,
failure/recovery and final gate evidence live in
`docs/tasks/artifacts/signoff_review/tenant-authority-issuance.md`.

## Migration delivery is a build dependency

A final compatibility run reproduced a stale authority executable returning
VersionMissing(56), before any policy assertion. The migration and newly compiled
guard/upgrade tests were current, but adding the file alone had not invalidated
that older executable. Stable SQLx tracks existing included SQL files; each crate
must additionally declare its migration directory to Cargo. All four embedding
crates now do so: server/MCP/CLI watch root migrations, while node watches its
separate journal schema. Build scripts derive the current root from Cargo at
execution; no persisted checkout path, nightly flag, dependency upgrade or cache
purge is needed. Qualification checks actual rebuild/cache behavior on directory
entry addition/removal and reruns the previously stale compatibility target.
([SQLx migration recompilation](https://docs.rs/sqlx/0.8.6/sqlx/macro.migrate.html))

## Evidence and compatibility

Keep authorization admission distinct from final administrative effect evidence.
An additive record linked to the admission names the closed operation, actual
tenant-bound target, submitted reason when applicable, and committed change or
refusal/no-op. It commits with the mutation; evidence failure rolls back the
protected effect. Domain refusal may commit its admission and final refusal while
leaving the protected target and revocation epoch unchanged. Historical absence
remains absent/unspecified rather than an inferred successful operation.

Define that schema, strict codecs and exact own-tenant readback in `.7` before
converting grant/boundary revocation in `.8`. Preserve existing 404/409 behavior and
zero-change epoch rules. Any added reason requirement or response field must be
documented and tested as a wire change. A final local effect record still does not
prove external provider execution or delivery to the caller.

The eight named frozen-tenant reads retain their existing exception and receipts.
Only their admission gains a coherent guarded authority view; their response
queries remain separate. Committed idempotent replay retains its established
meaning; authority/target hash binding and delegation consent remain `.3.4`.

## Qualification is per integrated path

Children `.2`–`.13` separate primitive/schema qualification, authority writers,
thread and node transactions, inspections, effect evidence, revocation, breakers,
node administration, profiles/cards, federation and final coverage reconciliation.
Tests must observe database waits for both race orders, shared concurrency,
different-tenant progress, queued expiry, audit failure, rollback and no-ops.
Timing sleeps or an HTTP status alone do not establish ordering or unchanged state.

Separate current gates in recruitment/automatic initiation, resolver/workflow
registries, policy/lifecycle/deployment and MCP remain owned by their existing
repair leaves. They must adopt the transaction contract when their actual caller,
target and action policy is repaired. A guard does not make an identity-only,
tenant-insensitive or source-insensitive permission check correct. Do not promote
this design or any partial rollout into project-wide production qualification.
