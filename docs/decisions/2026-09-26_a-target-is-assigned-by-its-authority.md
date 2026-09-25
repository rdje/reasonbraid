---
answers:
  - Who may point a deployment target at a publication?
  - Why is it the target's authority, and not only the publication's tenant?
  - Can one target serve several tenants' publications?
---
# A target is assigned by the holder of its owning authority

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.9.3.3.7`
- **Date:** 2026-09-26
- **Work unit:** `REASONBRAID-REPAIR-0528`
- **Cites:** ROADMAP §15.9 step 3 (*offer PR/change or direct apply according to
  target authority*), §23 queue item 021 (*target deployment authority*);
  `docs/adr/021-target-deployment.md`; `REASONBRAID-DOC-0071` (an assignment is
  owned by its publication's tenant); DOC-0029 (a target is site-wide);
  `docs/decisions/2026-09-25_a-target-names-its-reporter.md`

## The fact / decision

1. **Assigning a publication to a target requires HOLDING the target's owning
   authority**: the grant the target was registered under must be live, held by
   the caller, and cover `deployment_target_register` (`authority::grant_held_by`,
   the one predicate every caller-cites-an-authority surface uses). No request
   field changes: the target already records its authority.
2. **The existing binding stays**: the publication must be the caller's tenant's
   and effective, and the desired pair must be its own (`.9.3.3.1`).
3. **A caller that does not hold it is refused `403 unauthorized`**, as the
   publication verbs refuse an authority they do not hold. The check runs after
   the target is found (targets are listed site-wide, so its existence is no
   secret) and before any publication is looked up.

## Why

- **The assignment is an act ON the target.** It sets what the target should run.
  §15.9 puts deployment "according to target authority", and the module already
  records that authority at registration and checks it there (`.9.3.1`); the verb
  that uses the target never asked.
- **The publication's tenant is not enough.** Targets are site-wide, so the tenant
  check alone let any enrolled principal point ANY target, including another
  tenant's, at its own publication. Measured live before the repair: tenant M
  assigned its publication to tenant A's target, `200`.

## Consequences

- **A target serves the tenant of its authority's holder.** The caller must hold
  the target's grant AND own the publication, and grants are held within one
  tenant, so an assignment only ever joins a target and a publication of the same
  tenant. A host shared by several tenants would need a delegation of the target's
  authority; nothing needs one today, and none is built.
- **Only the holder assigns.** A grant has one subject, so another principal of the
  same tenant is refused even with `tenant_admin`, as it is for the publication
  verbs (`.9.3.4`'s accepted residual: a held grant is the unit of authority).
