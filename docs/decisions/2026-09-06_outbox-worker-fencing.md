# The WP2 leased outbox worker: claim with a fencing token, deliver deduped, acknowledge or lose the lease

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.2.2` (WP2 leased outbox worker + kill-point tests)
answers: how does an outbox worker claim work with a lease and a fencing value so a stale worker can never commit after a newer claim, and which kill points (after claim / after delivery / after acknowledgement) are proven?

## The fact / decision

`crates/reasonbraid-server/src/outbox.rs` implements the WP2 worker over the `.2.1` outbox
(`migrations/0002_outbox_worker.sql` — a NEW migration; 0001 is immutable). The worker loop is
three phases, **each its own commit point**, so the KICKOFF WP2 kill points are the seams
between them:

1. **Claim** — [`claim_ready`] leases up to `limit` ready items in ONE atomic
   `UPDATE … FROM (SELECT … FOR UPDATE SKIP LOCKED)` statement: the worker's identity is
   recorded, a fresh unguessable fencing token is issued per row
   (`gen_random_uuid()::text`), `lease_until` is set, and `attempt` is incremented. Concurrent
   workers cannot double-claim (each row is locked by exactly one claimer).
2. **Deliver** — [`deliver`] inserts the event into the `outbox_delivery` sink whose
   `event_id` primary key deduplicates: a redelivery after kill point 4 returns
   `DeliverOutcome::Duplicate` and produces no second effect.
3. **Acknowledge** — [`complete`] marks `dispatched = true` and clears the lease, **only if**
   the caller presents the CURRENT fencing token AND its lease is still live
   (`WHERE outbox_id = $1 AND lease_token = $2 AND lease_until > $3`). Otherwise the outcome
   is `CompleteOutcome::LeaseLost` and nothing is written.

Fencing therefore has two independent legs: a stale worker whose claim was superseded by a
newer one fails the token check; a lease that expired fails the liveness check even with a
matching token (the item must be re-claimed first). Every function takes the caller's `now`
(`chrono` bound to `TIMESTAMPTZ` via sqlx's `chrono` feature), so tests advance time past a
lease expiry deterministically — no sleeps, no dependence on application/database clock
agreement.

## Why

`ROADMAP.md` §17.3 requires workers that "claim durable jobs with lease owner, lease expiry,
attempt number" and states that "lease expiry permits recovery; fencing tokens prevent a stale
worker from committing after a newer lease." KICKOFF WP2 acceptance item 5 is exactly "stale
leased workers cannot commit after a newer fencing value," and kill points 3–5 (after claim,
after delivery, after acknowledgement) are unproven until each phase is its own commit point.

## Measured behavior (legs: re-derive · falsify · durable)

- Re-derived against live PostgreSQL 16.15 via `bash scripts/run_pg_tests.sh` →
  `test result: ok. 7 passed; 0 failed` (`outbox_worker`; the `.2.1` atomic suite stays
  `5 passed`). Tests: exclusive claim under two concurrent workers; re-claim after expiry
  issues a NEW token and increments `attempt`; **stale worker refused after a newer fencing
  value**; expired lease refused even with a matching token; kill points 3, 4, 5 recover with
  exactly one delivery effect.
- Falsified (would have caught the failure mode): the first live run failed because the tests
  shared ONE queue and ran in parallel — each test's global oldest-first claim picked up other
  tests' (and the `atomic_transaction` binary's) leftover rows. The suite now serializes under
  a module-level async mutex and purges the queue under the guard, so each test owns exactly
  the item it seeds. The failure was real and instructive: a shared queue demands exclusive
  ownership from its tests.
- Durable: the producer (this code + `migrations/0002` + `tests/outbox_worker.rs`) is tracked;
  the proof command is `scripts/run_pg_tests.sh`, which the `pg-tests` CI job mirrors.

## Rejected designs

- **Marking `dispatched` inside the claim transaction** — would collapse kill points 3 and 4
  into the claim: a crash "after claim" would already look delivered, and recovery could not be
  observed. Claim and acknowledgement must stay separate commits.
- **A worker-global generation/epoch table as the fence** — overkill for one outbox: a
  per-claim token achieves the same guarantee (a newer claim supersedes the old token) without
  a second table or cross-item coordination. A global epoch earns its keep only when a worker
  must be barred from ALL items at once (Phase 2 quarantine semantics, `ROADMAP.md` §17.3/§10.6).
- **Compare-and-swap on `attempt` alone as the fence** — an incremented attempt also rejects a
  stale claim, but it cannot refuse an EXPIRED lease that was never re-claimed; the
  `lease_until > now` liveness leg is separate and needed (a worker whose lease lapsed must
  re-claim before acknowledging).
- **Consulting the database clock for lease expiry** — would force tests to sleep or to fake
  the DB clock; caller-supplied `now` keeps the worker deterministic and the tests instant.
- **A "next eligible time" backoff column** — nothing in `.2.2` fails a delivery, so there is
  no backoff schedule yet; the column arrives with the Phase 2 retry policy rather than being
  guessed now (recorded in the migration header).

## How to apply

- The outbox worker contract is `claim → deliver → complete`, one commit per phase; keep the
  phases as separate transactions so kill points stay observable and recovery stays possible.
- `complete` must always carry the fencing token and the caller's `now`; never acknowledge
  without both checks.
- Deduplication belongs at the effect sink (the `event_id` primary key), not in the claim:
  redelivery after an acknowledged-but-lost delivery is the consumer's job to dedupe.
- Worker columns arrived via a NEW migration (`0002`); never edit a landed migration.
- The formal ADR-004 (persistence/outbox pattern) is deferred to WP8's ADR set; this record is
  the `.2.2` durable fact alongside [[2026-09-06_atomic-transaction]].
