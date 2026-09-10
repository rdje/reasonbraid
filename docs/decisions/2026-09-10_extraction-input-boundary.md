---
answers:
  - Can production extraction input names collide and return another document?
  - What must be proved before the server removes an extraction input?
  - Which extraction guarantees remain outside the input ownership repair?
---
# Bind extraction ownership to source bytes and worker completion

- Owner: `SIGNOFF-REPAIR.7.3.3.1`; REPAIR-0058.
- Evidence: docs/tasks/artifacts/signoff_review/extraction-input-boundary.md.

A controlled native-clock probe embeds the production API's input-name/write span
byte-for-byte and calls the unchanged server extraction module with the preserved
real worker. Thirty-two input owners share six paths; eight worker responses
correctly describe another owner's bytes. This is a reproduced production input
boundary defect. It is not an HTTP/database reproduction or a parser corruption
claim. The API's absent source/response digest comparison makes persistence
misattribution a source-derived consequence requiring integration qualification.

Repair the boundary in explicit stages. First establish whether the direct worker
was never started, has a consumed exit, or has unconfirmed completion. An error or
attempted kill is insufficient evidence that an input reader has stopped. Retain
owned input/evidence on unconfirmed completion and surface that state; never
silently classify it as successful cleanup. Qualify error and timeout paths with
owned controlled workers before changing storage cleanup.

Then create private inputs exclusively from a runtime-discovered repository root,
using checked same-volume directory/file ownership and descriptor-relative
operations where required to avoid path substitution. Candidate names do not prove
ownership. Hold the input through execution, compare the response parent digest
with SHA-256 of the acquired bytes before exposing/persisting derived results,
and clean only an identity-proven input after confirmed completion. Failures must
preserve unrelated data and report a precise refusal; diagnostics use relative
paths. Integrate the actual R2 API and adjacent spawner fixture with focused live
and worker controls before claiming the production repair.

Direct-child completion does not establish descendant containment, bounded pipe
I/O, reply allocation or aggregate storage retention. Those concrete source
boundaries remain `SIGNOFF-REPAIR.7.3.4`; the current process-per-extraction model
alone is not a sandbox proof. No worker or parser behavior is changed in this
diagnostic commit. The next work is `.7.3.3.2`, then `.7.3.3.3`, then the full
checkpoint before public push and actual remote CI.
