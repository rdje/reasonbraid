# Phase 1 gate record — G1–G2 + Demonstration A (`PHASE-1.8.2`)

- **gate_id:** `PHASE-1-G1G2`
- **Evidence revision:** `ecceb8d` … the `.1.8.2` close commit (this record ships in it)
- **Date:** 2026-09-07
- **Leaf:** `PHASE-1.8.2` (the evidence manifest: `docs/evidence/2026-09-07_phase1-evidence-manifest.md`)
- **Requirements:** `ROADMAP.md` §20.3 (exit gate), §19.6 (G1/G2), §26.1 (Demonstration A), §19.8 (subtraction record)
- **Outcome:** **Met** — with five named deferrals (each with a revisit trigger) recorded here and in the subtraction record.

## Context

Phase 1 is the trustworthy LAN vertical slice: a coordinator modular monolith
(PostgreSQL, aggregate/event/outbox, typed command API), a Rust node (SQLite
journal, enrollment, authenticated channel, leases/presence, durable inbox),
explicit participants + invitations + subscriptions, two genuinely distinct
harness adapters plus the deterministic fake, structured contributions with
rounds and the honest `inconclusive` close, the read-only inspection console,
and the packaged LAN story (`make dev` / `make release` / `deploy/`).

The exit gate requires G1–G2 to pass and Demonstration A's acceptance to hold.
The measured facts:

- The two-host demo passes **30/30** acceptance checks with REAL kill points —
  server SIGKILL + restart (no accepted command lost), node SIGKILL after
  dispatch (exactly one `outcome_unknown`, no silent retry, no effect),
  duplicate transport → one domain effect, budget denial journaled before
  provider contact, `inconclusive` + the unresolved register, and the audit
  reconstruction through the supported read surfaces only (`target/demo/20260907-021558/`,
  `target/demo/20260907-021623/` — the release-built run).
- The guard set is green: 39 offline suites, 12 live-PG server suites, the
  real-binary CLI e2e, the release-built demo, `make deny`, `make secret-scan`,
  `make gate` 13/13, `make book` — all rc=0 (`target/gate82_*.log`).

## The §26.1 acceptance table (evidence per row)

