---
answers:
  - How will tenant authority revocation be ordered against protected local effects?
  - Why does tenant authority need a dedicated guard instead of locking the identity row?
  - How will admission evidence differ from the final administrative mutation outcome?
---
# Keep tenant authority stable through the protected local transaction

- Owner: `SIGNOFF-REPAIR.3.3.4`; census/design child `.1`.
- Status: selected implementation contract; no guard or final-effect implementation yet.
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
The foundation child chooses and tests concrete timeout values before integration.

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
