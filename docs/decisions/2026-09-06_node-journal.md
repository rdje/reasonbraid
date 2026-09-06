# The WP3 node journal: the boundary record precedes the dispatch, and ambiguity is recovered — never guessed

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.3.1` (WP3 SQLite node journal + inspection CLI)
answers: how does the node journal make a crash after a possible provider dispatch honest — `outcome_unknown` unless the adapter proves the result — while keeping WAL plus an explicit, inspectable durability profile and an operator CLI that never opens SQLite by hand?

## The fact / decision

`crates/reasonbraid-node` lands the WP3 journal (`src/journal.rs`; schema
`migrations/0001_node_journal.sql`, embedded in the crate): SQLite through **sqlx** (the same
driver stack as the server's PostgreSQL side), with WAL + `synchronous=FULL` + `busy_timeout` +
`foreign_keys` applied and VERIFIED on the live connection at open, and recorded in the
`journal_meta` table (pragmas are per-connection; the meta row is what the read-only CLI
reports).

- **Boundary-before-boundary ordering** (§17.4): `record_dispatch` commits
  `prepared → dispatched` (with `provider_request_id` when known) BEFORE the caller invokes
  the adapter — a test proves a SECOND connection already sees the boundary record.
- **Recovery** (`recover`) reclassifies every attempt still `dispatched` (boundary durably
  crossed, no recorded result) as `outcome_unknown`; attempts still `prepared` are reported
  `safe_to_redeliver` (classification only — the boundary was never crossed).
- **The only exits from ambiguity are proof and adjudication:** `prove_result` lands
  `outcome_unknown → completed | failed_known` with the adapter's evidence (the §11.3
  provider-lookup recovery edges), and `reconcile` records the authorized `reconciled`
  terminal. Nothing retries silently.
- **Every status change goes through `reasonbraid-core`'s machine**, extended in this leaf:
  the `failed_known` state plus the edges `(dispatched, fail_known)` and
  `(outcome_unknown, complete | fail_known)`. This supersedes the `.1.3` note that
  `failed_known` is out of Phase 0 scope — a PROVEN failure (a definitive runtime rejection
  or a provider status-lookup hit) is not a guess, and the `.1.3` exclusion was about
  guessing. `cancelled_known` stays out of Phase 0.
- **Dedupe primitives for `.3.2`:** `commands.command_id` PK + `operations.command_id`
  UNIQUE, so a duplicated command never creates a second local operation;
  `attempt_transitions` is the before/after boundary ledger (§11.4); `outgoing_events`
  carries acknowledgement state plus the `ack_cursor` recorded at acknowledgement.
- **`rb-journal`** (the crate's binary) opens the journal READ-ONLY
  (`SQLITE_OPEN_READONLY`) and offers `inspect` (profile + `quick_check` + counts),
  `pending` (in-flight attempts + unacked events), and `ambiguous` (outcome_unknown
  attempts with their boundary history), each with `--json`. WAL mode lets it run beside a
  live node.

## Why

KICKOFF WP3 and `ROADMAP.md` §11.3–§11.4/§17.4 require exactly this. WAL alone is not a
power-loss guarantee (§11.4), so `synchronous=FULL` is the conservative default — and the
profile is RECORDED, not merely set, because a per-connection pragma is invisible to any
other connection that opens the file. The ambiguity acceptance ("a crash after possible
provider dispatch yields `outcome_unknown` unless the adapter can prove the result") is
impossible without a durable boundary record that precedes the dispatch: writing it after
the call would make "crashed mid-call" and "never called" indistinguishable, and marking a
`prepared` attempt ambiguous on recovery would lie about a boundary that was never crossed
(and poison the ambiguity signal operators rely on).

## Measured behavior (legs: re-derive · falsify · durable)

- Re-derived: `cargo test -p reasonbraid-node` → `test result: ok. 13 passed` (journal
  unit) + `test result: ok. 6 passed` (CLI) + `test result: ok. 10 passed` (kill points) —
  the WAL/FULL profile applied AND recorded; the boundary record visible to a second
  connection before the adapter runs; KP-1…KP-9 sweep every `.3.1` seam (before command →
  nothing persisted, between command and operation, after prepare → `safe_to_redeliver`,
  after dispatch → `outcome_unknown`, proven lookup → `completed`, after result →
  terminal, after ambiguity → stable, between event emission and ack, after ack →
  terminal); dedupe; invalid moves rejected by the core machine; garbage files fail
  cleanly. `cargo test -p reasonbraid-core` → `test result: ok. 24 passed; 0 failed; 1 ignored`
  (the extended machine's exhaustive tables).
- Falsified: the CLI's read-only claim is not asserted, it is exercised — every
  inspection is followed by a byte comparison of the journal file, and the CLI is run
  beside a live writer. The durability claim is likewise exercised by crash tests that
  drop the handle with no checkpoint; each transition is its own transaction, so what
  survives is exactly what a killed process would leave.
- Durable: the producer (this crate + its embedded migration) is tracked; the proof
  commands are `cargo test -p reasonbraid-node` / `make check`, which plain CI runs (no
  service needed — the journal is a file).

## Rejected designs

- **Rusqlite** — a synchronous driver in a Tokio node process: the sync calls would need
  `spawn_blocking` scaffolding, plus a hand-rolled migration runner (or an extra crate)
  where `sqlx::migrate!` is already proven in this repository. sqlx keeps ONE driver stack
  across the control plane (Postgres) and the node (SQLite).
- **Journaling the dispatch AFTER the adapter call** — collapses the ambiguity window: a
  crash mid-call leaves no boundary record, so recovery cannot distinguish "maybe
  dispatched" from "never dispatched". The boundary record must precede the call.
- **Auto-classifying `prepared` attempts as ambiguous on recovery** — the boundary was
  never crossed; claiming ambiguity would forbid a redelivery that is actually safe.
- **DELETE journal mode or relying on WAL alone** — §11.4 explicitly: "WAL alone is not a
  power-loss guarantee"; FULL synchronous is the conservative dev-profile default.
- **Reading pragmas for the operator health view** — `synchronous` is per-connection, so
  the CLI would report its own connection's value; `journal_meta` records the writer's
  profile instead.
- **Backoff (`next_eligible_at`) or retention machinery in the journal now** — no retry
  policy exists yet (Phase 2 owns it); retention is unconfigured in the dev profile and
  documented as such.
- **A second root-level migrations directory for the journal** — the repository-root
  `migrations/` is the control-plane PostgreSQL schema; the journal schema ships embedded
  in the node crate (node-local state, §17.1) and must never run against Postgres.

## How to apply

- Never invoke an adapter before `record_dispatch` has committed — that boundary rule is
  what the whole ambiguity contract rests on.
- A journal status change goes through `reasonbraid-core`'s `apply`; a new journal state
  means adding the machine edge first, then the migration.
- Journal migrations live in `crates/reasonbraid-node/migrations/`, are append-only, and
  bump `PRAGMA user_version` in the same file.
- The inspection CLI opens read-only and stays that way — inspection must be unable to
  mutate the journal.
- Node durability/retention policy (disk-full handling, checkpoint policy, encryption,
  retention) is a documented dev-profile default; Phase 2's retry policy and WP8's ADR set
  own the production decisions. See [[2026-09-06_state-transitions]] for the machine this
  record extends and [[2026-09-06_atomic-transaction]] for the server-side counterpart.
