# Phase 1 SubtractionRecord (`PHASE-1.8.2`; ROADMAP §19.8)

- gate_id: `PHASE-1-G1G2`
- evidence_revision: the `.1.8.2` close commit (this record ships in it)
- Date: 2026-09-07

The architecture ratchet counter: what Phase 1 actively did NOT build, and why
that is a decision rather than an omission. An empty list would need a
recorded explanation; none of these lists is empty.

## features_removed

- **Auto-accept on first contribution** — the `.6.2` implicit membership
  shortcut was REMOVED in `.1.3.1`: an invited role cannot act until its
  accept transaction dispatches the work (a typed `invitation_pending`
  refusal). The invitation is the capability.
- **`GET` polling on the node channel** — replaced by the POST-poll under the
  authenticated contract (`.1.2.2`): a credential never rides a query string.
- **Free-string contribution kinds** — removed in `.1.5.1` (typed
  deny-unknown enum over the §8.5 subset).
- **"Close" without an outcome** — removed in `.1.5.3` (`decided` default,
  `inconclusive` terminal, the dishonest-decided refusal).

## features_deferred (with revisit triggers)

| Feature | Revisit trigger |
| --- | --- |
| Workload certificate lifecycle (X.509/mTLS issuance, rotation, revocation) | non-loopback exposure — Phase 2 identity (ADR-007); `.1.2.1` stopped at the token + key fingerprint |
| Incarnation/run row writers (the 0007 hierarchy is schema + id-space today) | Phase 2 identity (ADR-008) |
| Scoped grants, delegated authority context, cached-decision rules | Phase 2 (`.1` — backlog 11, ADR 008/009) |
| Provider-attempt state machine, usage reconciliation, spend circuit breakers, ambiguous-outcome workflows | Phase 2 (`.3` — backlog 23/25, ADR 012/013) |
| Production leases/fencing, dead-letter/replay operations beyond the Phase-1 inbox | Phase 2 (`.2`) |
| Backup/PITR, object/Git inventory, upgrade/rollback testing | Phase 2 (`.4`) |
| OpenTelemetry, operator dashboards, SLO baselines, game days | Phase 2 (`.5`) |
| Adapter conformance kit + permanent failure fixture corpus | Phase 2 (`.6`) |
| Capability advertisement, directory, semantic matching, recruitment, notification storms | Phase 3 (the demo's two nodes are distinct enrolled roles, not a capability directory) |
| Arbitrary resource acquisition, resolvers, evidence snapshots, derivation graph | Phase 4 — Phase 1 attaches evidence REFERENCES only (`evidence_refs`, no acquisition) |
| Workflow engine, votes/abstentions, decision rules, expected-artifact semantics, LLM synthesis, minority reports | Phase 5 — Phase 1 has rounds + the unresolved register; `workflow_profile` is the typed hook |
| Policy/doctrine system, projection compiler, publication, deployment/correction | Phase 6 |
| Internet hardening, federation, A2A/MCP gateways | Phases 7–8 |
| TLS transport, supervision units (launchd/systemd), container images, PG install automation, config files (flags suffice) | Phase 2 ops — the `.1.7.2` runbook's subtraction table (`deploy/README.md`) |

## product_claims_narrowed

- **No multi-harness end-to-end claim.** The demo runs the deterministic fake
  adapter on both nodes (recorded in every bundle's `env.txt`); the two real
  adapters (Codex, Claude) are each live-qualified by a bounded env-gated
  run, not by the demo.
- **No exactly-once provider-billing claim** — the ambiguity path is the
  claim: `outcome_unknown` is visible and never silently retried.
- **No semantic discovery or directory claim** — invitations and joins only.
- **No binding-governance claim** — the dev authority engine enforces the
  lifecycle; publication/deployment authority is later-phase work.
- **No "deliberation improves answers" claim** — Phase 0's benchmark found
  structure did NOT beat single-agent at 2–4× cost; the Phase-1 routing
  default is single-agent (ADR-002), and the demo proves delivery mechanics,
  not epistemic quality.

## abstractions_or_generalizations_rejected

- No generic "resource" abstraction (evidence refs are URIs + optional
  digest/note — no resolver registry, no snapshot store).
- No workflow-DSL layer (rounds are a projection fact + one verb; profiles
  are typed enums with a stated default).
- No frontend build pipeline or framework — the console is a vanilla static
  page embedded at compile time (`ui-direction`/`ui-embedding` records).
- No broker (NATS/JetStream) — the PostgreSQL outbox + the node channel carry
  delivery.
- No ORM layer beyond the typed `agg` transaction body + the SQL queries the
  surfaces need.

## dependencies_or_services_avoided

- No NATS, no WebSocket/SSE server, no gRPC stack (plain axum HTTP/1).
- No policy engine (OPA/Cedar), no cert issuer, no external cache, no second
  database engine (PostgreSQL + per-node SQLite only).
- No MCP/A2A SDKs, no OTel stack (`eprintln` logging until Phase 2.5), no
  container runtime, no k8s/launchd/systemd units, no object store.
- No frontend toolchain (node/npm untouched — the page is hand-written
  HTML/JS/CSS).

## manual_fallbacks_accepted (with limits)

| Fallback | Limit |
| --- | --- |
| Trusted dev `x-reasonbraid-principal` header + tenant query | loopback dev profile only; workload identity replaces it before any non-loopback exposure (Phase 2) |
| Dev grant issuer stand-in for role enrollment | dev profile; real certificate issuance is Phase 2 (ADR-007) |
| psql as the demo's fencing-token oracle (to forge the duplicate transport) | credential read only — no STATE assertion uses it; a least-privilege API must not expose a live credential (documented in the script) |
| Operator adjudication of `outcome_unknown` attempts | manual, audited, visible via `rb-journal ambiguous` — never automatic retry |
| Fake-adapter determinism in the demo | the real adapters are separately live-qualified; the demo is the delivery proof, not a provider claim |
| Loopback two-node demo by default | the same script drives real two hosts over ssh (`--node-host`/`--remote-workdir`); a real-LAN run is an ops exercise in `deploy/README.md` |

## operations_and_persistent_entities_eliminated

- No sessions/certificates/key-rotation tables beyond the enrollment token +
  node key + lease rows the channel actually needs.
- No policy-storage entities (policy digests ride the authorization rows).
- No per-provider accounting tables beyond the budget ledger's ceilings +
  reservations (denials are rows in the same table — one entity, not two).
- No directory/profile/presence tables beyond the derived `node_presence`
  view (expiry flips offline; no background flipper).
- No separate audit tables — the authorization record IS the audit row.

## estimated effort and risk removed

- The deferred certificate/issuer, object-store, resolver, policy-compiler,
  and broker lanes each carry multi-week estimates plus an external
  dependency or review; keeping Phase 1 on the dev trust store + outbox +
  embedded console removed those before any of them could ratchet into the
  LAN slice.
- Keeping the demo deterministic (fake adapter) removed provider-side flake
  from the acceptance path — every kill point is a scripted, repeatable
  fact.

## proposals retained in parking_lot[]

- OTel/prometheus instrumentation (revived by Phase 2.5).
- WebSocket/SSE streaming (revived when a subscriber surface needs
  bidirectional push — Phase 3+).
- Semantic ranking/embeddings (Phase 3; shadow-mode first).
- A dedicated `reasonbraid-protocol` shared wire crate (revived when a second
  consumer appears — deliberate duplication until then).
- LLM-as-judge graders (rejected on principle in Phase 0; retained as
  rejected — deterministic graders only).
