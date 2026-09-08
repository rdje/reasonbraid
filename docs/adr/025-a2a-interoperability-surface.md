# ADR-025 — The A2A interoperability surface: a facade with recorded semantic losses, pinned and demonstrated

- **Status:** `accepted` (evidence-gated — the §9.7 A2A-boundary
  contract: the facade boundary, the semantic-losses vocabulary, the
  version-pin + the revalidation stance)
- **Date:** `2026-09-08`
- **Leaf:** `PHASE-8.2.1`
- **Requirements:** `ROADMAP.md` §9.7 (the A2A boundary)

## Context

The `.2` census measured the greenfield: nothing ships for A2A, the
§9.7 contract is precise, and the official `a2a-lf`/`a2a-client-lf`/
`a2a-server-lf` crates + the `a2a-cli` exist on crates.io (the
2026-09-04 baseline, re-checked — pre-1.0, so the compatibility must
be DEMONSTRATED, never inferred from SemVer). This record fixes the
vocabulary the `.2.2`–`.2.4` leaves implement.

## Decision

- **The facade boundary: A2A is the interoperability facade, NEVER the
  governance protocol.** The compatible semantics map (the discovery,
  the tasks, the messages, the status, the artifacts — where they
  align); the A2A message is an INPUT the local machinery evaluates —
  the local grants authorize every local effect, the local budgets
  bound every cost, the local policy digests decide every rule. The
  remote protocol confers NO authority (the Agent Card proves or
  describes an external service; it confers nothing — the same rule
  the federation's card import enforces).
- **The semantic losses are RECORDED, per exchange.** Each A2A message
  records which dimensions survive and which are lost: the authority
  (the remote's authority is not the local grant), the budget (the
  remote's cost is not the local ceiling), the evidence (the remote's
  references are the digest references, never the local evidence
  objects), the decision rule (the remote's rules are not the local
  policy digests), the policy lifecycle (the remote's task states are
  not the thread states). The record is the message's shadow — a
  silent merge is the failure mode this vocabulary forbids.
- **The version profile: the JSON-RPC/REST profile first, pinned and
  demonstrated.** The `.2.2` census pins the EXACT crate versions (the
  `Cargo.lock` committed — the application pin), selects the profiles
  the slice actually tests (test only what is needed), and records the
  tested A2A specification/conformance revision; a Git SHA is used
  ONLY for an explicitly documented unreleased fix. The pre-1.0
  compatibility is demonstrated by the `.2.4` roundtrip — never
  inferred.
- **The qualification stance: the A2A qualification precedes any broad
  interoperability claim; the gateway is deployable-off.** The dev
  profile ships the facade + the measured exchange; the "broad
  Internet agent interoperability" claim stays gated until the
  qualification record lands.

## answers:

- **The facade maps semantics and records the losses**: every A2A
  exchange carries its loss record — the local governance is the only
  authority that acts, and the record makes the boundary honest.
- **The version pin is the demonstrated-compatibility contract**: the
  exact versions + the tested revision + the roundtrip — the pre-1.0
  reality is met with the demonstration, not the assumption.
- **The qualification gate holds**: the facade ships deployable-off;
  the broad claim waits for the qualification record.
