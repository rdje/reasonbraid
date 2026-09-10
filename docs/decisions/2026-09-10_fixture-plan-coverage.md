---
answers:
  - What does the completed twenty-five-plan cleanup census establish?
  - Why do broad fixture plans still delete the deployment CA?
  - Does testing the MCP library fixture qualify MCP transport or agents?
---
# Bind fixture coverage to its explicit source population

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.7.4`; REPAIR-0054.
- Evidence: docs/tasks/artifacts/signoff_review/fixture-plan-coverage.md.

The original population is twenty-five literal-array DELETE-loop fixtures, now
all migrated to checked explicit arrays. Independently recover each original
file outside the import/cleanup block, validate every declared scope against the
live FK graph and run the actual suites consecutively. Do not equate a source
pattern census with coverage of all SQL or relationships without foreign keys.
The shared preflight and its existing adversarial tests protect declared plans
against future dependency omissions even when their child tables start empty.

Preserve each fixture's original scope policy. The six partial plans retain their
unrelated deployment CA. The final four broad plans already delete server_ca and
resource_references; retaining their policy means preserving the CA entry and
explicitly declaring the three previously implicit resource-cascade descendants.
RLS still deletes only its five event/outbox tables. The runtime helper never
expands scope or disables constraints.

The MCP crate imports this private support only under cfg(test). The live library
fixture exercises internal tools with the real database/HTTP bootstrap, while
its pure schema tests cover their separate contracts. Neither establishes MCP
wire transport, principal binding at that boundary or autonomous-agent behavior;
those remain separately owned under .6.2/.6.4. Full checkpoint and remote CI
likewise retain their own acceptance requirements.
