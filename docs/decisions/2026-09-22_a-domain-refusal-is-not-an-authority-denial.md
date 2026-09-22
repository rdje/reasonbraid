---
answers:
  - Should a site act's domain refusal return 403 or 400?
  - Does the existence-oracle argument justify a 403, or only an ordering?
  - Why did an authorized caller get told they needed a grant they held?
  - Which site refusals mean the caller lacks authority, and how many are there?
  - Why is this not a loosening of the site gate?
  - What does the audit record for a domain refusal versus an authority denial?
---
# A domain refusal is not an authority denial, and rendering both as 403 told authorized callers a falsehood

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.16`
- **Date:** 2026-09-22
- **Supersedes the RENDERING half of:** `SIGNOFF-REPAIR.6.1.5.4` (the policy registry's 400 → 403)
  and the same choice in `SIGNOFF-REPAIR.8.2.5.3` (the evaluation harness), made hours earlier
- **Leaves untouched:** both leaves' ORDERING decision, which is correct and is why this is safe

## The claim being corrected

`.6.1.5.4` moved the policy registry's domain refusals from 400 to 403, reasoning: *whether a
coordinate is taken is a question about the DATABASE, so it is answered INSIDE the site gate —
answering it before the gate would hand a caller with no site authority an existence oracle over
a registry it may not write.* `.8.2.5.3` applied the same reasoning to the evaluation harness.

⛔ **The reasoning is correct about ORDER and does not reach RENDERING, and the two were
conflated.** The oracle is closed by checking authority FIRST. A caller who has passed that check
holds the grant and is entitled to the answer, so returning it as 400 discloses nothing. Nothing
in the oracle argument selects a status code.

## What the conflation actually shipped

🔴 **The response body contradicted itself.** `site_receipt_response` rendered every `Refused`
with `message: "a current site grant for this action and its actual boundary are required"` — a
sentence that is FALSE of a caller who holds the grant and named a corpus that does not exist. An
operator debugging CI saw 403 and checked their grants; the defect was a typo.

🔴 **It polluted a security metric.** Every domain refusal incremented `authorization_denials`,
so input errors were counted as authorization failures.

🔴 **And it is the same defect class this repository keeps finding, for the fourth time in one
session.** `site_authority::Error::Refused` carried BOTH meanings in one variant — as
`EvaluationError::Duplicate` meant both *taken* and *does not exist* (`SIGNOFF-REPAIR.8.2.5.3`),
and as an `Err(_)` from an INSERT meant both *duplicate* and *store fault* (`.8.2.2`). One name,
two meanings, and the boundary renders whichever it was told.

⭐ **The correct principle was already written down — at exactly one call site.**
`site_registry_response` hand-matched the single reason `undeclared_region` to render it as a bad
request, with the comment *a domain refusal, not an authority one: the caller held the grant and
asked for something the registry cannot express.* ⛔ That one-off is why the principle reached
**one of sixteen** domain refusals. A rule implemented as a remembered special case holds exactly
where it was remembered.

## Decision

**Split the type, so the compiler reaches every site.**

- `Error::Denied { reason, audit_id }` — the caller does NOT hold the authority. Rendered **403**,
  and the only case counted in `authorization_denials`.
- `Error::Refused { reason, audit_id }` — the caller HELD the authority and the act was refused on
  its own terms. Rendered **400**, with its own reason as the code and the `audit_id` beside it.

⭐ **The line is checkable rather than a judgement call: exactly TWO reasons in the whole site
layer mean the caller lacks authority** — `site_authority_required` and `operator_required`.
Every other reason is a domain refusal. The operator CLI follows the same split: exit `3` stays
the authority exit, and a domain refusal exits `2` beside `invalid_input`.

⚠️ **The AUDIT is unchanged.** Both are recorded `denied`, because the act did not take effect
either way. What changed is only what the CALLER is told, and the `audit_id` still resolves the
full reason in both cases.

## Consequences

- ⛔ **This is not a loosening of the site gate.** Authority is still evaluated first, and an
  unauthorized caller still receives 403 having learned nothing about the registry's contents.
  The existence answer is still reached only after the gate passes.
- ✅ **The one-off special case is deleted**, subsumed by the type. A seventeenth domain refusal
  now renders correctly without anyone remembering to add it.
- ⚠️ **A wire change on two shipped surfaces.** The policy registry's duplicate and ghost-authority
  refusals return to 400, as they were before `.6.1.5.4`; the evaluation harness's seven return to
  400, hours after being changed. Both are documented in the book.
- ⭐ **Every test call site now STATES which class it expects**, through two helpers rather than
  one. A helper matching both is what let the distinction go unnoticed.

## What would make this wrong

- ⛔ If a caller could reach a domain refusal WITHOUT passing the authority check, the 400 would
  become an oracle and `.6.1.5.4` would be right. It cannot: `authorized()` returns `Denied`
  before the effect closure runs, and the split makes that structural rather than incidental.
- ⛔ If a third class appeared — an act refused for a reason that is neither the caller's authority
  nor the request's own terms — the two variants would be as wrong as one was. None exists today.

## Alternatives considered

1. **Leave it at 403 for consistency with the four existing site surfaces.** Rejected: consistency
   with a false message is not a virtue, and the message was the strongest evidence the class was
   wrong.
2. **Keep the string match and add fifteen more.** Rejected: that is the mechanism that failed.
3. **409 for a taken coordinate.** Rejected here, not on merit: it would introduce a status this
   API does not use anywhere, which is a separate wire decision.