| Acceptance | Evidence | Verdict |
| --- | --- | --- |
| no manual message relaying between agents | demo beat: the agent content is the adapter's scripted chunks | **Met** |
| restart/reconnect loses no accepted command and creates no duplicate domain effect | demo sections 4–7: `accepted:false` duplicate + exactly one contribution; server SIGKILL restart preserves every accepted command; node SIGKILL after dispatch → exactly one `outcome_unknown`, no revision | **Met** |
| offline inbox and resume cursor work | demo sections 6–7 (work queued while the node is stopped; the restart re-handshakes and picks it up); `node_inbox` (3) + `node_channel` (17) suites | **Met** |
| agent role, incarnation, harness, model/provider route, and attempt remain distinguishable | branded id families incl. `inc`/`run`/`patt`; the 0007 identity hierarchy; attempt records with provider handles — **the incarnation/run row writers are deferred to Phase 2 identity** (deferral #4) | **Met with a named deferral** |
| spend and uncertainty are visible | `GET /v1/threads/{id}/budget` (`.1.6.1`) + the demo's live ledger fetches (THREAD_B's denied reservation row with the engine's reason) | **Met** |
| the system can conclude `inconclusive` with minority/unresolved items | `.1.5.3` core terminal + THREAD_B's demo beat (state `inconclusive`, the register rides the close event) | **Met** |
| all state is inspectable through supported CLI/UI, not database surgery | every demo state assertion runs through the `rb` CLI / control API / console read surfaces / `rb-journal`; the two psql reads obtain the fencing token to forge the duplicate (a credential oracle — a least-privilege API must not expose a live credential; documented in the script) | **Met** |

## G1 — component gate

- **Unit baseline:** 39 offline suites + 12 live-PG suites + CLI e2e, all
  green (rc=0).
- **Property-flavored baseline:** exhaustive transition tables, grant
  canaries, kill-point sweeps (KP-1…KP-9). **Fuzz:** deferred — no
  untrusted-parser surface exists before Phase 4's resource packs (deferral
  #5).
- **Dependency + license checks:** `make deny` → advisories/bans/licenses/
  sources all ok (rc=0); wired in CI (`supply-chain.yml`).
- **Secret scan:** `make secret-scan` → no leaks found (rc=0); CI-pinned
  gitleaks 8.30.1.

## G2 — vertical slice gate

- **Real durable stores:** PostgreSQL + per-node SQLite journals — exercised
  by the demo and the suites.
- **Node journal:** durable boundary records, `outcome_unknown` recovery,
  read-only `rb-journal` inspection.
- **Adapter:** two genuinely distinct real adapters (Codex `.4.2`, Claude
  `.1.4.2` live-qualified) + the deterministic fake as the CI oracle.
- **Recovery demonstration:** the two-host demo's real kill points (above).
- **Exit-gate clauses:** a killed node resumes without duplicated ReasonBraid
  effects ✓ (exactly one ambiguous attempt, no effect); a provider ambiguity
  is visible ✓ (`rb-journal ambiguous` boundary history beat); all accepted
  messages appear once in domain state despite transport redelivery ✓
  (duplicate → `accepted:false`, one contribution; the idempotency
  claim/replay tests).

## Named deferrals (each with a revisit trigger — also in the subtraction record)

| # | Deferred | Revisit trigger | Owner |
| --- | --- | --- | --- |
| 1 | Capability advertisement ("advertise distinct capabilities") | the Phase 3 directory work (`PHASE-3.1`) | Phase 3 |
| 2 | Expected-artifact + manual-decision-rule create fields | the Phase 5 workflow/decision-rule engine; `objective` + typed `workflow_profile` stand in today | Phase 5 |
| 3 | LLM synthesis (moderator/synthesizer) | the Phase 5 deliberation-quality track; the demo's "synthesis + unresolved register" is the register, honestly labelled | Phase 5 |
| 4 | Incarnation/run row WRITERS (schema + id space + attempt records exist) | Phase 2 identity (ADR-007/008) | Phase 2 |
| 5 | Fuzz baseline | the first untrusted parser (Phase 4's resource packs) | Phase 4 |
| 6 | TLS/mTLS, supervision units, containers, PG automation, config files | non-loopback exposure / Phase 2 ops (`deploy/README.md` subtraction table) | Phase 2 |

## Options considered

1. **Met** — accept the evidence above, close Phase 1.
2. **Rework** — nothing in the measured evidence asks for this: every
   acceptance point either passes or is a named, triggered deferral that a
   later phase owns by construction.
3. **Deferred/descoped** — not applicable: the deferrals are phase-owned
   items with owners, not Phase-1 scope silently dropped.
4. **Waiver** — not used: no waivable requirement needed one.

## Release authority

The gate record is the guarantor's evidence package. The release authority is
the accountable owner (director — `docs/decisions/2026-09-06_accountable-owners.md`);
acceptance is their review of this record + the evidence manifest.

## answers:

- **Phase 1 exits on the §26.1 evidence mapped in the table above** — the
  demo's 30 acceptance checks (debug AND release-built) + the guard set are
  the re-runnable proof; every row cites a command or a bundle file, never a
  prose assertion.
- **The thread-scoped audit view starts at the invite** — `thread.create`
  authorizes against the tenant scope (the thread does not exist yet); the
  demo's reconstruction beat asserts exactly that shape.
- **"Inspectable without database surgery" excludes credential reads** — the
  demo's two psql reads obtain the fencing token to forge the duplicate
  transport as the node itself; a least-privilege API must not expose a live
  credential, so the oracle stays, documented in the script.
- **Deferrals are phase-owned with triggers, not omissions** — the five
  numbered deferrals above each name the owning phase and the condition that
  revives it.
