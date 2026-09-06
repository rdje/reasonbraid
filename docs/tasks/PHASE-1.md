# PHASE-1: trustworthy LAN vertical slice

## Metadata

- Tree ID: `PHASE-1`
- Status: `active`
- Roadmap lane: Phase 1 (`ROADMAP.md` §20.3)
- Created: `2026-09-05`
- Estimate: 14–22 engineer-weeks
- Depends on: Phase 0 contracts
- Exit: G1–G2; Demonstration A (`ROADMAP.md` §26.1)

## Goal

A killed node resumes without duplicated ReasonBraid effects; a provider
ambiguity is visible; all accepted messages appear once in domain state despite
transport redelivery. Invited users/agents on a trusted LAN can hold a durable
conversation without binding-governance claims.

## Non-Goals

- Automatic semantic discovery, arbitrary Web fetching, binding policy
  publication, public Internet exposure.

## Task Tree

- ID: `PHASE-1.1`
  Status: `in_progress`
  Goal: coordinator modular monolith, PostgreSQL migrations, aggregate/event/outbox patterns
  Backlog: 9, 10, 15
  ADR: 002, 004
  Children: `.1.1.1`–`.1.1.3` (decomposed `2026-09-06` so each child is one signoff-sized slice)

  - ID: `PHASE-1.1.1`
    Status: `done`
    Goal: the aggregate/event/outbox library — extract the WP2 claim → authorize →
      validate → apply machinery (`reasonbraid-server/src/tx.rs`) into a typed,
      reusable aggregate module: revision-checked state transitions (the locked
      head), ordered event append, idempotency claim/replay/conflict, outbox
      enqueue in ONE transaction, plus in-tx test helpers. Every Phase 0 caller
      switches to it with zero behavior change.
    Backlog: 9
    ADR: 004
    Acceptance: all existing offline suites + the live-PG suites stay green; the
      library owns the claim-first and revision semantics (the transaction body is
      the single write path); a new helper proves fresh-apply vs replay against a
      test aggregate.

  - ID: `PHASE-1.1.2`
    Status: `pending`
    Goal: migration 0007 — first-class identity store: `tenants`, `hosts`, `nodes`,
      `agent_roles`, `incarnations`, `runs`, `human_principals` (the `.6.1`
      `enrollments` table is the dev stand-in). Enroll writes the enrollment row
      AND the identity row in one transaction; the existing surfaces keep working
      unchanged.
    Backlog: 10
    Acceptance: the new tables exist with UUIDv7 ids and the §17.2 conventions
      (tenant on every material record); enroll/re-enroll tests green; no existing
      suite regresses.

  - ID: `PHASE-1.1.3`
    Status: `pending`
    Goal: thread command API completion — `thread.cancel` (the `open → cancelled`
      edge), typed classification + workflow profile + participant rules on
      `thread.create` (default: single-agent routing, per ADR-002), and the
      existing create/read/list/idempotency re-verified against the `.1.1.1`
      library. Backlog 15's API-shape portion; the invitation accept/decline/
      timeout semantics stay with `.1.3`.
    Backlog: 15
    Acceptance: `thread.cancel` lands on the core machine and is inspected through
      the API only; create carries the three new fields with deny-unknown typing;
      the single-agent default is stated, not an empty profile.

