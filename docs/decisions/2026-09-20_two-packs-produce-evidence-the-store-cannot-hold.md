---
answers:
  - Why do the GIT and BROWSE resolutions record no derivation edge?
  - Should a git acquisition write a derivation edge, or a snapshot?
  - What blocks an EvidenceSnapshot for a rendered page?
  - What does `storage_class` do today?
---
# Two packs produce evidence the store cannot hold

- **Type:** decision
- **Status:** accepted; the dispositions recorded, the implementation decomposed
- **Owner:** `SIGNOFF-REPAIR.11.24.1.3`
- **Date:** 2026-09-20
- **Raised by:** `SIGNOFF-REPAIR.11.24.1`, which found the gap deferred twice —
  `PHASE-4.6.1` handed the GIT/BROWSE auto-submissions to `.6.2`, and
  `PHASE-4.6.2` handed the R3 render's derivations to `.6.4`; neither target
  carries them
- **Related:** `docs/decisions/2026-09-19_an-advertised-line-carries-an-adjudicated-verdict.md`

## The leaf's premise understated the gap

🔴 The finding was opened as *only the EXTRACT resolution records derivations;
GIT and BROWSE do not*. Measured at the call sites rather than from the
sentence, it is larger: **GIT and BROWSE record nothing at all.**

`crates/reasonbraid-server/src/api.rs`'s resolve handler has exactly four
`snapshots::submit` call sites, and the fourth is the explicit
`POST /v1/snapshots` route outside the handler. Per acquiring pack:

| Pack | `EvidenceSnapshot` | Derivation edge |
| --- | --- | --- |
| R0 `r0-https-fetcher` | ✅ written | none owed — the bytes *are* the original |
| R5 `r5-credential-broker` | ✅ written | none owed |
| R2 `r2-extract-worker` | ✅ written | ✅ one per chunk |
| **R1 `r1-git-fetcher`** | 🔴 **none** | 🔴 none |
| **R3 `r3-browser-worker`** | 🔴 **none** | 🔴 none |

RX publishes a capability call and acquires nothing, so it is outside this
question by design.

⛔ **So the missing edges are a symptom.** There is no parent snapshot for an
edge to hang from, and the reason the two packs have no snapshot is the same
for both: **the evidence store can only hold an artefact that is a byte string
this process has in memory.** `evidence_snapshots.raw_digest` is a foreign key
into `snapshot_objects`, whose `bytes` column is `BYTEA NOT NULL`. R0, R5 and
R2 all hold their bytes. R1's product is an on-disk object database addressed
by `odb_path`; R3's worker returns `parent_digest` and never the rendered
bytes.

## §12.6, quoted, per pack

> A live Web page or branch can change. Deliberation evidence therefore points
> to an immutable `EvidenceSnapshot` containing: … raw-byte digest, length,
> media type, storage/retention class; … derived text/chunk digests and parent
> links …
>
> Every transformation is a `Derivation` edge. A quote, summary, OCR result,
> model-generated caption, or repository analysis is not the original source.

✅ **R3 (BROWSE) is owed a snapshot AND edges**, and it is the archetype the
section opens with — a live Web page. Its chunks are *derived text/chunk
digests and parent links* verbatim, and R2 already does exactly this shape one
branch away. ⛔ Blocked on the worker's wire response, which carries the
rendered page's digest and not its bytes. Owned at `.11.24.1.3.1`.

✅ **R1 (GIT) is owed a snapshot and NOT an automatic edge**, which is the
question the leaf flagged as the real one — *R1's product is the repository
itself, which may be a snapshot rather than a derivation*. §12.6 answers it
directly and names the case: *a branch can change* puts the acquired tree in
scope as evidence, and **repository analysis is not the original source** makes
the repository the **parent** and an analysis of it the edge. Nothing in the
resolve path analyses the tree, so there is no transformation and no edge to
write; an edge invented to fill a graph would be an edge with nothing behind
it. ⛔ Blocked on a storage question. Owned at `.11.24.1.3.2`.

## `storage_class`, measured

§12.6 lists a *storage/retention class* among a snapshot's fields, which is the
field a repository-shaped artefact would use. The schema carries it.
`git grep -n storage_class -- crates/*/src` returns **10 occurrences across 2
files**: a struct field, the `INSERT` bind, the read-back for display, and
three call sites that all bind the literal `"standard"`. **No predicate
anywhere branches on it**, and `snapshot_objects` is the only store.

⚪ That is recorded as the evidence this decision rests on and is deliberately
**not** graded a defect. It is a value written and echoed rather than an
advertised behaviour the system fails to perform — the same arrangement
`SIGNOFF-REPAIR.11.24.1.1.2.1.1.1.1` declined to grade for `accepted_at`. What
it establishes is narrower and load-bearing here: there is exactly one storage
class behind the label, and it is inline bytes.

## What is not decided here

⚠️ **No storage design.** Whether a repository snapshot stores a pack file, a
digest with an external reference, or nothing at all is `.11.24.1.3.2`'s
question, and §12.6 explicitly permits *a verifiable external archival
reference* as an alternative to remaining addressable.

⚠️ **No claim that R0, R5 or R2 are complete.** This census asked one question
— which packs persist evidence — and answers only that.

## Verification

No behaviour changed: the two dispositions are recorded as comments at the R1
and R3 branches, which the acceptance required to sit at the pack's own site
rather than only in the tree. Strict `-D warnings` clippy on the server library
rc=0, `cargo fmt --all --check` rc=0, `make book` rc=0, doctrine gate green.
The discriminating evidence is the call-site census, and it was capable of the
other answer: a `snapshots::submit` inside either branch would have closed the
finding outright.
