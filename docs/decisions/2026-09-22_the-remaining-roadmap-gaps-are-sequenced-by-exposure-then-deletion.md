---
answers:
  - In what order should the remaining roadmap-completeness gaps be worked?
  - Why is a live exposure worked before a cheap decision that could delete work?
  - Why are all the outstanding scope decisions taken before any of the builds they authorize?
  - Which of the five findings is actually blocked, and on what?
  - Can the evaluation harness be authority-gated today, or does it need new grant vocabulary?
  - Why is the ballot not started before the decision-rule contract?
  - What would make this sequence wrong?
---
# The remaining roadmap gaps are sequenced by exposure, then by deletion, then by dependency

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.15`
- **Date:** 2026-09-22
- **Cites:** `docs/decisions/2026-09-19_the-policy-registry-is-a-shared-control-surface.md` (the
  evaluation family's site-wide verdict), `SIGNOFF-REPAIR.9.3.4` (the `GrantAction` extension),
  `docs/CLAIM_VERIFICATION.md` (leg 2 — an earlier ruling wins unless the difference is named)

## Context

Five roadmap commitments were surfaced on 2026-09-22 as absent from the shipped system. The
director's instruction was that all five be task-tree owned, tracked and worked, and that the
SEQUENCE be decided here rather than referred back. The objective the sequence serves is
stated in that instruction: every roadmap goal implemented and working.

⛔ **Two of the five had no executable owner when the audit ran**, which is the thing this
record exists to have fixed rather than noticed. `TOOLBOX.md`'s test — *can someone open that
leaf and finish it?* — fails for a clause routed to a container, and `.8.2` is a container.

## The correction the audit produced, before any sequencing

🔴 **The one finding everyone called blocked is the one that is most ready.** `.8.2` clause 1
and the published book chapter both say *no `GrantAction` and no `TargetSelector` can name a
corpus or a gate, so there is nothing for an authority check to bind to yet*. That is wrong
in two independent ways, and each was a reading rather than a measurement:

- The evaluation family's gate does **not** bind through `GrantAction`. DOC-0029 already ruled
  the seven `evaluation_*` tables **site-wide by design, gated by SITE-OPERATOR grants**, and
  four site-wide surfaces already use that mechanism — `site_authority::workflows::register_profile`,
  `::policies::register_policy`, `::retention::expire_evidence`, `::retention::tombstone_evidence`.
- `SIGNOFF-REPAIR.9.3.4` did not rule the vocabulary closed. It ruled that **`GrantAction`
  EXTENDS**, and extended it five times.

⇒ The finding with the widest exposure is unblocked and has four shipped precedents.

## Decision

Work the five in three waves, ordered by a rule stated here so a later reader can check the
order rather than trust it.

**The ordering rule, in priority order:**

1. **A live exposure on a SHIPPED surface outranks everything.** It is already reachable by a
   caller; every other item is work not yet done.
2. **Then every outstanding SCOPE DECISION, before any build it authorizes.** A decision here
   is hours and can only shrink what follows; a build taken against an undecided contract is
   rework by construction. Within the wave, largest possible deletion first, so the remaining
   plan stabilises soonest.
3. **Then the builds, in dependency order**, cost as the tie-break.

**The resulting sequence:**

| # | wave | leaf | what it is | why here |
| --- | --- | --- | --- | --- |
| 1 | A — exposure | `.8.2.5` | the evaluation family's site-operator gate | the only live exposure: seven tables, every write admitted on bare enrolment, every record readable by any enrolled principal |
| 2 | B — decisions | `.9.3.5` | publication content: Git, object store, both, or deferred | the largest possible deletion — the roadmap names two stores and the answer may be one, or none yet |
| 3 | B — decisions | `.4.6` | observability: which of the sink, dashboards, SLO baselines and game days are owed | four commitments carried as one row; the answer may delete three |
| 4 | B — decisions | `.11.4.7.2.1.2` | the decision-rule and expected-artifact contract | gates item 6, and settles the deliberation surface's largest open question |
| 5 | B — decisions | `.11.4.7.2.1.3` | §13.5's moderator: product or benchmark-only | may delete itself entirely; cheapest, so last in the wave |
| 6 | C — builds | `.8.1.1` | the counted outcome: what a `vote` step records | depends on 4 — a tally against an undecided rule is the rework this sequence avoids |
| 7 | C — builds | whatever waves B keeps | the publication store, the observability items, the moderator | scoped by 2, 3 and 5 |

## Consequences

- ⭐ **After item 5 the remaining build is fully scoped**, and that is a deliberate checkpoint:
  the director sees the true size of what is left before any large build starts.
- ⛔ **No item in wave C may start before its wave-B decision closes.** `.8.1.1`'s own
  acceptance names `.11.4.7.2.1.2` for this reason, so the constraint is enforced by the leaf
  rather than by this record being remembered.
- ⚠️ **The sequence is not a promise about wall-clock.** Item 1 is a repair with four
  precedents; item 7 may be the largest build remaining in the project. Sizing is per leaf.
- ⚠️ **Two of the five are DECISIONS whose honest answer may be "not before G9".** That is a
  legitimate outcome and the leaves say so — but a deferral must name a condition a later pass
  can EVALUATE, never a phase that has already closed, which is the defect `.11.4.7.2.1`
  measured at 27 of 27.

## What would make this wrong

- ⛔ If the evaluation harness turns out to be tenant-facing rather than release engineering,
  item 1's remedy is tenant scoping and not a site-operator gate. The verdict relied on is
  DOC-0029's, cited above; overturning it needs `docs/CLAIM_VERIFICATION.md` leg 2's named
  difference, not a fresh opinion.
- ⛔ If a wave-B decision turns out to depend on a wave-C build, the wave order inverts for
  that pair. None of the four does today: each is a scope question answerable from the roadmap
  and the shipped code.
- ⛔ If a sixth gap is found, it is inserted by the same rule rather than appended.

## Alternatives considered

1. **All decisions first, exposure second.** Rejected: the exposure is reachable now, and no
   wave-B answer changes item 1. Tidiness of planning does not outrank a live surface.
2. **Largest roadmap gap first (the publication store).** Rejected: its scope is undecided, so
   starting it is the rework the rule exists to prevent. Its DECISION is early instead.
3. **Sequence by roadmap phase order.** Rejected: it ignores both exposure and dependency, and
   would start the ballot before the rule that counts it.
