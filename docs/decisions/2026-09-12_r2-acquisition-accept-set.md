---
answers:
  - Why is a document refused when it is served under the very media type its resolver advertises?
  - What decides whether the R0 acquisition leg accepts a response body?
  - Should a capability pack's advertised formats be narrowed to what its acquisition leg accepts?
---
# The R2 pack's acquisition leg must accept the formats the ranked pack advertises

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.7.3.3.5.1`; implemented by `.7.3.3.5.2`
- **Date:** 2026-09-12

## Context

The R2 resolution arm (`crates/reasonbraid-server/src/api.rs:2012`) acquires its
bytes through the R0 fetcher — one call, `state.fetcher.fetch`, confirmed by
`git grep -n "R2_RESOLVER_ID" -- 'crates/**/*.rs'` returning three hits: the
constant, that arm, and the `resolver_id` written into the snapshot.

`resolvers::resolve` ranks the R2 pack on the `media_types` its registry row
advertises. Migration `0027_r2_extract_worker_entry.sql` advertises five:
`application/pdf`, `application/zip`, `application/x-tar`,
`application/atom+xml`, `application/rss+xml`.

`.7.3.3.4.1` measured the first live instance of the mismatch: the identical
Atom document succeeds served as `text/xml` and is refused `media_type_refused`
served as `application/atom+xml`.

## The census (`.7.3.3.5.1`)

Two mechanical controls in `fetcher.rs`, over `sniff_kind` — the function that
decides what the leg accepts. They PASS on unchanged production, because a
census is not a regression test, and they were falsified by adding
`application/pdf` to the accept set, which fails the first one.

**Declared content type.** All five advertised types are refused. The complete
accepted set is `text/html`, `application/xhtml+xml`, and any `text/*` — three
arms, enumerated in both directions, so a sixth arm added later fails the census
rather than passing unnoticed.

**No content type.** The verdict is a property of the BYTES, not of the format:
an all-printable body is accepted whatever format it belongs to, and the same
body with one non-text byte is refused. ZIP and tar cannot reach that branch at
all — a ZIP local file header is `PK\x03\x04` plus nine little-endian integer
fields, and a tar header is a 512-byte block with a NUL-padded 100-byte name —
so the refusal there is structural rather than incidental.

## What the census changed about the finding

The first statement of this finding — "the R2 pack advertises five formats its
acquisition leg refuses" — is true but the wrong SHAPE, and a repair derived
from it would have been wrong. The leg's rule is not about formats at all. It is
about transport-declared types and, failing that, about whether the bytes are
printable.

That is why the census came before the repair, and why `.7.3.3.5` was decomposed
rather than implemented directly.

## Decision

**Admit the ranked resolver's advertised media types at the acquisition leg.**
The R2 arm acquires for a pack whose whole purpose is non-text documents in a
sandboxed worker (`ROADMAP.md` §12.3, pack R2: "PDF and selected open formats in
sandboxed workers", isolation `process`). Inheriting R0's text-only accept set is
the implementation reusing one fetcher, not the architecture's intent.

### The rejected option, and why

**Narrowing the registry row's `media_types` cannot express the truth.** There is
no subset of the five that the leg accepts: a feed served as `application/atom+xml`
is refused exactly like a PDF. Narrowing to the types the leg DOES accept would
advertise `text/*`, which is the R0 pack's own row and would make the R2 pack
rank for documents that need no extraction. Narrowing to nothing deletes the
pack. The advertisement is not the thing that is wrong.

### What the repair may not do

- It may not widen the DESTINATION policy, the scheme list, or any SSRF control.
  Those are separate gates and this decision touches none of them.
- It may not accept a type the ranked pack does not advertise. The admitted set
  is the ranked row's own `media_types`, read at resolution time — not a constant
  in the fetcher, and not the union of every pack.
- It may not relax R0's own acquisitions. A reference that ranks the R0 pack
  keeps the text/HTML accept set exactly as shipped.
- The byte ceiling, the decompression-ratio brake, the redirect policy and the
  time ceiling are unchanged: this decision is about which declared type is
  admitted, and about nothing else.

### The risk this accepts, stated plainly

Bytes that were previously never fetched will now be fetched, written to an owned
private input, and read by the extraction worker. That is the R2 pipeline working
as designed — the worker is the quarantine boundary (`process` isolation, one
worker per extraction, the bounded limits, the named refusals) — but it is a real
increase in what reaches a parser. `.7.3.4` still owns pipe bounds, descendant
containment and aggregate retained storage, and none of those becomes less
necessary because of this change.

## Consequences

- `.7.3.3.5.2` implements it; its acceptance is the live R2 control acquiring a
  document served under an advertised type, with no previously refused
  destination or scheme becoming reachable.
- The census controls stay as the standing enumeration: a change to the accept
  set that does not revisit them fails.
- `docs/book/src/deployment.md` and `qualification-review.md` carry the finding
  and must carry its resolution.

## Revisit trigger

A new capability pack whose acquisition leg is also the R0 fetcher, or any
proposal to widen `sniff_kind`'s unconditional accept set rather than the
per-resolver one.
