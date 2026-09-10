---
answers:
  - Which node-fixture suites now use checked cleanup?
  - Does declaring resource cascade children change production referential actions?
  - Which fixture cleanup repairs remain after the fourteen node plans?
---
# Apply the checked cleanup contract without changing feature assertions

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.7.2`; REPAIR-0049.
- Evidence: docs/tasks/artifacts/signoff_review/node-fixture-cleanup.md.
- Contract: docs/decisions/2026-09-10_checked-fixture-cleanup.md.

The first adoption group comprises command_api, escalation, evaluation,
identity_store, invitations, node_channel, node_enrollment, node_inbox,
node_replacement, node_work, policy, profiles, routing and CLI cli_end_to_end.
Add the missing MCP-listener dependency to each plan and the spend breaker to the
CLI plan. Declare assessments and derivations before snapshots before resource
references: production already cascades those rows, and explicit fixture scope
must expose that effect without changing the schema.

Preserve original table order, the identity fixture's unrelated deployment CA,
routing's seeded constants and every feature assertion. The qualified shared
checker provides continuing catalog-based refusal even when omitted children
would be empty. Real predecessor/consumer executions complement that safeguard;
a fresh isolated pass alone is insufficient evidence for this failure class.

Six partial fixture plans still need transitive cleanup closure under `.2.7.3`.
The remaining explicit-plan adoption and complete scoped census are `.2.7.4`.
Do not infer a complete checkpoint, MCP transport qualification or project-wide
zero-defect status from this bounded fixture migration.

Actual adoption qualification executes every cleanup plan. Twelve complete suites
pass; two assertion failures reproduce with exact pre-edit fixtures and unchanged
production code: participant removal uses an incompatible authority target (.2.8),
and the retention test supplies a calendar cutoff before snapshot creation (.2.9).
Those fixes are immediate prerequisites after this fixture commit. Preserve their
failed databases and report the 135-pass/two-failure affected census honestly.
