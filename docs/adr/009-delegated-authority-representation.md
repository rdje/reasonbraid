# ADR-009 — Delegated authority representation: chain-in-envelope for the dev profile

- **Status:** `accepted` for the development envelope; comparative wire-size
  evidence is withdrawn by `SIGNOFF-REPAIR.3.3.1`. The subset prototype remains
  historical evidence; representation benchmarking is owned by `.3.4`.
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-2.1.4.1`
- **Requirements:** `ROADMAP.md` §23 queue item 009; §16.3 (delegation
  invariants); §4.2 (scoped grants)

## Context

Phase 2's `.1.4` adds delegation: an enrolled actor acts on behalf of another
principal whose grant is the authority source. §16.3 fixes the invariants —
a delegate cannot widen a grant/duration/tenant/target/cost/approval,
forwarding preserves the chain, both caller and subject permission are
evaluated — but the REPRESENTATION is open: (a) **chain-in-envelope** (the
request carries the structured delegation context) or (b) **attenuated
capability tokens** (the delegator issues a separately signed credential the
delegate presents).

## Options

1. **Chain-in-envelope** — the envelope gains an optional structured
   `authority_context` (`on_behalf_of` + `purpose` + `scope`); the server
   evaluates it at the boundary and records it in the audit row.
2. **Capability tokens** — a signed, expiring token issued by the delegator,
   presented by the delegate; needs issuance, storage, and verification
   machinery plus a token lifecycle.

## Evidence

- **The plumbing is pre-shaped for option 1**: `CommandAuthz.delegate_subject`
  exists and the authorization-records INSERT already writes the subject
  split when delegation applies (the `.1.4` census — `grep -n
  "delegate_subject" crates/reasonbraid-server/src/authority.rs` → lines
  45/579). Option 2 needs none of the existing machinery.
- **The widening invariant is a pure function** — the spike implemented
  `delegation_scope_is_subset` + `DelegationConstraints` in the core crate
  with offline tests (narrower passes, equal passes, empty passes, a
  tenant-wide request over a thread-scoped grant refused, a foreign thread
  refused): `cargo test -p reasonbraid-core` → `test result: ok. 39 passed`.
- **Wire-size correction:** the historical test hand-built JSON, computed its
  byte length N and asserted N < N + 64. It measured neither a capability-token
  encoding nor delegation depths 1–3, so it establishes no comparative size
  advantage. `SIGNOFF-REPAIR.3.3.1` replaces that assertion with an actual public
  AuthorityContext serialization/round-trip control; `.3.4` owns the missing
  comparative prototype and measurements before a size advantage is claimed.
- **Expiry and revocation ride the existing `.1.3` filters**: the subject's
  grant is the authority source; a revoked grant refuses the delegation at
  the next decision with zero new machinery. A token would duplicate that
  lifecycle.

## Choice

Option 1 for the dev profile: the delegation is structured envelope data; the
authority source stays the subject's grant (checked by the existing
`status = 'active'` filters), the dual check (caller AND subject) rides the
same evaluation, and the audit row carries the chain.

## Consequences

- No token issuance/store/signature layer (subtraction); no new credential
  lifecycle to reconcile with `.1.3`.
- Honest limits: the shipped AuthorityContext identifies one delegated subject
  and a requested scope; the source prototype did not measure a multi-hop chain
  at depths 1–3. Multi-hop chains, cross-service delegation and high-frequency
  re-presentation require the revisit below. Current delegation constraints and
  authority selection are under `SIGNOFF-REPAIR.3.3`/`.3.4` repair.

## Rollback / revisit trigger

- Multi-hop delegation (depth > 2) or a measured re-presentation cost that
  matters — reopen the capability-token option with numbers, not
  anticipation. Internet qualification (G6) re-evaluates the transport-side
  proof as well.
