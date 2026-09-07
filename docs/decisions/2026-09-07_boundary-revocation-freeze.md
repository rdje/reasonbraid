# Boundary revocation freezes writes, never the operator's eyes (`PHASE-2.1.3.2`)

- Date: 2026-09-07 · Leaf: `PHASE-2.1.3.2` · Decision record

## Context

The `.1.3.2` boundary-revocation tests surfaced a real governance-semantics
question: the tenant_admin authorization itself evaluates against the ACTIVE
enrollment boundary, so revoking the boundary refused EVERY subsequent admin
call — including the inspection lists meant to prove the revocation. A revoked
boundary would have been un-observable, which contradicts the leaf's own
acceptance ("revocation is observable through the inspection surfaces").

## Decision

- **The freeze stops writes, never reads.** The admin inspection lists
  (`GET /v1/admin/grants`, `GET /v1/admin/boundaries`) authorize against the
  principal's OWN grant (carries `tenant_admin`, within its validity window)
  WITHOUT the boundary ceiling — `authorize_tenant_admin_read` in
  `api.rs`. The revoke verbs keep the ceiling-checked authorization: after a
  boundary revocation, no further admin WRITE passes (the freeze), while the
  state stays inspectable.
- **A boundary revocation is the tenant freeze.** Every grant under it is
  refused at the next decision (the evaluation's `status = 'active'` filters +
  the missing ceiling) — including the bootstrap human's, so the freeze is
  honest: no actor can keep writing through a revoked ceiling.

## answers:

- **The dev-profile root trust is the bootstrap human's grant**, not the
  boundary row — the read carve-out is that trust, named explicitly rather
  than smuggled through the boundary check.
- **The freeze is reversible only by a future superseding act** (a new
  boundary + grants — Phase 5's correction machinery); until then the tenant
  is read-only, which is the intended semantics of revoking a ceiling.
