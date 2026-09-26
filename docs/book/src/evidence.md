# Evidence snapshots

A deliberation cites evidence, and a live page or a branch can change under it.
So a citation never points at a URL: it points at an **evidence snapshot**, an
immutable record of one acquisition (ROADMAP §12.6). The snapshot records where
the bytes came from, who fetched them and with what, the receipts the transport
produced, and the immutable version of the source where one exists.

## Two storage classes, and why there are two

§12.9 states the durability rule in two alternatives:

> Snapshots used by binding decisions remain addressable for the charter's audit
> period **or** retain a verifiable external archival reference; deletion creates
> a tombstone and reason, not silent disappearance.

The product uses both, and which one an acquisition gets is a property of the
artefact rather than a setting.

| `storage_class` | `raw_digest` | `byte_length` | `external_reference` | Used by |
| --- | --- | --- | --- | --- |
| `standard` | the ADR-011 digest of the stored bytes | the stored bytes' length | absent | R0 fetch, R2 extract, R3 render, R5 |
| `external-reference` | absent | `0` | the verifiable pointer | R1 git |

`raw_digest` being absent is the signal a client branches on. It is not a
degraded snapshot: it says *this store holds no bytes for this row, and here is
what to re-acquire instead*. `byte_length` is `0` for the same reason — it
reports how much this store holds, not how large the artefact is. The
acquisition's own measurements (object count, file count, object-database size,
the manifest) are in `provider_receipt`, where §12.6 puts the provider receipts.

The database enforces the pairing rather than trusting the writer: a row may be
one shape or the other and never a mixture, and an `external-reference` row must
carry the immutable source version its identity rests on.

## Why a git acquisition is stored by reference

A git acquisition's product is an on-disk object database — a pack built by the
**remote**, from the remote's own packing configuration. That pack is not a
property of the evidence, and the product measures this rather than assuming it:
acquiring one immutable commit twice from the same source, across a server-side
`git repack` that turns delta compression off, yields two object databases of
6857 and 10071 bytes with two different digests, for a commit that never changed.

Storing those bytes and keying the snapshot on their digest would therefore file
a **second** evidence snapshot every time an upstream forge ran housekeeping.

The commit id does not move. Git's object model makes it a cryptographic
commitment to the whole tree, so it is both the snapshot's identity and its
verification procedure: re-acquire the remote at that ref and compare the
resolved commit. That is what makes the reference *verifiable* in §12.9's sense
rather than merely a pointer, and it is why a git snapshot records

```json
{
  "kind": "git-commit",
  "remote": "https://git.example.org/repo.git#main",
  "requested_ref": "main",
  "resolved_commit": "8be5445a05aeb912c85c1257cdad3282cf2073b3"
}
```

and nothing about the object database. The acquisition's own odb digest is in
`provider_receipt.git.digest`, labelled there as the record of one acquisition —
a verifier that reached for it as a re-clone check would get a false mismatch.

⛔ **A git acquisition writes no derivation edge.** §12.6 says *a quote, summary,
OCR result, model-generated caption, or repository analysis is not the original
source* — which makes the repository the parent and an analysis of it the edge.
Nothing in the resolve path analyses the acquired tree, so there is no
transformation for an edge to record, and an edge invented to fill the graph
would be an edge with nothing behind it.

## What this changes for a caller

**Resolving a `git` reference now writes evidence.** `POST
/v1/resources/{id}/resolve` files the snapshot before it returns the receipt. If
the store refuses, the acquisition is reported as failed with
`evidence_unstored` and no receipt is produced — a receipt for evidence nobody
kept would be a claim the system cannot support.

**Re-resolving the same commit does not accumulate rows.** A second acquisition
of the same reference at the same commit replays onto the existing snapshot,
records the re-acquisition time (`refreshed_at`), and records the citation. This
class declares no freshness horizon: the commit id is what makes the reference
verifiable. A *different* commit
on the same reference is a new snapshot, because the reference legitimately holds
many versions.

**A pinned reference refuses it, by name.** §12.1's `expected_digest` says
*these exact bytes*, and a snapshot in this class holds none to compare against.
Register the repository as an unpinned reference; its commit id is the pin that
applies to it.

**An excerpt assessment cannot be checked against it.** `POST /v1/assessments`
validates that the excerpt appears in the snapshot's bytes, and there are none
here. The refusal names the storage class rather than reporting the snapshot as
absent — a tenant reading evidence it cited itself is never told that evidence
does not exist.

## Retention and deletion

Retention is unchanged by the storage class. A snapshot expires by its
`retention_class` (the `audit` class never expires, so a binding decision's
evidence stays addressable for the charter's audit period), and expiry writes a
**tombstone** — `deleted_at` and `deletion_reason` on the same row — rather than
removing it. See [Site authority](site-authority.md) for who may invoke the
sweep and how it is audited. A tombstoned row stays readable and nothing new may
rest on it: no derivation, no assessment, and a re-acquisition of the same
content makes a new row rather than re-citing it (see
[What a tombstoned snapshot refuses](deployment.md#what-a-tombstoned-snapshot-refuses)).
