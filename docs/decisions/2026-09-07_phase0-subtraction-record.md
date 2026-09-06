# Phase 0 SubtractionRecord (`PHASE-0.8.1`; ROADMAP §19.8)

- gate_id: `PHASE-0-G0`
- evidence_revision: `657be6e` + the WP8 commit (this package)
- Date: 2026-09-07

The architecture ratchet counter: what Phase 0 actively did NOT build, and why
that is a decision rather than an omission. An empty list would need a
recorded explanation; none of these lists is empty.

## features_removed

- `cancelled_known` provider-attempt state — dropped from the Phase 0 machine
  (a cancelled attempt whose provider fate is unproven is `outcome_unknown`;
  cancellation is `BestEffort` and never claimed stronger).
- The §20.2 experiment list's breadth — see `features_deferred`.

## features_deferred (with revisit triggers)

| Feature | Revisit trigger |
| --- | --- |
| Authenticated streaming channel (HTTP/2/gRPC, mTLS workload identity) + signed reservations | any non-loopback/`127.0.0.1` exposure (Phase 2 identity/recovery; WP7) |
| WebSocket/SSE transport spike + NATS JetStream candidate evaluation | the Phase 1 slice needs multi-host delivery beyond the loopback profile |
| Policy engine spike (OPA/Cedar candidates) | governance experimentation (Phases 5–6); the dev engine suffices until then |
| Git publication reconciliation + object store experiments | evidence-enabled deliberation preview (Phase 4) |
| MCP / A2A Rust interop spikes | a Phase 1+ integration need names a concrete boundary |
| Second real adapter (Claude-family) | Phase 1 (recorded recommendation in `2026-09-06_real-adapter-codex.md`) |
| `reasonbraid-protocol` shared wire crate | a second consumer of the wire types appears (each side keeps its own `deny_unknown_fields` contract until then — deliberate duplication) |
| Directory / node-binding registry | Phase 1 directory (the dev rule node-id = role-id stands in) |
| `README_POLICY.md` upstream revision review | deliberate review — owned by `PHASE-0-MAINT-1` (parking-lot row 2026-09-06) |

## product_claims_narrowed

- **No claim that structured deliberation beats a single well-prompted agent.**
  The WP7 real run (4 differential cases) found structure did NOT beat single
  at 2–4× the cost — the Phase 1 routing default is single-agent, structured
  workflows only behind case-class justification (`§13.8`).
- **No multi-harness claim.** One real adapter (Codex CLI) is qualified; the
  two-host demo runs on the deterministic fake adapter (recorded in every
  bundle).
- **No exactly-once claim** (`R-EXACT`): the system states what is known,
  unknown, retryable, and potentially duplicated — never more.
- **No calibration claim** from a 4-case feasibility sample.

## abstractions_or_generalizations_rejected

- A generic "resource" abstraction for Web/Git/browser access — not built; the
  capability packs stay future work.
- A provider-agnostic "agent runtime" layer — the adapter contract is the only
  boundary; the node is concrete.
- Executor-generic database plumbing beyond the in-tx variants the command
  transaction actually needs (`.6.1`/`.6.2`).
- An LLM-as-judge grader — rejected on principle (correlated errors), replaced
  by deterministic graders.

## dependencies_or_services_avoided

- No NATS/JetStream, no message broker of any kind.
- No WebSocket/SSE server layer (plain axum HTTP/1).
- No policy engine dependency (OPA/Cedar), no cert-issuer service, no external
  cache, no second database engine (PostgreSQL + per-node SQLite only), no MCP/
  A2A SDKs, no OTel stack (eprintln logging until the observability work).

## manual_fallbacks_accepted (with limits)

| Fallback | Limit |
| --- | --- |
| Trusted dev `x-reasonbraid-principal` header | loopback dev profile only; MUST be replaced by workload identity before any non-loopback exposure |
| Single-writer transaction assumptions (dev profile) | Phase 2 concurrency hardening before multi-writer deployments |
| Operator adjudication of `outcome_unknown` attempts | manual, audited, visible via `rb-journal ambiguous` + the handshake — never automatic retry |
| Dev grant issuer stand-in for role enrollments | dev profile; real certificate issuance is Phase 2 |

## operations_and_persistent_entities_eliminated

- No sessions, certificates, key, or credential tables (the design needs none
  in Phase 0 — credentials resolve out of band at the adapter).
- No node-binding table (the dev rule node-id = role-id replaced it).
- No policy-storage entities (policy digests ride the authority rows).
- No per-provider accounting tables beyond the budget engine's reservations
  (denials are rows in the same table — one entity, not two).

## estimated effort and risk removed

- The deferred spikes (§20.2's full experiment list) removed roughly 6–8
  engineer-weeks of parallel spike work from the Phase 0 critical path at the
  cost of narrower claims (recorded above).
- Phase 0 measured effort landed within the 8–14 engineer-week roadmap range
  (KICKOFF WP0–WP8 sum ≈ 9.5); the 2×-estimate review is NOT triggered
  (threshold 28 engineer-weeks / twice the range — neither exceeded; recorded
  so the arithmetic is visible).

## proposals retained in parking_lot

- See `docs/parking-lot.md` (the README_POLICY review row and its peers).

## owner, reviewers, rationale, signatures

- Owner: Richard DJE (architecture + security; `docs/decisions/2026-09-06_accountable-owners.md`)
- Rationale: every removed or deferred item above either lacks executable
  evidence, belongs to a later gate, or would widen a claim the Phase 0
  measurements did not support.
- Signatures: Richard DJE — **pending the director's review of the Phase 1 go
  recommendation (ADR-002)**.
