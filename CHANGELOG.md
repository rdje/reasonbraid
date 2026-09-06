# CHANGELOG.md

## 2026-09-06 — WP4 first real harness: the Codex-family CLI behind `codex exec --json` (`PHASE-0.4.2`)

- Landed `CodexCliAdapter` (`crates/reasonbraid-adapter/src/codex.rs`): the first REAL adapter supervises `codex exec --json --skip-git-repo-check --ephemeral --sandbox read-only <prompt>` as a child process — the narrowest supported machine interface (§11.6), qualified against codex-cli 0.153.4 (Apache-2.0, verified from the primary source).
- **Qualified LIVE**: one bounded real dispatch (`"Reply with exactly: ok"`) through the real supervisor + journal — `test result: ok. 1 passed`. The JSONL stream maps to the contract: `thread.started` → `ProviderRequestId` (the thread id attached to the attempt as its proof handle — the contract gained this event because Codex reveals the handle AFTER dispatch), `item.completed` → output chunks, `turn.completed` → `Completed` with an exact token receipt, non-zero exit → `failed_known` with the stderr tail.
- The acceptance's honest legs hold on the REAL harness: **status lookup is genuinely `Unsupported`** (no first-class query for a past attempt), so a lost response lands `outcome_unknown` with NO retry language; cancellation is `BestEffort` (kill the child); receipts report tokens, never cost → normalized cost stays unknown, never zero.
- Offline supervision suites (stub binary, no spend): `codex_adapter` 9 passed + `supervisor_codex_stub` 2 passed — spawn refusal, JSONL parsing, chunk order, exit verdicts, lost response, kill, receipt shapes, the full supervisor flow.
- The dependency ledger's Codex row was revalidated (its own trigger fired at this spike): `checked_at 2026-09-06`, `tested_versions ["0.153.4"]`, license, transports, auth modes, semantic losses, and conformance results all filled from the probes.
- Evidence report `docs/evidence/2026-09-06_codex-adapter-qualification.md` (`reported`) answers the WP4 acceptance's evidence-report leg: the second adapter (Claude-family) is recommended for **Phase 1** — director-owned open question, recorded in the decision record. **WP4 complete; frontier is `PHASE-0.5.1`.**

## 2026-09-06 — WP4 fake harness adapter + execution supervisor (`PHASE-0.4.1`)

- Landed `crates/reasonbraid-adapter` — the harness adapter boundary (`KICKOFF.md` §3). The `Adapter` contract (`ROADMAP.md` §11.2) declares capabilities (streaming, cancellation strength, provider idempotency, status lookup, tool support, policy-injection mode), makes **dispatch acknowledgement distinct from completion** (`Accepted(ack, handle)` → streamed chunks → terminal event), carries **no credential field**, and treats an unsupported status lookup as an honest fact — never a retry recommendation.
- The deterministic `FakeAdapter` (`§11.6` conformance oracle): per-operation scripts (`emit_chunk`, `malformed_output`, `complete`, `fail_known`, `fail_before_dispatch`, `hang_forever`, `ignore_cancellation`, `lose_response`) with **no sleeps** — the hang is a cancellation `Notify`, so the same script yields the same event sequence every time.
- The sanitized outcome corpus (`fixtures/`, 10 files): mechanically credential-scanned, coverage-checked (every step and outcome class must appear), and replayed end to end.
- The node's **execution supervisor** (`src/supervisor.rs`): `execute_attempt` journals `prepared` → the dispatch boundary → `invoke` → the honest terminal (`failed_before_dispatch | completed | failed_known | outcome_unknown`). A lost response without a lookup lands `outcome_unknown` with an error carrying **no retry language**; a proven lookup lands the result with the provider handle attached.
- One core machine extension: the proof-gated `(dispatched, fail_before_dispatch) → failed_before_dispatch` edge — the conservative pre-invoke boundary record is corrected when the adapter CERTIFIES no dispatch began (the inverse of the §11.3 proof edges).
- The conformance corpus did its job on first replay: it caught a real design conflict (boundary-vs-refusal), then probes caught two more real bugs — a `notify_waiters` scheduling race (fixed with `notify_one`, which stores a permit) and the supervisor pulling the stream past a terminal event (now it breaks). All recorded in the decision record's falsified leg.
- Proven: adapter `12 passed` + corpus `3 passed`; node supervisor `8 passed`; `make check`/`make gate` 13-13/`make deny`/`make secret-scan`/`make book` green; live-PG suite re-proven (5 + 7 + 13). No new dependency allowances needed.
- Recorded `docs/decisions/2026-09-06_fake-adapter.md` (`answers:` present). The mdBook gains the adapter-boundary chapter. **Frontier is `PHASE-0.4.2`.**