- ID: `PHASE-1-MAINT-1`
  Status: `pending`
  Goal: §13 same-volume locality for the ephemeral PostgreSQL cluster —
    `scripts/run_pg_tests.sh` currently defaults its data dir to
    `${TMPDIR:-/tmp}/reasonbraid-pg.XXXXXX` (off the repo's volume); re-derive
    it from the repo root (`$ROOT/target/pg-ephemeral`, gitignored).
  Defect (tracked `2026-09-06`, pre-existing from `.2.1`): the script predates
    the §13 adoption; the acceptance checklist is written when the leaf executes
    (fix = the one-line data-dir change + comment; verification = a full rerun).

- ID: `PHASE-1.2`
  Status: `proposed`
  Goal: Rust node with SQLite journal, enrollment, lease/presence, reconnect, durable inbox
  Backlog: 11–14

- ID: `PHASE-1.3`
  Status: `proposed`
  Goal: invitation/subscription semantics — explicit participants, invitations
    accept/decline/timeout, simple subscriptions (the create/read/list/cancel API
    shapes are owned by `.1.1.3`)
  Backlog: 15, 16

- ID: `PHASE-1.4`
  Status: `proposed`
  Goal: two genuinely distinct harness adapters where access permits, plus deterministic fakes for CI
  Backlog: 19–22
  Note: second real adapter may land here if Phase 0 deferred it

- ID: `PHASE-1.5`
  Status: `proposed`
  Goal: structured contributions, phases/rounds, evidence attachments, manual close, honest inconclusive outcome
  Backlog: 17

- ID: `PHASE-1.6`
  Status: `proposed`
  Goal: basic Web UI/CLI for threads, nodes, inbox, budgets, audit timeline
  Backlog: 18

- ID: `PHASE-1.7`
  Status: `proposed`
  Goal: local/LAN deployment packaging and one-command development environment

- ID: `PHASE-1.8`
  Status: `proposed`
  Goal: G1–G2 exit + Demonstration A (two hosts, blind contributions, kill-after-dispatch → ambiguous, duplicate delivery → one effect, inconclusive allowed)
  Acceptance: no manual relaying; restart/reconnect loses no accepted command; spend/uncertainty visible; inspectable via CLI/UI not database surgery
  Gate: G1, G2; subtraction record required

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-1.1.2` | `pending` | `.1.1.1` is done (the aggregate library is the write path the identity store and the thread API compose over); migration 0007 gives identity its first-class rows — backlog 10 |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.3, §26.1, backlog 9–22.
- `2026-09-06`: Opened by the Phase 0 go — ADR-002 `accepted` (signed by the accountable owner, `PHASE-0.8.2`); `.1` unblocked.
- `2026-09-06`: `.1` decomposed into `.1.1.1` (aggregate/event/outbox library — backlog 9, ADR-004), `.1.1.2` (migration 0007 identity store — backlog 10), `.1.1.3` (thread command API completion — backlog 15's API-shape portion; the invitation semantics stay with `.1.3`); `.1.3`'s goal reworded to remove the double-claim of backlog 15; frontier → `.1.1.1`.
- `2026-09-06`: `.1.1.1` done — ADR-004 accepted; defect leaf `PHASE-1-MAINT-1` opened (§13 gap in `run_pg_tests.sh`); frontier → `.1.1.2`.

## Acceptance Checklist (PHASE-1.1.1)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/agg.rs` (new),
`src/tx.rs`, `src/lib.rs`, `src/api.rs`, `src/threads.rs` (comment), the new
`tests/aggregate_library.rs`, and `scripts/run_pg_tests.sh` (all match `\.rs$`/`\.sh$` in
`.doctrine/code_paths.txt`). Enforced by the `TASK-ACCEPTANCE` doctrine.

- [x] **REPRODUCE / ISSUE** — backlog 9 ("aggregate transaction library: revision checks,
  events, outbox, authorization/audit context, and test helpers") is open, and the WP2
  six-write machinery still lives inline in `tx.rs` with no reusable typed surface and no
  revision precondition. `git log -S 'claim_idempotency_in_tx' --oneline --
  crates/reasonbraid-server/src/tx.rs` → `35f395d REASONBRAID-PHASE0-0022 (leaf
  PHASE-0.6.1): WP6 control API + CLI — …` (the claim split; the writes themselves landed
  with `REASONBRAID-PHASE0-0013`, `.2.1`).
- [x] **ROOT CAUSE (WHY + WHERE)** — the machinery was proven INLINE because WP2's job was
  the proof, not the abstraction; every future aggregate (`.1.1.2` identity, `.1.1.3`
  thread completion) would re-derive the six writes or grow `tx.rs` special cases. The
  extraction point is the whole of `apply_fresh_in_tx` (tx.rs, `git log -S 'apply_fresh_in_tx'
  --oneline` → `.2.1` + `.6.1` commits) — the writes are already one auditable body; the
  library makes them THE body (`agg.rs` 427 lines; the shim shrank `tx.rs` 272 → 224 lines
  and owns no SQL).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no library module, no
  revision precondition, rejections stored via inline SQL in `api.rs`. After:
  `reasonbraid-server::agg` is the single write path (claim → locked head → event → state →
  outbox → result, one transaction; `expected_revision` precondition default-off); `tx` is a
  SQL-free shim; `store_rejection` rides `agg::store_result_in_tx`. Live proof through the
  library's own surface: `bash scripts/run_pg_tests.sh` → `test result: ok. 4 passed; 0
  failed` (`aggregate_library`: fresh-apply vs replay, hash conflict, revision precondition
  hold/refusal, outbox→event integrity) against live PostgreSQL 16.15. Offline:
  `cargo test --all` → every suite green (the server unit suite `test result: ok. 5 passed`
  includes the new `agg::tests`).
- [x] **NO REGRESSION** — `cargo test --all` → all offline suites green (PG-gated suites
  skip by design); `bash scripts/run_pg_tests.sh` → all eight live server suites green
  (`test result: ok.` 4 + 5 + 9 + 5 + 7 + 13 + 6 + 7 `passed`) + the real-binary CLI e2e
  `test result: ok. 2 passed` + the two-host demo `ALL acceptance checks passed` (12 PASS
  checks, `rc=0`); `cargo clippy --all-targets --all-features -- -D warnings` → clean;
  `make gate` → `=== all doctrines green ===` (13/13) at commit.
- [x] **FIX** — `src/agg.rs` (the library: `claim_in_tx`/`apply_fresh_in_tx`/`apply_in_tx`/
  `apply`/`store_result_in_tx`, `AggregateCommand`/`AggregateOutcome`/`AggregateError` with
  the revision precondition), `src/tx.rs` (compat shim — no SQL, documented `expected_revision`
  invariant with an `unreachable!` arm), `src/lib.rs` (`pub mod agg`), `src/api.rs`
  (rejection store → `agg::store_result_in_tx`), `tests/aggregate_library.rs` (4 live-PG
  proofs, 261 lines), `scripts/run_pg_tests.sh` (suite registered).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted → `docs/decisions/2026-09-06_aggregate-library.md` gained `answers:`), MEMORY,
  LIVE_STATUS, this tree's logs below, `docs/TASK_TREE.md` frontier, ADR-004 +
  `docs/adr/INDEX.md` — same commit.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-09-06` | `PHASE-1.1.1` | `cargo clippy --all-targets --all-features -- -D warnings` → clean; `cargo test --all` → every offline suite green (server unit suite `test result: ok. 5 passed` incl. the new `agg::tests`); `bash scripts/run_pg_tests.sh` → all eight live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 7 + 13 + 6 + 7 `passed`) + CLI e2e `2 passed` + two-host demo `ALL acceptance checks passed` (12 PASS, `rc=0`); `make gate` → 13/13 | aggregate/event/outbox library landed; ADR-004 accepted |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-1.1.1` | `REASONBRAID-PHASE1-0002` | `agg` library + `tx` shim + `tests/aggregate_library.rs` + ADR-004; zero call-site churn |
