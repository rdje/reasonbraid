---
answers:
  - How does a completed CLI writer release a lock retained by a child?
  - Can an old child exit release the next writer's exclusion?
  - Does explicit lock release guarantee immediate recovery after process death?
---
# Own flock release independently of descriptor close

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.11.2`; REPAIR-0056.
- Evidence: docs/tasks/artifacts/signoff_review/state-writer-lock-release.md.
- Prior diagnosis: docs/decisions/2026-09-10_state-writer-lock-lifetime.md.

Keep the existing nonblocking flock protocol and install an explicit-release
StateLock immediately after successful acquisition. Its lifetime includes later
validation/synchronization failures and every intermediate publication. Drop
unlocks the shared open-file description before closing its own File; retry
interrupted unlocks and retain close fallback for unexpected OS refusal. A plain
Publication destructor alone would miss failures before Publication construction.

The durable five-path regression retains a real duplicate in a bounded child,
checks exclusion while active, and acquires a successor before that child exits.
Its exit must not release the successor's separate lock. All five paths fail on
unchanged production and pass after correction. Independent raw-fork public-API
success/error/cancellation controls agree. Existing test-only successful probes
also need explicit release so they cannot recreate the same lifetime defect.

This repairs normal completion, errors, cancellation and unwinding. Destructors
do not run after SIGKILL/crash. Surviving inherited descriptors after abrupt owner
death retain concrete ownership under the broader CLI interruption/restart leaf;
no immediate process-loss guarantee follows. Keep original checkpoint attribution
scoped to its missing holder observation and retain its failed result unchanged.
