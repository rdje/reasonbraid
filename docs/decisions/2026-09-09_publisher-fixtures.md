---
answers:
  - Why must publisher tests exclusively create their fixture directories?
  - When can a publisher fixture be removed and when must it be retained?
  - Does the slow native compiler observation invalidate the publisher fixture controls?
---
# Give each publisher test an exclusive directory lifetime

- Owner: `SIGNOFF-REPAIR.11.4.3.1.4`; REPAIR-0037.
- Evidence: docs/tasks/artifacts/signoff_review/publisher-fixtures.md.

A process-local counter does not isolate concurrent test processes. The exact old
helper reproduces deletion of a live owner's witness from a second process. Use
exclusive private creation under validated repository-volume parents; collision
refuses without removing existing entries. Keep the original directory identity.
After all gix handles/readers close, explicit finish verifies that identity, removes
only owned data and checks absence. Drop retains incomplete or failed fixtures.
This is a local test ownership contract, not a hostile same-user race sandbox.

Both original publisher behaviors and three ownership controls pass. Separate
helper processes prove simultaneous private fixtures and independent cleanup;
two concurrent real test executables pass all five controls each with ten distinct
removed directories. Nineteen historical files retain their names, sizes and hashes.
Production publication semantics remain unchanged; broader governance work is open.

A compiler wait is separately measured: the sampled worker is loading a procedural
macro through dyld into __fcntl, while the main thread waits to join it. Compilation,
strict lint and the sampler all finish naturally with their results consumed. This
locates the observed wait without asserting the fcntl operation or its OS cause.
Preserve the evidence under existing .11.2 host investigation; do not weaken tests,
change OS security settings or classify a slow compiler as a product test failure.
