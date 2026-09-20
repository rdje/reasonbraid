---
answers:
  - Does a browser render write an EvidenceSnapshot, and what are its bytes?
  - Why is the snapshot the document rather than the rendered text?
  - What happens when the render worker's declared digest is not its document's?
  - Why is the `document` field required rather than defaulted?
---
# A render is evidence, and its worker must describe it

- **Type:** decision
- **Status:** accepted; implemented
- **Owner:** `SIGNOFF-REPAIR.11.24.1.3.1`
- **Date:** 2026-09-20
- **Raised by:** `SIGNOFF-REPAIR.11.24.1.3`'s call-site census, which found the
  R3 branch persisting nothing at all rather than merely skipping an edge
- **Related:** `docs/decisions/2026-09-20_two-packs-produce-evidence-the-store-cannot-hold.md`

## What was missing

🔴 The R3 branch built a `BrowserReceipt`, returned it, and wrote **no**
`EvidenceSnapshot` and **no** `Derivation`. §12.6 opens on exactly this
artefact — *a live Web page or branch can change. Deliberation evidence
therefore points to an immutable `EvidenceSnapshot`* — and the render's chunks
are that section's *derived text/chunk digests and parent links* verbatim. R0,
R5 and R2 all persisted; R1 and R3 did not.

⛔ The blocker was not effort. `evidence_snapshots.raw_digest` is a foreign key
into `snapshot_objects (bytes BYTEA NOT NULL)`, and the worker sent a digest
and no bytes — and `SIGNOFF-REPAIR.11.24.1.3.1.1` then found that the digest
was of the derived chunk rather than of any page, so there was no parent
artefact even in principle. That leaf made the parent exist; this one stores
it.

## The snapshot is the document

✅ The snapshot's bytes are the **serialized page**, and its `raw_digest` is
that document's. The rendered text is a **derivation** of it, one edge per
chunk, parented on the snapshot the render just wrote and cited — so the
resolving tenant is a citer by construction.

⭐ The control asserts the inequality as well as the equality: the snapshot's
digest must **not** be the chunk's. A handler that snapshotted the derived text
would satisfy *a snapshot exists* and fail that arm — which is the arrangement
the pack actually had.

## The worker must describe its own document

⛔ **A render whose declared `parent_digest` is not the digest of the document
it sent is refused**, with the kind `render_source_mismatch`, before any
snapshot, derivation or receipt is written.

R2 already refuses a receipt whose `parent_digest` is not the digest of the
bytes **the request supplied**. Here the worker supplies the bytes itself, so
the comparison is against what it sent — but the rule is the same one: a
process that mis-describes the artefact it is reporting is malfunctioning, and
§12.6's evidence must not be built on it.

⚠️ **Silently re-deriving was the alternative, and it was refused.** The server
could have ignored the worker's claim and addressed the snapshot by its own
measurement. That stores the right bytes and ships an inconsistency: the
receipt would carry the worker's `parent_digest` and the snapshot a different
one, for the same render. A refusal keeps them equal by construction.

## `document` is required, not defaulted

⚠️ The neighbouring `refused_requests` field carries `#[serde(default)]` so
that a worker built before it existed still parses — and that is right, because
an older worker that performed no refusals genuinely has none to report.
`document` takes the opposite choice deliberately. An older worker that sends
no document has not rendered nothing; it has rendered something this server
cannot record, and defaulting to an empty string would store an empty snapshot
and call it the page. The decode failure is the honest outcome, and the two
binaries ship together.

## What is not claimed

⚠️ **Nothing about R1.** The git pack still persists no snapshot, for the
storage reason `.11.24.1.3` recorded; `.11.24.1.3.2` owns it.

⚠️ **No new limit.** The worker's 4 MiB output ceiling, which now bounds the
document and the text together, was decided at `.11.24.1.3.1.1`.

## Verification

`profiles` **63 passed, 0 failed** (62 before), including both arms of
`the_r3_render_persists_the_document_and_one_edge_per_chunk`: an honest stub
worker whose document becomes the snapshot and whose chunk becomes an edge, and
a dishonest one whose declared digest disagrees with its document and which
persists nothing, asserted by whole-table counts before and after.

⭐ **Falsified four ways, and each verdict names the assertion that caught it**
rather than a count — `api.rs` restored byte-identical (SHA-256) after every
mutation:

| Mutation | Verdict | Caught by |
| --- | --- | --- |
| the mismatch refusal removed, the measurement kept | RED | the `render_source_mismatch` arm |
| the snapshot's digest addresses the chunk, its bytes still the document | RED | *the render succeeds* — see below |
| the derivation edges removed | RED | the edges arm |
| the chunk stored as the artefact, bytes **and** digest together | RED | the artefact arm (`raw_digest`, `byte_length`) |

🔎 **The second mutation is caught by something this decision did not write,
and that is worth recording.** It was expected to trip the artefact arm; it
never reaches it, because `snapshots::submit` independently refuses a
`raw_digest` that is not the digest of the bytes handed to it, so the
acquisition fails first with `evidence_unstored`. **The snapshot store verifies
its own addressing** — a property worth knowing, and the reason the fourth
mutation exists: only a snapshot that is internally consistent and simply
describes the wrong artefact can isolate the arm that checks which artefact it
is.

⚪ The control's `assert_ne!` — *the parent is not the derivation* — is
**measured redundant**: the equality assertion above it fires first on every
mutation that reaches either. It is kept, for the same reason
`SIGNOFF-REPAIR.11.24.1.1.2.1.1.1` kept a clause it had measured redundant: a
reader of this control must see the distinction stated, not have to infer it
from two digest constants.
