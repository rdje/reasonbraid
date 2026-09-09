---
answers:
  - How do browser tests bound worker groups, output and origin lifetime?
  - Can EPERM during group teardown be accepted as process absence?
  - Does a passing render prove the production browser worker completed cleanup?
---
# Verify browser tests through owned process and origin lifetimes

- Owner: `SIGNOFF-REPAIR.11.4.3.1.5.1`; REPAIR-0038.
- Evidence: docs/tasks/artifacts/signoff_review/browser-test-lifetimes.md.

Use a private on-volume fixture for each command, a separate worker process group,
a whole I/O/exit deadline and bounded stdout/stderr. Close the actual input pipe
for EOF. Consume origin graceful shutdown and verify group absence before deleting
successful owned data. Retain failed or unconfirmed fixtures. Budget admission
needs no installed browser; absence of Chrome leaves rendering explicitly unqualified.

The native Darwin control shows an owned unreaped child's group can return EPERM
until reaping, after which it returns ESRCH. The original real-render failure's
exact member state was not captured. Allow a bounded wait for an uncertain
observation to resolve; neither observation nor signal-request EPERM is absence.
Persistent denial remains an error with retained evidence. Transient/persistent
controls preserve this distinction; do not weaken the cleanup postcondition or
change OS permissions to make the test pass.

Eight final controls, strict focused lint and independent process checks pass.
The real render still leaves a group observable after worker exit, prompting a
test-side stop request. Record requested assistance explicitly; a passing harness
does not establish production-owned shutdown, indefinite leakage or signal causality.
Production profile/process/handler/network-task lifetime remains .5.2, followed by
combined qualification .5.3. Package versions and production worker source remain
unchanged; full local/remote gates remain the later checkpoint.
