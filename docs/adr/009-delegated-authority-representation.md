# ADR-009 — Delegated authority representation: chain-in-envelope for the dev profile

- **Status:** `accepted` for the development envelope. The original comparative
  wire-size evidence was withdrawn by `SIGNOFF-REPAIR.3.3.1`; `SIGNOFF-REPAIR.3.4.5`
  replaces it with an actual measurement at depths 1–3 (below), which supports the
  choice on size without having been what decided it.
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
  encoding nor delegation depths 1–3, so it established no comparative size
  advantage. `SIGNOFF-REPAIR.3.3.1` replaced that assertion with an actual public
  AuthorityContext serialization/round-trip control.

- **Wire-size measurement (`SIGNOFF-REPAIR.3.4.5`).** The instrument is
  `crates/reasonbraid-core/tests/delegation_representation.rs`; it re-runs with
  `cargo test -p reasonbraid-core --test delegation_representation -- --nocapture`
  and asserts the table below, so this document cannot drift from it. Both shapes
  carry the *same* request and the *same* delegation scope; the baseline is that
  request with no delegation, at **308 B**.

  | Depth | Chain-in-envelope | Δ vs baseline | Token (ES256) | Δ | Token (HS256) | Δ |
  | --- | --- | --- | --- | --- | --- | --- |
  | 1 | 505 B | +197 | 1,066 B | +758 | 1,023 B | +715 |
  | 2 | 684 B | +376 | 1,824 B | +1,516 | 1,738 B | +1,430 |
  | 3 | 861 B | +553 | 2,582 B | +2,274 | 2,453 B | +2,145 |

  Per additional hop the envelope costs **177 B** and a token costs a constant
  **758 B** (ES256) or **715 B** (HS256) — roughly **4.3×**. The gap is
  structural rather than an encoding detail: a credential must carry an issuer,
  an audience, a lifetime, a replay identifier, a key id and a signature, and an
  in-request context needs none of them because the server already knows all six.

  ⚠️ **What this measurement does and does not settle.** Depth 1 encodes the
  SHIPPED `CommandEnvelope`; depths 2–3 are prototype-vs-prototype, because the
  shipped `AuthorityContext` holds one `on_behalf_of` and cannot express a chain
  at all (see Consequences). Every token byte is genuinely encoded except the
  signature block, whose LENGTH is the algorithm's (32 B HMAC-SHA256, 64 B
  Ed25519/P-256) rather than a number chosen here — the distinction that makes
  this different in kind from the withdrawn `N < N + 64`. And size is not the
  interesting axis: tokens buy offline verification and delegation while the
  delegator is unreachable, which no byte count settles. The choice below rests
  on the subtraction and the existing lifecycle, not on this table.
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
- Honest limits: the shipped `AuthorityContext` identifies **one** delegated
  subject and a requested scope. 🔴 **This ADR is titled "chain-in-envelope", and
  no chain exists** — the field is a single object, so a depth-2 delegation has
  nowhere to go. `SIGNOFF-REPAIR.3.4.5` pins that with a control
  (`the_shipped_envelope_carries_one_hop_not_a_chain`) so the title cannot go on
  implying a capability the type does not have, and `.3.4.1` separately measured
  that the grant-chain machinery (`delegable`, `max_delegation_depth`) has no
  producer either. Multi-hop chains, cross-service delegation and high-frequency
  re-presentation require the revisit below. Current delegation constraints and
  authority selection are under `SIGNOFF-REPAIR.3.3`/`.3.4` repair.

## Whether a delegated subject consents (`SIGNOFF-REPAIR.3.4.7`)

**It does not, and in the dev profile that is deliberate: a delegation here is
trusted impersonation inside one tenant, bounded to attribution.** The subject
is not asked and cannot refuse; what the subject's grant does is supply the
authority, and what the audit does is name the subject alongside the actor.

**Consent was never a delegation requirement in this project.** Measured, so the
next reader does not re-derive it: `grep -ic consent ROADMAP.md` returns **8**
and **zero** of them fall inside §16.3 (lines 1754–1768), which is where the
delegation invariants live. The roadmap's consent is §4.4's — an enrollment and
mandate-domain act, carried on the `EnrollmentAuthorityBoundary` as
`target_disclosure_and_acknowledgement`, shown to the target owner before
enrollment completes. That is a different mechanism in a different lane. §16.3
states six invariants and subject consent is not among them. This ADR was silent
on consent because there was nothing to decide.

**What bounds the consequence to attribution**, each clause pinned by a named
control rather than by this paragraph:

| The gate | What it refuses | The control that would fail if it went away |
| --- | --- | --- |
| the actor's own authority | a delegation the actor could not have made alone — `authorize_in_tx` re-evaluates the caller with `delegation: None` against the SAME action and target, and a denial there denies the whole request | `removal_keeps_tenant_binding_and_delegation_attenuation`, which asserts the refusal text `the caller's own authority failed` |
| the subject's authority | a subject whose grant does not cover the action or target | `delegation_succeeds_within_the_subjects_grant_and_refuses_widening` |
| the widening invariant | a claimed scope wider than the subject's grant or narrower than the request's target | `a_delegation_scope_is_the_ceiling_even_for_a_deputy_who_could_act_alone` |
| participation | an actor who is not a participant of the thread, whoever is named as subject | `a_non_participant_actor_cannot_borrow_participation_by_delegating` |

So a delegation reaches nothing the actor could not reach alone. What the
subject loses is not access but **narrative**: an authorization record, which
§16.9 makes high-impact evidence, names a principal as the authority behind an
act it never agreed to.

⚠️ **That is a real cost and it is accepted rather than dismissed.** It is
accepted because the alternative is a mechanism this ADR has already subtracted:
a subject cannot express consent without something for it to issue and something
to verify — an issuance step, a store, a lifetime and a revocation path, which is
option 2 wearing a different name.

**Revisit trigger, mechanical rather than atmospheric:** a delegation that can
cross a tenant boundary, a subject that is not an enrolled principal of the same
tenant, or any relaxation of the four gates above. Each would break the
containment that makes attribution the only consequence, and each is visible as a
failing control rather than as a judgement call.

## Rollback / revisit trigger

- Multi-hop delegation (depth > 2) or a measured re-presentation cost that
  matters — reopen the capability-token option with numbers, not
  anticipation. The numbers now exist for size (above); what would actually
  trigger the revisit is a requirement size does not answer — offline
  verification, or delegation issued while the delegator is unreachable.
  Internet qualification (G6) re-evaluates the transport-side proof as well.
