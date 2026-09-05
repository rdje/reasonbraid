# PHASE-8: federation, interoperability, and ecosystem

## Metadata

- Tree ID: `PHASE-8`
- Status: `proposed`
- Roadmap lane: Phase 8 (`ROADMAP.md` §20.10)
- Created: `2026-09-05`
- Estimate: 16–30 engineer-weeks
- Depends on: stable trust and compatibility contracts
- Exit: G8. External implementations interoperate without sharing ReasonBraid’s database or trusting its internal types.

## Goal

MCP and A2A are interoperability boundaries, not ReasonBraid’s governance
protocol. Record semantic losses. Federation is explicit, not a transitive
default.

## Non-Goals

- Fully decentralized federation in the first Internet-capable release (already a v1 non-goal; this phase is post-v1 unless a narrower interop slice is pulled forward).
- Letting an A2A Agent Card confer ReasonBraid enrollment or governance authority.

## Task Tree

- ID: `PHASE-8.1`
  Status: `proposed`
  Goal: explicit federation trust agreements, tenant-to-tenant visibility, remote recruitment, portable agent cards/profiles, cross-domain audit receipts
  ADR: 026
  Roadmap: §6.6 Federation profile
  Kill: remote domain must not authorize local effects without a local grant (`ROADMAP.md` §25)

- ID: `PHASE-8.2`
  Status: `proposed`
  Goal: current A2A interoperability for compatible task/message exchange
  ADR: 025
  Roadmap: §9.7
  Baseline: official `a2a-rs` crates published as of 2026-09-04; pin and revalidate

- ID: `PHASE-8.3`
  Status: `proposed`
  Goal: MCP servers/clients for tool/resource exposure; ReasonBraid owns durable continuation of listen streams
  ADR: 024
  Roadmap: §9.6
  Baseline: MCP 2026-07-28; `subscriptions/listen` does not auto-resume

- ID: `PHASE-8.4`
  Status: `proposed`
  Goal: adapter/resolver SDK, compatibility matrix, certification suite, signed plugin registry or allowlist
  ADR: 027

- ID: `PHASE-8.5`
  Status: `proposed`
  Goal: regional routing, store-and-forward for intermittent sites, export/import, documented exit path

- ID: `PHASE-8.6`
  Status: `proposed`
  Goal: protocol extension process and independent implementation exercise; G8 exit
  Gate: G8; subtraction record required

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `PHASE-8.1` | `proposed` | blocked on stable trust and compatibility contracts |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.10, §9.6–9.7, ADR 024–027.
