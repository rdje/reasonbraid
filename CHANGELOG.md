# CHANGELOG.md

## 2026-09-06 — Server-assigned rounds: the advance verb and its grant (`PHASE-1.5.2`)

- Rounds landed as **server-assigned facts**: a new thread is round 1 (the additive `current_round` projection field, `#[serde(default)]`), every contribution lands in the current round and its event carries the number, and `thread.advance_round` (event `thread.round_advanced`) is the only mover — the client never names a round, so round skew cannot be submitted by construction.
- Advancement is a new **`thread_advance_round` grant** (the wire-name canary extended first — it failed until the registry row landed): humans carry it via the 9-action dev admin set; roles are deny-by-default (typed 403 with the audit row) — they shape content, humans shape the process. A closed thread refuses advancement (`invalid_transition`).
- CLI: `rb thread advance-round`; the two-host demo's THREAD_A now advances after the agent contribution and asserts the projection round (2) and the contribution's round (1) — 16 PASS checks total. The demo's first run caught a positional-vs-`--thread` slip in the new beat, fixed. All twelve live suites (command_api 11) + e2e + demo `rc=0`; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_rounds.md` (`answers:`).

## 2026-09-06 — The structured contribution body: typed kinds + evidence references (`PHASE-1.5.1`)

- Backlog 17's first contract landed: `thread.contribute` gains `kind` — a typed deny-unknown `ContributionKind` enum over the §8.5 initial subset (`position` **stated default** | `claim` | `assumption` | `evidence_reference` | `question` | `summary`) — and `evidence_refs`, a list of `{uri, digest?, note?}` REFERENCES (deny-unknown at the ref itself; absent fields are omitted on the wire). Out-of-registry kinds and foreign ref fields are typed 400 `invalid_command` refusals.
- Content lives in the event log, so the change is event-layer growth: the projection is untouched and pre-`.1.5.1` stored projections parse by construction. The events/inspection view renders kind + refs — nothing silently dropped.
- CLI: `rb thread contribute --kind …` (kebab→snake normalized, the `.1.1.3` lesson pre-applied) + repeatable `--evidence-uri`. New 4-leg `command_api` test + the e2e drives the kebab spelling through the real binary. The suite's first run caught a real wire-shape defect (absent ref fields serialized as `null` instead of omitted) — fixed with `skip_serializing_if`, rerun green: all twelve live suites + e2e + demo `rc=0`; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_structured-contributions.md` (`answers:`).

## 2026-09-06 — PHASE-1.5 decomposed: typed contribution bodies, rounds, honest close (`PHASE-1.5`)

- Tree-first decomposition on a measured gap census: the contribution body's `kind` is a FREE STRING (no typed enum), there are no evidence references, no round fields anywhere, and the core thread machine has NO `Inconclusive` terminal (Open/Closing/Closed/Cancelled only) — so backlog 17 needs three independent contracts.
- `.1.5.1` the structured contribution body (`kind` → a typed deny-unknown §8.5 enum + `evidence_refs` — references only, acquisition stays Phase 4); `.1.5.2` rounds (a contribution names the round; enforced at the boundary, visible in inspection); `.1.5.3` the honest close (`outcome: decided|inconclusive` + the unresolved register + the core `Inconclusive` terminal). Votes/abstentions and the workflow-phase concept defer to Phase 5's workflow engine. `make gate` → 13/13 at commit.

## 2026-09-06 — `.1.4` complete: the Claude adapter is LIVE-qualified (`PHASE-1.4.2`)

- The env-gated live qualification test (`crates/reasonbraid-node/tests/claude_live.rs`, `RB_LIVE_CLAUDE=1`, ignored by default — the `.4.2` codex_live mirror) dispatched ONE bounded real run through the real supervisor + journal and **passed on its first run**: completed, the reply streamed, exact usage, **money cost** (`total_cost_usd` — the leg Codex cannot prove), the session id attached as the provider handle, and the honest unsupported status lookup.
- The dependency-ledger Claude row is rewritten with the verified facts (checked_at 2026-09-06, tested 2.1.263, the stream-json interface, the required `--verbose`, content-only `--restricted --tools ''`, the conformance results); the book's adapter chapter gains the live-test command and the honest-limits bullets now name two real adapters; decision record `docs/decisions/2026-09-06_claude-cli-adapter.md` (`answers:`) + DEV_NOTES promoted to it.
- **`.1.4` is complete — backlogs 19–21 done: the deterministic fake (Phase 0), the Codex adapter (Phase 0), and now the Claude adapter.** Two genuinely distinct harness adapters exist; each is qualified on one host + one CLI version with a revalidation trigger. `cargo test --all` green (the live test SKIPs without the env var), clippy clean, `make gate` 13/13, `make book` builds. Frontier → `.1.5` (structured contributions).

## 2026-09-06 — The Claude CLI adapter core: `claude.rs`, the `.4.2` mirror (`PHASE-1.4.1`)