## 2026-09-06 — WP3 outbound node channel with cursor resume + reconciliation handshake (`PHASE-0.3.2`)

- Landed the WP3 channel on both sides. **Server** (`crates/reasonbraid-server/src/node_channel.rs` + `migrations/0003_node_inbox.sql`): a durable per-node inbox (`node_inbox` — monotonic per-node cursor, acknowledgement state) and deduplicated node-event receipts (`node_events`, keyed on the node-assigned event id); axum routes `POST /v1/nodes/handshake`, `POST /v1/nodes/events`, `POST /v1/nodes/ack`, `GET /v1/nodes/poll` — versioned and `deny_unknown_fields`-strict.
- **Node** (`reasonbraid-node`: `src/channel.rs` reqwest client, `src/node.rs` lifecycle facade): `Node::reconcile` runs the reconnect protocol end to end — recover crashed attempts, report the node's durable resume facts, journal the replay (deduplicated), apply directives, re-emit pending events with ORIGINAL ids (skipping the ones the server reports as held), acknowledge the cursor both sides — and only then becomes `Schedulable`. `emit_event` is refused before reconciliation; any failure returns the node to `Offline`, and the protocol is idempotent to retry.
- The acceptance, proven live (PostgreSQL 16.15 + real `127.0.0.1` sockets, 13 channel tests): **reconnect exchanges the last acknowledged cursor + pending operation ids** (tail-only replay; the pending-operation exchange is load-bearing — known events are not re-sent); **a duplicated command never creates a second local operation** (crash-window cursor rewind → identical operation ids); **the node is not schedulable until reconciliation completes** (emit refused before reconcile; failed reconcile stays unschedulable).
- Reconciliation from receipts: ambiguous attempts are `adjudicated` (→ `reconciled`) when the server holds the operation's event, `needs_adjudication` (stays `outcome_unknown`, bounded) otherwise. A node reporting a cursor ahead of the server's ledger is refused with a typed `version_conflict` — a journal-lost-class anomaly, never a silent re-base.
- WP3 exercises proven: server restart resumes from the durable inbox; network loss leaves the node unschedulable until a successful reconcile; double event emission → one receipt; version mismatch (400) and forged fields (422) rejected; poll tail for live delivery.
- Node journal gains the channel state (`0002_node_channel.sql`: the acknowledgement cursor, `pending_operations`, `known_events` acknowledgement); `scripts/run_pg_tests.sh` and the CI `pg-tests` job now run all three suites (5 + 7 + 13 live). `make deny` passes with the axum + reqwest trees (no new allowances).
- Recorded `docs/decisions/2026-09-06_node-channel.md` (`answers:` present). The mdBook gains the node-channel chapter. **WP3 complete; frontier is `PHASE-0.4.1`.**

## 2026-09-06 — WP3 SQLite node journal + inspection CLI (`PHASE-0.3.1`)

