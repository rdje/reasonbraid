---
answers:
  - Which authority target does participant removal require?
  - Does tenant-target authorization let removal cross tenant boundaries?
  - Does correcting participant removal also qualify CLI delegation or revocation ordering?
---
# Authorize participant removal against its administrative tenant

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.8`; REPAIR-0050.
- Evidence: docs/tasks/artifacts/signoff_review/participant-removal-authority.md.

Participant removal retains its existing TenantAdmin action. That action requires
a Tenant target and tenant-wide grant coverage; changing the caller's target is
the bounded correction. Do not weaken the evaluator or invent an implicit
administrative permission for a thread-only grant.

The domain executor still selects and locks the aggregate by tenant and thread ID.
Successful removal records a tenant-target allowance and a thread removal event.
Authority refusal preserves domain effects while recording the denial; a domain
lookup refusal rolls the provisional admission back. The existing idempotency
contract preserves committed denials instead of reinterpreting them after a fix
or authority change; fresh intent uses a new key.

The historical source audit binds the evaluator restriction to d7406e05 and the
older caller mapping/target to d7b76733/35f395d9. Original focused evaluator and
command tests did not cover this invitation lifecycle, so retain their scoped
results and annotate the later compatibility repair in the original task.

CLI delegated existing-thread verbs currently construct thread-only scope. Its
removal-specific companion correction and real CLI qualification are .2.10; the
server's direct-request fix does not claim that caller has migrated. Broader
consent/depth and authority/revocation ordering remain .3.4/.3.3.4.4. The development
principal-header trust model and the production evaluator are unchanged.