- Backlog 21's first half landed: `ClaudeCliAdapter` (`crates/reasonbraid-adapter/src/claude.rs`) supervises `claude -p --output-format stream-json --restricted --tools '' --verbose -- <prompt>` — the narrowest supported machine interface, qualified against the INSTALLED Claude Code 2.1.263. The stream maps: `system/init` (`session_id`) → `ProviderRequestId`; `assistant` text blocks → one chunk each (thinking blocks skipped — the reply is the text); `result` `is_error:false` → `Completed` with the FULL result event (usage under `usage`, money under `total_cost_usd` — Claude reports COST, so `cost` is `Some`, unlike Codex's `None`); `result` `is_error:true` → `FailedKnown` with the provider's own message; non-zero exit → `FailedKnown` with the stderr tail; EOF without a result → lost response.
- The wire facts were pinned by three bounded live probes BEFORE code (no guessed formats): `--verbose` is REQUIRED by the CLI with `stream-json` (the no-verbose probe was refused pre-dispatch), the prompt must follow `--` (variadic `--tools` otherwise swallows it), and Anthropic's token counts already include caches/thinking (no folding — the one place the Codex normalizer differs).
- `--restricted` + `--tools ''` make the boundary content-only; the adapter holds no credentials (ambient Claude login); status lookup is honestly `Unsupported` (`--resume` continues, it does not query). New `tests/claude_adapter.rs` (10 offline tests over a stub binary — the REAL subprocess boundary; the suite's first run caught a test-authoring slip on the multi-chunk assertion, fixed). All offline suites green + all twelve live-PG suites + CLI e2e + two-host demo `rc=0`; clippy clean; `make gate` 13/13. The book's adapter chapter gains the Claude section (the live-test command arrives with `.1.4.2`).

## 2026-09-06 — PHASE-1.4 decomposed: the Claude CLI adapter (`PHASE-1.4`)

- Tree-first decomposition on a measured gap census: backlogs 19 (deterministic fake) and 20 (Codex adapter) are Phase-0-proven (`.4.1`/`.4.2`), so `.1.4`'s delta is backlog 21 — the Claude-family adapter as the `.4.2` mirror. The live `claude` CLI is **installed** (2.1.263), so the real harness leg runs for real, env-gated like `RB_LIVE_CODEX`.
- The machine interface is VERIFIED against the real binary before any code: three bounded probes pinned the 2.1.263 stream — `system/init` carries `session_id`; `assistant` text blocks are the reply; `result` carries `usage` AND `total_cost_usd` (Claude reports money — normalized cost, unlike Codex); the CLI REFUSES `stream-json` without `--verbose` (pre-dispatch error). Probe evidence kept on-volume in `target/claude-probes/` (the /tmp originals deleted, residue-census-verified — §13).
- Children: `.1.4.1` the adapter core (`claude.rs` + offline stub suite) → `.1.4.2` live qualification + dependency-ledger row + book chapter + decision record. `make gate` → 13/13 at commit.

## 2026-09-06 — §13 same-volume locality for the ephemeral PG test cluster (`PHASE-1-MAINT-1`)

- The defect leaf opened during `.1.1.1` is closed: `scripts/run_pg_tests.sh` no longer defaults its ephemeral PostgreSQL data dir to `${TMPDIR:-/tmp}/reasonbraid-pg.XXXXXX`. The cluster now lives at `$ROOT/target/pg-ephemeral.XXXXXX` — derived at runtime from the script's own location, same volume as the repo, gitignored via `/target`, still unique per run, still cleaned by the exit trap (only the parent directory moved; the `mktemp`/cleanup mechanics are untouched).
- Verification is the script's own full rerun from the new location, twice: all **twelve** live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 4 + 17 + 3 + 3 + 6 + 7 `passed`) + CLI e2e `2 passed` + the two-host demo `ALL acceptance checks passed` (both `rc=0`); a 2 s poll observed the cluster on the repo volume mid-run and the residue census left nothing under `/tmp` or `target/` after. `make gate` → 13/13 at commit. Decision recorded: `docs/decisions/2026-09-06_same-volume-pg-ephemeral.md` (`answers:`).

## 2026-09-06 — Simple subscriptions: `thread.join`, enforced participant doors (`PHASE-1.3.2`)

- `thread.join` lands (event `thread.participant_joined`, `via: join_request`): a thread whose `allow_join_requests` is on admits the role directly as `accepted` — the self-request path, no invitation, no reservation machinery (a subscription is an ordinary `thread_contribute` act). A closed door and a double join are typed refusals.
- The `.1.1.3` participant rules became load-bearing: `allow_explicit_invites=false` now REFUSES the invite verb at the boundary (the join door still works), and `allow_join_requests` gates the join verb. The listing surface is the existing inspection view (full participant states + invitation meta).
- The race test taught a domain fact first: concurrent accept+remove BOTH succeed — they are COMPATIBLE transitions (accept-then-revoke is a legitimate sequence; the snapshot is `revoked` either way). The conflict pair is accept vs decline; the suite now races that and asserts exactly one winner. New fourth `invitations` test; `rb thread join`; full live-PG regression (twelve suites) + CLI e2e + two-host demo green (`rc=0`); clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_join-subscriptions.md` (`answers:`). **The `.1.3` coordinator leaf is complete** (backlogs 15/16).

## 2026-09-06 — The explicit-participants contract: pending invitations, accept/decline/remove, dispatch-on-accept (`PHASE-1.3.1`)

- Backlog 16 + 15's membership semantics landed: `thread.invite` records a **pending offer** (typed optional `expires_in_seconds`; `invited_at`/`expires_at` ride the event AND the new additive `invitations` projection map) and **enqueues nothing** — the `.6.2` invite-time dispatch is gone.
- The invited role **accepts** (`thread.accept_invitation`, event `thread.invitation_accepted`) — and the accept is the transaction that dispatches the contribute work item with its reservation (`work_{accept_event_id}`: an accepted invitation exists iff its work does). `thread.decline_invitation` (event `thread.invitation_declined`) refuses the offer; `thread.remove_participant` (tenant_admin; the core machine gains `revoked` + the `Revoke` transition; event `thread.participant_removed`). Re-invitation is allowed over any terminal state.
- **The invitation IS the acceptance capability** (offer/reserve): accept/decline authorize against the pending invitation naming the actor, under a NEW `thread_invitation_respond` grant (the grant registry + its wire-name canary extended first; the role default gains it). An invited role may NOT act until accepted — the auto-accept on first contribution is gone (typed `invalid_transition` naming the accept verb).
- **Expiry is derived, never swept** (the lease-presence pattern): the inspection view reads past-expiry offers as `expired`, and accept/decline refuse them 409 — no sweeper, no stored flag, no expiry event.
- The wiring suites, the CLI e2e, and the two-host demo moved to the explicit contract (`rb thread accept` as the role before the node starts; the invite step produces no work). New `tests/invitations.rs` (3 live-PG tests: the full lifecycle + typed refusals; decline/expiry/re-invitation; the concurrent accept/remove race — exactly one winner). Full live-PG regression — **twelve** server suites green + CLI e2e + two-host demo `ALL acceptance checks passed` (`rc=0`); offline suites green; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_explicit-participants.md` (`answers:`).

## 2026-09-06 — PHASE-1.3 decomposed: invitation lifecycle, dispatch-on-accept, subscriptions (`PHASE-1.3`)

- The invitation/subscription leaf is decomposed into three signoff-sized children (tree-first; no code change), on a measured gap census: the `.6.2` wiring dispatches work IN the invite transaction with no acceptance step (`ensure_participant` auto-accepts an invited role on its first contribution), there are no accept/decline/expire/remove verbs or invitation records, `allow_join_requests` is typed but inert, and `allow_explicit_invites=false` is recorded but not enforced.
- `.1.3.1` the explicit-participants contract (backlog 16 + 15: invite records a PENDING invitation with a typed optional expiry and enqueues NO work; accept/decline/remove verbs; the work item + reservation enqueue with the ACCEPT event; derived expiry; the invitation IS the acceptance capability), `.1.3.2` simple subscriptions (`thread.join` under `allow_join_requests`, `allow_explicit_invites=false` enforcement, the subscription listing + CLI verbs). `make gate` → 13/13 green at commit. Amended same-day: the lifecycle and the dispatch move are one contract — the original `.1.3.2` merged into `.1.3.1` (separating them leaves an incoherent interim).

## 2026-09-06 — Durable inbox hardening: quarantine, measured retention, inspection (`PHASE-1.2.3`)

- Backlog 14's remainder landed: **quarantine is a database fact on the row** — migration 0010 adds `quarantined_at` + `quarantine_reason` to `node_inbox`, and the replay/poll queries filter `quarantined_at IS NULL`, so a quarantined command is **never re-delivered**, whatever cursor the node reports. The reason rides the row: the skip is explainable, never silent. Quarantine controls *delivery*, not result application (a result from a command delivered before the quarantine still applies).
- **Retention is an explicit, measured operator action**: `POST /v1/nodes/inbox/prune` deletes only DELIVERED rows older than the `min_age_seconds` window, with before/delete/after computed in ONE transaction — the response is the operator's receipt. No background sweeper.
- The operator surface is the **`tenant_admin`-audited control API** (the same gate as token issuance — the authorization record is the audit, no new audit table): `POST /v1/nodes/quarantine` (typed refusals: unknown command 400, re-quarantine 409, empty reason 400, non-admin 403), `GET /v1/nodes/inbox` (delivery + quarantine facts per row, with the payload), `POST /v1/nodes/inbox/prune` (negative window 400, non-admin 403).
- CLI: `rb node quarantine|inbox|prune`. New `tests/node_inbox.rs` (3 live-PG tests: quarantine skipped by replay AND poll + reason rides the row + inspection; typed refusals; measured prune with only old delivered rows gone). Full live-PG regression — **eleven** server suites green + CLI e2e + two-host demo `ALL acceptance checks passed` (`rc=0`); offline suites green; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_node-inbox-retention.md` (`answers:`). **The `.1.2` coordinator leaf is complete** (backlogs 11–14).

## 2026-09-06 — The authenticated node channel: key-proof handshake, leases, observable presence (`PHASE-1.2.2`)

- Backlog 13's remainder landed: the channel is now authenticated end to end, `CHANNEL_VERSION` **2**. The handshake carries an **HMAC-SHA256 key-proof** over the canonical channel fields (the mirrored `ProofCoverage` shape IS the canonicalization), keyed with the `.1.2.1` dev secret — verified in constant time and refused `401 unauthorized` BEFORE any ledger fact is read; a missing field is a malformed request (422), a wrong proof and an unenrolled node fail identically (no existence leak).
- A successful handshake issues a **lease** with a fresh **fencing token** (`fnc_<uuid>`, generated in PostgreSQL): the only token that renews the lease (`POST /v1/nodes/heartbeat` — live leases only) or guards `events`/`ack`/`poll`. Every handshake rotates it, so a stale process is fenced the moment a newer handshake lands. `poll` became a POST — the token never rides a query string.
- **Presence is derived, never stored:** migration 0009's `node_leases` + `node_presence` view compute `online` from the 60 s expiry clock — expiry flips a node observably `offline` (`GET /v1/nodes/presence`; the token is never exposed), and only a fresh key-proof restores it.
- The channel identity space widened: issuance + enrollment now accept the `rol_…` role wire id alongside `nod_…` (the dev wiring collapses node == role; a latent `.1.2.1` strictness the authenticated handshake exposed — a superset, the `.1.2.1` suites never asserted `nod`-only).
- The node client keeps the fencing token in shared state (worker poll + heartbeat task + reconcile rotation observe one lease); `rb-node --node-secret` is required and runs a 15 s heartbeat loop. All 13 channel tests moved to the authenticated contract + 4 new (refusal classes, renewal + presence, fencing rotation, expiry → offline → re-handshake); the two-host demo now enrolls its nodes, re-POSTs the duplicate with the live fencing token, and asserts presence online before AND after the server restart (14 PASS checks, `rc=0`). Full live-PG regression green; offline suites green; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_node-channel-auth.md` (`answers:`).

## 2026-09-06 — Dev-profile node enrollment: one-time tokens, one auditable transaction (`PHASE-1.2.1`)

- Backlog 11 landed: `migrations/0008_node_enrollment.sql` (one-time tokens bound to tenant + node id + host claim + nonce + expiry; `node_keys` holding the dev signing secret + its SHA-256 fingerprint; the `node_enroll_audit` refusal log) plus the hosts get-or-create index on the 0007 table.
- The flow: an authorized human issues a token (`POST /v1/nodes/enroll-tokens`, `tenant_admin` authority — the authorization engine audits the issuance); the node consumes it (`POST /v1/nodes/enroll`, the token IS the credential) with its dev secret — host + node + key + token-consumption + audit land in ONE transaction. Certificate issuance stays deferred to Phase 2 (ADR-007); the server-as-trust-store dev stance is the documented `.6.1` pattern.
- **One-time is a database fact:** `FOR UPDATE` on the token row + `used_at` makes replay impossible; every refusal (unknown/used/expired/mismatched/nonce) is a committed audit row before the typed error returns — the budget engine's denial-row pattern.
- Operator surface: `rb node issue-token` + `rb-node --enroll-token … --enroll-nonce … --host-claim … --node-secret …` (the node enrolls before any channel traffic). New `tests/node_enrollment.rs` (3 live-PG tests: one-time + identity rows; four audited refusal classes; tenant-admin-only issuance). Full live-PG regression + two-host demo green; offline suites green; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_node-enrollment.md` (`answers:`).

## 2026-09-06 — PHASE-1.2 decomposed: enrollment, authenticated channel + leases, inbox hardening (`PHASE-1.2`)

- The node leaf is decomposed into three signoff-sized children (tree-first, no code change), on a measured gap census: node enrollment is absent, the node channel has no leases/presence (only the outbox worker leases), the per-node inbox has no retention/quarantine, and backlog 12's journal is already Phase-0-proven (the WP3 kill-point sweep carries it).
- `.1.2.1` dev-profile node enrollment (backlog 11: one-time tokens, registration into the 0007 `nodes` table, dev signing key, audit; certificate issuance deferred to ADR-007), `.1.2.2` authenticated channel + lease/presence (backlog 13's remainder: key-proof handshake, heartbeat leases, expiry → visible offline state), `.1.2.3` durable inbox retention + quarantine (backlog 14's remainder; filtered delivery stays with Phase 3's directory). `make gate` → 13/13 green at commit.

## 2026-09-06 — Thread command API completion: cancel + typed create profiles (`PHASE-1.1.3`)

- `thread.cancel` lands as the abandonment terminal — the core `open|closing → cancelled` edge wired through the command API, with `cancel_reason` in the projection and a `thread.cancelled` event; distinct from a decided close (separate reasons, separate events, and the API test asserts `close_reason` stays null on cancel). `thread_cancel` joins the grant registry as its own action (the registry's wire-name test extended first — the canary that failed and taught the entry).
- `thread.create` gains three typed, deny-unknown fields: `classification` (`general` default | `confidential`), `workflow_profile` (`single_agent` **stated default** per ADR-002 | `blind_independent` | `critique_revise` | `moderator` — non-default profiles recorded and executed as single-agent until routing work), `participant_rules` (`allow_explicit_invites` default on, `allow_join_requests` default off — §20.3 explicit participants first). Foreign fields and out-of-registry values are typed `invalid_command` rejections.
- The CLI gains `thread cancel` and the three create flags (kebab-case human spellings normalized to the wire's snake_case — the e2e's first run caught the mismatch and the server's typed error named the values); inspection now shows classification/workflow and the cancel reason. Projection growth is additive (`#[serde(default)]`), so pre-`.1.1.3` projections still parse.
- New live-PG tests (cancel inspectable/terminal/audited; typed fields + stated defaults + rejections); CLI e2e extended with the typed-create + cancel leg. Full live-PG regression + two-host demo green; offline suites green; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_thread-api-completion.md` (`answers:`).

## 2026-09-06 — Migration 0007: the first-class identity store (`PHASE-1.1.2`)

- Backlog 10's identity schema landed as `migrations/0007_identity_store.sql`: `tenants`, `human_principals`, `agent_roles`, `hosts`, `nodes`, `incarnations`, `runs` — the §8.1 hierarchy as records, with `tenant_id` on every material record (§17.2), UUIDv7 wire ids, and fail-closed foreign keys (a principal whose tenant does not exist is refused by the database). The `.6.1` `enrollments` table stays the dev bootstrap's name→id map; the incarnation carries only the §8.1-defining facts (provider/model/harness/config, validity interval) — later-feature columns arrive with their features (the 0002 precedent).
- Enroll now writes the tenant row (bootstrap), the identity row, the grant, the boundary, and the enrollment row in ONE transaction — an enrollment implies its identity row; a re-enroll (same tenant + kind + name) replays and duplicates nothing at either layer.
- New `tests/identity_store.rs` (bootstrap commits tenant+identity+enrollment together; role identity + replay duplicates nothing; FKs fail closed); the API-driving suites' purge lists gained the identity tables in FK order. Full live-PG regression + two-host demo green; offline suites green; clippy clean; `make gate` 13/13.
- Decision recorded: `docs/decisions/2026-09-06_identity-store.md` (`answers:` present — the table is the record, the FK is the enforcer, the re-enroll is a replay at the identity layer too). Test-authored defect caught by the new suite and fixed in the same leaf (the human re-enroll assertion omitted `tenant_id`, which the dev API reads as a fresh bootstrap — the corrected test replays through the explicit tenant).

## 2026-09-06 — The aggregate/event/outbox library: one auditable write path (`PHASE-1.1.1`)

- Backlog 9 landed as `reasonbraid-server::agg` — the WP2 machinery extracted from `tx.rs` into a typed library: the idempotency claim (the `(tenant_id, idempotency_key)` primary key is the serialization point), the locked aggregate head (the revision serialization point), the ordered event append, the current-state upsert, the outbox enqueue (its FK proves an outbox item implies its event is durable), and the semantic result — one transaction, composed by callers with authorization/validation in the same transaction. An optional `expected_revision` precondition adds optimistic concurrency; it defaults OFF, so Phase 0 behavior is preserved byte-for-byte.
- `tx.rs` is now a typed compatibility shim that owns NO SQL (type conversion + delegation; its documented invariant makes an impossible drift a crash). `api.rs`'s rejection store rides the library's step 6. Zero call sites changed shape.
- Proven through the library's own surface: new `tests/aggregate_library.rs` (fresh-apply vs replay with the original result, request-hash conflicts, the revision precondition hold/refusal, outbox→event integrity) — green against live PostgreSQL 16.15, alongside the full regression: all eight server suites, the real-binary CLI e2e, and the two-host demo (every acceptance check PASS). Offline `cargo test --all` all green; clippy `-D warnings` clean; `make gate` 13/13.
- Decision recorded: ADR-004 `accepted` (evidence-gated) — locked head + claim-first + one-transaction writes; a separate store crate stays forbidden until a measured boundary need (ADR-002); the durable lesson is promoted to `docs/decisions/2026-09-06_aggregate-library.md` (`answers:`). Defect tracked: `PHASE-1-MAINT-1` (the ephemeral PG cluster's data dir defaults to `/tmp` — §13 same-volume locality gap in `scripts/run_pg_tests.sh`, pre-existing from `.2.1`).

## 2026-09-06 — PHASE-1.1 decomposed: three signoff-sized coordinator slices (`PHASE-1.1`)

- The coordinator leaf is decomposed into children (tree-first; no code change): `.1.1.1` the aggregate/event/outbox library (extract the `tx.rs` claim → authorize → validate → apply machinery into a typed reusable module with revision-checked transitions + in-tx test helpers; backlog 9, ADR-004), `.1.1.2` migration 0007 identity store (`tenants`/`hosts`/`nodes`/`agent_roles`/`incarnations`/`runs`/`human_principals`; backlog 10), `.1.1.3` thread command API completion (`thread.cancel`, typed classification/workflow-profile/participant-rules with the ADR-002 single-agent default; backlog 15's API-shape portion — the invitation accept/decline/timeout semantics stay with `.1.3`).
- Lockstep drift fixed in the same commit: the book's roadmap chapter names Phase 1 as current (Phase 0 closed, ADR-002 signed), and `PROGRAM.md`'s Phase-1 row + frontier line moved off the stale `proposed`/"next work is PHASE-0" wording. `make gate` → 13/13 green at commit.

## 2026-09-06 — ReasonBraid-only naming: the 90-token sweep (`PHASE-0-MAINT-2`)

- Director directive: no more scaffold-name references — only ReasonBraid. Census (case-insensitive `git grep` over the scaffold-name token) → **90 occurrences in 28 tracked files** (the README landing page, `DOCTRINE_VERSION`, `cargo-generate.toml`, provenance notes in `COMMIT.md` + six doctrine checkers + two artifact probes, `scripts/bootstrap.sh` + `scripts/update_scaffold.sh`, historical CHANGELOG entries, tree and live docs).
- Sweep: ordered token map (the scaffold tracker-id prefix → `REASONBRAID-MAINTENANCE`, the maintainer-note markers → `REASONBRAID-MAINTAINER-NOTE`, the template version prefix → `reasonbraid-scaffold`, then bare tokens) plus prose polish ("the ReasonBraid spine", "scaffold-URL placeholder"); after: the same census → none. Provenance facts preserved; `bash -n` on all 11 touched scripts ok; the guard's self-test fixture now matches the README (`<reasonbraid-url>`).
- **The PHASE-0 tree is complete** — next executable work is `PHASE-1.1`. `make gate` → 13/13 green at commit.

## 2026-09-06 — README_POLICY re-adopted: derived caps and routing-pressure closure (`PHASE-0-MAINT-1`)

- `README_POLICY.md` re-adopted at the upstream 2026 revision (fenced ReasonBraid adoption note + neutral body: Authority and provenance, duplication probe, Routing pressure closure, derived caps, unconditional-check rule, 9-step checklist).
- `scripts/check_readme_stability.sh` rewritten: **derived caps** (60 lines / 2,400 bytes from the reviewed 47-line / 1,772-byte landing page — template defaults retired), **routing-pressure closure** over `.doctrine/readme_routes.txt` (17 governed rows; prefix closure; transitive control-field leg), the CHANGELOG **96,000-byte rotation threshold** (the measured 48,495-byte baseline recorded as governed debt), and a `--self-test` arm. The closure leg's first run caught three genuinely unrouted destinations (`COMMIT.md`, `docs/adr/…`, the scaffold-URL placeholder) — all given governed rows or reworded.
- Guard falsification matrix: cap override → red; injected unrouted link → red (tree restored byte-identical); malformed registry row → red; self-test ground truth → ok.
- Decision record `docs/decisions/2026-09-06_readme-policy-readoption.md` (+ `answers:`) and INDEX row; DOCTRINE_ENFORCEMENT mirror updated. `make gate` → 13/13 green at commit.

## 2026-09-06 — Phase 0 exit gate closed: ADR-002 signed by the accountable owner, PHASE-1 opened (`PHASE-0.8.2`)

- The accountable owner signed ADR-002 (**GO**, `accepted`) via an explicit session decision — **Phase 0 formally exits** (KICKOFF §7: "a named owner signs a go, rework, pivot, or stop record"). `docs/adr/002-phase1-scope.md` (status + signature line) and the ADR INDEX updated; the PHASE-1 tree opened (`active`, frontier `.1` unblocked); PHASE-0 Blockers resolved; LIVE_STATUS: Phase 0 → Done, Phase 1 → Not Started.
- Docs-only commit (no code paths touched); `make gate` → 13/13 green at commit.
- Dating note: this entry is dated by the host/git clock (2026-09-06); the two entries above were dated 2026-09-07 by their session — flagged to the director; no committed history was rewritten.

## 2026-09-07 — WP8 Phase 0 decision and subtraction package: the gate is assembled (`PHASE-0.8.1`)

- Published the **evidence manifest** (`docs/evidence/2026-09-07_phase0-evidence-manifest.md`): the G0 map (identity / authority / thread / delivery / budget — each with its suites and reproducible commands), the fixtures and measurements, and every Phase 0 failure with its disposition (all fixed with regression coverage).
- Published the **ADR set** (`docs/decisions/2026-09-07_phase0-adr-set.md`): the audit map from each KICKOFF-required ADR topic (persistence/outbox, node journal, transport, adapter boundary, provider ambiguity, initial authorization, Phase 1 scope) to its accepted decision record.
- Published the **SubtractionRecord** (`docs/decisions/2026-09-07_phase0-subtraction-record.md`, §19.8 shape): removed/deferred features with revisit triggers (authenticated channel, NATS/transport spikes, policy engine, Git/object-store experiments, MCP/A2A, second adapter, shared wire crate, directory), narrowed product claims (no structure-beats-single claim — the WP7 null; no multi-harness claim; no exactly-once claim), rejected abstractions, avoided dependencies, bounded manual fallbacks, eliminated entities, and the effort accounting (≈9.5 engineer-weeks vs the 8–14 range — the 2×-estimate review is NOT triggered; recorded).
- Proposed **ADR-002** (`docs/adr/002-phase1-scope.md`): Phase 1 GO on the §20.3 LAN vertical slice with single-agent-default routing (from the WP7 null result), the second real adapter, and dev-profile trust replaced before any non-loopback exposure — **awaiting the accountable owner's (director's) signature**; `v0.5.0` remains forbidden.
- Refreshed the risk register: R-AMB mitigated (`.3.1`/`.4.1` proof), R-VALUE narrowed by the null result, and two NEW rows from the real run (R-VARIANCE: provider run-to-run variance on identical prompts; R-OVERHEAD: ~16k ambient input tokens per real call distorting H6 accounting).
- `make check` (all offline suites), `make gate` 13/13, `make deny`, `make secret-scan`, `make book` green. **The Phase 0 tree is exhausted; the gate awaits the director's signature on ADR-002.**

## 2026-09-07 — WP7 deliberation/routing benchmark: versioned corpus, four workflows, deterministic grading (`PHASE-0.7`)

- Landed the benchmark harness in `crates/reasonbraid-adapter` (`src/bench/` + the `rb-bench` binary + `bench/v1/` corpus/prompts): a versioned eight-case corpus (factual / code-review / ambiguous-policy / insufficient-evidence) run through four workflows — single agent, blind-independent answers + a DETERMINISTIC adjudicator, critique/revise, moderator/synthesis with a REQUIRED structured `UNRESOLVED` register — with per-call cost accounting and per-case confidence.
- **Grading is deterministic, never an LLM judge** (a judge would share the measured models' correlated errors): factual keys, rubric presence checklists, REQUIRED parsed `CONFIDENCE: 0.XX` lines, `CITE[n]` range audits, and an insufficient-evidence honesty trap for asserted numbers. **No independence score** (agreement is a descriptive count with the warning that agreement ≠ correctness — enforced by test); aggregates are spread-bearing (n/min/mean/max); Brier is factual-only; unmeasured metrics (human-review minutes → WP8) are recorded as `not_measured`, never zero.
- **The corpus carries its own oracle**: every case ships scripted outputs AND the scores the harness MUST produce; `tests/bench_harness.rs` asserts computed == expected over the whole corpus — the measurement pipeline is proven before any token is spent. Corpus and prompt digests ride every report.
- **The FIRST real run caught two harness defects the scripted oracle cannot see** (recorded in `docs/evidence/2026-09-07_benchmark-codex-run.md`): the revision leg re-rendered the CRITIQUE template (no confidence lines anywhere, one answer broken 1.0 → 0.0), and the honesty trap flagged the question's own echoed year. Both fixed with regression tests (the prompt-split check lives in the corpus test — the scripted agent answers by role, so the corpus itself must guard prompt wiring). The corrected real run's numbers are in the evidence record — including a negative-or-null-tolerant read for the WP8 routing memo.
- Recorded `docs/decisions/2026-09-07_deliberation-benchmark.md` (offline harness over the adapter contract; corpus-carried oracle; deterministic graders; env-gated call-budgeted real mode `RB_LIVE_CODEX=1 --max-calls`). The mdBook gains the benchmark chapter. **Frontier is `PHASE-0.8`.**

## 2026-09-07 — WP6 node wiring + the two-host crash/reconnect demonstration (`PHASE-0.6.2`)

- **Inbox dispatch rides the command transaction.** An accepted `thread.invite`/`thread.challenge` now hands a work item to the target role's node in the SAME transaction as the thread event: a best-effort reservation against the thread ceiling (a refusal is a recorded denial row, and the work item still enqueues — without a reservation, with the reason), then the inbox row (`work_{event_id}` as the command id). An invitation exists iff its work does.
- **Node results fold into the thread through the same claim → authorize → validate → apply flow** (`apply_node_result_in_tx` in `api.rs`, called from the channel's events handler in one transaction): the idempotency key is the inbox command id, so a duplicated transport replays the ORIGINAL result instead of producing a second domain effect; the reservation settles with the reported usage in the same transaction; a rejection (e.g. after close) is stored as the work command's idempotent result while the event receipt still commits.
- **The node grew a worker** (`reasonbraid-node/src/worker.rs` + the `rb-node` binary): poll → journal (deduped) → execute work items whose attempt is absent or `prepared` — **never a silent retry** of `dispatched`/`outcome_unknown` — and emit `work_result` events for completed attempts. The journal gained `work_items` and `emitted_events`; `rb-journal` gained the `events` view.
- **The demonstration is the acceptance test**: `scripts/demo_two_host.sh` drives the REAL binaries with real kill points — server SIGKILL + restart, node SIGKILL after a durable dispatch boundary (recovery: `outcome_unknown`, bounded, visible, not retried), duplicate delivery re-POSTed verbatim (`accepted:false`, one domain effect), budget exhaustion (`calls:1` thread: the second dispatch denied at the server AND refused by the node's budget gate before any provider contact), closure preserving the contribution AND the unresolved challenge — and asserts every KICKOFF WP6 acceptance point, writing the evidence bundle under `target/demo/<run-id>/`. It runs in the focused gate (`run_pg_tests.sh`, `make demo`) and the CI `pg-tests` job. Two-host mode via `--node-host`/`--remote-workdir` (ssh).
- Proven live (PostgreSQL 16.15): `node_work` → `6 passed` (invite dispatches work with a reservation; one contribution despite duplicates at both layers; challenge → revise → revision lands and answers the challenge; budget-denied work enqueued without a reservation + denial row; post-close results stored as rejections; ordinary channel events stay receipts) — all seven live-PG suites green (5+9+5+7+13+7+6) plus the CLI e2e, and the demo passes every acceptance check in ~5 s. `make check` (all suites offline), `make gate` 13/13, `make deny`, `make secret-scan`, `make book` green.
- Recorded `docs/decisions/2026-09-07_node-channel-wiring.md` (three real bugs caught and fixed: the revise-target inversion the suite caught, the `node_work` harness purge race, and the demo script's `$$`-in-a-subshell pid bug). The mdBook gains the two-host demo chapter. **WP6 complete; frontier is `PHASE-0.7`.**

## 2026-09-06 — WP6 control API + CLI: one transaction per thread command, inspection without database surgery (`PHASE-0.6.1`)

- Landed the thread domain (`reasonbraid-server/src/threads.rs`): the six operations (`thread.create/invite/contribute/challenge/revise/close`) with typed `deny_unknown_fields` bodies and named events; a minimal projection in `aggregate_state.state`; core-machine validation with dev rules — the creator is seated as an accepted participant, the first contribution auto-accepts an invitation, challenge/revise targets are checked against the event log, close folds `open → closing → closed` (both edges validated), and closure preserves contributions + the unresolved register (`open_challenges`).
- Landed the control API (`src/api.rs` + `src/bin/rb-server.rs` + `migrations/0006_control_api.sql`): enroll bootstrap (tenant + boundary + admin grant + enrollment row in ONE transaction; re-enroll replays the original principal id), `/v1/threads` create/commands/state/events/audit, the trusted dev `x-reasonbraid-principal` header (deterministic UUIDv5 actor handles — audit rows stay linkable), and a SHA-256 request hash. **Every command runs claim → authorize → validate against the LOCKED projection → apply (+ the budget ceiling) in one transaction** (`tx.rs` split into `claim_idempotency_in_tx`/`apply_fresh_in_tx`; public behavior unchanged). **Rejections are idempotent results**: a denial or domain refusal is stored, and a replay reproduces the ORIGINAL status and body.
- Landed `crates/reasonbraid-cli` (the `rb` binary): enroll, thread create/invite/contribute/challenge/revise/close, inspect thread/threads — with a repo-local state dir (`./.reasonbraid-cli`, `REASONBRAID_CLI_STATE`; §13 locality). Core gains `GrantAction::ThreadClose` and `actor_handle_for_subject`.
- Proven live (PostgreSQL 16.15): `command_api` → `7 passed` (full flow with an ordered 6-event timeline + 8 digest-carrying audit records, denials recorded and effect-free, replay-vs-conflict, invalid transitions, forged-field/header rejection, challenge targets) and `cli_end_to_end` → `2 passed` (the REAL `rb` binary drives the whole flow and inspects everything through its own stdout — the acceptance, mechanically). All six live-PG suites green (5+7+13+9+5+7) plus the CLI e2e; `make check` (all 28 suites offline), `make gate` 13/13, `make deny`, `make secret-scan`, `make book` green.
- Recorded `docs/decisions/2026-09-06_control-api-cli.md` (`answers:` present; two falsified bugs — the dev-grant window overrun caught by the `.5.1` subset checker, and the CLI's query-string join caught by the e2e run). The mdBook gains the CLI chapter. **Frontier is `PHASE-0.6.2`** (the two-host crash/reconnect demo wires the node channel into this surface).

## 2026-09-06 — WP5 budget engine: reserve before dispatch, at both boundaries (`PHASE-0.5.2`)

- Landed the budget model in `reasonbraid-core/src/budget.rs`: multi-dimensional `BudgetDimensions` (calls, tokens in/out, wall-clock) with **fail-closed** coverage — a requested dimension the ceiling does not meter is refused, never silently allowed (§14.1/§14.6) — total fallible arithmetic (an over-release is a typed underflow, never saturating), and the `ReservationReference` a node verifies before dispatching.
- Landed the server engine (`reasonbraid-server/src/budget.rs` + `migrations/0005_budget.sql`): `create_reservation` checks the ceiling against everything held (active unexpired reservations + settled usage) in one transaction — refusals are **denial rows** (the audit trail covers what was NOT reserved); `settle_reservation` records ACTUAL usage (lower frees the difference; higher is an **overrun reported in full**, never clamped); `release_reservation` returns the hold; expired reservations stop holding on the caller's clock.
- The node supervisor gained the **dispatch gate**: `execute_attempt` now REQUIRES a `ReservationReference` and `LocalBudget` headroom, both checked BEFORE the dispatch boundary record — a refusal is journaled `failed_before_dispatch` (audited) and the adapter is never invoked. Completed attempts settle actual usage locally; **an indeterminate attempt KEEPS its hold** (§14.6: release only amounts not potentially consumed).
- Proven live: `tests/budget.rs` → `5 passed` (hold/deny-and-record, settle-frees-remainder, release, expiry, overrun) + node `supervisor_budget` → `4 passed` (refusal before the boundary with a counting adapter proving zero invocations, local headroom denial, actual-usage settlement, held-through-ambiguity); core `35 passed`; all five live-PG suites green (5+9+5+13+7); `make check`/`gate` 13-13/`deny`/`secret-scan`/`book` green.
- Recorded `docs/decisions/2026-09-06_budget-reservation.md` (`answers:` present). The mdBook gains the budgets chapter. **WP5 complete; frontier is `PHASE-0.6.1`.**

## 2026-09-06 — WP5 authority engine: boundary ceiling, scoped grants, audited decisions (`PHASE-0.5.1`)

- Landed the authority model in `reasonbraid-core/src/authority.rs`: `EnrollmentAuthorityBoundary` (the §4.4 root/parent-granted ceiling), scoped `AuthorityGrant`s (typed actions `thread_create/invite/contribute/inspect` + explicit `tenant_admin`, tenant-wide or thread-set selectors), and the deterministic **subset checker** — a grant's actions, risk ceiling, spend limits, delegation, and validity window must each fit inside its boundary.
- Landed the authority engine in `reasonbraid-server/src/authority.rs` (+ `migrations/0004_authority.sql`): `create_grant` **refuses** overreaching grants (nothing stored); `authorize` evaluates membership (nothing without a grant — **tenant membership alone grants nothing**), scope, expiry, and the subset rule (re-checked at every evaluation), and writes an `authorization_records` row for **allowances AND denials** — actor, delegated subject, grant + boundary references, decision + reason, and the SHA-256 policy digest + version.
- `apply_authorized_command` commits the audit record and the `.2.1` state/event/idempotency/outbox writes in **one transaction** (the `.2.1` body was extracted to `apply_command_in_tx`; the public `apply_command` is unchanged) — an accepted command implies its audit record; a denied command commits its denial record and applies NOTHING.
- Proven live (PostgreSQL 16.15): `tests/authority.rs` → `9 passed` — membership denial audited with no domain effect, admin never implied, accepted records re-derive their digests from the same inputs, overreaching grants refused, scope/expiry/boundary-less denials, stable digests. Core: `31 passed`.
- The first live run failed 6/9 because the subset checker was MORE precise than the fixtures (a grant built microseconds after its boundary outlived it — the temporal rule really binds); fixtures now use wide boundary windows. A second fix: `TargetSelector::Threads` became struct-like (serde cannot tag a newtype variant wrapping a sequence).
- Recorded `docs/decisions/2026-09-06_authority-boundary.md` (`answers:` present). The mdBook gains the authority chapter. **Frontier is `PHASE-0.5.2`.**

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

- Replaced the scaffold placeholder `ROADMAP.md` with ReasonBraid v0.4.1 (execution baseline).
- Tracked `KICKOFF.md` as the Phase 0 companion: scope/gates in `ROADMAP.md`, day-to-day Phase 0 execution in `KICKOFF.md`.
- Recorded `docs/decisions/2026-09-05_kickoff-companion-to-roadmap.md` and `docs/decisions/2026-09-05_roadmap-v0.4.1-frozen.md`.
- mdBook introduction now describes ReasonBraid rather than the template skeleton.

## reasonbraid-scaffold 0.6.1 — creating a project is foolproof through its first commit

`REASONBRAID-MAINTENANCE.2.7`.

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

## reasonbraid-scaffold 0.6.0 — four evidence and ratchet doctrines: lessons reach the retrievable layer, routings carry evidence, gap claims carry their census, tables keep their columns

`REASONBRAID-MAINTENANCE.2.6`.

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

## reasonbraid-scaffold 0.5.0 — the day-one batch: no agent trailers, a handoff census, no self-reported dates

`REASONBRAID-MAINTENANCE.2.5`.

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

## reasonbraid-scaffold 0.4.0 — TASK-ACCEPTANCE: a change lands with evidence, not with a claim

`REASONBRAID-MAINTENANCE.2.4`.

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

`REASONBRAID-MAINTENANCE.2.3`. Process only; no check changed, so `DOCTRINE_VERSION` is unmoved
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
- **The portability seam to look for:** does the check presume anything beyond what the spine ships?
  If yes, give it a project-declared seam or leave it upstream — never hardcode one project's
  answer and call it neutral.
- ✅ Retroactive audit: all four already-ported items PASS Q1. Nothing retracted.

## reasonbraid-scaffold 0.3.0 — WAIVER-ROUTING, and the neutrality bar for every future port

`REASONBRAID-MAINTENANCE.2.2`.

- **Added the `WAIVER-ROUTING` doctrine** (`scripts/check_waiver_routing.sh`): a task leaf saying a
  gate DOES NOT APPLY must name the leaf that owns fixing the gate. ⭐ An author writing a waiver
  IS the gate reporting a missing capability — the highest-signal defect report a gate can get.
  Deliberately does **not** punish honesty: the waiver stays legal, it just has to name an owner.
- **Chosen by measurement.** All 15 upstream doctrines were classified by domain-dependence of
  their LOGIC (comments stripped). `WAIVER-ROUTING` scored **0** — portable essentially unchanged.
  The ranked remainder is now a frontier in `docs/tasks/REASONBRAID-MAINTENANCE.md`, not a wish list.
- ⭐⭐ **The port FIXED a defect rather than inheriting one**: the origin's `printf … | grep -q …
  || continue` returns failure ON SUCCESS past the pipe buffer under `pipefail`, silently SKIPPING
  the file — a **fail-open**. Both sites here read a file instead. Threshold measured, not assumed:
  65,606 B → no SIGPIPE; 131,139 B → SIGPIPE.
- **Wrote down the neutrality bar** (`MAINTAINING.md`): every doctrine here must be objectively
  applicable to ANY project, with a measurable admission test and its honest bound — plus the rule
  that **transfer runs both ways**, after this repo's layer-C check turned out to be stronger than
  the reference deployment's.
- Probes 5/0; `make gate` 7/7; added to the `update_scaffold.sh` NEUTRAL allow-list.

## reasonbraid-scaffold 0.2.0 — README Stability Policy + a layer-A byte cap

`REASONBRAID-MAINTENANCE.2.1`. Transferred from the reference deployment by maintainer order.

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
  pull them with `scripts/update_scaffold.sh <reasonbraid-url>`.
- Verified: `make gate` 6/6 green; a 13-line / 19,304-byte fixture is REJECTED by the byte cap
  while being well under the line cap; the **retired** layer-A guard PASSES that same file
  (exit 0) — the change is proven necessary by execution, not by argument.

Changelog-style summary of completed work + its validation (internal continuity surface;
the immutable audit trail proper is `git log` — memory layer D). Newest first.

## _(YYYY-MM-DD)_ — bootstrap

Instantiated from the ReasonBraid discipline-spine template. Next: replace `ROADMAP.md` and
seed the first task-tree.
