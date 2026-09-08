# ADR-026 — The federation trust agreement: explicit, named, and local-grant-gated

- **Status:** `accepted` (evidence-gated — the §6.6 federation-profile
  contract: the explicit-agreement shape, the local-grant rule, the
  visibility/recruitment/receipt vocabulary over the shipped
  machinery)
- **Date:** `2026-09-08`
- **Leaf:** `PHASE-8.1.1`
- **Requirements:** `ROADMAP.md` §6.6 (the Federation profile)

## Context

The `.1` census lifted the block (the trust + the compatibility
contracts ship + are measured) and split the lane: the visibility
scopes ship in their intra-tenant form (the network-pseudonym class +
the ADR-034 explicit opt-in); the remote recruitment, the portable
cards, and the cross-domain receipts are the greenfield. The kill
line stands: "remote domain must not authorize local effects without
a local grant" (`ROADMAP.md` §25). This record fixes the vocabulary
the `.1.2`–`.1.4` leaves implement.

## Decision

- **The federation is EXPLICIT — a NAMED agreement, never a
  transitive default.** Two domains federate by recording the
  agreement (the tenant-to-tenant pairing with its scope); a third
  domain gains NOTHING transitively. The tree's goal line is the
  mechanical rule: no agreement record, no federation effects.
- **The remote domain NEVER authorizes local effects.** Every local
  effect rides a LOCAL grant (the Phase-2 authority machinery — the
  boundary + the grant the local operator issued). The remote
  agreement is the capability source for the REMOTE half of a
  federated action only: the remote domain vouches for its own
  records, the local domain acts on its own grants. The kill line is
  the implementation's invariant, not a policy hope.
- **The visibility rides the shipped scopes + the opt-in.** Without
  an agreement, a remote tenant sees the network pseudonym (the
  shipped class) and nothing more; the agreement's scope widens the
  visibility to exactly what it names — the ADR-034 stance applied
  to the cross-tenant form.
- **The remote recruitment is the agreement-scoped opt-in.** The
  cross-tenant recruitment (the Phase-3 call machinery's remote form)
  accepts remote participants ONLY under the agreement's named
  scope; the explicit acceptance is the participant's and the
  operators' — never the default.
- **The cross-domain audit receipts CROSS-REFERENCE, never merge.**
  A receipt names the remote domain's OWN records (its digest-pinned
  references); the local audit chain stays the local truth (the
  ADR-022 groundwork's federation form — the remote chain is
  verifiable, not authoritative).
- **The portable cards verify through the ADR-027 ladder.** The
  exported agent card/profile is the digest-pinned portable form
  (the §10.1 profile + the capability declaration); the IMPORTING
  domain runs the five-rung ladder (allowlist → digest → signature →
  compatibility → capability) before the card confers anything.

## answers:

- **The agreement record is the federation's single capability
  source**: no record, no cross-domain effect — the explicitness is
  checkable, not aspirational.
- **The local grant is the ONLY authority that acts locally**: the
  remote domain vouches for its own records; the local effects are
  the local grants' business (the kill line as the invariant).
- **The receipts and the cards reuse the shipped verification
  machinery**: the digest-pinned references + the ADR-027 ladder —
  the federation adds the POLICY layer, not new trust primitives.
