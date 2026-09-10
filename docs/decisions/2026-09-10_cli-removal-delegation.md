---
answers:
  - Why does delegated participant removal request tenant-wide scope?
  - Does correcting removal broaden other CLI thread commands?
  - What proves the CLI removal scope correction and what remains unqualified?
---
# Match CLI delegation to the operation's authorization target

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.10`; REPAIR-0051.
- Evidence: docs/tasks/artifacts/signoff_review/cli-removal-delegation.md.

Participant removal already requires TenantAdmin against a Tenant target. Request
TenantWide scope for this one operation; an existing thread ID in the URL does not
make its authority thread-scoped. Both caller and delegated source still require
their own eligible authority. The flag cannot create or borrow administration,
and domain preparation still binds the thread to that tenant.

Preserve single-thread scope for every other existing-thread verb and the existing
tenant scope for creation. A real CLI control with a one-thread invitation grant
protects this distinction; a deliberate all-tenant-wide mutation is refused by the
same server and must fail that control. Restore and hash-check the selected source
before final qualification.

The historical scope/flags originate in b62f6a88, PHASE-2.1.4.2. Actual rb removal
against the corrected server reproduces the remaining 403 scope failure, while
ordinary delegated invitation passes. Keep that before/after record and annotate
the historical leaf instead of claiming its prior gate covered all administrative
callers. Final actual CLI/audit/domain outcomes belong to the evidence record.

This is a caller mapping correction in the existing development profile. It does
not establish delegated consent/depth, concurrent revocation serialization or
bounded HTTP transport; those retain their existing repair owners. The server
and its evaluator stay unchanged in this leaf.
