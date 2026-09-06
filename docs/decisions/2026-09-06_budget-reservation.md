# The development budget engine: reservations before dispatch, denials at both boundaries, overruns recorded — never clamped

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.5.2` (WP5 call/token/time reservation + budget denial path)
answers: how does the development profile guarantee that no provider dispatch begins without an applicable budget reservation, with denial tests crossing both the server and the node boundary?

## The fact / decision

`reasonbraid-core/src/budget.rs` lands the MODEL and
`crates/reasonbraid-server/src/budget.rs` (+ `migrations/0005_budget.sql`) the
server-side ENGINE; the node supervisor gains the dispatch GATE:

- **Dimensions are unknown-or-measured, never zero** (§14.1): `BudgetDimensions`
  (`calls`, `input_tokens`, `output_tokens`, `wall_clock_seconds`) with `None` =
  "not metered here". `covers` is FAIL CLOSED: a requested dimension the ceiling
  does not meter is refused (never silently allowed). `add`/`subtract` are total and
  fallible — an over-release is a typed underflow, never saturating.
- **The server boundary** (`create_reservation`): held = active (unexpired)
  reservations + settled usage, checked against the ceiling in ONE transaction; a
  refusal commits a DENIAL ROW (the audit trail covers what was NOT reserved).
  `settle_reservation` records ACTUAL usage — lower frees the difference implicitly,
  higher is an OVERRUN reported in the return value, never clamped; `release` returns
  the hold; expired reservations stop holding (caller-supplied clock).
- **The node boundary** (`LocalBudget` + the supervisor gate): `execute_attempt` now
  REQUIRES a `ReservationReference` and local headroom — both checked BEFORE the
  dispatch boundary record. A refusal is journaled `failed_before_dispatch` (audited)
  and the adapter is never invoked. Completed attempts settle actual usage locally;
  pre-dispatch refusals release the hold; an INDETERMINATE attempt KEEPS its hold
  (§14.6: release only amounts not potentially consumed — adjudication owns it).
- **The acceptance's two boundaries are the same invariant twice**: the server
  refuses to ISSUE what the ceiling cannot cover; the node refuses to DISPATCH what
  it has not been issued. Denial tests exist on both sides.

## Why

KICKOFF WP5 acceptance items 4–5 (§14.3 step 4, §14.6): "no provider dispatch begins
without an applicable budget reservation"; "authorization and budget denial tests
cross both server and node boundaries". This completes WP5 and closes kill-risk
question 5 (small, evolving identity/authority/budget contracts).

## Measured behavior (legs: re-derive · falsify · durable)

- Re-derived live (PostgreSQL 16.15): `bash scripts/run_pg_tests.sh` →
  `test result: ok. 5 passed` (`budget`) — within-ceiling holds / beyond-ceiling
  denied AND recorded; settlement frees the unused remainder; release frees
  everything; expired reservations stop holding; overruns reported, never clamped,
  double-settlement a no-op. Node side: `test result: ok. 4 passed`
  (`supervisor_budget`) — a reservation covering no dispatch is refused BEFORE the
  boundary record (adapter never invoked, counted); exhausted local headroom
  refused with the reason journaled; completed attempts settle ACTUAL usage (100
  tokens, not the 5000 hold); an indeterminate attempt KEEPS its hold.
  `cargo test -p reasonbraid-core` → `test result: ok. 35 passed`.
- Falsified: the first `covers` semantics shipped with the doc claiming "untracked
  dims impose no constraint" while the code denied them — the tests caught the
  contradiction and the FAIL-CLOSED semantics were pinned (and documented) instead.
- Durable: model + engine + migration + both test suites are tracked; the proof
  commands run in `scripts/run_pg_tests.sh` / the `pg-tests` CI job (now five suites).

## Rejected designs

- **Making `execute_attempt`'s reservation optional** — the acceptance is absolute
  (NO dispatch without a reservation); the parameter is mandatory and the refusal is
  an audited `failed_before_dispatch`, not a panic.
- **Saturating arithmetic** — an over-release or over-settlement is a bug to be
  reported, not a rounding to absorb.
- **Clamping settlement to the reservation** — §14.3 wants estimate errors to feed
  routing; the overrun is returned (and the usage stored) in full.
- **Releasing the hold on an indeterminate attempt** — §14.6 forbids it (the attempt
  may have consumed); the hold persists until adjudication/settlement.
- **Reservation signatures now** — the dev profile trusts the channel as the
  transport of a server-issued reference; workload-identity signatures arrive with
  WP7 (recorded in the type's doc).
- **A shared budget ledger across server and node** — they are DIFFERENT authorities
  (§14.3 step 4 is a local check); two ledgers, one invariant.

## How to apply

- Any new dispatch path goes through `execute_attempt` — the gate is in one place.
- Keep `covers` fail-closed; a ceiling change that un-meters a dimension is a
  breaking policy change (bump `policy_version`).
- The reservation reference is the WP6 correlation key (`ExecutionReport.reservation_id`).
