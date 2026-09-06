# The control API and the CLI: one transaction per thread command — claim, authorize, validate, apply; inspection without database surgery

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.6.1` (WP6 CLI — enroll, create thread, invite, contribute, challenge, revise, close, inspect)
- answers: what is the Phase 0 command surface the CLI drives, how does each thread command flow through the WP2 transaction and the WP5 authorization engine, and how is state inspected without database surgery?

## The fact / decision

`crates/reasonbraid-server/src/threads.rs` lands the thread DOMAIN, `src/api.rs` the
control API (axum), `src/bin/rb-server.rs` the control-plane binary, and the new
`crates/reasonbraid-cli` crate the `rb` binary. The operation catalogue is
`thread.create` · `thread.invite` · `thread.contribute` · `thread.challenge` ·
`thread.revise` · `thread.close`, each with a typed `deny_unknown_fields` body and a
named event (`thread.created`, `thread.participant_invited`,
`thread.contribution_submitted`, `thread.challenge_posted`,
`thread.revision_submitted`, `thread.closed`).

- **One transaction per command** (`run_thread_command`): idempotency claim →
  `authorize_in_tx` (the audit row commits with whatever happens next) → domain
  preparation against the LOCKED projection (`FOR UPDATE` — the same row
  `apply_fresh_in_tx` re-reads, so the validated state and the written version can
  never diverge) → the `.2.1` durability writes + the budget ceiling (create) →
  commit. `tx.rs` gained the `claim_idempotency_in_tx` / `apply_fresh_in_tx` split
  (public `apply_command` behavior unchanged, its tests stay green).
- **Rejections are idempotent results**: a denial or a domain rejection stores
  `{"ok": false, "error": {code, message}}` in the idempotency row; a replay
  reproduces the ORIGINAL status and body via the stable code→status map, and a
  replayed success returns the original result with `"replayed": true` added — the
  stored value itself is never mutated. A replay never re-validates against state
  the original command may have since changed.
- **The request hash** (§9.2) is SHA-256 over `operation` + the presented principal +
  the canonical (struct-field-order) body JSON: the same key with a different body
  is a typed `idempotency_mismatch` conflict.
- **Dev-profile identity**: the `x-reasonbraid-principal` header carries the
  presented principal (`hpr_…`/`rol_…`) — trusted and documented (no certificate
  issuer in Phase 0). The audit actor is derived deterministically via
  `actor_handle_for_subject` (UUIDv5 over the subject description, `agt_`), so audit
  rows for the same principal stay linkable; the SHA-1 inside UUIDv5 is namespacing,
  not a security boundary. A missing/malformed header is `unauthenticated`.
- **Authority mapping**: challenge and revise are `thread_contribute` grants (the
  WP5 action registry stays frozen; the three content verbs differ in message kind
  and event type, not authority — THREAD-004); close is the new `thread_close`
  action (the one core registry addition); inspect reads are `thread_inspect` and
  are AUDITED TOO (the audit view is complete).
- **Dev thread rules**: the creator is an `accepted` participant from creation; the
  first contribution auto-accepts an `invited` role (`invited → accepted` via the
  core machine); content verbs require the thread `open` and the actor a
  participant in `invited`/`accepted`; close folds `open → closing → closed` (both
  core edges validated, one terminal event); challenges target a contribution of
  the same thread, revisions a challenge (wrong/missing targets are typed
  `invalid_command`); closure preserves contributions and the unresolved register
  (`open_challenges` in the projection).
- **The projection** (`aggregate_state.state`) keeps only what validation and
  inspection need: state, participants, counters, close reason, ceiling id, budget.
  Content lives in the event log; `inspect` reconstructs state + ordered events +
  authorization records through `GET /v1/threads/{id}`, `/events`, `/audit` — the
  `.6.1` acceptance, mechanically proven by the e2e suite which drives the REAL `rb`
  binary and never touches the database.
- **Enroll bootstrap** (`POST /v1/enrollments`, `migrations/0006`): a human without a
  tenant bootstraps tenant + boundary + admin grant + enrollment row in ONE
  transaction (the `.5.1` authority writers gained executor-generic in-tx
  variants); a role enrolls into an existing tenant with a requested action set
  (default `thread_contribute`). (tenant, kind, name) is unique, so a re-run returns
  the ORIGINAL principal id (`replayed: true`). **Dev grants are coextensive with
  their boundary's validity window** — the first live run hit a real subset-rule
  violation: a grant issued microseconds after its boundary "outlived" it
  (`grant.expires_at > boundary.expires_at`), the same wall-clock-skew class the
  `.5.1` fixtures exposed, and the checker refused it exactly as designed.
- **The CLI keeps its state repo-local** (§13): `./.reasonbraid-cli` by default
  (`REASONBRAID_CLI_STATE`), server `http://127.0.0.1:4310` (`REASONBRAID_SERVER`);
  the e2e scratch dirs live under `target/`. Every invocation uses a fresh opaque
  idempotency key (§9.2).

