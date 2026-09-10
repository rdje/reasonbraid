---
answers:
  - Why did concurrent extraction tests receive another fixture's content?
  - How are extraction test inputs owned and retained after failure?
  - Does repairing extraction fixtures also repair server acquisition storage?
---
# Exclusive creation establishes input ownership

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.12`; REPAIR-0057.
- Evidence: docs/tasks/artifacts/signoff_review/extraction-fixture-ownership.md.

The original fixture name combined a process ID with only the fractional-second
clock value. std::fs::write accepted and truncated an existing path. An unchanged
extractor executable received another test's nested ZIP. A separate byte-identical
helper control, without a mocked clock, then observed three duplicate names and
wrong-owner payloads among 32 simultaneous writers. Passing isolated reruns and
100 instrumented concurrent runs do not negate those observations. The original
checkpoint's two PDF input paths were not recorded; retain that attribution limit.

Use a private shared test helper with runtime repository-root discovery, checked
same-volume storage parents and exclusive file creation. Process/counter names
only propose candidates; create_new establishes ownership, with bounded retries
for existing candidates. Never truncate or follow an existing file/link. Hold the
input through assertions, check its open-file identity before successful cleanup,
and retain it during panic. These tests do not authorize hostile concurrent
mutation of the repository directory by another same-user process.

Apply the helper to extractor unit and stdio fixtures without altering parser,
refusal or concurrency assertions. Keep failed baseline files and exact source/
binary identities. The production R2 input path has the same source pattern and
requires its own actual-boundary repair under .7.3.3 before the checkpoint resumes.
Git acquisition scratch belongs to .7.2.1; remaining journal/stub fixtures belong
to .11.2.1. No fixture pass closes those production or broader isolation boundaries.
