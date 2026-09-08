# ADR-021 — The target deployment: per-target waves, never globally atomic — the receipt attests the digest, the correction authorities stay distinct

- **Status:** `accepted` (evidence-gated — the §15.9–15.11/§4.7
  contract: the per-target waves, the desired/observed pair, the
  receipts, the drift vocabulary, and the correction authorities
  are the shapes the `.5.2`/`.5.3` leaves implement)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-6.5.1`
- **Requirements:** `ROADMAP.md` §23 queue item 021 (the target
  deployment authority and the receipts); §15.9 (the target
  deployment), §15.10 (the runtime attestation and the drift),
  §15.11 (the outcomes and the correction), §4.7 (the
  correction authorities)

## Context

The `.5` census mapped §15.9–15.11 against the shipped surface.
The deployment lane is a GREENFIELD: no target record, no wave,
no receipt, no drift category, no correction record exists
(`git grep -c "deployment_target\|waiver\|retraction" HEAD --
crates/reasonbraid-server/src/` → rc=1). The INPUTS ship: the
`.4` effective publications (the immutable refs + the digests —
the desired-publication half), the lifecycle records (`.2`),
the Phase-4 receipt shapes. The queue's 021 is this lane's ADR.

## Decision

- **The deployment is per-target waves, NEVER globally atomic**
  (§15.9, verbatim): each target carries its own DESIRED and
  OBSERVED state; the rollouts use the waves/canaries; the
  repository commits, the node policy activation, and the
  service configuration cannot share one global transaction.
  "Release complete" is an explicit TARGET COVERAGE THRESHOLD
  plus the named exceptions — never an implicit all-or-nothing.
- **The receipt attests the DIGEST.** A deployment receipt is a
  per-target row: the target + the effective publication's ref
  id + the attested projection digest + the observed state. The
  receipt is the drift detection's comparison input (§15.10:
  the desired vs the observed bytes/digest) — the receipt
  attests what the target OBSERVED, never what the operator
  hoped.
- **The drift is a six-way vocabulary** (§15.10, verbatim): the
  expected override, the pending rollout, the unauthorized
  modification, the unsupported target, the unverifiable load,
  and the stale agent incarnation. The drift record names the
  category + the pair it compared — an unverifiable load is
  reported as `policy_application_unverified`, never guessed.
- **The correction authorities stay distinct** (§4.7): the
  emergency suspension (the fast, scoped, EXPIRING,
  conspicuous), the deployment rollback (the previously
  approved version), the permanent retraction (the preserved
  original + the effective time + the affected targets + the
  reason/evidence/authority/remediation/replacement), the
  supersession (the linked old/new), the waiver (the
  time-bounded exception), the historical correction (the
  additive metadata — never a record deletion). The reversal
  is technically fast; the authority is NOT universally lower
  (§4.7's closing line — the retraction's authority is often
  the adoption's equal).
- **The outcome record links the policy to what happened
  after** (§15.11): the observations, the measurements, the
  incidents, the complaints, the reversals, the unintended
  effects — the review triggers (the elapsed interval, the
  dependency change, the adverse threshold, the external
  standard change, the repeated waiver, the drift, the
  evaluator regression) ride the record.

## Consequences

- `.5.2` implements the deployment records + the waves (the
  targets, the desired/observed pair, the canary assignments,
  the receipts) and `.5.3` the drift + the corrections (the
  six categories, the §4.7 operations, the outcome records) —
  each against this contract verbatim; a deviation is a
  contract change.
- The receipts consume the `.4` publication ref ids: the
  drift comparison is the digest equality, never the prose
  read (the ADR-020 primitive, applied to the targets).
- The `.6` outcome-monitoring leaf consumes the outcome
  records (the scheduled review triggers).

answers:

- **The per-target pair is the deployment's honesty.** A
  global-atomic claim would be a lie the first failed target
  exposes; the desired/observed pair per target makes the
  rollout's truth explicit — the same separate-record
  doctrine as the lifecycle (`.2`).
- **The receipt is an attestation, not an aspiration.** The
  receipt records the OBSERVED digest; the operator's hope
  lives in the desired state — the gap between them IS the
  drift, categorized, never smoothed over.
- **Correction is authority work, not apology.** The §4.7
  table's point is that the retraction/supersession carry
  their own authority proofs (the same grant re-check the
  approvals carry — ADR-032), so a reversal is fast without
  being cheap.
