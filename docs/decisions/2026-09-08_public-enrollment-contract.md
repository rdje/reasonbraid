# The public-enrollment contract — the vetting is re-derivation, the suspicion quarantines through the shipped verbs, and the exposure stays a qualified profile (`PHASE-7.2.4`)

- Date: 2026-09-08 · Leaf: `PHASE-7.2.4` · Decision record (the §16.10/§16.12 enrollment policy)

## Context

The `.2` census mapped the shipped surface: the dev enrollment (the
one-time tokens + the server-as-trust-store), the workload-identity CA (the
leaf + the cert proof), the quarantine (the explicit verb + the dead-letter
auto-quarantine + the `.1.3.3` evidence rule), and the revocation
propagation (the cert revocation + the suspended presence + the epoch fence
at the dispatch + the replacement drill). The PUBLIC enrollment — an
unvetted node entering through a public surface — has no policy record.
This record is the contract: the vocabulary the future exposure consumes.
The EXPOSURE itself stays OFF (the `.5` kill/pivot: no remote enrollment
while the qualification gate is incomplete).

## Decision

- **The enrollment is a STAGED vetting ladder.** (1) The identity claim —
  the node declares its §8.1 facts (the incarnation writer's shape). (2)
  The capability declaration — the node names its work scope (the adapter
  set + the profile it serves). (3) The operator's acceptance — the
  explicit grant; the dev profile's one-time token IS the pre-vetted
  acceptance. (4) The certificate issuance — the CA leaf, the workload
  identity. The PUBLIC form adds ONE stage before the acceptance: the
  claim VETTING — the operator re-derives the claim against its evidence
  (the same discipline the SECURITY.md vetting rides: a claim is verified,
  never trusted on its wording).
- **The suspicion → the quarantine is the OPERATOR's typed action over the
  SHIPPED machinery.** The suspicion vocabulary is the §16.11 observable
  set the Phase-2 counters already measure (the repeated authorization
  failures, the abnormal fan-out, the quota denials, the abnormal
  publication attempts). A suspicion crossing triggers the typed
  quarantine verb (the shipped operator action) + the evidence
  preservation (the `.1.3.3` rule: the quarantine is a ROW FACT, the
  retention never deletes it). No new silent state — the suspicion is a
  decision with an audit record, exactly like a refusal.
- **The revocation propagation INHERITS the shipped ladder verbatim.** The
  public-enrollment lifecycle's revocation = the cert revocation → the
  suspended presence → the epoch fence at the next dispatch → the
  replacement drill (the `.2.7.2` measured ritual). The contract maps it;
  nothing new builds.
- **The exposure is a QUALIFIED profile, never an experimental default.**
  The public enrollment endpoint opens only under the `.5` gate: the
  G6/G7 evidence for the NAMED capability profile (the qualified-surface
  rule — ADR-034). Until then the enrollment stays the dev/LAN shape.

## answers:

- **The vetting is re-derivation, never trust**: the public form differs
  from the dev form in ONE stage — the operator verifies the claim
  against its evidence before the acceptance; the rest of the ladder is
  already shipped.
- **The suspicion quarantines through the shipped verbs**: the observable
  signals exist (the counters), the action exists (the quarantine verb),
  the preservation exists (the evidence rule) — the public enrollment
  adds the POLICY that connects them, not new machinery.
- **The revocation ladder is the public lifecycle's revocation path**:
  the measured ritual (the replacement drill) is the reference
  implementation the exposure reuses.
- **The kill/pivot holds**: the exposure waits for the complete
  qualification gate — the contract exists so the gate has a vocabulary
  to judge against, not so the endpoint opens early.
