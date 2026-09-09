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

The original exception covered seven existing GET routes: admin nodes/presence,
grants, boundaries, incarnations, runs, breakers and usage; their response shapes
remain unchanged. Child `.3.3.3.2.2.3` adds one explicit eighth purpose: exact
tenant-scoped authorization-record lookup, with the same eligibility and admission
audit. Thread inspection/audit, cross-domain receipts, process metrics and site
registries use other gates; this decision gives them no new authority.

The original eligibility child used a read-only transaction and retained the
historical absence of inspection records. That implementation is superseded by
`.3.3.3.2.2.2`: the same eligibility rules now commit explicit inspection-purpose
evidence and return a receipt before fetching response data. An allowed tenant_admin
action alone cannot identify the exception; the evaluation field records its
purpose and actual parent status. The admission transaction still does not share
a response-query snapshot or serialize revocation with delivery. Transaction/effect
ordering remains `.3.3.4`; an admission receipt is not a delivery guarantee.

The owning leaf records the matched baseline, corrected controls and commit.
