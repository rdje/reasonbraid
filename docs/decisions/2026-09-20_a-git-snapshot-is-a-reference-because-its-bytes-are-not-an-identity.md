---
answers:
  - How is a git acquisition stored as an EvidenceSnapshot?
  - Why does a git snapshot hold no bytes?
  - Are two acquisitions of the same commit byte-identical?
  - What does `storage_class` do, and what reads it?
  - Why can a pinned reference not hold a git snapshot?
---
# A git snapshot is a reference, because its bytes are not an identity

- **Type:** decision
- **Status:** accepted; implemented in `migrations/0080` + `snapshots::submit_external`
- **Owner:** `SIGNOFF-REPAIR.11.24.1.3.2`
- **Date:** 2026-09-20
- **Raised by:** `SIGNOFF-REPAIR.11.24.1.3`, which decided R1 is owed a snapshot
  and found the store had no shape for one
- **Related:** `docs/decisions/2026-09-20_two-packs-produce-evidence-the-store-cannot-hold.md`

## The question

§12.9 offers two durability alternatives in its own words:

> Snapshots used by binding decisions remain addressable for the charter's audit
> period **or** retain a verifiable external archival reference; deletion creates
> a tombstone and reason, not silent disappearance.

R1's product is a `GitAcquisition` — an on-disk object database addressed by
`odb_path` — and `evidence_snapshots.raw_digest` was a `NOT NULL` foreign key
into `snapshot_objects (bytes BYTEA NOT NULL)`. So the only representable
snapshot was one whose bytes inline into a Postgres column. The leaf's question
was which alternative to take, and §12.9 does not answer it: it permits both.

## The measurement that answers it

🔴 **The acquired object database is not a property of the evidence. It is a
property of the remote's packing configuration.**

`git::tests::the_acquired_object_database_is_not_a_stable_identity` acquires the
same immutable commit four ways and measures `git_digest` each time:

| Leg | What changed between acquisitions | `resolved_commit` | odb digest |
| --- | --- | --- | --- |
| A | nothing | same | **same** |
| B | the source gained an unrelated commit on another ref | same | **same** |
| C | the source repacked — 4 objects, no deltas to choose between | same | **same** |
| **D** | the source repacked with `pack.window 0` / `pack.depth 0` — 66 objects, delta-capable | **same** | 🔴 **different** |

Leg D: 6857 bytes against 10071, two different digests, one commit
(`1e35b82e05c6867b3eeef749cf71b50a93d4ead0`) that never changed. `pack.window`
is read by `git-upload-pack` on the **source** side, so this is the remote
deciding to pack differently — and nobody here controls a remote.

⭐ **Re-derived by a different instrument.** Two plain `git clone --depth 1`
runs against one source repository, across the same config change, with no
product code in the path: commit `dcb8b30f…` both times, object-database digests
`1deb086c…` and `159101ec…`. The property is git's, not this acquirer's.

⭐ **The negative legs are load-bearing, not filler.** Leg C is a repack that
changes nothing, because a four-object pack has no deltas to vary. A control
written with leg D alone would look like proof and would be proving something
weaker than it claims; keeping A–C is what stops the finding being read as *any
repack changes the bytes*.

## The decision

**The external archival reference, and not because inlining is expensive.**
Keyed on the odb bytes, an upstream forge's routine housekeeping would file a
**second** evidence snapshot for evidence that did not change — so the first
alternative is not merely costly here, it is unsound for this artefact class.

The commit id does not move, and §12.6 already has the column for it —
*immutable source version where available*. Git's object model makes that id a
commitment to the entire tree, so re-acquiring the remote at that ref and
comparing the resolved commit is a complete verification. That is what §12.9's
*verifiable* asks for; a pointer with no check behind it would not qualify.

The stored reference is deliberately minimal:

```json
{ "kind": "git-commit", "remote": "…#main", "requested_ref": "main",
  "resolved_commit": "8be5445a05aeb912c85c1257cdad3282cf2073b3" }
```

⛔ **The odb digest is excluded from it on purpose.** It rides
`provider_receipt.git.digest`, beside a note saying what it is, because a
verifier that reached for it as a re-clone check would get a false mismatch —
which is precisely what leg D measures. The same correction applies to the
shipped `GitReceipt`: its `digest` is the record of one acquisition's transport
encoding, and `git_digest`'s doc comment now says so rather than reading as a
stable property.

## What reads `storage_class`

`.11.24.1.3` measured the field as written by three callers as the literal
`"standard"` and **read by no predicate**, recorded it, and declined to grade it.
Shipping a second value that nothing consults would have made it a defect, so
the class gained readers in the same change — two of them in the database, and
therefore binding on every writer including one nobody has written yet:

1. `migrations/0080`'s CHECK: an `external-reference` row holds no digest, no
   bytes (`byte_length = 0`), carries a reference, and **must** carry the
   immutable source version its identity rests on; any other class is the
   inverse. A row cannot be both shapes or neither.
2. `migrations/0080`'s partial unique index on
   `(reference_id, immutable_source_version) WHERE storage_class =
   'external-reference'` — this class's replay key, because the inline class's
   key is `raw_digest` and `= NULL` never matches. An index rather than only a
   SELECT, so two concurrent acquisitions of one commit cannot both insert.
3. `snapshots::submit` refuses the class **by name**, so a caller handing the
   inline surface an external submission gets a caller error instead of a
   constraint violation reported as a store fault.
4. `claims::submit` carries it back through a LEFT JOIN and names it in the
   refusal, so a tenant whose excerpt cannot be checked is told the storage class
   rather than that its own cited evidence does not exist.

## What follows, and what was rejected

- ⛔ **A pinned reference refuses it, by name.** §12.1's `expected_digest` says
  *these exact bytes*; this class holds none to compare. Filing the snapshot
  anyway would let it escape the constraint its own reference declares.
- ⛔ **`raw_digest` was made NULLABLE rather than filled with a stand-in.** The
  rejected alternative was to store a manifest — or the reference document
  itself — under the digest, keeping the column `NOT NULL`. That makes
  `raw_digest` mean two different things depending on a sibling column, with
  nothing stopping a reader joining `snapshot_objects` and calling the result
  *the evidence*. A nullable column makes the branch structural: the type is an
  `Option` and the compiler asks.
- ⛔ **No derivation edge**, which `.11.24.1.3` settled and this leaf did not
  reopen: §12.6's *repository analysis is not the original source* makes the
  repository the parent, and nothing in the resolve path analyses the tree.
- ⚠️ **No general external blob store was built**, and the second consumer
  waiting on this decision (`PEER-COLLAB.2`, a peer attaching a reproduction)
  did **not** widen it. Its requirements are ungraded — `PEER-COLLAB.1` has not
  yet decided whether that scenario needs a tree — and a store designed for an
  ungraded caller is a store designed by guess. What it inherits is the seam:
  a class whose meaning is *this store holds no bytes; here is what to
  re-acquire*, with the database enforcing the shape.