- Landed `crates/reasonbraid-node` — the first node crate (`KICKOFF.md` §3). The WP3 journal is SQLite through sqlx (the same driver stack as the server's Postgres side): WAL + `synchronous=FULL` + busy-timeout + foreign keys, applied and VERIFIED on the live connection at open, and recorded in `journal_meta` so the profile is inspectable (`ROADMAP.md` §11.4: "select and document synchronous mode" — WAL alone is not a power-loss guarantee).
- **Boundary-before-boundary ordering** (§17.4): `record_dispatch` commits `prepared → dispatched` (with the provider request id when known) BEFORE the adapter is invoked — a test proves a second connection already sees the boundary record. Crash recovery is therefore honest: `dispatched` → `outcome_unknown` (`recover`), `prepared` → `safe_to_redeliver` (never lied about as ambiguous).
- **The only exits from ambiguity are proof and adjudication:** `prove_result` (adapter status lookup → `completed`/`failed_known`, the §11.3 provider-lookup edges) and `reconcile`. The core machine was extended for this — the `failed_known` state plus `(outcome_unknown, complete|fail_known)` — superseding the `.1.3` "failed_known out of scope" note: a PROVEN failure is not a guess (`cancelled_known` stays out).
- Dedupe primitives for `.3.2`: `commands.command_id` PK + `operations.command_id` UNIQUE (a duplicated command never creates a second local operation), the `attempt_transitions` before/after boundary ledger (§11.4), and `outgoing_events` with acknowledgement state + `ack_cursor`.
- `rb-journal` inspection CLI — read-only by construction (`SQLITE_OPEN_READONLY`, proven non-mutating by byte comparison) and WAL-concurrent beside a live node: `inspect` (profile + `quick_check` + counts), `pending`, `ambiguous` (with boundary history), `--json` for scripting.
- Proven: 13 journal unit + 6 CLI + 10 kill-point tests (KP-1…KP-9 sweep every `.3.1` seam by dropping the journal handle mid-flight, no checkpoints, no sleeps); core 24 passed. All gates green — `make check`, `make gate` 13/13, `make deny` (clap + libsqlite3-sys tree; the path dependency is pinned `version = "0.1.0"` to satisfy the wildcard ban), `make secret-scan`, `make book`; the PG suite re-run green (`5 passed` + `7 passed`).
- Recorded `docs/decisions/2026-09-06_node-journal.md` (`answers:` present). The mdBook gains its first operator-facing chapter (`docs/book/src/node-journal.md`). **Frontier is `PHASE-0.3.2`.**

## 2026-09-06 — WP2 leased outbox worker with fencing (`PHASE-0.2.2`)

- Landed `crates/reasonbraid-server/src/outbox.rs`: the worker loop is three phases, each its own commit — `claim_ready` (one atomic `UPDATE … FOR UPDATE SKIP LOCKED` leasing ready items with a fresh `gen_random_uuid()` fencing token, expiry, and incremented `attempt`), `deliver` (deduplicated `outbox_delivery` sink keyed on `event_id`), and `complete` (acknowledges only with the CURRENT token AND a live lease — otherwise `LeaseLost`, nothing written). The lease clock is caller-supplied (`chrono` ↔ `TIMESTAMPTZ` via sqlx), so kill-point tests advance expiry deterministically with no sleeps.
- Added `migrations/0002_outbox_worker.sql` (0001 stays immutable): `lease_owner`/`lease_token`/`lease_until`/`attempt` with an all-or-nothing CHECK + claim index, and the `outbox_delivery` sink whose FK chain (`→ outbox → event_log`) makes a delivery effect imply a durable event.
- Proved the acceptance against live PostgreSQL 16.15: `scripts/run_pg_tests.sh` → `7 passed` (outbox worker) + `5 passed` (atomic transaction). Tests: exclusive claim under concurrent workers, re-claim after expiry issues a new token, **stale worker refused after a newer fencing value**, expired lease refused even with a matching token, and kill points 3/4/5 (after claim / after delivery / after ack) recovering to exactly one domain effect.
- First live run failed 7/7 and the failure was root-caused with a probe (TOOLBOX): the tests share one queue, and parallel tests plus the `atomic_transaction` binary's leftover rows were claimed by each test's global oldest-first claim. The suite now serializes under a module-level async mutex and purges the queue under the guard.
- Server `Cargo.toml` gains `chrono` + sqlx `chrono` feature (both permissive-licensed; `make deny` re-verified ok). `run_pg_tests.sh` and the CI `pg-tests` job run both integration binaries; `docs/ci.md` updated.
- Recorded `docs/decisions/2026-09-06_outbox-worker-fencing.md` (`answers:` present; measured behavior + rejected designs). **WP2 complete; frontier is `PHASE-0.3.1`.**

## 2026-09-06 — WP2 atomic transaction (`PHASE-0.2.1`)

- Landed `crates/reasonbraid-server` — the first control-plane crate (`KICKOFF.md` §3). `apply_command` writes the four durability tables (`idempotency`, `event_log`, `aggregate_state`, `outbox`) in **one** `BEGIN … COMMIT`, proving the WP2 acceptance against a live PostgreSQL 16.15.
- Claim-first idempotency (`INSERT … ON CONFLICT DO NOTHING` on the `(tenant_id, idempotency_key)` primary key): a redelivery with the same key+hash replays the *original* stored result; a different hash is `IdempotencyConflict`. Transport redelivery produces exactly one domain effect.
- Schema lives in repository-root `migrations/0001_atomic_transaction.sql` (outbox carries a FK to `event_log`, so an outbox item implies its event is durable); applied via `sqlx::migrate!`.
- Proof harness: `scripts/run_pg_tests.sh` (ephemeral `initdb`/`pg_ctl` server, no background service) + a `pg-tests` GitHub Actions job. Tests skip offline (`DATABASE_URL` unset) so `make check` stays green.
- `deny.toml` corrected for cargo-deny 0.20: `[advisories].unmaintained` is a scope (not a lint level), `BSD-3-Clause` added for `subtle`, and `getrandom`/`hashbrown`/`syn` `skip` entries for the reviewed sqlx-tree duplicates. `make deny` → advisories/bans/licenses/sources ok; `make secret-scan` → no leaks; `make book` builds.
- Recorded `docs/decisions/2026-09-06_atomic-transaction.md` (`answers:` present).

## 2026-09-06 — WP1 typed errors + reason-code registry (`PHASE-0.1.4`)

- Added `src/error.rs` to `reasonbraid-core`: `KnownReasonCode` (the complete §9.8 registry, 20 codes, snake_case), `ReasonCode` (wraps known codes and preserves unknown codes verbatim via `Unknown(String)`), `Retryability` (tri-state), and `DomainError` (code + retryability + safe message + optional correlation/details).
- Unknown codes round-trip: a code this build does not recognize deserializes to `ReasonCode::Unknown(raw)` and re-serializes to the same string — the WP1 "unknown codes remain preservable" acceptance.
- `From<TransitionError> for DomainError` classifies state-machine rejections as `invalid_transition`, wiring the reason-code registry to `.1.3`'s deterministic `apply`.
- Recorded `docs/decisions/2026-09-06_reason-codes.md` (`answers:` present). **WP1 (minimal contracts) is complete.**

## 2026-09-06 — WP1 minimal state machines (`PHASE-0.1.3`)

- Added three minimal orthogonal lifecycles to `reasonbraid-core` (`src/state.rs`): `ThreadState` (`open`/`closing`/`closed`/`cancelled`), `ParticipationState` (`invited`/`accepted`/`declined`/`expired`/`left`), and `ProviderAttemptState` (`prepared`/`dispatched`/`completed`/`failed_before_dispatch`/`outcome_unknown`/`reconciled`). Each exposes a single fallible `apply(transition) -> Result<state, TransitionError>`; invalid moves are rejected deterministically, never panic, and never rewind history.
- Added the deferred `ProviderAttemptId` family (wire prefix `patt`), completing the WP1 "role/incarnation/run/provider-attempt cannot be confused in types" acceptance.
- Exhaustive edge-table tests cover every (state, transition) pair so an undocumented edge fails CI.
- Recorded `docs/decisions/2026-09-06_state-transitions.md` (`answers:` present): the minimal edge set, the two-step thread close, and the `outcome_unknown → reconciled` handling of indeterminate attempts (kill-risk Q4).

## 2026-09-06 — WP1 command/event envelopes (`PHASE-0.1.2`)

- Added `CommandEnvelope` (client intent), `ClientContext`, and `CommittedEvent` (server authority) to `reasonbraid-core`, with `PROTOCOL_VERSION = "reasonbraid/0.4"`. Both envelopes use `#[serde(deny_unknown_fields)]`, so a client-supplied authoritative field (actor/tenant/sequence/timestamps/authority) is rejected at deserialization, not ignored or trusted.
- Added five envelope-scoped ID families: `EventId` (`evt`), `RequestId` (`req`), `CorrelationId` (`corr`), `ActorPrincipalId` (`agt`), `AuthorizationRecordId` (`authz`).
- Added `schemars` (derive) as a dependency and a manual `JsonSchema` impl for `Id<K>`; generated JSON Schema goldens (`schema/`) with a drift test, plus golden wire fixtures (`fixtures/`) for `thread.create`/`thread.created` and a forged-command fixture.
- Recorded `docs/decisions/2026-09-06_envelope-representation.md` (`answers:` present): client expresses intent, server assigns authority.

## 2026-09-06 — WP1 strong identifiers (`PHASE-0.1.1`)

- Landed `crates/reasonbraid-core` — the first real crate (the scaffold's placeholder `crates/app` binary is removed). This is the `KICKOFF.md` §3 `reasonbraid-core`: IDs now, envelopes/thread/attempt states later.
- Implemented strong IDs as branded newtypes over UUIDv7: `TenantId`, `HumanPrincipalId`, `HostId`, `NodeId`, `AgentRoleId`, `AgentIncarnationId`, `RunId`, `ThreadId`. Distinct types *and* distinct wire prefixes (validated on deserialize), so role/incarnation/run/thread cannot be confused in code or on the wire.
- First crates.io dependencies: `serde` (derive) + `uuid` (v7); dev-dep `serde_json`. All permissive-licensed; `cargo deny` re-runs in CI on push (supply-chain workflow).
- Recorded `docs/decisions/2026-09-06_id-representation.md` (`answers:` present): the prefix table and the explicit-construction rule.

## 2026-09-06 — G0 contract drafts (`PHASE-0.0.8`)

- Added the five G0 contract drafts under `spec/`, all headed "draft — not normative": `README.md` (orientation + traceability map), `glossary.md` (frozen term distinctions), `requirements.md` (stable `ID-*`/`AUTH-*`/`THREAD-*`/`DELIV-*`/`BUDGET-*` requirement catalogue), `lifecycle.md` (orthogonal lifecycle tables), `threat-model.md` (11 trust boundaries + assets/adversaries/abuse/mitigations), and `governance/charter.md` (bootstrap human root = Richard DJE).
- Recorded `docs/decisions/2026-09-06_g0-contract-id-scheme.md` (`answers:` present): `THREAD-*` and `BUDGET-*` are added to the §19.1 prefix list to name the five §20.2 G0 boundaries; `RES-*`/`POL-*`/`SEC-*` reserved for later phases; contract drafts live in `spec/`.

## 2026-09-06 — supply-chain skeleton (`PHASE-0.0.7`)

- Added `deny.toml` (cargo-deny: advisories/bans/licenses/sources), `.github/workflows/supply-chain.yml` (cargo-deny + gitleaks secret scan), and `docs/ci.md`; the Makefile gained `make deny` / `make secret-scan`. Explicitly a *skeleton* — no SBOM, provenance, or release-signing claim.

## 2026-09-06 — accountable owners (`PHASE-0.0.6`)

- Recorded `docs/decisions/2026-09-06_accountable-owners.md`: Richard DJE is accountable for both final architecture decisions and release/security gate records (one person, both roles).

## 2026-09-06 — external dependency ledger (`PHASE-0.0.5`)

- Added `docs/dependencies/external-ledger.yaml`: `ROADMAP.md` §7.4 schema skeleton with MCP/A2A/Codex/Claude rows stubbed from the 2026-09-04 corrected baseline (§28.1).

## 2026-09-05 — live risk register (`PHASE-0.0.4`)

- Added `docs/risks.md`: Phase 0 subset of `ROADMAP.md` §25 with owner roles and stop/reframe triggers.

## 2026-09-05 — parking lot (`PHASE-0.0.3`)

- Added `docs/parking-lot.md`: non-blocking ideas need a revisit trigger or they are dropped.

## 2026-09-05 — ADR and evidence templates (`PHASE-0.0.2`)

- Added `docs/adr/TEMPLATE.md` + `INDEX.md` (shape taken from ADR-001).
- Added `docs/evidence/TEMPLATE.md` + `INDEX.md` (question, options, fixture, result, deletion plan).

## 2026-09-05 — ADR-001 uncleared working name (`PHASE-0.0.1`)

- Recorded `docs/adr/001-uncleared-working-name.md`: ReasonBraid is internal-only until professional clearance.
- README is now a ReasonBraid landing page (private repo; no public namespace claims).

## 2026-09-05 — adopt claim-verification (`RB-SEED.3`)

- Project-owned `docs/CLAIM_VERIFICATION.md` (portable architecture #5).
- Bootstrap (`CLAUDE.md`) now requires the three legs before publishing a number.
- `RB-SEED` complete; Phase 0 frontier is ADR-001.

## 2026-09-05 — convert v0.4.1 roadmap into task-trees (`RB-SEED.2`)

- Added `docs/tasks/PROGRAM.md` (phase/track/gate/backlog/ADR/demo map) and `PHASE-0`…`PHASE-9`.
- Phase 0 follows companion `KICKOFF.md` WP0–WP8 (issues 1–15) plus G0 contract drafts.
- Later phases stay `proposed` until their predecessor exit gate.

## 2026-09-05 — land ROADMAP v0.4.1 and companion KICKOFF.md (`RB-SEED.1`)

- Replaced the bedrock placeholder `ROADMAP.md` with ReasonBraid v0.4.1 (execution baseline).
- Tracked `KICKOFF.md` as the Phase 0 companion: scope/gates in `ROADMAP.md`, day-to-day Phase 0 execution in `KICKOFF.md`.
- Recorded `docs/decisions/2026-09-05_kickoff-companion-to-roadmap.md` and `docs/decisions/2026-09-05_roadmap-v0.4.1-frozen.md`.
- mdBook introduction now describes ReasonBraid rather than the template skeleton.

## bedrock-scaffold 0.6.1 — creating a project is foolproof through its first commit

`BEDROCK-MAINTENANCE.2.7`.

- ⛔ **Measured on a fresh clone of 0.6.0:** `bootstrap.sh` left the crate rename — a CODE change — with no owning
  leaf, so the new project's FIRST commit was refused by `TASK-TREE-OWNERSHIP` and `TASK-ACCEPTANCE`. A new user's
  first contact with the discipline was a refusal about a rename the tool made.
- **`bootstrap.sh` now seeds `docs/tasks/BOOTSTRAP.md`** on a fresh de-template: a done leaf that owns the bootstrap,
  its ticked checklist carrying the evidence of that very run (crate-name count before/after, hooks path, the
  enforcer's summary and verdict with `rc=0`), registered in `docs/TASK_TREE.md`, pointed to by `MEMORY.md`; and it
  prints the exact first-commit command as step 0. Idempotent.
- Proven: clone → `bootstrap.sh <name>` → the printed commit → hooks green → `make gate` green → `make check` green,
  with no hand edits. Two defects in the fix were caught by the trial itself (an enforcer run before the map
  existed; a `grep -c` fallback that split a checklist bullet).

## bedrock-scaffold 0.6.0 — four evidence and ratchet doctrines: lessons reach the retrievable layer, routings carry evidence, gap claims carry their census, tables keep their columns

`BEDROCK-MAINTENANCE.2.6`.

- **Added `LESSON-PROMOTION`**: a new dated lesson heading staged in `DEV_NOTES.md` must be promoted (a
  `docs/knowledge/` change or a `docs/decisions/` record gaining `answers:`) or explicitly declined
  (`promotion: declined (<reason>)` in the owning leaf). Pure verdict with 9 controls at import.
- **Added `ROUTING-EVIDENCE`**: a leaf that routes a finding out to another tree carries a `ROUTING EVIDENCE`
  section. Keyed on the semantics of leaving the tree; 5-arm `--self-test`.
- **Added `GAP-CLAIM-CENSUS`**: a leaf that ADDS a "nothing checks X" claim records the census it rests on in
  the same section (or `census: not run (<why>)`). Staged-diff-scoped; `--all` reports the backlog; 10-arm
  `--self-test` pinning the founding active and passive sentences.
- **Added `TABLE-ARITY-RATCHET`** (a fresh minimal implementation): a staged `.md` may not raise the number of
  table rows whose cell count disagrees with their header; code spans and escaped pipes respected; 8-arm
  `--self-test`.
- ⛔ Two defects in the ports were caught by their own RED arms before the gate ran: a heredoc that consumed
  the table detector's stdin (every arm read 0), and a `pipefail` control in lesson promotion.
- All four scripts join the `NEUTRAL` allow-list of `scripts/update_scaffold.sh`. Backlog notes record the
  input-bound principles (`BASELINE-IDENTITY`, `IDENTITY-CARRIER-CURRENCY`, `SCRATCH-SLOT-HEADER`, the full
  `LIVE-DOC-CURRENCY` instrument) for a future seam.

## bedrock-scaffold 0.5.0 — the day-one batch: no agent trailers, a handoff census, no self-reported dates

`BEDROCK-MAINTENANCE.2.5`.

- ⛔ **`COMMIT.md` had the trailer rule backwards.** It told every generated project to *end commit
  messages with the project's co-authorship trailer*; the upstream maintainer ruled the opposite on
  2026-08-22 (a commit message ends with its own last line — no agent/tool attribution trailers,
  harness-agnostic). The rule is rewritten and `.githooks/commit-msg` now refuses the known
  agent-attribution shapes mechanically; a human co-author's `Co-Authored-By:` still passes.
- **Added `scripts/check_no_background_jobs.sh`**, the handoff census: pattern-free (`lsof` over the
  caller's uid — an open handle under the repo, or a command line naming the checkout), run before
  a session ends; deliberately not a commit gate. Named in `CLAUDE.md`'s non-negotiables.
- **Added the `LIVE-DOC-CURRENCY` doctrine** (principle): no tracked `.md` reports its own currency
  (`Last updated:` and kin) — git carries it, a hand-kept date is false the day after. The field is
  deleted from `docs/tasks/TEMPLATE.md` and the maintenance tree; `scripts/check_live_doc_currency.sh`
  is structural over `git ls-files '*.md'` with a 3-arm `--self-test`.
- Both scripts join the `NEUTRAL` allow-list of `scripts/update_scaffold.sh`.
- Part 2 of the same transfer (`LESSON-PROMOTION`, `ROUTING-EVIDENCE`, `GAP-CLAIM-CENSUS`, a fresh
  `TABLE-ARITY-RATCHET`) is classified in the `.2.5` leaf and queued as `.2.6`, paused by the maintainer.

## bedrock-scaffold 0.4.0 — TASK-ACCEPTANCE: a change lands with evidence, not with a claim

`BEDROCK-MAINTENANCE.2.4`.

- **Added the `TASK-ACCEPTANCE` doctrine**: a staged CODE change must be owned by a task-tree leaf
  whose checklist has ROOT CAUSE / ADDRESSED / NO REGRESSION **ticked**, each backed by output from
  a tool that was actually run — **inside that box's own bullet**.
- ⭐⭐ **Box-scoping is the soundness property**, not a nicety. It closes two measured leakage
  holes: a co-staged, unrelated leaf supplying the evidence, and a token matched anywhere in the
  file rather than in the box it backs. `CTRL-1` demonstrates it directly — a whole-file grep
  PASSES the fixture that the shipped check REJECTS.
- **Neutral by seam, not by rename.** Default signatures are universal to any Rust project
  (`error[E1234]`, `could not compile`, `clippy::…`, `test result: ok`, panics, profilers) plus any
  project's build-flow forensics (`git log -S`, `shellcheck`, `bash -n`, `make -n`, `ENOSPC`…).
  Project-specific tooling is declared in `.doctrine/evidence_tokens.txt`, and what counts as a
  code change in `.doctrine/code_paths.txt` — both optional, both defaulted, both documented in
  `.doctrine/README.md`. ⭐ `CTRL-4`/`CTRL-4b` prove the seam is load-bearing: the same leaf passes
  WITH the declaration and fails WITHOUT it.
- ⛔ **Fixed a portability defect the probes caught**: the box extractor used `IGNORECASE`, a gawk
  extension that BSD awk silently ignores — every leaf would have been reported as having no
  checklist. Rewritten with POSIX `tolower()`.
- ⚠️ Honest limit, stated in the check itself: it proves the author cited something re-runnable,
  never that the output is true. The un-fakeable leg is re-running the cited command in CI.
- Probes 9/0; `make gate` 8/8.

## unreleased — the admission test asks about VALUE first, not vocabulary

`BEDROCK-MAINTENANCE.2.3`. Process only; no check changed, so `DOCTRINE_VERSION` is unmoved
(`MAINTAINING.md` and the maintenance tree are maintainer-only, not re-syncable spine files).

- **The admission test is now two ordered questions.** Q1 (primary, about VALUE): *does this
  objectively benefit any present and any future project?* — answered by stating what the check
  prevents using no project's nouns, then asking whether a brand-new project is better off with it
  on day one. Q2 (secondary, a filter): *can it be expressed without domain nouns?*
- ⛔ **Q2 cannot substitute for Q1.** A check can score 0 domain nouns and still encode a workflow
  only one project needs — neutral vocabulary, project-shaped substance. Q2 measures whether a
  thing CAN be neutralized; Q1 asks whether it SHOULD be. Running Q2 first waves impostors through.
- ⭐ **Measured worked example, which changed a verdict.** A "destructive automation must require
  confirmation" check scored well on Q2 and was ranked an easy win; its logic hardcodes a Makefile
  path and a `clean:` recipe, so it really offers *"benefits any project that builds with make"* —
  a conditional. **Rejected as-is.** Meanwhile `ROUTING-EVIDENCE` measures 0 build-system
  references and presumes only the task-tree system this template ships ⇒ promoted to top.
- **The portability seam to look for:** does the check presume anything beyond what bedrock ships?
  If yes, give it a project-declared seam or leave it upstream — never hardcode one project's
  answer and call it neutral.
- ✅ Retroactive audit: all four already-ported items PASS Q1. Nothing retracted.

## bedrock-scaffold 0.3.0 — WAIVER-ROUTING, and the neutrality bar for every future port

`BEDROCK-MAINTENANCE.2.2`.

- **Added the `WAIVER-ROUTING` doctrine** (`scripts/check_waiver_routing.sh`): a task leaf saying a
  gate DOES NOT APPLY must name the leaf that owns fixing the gate. ⭐ An author writing a waiver
  IS the gate reporting a missing capability — the highest-signal defect report a gate can get.
  Deliberately does **not** punish honesty: the waiver stays legal, it just has to name an owner.
- **Chosen by measurement.** All 15 upstream doctrines were classified by domain-dependence of
  their LOGIC (comments stripped). `WAIVER-ROUTING` scored **0** — portable essentially unchanged.
  The ranked remainder is now a frontier in `docs/tasks/BEDROCK-MAINTENANCE.md`, not a wish list.
- ⭐⭐ **The port FIXED a defect rather than inheriting one**: the origin's `printf … | grep -q …
  || continue` returns failure ON SUCCESS past the pipe buffer under `pipefail`, silently SKIPPING
  the file — a **fail-open**. Both sites here read a file instead. Threshold measured, not assumed:
  65,606 B → no SIGPIPE; 131,139 B → SIGPIPE.
- **Wrote down the neutrality bar** (`MAINTAINING.md`): every doctrine here must be objectively
  applicable to ANY project, with a measurable admission test and its honest bound — plus the rule
  that **transfer runs both ways**, after this repo's layer-C check turned out to be stronger than
  the reference deployment's.
- Probes 5/0; `make gate` 7/7; added to the `update_scaffold.sh` NEUTRAL allow-list.

## bedrock-scaffold 0.2.0 — README Stability Policy + a layer-A byte cap

`BEDROCK-MAINTENANCE.2.1`. Transferred from the reference deployment by maintainer order.

- **Added `README_POLICY.md`** (project-neutral, verbatim) — keeps `README.md` a stable landing
  page instead of a changelog/roadmap/catalogue, and states the caps rule.
- **Added the `README-STABILITY` doctrine** (`scripts/check_readme_stability.sh`): a line cap
  AND a byte cap, a dated-line (release-history) tripwire, and a required link back to the
  policy. Non-mutating; REFUSES (exit 2) rather than passing when the README or policy is
  absent. Template defaults 300 lines / 16384 bytes — generous on purpose, because they ship to
  a project whose README is not this one; tighten after your own trim.
- ⛔ **Closed a bypass the spine was itself shipping.** `scripts/check_memory_architecture.sh`
  capped layer-A `MEMORY.md` by LINES only (cap 120, no byte bound), exactly as
  `MEMORY_ARCHITECTURE.md` §9's reference check prescribed — so **every adopting project
  inherited a bound that does not bind.** Measured on a real project running this spine:
  60 lines (passing, exactly at its cap) carrying **138,403 bytes** — 2,306 B/line, one line of
  18,816 B. Now both caps, in the check **and** in the standard (§6 / §9 / §9.1).
  Layer-A caps: **50 lines** (tightened from 120, to match the "≤ ~50 lines" §6 already stated)
  and **7168 bytes**. Both env-overridable.
- Both new files added to the `update_scaffold.sh` NEUTRAL allow-list, so existing projects
  pull them with `scripts/update_scaffold.sh <bedrock-url>`.
- Verified: `make gate` 6/6 green; a 13-line / 19,304-byte fixture is REJECTED by the byte cap
  while being well under the line cap; the **retired** layer-A guard PASSES that same file
  (exit 0) — the change is proven necessary by execution, not by argument.

Changelog-style summary of completed work + its validation (internal continuity surface;
the immutable audit trail proper is `git log` — memory layer D). Newest first.

## _(YYYY-MM-DD)_ — bootstrap

Instantiated from the `bedrock` discipline-spine template. Next: replace `ROADMAP.md` and
seed the first task-tree.
