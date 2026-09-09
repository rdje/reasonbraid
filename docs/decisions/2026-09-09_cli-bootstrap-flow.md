---
answers:
  - When does CLI bootstrap reuse an existing request instead of creating a tenant?
  - How can a caller recover a completed bootstrap after losing CLI output?
  - Does recovering a local bootstrap receipt contact the server or establish live authority?
---
# Preserve bootstrap intent before dispatch and recover explicit historical output

- Owner: `SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.2`; REPAIR-0031.
- Status: implemented; thirty-three selected controls, final output-window rerun and all-target strict CLI lint pass. All results/shutdown consumed; unique fixtures and owned cluster absent. Book build and twelve rendered markers/link/three JSON examples pass; the escalated handoff census precedes commit.
- Evidence: docs/tasks/artifacts/signoff_review/cli-bootstrap-flow.md.
- Prerequisite: docs/decisions/2026-09-09_cli-bootstrap-state.md.

The CLI's human enrollment without a tenant persists a canonical request key and
canonical configured endpoint before HTTP while retaining its state writer lock.
An unresolved matching endpoint/name reuses the saved request, including original
ignored human actions. Another intent refuses before dispatch. When no pending
request exists, a normal invocation intentionally creates a fresh request, even
if an earlier completed request has the same name.

Explicit --resume-bootstrap is valid only for human enrollment without a tenant.
It selects the matching pending request, or the most recent completed request
when pending is absent. Missing or conflicting recovery refuses; it never falls
through into new creation. Public run_enroll remains the convenience entrypoint;
a separate deliberate recovery entrypoint carries this intent from clap.

If a matching completed receipt already exists, recovery uses it locally without
another HTTP request. This includes a completed snapshot whose pending cleanup
was interrupted. Repeating a known completed creation solely to recover output
would unnecessarily rely on the configured URL still referring to the same
server database. A cached receipt is historical data, not proof of current
remote existence, authority, or endpoint/database continuity. Pending requests
without a matching completion still require a same-key request to the expected
authoritative store; endpoint authentication remains its existing repair owner.

A fresh HTTP reply must be a complete strict keyed outcome, decoded from original
bytes to preserve duplicate-field rejection, and bound to the saved request and
canonical source identities. Publish the principal plus completion while pending
remains, then clear pending in another synchronized snapshot under the same lock.
Transport, server, malformed-reply and local-publication errors cannot discard
that request or invent a replacement key. A locally restored receipt follows the
same publication rules; later name-map changes do not erase its provenance.

Machine-readable keyed CLI output adds recovery_source with server or
local_receipt. The historical server replayed field remains unchanged when a
receipt is recovered locally. Human output names historical recovery explicitly.
Neither successful file publication nor returning text from a library proves
that a person or consuming process received stdout. The receipt remains available
for explicit recovery after pending cleanup. A real process/output-backpressure
control qualifies that window; no universal automatic-retry or power-cut claim.

HTTP time and size bounds remain the following .2.3 child; full integration
reconciliation and broader restart qualification remain .2.4 and .3.

Completion-capacity preflight is implemented under
SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.3.1: thirty-one selected controls, the final boundary
matrix and strict CLI lint pass; all results consumed and fixtures absent. Before
publishing pending or sending HTTP, the CLI validates that the complete encoded
principal/receipt snapshot fits the 8 MiB limit. A near-limit pending snapshot
alone is insufficient. Refusal preserves the original snapshot and any pending
key without dispatch. The sizing sample stays private memory; only an actual
checked outcome or a saved historical receipt can be published or reported.
This checks the format limit, not physical disk reservation or later write success.
