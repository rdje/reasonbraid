# The WP2 atomic transaction: state + event + idempotency + outbox in one PostgreSQL transaction

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.2.1` (WP2 atomic-transaction proof)
answers: how does the control plane record an accepted command so that a successful response is committed durable state, a redelivery is idempotent, and a different request hash is a conflict — all in one database transaction?

## The fact / decision

`crates/reasonbraid-server` proves the WP2 acceptance with a single `BEGIN … COMMIT`
transaction (`src/tx.rs` `apply_command`) that writes four tables (schema in the
repository-root `migrations/0001_atomic_transaction.sql`, applied by `sqlx::migrate!`):

1. `idempotency` — keyed on `(tenant_id, idempotency_key)`, the serialization point;
2. `event_log` — append-only, unique `(tenant_id, aggregate_id, aggregate_version)`;
3. `aggregate_state` — the current aggregate state, upserted;
4. `outbox` — an outbox item with a foreign key to `event_log`.

The transaction's order is load-bearing. It **claims the idempotency slot first**
(`INSERT … ON CONFLICT (tenant_id, idempotency_key) DO NOTHING`). A second submission of
the same key blocks on the unique index until the first commits, then takes one of two
branches: **same hash → replay** (returns the *original* stored `response_result`, writes
nothing), **different hash → `IdempotencyConflict`**. A fresh claim proceeds to lock the
aggregate row (`SELECT … FOR UPDATE`), derive `next_version = MAX+1`, insert the event,
upsert the state, insert the outbox item, and finally record the semantic result back on
the idempotency row — then commit.

Atomicity is a *property of the code path*, not of a transaction setting: every write runs
on one transaction, and a failure at any step drops it (rolling back the idempotency claim
too). The outbox FK to `event_log` means an outbox item *implies* its event is durable.

## Why

`ROADMAP.md` §8.6 and KICKOFF WP2 require "one transaction writes current state, ordered
event, idempotency result, and outbox item," with the acceptance "successful response ⇔
committed durable state; same key+hash returns original result; different hash is conflict;
transport redelivery produces one domain effect." Four separate `INSERT`s on autocommit
could tear (an event without an outbox item, a result without a state), and naive
idempotency ("check then insert") races under redelivery. The claim-first pattern makes the
primary key the concurrency control, which is why a redelivered message produces exactly
one domain effect.

**Honest limits, stated rather than hidden:**
- `next_version = MAX+1` under `FOR UPDATE` serializes writers to an *existing* aggregate
  row, but a brand-new aggregate's first insert is not gap-locked, so two concurrent
  first-writes to the *same new* aggregate are not fully serialized. This is a Phase-1
  concern (multi-writer threads), out of WP2's single-writer scope.
- The server operates on plain string identifiers, not `reasonbraid-core`'s branded
  `Id<K>` types; wiring those into the command handlers is WP5/WP6, not this proof.
- The tests skip when `DATABASE_URL` is unset, so the real proof runs only in
  `scripts/run_pg_tests.sh` (ephemeral server) and the `pg-tests` CI job — not in `make check`.

## How to apply

- Keep all four writes in one transaction; never split state/event/idempotency/outbox
  across autocommit statements. The idempotency claim is always step 1.
- Replay returns the stored `response_result` verbatim; never recompute a result on replay.
- Add lease/fencing columns to `outbox` in a **new** migration for `.2.2`, not by editing
  `0001` (a landed migration is immutable).
- Source the `migrations/` directory from the repository root (KICKOFF §3), and apply it
  with `sqlx::migrate!("../../migrations")` from the server crate.
- `sqlx` is used with `tls-none` + `sslmode=disable` (local/CI Postgres is plaintext); if a
  TLS-required Postgres appears, that is a deliberate config change, not a silent default.
- The formal ADR-004 (persistence/outbox) is deferred to WP8's ADR set; this record is the
  `.2.1` durable fact, see [[2026-09-06_state-transitions]] (whose aggregates this persists)
  and [[2026-09-06_reason-codes]] (whose typed errors the command layer will emit).
