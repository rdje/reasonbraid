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
  Status: `done`
  Goal: coordinator modular monolith, PostgreSQL migrations, aggregate/event/outbox patterns
  Backlog: 9, 10, 15
  ADR: 002, 004
  Children: `.1.1.1`–`.1.1.3` (decomposed `2026-09-06` so each child is one signoff-sized slice; all three `done`)

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
    Status: `done`
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
    Status: `done`
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
  Status: `done`
  Goal: §13 same-volume locality for the ephemeral PostgreSQL cluster —
    `scripts/run_pg_tests.sh` currently defaults its data dir to
    `${TMPDIR:-/tmp}/reasonbraid-pg.XXXXXX` (off the repo's volume); re-derive
    it from the repo root (`$ROOT/target/pg-ephemeral`, gitignored).
  Defect (tracked `2026-09-06`, pre-existing from `.2.1`): the script predates
    the §13 adoption.
  Done (`2026-09-06`): the cluster now lives at `$ROOT/target/pg-ephemeral.XXXXXX`
    (runtime-derived from the script's location, gitignored, per-run unique,
    trap-cleaned — never `/tmp`); the acceptance checklist below is written with
    this leaf's execution and verified by two full reruns (both `rc=0`) with an
    on-volume probe and a residue census.

- ID: `PHASE-1-MAINT-2`
  Status: `pending`
  Goal: reproduce and fix (or explain) a ONE-OFF `codex_adapter` suite failure —
    during a `make check` under full-workspace load, `test result: FAILED. 8
    passed; 1 failed` (the failing test's name was not captured); three immediate
    reruns and a full `make check` rerun all passed 9/9. The suite uses
    nanos-scoped stub dirs and no fixed ports, so a port/filesystem collision is
    suspected, not a logic defect.
  Defect (tracked `2026-09-06`, first seen during the `.1.3.1` verification): the
    acceptance checklist is written when the leaf executes (fix = capture the
    failing test with `--nocapture` under repeated parallel runs, root-cause,
    then either fix or record the measured explanation).

- ID: `PHASE-1.2`
  Status: `done`
  Goal: Rust node with SQLite journal, enrollment, lease/presence, reconnect, durable inbox
  Backlog: 11–14
  Note: backlog 12 (the journal's durability profile, fencing, crash fixtures,
    inspection CLI) is PROVEN by Phase 0 — the WP3 kill-point sweep
    (`journal_kill_points.rs`, 11 tests) carries it; the Phase-1 delta is the
    lease/presence state `.1.2.2` adds to the journal.
  Children: `.1.2.1`–`.1.2.3` (decomposed `2026-09-06`; gap census: node
    enrollment absent, no node-channel leases, no inbox retention/quarantine) —
    all three `done`: enrollment, authenticated channel + lease/presence,
    inbox retention + quarantine.

  - ID: `PHASE-1.2.1`
    Status: `done`
    Goal: dev-profile node enrollment (backlog 11) — one-time enrollment tokens
      (tenant + host claim + node id + expiry + nonce), the node registers into
      the 0007 `nodes` table with a dev signing key, the server stores the key
      fingerprint, and the enrollment is audited. Certificate issuance
      (X.509/mTLS) is EXPLICITLY deferred to Phase 2 (ADR-007); the Phase 1
      boundary is the token + signature.
    Backlog: 11
    Acceptance: a token enrolls exactly once (a replay is refused with the audit
      row); the node's row + key land in `nodes`; expired/unknown tokens are
      refused; the existing unauthenticated channel keeps working (the key-proof
      handshake is `.1.2.2`'s contract change); enroll/re-enroll tests green and
      no existing suite regresses.

  - ID: `PHASE-1.2.2`
    Status: `done`
    Goal: authenticated channel + lease/presence (backlog 13's remainder; the
      journal's Phase-1 delta for backlog 12) — the handshake carries a key-proof
      signature over the channel fields (`.1.2.1`'s key), heartbeats renew a
      server-side lease, expiry leaves the node `Offline` with visible presence
      state, and a fencing token guards lease renewal. Reconnect/cursor/version
      negotiation are already proven (`PHASE-0.3.2`).
    Backlog: 13
    Note: the authenticated handshake exposes a latent `.1.2.1` strictness — the
      dev wiring's node id IS the role wire id (`rol_…`), but issuance + enroll
      accepted only `nod_…`. The channel identity space is now both (a superset;
      the `.1.2.1` suites never asserted `nod`-only), recorded in the decision
      record. `CHANNEL_VERSION` is now 2.
    Acceptance: a handshake without a valid proof is refused; lease expiry and
      renewal are observable through the API; every existing channel suite is
      updated to the authenticated contract and stays green; the two-host demo
      still passes.

  - ID: `PHASE-1.2.3`
    Status: `done`
    Goal: durable inbox hardening (backlog 14's remainder) — a retention window
      for delivered rows and a quarantine status (with reason) that the replay
      path skips, plus an inspection surface for both. Filtered delivery by
      eligibility stays with Phase 3's directory.
    Backlog: 14
    Acceptance: a quarantined command is never re-delivered; retention cleanup
      is an explicit operator action with a measured before/after; the existing
      channel/worker suites stay green.

- ID: `PHASE-1.3`
  Status: `done`
  Goal: invitation/subscription semantics — explicit participants, invitations
    accept/decline/timeout, simple subscriptions (the create/read/list/cancel API
    shapes are owned by `.1.1.3`)
  Backlog: 15, 16
  Note: gap census (`2026-09-06`) — the `.6.2` wiring dispatches work IN the
    invite transaction (no acceptance step: `ensure_participant` auto-accepts an
    invited role on its first contribution), there is no accept/decline/expire/
    remove command or invitation record, `allow_join_requests` is typed but inert
    (no join verb, no subscription listing), and `allow_explicit_invites=false`
    is recorded but not enforced. Backlog 16's "offer/reserve/accept/decline/
    expire/remove with snapshot semantics and race tests" therefore needs THREE
    moves: the invitation lifecycle in the state machine, the dispatch-on-accept
    rewiring (the demo + wiring suites move to the explicit contract), and the
    join/subscription surface.
  Children: `.1.3.1`–`.1.3.2` (decomposed `2026-09-06`; amended `2026-09-06`:
    the lifecycle and the dispatch-on-accept move are ONE contract — the
    original `.1.3.2` merged into `.1.3.1`) — both `done`: explicit participants
    + simple subscriptions.

  - ID: `PHASE-1.3.1`
    Status: `done`
    Goal: the explicit-participants contract — the invitation lifecycle in the
      thread state machine AND the dispatch move. `thread.invite` records a
      PENDING invitation (typed optional expiry on the invite body, the
      `invitations` projection map; additive `#[serde(default)]`), the invite
      transaction enqueues NO work; `thread.accept_invitation` (invited role →
      `accepted`, event `thread.invitation_accepted`) is the transaction that
      enqueues the contribute work item WITH the reservation (`work_{accept_event_id}`
      — an accepted invitation exists iff its work does); `thread.decline_invitation`
      (`declined`, event); `thread.remove_participant` (tenant_admin; `revoked` —
      a new core state, event `thread.participant_removed`); expiry is DERIVED
      from `expires_at` at read/accept time (never swept, like the lease
      presence). The invitation IS the acceptance capability (offer/reserve) —
      accept/decline authorize against the pending invitation AND a new
      `thread_invitation_respond` grant (the role's default gains it); an
      invited role may NOT act until accepted (the auto-accept on first
      contribution goes away — `thread.contribute` from an invited role is a
      typed `invitation_pending`). Challenge-dispatch is unchanged (a challenged
      author is already accepted). Wiring suites + the two-host demo move to the
      explicit contract (`rb thread accept` as the role before the node starts).
    Backlog: 16, 15
    Acceptance: every transition is an event with actor + precondition; no work
      item exists before acceptance; the accept dispatches exactly once
      (idempotency + dedupe); accept after decline/expiry/removal is a typed
      refusal; expiry is observable (invitation meta + derived view); race tests
      prove exactly one winner under concurrent accept/decline/remove (the
      aggregate head lock serializes); every existing channel/worker/CLI suite
      moves to the explicit contract and stays green; the two-host demo still
      passes.

  - ID: `PHASE-1.3.2`
    Status: `done`
    Goal: simple subscriptions — `thread.join` (a role joins a thread whose
      `allow_join_requests` is on; the `thread_contribute` grant still gates
      acting; `joined` records the self-request path, event
      `thread.participant_joined`), enforcement of `allow_explicit_invites=false`
      (invite refused on such threads), and a subscription listing on the
      inspection surface (participants with states + timestamps; pending/expired
      invitations). Filtered delivery by eligibility stays with Phase 3's
      directory. CLI: `rb thread accept/decline/join/remove-participant` across
      the children.
    Backlog: 14 (the simple-subscription sliver), 16
    Acceptance: a join on a closed-join thread is a typed refusal; a joined role
      appears in the participant list with its path; the listing exposes
      invitation states; the book + CLI chapters document the surface.

- ID: `PHASE-1.4`
  Status: `done`
  Goal: the second genuinely distinct harness adapter — the Claude-family CLI — plus the deterministic fake for CI
  Backlog: 21 (19 and 20 are Phase-0-proven: the deterministic fake is `.4.1`, the Codex adapter `.4.2`)
  Note: gap census (`2026-09-06`) — backlogs 19 (deterministic fake) and 20 (Codex adapter)
    landed in Phase 0 (`.4.1`/`.4.2`), so `.1.4`'s delta was backlog 21 (Claude-family adapter),
    built as the `.4.2` mirror; the live CLI was INSTALLED (claude 2.1.263), so the real
    harness leg ran for real (env-gated like `RB_LIVE_CODEX`); backlog 22 (generic
    process/MCP adapter) is not Phase 1's need — two genuinely distinct adapters exist.
  Children: `.1.4.1`–`.1.4.2` (decomposed `2026-09-06` at the code-vs-live-qualification seam,
    the `.4.2` mirror) — both `done`: the adapter core + the live-qualified leg.
    **`.1.4` is COMPLETE** (backlogs 19–21: the fake, Codex, Claude).

  - ID: `PHASE-1.4.1`
    Status: `done`
    Goal: the Claude CLI adapter core (`crates/reasonbraid-adapter/src/claude.rs`) — supervises
      `claude -p --output-format stream-json --restricted --tools '' --verbose -- <prompt>` as a
      child process, mapping the VERIFIED 2.1.263 stream: `system/init` (`session_id`) →
      ProviderRequestId; `assistant` text blocks → OutputChunk (thinking blocks skipped — the
      reply is the text); `result` (`is_error:false`) → Completed with usage AND
      `total_cost_usd` (Claude reports money — normalized cost, unlike Codex); non-zero exit →
      FailedKnown with the stderr tail; EOF without a result → lost response. `--restricted`
      removes code-running tools, `--tools ''` disables ALL tools (content-only), and the CLI
      itself REQUIRES `--verbose` with stream-json (the probe that omitted it was refused
      pre-dispatch with exactly that error). The prompt travels as the USER prompt (after
      `--`), never config; the adapter holds no credentials (ambient Claude login).
      Offline stub suite (`tests/claude_adapter.rs`) over the real subprocess boundary.
    Backlog: 21
    Acceptance: the offline suite proves the event mapping (incl. the thinking-block skip and
      the usage+cost normalization), the lost-response/exit-status/cancel/missing-binary legs,
      and the declared capabilities; every existing adapter suite stays green; clippy clean.
    Done (`2026-09-06`): `claude.rs` landed as the `.4.2` mirror over the verified 2.1.263
      interface; the offline suite is 10 tests (the stub exercises the REAL subprocess
      boundary); the acceptance checklist below records the evidence.

  - ID: `PHASE-1.4.2`
    Status: `done`
    Goal: the live qualification leg — `crates/reasonbraid-node/tests/claude_live.rs`
      (`RB_LIVE_CLAUDE=1`, ignored by default) dispatching ONE bounded real run through the
      real supervisor + journal (the `.4.2` codex_live mirror); the dependency-ledger Claude
      row updated with the verified 2.1.263 interface + probe evidence; the book's
      adapter-boundary chapter gains the Claude section; decision record.
    Backlog: 21
    Acceptance: the live test passes against claude 2.1.263 (dispatch → completed, exact usage +
      cost, session id attached as the provider handle, honest unsupported lookup); the ledger
      row carries checked_at + tested version + the probe evidence path; the book + decision
      record land; full regression green.
    Done (`2026-09-06`): the live qualification passed on its FIRST run against the real CLI
      (1 passed in ~2 s — completed, exact usage + money cost, session id attached, honest
      unsupported lookup); the acceptance checklist below records the evidence.

- ID: `PHASE-1.5`
  Status: `active`
  Goal: structured contributions, phases/rounds, evidence attachments, manual close, honest inconclusive outcome
  Backlog: 17
  Note: gap census (`2026-09-06`) — the contribution body's `kind` is a FREE STRING
    (`grep -n 'kind: &str' crates/reasonbraid-server/src/threads.rs` → the contribute arm),
    there are no evidence references, no round fields anywhere, `thread.close` exists but
    the core machine has NO `Inconclusive` terminal (Open/Closing/Closed/Cancelled only —
    `crates/reasonbraid-core/src/state.rs`), and the §8.5 message kinds are untyped prose.
    Votes/abstentions and the workflow PHASE concept are deferred to Phase 5's workflow
    engine (`.1.5` needs rounds, not phases); evidence acquisition stays Phase 4 (`.1.5`
    attaches REFERENCES only — §3.7).
  Children: `.1.5.1`–`.1.5.3` (decomposed `2026-09-06` at the body-vs-rounds-vs-close seams;
    each contract is independent — no incoherent interim possible)

  - ID: `PHASE-1.5.1`
    Status: `active`
    Goal: the structured contribution body — `kind` becomes a typed deny-unknown enum over
      the §8.5 message kinds a contribution can carry (position/claim/assumption/
      evidence_reference/challenge/rebuttal/question/summary — the initial subset; unknown
      kinds preserve through the wire as typed refusals), and contributions gain
      `evidence_refs` (a list of `{uri, digest?, note}` — references ONLY, no acquisition,
      §3.7); additive `#[serde(default)]` projection growth; the CLI passes both; the
      inspection view renders them.
    Backlog: 17
    Acceptance: an out-of-registry kind is a typed refusal; a contribution with evidence
      refs renders them in the inspection view; pre-`.1.5.1` stored projections still parse;
      all existing thread/wiring/CLI suites stay green.

  - ID: `PHASE-1.5.2`
    Status: `proposed`
    Goal: rounds — contributions carry a `round` number (the thread's current round,
      enforced: a contribution must name the current round or advance it by exactly one? —
      the execution leaf settles the rule with a test-first pass), the projection records
      the current round, and the inspection view shows rounds; the two-host demo's blind
      round gains a real round label.
    Backlog: 17
    Acceptance: round assignment + boundary enforcement are typed and tested; the
      inspection view shows the round of every contribution; existing suites + the demo
      stay green.

  - ID: `PHASE-1.5.3`
    Status: `proposed`
    Goal: the honest close — `thread.close` gains `outcome` (`decided` default |
      `inconclusive`) and an `unresolved` register (the items/objections that prevented a
      decision), the core machine gains the `Inconclusive` terminal state + transition
      (distinct from Closed, its own reason field), the inspection view shows the outcome
      and the register; the demo gains an inconclusive-close beat.
    Backlog: 17
    Acceptance: a close with `outcome: inconclusive` lands the thread on the `Inconclusive`
      terminal with the register preserved and inspectable; a decided close behaves exactly
      as before; the state-machine tests cover the new edge; existing suites stay green.

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
| 1 | `PHASE-1.5` | `active` | `.1.4` is COMPLETE (Codex + Claude + the deterministic fake — backlogs 19–21); `.1.5` decomposed (`2026-09-06`) at the body-vs-rounds-vs-close seams: `.1.5.1` (typed contribution kinds + evidence references) → `.1.5.2` (rounds) → `.1.5.3` (honest inconclusive close); next executable leaf `.1.5.1` |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.3, §26.1, backlog 9–22.
- `2026-09-06`: Opened by the Phase 0 go — ADR-002 `accepted` (signed by the accountable owner, `PHASE-0.8.2`); `.1` unblocked.
- `2026-09-06`: `.1` decomposed into `.1.1.1` (aggregate/event/outbox library — backlog 9, ADR-004), `.1.1.2` (migration 0007 identity store — backlog 10), `.1.1.3` (thread command API completion — backlog 15's API-shape portion; the invitation semantics stay with `.1.3`); `.1.3`'s goal reworded to remove the double-claim of backlog 15; frontier → `.1.1.1`.
- `2026-09-06`: `.1.1.1` done — ADR-004 accepted; defect leaf `PHASE-1-MAINT-1` opened (§13 gap in `run_pg_tests.sh`); frontier → `.1.1.2`.
- `2026-09-06`: `.1.1.2` done — migration 0007 identity store + enroll wiring (one transaction, FKs fail closed); decision record `docs/decisions/2026-09-06_identity-store.md`; frontier → `.1.1.3`.
- `2026-09-06`: `.1.1.3` done — thread command API completion (cancel terminal + typed create profiles, stated single-agent default); decision record `docs/decisions/2026-09-06_thread-api-completion.md`; **the `.1` coordinator leaf is complete** — frontier → `.1.2`.
- `2026-09-06`: `.1.2` decomposed (gap census first: enrollment absent, no node leases, no inbox retention/quarantine; backlog 12's journal is Phase-0-proven) into `.1.2.1` (dev-profile enrollment — cert issuance deferred to ADR-007), `.1.2.2` (authenticated channel + lease/presence), `.1.2.3` (inbox retention + quarantine); frontier → `.1.2.1`.
- `2026-09-06`: `.1.2.1` done — one-time enrollment tokens + `node_keys` + audited refusals (denial-row pattern); the suite's first run caught a real defect (a re-issue 500 on the wire — fixed to a typed 409 with a regression assertion) and a test-side status expectation (node-channel `unauthorized` = HTTP 401); decision record `docs/decisions/2026-09-06_node-enrollment.md`; frontier → `.1.2.2`.
- `2026-09-06`: `.1.2.2` done — the authenticated channel (CHANNEL_VERSION 2): HMAC key-proof handshake (refused before any ledger read), lease + fencing token (events/ack/poll/heartbeat ride it; every handshake rotates it), 60 s lease with DERIVED presence (`node_presence` view — expiry flips `offline`, only a fresh handshake restores), `poll` became a POST (the token never rides a query string), and the channel identity space widened to the dev role wire ids (the `.1.2.1` surfaces accepted only `nod_…`; the dev wiring collapses node == role). All 13 channel tests moved to the authenticated contract + 4 new ones; the demo now enrolls its nodes and asserts presence before/after the server restart; decision record `docs/decisions/2026-09-06_node-channel-auth.md`; frontier → `.1.2.3`.
- `2026-09-06`: `.1.2.3` done — durable inbox hardening (migration 0010): quarantine is a row fact WITH its reason and the replay/poll paths ALWAYS skip it (never re-delivered); retention cleanup is an explicit measured operator action (`POST /v1/nodes/inbox/prune`: delivered rows older than the window, before/deleted/after in one transaction); the operator surface is the tenant_admin-audited control API (`POST /v1/nodes/quarantine`, `GET /v1/nodes/inbox`, `POST /v1/nodes/inbox/prune`) + `rb node quarantine|inbox|prune`; new `tests/node_inbox.rs` (3 live-PG tests); decision record `docs/decisions/2026-09-06_node-inbox-retention.md`; **the `.1.2` coordinator leaf is complete** — frontier → `.1.3`.
- `2026-09-06`: `.1.3.2` done — simple subscriptions: `thread.join` (the self-request path: a thread whose `allow_join_requests` is on admits the role as `accepted` with event `thread.participant_joined` carrying `via: join_request`; a closed door and a double join are typed refusals), `allow_explicit_invites=false` is ENFORCED at the invite boundary (typed refusal; the join door still works), the listing surface shows every participant state + invitation meta; `rb thread join`; the fourth `invitations` test; decision record `docs/decisions/2026-09-06_join-subscriptions.md`; **the `.1.3` coordinator leaf is complete** — frontier → `.1.4`.
- `2026-09-06`: `.1.3.1` done — the explicit-participants contract: invite records a PENDING offer (typed `expires_in_seconds`, the additive `invitations` map), the invite enqueues NOTHING, the ACCEPT transaction dispatches the work with its reservation; `thread.accept_invitation`/`thread.decline_invitation` (the invitation IS the capability, gated by the new `thread_invitation_respond` grant — the role default gains it) and `thread.remove_participant` (tenant_admin → core `revoked`); expiry is DERIVED at read/accept (the lease-presence pattern — no sweeper, no expiry event); invited roles may not act (the auto-accept is gone, typed `invalid_transition`); re-invitation allowed over terminal states; the wiring suites + CLI e2e + two-host demo moved to the explicit contract (`rb thread accept` as the role); new `tests/invitations.rs` (3 live-PG tests incl. the concurrent accept/remove race — exactly one winner); decision record `docs/decisions/2026-09-06_explicit-participants.md`; frontier → `.1.3.2`.
- `2026-09-06`: `.1.3` decomposed (gap census first: the `.6.2` invite dispatches work in the invite transaction with NO acceptance step — `ensure_participant` auto-accepts an invited role on first contribution; no accept/decline/expire/remove verbs or invitation records; `allow_join_requests` typed but inert; `allow_explicit_invites=false` recorded but not enforced) into `.1.3.1` (the explicit-participants contract: invitation lifecycle — invite records a pending invitation, accept/decline/remove, derived expiry, the invitation IS the acceptance capability — AND the dispatch move: work enqueues with the ACCEPT event; wiring suites + the two-host demo move to the explicit contract) and `.1.3.2` (simple subscriptions — `thread.join` under `allow_join_requests`, invite enforcement, subscription listing + CLI verbs); frontier → `.1.3.1`. Amended same-day: the lifecycle and the dispatch move are ONE contract (separating them leaves an incoherent interim — work arriving to a role that cannot act), so the original `.1.3.2` merged into `.1.3.1`.
- `2026-09-06`: `PHASE-1-MAINT-1` done — §13 same-volume locality for the ephemeral PG cluster: `scripts/run_pg_tests.sh` now derives `ROOT` at runtime and places the data dir at `$ROOT/target/pg-ephemeral.XXXXXX` (gitignored, per-run unique, trap-cleaned — never `/tmp`); verified by two full reruns (twelve live suites + CLI e2e + demo, both `rc=0`) with a polled on-volume probe and a residue census; decision record `docs/decisions/2026-09-06_same-volume-pg-ephemeral.md`. Frontier unchanged: `.1.4`.
- `2026-09-06`: `.1.4` decomposed (gap census first: backlogs 19/20 — the deterministic fake and the Codex adapter — are Phase-0-proven, so `.1.4`'s delta is backlog 21; the live `claude` CLI is INSTALLED, 2.1.263, so the real leg runs for real) into `.1.4.1` (the Claude CLI adapter core — the `.4.2` subprocess mirror over the VERIFIED `-p --output-format stream-json --restricted --tools '' --verbose` interface, with the offline stub suite) and `.1.4.2` (live qualification + dependency-ledger row + book chapter + decision record); frontier → `.1.4.1`.
- `2026-09-06`: `.1.4.1` done — the Claude CLI adapter core: `claude.rs` supervises `claude -p --output-format stream-json --restricted --tools '' --verbose -- <prompt>` (the interface pinned by 3 live probes BEFORE code: `system/init` session id, assistant text blocks, `result` usage + `total_cost_usd` money, `--verbose` required); the offline suite is 10 tests over a stub binary (real subprocess boundary; thinking blocks skipped, `is_error` results, lost responses, cancel, missing binary); all offline + all twelve live suites + demo green, clippy clean; the book's adapter chapter gains the Claude section; frontier → `.1.4.2`.
- `2026-09-06`: `.1.4.2` done — the live qualification leg: `RB_LIVE_CLAUDE=1 cargo test -p reasonbraid-node --test claude_live -- --ignored` dispatched ONE bounded real run through the real supervisor + journal and passed on its FIRST run (`test result: ok. 1 passed` — completed, exact usage + money cost, session id attached as the provider handle, honest unsupported lookup); the dependency-ledger Claude row now carries the verified 2.1.263 interface (checked_at, tested_versions, conformance), the book gains the live-test command, and the decision record `docs/decisions/2026-09-06_claude-cli-adapter.md` records the whole leaf; **`.1.4` is COMPLETE** (backlogs 19–21: the deterministic fake + Codex + Claude) — frontier → `.1.5`.
- `2026-09-06`: `.1.5` decomposed (gap census first: the contribution `kind` is a free string, no evidence references, no round fields, and the core machine has NO `Inconclusive` terminal — Open/Closing/Closed/Cancelled only; votes/abstentions + workflow phases defer to Phase 5, evidence acquisition to Phase 4) into `.1.5.1` (the structured contribution body: typed §8.5 `kind` enum + `evidence_refs` — references only), `.1.5.2` (rounds: a round number on contributions, enforced at the boundary, visible in inspection), and `.1.5.3` (the honest close: `outcome: decided|inconclusive` + the unresolved register + the core `Inconclusive` terminal); frontier → `.1.5.1`.

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

## Acceptance Checklist (PHASE-1.1.2)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/api.rs` (enroll
wiring), `crates/reasonbraid-server/tests/identity_store.rs` (new), the purge-list
edits in `crates/reasonbraid-server/tests/{command_api,node_work}.rs` and
`crates/reasonbraid-cli/tests/cli_end_to_end.rs`, and `scripts/run_pg_tests.sh`
(all match `\.rs$`/`\.sh$` in `.doctrine/code_paths.txt`); `migrations/0007_identity_store.sql`
is schema (non-code per the same seam).

- [x] **REPRODUCE / ISSUE** — backlog 10 ("initial identity, grant, thread, event, job,
  budget, and idempotency tables") is open; identity exists only as the `.6.1` dev map —
  the §8.1 hierarchy (hosts, nodes, incarnations, runs) has NO rows anywhere.
  `git log -S 'INSERT INTO enrollments' --oneline -- crates/reasonbraid-server/src/api.rs` →
  `35f395d REASONBRAID-PHASE0-0022 (leaf PHASE-0.6.1): …` (the enroll map's only writer;
  no identity writer exists at all).
- [x] **ROOT CAUSE (WHY + WHERE)** — the dev bootstrap needed only the
  (tenant, kind, name) → id map, so `.6.1` stopped there; the §8.1 hierarchy had no
  durable records, which node/incarnation/run lineages will require. The fix point is
  `api.rs`'s enroll transaction (lines 446–512, the ONE-transaction block) plus a new
  migration — `migrations/0007_identity_store.sql` (7 tables, FKs fail closed).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no identity tables;
  enroll wrote 4 rows. After: migration 0007 (7 tables, tenant_id on every material
  record, §17.2) + enroll writes the tenant row (bootstrap) and the identity row in the
  SAME transaction. The new suite caught a test-authored defect on its first run
  (`test result: FAILED. 2 passed; 1 failed` — the human re-enroll omitted `tenant_id`,
  which the dev API reads as a fresh bootstrap); after the correction
  `bash scripts/run_pg_tests.sh` → `test result: ok. 3 passed; 0 failed`
  (`identity_store`: bootstrap commits tenant+identity+enrollment together; role
  identity + replay duplicates nothing; FKs fail closed).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green;
  `bash scripts/run_pg_tests.sh` → all nine live server suites green (`test result: ok.`
  4 + 5 + 9 + 5 + 7 + 3 + 13 + 6 + 7 `passed`) + the real-binary CLI e2e
  `test result: ok. 2 passed` + the two-host demo `ALL acceptance checks passed`
  (12 PASS checks, `rc=0`); `cargo clippy --all-targets --all-features -- -D warnings` →
  clean; `make gate` → `=== all doctrines green ===` (13/13) at commit.
- [x] **FIX** — `migrations/0007_identity_store.sql`; `api.rs` enroll (tenants insert
  in the bootstrap branch, identity row before the enrollment row — parent-row-first,
  FK-enforced); `tests/identity_store.rs` (3 live-PG proofs); the purge lists of
  `command_api`/`node_work`/`cli_end_to_end` gained the identity tables in FK order;
  `scripts/run_pg_tests.sh` registers the suite.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted → `docs/decisions/2026-09-06_identity-store.md` gained `answers:`), MEMORY,
  LIVE_STATUS, this tree's logs below, `docs/TASK_TREE.md` frontier — same commit.

## Acceptance Checklist (PHASE-1.1.3)

The CODE change owned by this leaf: `crates/reasonbraid-core/src/authority.rs`
(`GrantAction::ThreadCancel`), `crates/reasonbraid-server/src/threads.rs` (typed
create fields + the cancel arm), `crates/reasonbraid-server/src/api.rs` (the cancel
route + `ADMIN_ACTIONS`), `crates/reasonbraid-cli/src/{lib,main}.rs` (verb + flags),
and the test files (all match `\.rs$` in `.doctrine/code_paths.txt`).

- [x] **REPRODUCE / ISSUE** — backlog 15's API-shape portion is open: `thread.cancel`
  has no operation (the core `open → cancelled` edge exists but nothing drives it) and
  `thread.create` carries no classification/workflow/participant-rules typing.
  `git grep -n "thread.cancel"` over `crates/` → no wire operation; the core edge is
  provable in `reasonbraid_core::state` (`ThreadTransition::Cancel`).
- [x] **ROOT CAUSE (WHY + WHERE)** — `.6.1` shipped the WP6 verbs (create/invite/
  contribute/challenge/revise/close) and left the two remaining backlog-15 items for
  Phase 1; the fix points are `threads.rs`'s operation catalogue (one arm per verb)
  and the `GrantAction` registry — a new lifecycle verb needs its own authority name,
  never a borrowed one (`docs/decisions/2026-09-06_thread-api-completion.md`).
- [x] **ADDRESSED (verified)** — measured before→after. Before: `thread.cancel` →
  `unknown thread operation` (400); create ignored no profile fields (unknown fields
  were already rejected by deny-unknown). After: `bash scripts/run_pg_tests.sh` →
  `test result: ok. 9 passed; 0 failed` (`command_api`, +2: cancel inspectable/
  terminal/audited; typed fields + stated defaults + rejections) and
  `test result: ok. 2 passed` (`cli_end_to_end`, extended with the typed-create +
  cancel leg). The e2e's FIRST run caught a real defect — the CLI passed
  `--workflow-profile critique-revise` while the wire enum is `critique_revise`
  (`unknown variant … expected one of …`) — fixed by normalizing the human kebab
  spelling to the wire form; the rerun is green.
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green;
  `bash scripts/run_pg_tests.sh` → all nine live server suites green (`test result: ok.`
  4 + 5 + 9 + 5 + 9 + 3 + 13 + 6 + 7 `passed`) + CLI e2e `test result: ok. 2 passed` +
  the two-host demo `ALL acceptance checks passed` (12 PASS checks, `rc=0`);
  `cargo clippy --all-targets --all-features -- -D warnings` → clean; `make gate` →
  `=== all doctrines green ===` (13/13) at commit; `make book` builds.
- [x] **FIX** — `GrantAction::ThreadCancel` (registry + wire-name test extended);
  `threads.rs`: `OP_CANCEL`/`EVENT_CANCELLED`/`CancelBody`, the cancel arm (core
  `open|closing → cancelled`, `cancel_reason` in the projection), typed
  `Classification`/`WorkflowProfile`/`ParticipantRules` with `#[default]` variants and
  additive `#[serde(default)]` projection fields; `api.rs` cancel route + admin set;
  CLI `thread cancel` + the three create flags (+ the kebab→snake normalization);
  `command_api.rs` (+2 tests), `cli_end_to_end.rs` (cancel leg), book `cli.md`/
  `authority.md` updated.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted → `docs/decisions/2026-09-06_thread-api-completion.md` gained `answers:`), MEMORY,
  LIVE_STATUS, this tree's logs below, `docs/TASK_TREE.md` frontier, the book
  chapters — same commit.

## Acceptance Checklist (PHASE-1.2.1)

The CODE change owned by this leaf: `migrations/0008_node_enrollment.sql` (schema,
non-code per the seam), `crates/reasonbraid-server/src/api.rs` (the issue-token
endpoint), `crates/reasonbraid-server/src/node_channel.rs` (the enroll endpoint),
`crates/reasonbraid-cli/src/{lib,main}.rs` (the `rb node issue-token` verb),
`crates/reasonbraid-node/src/channel.rs` + `src/bin/rb-node.rs` (the node-side
enroll client + flags), the purge-list edits, `crates/reasonbraid-server/tests/node_enrollment.rs`
(new), and `scripts/run_pg_tests.sh`.

- [x] **REPRODUCE / ISSUE** — backlog 11 is open: no node enrollment exists (the
  gap census: `grep -rn "node.*enroll" crates/reasonbraid-server/src --include='*.rs'`
  → no matches outside the human/role machinery); the dev rule "a node id IS the
  role wire id" is the only node identity.
- [x] **ROOT CAUSE (WHY + WHERE)** — `.6.2` deferred real node identity ("until the
  Phase 1 directory exists") and the 0007 `nodes`/`hosts` tables were schema-only;
  the fix point is the channel surface + the identity tables — the token row
  (`FOR UPDATE` + `used_at`) is the serialization point, and refusals ride the
  budget engine's denial-row pattern (`docs/decisions/2026-09-06_node-enrollment.md`).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no endpoint, no
  tables. After: `bash scripts/run_pg_tests.sh` → `test result: ok. 3 passed; 0
  failed` (`node_enrollment`: one-time + identity rows; four audited refusal
  classes; tenant-admin-only issuance). The suite's FIRST run caught a real
  wire defect — re-issuing for a node with an unused token returned HTTP 500
  (`duplicate key value violates unique constraint`) — fixed to a typed 409
  `invalid_command` with a regression assertion; and a test-side expectation
  (node-channel `unauthorized` = HTTP 401, not 403) — both corrected, rerun green.
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green;
  `bash scripts/run_pg_tests.sh` → all ten live server suites green (`test result:
  ok.` 4 + 5 + 9 + 5 + 9 + 3 + 13 + 3 + 6 + 7 `passed`) + CLI e2e `test result:
  ok. 2 passed` + the two-host demo `ALL acceptance checks passed` (12 PASS
  checks, `rc=0`); `cargo clippy --all-targets --all-features -- -D warnings` →
  clean; `make gate` → `=== all doctrines green ===` (13/13) at commit; `make
  book` builds.
- [x] **FIX** — migration 0008 (tokens/keys/audit + the hosts get-or-create index);
  `POST /v1/nodes/enroll-tokens` (tenant_admin-audited issuance, typed re-issue
  refusal) and `POST /v1/nodes/enroll` (one transaction: validate → host → node →
  key → consume → audit; refusals commit their audit row); `rb node issue-token`;
  `rb-node --enroll-token/--enroll-nonce/--host-claim/--node-secret`; purge-list
  updates; the book's node-channel chapter now names the `.1.2.1` state.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted → `docs/decisions/2026-09-06_node-enrollment.md` gained `answers:`), MEMORY,
  LIVE_STATUS, this tree's logs below, `docs/TASK_TREE.md` frontier, the book
  chapter — same commit.

## Acceptance Checklist (PHASE-1.2.2)

The CODE change owned by this leaf: `migrations/0009_node_leases.sql` (schema,
non-code per the seam), `crates/reasonbraid-server/src/node_channel.rs` (wire
v2 + proof/lease/presence handlers), `crates/reasonbraid-server/src/api.rs`
(the issue-token identity relaxation), `crates/reasonbraid-node/src/channel.rs`
+ `src/node.rs` + `src/bin/rb-node.rs` (proof client, heartbeat task, required
secret), the channel/wiring test updates, the purge-list edits, and
`scripts/demo_two_host.sh`.

- [x] **REPRODUCE / ISSUE** — backlog 13's remainder is open: the `.1.2.1`
  handshake is unauthenticated (the book's honest-limits said so), no node
  leases/presence exist, and the demo never enrolls its nodes
  (`grep -n "enroll" scripts/demo_two_host.sh` → no node-enrollment before this
  leaf).
- [x] **ROOT CAUSE (WHY + WHERE)** — the channel was Phase-0-proven WITHOUT
  identity by design ("the authenticated streaming profile arrives with WP5
  identity"); `.1.2.1` registered the credential but nothing consumed it. The
  fix point is the handshake (proof + lease issuance) + the fencing token as
  the channel's credential + a DERIVED presence view (no background flipper);
  AND a latent `.1.2.1` strictness — issuance/enroll accepted only `nod_…`
  while the dev wiring's node id IS the `rol_…` role wire id (the demo's own
  contract) — the identity space must accept both
  (`docs/decisions/2026-09-06_node-channel-auth.md`).
- [x] **ADDRESSED (verified)** — measured before→after. Before: unauthenticated
  handshake, no leases, GET poll, `CHANNEL_VERSION 1`. After: HMAC-SHA256
  key-proof (constant-time, refused before any ledger read — missing field 422
  malformed vs wrong proof 401), handshake issues a lease with a fresh
  `fnc_<uuid>` fencing token, events/ack/poll/heartbeat verify the token,
  `heartbeat` renews only a LIVE lease, presence derives `online` from the
  expiry clock (migration 0009 view), poll is a POST. Live proof:
  `bash scripts/run_pg_tests.sh` → `test result: ok. 17 passed; 0 failed`
  (`node_channel`: the 13 original tests moved to the authenticated contract +
  4 new — missing/wrong proof refused, heartbeat renewal + observable
  presence, fencing rotation refused on every surface, expiry → offline →
  re-handshake heals) + the demo asserts presence online before AND after the
  server restart (`ALL acceptance checks passed`, `rc=0`).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green;
  `bash scripts/run_pg_tests.sh` → all ten live server suites green (`test
  result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 17 + 3 + 6 + 7 `passed`) + CLI e2e
  `test result: ok. 2 passed` + the two-host demo `ALL acceptance checks
  passed` (14 PASS checks, `rc=0`); `cargo clippy --all --all-targets -- -D
  warnings` → clean; `make gate` → 13/13 at commit; `make book` builds.
- [x] **FIX** — migration 0009 (`node_leases` + `node_presence`);
  `node_channel.rs` (wire v2: proof-covered handshake, token-guarded
  events/ack/poll, POST poll, heartbeat, presence; `verify_handshake_proof`/
  `verify_fencing`/`issue_lease`/`renew_lease`/`presence`); `api.rs` +
  enroll (`is_valid_node_identity`: `nod_…` OR `rol_…`); the node client
  (secret + shared fencing-token state, `compute_key_proof`, `heartbeat`,
  `NotAuthenticated`); `Node::open` gains the secret; `rb-node --node-secret`
  required + the 15 s heartbeat task; the channel/wiring tests; the demo
  (enrollment + secrets + fencing-token duplicate POST + authenticated poll
  probe + presence evidence); the book's node-channel + two-host-demo
  chapters rewritten.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted → `docs/decisions/2026-09-06_node-channel-auth.md` gained `answers:`), MEMORY,
  LIVE_STATUS, this tree's logs below, `docs/TASK_TREE.md` frontier, the book
  chapters — same commit.

## Acceptance Checklist (PHASE-1.2.3)

The CODE change owned by this leaf: `migrations/0010_node_inbox_retention.sql`
(schema, non-code per the seam), `crates/reasonbraid-server/src/node_channel.rs`
(the replay filter), `crates/reasonbraid-server/src/api.rs` (the quarantine/
inspect/prune endpoints), `crates/reasonbraid-cli/src/{lib,main}.rs` (the three
verbs), `crates/reasonbraid-server/tests/node_inbox.rs` (new), and
`scripts/run_pg_tests.sh`.

- [x] **REPRODUCE / ISSUE** — backlog 14's remainder is open: the inbox has no
  quarantine status and no retention cleanup (`grep -n "quarantine\|prune"
  crates/reasonbraid-server/src crates/reasonbraid-cli/src --include='*.rs'` →
  no matches before this leaf), and the book's honest-limits said so.
- [x] **ROOT CAUSE (WHY + WHERE)** — the inbox was Phase-0-proven for DELIVERY
  only: delivered rows accumulate and nothing can stop a re-delivery. The fix
  point is the row itself (quarantine as nullable columns the replay/poll
  queries filter) + the operator surface (the tenant_admin-audited control API,
  the same gate as token issuance — the authorization record IS the audit, so
  no new audit table) + an explicit measured prune (before/delete/after in one
  transaction — no background sweeper).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no quarantine,
  no prune, replay served every row. After: `quarantined_at`/`quarantine_reason`
  on `node_inbox`; replay + poll filter `quarantined_at IS NULL`; quarantine is
  one-per-row (409 on re-quarantine) with a required reason; prune deletes only
  DELIVERED rows older than the window with `before`/`deleted`/`after` in the
  response. Live proof: `bash scripts/run_pg_tests.sh` → `test result: ok. 3
  passed; 0 failed` (`node_inbox`: quarantine skipped by replay AND poll + the
  reason rides the row + inspection; typed refusals incl. the role 403 and the
  re-quarantine 409; prune measured before/after with only old delivered rows
  gone).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green;
  `bash scripts/run_pg_tests.sh` → all eleven live server suites green (`test
  result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 17 + 3 + 3 + 6 + 7 `passed`) + CLI e2e
  `test result: ok. 2 passed` + the two-host demo `ALL acceptance checks
  passed` (14 PASS checks, `rc=0`); `cargo clippy --all --all-targets -- -D
  warnings` → clean; `make gate` → 13/13 at commit; `make book` builds.
- [x] **FIX** — migration 0010 (two nullable columns); the replay filter;
  `POST /v1/nodes/quarantine` + `GET /v1/nodes/inbox` + `POST
  /v1/nodes/inbox/prune` (tenant_admin-audited via the shared
  `authorize_tenant_admin` helper); `rb node quarantine|inbox|prune`;
  `tests/node_inbox.rs` (3 live-PG tests); the book's node-channel + cli
  chapters.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted → `docs/decisions/2026-09-06_node-inbox-retention.md` gained `answers:`), MEMORY,
  LIVE_STATUS, this tree's logs below, `docs/TASK_TREE.md` frontier, the book
  chapters — same commit.

## Acceptance Checklist (PHASE-1.3.1)

The CODE change owned by this leaf: `crates/reasonbraid-core/src/{state,authority}.rs`
(Revoked/Revoke; the `thread_invitation_respond` grant + canary), `crates/reasonbraid-server/src/threads.rs`
(the lifecycle arms, the `invitations` map, `ensure_participant`, the derived view),
`crates/reasonbraid-server/src/api.rs` (op arms, the dispatch move, role defaults,
the inspection view), `crates/reasonbraid-cli/src/main.rs` (three verbs), the
suite updates, `crates/reasonbraid-server/tests/invitations.rs` (new), and
`scripts/demo_two_host.sh`.

- [x] **REPRODUCE / ISSUE** — backlog 16 is open: the `.6.2` invite dispatches
  work with NO acceptance step and `ensure_participant` auto-accepts an invited
  role on its first contribution (`grep -n "accept_invited" crates/reasonbraid-server/src/threads.rs`
  → the auto-accept arm before this leaf); no accept/decline/expire/remove
  verbs exist (`grep -n "accept_invitation" crates/ --include='*.rs'` → no
  matches before this leaf).
- [x] **ROOT CAUSE (WHY + WHERE)** — `.6.2` needed an end-to-end demo beat, not
  a membership lifecycle; implicit acceptance was the shortcut. The fix point
  is the thread state machine (the invitation becomes a pending offer with the
  core machine's transitions + a new `invitations` time-facts map) AND the
  dispatch site (work rides the accept transaction — an accepted invitation
  exists iff its work does, the `.6.2` invariant moved). The two halves are ONE
  contract: separating them leaves an incoherent interim (work arriving to a
  role that cannot act) — the decomposition was amended accordingly
  (`docs/decisions/2026-09-06_explicit-participants.md`).
- [x] **ADDRESSED (verified)** — measured before→after. Before: invite
  dispatches immediately; invited roles act (auto-accept); no decline/remove/
  expiry. After: invite records a pending offer (typed `expires_in_seconds`,
  `invited_at`/`expires_at` in the event + projection) and enqueues NOTHING;
  accept (`thread.invitation_accepted`) is the dispatch transaction; decline
  (`thread.invitation_declined`); remove (tenant_admin, `revoked`,
  `thread.participant_removed`); expiry DERIVED at read/accept (the state view
  reads `expired`; accept/decline refuse 409); an invited role contributing
  gets the typed `invalid_transition` naming the accept verb. Live proof:
  `bash scripts/run_pg_tests.sh` → `test result: ok. 3 passed; 0 failed`
  (`invitations`: the full lifecycle + typed refusals; decline/expiry/
  re-invitation; the concurrent accept/remove race — exactly one 200, one
  transition event, the snapshot matches the winner).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green;
  `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test
  result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 3 + 17 + 3 + 3 + 6 + 7 `passed`) + CLI
  e2e `test result: ok. 2 passed` + the two-host demo `ALL acceptance checks
  passed` (14 PASS checks, `rc=0`) — the wiring suites, the CLI e2e, and the
  demo all moved to the explicit contract; `cargo clippy --all --all-targets --
  -D warnings` → clean; `make gate` → 13/13 at commit; `make book` builds.
- [x] **FIX** — core (`Revoked` + `Revoke`; `thread_invitation_respond` in the
  registry + canary); threads.rs (three ops/events, the additive
  `invitations` map, the accept/decline/remove arms, `ensure_participant`
  requires `accepted`, `derived_view`); api.rs (three command arms, the
  dispatch moved to accept, the role default gains the respond grant, the
  inspection view derives expiry); CLI verbs; the suite updates (command_api
  audit timeline + happy path, node_work explicit accepts + the no-work-before-
  acceptance assertion, cli e2e accept step); `tests/invitations.rs`; the demo.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted → `docs/decisions/2026-09-06_explicit-participants.md` gained `answers:`), MEMORY,
  LIVE_STATUS, this tree's logs below, `docs/TASK_TREE.md` frontier, the book
  chapters — same commit.

## Acceptance Checklist (PHASE-1.3.2)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/threads.rs`
(the join arm + invite-door enforcement), `crates/reasonbraid-server/src/api.rs`
(the join command arm), `crates/reasonbraid-cli/src/main.rs` (`rb thread join`),
and the fourth test in `crates/reasonbraid-server/tests/invitations.rs`.

- [x] **REPRODUCE / ISSUE** — backlog 14's simple-subscription sliver + 16's
  join door are open: `allow_join_requests` is typed but inert and
  `allow_explicit_invites=false` is recorded but not enforced
  (`grep -n "allow_join_requests" crates/reasonbraid-server/src/threads.rs` →
  only the struct field before this leaf).
- [x] **ROOT CAUSE (WHY + WHERE)** — `.1.1.3` typed the rules without a verb
  to enforce them against (the request flow did not exist). The fix point is
  the command boundary: the join arm checks the door, the invite arm checks
  its door — a recorded rule becomes doctrine the day a boundary refuses on
  it.
- [x] **ADDRESSED (verified)** — measured before→after. Before: join refused
  as an unknown operation; both doors inert. After: `thread.join` admits the
  role on an open-door thread (event `thread.participant_joined`, `via:
  join_request`; the joiner acts immediately; a double join and a closed door
  are typed 400s) and `allow_explicit_invites=false` refuses the invite verb
  while the join door still works. Live proof: `bash scripts/run_pg_tests.sh`
  → `test result: ok. 4 passed; 0 failed` (`invitations`, incl. the new
  join/enforcement test and the accept-vs-decline race).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green;
  `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test
  result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 4 + 17 + 3 + 3 + 6 + 7 `passed`) + CLI
  e2e `test result: ok. 2 passed` + the two-host demo `ALL acceptance checks
  passed` (14 PASS checks, `rc=0`); `cargo clippy --all --all-targets -- -D
  warnings` → clean; `make gate` → 13/13 at commit; `make book` builds.
- [x] **FIX** — threads.rs (OP_JOIN/EVENT_JOINED/JoinBody, the join arm, the
  invite-door refusal); api.rs (the join arm on `thread_contribute`); the CLI
  verb; the fourth `invitations` test; the race test REWRITTEN (the first
  shape raced accept vs remove and the suite itself proved both 200 — they are
  compatible transitions; the conflict pair is accept vs decline).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted → `docs/decisions/2026-09-06_join-subscriptions.md` gained `answers:`), MEMORY,
  LIVE_STATUS, this tree's logs below, `docs/TASK_TREE.md` frontier, the book
  chapter — same commit.

## Acceptance Checklist (PHASE-1-MAINT-1)

The CODE change owned by this leaf: `scripts/run_pg_tests.sh` (matches `\.sh$` in
`.doctrine/code_paths.txt`). Defect leaf opened during `.1.1.1`; the checklist is
written now that the leaf executes.

- [x] **REPRODUCE / ISSUE** — §13 was adopted after the script was written, and
  the ephemeral cluster kept defaulting off-volume:
  `git log -S 'reasonbraid-pg' --oneline -- scripts/run_pg_tests.sh` →
  `6e82e2c REASONBRAID-PHASE0-0013 (leaf PHASE-0.2.1)` — the data dir arrived
  with the `.2.1` script and was never re-touched; the last touch before this
  leaf is a `.1.3.1` suite-list edit (`d7b7673`).
  `grep -n 'TMPDIR' scripts/run_pg_tests.sh` (before this leaf) → line 26
  `TMP="$(mktemp -d "${TMPDIR:-/tmp}/reasonbraid-pg.XXXXXX")"`.
- [x] **ROOT CAUSE (WHY + WHERE)** — a policy adoption is forward-looking: no
  reader re-derived the script's project-owned temp data from the repo root
  after §13 landed, and nothing mechanical checked it (the defect leaf itself
  is that re-check). The fix point is the single data-dir line — derive `ROOT`
  from the script's own location and place the cluster under `$ROOT/target/`
  (already gitignored: `grep -n 'target' .gitignore` → `/target`), keeping the
  `mktemp` per-run uniqueness the `trap cleanup EXIT` (`rm -rf "$TMP"`) relies
  on. `git log -S 'mktemp -d' --oneline -- scripts/run_pg_tests.sh` →
  `6e82e2c` — the mechanics date from `.2.1` and stay untouched.
- [x] **ADDRESSED (verified)** — measured before→after. Before: data dir under
  `${TMPDIR:-/tmp}` (off the repo's volume: repo on `/Volumes/SSD/...`, `/tmp`
  on the system volume). After: `bash scripts/run_pg_tests.sh` creates
  `$ROOT/target/pg-ephemeral.XXXXXX` — the second run's 2 s poll observed it
  mid-run: `PROBE-OK at poll 2 (~4s): target/pg-ephemeral.BPJbkS` (the first
  run's one-shot 25 s probe missed it — timing noise, recorded honestly; the
  polled rerun is the proof) + `CENSUS-OK: no /tmp reasonbraid-pg usage during
  run` + `rc=0`; a standalone timing check shows initdb itself completes in
  ~0.4 s, so nothing hides behind it.
- [x] **NO REGRESSION** — the script itself is the guard set: TWO full runs,
  `bash scripts/run_pg_tests.sh` → all twelve live server suites green
  (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 4 + 17 + 3 + 3 + 6 + 7 `passed`
  in `target/mait1_run.log`/`mait1_run2.log`) + CLI e2e `test result: ok. 2
  passed` + the two-host demo `ALL acceptance checks passed` (both `rc=0`);
  post-run census → `RESIDUE-OK: ephemeral dir cleaned on exit`;
  `bash -n scripts/run_pg_tests.sh` → clean; `make gate` → 13/13 at commit.
  No Rust changed, so the script's own full rerun is the selected guard set
  (§16).
- [x] **FIX** — `scripts/run_pg_tests.sh`: `ROOT="$(cd "$(dirname
  "${BASH_SOURCE[0]}")/.." && pwd)"`, `mkdir -p "$ROOT/target"`,
  `TMP="$(mktemp -d "$ROOT/target/pg-ephemeral.XXXXXX")"`; the header comment
  now states the §13 locality (never `/tmp`, never off-volume).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted → `docs/decisions/2026-09-06_same-volume-pg-ephemeral.md` gained `answers:`), MEMORY,
  LIVE_STATUS, this tree's logs below — same commit. Frontier unchanged
  (`.1.4`), so `docs/TASK_TREE.md` and the book need no update.

## Acceptance Checklist (PHASE-1.4.1)

The CODE change owned by this leaf: `crates/reasonbraid-adapter/src/claude.rs` (new),
`src/lib.rs` (module + re-export), `tests/claude_adapter.rs` (new) — all match `\.rs$`
in `.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — backlog 21 (Claude-family adapter) is open: the Codex
  adapter is the only real one (`ls crates/reasonbraid-adapter/src` → `codex.rs`, no
  `claude.rs` before this leaf; `grep -n "ClaudeCliAdapter" crates/ --include='*.rs'`
  → no matches), and the book's adapter chapter still says whether the second real
  adapter lands "is an open, director-owned question".
- [x] **ROOT CAUSE (WHY + WHERE)** — a harness adapter is the vendor's wire format
  mapped onto the narrow contract; the risk is a GUESSED wire format. The fix point
  is the `.4.2` mirror (`codex.rs`'s supervision mechanics are format-neutral) with
  the Claude stream pinned by LIVE probes FIRST — 3 bounded probes against the
  installed 2.1.263 (evidence on-volume in `target/claude-probes/`): `system/init`
  carries `session_id`; `assistant` text blocks are the reply; `result` carries
  `usage` AND `total_cost_usd`; `--verbose` is REQUIRED (the no-verbose probe was
  refused pre-dispatch: `Error: … stream-json requires --verbose`, `rc=1`).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no Claude module, no
  suite. After: `claude.rs` (the `.4.2` mirror: `system/init` → ProviderRequestId;
  assistant TEXT blocks → one chunk each, thinking blocks skipped; `result`
  `is_error:false` → Completed with the FULL result event — usage under `usage`,
  money under `total_cost_usd`; `is_error:true` → FailedKnown with the provider's
  own message; non-zero exit → FailedKnown with the stderr tail; EOF without a
  result → lost response; `--restricted` + `--tools ''` = content-only; the prompt
  after `--`, never config). Live proof (offline stub over the REAL subprocess
  boundary): `cargo test -p reasonbraid-adapter --test claude_adapter` →
  `test result: ok. 10 passed; 0 failed` (`target/claude_offline2.log`).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green
  (`target/claude_all.log`; the only ignored test is the env-gated codex live
  dispatch); `bash scripts/run_pg_tests.sh` → all twelve live server suites green
  (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 4 + 17 + 3 + 3 + 6 + 7 `passed`) + CLI
  e2e `test result: ok. 2 passed` + the two-host demo `ALL acceptance checks passed`
  (14 PASS, `rc=0`); `cargo clippy --all --all-targets -- -D warnings` → clean;
  `make gate` → 13/13 at commit.
- [x] **FIX** — `claude.rs` (EXEC_ARGS with the REQUIRED `--verbose` and the `--`
  prompt separator, the three event arms + the pending-chunk queue, the
  full-result-event receipt, `total_cost_usd` cost normalization);
  `lib.rs` (`mod claude` + `pub use ClaudeCliAdapter`);
  `tests/claude_adapter.rs` (10 tests; the suite's FIRST run caught a
  test-authoring slip — `test result: FAILED. 9 passed; 1 failed`: the multi-chunk
  assertion loop asserted every chunk while the Claude stub emits two text blocks;
  the first chunk is the verbatim prompt, fixed to assert the first).
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's logs below,
  `docs/TASK_TREE.md` frontier, the book chapter (`adapter-boundary.md` — the
  Claude section lands now, minus the live-test command which is `.1.4.2`'s
  evidence) — same commit. DEV_NOTES: the durable lessons land with `.1.4.2`'s
  decision record (one record for the whole `.1.4` leaf); the code's own doc
  comments carry the verified interface meanwhile.

## Acceptance Checklist (PHASE-1.4.2)

The CODE change owned by this leaf: `crates/reasonbraid-node/tests/claude_live.rs`
(new, `\.rs$` in `.doctrine/code_paths.txt`); the ledger row, book chapter, and
decision record are the non-code deliverables.

- [x] **REPRODUCE / ISSUE** — backlog 21's live leg is open: no env-gated Claude
  live test exists (`ls crates/reasonbraid-node/tests` → `codex_live.rs`, no
  `claude_live.rs` before this leaf), the dependency-ledger Claude row still
  reads "fill at spike" (`grep -n "fill at spike" docs/dependencies/external-ledger.yaml`
  → the three empty Claude rows), and the book's Claude section has no live-test
  command.
- [x] **ROOT CAUSE (WHY + WHERE)** — the `.1.4.1` offline suite proves the
  supervision mechanics against a STUB; the acceptance "a real harness passes
  the contract" needs ONE bounded dispatch through the REAL CLI (the `.4.2`
  `codex_live` pattern: env-gated, ignored by default, journaled through the
  real supervisor). The fix point is that test + the evidence records it feeds
  (ledger `checked_at`/`tested_versions`/`conformance_results`, the book's
  command, the decision record).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no live leg, an
  empty ledger row, no decision record. After: `RB_LIVE_CLAUDE=1 cargo test -p
  reasonbraid-node --test claude_live -- --ignored --nocapture` → `test result:
  ok. 1 passed; 0 failed` (target/claude_live.log) with
  `LIVE CLAUDE OK: attempt patt_01a0784e-… completed via session
  12361df0-aa5b-42b7-864f-03654d76e624` — the REAL harness proved: completed
  state, the reply streamed, input tokens > 0, **cost is Some** (the
  `total_cost_usd` leg Codex cannot prove), the session id attached as the
  provider handle, and `query_status` honestly Unsupported. Without the env var
  the test prints SKIP and passes (never a hidden dispatch).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green (the
  claude_live test SKIPs without `RB_LIVE_CLAUDE`; the codex_live test stays
  ignored); `cargo clippy --all --all-targets -- -D warnings` → clean;
  `make gate` → 13/13 at commit; `make book` builds. No server/CLI path
  changed, so the selected guard set is the offline workspace + the live test
  itself (§16 — the full PG suites re-run at the next server-touching leaf).
- [x] **FIX** — `tests/claude_live.rs` (the `.4.2` codex_live mirror with the
  Claude-specific assertion: `usage.cost.is_some()`); the dependency-ledger
  Claude row rewritten (name → Claude Code CLI, `checked_at` 2026-09-06,
  `tested_versions` 2.1.263, the verified feature/transport/loss/conformance
  facts, the revalidation trigger now keyed on CLI releases); the book's Claude
  section gains the live-test command + the honest-limits bullets now name TWO
  real adapters; `docs/decisions/2026-09-06_claude-cli-adapter.md` (answers:);
  DEV_NOTES entry promoted to it.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted → `docs/decisions/2026-09-06_claude-cli-adapter.md` gained `answers:`), MEMORY,
  LIVE_STATUS, this tree's logs below, `docs/TASK_TREE.md` frontier, the book
  chapter, `docs/decisions/INDEX.md`, KNOWLEDGE_MAP — same commit.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-09-06` | `PHASE-1.1.1` | `cargo clippy --all-targets --all-features -- -D warnings` → clean; `cargo test --all` → every offline suite green (server unit suite `test result: ok. 5 passed` incl. the new `agg::tests`); `bash scripts/run_pg_tests.sh` → all eight live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 7 + 13 + 6 + 7 `passed`) + CLI e2e `2 passed` + two-host demo `ALL acceptance checks passed` (12 PASS, `rc=0`); `make gate` → 13/13 | aggregate/event/outbox library landed; ADR-004 accepted |
| `2026-09-06` | `PHASE-1.1.2` | `cargo clippy` → clean; `cargo test --all` → all offline suites green; `bash scripts/run_pg_tests.sh` → all nine live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 7 + 3 + 13 + 6 + 7 `passed`) + CLI e2e `2 passed` + two-host demo `ALL acceptance checks passed` (12 PASS, `rc=0`); `make gate` → 13/13 | identity store landed (migration 0007 + enroll wiring); the new suite caught a test-authored bootstrap/replay confusion on its first run — fixed, `test result: ok. 3 passed` |
| `2026-09-06` | `PHASE-1.1.3` | `cargo clippy` → clean; `cargo test --all` → all offline suites green; `bash scripts/run_pg_tests.sh` → all nine live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 13 + 6 + 7 `passed`) + CLI e2e `2 passed` + two-host demo `ALL acceptance checks passed` (12 PASS, `rc=0`); `make gate` → 13/13; `make book` builds | thread command API complete — cancel terminal + typed create profiles with stated defaults; the e2e's first run caught the kebab-vs-snake profile spelling, fixed by CLI normalization |
| `2026-09-06` | `PHASE-1.2.1` | `cargo clippy` → clean; `cargo test --all` → all offline suites green; `bash scripts/run_pg_tests.sh` → all ten live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 13 + 3 + 6 + 7 `passed`) + CLI e2e `2 passed` + two-host demo `ALL acceptance checks passed` (12 PASS, `rc=0`); `make gate` → 13/13; `make book` builds | node enrollment landed (one-time tokens + keys + audited refusals); the suite caught a real re-issue-500 defect (fixed to typed 409 + regression assertion) and the 401-vs-403 expectation |
| `2026-09-06` | `PHASE-1.2.2` | `cargo clippy` → clean; `cargo test --all` → all offline suites green; `bash scripts/run_pg_tests.sh` → all ten live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 17 + 3 + 6 + 7 `passed`) + CLI e2e `2 passed` + two-host demo `ALL acceptance checks passed` (14 PASS, `rc=0`); `make gate` → 13/13; `make book` builds | authenticated channel landed (key-proof handshake, lease/fencing, derived presence); the suite's own first runs caught the missing-field-422 vs wrong-proof-401 wire distinction and the tenant-purge FK gap — both fixed, rerun green |
| `2026-09-06` | `PHASE-1.3.2` | `cargo clippy` → clean; `cargo test --all` → all offline suites green; `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 4 + 17 + 3 + 3 + 6 + 7 `passed`) + CLI e2e `2 passed` + two-host demo `ALL acceptance checks passed` (14 PASS, `rc=0`); `make gate` → 13/13; `make book` builds | simple subscriptions landed (`thread.join` + enforced doors); the race test's first shape (accept vs remove) was disproven by the suite itself — both 200 because they are COMPATIBLE transitions; rewritten to the true conflict pair (accept vs decline), rerun green |
| `2026-09-06` | `PHASE-1.3.1` | `cargo clippy` → clean; `cargo test --all` → all offline suites green; `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 3 + 17 + 3 + 3 + 6 + 7 `passed`) + CLI e2e `2 passed` + two-host demo `ALL acceptance checks passed` (14 PASS, `rc=0`); `make gate` → 13/13; `make book` builds | the explicit-participants contract landed (pending invitations, accept/decline/remove, derived expiry, dispatch-on-accept); the first full run's only failure was the expected audit-timeline delta (the accept's audit record) — updated, rerun green |
| `2026-09-06` | `PHASE-1.2.3` | `cargo clippy` → clean; `cargo test --all` → all offline suites green; `bash scripts/run_pg_tests.sh` → all eleven live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 17 + 3 + 3 + 6 + 7 `passed`) + CLI e2e `2 passed` + two-host demo `ALL acceptance checks passed` (14 PASS, `rc=0`); `make gate` → 13/13; `make book` builds | inbox hardening landed (quarantine + measured prune + inspection); the suite's own first runs caught the missing seed tenant and a `(i64,)`-vs-scalar sqlx annotation — both fixed, rerun green |
| `2026-09-06` | `PHASE-1-MAINT-1` | `bash -n` → clean; `bash scripts/run_pg_tests.sh` × 2 → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 4 + 17 + 3 + 3 + 6 + 7 `passed`) + CLI e2e `2 passed` + two-host demo `ALL acceptance checks passed` (both `rc=0`); polled probe → cluster on the repo volume (`PROBE-OK at poll 2 (~4s): target/pg-ephemeral.BPJbkS`), no `/tmp` usage, cleaned on exit; `make gate` → 13/13 | §13 same-volume locality: the ephemeral PG cluster now lives at `$ROOT/target/pg-ephemeral.XXXXXX` (runtime-derived, gitignored, per-run unique, trap-cleaned — never `/tmp`); defect leaf from `.1.1.1` closed |
| `2026-09-06` | `PHASE-1.4.1` | `cargo test -p reasonbraid-adapter --test claude_adapter` → `test result: ok. 10 passed; 0 failed`; `cargo test --all` → every offline suite green; `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 4 + 17 + 3 + 3 + 6 + 7 `passed`) + CLI e2e `2 passed` + two-host demo `ALL acceptance checks passed` (14 PASS, `rc=0`); `cargo clippy --all --all-targets -- -D warnings` → clean; `make gate` → 13/13 | the Claude CLI adapter core landed (`claude.rs` — the `.4.2` mirror over the live-verified 2.1.263 `-p --output-format stream-json --restricted --tools '' --verbose` interface; money cost from `total_cost_usd`); the suite's first run caught a test-authoring slip (multi-chunk assertion), fixed |
| `2026-09-06` | `PHASE-1.4.2` | `RB_LIVE_CLAUDE=1 cargo test -p reasonbraid-node --test claude_live -- --ignored --nocapture` → `test result: ok. 1 passed; 0 failed` (`LIVE CLAUDE OK: attempt patt_01a0784e-… completed via session 12361df0-…`) — completed + exact usage + MONEY cost + session id attached + unsupported lookup, on the REAL harness; `cargo test --all` → all offline suites green; `cargo clippy --all --all-targets -- -D warnings` → clean; `make gate` → 13/13; `make book` builds | the live qualification leg landed (env-gated `RB_LIVE_CLAUDE=1`, first-run pass); ledger row + book chapter + decision record updated; **`.1.4` complete** — two genuinely distinct harness adapters (backlogs 19–21) |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-1.1.1` | `REASONBRAID-PHASE1-0002` | `agg` library + `tx` shim + `tests/aggregate_library.rs` + ADR-004; zero call-site churn |
| `PHASE-1.1.2` | `REASONBRAID-PHASE1-0003` | migration 0007 + enroll identity wiring + `tests/identity_store.rs` + decision record |
| `PHASE-1.1.3` | `REASONBRAID-PHASE1-0004` | `thread.cancel` + typed create profiles + CLI verb/flags + decision record; `.1` complete |
| `PHASE-1.2.1` | `REASONBRAID-PHASE1-0006` | enrollment tokens + node keys + audited refusals + decision record |
| `PHASE-1.2.2` | `REASONBRAID-PHASE1-0007` | authenticated channel v2: key-proof handshake + lease/fencing + derived presence + identity-space relaxation + demo/demo-book updates |
| `PHASE-1.2.3` | `REASONBRAID-PHASE1-0008` | inbox hardening: quarantine (reason-riding row the replay/poll skip) + measured explicit prune + inspection + CLI verbs; `.1.2` complete |
| `PHASE-1.3.1` | `REASONBRAID-PHASE1-0011` | explicit participants: pending invitations + accept/decline/remove + derived expiry + dispatch-on-accept + race tests; `.1.3` decomposition amended in `REASONBRAID-PHASE1-0010` |
| `PHASE-1.3.2` | `REASONBRAID-PHASE1-0012` | simple subscriptions: `thread.join` + enforced participant doors + the accept/decline race; `.1.3` complete |
| `PHASE-1-MAINT-1` | `REASONBRAID-PHASE1-0013` | §13 same-volume locality: ephemeral PG data at `$ROOT/target/pg-ephemeral.XXXXXX`, never `/tmp`; two full reruns green + on-volume probe + residue census |
| `PHASE-1.4.1` | `REASONBRAID-PHASE1-0015` | the Claude CLI adapter core: `claude.rs` (`.4.2` mirror over the verified 2.1.263 stream-json interface) + the 10-test offline stub suite + book section |
| `PHASE-1.4.2` | `REASONBRAID-PHASE1-0016` | the live qualification leg: env-gated `claude_live` (first-run pass on the real harness) + dependency-ledger row + book command + decision record; `.1.4` complete |
