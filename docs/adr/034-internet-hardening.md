# ADR-034 — The Internet hardening: capability per the qualified surface — the transport is context, never identity, and the isolation is defense in depth

- **Status:** `accepted` (evidence-gated — the §16.2/§16.8/§16.11
  contract: the qualified-surface rule, the transport-context rule,
  the defense-in-depth isolation, and the quota/abuse vocabulary
  are the shapes the `.1.2`–`.1.4` leaves implement)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-7.1.1`
- **Requirements:** `ROADMAP.md` §16.2 (the identity and the
  transport), §16.8 (the tenant and the data isolation),
  §16.11 (the abuse, the safety, and the containment)

## Context

The `.1` census mapped §16.2/§16.8/§16.11 against the shipped
surface. The block is LIFTED: the Phase-1 channel + the
CA/certificate infra ship (the mTLS groundwork — LAN-grade), the
Phase-2 authority/budget ship (the ceilings/reservations/
breakers). The SHIPPED halves: the tenant_id on every aggregate
key (the §16.8 shape; the RLS unshipped — the named
defense-in-depth), the budget machinery (the per-thread spend;
the per-principal/per-resolver quotas unshipped), the
Classification (the data-class shape; the regional controls
unshipped). The GREENFIELD: the secret-manager integration, the
regional controls, the per-principal quotas, the mTLS principal
binding at the transport.

## Decision

- **The Internet capability is claimed per the QUALIFIED
  SURFACE, never in general.** The lane's exit names the
  feature set that qualified; adding the remote discovery, the
  arbitrary resources, the governed policy, or another adapter
  REOPENS the applicable G4–G7 portions (the tree's goal line,
  made structural: the qualification records name their
  surface). An unqualified surface exposed by default is the
  typed refusal — nothing ships "experimental-default".
- **The transport is CONTEXT, never identity** (§16.2): the
  overlay, the VPN, the LAN address, the API token, and the
  provider account are the transport context; the principal
  identity is the workload certificate's fingerprint bound to
  the durable principal. The reimaging/incarnation separation
  is structural: the certificate identity, the durable
  principal ID, the role ID, and the incarnation ID remain
  distinct — a changed host never inherits the history.
- **The isolation is DEFENSE IN DEPTH** (§16.8): the
  tenant-scoped aggregate keys are the first layer (shipped);
  the PostgreSQL RLS is the SECOND layer (the `.1.3` work) —
  never the only one. The cross-tenant recruitment stays the
  explicit federation/visibility policy (the opt-in), never
  the default.
- **The quotas bound the abuse by the principal, the tenant,
  the resolver, and the destination** (§16.11): the vocabulary
  rides the Phase-2 budget machinery (the ceilings + the
  breakers) — the notification floods, the invitation storms,
  the expensive loops, and the scraping each have a bound. The
  quarantine preserves the evidence: a quarantined node/thread
  keeps its records (the tombstone doctrine, applied to the
  abuse cases).
- **The secrets and the regions are DECLARED profiles.** The
  secret-manager integration is an interface over the declared
  store profiles (the external secret store is a configuration
  choice, never an ambient dependency); the classification
  drives the storage region, the retention, the export, and
  the evaluator access decisions (§16.8's thread-classification
  controls) — a classification without the controls is the
  typed refusal, never a silent general.

## Consequences

- `.1.2` implements the mTLS workload identity (the transport
  hardening), `.1.3` the tenant isolation + the quotas (the
  RLS second layer + the per-principal bounds), `.1.4` the
  secret-manager interface + the regional/data-class controls —
  each against this contract verbatim; a deviation is a
  contract change.
- The `.5` exit's named capability profile consumes the
  qualified-surface records: the G6/G7 gates close per
  surface, never globally.
- The `.2` enrollment/quarantine lane consumes the
  quarantine-preserves-evidence rule (the evidence survives
  the quarantine — the §16.10/§16.12 lanes' substrate).

answers:

- **The qualified surface is the claim's unit.** A global
  "Internet-ready" claim would be false the first unqualified
  feature exposed; the per-surface qualification keeps the
  claim honest — the same subtraction doctrine the G5 exit
  used for the quality claims.
- **The transport-context rule is the identity's firewall.**
  Every ambient fact (the VPN, the address, the token) is
  context; the certificate fingerprint is the identity — the
  separation is structural, so the transport can change
  without the identity silently changing with it.
- **Defense in depth is the isolation's honesty.** The
  tenant-scoped keys already partition the data; the RLS is
  the SECOND layer that catches the query-level mistakes —
  never the only layer, because a single-layer isolation is
  a single point of failure.