## Why

KICKOFF WP6 / issue 12: "Implement CLI flow: enroll, create thread, invite,
contribute, challenge, revise, close, inspect", with the acceptance "no database
surgery required to inspect state" — the human surface of the two-host vertical
slice (`.6.2` wires the node half: server→inbox dispatch and node result→thread,
reusing this same command surface and transaction flow).

## Measured behavior (legs: re-derive · falsify · durable)

- Re-derived live (PostgreSQL 16.15): `cargo test -p reasonbraid-server --test
  command_api` → `test result: ok. 7 passed` — the full flow (bootstrap → create →
  invite → auto-accepted contribution → challenge → revise → close → inspect with
  the ordered 6-event timeline and 8 audit records, each with a 64-hex digest);
  denials recorded and effect-free; replay returns the original result and
  conflicts are typed; double-close and post-close contribution are
  `invalid_transition`; forged authoritative fields rejected (422, the field
  named); missing/malformed principal headers `unauthenticated`; challenge targets
  checked. `cargo test -p reasonbraid-cli --test cli_end_to_end` →
  `test result: ok. 2 passed` — the REAL `rb` binary drives the whole flow against
  the in-process API and inspects everything through its own stdout; typed denials
  and `scope_hidden` surface on stderr with exit 1.
- Falsified: (1) the dev-grant window overrun above — the checker caught the
  bootstrap bug before it shipped; (2) the CLI's inspect URL joined `/events` AFTER
  the query string (`?tenant_id=…/events`), which the e2e run caught; both fixed
  with regression tests in the same leaf.
- Durable: model + API + binary + migration + both suites are tracked;
  `scripts/run_pg_tests.sh` and the `pg-tests` CI job run the command-API and the
  CLI e2e suites against a live Postgres.

## Rejected designs

- **Client-supplied thread ids** — aggregate identity is server-assigned (the
  client proposes intent, never the aggregate); a create replay returns the
  original id from the idempotency row.
- **Re-validating on replay** — would let a replayed command fail against state the
  original changed (e.g. replaying a contribution after close); the claim runs
  FIRST and returns the stored result without touching the domain.
- **Separate actions for challenge/revise** — they are typed CONTENT submissions
  (THREAD-004); adding registry actions for every message kind would churn the
  authority surface the WP5 acceptance froze.
- **A separate transaction for the budget ceiling** — a thread must exist with its
  ceiling (`BUDGET-003`); the ceiling insert rides the command's transaction via
  `create_ceiling_in_tx`.
- **Storing only successes in idempotency** — a rejection is the command's semantic
  result too; storing it makes replays honest (same status, same body) and keeps
  the key's history complete.
- **An `accept` verb for invitations** — the demo's first contribution is the
  accept (the core `invited → accepted` edge is applied at that moment, not a
  guess).

## How to apply

- New thread operations extend `crates/reasonbraid-server/src/threads.rs`
  (typed body + event + projection transition) and the CLI's verb set in lockstep;
  the transaction flow in `run_thread_command` is the single path — do not bypass
  it with a pool-based shortcut.
- Keep the dev-trust boundary visible: anything security-relevant must say "dev
  profile" in its doc until WP7 workload identity lands.
- The `.6.2` demo wires the node channel INTO this surface (inbox dispatch +
  node-result events); the transaction flow and the projection are the contract it
  builds on.
