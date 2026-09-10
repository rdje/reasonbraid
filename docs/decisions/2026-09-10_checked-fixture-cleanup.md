---
answers:
  - Why must fixture cleanup validate its entire plan before deletion?
  - How are cascading and cross-schema fixture dependencies handled?
  - Does checked cleanup guarantee atomic rollback on a database error?
---
# Validate declared fixture dependencies before mutation

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.7.1`; REPAIR-0048.
- Evidence: docs/tasks/artifacts/signoff_review/fixture-cleanup-plan-check.md.

Actual MCP-listener, spend-breaker and incarnation residue reproduces failures in
later fixtures. A clean isolated database can hide these missing dependencies.
Validate the complete declared plan against the live FK catalog before any DELETE,
including when child tables are empty. Keep caller adoption separate from the
qualification of the shared test-only primitive.

Require canonical names, supported ordinary public tables and explicit
child-before-parent order. Include existing cascade/null/default effects in the
declared scope instead of implicitly deleting or changing undeclared children.
Cross-schema dependencies cannot be satisfied by an unrelated same-named public
table. Refuse unsupported inheritance/partitioning, views and cycles; PostgreSQL
continues enforcing qualified single-table self-references.

The caller supplies the ownership-verified pool and exclusive fixture use.
Catalog validation and deletion hold one acquired connection; concurrent DDL is
outside this contract. Preflight failures cannot mutate rows. Statement execution
retains its existing non-atomic behavior: a later trigger error may follow earlier
committed deletes. Preserve the original SQL error and that visible partial state;
stop and retain the failed fixture rather than implying rollback or recovery.

Production migrations, referential actions and application paths remain unchanged.
The broader source census is a list of dependency candidates, not a runtime defect
count or a claim that every fixture or implicit application relationship is covered.
