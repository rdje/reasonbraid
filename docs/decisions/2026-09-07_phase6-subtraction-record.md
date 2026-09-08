# Phase 6 SubtractionRecord (`PHASE-6.7.2`; ROADMAP §19.8)

- gate_id: `PHASE-6-G3`
- evidence_revision: the `.7.2` close commit (this record ships in it)
- Date: 2026-09-07

The architecture ratchet counter: what Phase 6 actively did NOT build, and
why that is a decision rather than an omission. No list is empty.

## features_removed

- **The "binding policy governance" claim** — never shipped, and the
  §25.1 kill/pivot line keeps it WITHDRAWN: the exit claims the
  machinery and the records; the real owners' acceptance of the
  authority/correction model is the named condition.

## features_deferred (with revisit triggers)

| Feature | Revisit trigger |
| --- | --- |
| The binding policy USE | the real owners accept the authority/correction model (§25.1) |
| The remote publication profile | the G7 Internet-qualified profiles (the PHASE-7 lane) |
| The G7 publication portion's full gate | the PHASE-7 publication gate evidence |
| The human administrative checklists + the MCP/host-config projection targets | the projection adapters (the ADR-033 named deferrals) |
| The extract-fixture quarantine (the §19.7 owner + expiry + risk for the `/tmp` pid+nanos flake) | a MAINT leaf |

## product_claims_narrowed

- **"Governance ships" → "governance MACHINERY ships, demonstration-grade."**
  The semantic policy, the lifecycle, the compiler, the publication, the
  deployment, and the correction records exist and test; the BINDING USE is
  unclaimed.

## abstractions_or_generalizations_rejected

- **A re-resolving compiler** (ADR-033): the compiler renders the resolved
  set — a second judge rejected.
- **A derived routing-class classifier** (ADR-031): the class is a
  submitted input.
- **A folded lifecycle record** (ADR-032): the five records never fold.

## dependencies_or_services_avoided

- **The git CLI**: the publisher uses the gix plumbing (the pure-Rust
  doctrine).
- **A randomness dependency**: the trial assignment's splitmix64 is
  dependency-free (Phase 5).

## manual_fallbacks_accepted (with limits)

- **The declared repo path on the publish verb** — the dev-trusted operator
  surface; the `.5` deployment lane's named limitation, tightened by the
  PHASE-7 ops work.

## operations_and_persistent_entities_eliminated

- **No global deployment transaction** — the per-target waves replaced the
  atomic-rollout fiction (§15.9).

## estimated effort and risk removed

- The withdrawn binding-use claim removes the external-owner dependency
  (the acceptance round-trips) from the exit path; the shadow-only
  recommendation (Phase 5) keeps the learned-routing risk surface closed.

## proposals retained in parking_lot

- The binding-policy enablement (the owners' acceptance + the G7
  evidence).
- The remote publication profile + the signed transport.

## owner, reviewers, rationale, signatures

- Owner: the Phase-6 lane owner (director-attested records). Reviewers:
  the gate package rides the doctrine gates (13/13) + the full guard.
  Rationale: §25.1's pre-Phase-6 gate executed verbatim — the binding
  governance stays unshipped until the real owners accept the model.
