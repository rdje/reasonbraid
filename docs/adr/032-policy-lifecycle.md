# ADR-032 — The policy lifecycle: five records that never fold — the discussion rides the thread, the approval carries its authority proof

- **Status:** `accepted` (evidence-gated — the §4.5/§15.6 contract:
  the separate-record lifecycle, the proposal's references, the
  decision's electorate snapshot, and the approval's authority
  proof are the shapes the `.2.2`/`.2.3` leaves implement)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-6.2.1`
- **Requirements:** `ROADMAP.md` §4.5 (the decision rules and the
  authority proofs), §15.6 (the policy proposal lifecycle)

## Context

The `.2` census mapped §15.6 against the shipped surface. The
lifecycle records are a GREENFIELD: no proposal row, no approval
row, no decision row exists (`git grep -c "proposal" HEAD --
crates/reasonbraid-server/src/` finds only the profile/step
names). The substrate ships: the `.1` policy registry (the
proposal targets a policy version), the Phase-5 deliberation
machinery (the discussion rides a THREAD — the
contribute/challenge/revise/verdict verbs, the twelve terminals,
the minority report), the Phase-2 authority model (the
grants/audit — the proof's substrate), and the thread's
participants (the electorate snapshot's source).

## Decision

- **The five records never fold.** The discussion, the
  decision, the approval, the publication, and the deployment
  are SEPARATE records (§15.6's closure list): the discussion
  rides the existing THREAD machinery (the verdict kind is its
  decision-shaped contribution — one thread may yield multiple
  decisions); the decision, the approval, the publication, and
  the deployment are their own typed rows, each referencing
  the previous stage's record id. A lifecycle stage that skips
  its predecessor is the typed refusal (no approval without a
  decision, no publication without an approval).
- **The proposal is a reference, not a copy.** The proposal
  record references the target POLICY VERSION (the `.1`
  registry's digest-pinned row) and the DELIBERATION THREAD
  that discusses it. The proposal never embeds the policy
  content — the version reference + the digest keep the
  lineage honest (a policy change is a new version, a new
  proposal, never a silent edit).
- **The decision records the electorate snapshot.** The
  decision row carries the rule it applied + the ELECTORATE
  SNAPSHOT (the participants + the denominator + the
  abstentions at the decision time — §4.5's "eligible
  electorate snapshot") + the verdict reference it rests on.
  The snapshot is frozen AT the action time — a later
  membership change never rewrites a past decision.
- **The approval carries its authority proof.** The approval
  row names the approver + the GRANT the approval rests on;
  the approval boundary RE-CHECKS the grant (the status, the
  expiry, the scope) at the approval time — the authority at
  the action time, not at the proposal time (§4.5's
  "participant/approver identity and authority at action
  time"). An approval whose grant lapsed is the typed refusal.
- **The publication and the deployment stay out of this
  contract's scope** (the `.4`/`.5` leaves' — ADR-020/021) —
  but the record ids they will reference are reserved here:
  the decision/approval rows are the inputs the publication's
  authority verification consumes.

## Consequences

- `.2.2` implements the proposal + the decision records (the
  typed rows, the references, the electorate snapshot) and
  `.2.3` the approval records + the authority proofs — each
  against this contract verbatim; a deviation is a contract
  change.
- The thread machinery stays the ONLY discussion surface: the
  lifecycle adds records over the existing verbs, never a new
  discussion capability (the ADR-016 boundary, applied to the
  governance records).
- The `.4` publication lane's step-1 ("verify decision,
  approvals, authority proof, and immutable inputs") consumes
  exactly these rows.

answers:

- **Separate records are the lifecycle's honesty.** The
  §15.6 list ("discussion closure, decision, approval,
  publication, and deployment are separate records") is the
  audit's backbone: a folded record could silently conflate
  what was discussed with what was decided — the same
  dishonesty the close family rule (`.2.4.1`) forbids in the
  threads.
- **The snapshot is the decision's frozen ground.** §4.5's
  "at action time" is structural: the electorate snapshot
  rides the decision row, so a later membership change cannot
  rewrite history — the immutable-event doctrine (Phase 1)
  applied to the governance records.
- **The proof is a re-check, not a memory.** The approval
  boundary re-evaluates the grant instead of trusting the
  proposal's record: the authority is a live fact at the
  approval time, the same way the thread's verbs re-run the
  authorization on every step (ADR-016).
