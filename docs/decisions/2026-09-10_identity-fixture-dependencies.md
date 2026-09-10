---
answers:
  - Why must identity fixtures remove certificates before their nodes?
  - How can a passing isolated database suite conceal an ordering failure?
  - What does the b0cddfe checkpoint actually qualify?
---
# Qualify fixture cleanup against real predecessor residue

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.6`; REPAIR-0047.
- Evidence: docs/tasks/artifacts/signoff_review/identity-fixture-cleanup.md.

Fixture cleanup is part of a suite's executable contract. `identity_store` passes
on a fresh database but fails after the real node-work suite leaves a certified
node. The live schema requires the certificate child to be removed first. Add
that explicit cleanup entry and retain a regression that first proves the exact
foreign key still rejects parent deletion, then verifies the fixture clears the
hierarchy. Keep production constraints and unrelated deployment CA state intact.

Qualify both fresh execution and a relevant predecessor sequence. A clean-only
pass does not falsify a residue-dependent failure. Compare explicit cleanup lists
with the migrated database's incoming foreign keys, and give every additional
candidate an executable repair owner. The MCP-listener and CLI spend-breaker
dependencies are `.11.4.3.1.2.7`; their source/catalog census is not yet runtime
qualification of every affected fixture.

The b0cddfe checkpoint passed workspace/browser and eight other gates, then failed
on the fourteenth PostgreSQL command. Earlier suites passing, later suites being
unstarted and a safely stopped failed database are distinct facts. Preserve all
three. Native pre-entry waits have separate diagnostics and ownership; neither
those waits nor nominal database skips justify suppressing an assertion failure
or claiming the full checkpoint passed.
