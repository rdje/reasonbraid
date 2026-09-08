# Runbook: runaway budget

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN)
- Scope: spend crossing the declared bounds — a thread burning its ceiling,
  a tenant's aggregate spend trending out of control, or a breaker trip.

## Detection

- **The breaker trip:** the per-tenant spend latch trips IN the reservation
  transaction the moment the tenant's recorded spend (the settled usage + the
  active holds) crosses the threshold (`GET /v1/admin/breakers` via the
  `rb breaker` inspection).
- **The ceiling pressure:** `rb inspect usage` shows the held/settled/overrun
  split approaching the ceiling.
- **The denial rows:** the budget denials (the recorded `denied` reservation
  rows) rise.

## Authority

- Arming/resetting the breaker: `tenant_admin` (`rb breaker arm|reset`) — an
  audited write.
- Reading: `rb inspect usage`, `rb inspect breakers`, `rb inspect budget`
  (read-only).

## Safe first actions

1. **Do not reset the breaker to "see if it helps".** A tripped breaker is
   the proof the spend crossed the declared bound — the reset happens AFTER
   the cause is found.
2. **Freeze the picture:** the usage surface + the breaker state + the
   affected threads' ceilings.

## Diagnostic queries

- `rb inspect usage` — the held/settled/overrun/denied sums per tenant.
- `rb inspect breakers` — the latch state + the threshold.
- `rb inspect budget --thread <id>` — the per-thread ledger rows.
- The authorization/denial audit rows (who dispatched what).

## Containment

- The tripped breaker refuses every NEW reservation in the tenant (the
  in-transaction latch — nothing new starts).
- The held reservations stay held; the already-dispatched work completes and
  settles (the settlement reports the overrun, never clamps).

## Recovery

- Find the cause (the thread/loop that crossed) — the usage surface names it.
- Stop that dispatch source (the thread's budget, the quarantine, or the
  human decision).
- `rb breaker reset` — the tenant resumes under its declared bounds.
- The overrun is reported in the ledger (the honest settle — never clamped).

## Evidence preservation

- The reservation + the settlement rows (the ledger).
- The breaker's trip/reset rows (the audited writes).
- The denial rows (the recorded refusals).

## Communication

- The operator reports the trip (the crossing evidence + the cause) to the
  accountable owner; the reset is a declared action with its reason.

## Closure tests

- The budget suite (9 tests: the reservations, the settlements, the breaker
  trip/reset legs) on every guard pass.
- The usage-reconciliation surface (the command_api metrics leg) on every
  guard pass.
- The load harness (`.4.1`) can drive the ceiling pressure when the capacity
  question needs the measurement.
