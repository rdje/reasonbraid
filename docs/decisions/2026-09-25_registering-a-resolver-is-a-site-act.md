---
answers:
  - Who may register a resolver, and why is it not a tenant administrator?
  - What does POST /v1/resolvers take and answer since SIGNOFF-REPAIR.7.1.3.1?
---
# Registering a resolver is a site act

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.7.1.3.1`
- **Date:** 2026-09-25
- **Work unit:** `REASONBRAID-REPAIR-0496`
- **Source:** the triage `docs/decisions/2026-09-25_the-corrective-tree-ends-at-a-bug-bar.md` classed the finding blocking (class 1, cross-tenant); the director delegated the corrective decisions (2026-09-25).

## The fact / decision

`POST /v1/resolvers` is the `resolver_register` site act (`migrations/0110`): only a principal holding a current site grant for that action registers a resolver, the act is audited, and a tenant administrator's own grant no longer suffices. The body is `{"advertise": {...}, "reason": "..."}`. An id that already exists is the act's domain refusal, `400` with an `audit_id`, behind the gate; the insert inside the act is insert-only.

## Why

- **The table is site configuration.** `resolver_capabilities` has no tenant column and every tenant's resolution ranks every row. A resolver describes what this server can acquire through. Before `SIGNOFF-REPAIR.7.1.3` one tenant's fast-advertised row silenced every tenant's acquisition; since then such a row is skipped and named, so a tenant's row could never act, only appear in other tenants' answers.
- **A tenant-scoped row was the rejected alternative.** A per-tenant resolver the server cannot execute buys the tenant nothing, and adding a tenant column would have made the registry a second meaning of "resolver" beside the site's own packs.
- **It is the template every other site-wide registry took**: the retention sweep, the workflow registry (`workflow_register`), the governance library (`policy_register`, `docs/decisions/2026-09-19_the-policy-registry-is-a-shared-control-surface.md`), the evaluation harness and the charter registry. The existing-id refusal follows the governance library: a question about the database is answered behind the gate, so a caller without authority learns nothing, and `docs/decisions/2026-09-22_a-domain-refusal-is-not-an-authority-denial.md` makes it `400`, not `403`.
- **Two wire changes, both deliberate.** The body gained the reason every site act requires, nested because `ResolverAdvertise` refuses unknown fields and serde cannot combine `flatten` with `deny_unknown_fields`. The existing-id answer moved from `409 invalid_transition` (naming the id, answered before any authority check) to the audited `400`, whose audit record names the id.

## How to apply

- Registering a resolver needs a site grant carrying `resolver_register`, issued through `rb-site` like any other site capability.
- Replacing an advertisement is still not this verb's; the product's boot-time sync (`resolvers::register`) is the only replacing path.
- A new site-wide registry takes the same shape; a new action ships with its migration (the vocabulary control in `tests/site_authority.rs` fails otherwise).
