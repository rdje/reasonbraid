---
answers:
  - How is a §13.5 moderator seated, and what may it do?
  - Which verb enforces each of the six §13.5 prohibitions?
  - Why is a moderator excluded from the electorate?
  - Which of DOC-0126's findings were wrong, and how were they found?
---
# A moderator is a seat restricted to the moderation vocabulary

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.3.1`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §13.5; ADR-030 (the moderation kinds);
  `docs/decisions/2026-09-22_the-moderator-is-a-role-boundary-not-a-feature-and-five-of-its-six-acts-already-ship.md`
  (DOC-0126, which this record implements and corrects in two places)

## Two corrections to DOC-0126, made before building on it

🔴 **1. The permitted acts were mapped to the wrong verbs.** DOC-0126 mapped *classify* to
`ContributionKind`'s author-declared kinds, *propose round closure* to
`thread.advance_round`, and *draft summaries* to `Summary` + `SynthesisInput`. It did not
notice that **ADR-030 already ships a closed moderation vocabulary that matches the five acts
one-to-one**: `classify`, `request_clarification`, `propose_close`, `identify_unanswered`,
`draft_summary`. These are legal only on a `moderate` step, and they refuse every
capability-shaped field. The conclusion (*a role boundary, not a feature*) holds and is
stronger. The acts do not merely *exist as verbs*; they exist as exactly the vocabulary §13.5
describes.

🔴 **2. The dissent-suppression census was vacuous.** DOC-0126 proved *suppress a visible
dissent is structurally impossible* with
`git grep -niE "DELETE FROM thread_contributions|UPDATE thread_contributions"`. No table
named `thread_contributions` exists (`git grep -c "CREATE TABLE thread_contributions" --
migrations` finds none). Contributions live in `event_log`. The command would therefore return
*no match* whatever the code did. **The corrected census holds:**
`git grep -nE "(UPDATE|DELETE FROM)[[:space:]]+(public\.)?event_log" --
crates/reasonbraid-server/src crates/reasonbraid-core/src` returns no match. ⚠️ The guarantee
comes from the code alone; no database trigger enforces it.

⭐ Both are evidence for `.11.4.7.2.1.5`. They are verdicts from the same adjudication pass,
re-derived here by a different route, and both needed correcting.

## Decision

1. **The seat.** `thread.invite` takes `"seat": "moderator"`. The projection records the
   seated principals. Re-inviting a role as an ordinary participant removes the seat.
2. **The gate.** Before any verb runs, a seated principal passes through
   `threads::moderator_may`. It may answer its own invitation and post a moderation-kind
   contribution. Every other verb is refused with `403`, the same status as
   `NotAParticipant`, which is the thread path's precedent for acting outside one's
   membership. The refusal names the §13.5 clause it enforces. **The gate applies even when
   the principal holds the verb's grant.** The seat removes authority and never adds any.
3. **The six prohibitions, each with what enforces it:**

   | §13.5 prohibition | enforced by |
   | --- | --- |
   | add a vote or approval | the seat refuses `ballot` and `verdict`; the policy approval is grant-gated and bound to the caller (`.9.3.1`) |
   | suppress a visible dissent | no product code updates or deletes `event_log` (the corrected census); moderation kinds *refer* via `ref_event_id` and never rewrite |
   | fabricate evidence or change citations | the seat refuses `evidence_reference` and `assessment`; ADR-030 refuses evidence fields on moderation kinds; snapshots are digest-verified |
   | change electorate, quorum or proposal digest | the seat refuses `invite`, `remove_participant`, `join`, `revise`, `close` and `cancel`; the quorum is derived (`.11.4.7.2.1.2.3`); the proposal digest is server-computed |
   | authorize more spend or tools | the seat refuses `invite` (which dispatches reserved work); spend is grant-bounded |
   | publish or deploy policy | grant-gated; the seat grants nothing |

4. **A moderator is never in the electorate.** It may not cast a ballot. If it were counted,
   its ballot would stay outstanding forever, and every `unanimity` vote would end
   `no_quorum`. This is the *change … quorum* prohibition observed against the real count:
   the live control closes a unanimity thread with a seated moderator, and it ends
   `accepted_unanimously`, which it could not do if the moderator were counted.
5. **Unchanged:** *enforce format/length* stays declined (DOC-0126 decision 3), and the
   moderation kinds stay open to every participant, as ADR-030 set them. The seat restricts
   the moderator. It does not reserve the vocabulary.

## What would make this wrong

- ⛔ If a moderator must be able to advance the round rather than only propose it, the seat is
  too strict. §13.2 says the moderator *proposes scheduling*, so proposing is the act.
- ⛔ If moderation acts must be reserved to moderators, ADR-030's open vocabulary needs
  revisiting. Nothing in §13.5 says a non-moderator may not classify its own message.
