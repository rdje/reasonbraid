# ADR-024 — The MCP interoperability surface: the same handlers behind the tools, and the listen stream is transport state

- **Status:** `accepted` (evidence-gated — the §9.6 MCP-boundary
  contract: the tool vocabulary + the same-handlers mapping, the
  read/write split, the listen-stream durability, the version-profile
  stance)
- **Date:** `2026-09-08`
- **Leaf:** `PHASE-8.3.1`
- **Requirements:** `ROADMAP.md` §9.6 (the MCP boundary)

## Context

The `.3` census measured the greenfield: nothing ships for MCP; the
§9.6 contract is precise; the parking-lot entry (the director's
2026-09-08 brainstorm) fixed the split — the READ half first, the WRITE
half as the qualified capability profile. This record fixes the
vocabulary the `.3.2`–`.3.5` leaves implement.

## Decision

- **The tools ARE the same command handlers.** Every MCP tool
  (`ask_network`, `get_thread`, `respond`, `join_call`, `list_inbox`,
  `propose_policy_change`, `get_policy_bundle`) maps to the HTTP
  handler — the MCP surface is a re-expression of the existing
  verbs, never a new authority path. The resources (the thread
  timelines, the evidence, the authorized policy sets) are the read
  surfaces. A tool that no HTTP handler backs is NOT exposed.
- **The READ/WRITE split (the parking-lot's stance, made the
  contract):** the READ tools ship first — the inspection verbs over
  the cross-store corpus, the grant-scoped principal's ordinary read
  path. The WRITE tools (`respond`, `join_call`,
  `propose_policy_change`) are the QUALIFIED capability profile: the
  MCP client enrolls as a principal (the workload identity — the
  mTLS/cert proof), every write rides a per-verb LOCAL grant, the
  per-principal quota binds the call volume (the `.1.3.2` machinery's
  `principal` scope — the named deferral re-opens), and every effect
  is audited. The remote MCP metadata NEVER grants authority; the
  OAuth/authorization maps to the tenant identity + the scoped
  grants; the tokens are never copied into the thread content.
- **The listen stream is the EPHEMERAL transport state.** The MCP
  2026-07-28 `subscriptions/listen` does not auto-resume over the
  Streamable HTTP reconnect; ReasonBraid therefore treats the stream
  as transport state. The DURABLE state — the subscription, the last
  accepted ReasonBraid cursor, the delivery IDs, the deduplication —
  stays in REASONBRAID. The reconnect ritual: the reauthorize → the
  recreate of the listen request → the reconcile of any
  source-specific gap → the resume from the OWN cursor → the explicit
  possible-gap condition when the upstream offers no replay. The MCP
  continuation is NEVER advertised as stronger than the upstream can
  prove.
- **The version profile:** the official Rust SDK behind
  `reasonbraid-mcp` (the `.3.2` census pins the tested release +
  the protocol profile — the MCP 2026-07-28 baseline), the
  independent conformance fixtures, no maturity-tier or feature-list
  prose — the same demonstrated-compatibility discipline the A2A
  lane fixed.

## answers:

- **The authority path is unchanged**: the MCP tool is the HTTP
  handler's re-expression — the grants, the budgets, the audits, and
  the quotas all apply exactly as they do over HTTP.
- **The durability boundary is the local cursor**: the stream may
  die; the subscription + the cursor + the dedup live in ReasonBraid
  — the resume is the local state's, and the gap is surfaced, never
  papered over.
- **The read-first split is the exposure discipline**: the read tools
  are the low-risk half; the write tools open only as the qualified
  profile — the quota vocabulary already exists to bind them.
