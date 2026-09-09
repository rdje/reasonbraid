---
answers:
  - What qualifies deletion of an older Rust incremental session?
  - How is the compiler cache locking protocol verified on macOS?
  - What does the periodic artifact cleanup prove and retain?
---
# Qualify compiler cache retirement before deletion

- Owner: `SIGNOFF-REPAIR.11.4.3.1.6`; REPAIR-0041.
- Evidence: docs/tasks/artifacts/signoff_review/compiler-artifact-disposition.md.

Use the exact installed compiler's persistence contract. Finalized sessions are
immutable cache snapshots; preserve all newest timestamp ties and exclude young,
working, partial, unknown and evidence-dependent data. For this operation, also
retain any session containing hard links. Delete a complete obsolete session only
from a frozen root-relative identity manifest after an inactivity census and an
exclusive lock interoperable with the compiler. Age is a selection condition,
not proof of disposability. This does not authorize shared-cache deletion.

On the pinned macOS rustc, the protocol is POSIX fcntl whole-file locking. Native
controls hold both shared and exclusive locks across later real compilations,
prove protected content is unchanged, then release and observe compiler retirement
and successful executable output. A paused linker does not imply an active cache
lock: the observed session was already finalized. Preserve that failed diagnostic
premise and its corrected phase observation.

Anchor cleanup to no-follow directory descriptors and original device/inode/owner
identities. Verify the full session before its first unlink and each file again
before removal. Keep the same lock descriptor open throughout; closing a second
same-process descriptor can release POSIX locks. Retain ambiguous data and refuse
changed identities. Keep a per-session synced receipt and verify exact residue,
unchanged retained source/evidence and a successful affected workflow afterward.

This checkpoint removes 645 obsolete sessions / 1,984 files / 5,116,558,334 logical
bytes; all remaining cache file metadata and 3,545 inspected source/diagnostic
hashes are preserved. Zero-byte locks remain for the compiler's own collection.
Report logical bytes separately from physical allocation, especially with cloning
filesystems. Requalify the protocol before extending this operation to another
compiler/platform. This diagnostic procedure is not a new automatic cleanup API.
