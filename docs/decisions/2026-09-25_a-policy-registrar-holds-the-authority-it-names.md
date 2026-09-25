---
answers:
  - Must the owning authority named at policy registration be a grant the registrar holds?
  - How is a policy owned by someone other than the site operator registered?
  - Why is a ghost grant refused with the same text as another principal's grant?
---
# A policy registrar holds the authority it names

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.9.1.2`
- **Date:** 2026-09-25
- **Work unit:** `REASONBRAID-REPAIR-0503`
- **Answers by name:** `docs/decisions/2026-09-19_the-policy-library-is-shared-the-lifecycle-is-its-tenants.md` (DOC-0071), which left *the registrar-holds-the-grant question* to `SIGNOFF-REPAIR.9.1` without deciding it.

## The fact / decision

Registering a policy version names an `owning_authority`: the grant the publication verbs then treat as the policy's owner. That grant must be **live, covering `policy_version_register`, and held by the registrar**. The check is `authority::grant_held_by`, the same predicate every other place that cites an authority already uses (`SIGNOFF-REPAIR.9.3.1`).

Until this decision, `policy::register` asked whether the grant was live and covering, and never whose it was. So a principal holding the `policy_register` site capability could attach any other principal's covering grant, in any tenant, to a document that principal never wrote.

## Why

- **Two different permissions.** `policy_register` lets a principal write the site's library. It is not permission to speak with someone else's authority. The owner of a policy decides whether it is put into force, and a designation the owner never made gives them a policy they did not write.
- **One rule for citing an authority.** Every lifecycle verb that cites an authority already requires the caller to hold it, because a grant id is derivable from a principal id (`grt_<principal>` in the dev enrolment). Registration was the last site that accepted a citation.
- **The legitimate case stays open.** A policy owned by someone other than the site operator is registered by its owner, once the owner holds `policy_register`. ⚠️ Delegation is NOT a path today: nothing issues a grant from another grant (`docs/book/src/authority.md`, *What `delegable` and `max_delegation_depth` do not do*). When grant chains exist, a delegated grant covering `policy_version_register` would let an owner authorize a registrar and record the chain.

## Rejected

- **Accept an operator's attribution as an audited site act.** The audit record would show who attributed the document, but the library itself would still present it under the other principal's authority, and every reader of the library sees the attribution, not the audit.
- **Separate refusals for a ghost grant and for another principal's grant.** A registrar would learn which grant ids are live without holding them. One refusal text covers both: *the named owning authority is not a live grant the caller holds that covers policy_version_register*.

## Consequences

- The refusal stays a domain refusal: `400` with an `audit_id`, because the caller passed the site gate (`SIGNOFF-REPAIR.16`). The audited reason text changes from *not an active, unexpired grant* to the one above.
- A registrar can no longer name a grant whose subject is someone else, a role included: `grant_held_by` compares the grant's subject with the caller. The publication verbs make the same comparison, so a policy owned by a role could only ever be put into force by that role's own principal. The registration now says so at the start rather than at publication.
- Existing rows are not re-judged. Nothing re-attributes a stored policy, and every fixture in the suite already registers under its registrar's own grant.
