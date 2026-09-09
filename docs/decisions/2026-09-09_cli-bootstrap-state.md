---
answers:
  - How is a pending bootstrap identity preserved across local publication failures?
  - Why retain a completed request receipt after pending cleanup?
  - What prevents a stale state save from discarding bootstrap recovery metadata?
  - Which local recovery record fields and endpoint bindings are validated?
---
# Keep pending and completed bootstrap identity in versioned state

- Owner: `SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.1`; CLI flow/explicit recovery, deadlines and reconciliation remain .2.2–.2.4.
- Status: schema/publication support qualified by twenty-four selected controls, all-target CLI strict lint and book checks. Results are consumed and unique fixtures absent. The interrupted pre-main launch and unchanged-hash successful retry remain explicit evidence. No integrated CLI key or resume-command claim yet.
- Evidence: docs/tasks/artifacts/signoff_review/bootstrap-state-schema.md.

## Data and compatibility

StateFile version two carries optional-at-the-Rust-level bootstrap metadata with
required nullable pending/completed fields. Version two requires at least one of
them; versions zero/one must omit bootstrap entirely and retain their existing
wire shape. Public data records are BootstrapRecovery, BootstrapRequest,
CompletedBootstrap and BootstrapOutcome. Existing struct-update constructors stay
compatible, while explicit full literals must provide bootstrap: None. An older
qualified client refuses version two instead of ignoring recovery intent.

The request retains req_-prefixed canonical RFC UUIDv7, canonical configured
HTTP(S) base up to 4096 bytes (no credentials/query/fragment), exact name and the
required nullable original actions. Actions stay ignored human input but are
saved for stable resends. The complete required outcome binds request key,
human kind/name, canonical human/tenant IDs and their actual boundary/grant
source strings. Unknown/duplicate fields, omitted nullable fields, non-object
records and malformed/conflicting bindings refuse. Canonical older UUID versions
remain valid source identities. The whole state retains its 8 MiB limit.

A canonical URL identifies configuration, not cryptographic endpoint/database
continuity. Local records are data rather than credentials or authenticated
server receipts. A completed outcome is historical and may survive later local
mapping changes. Retain only the most recent completed request/outcome and one
pending request; there is no unbounded local receipt archive in this design.

## Guarded transitions

Borrowed Writer::persist reuses the qualified Publication while retaining its
lock; consuming publish delegates to it. Ordinary Writer::open refuses a stored
pending request before HTTP, including in this prerequisite commit. The later
keyed coordinator receives the deliberate recovery entrypoint.

Before changing working files, publication loads the actual current state and
validates the exact encoded replacement plus recovery continuity. Existing
metadata cannot disappear. Pending identity cannot change. A prior completed
receipt can change through its matching pending request, and pending removal
requires that completed outcome and its principal mapping in the snapshot.
A valid first snapshot can restore recorded metadata to a legacy/empty store;
this is data restoration, not server authentication. These rules prevent a stale
full-snapshot save from silently discarding unresolved intent.

The next coordinator persists pending before HTTP, publishes the matched
principal/outcome while retaining pending, then clears pending in another
synchronized version-two snapshot. No separate pending pathname is deleted.
If synchronization fails after replacement, callers preserve unconfirmed phase;
borrowed persistence keeps exclusion until the owner is dropped. The next reader
sees a complete earlier/newer snapshot and can retain the original key.

## Explicit recovery after output uncertainty

Clearing pending must not discard the only recoverable request identity. Retain
the completed receipt and add explicit --resume-bootstrap in the next CLI slice.
Matching active pending requests reuse their saved identity; an explicit resume
can also select the retained completed request after cleanup or lost CLI output.
A normal fresh invocation remains distinct when no pending request exists.
A different request/server refuses unresolved work. Do not infer that equal names
mean equal logical operations, or that file publication proves stdout consumption.
Qualify actual process/output interruption separately before claiming this flow.
