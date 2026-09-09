---
answers:
  - What authority remains valid for administrative inspection after a tenant freeze?
  - Can a frozen-read exception ignore the grant's parent, scope or expiration?
  - Does administrative read eligibility imply an audit or a serialized response snapshot?
---
# Frozen administrative reads retain every check except boundary status

- Owner: `SIGNOFF-REPAIR.3.3.3.2.1`.
- Status: implemented and verified; all 40 live authority/command API tests, ten pure evaluator controls and strict focused lint pass. All results consumed and the owned cluster stopped/removed.
- Refines: `docs/decisions/2026-09-07_boundary-revocation-freeze.md`.
- Candidate selection: `docs/decisions/2026-09-09_command-authority-selection.md`.

The approved exception lets an otherwise eligible administrator inspect its own
tenant when the actual enrollment boundary is active, suspended or revoked. It
does not dispense with that parent or its ceilings. A grant must name that parent,
belong to its tenant and to the direct caller, carry tenant_admin, select the whole
tenant, and remain within every parent ceiling and both validity windows.
Validity includes the start and excludes expiration; empty windows never qualify.
Revoking the grant itself removes its inspection authority.

The closed read purpose constructs only a direct TenantAdmin/Tenant context.
Its evaluator refuses delegation, other actions and thread targets before
applying the normal evaluator with only a temporary boundary-status projection.
The original selected boundary retains its actual status. Normal command
evaluation does not use this exception. Candidate selection uses the same
deterministic pages and actual-parent checks as commands, so a newer unusable
grant cannot hide an eligible older one. Malformed authority is a storage failure,
not a fabricated denial or an inspection allowance.

The exception is limited to seven existing GET routes: admin nodes/presence,
grants, boundaries, incarnations, runs, breakers and usage. Their response shapes
remain unchanged. Thread inspection/audit, cross-domain receipts, process metrics
and site registries use other gates; this decision gives them no new authority.

This child replaces the eligibility check in a read-only transaction. It neither
shares a response-query snapshot nor serializes revocation with response delivery.
It also preserves the existing lack of an inspection authorization record. The
following child `.3.3.3.2.2` owns explicit inspection-purpose evidence: an allowed
tenant_admin action alone would misrepresent a read exception as write authority.
Transaction/effect ordering remains `.3.3.4`. These limits prevent treating an
eligibility decision as a complete administrative audit or delivery guarantee.

The owning leaf records the matched baseline, corrected controls and commit.
