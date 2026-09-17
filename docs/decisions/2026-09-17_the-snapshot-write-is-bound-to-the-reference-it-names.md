# A snapshot is filed against a reference its own tenant registered

- Date: 2026-09-17
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.14.3.11`
- Extends: `2026-09-17_the-reference-read-is-bound-to-its-registrants.md`, whose
  census this leaf corrects **in reach** rather than in verdict.

## The finding, and it is a write

`POST /v1/snapshots` admitted on enrolment alone and passed the caller's
`reference_id` to the store, which asked its reference table only whether the id
existed. Driven with a second tenant against a reference it had never
registered, beside an absent id:

```text
foreign: 200 {"replay":false,"snapshot_id":"snp_01a0aee3660d7d32906d5524d6423f98"}
absent:  400 {"code":"invalid_command","message":"the reference does not exist"}
```

So it was not merely an existence oracle over `res_…` ids. The foreign
submission **succeeded**: a snapshot was attached to a reference another tenant
registered.

⛔ **And nothing could see it.** `grep -rn "WHERE reference_id"
crates/reasonbraid-server/src/*.rs` returns **1** — the replay lookup inside
`submit` itself. No route lists a reference's snapshots, so the attachment is
invisible to the reference's own registrants. That bounds the harm and does not
excuse it: an invisible write is worse evidence than a visible one.

## How it was missed

⭐ This is a gap in `.11.14.3.4`'s own census, taken one commit earlier:

```bash
grep -n '"/v1/resources' crates/reasonbraid-server/src/api.rs   # -> 3 routes
```

Three routes, two unbound, both bound. `POST /v1/snapshots` names a
`reference_id` in its **body**, so a route-prefix enumeration cannot see it. The
census was not wrong — it was **silent**, which is the more dangerous failure,
because a wrong census invites a second look and a silent one reads as complete.

⚠️ `.11.14.3.4`'s verdicts stand for the routes it enumerated. What is corrected
is the claim a reader would take from them. The rule is promoted as
`docs/knowledge/a-census-is-as-wide-as-its-key.md`: **enumerate on the
identifier, not on the address shape** — this is the second instance of the shape
in this repository, after `.3.5.3`.

## The decision

**The snapshot write takes the same registration binding the read took.** The
reference lookup gains the predicate in the same statement:

```sql
SELECT r.expected_digest FROM resource_references r
JOIN reference_registrations g
  ON g.resource_id = r.resource_id AND g.tenant_id = $2
WHERE r.resource_id = $1
```

⛔ **No new error variant.** A reference the caller did not register answers
`ReferenceMissing` — the same answer an absent id gets — so there is nothing for
the distinction to leak through. The message becomes *"the reference does not
exist, or this tenant did not register it"*: one sentence for two cases,
deliberately.

## The leaf's warning, answered rather than obeyed

The leaf recorded: ⛔ *"Do NOT assume symmetry with the read settles it. A
snapshot submission carries the BYTES, so a caller reaching this route already
holds the content — which is the asymmetry that made the snapshot family's replay
safe, and it cuts the other way here."*

It does cut the other way, and it is still not a reason to leave the write open.
⭐ **A `SnapshotSubmission` also carries `original_locator`.** A caller that can
make one therefore already holds everything registration needs: it registers the
pair, receives the **same** reference id back, and files. That is the same
"breaks no reachable caller" measurement the two preceding leaves made, and it is
a control arm here rather than an assertion.

## The alternatives, and why each was rejected

1. **Leave it and publish** — ❌ the surface is a write, not a read. Publishing an
   exposure is an honest disposition for a disclosure whose closure would cost a
   working path; here the closure costs nothing and the exposure includes
   *modifying another tenant's evidence graph invisibly*.
2. **A distinct refusal for "registered by someone else"** — ❌ that is the oracle
   written down. The whole point of reusing `ReferenceMissing` is that the two
   cases become one answer.
3. **Bind at the handler rather than in the store** — ❌ same reasoning as
   `.11.14.3.8` and `.11.14.3.6`: the store is what every writer reaches, and the
   resolvers reach it too. Binding it there also makes the resolver arms
   consistent for free, since a resolving tenant is already a registrant
   (`.11.14.3.4`).

## What it does NOT close

- ⚠️ **A snapshot still records a locator its reference need not carry.**
  `snapshots::submit` binds `submission.original_locator` straight into the
  insert and never compares it with the reference's, so a registrant may file a
  snapshot that says it is an acquisition of one document while its reference
  names another. `SIGNOFF-REPAIR.11.14.3.13` owns it. ⛔ Not an obvious equality
  check: §12.1 keeps canonicalization separate and scheme-specific, so an
  equality check is a canonicalization decision wearing a different name.
- ⚠️ **No sweep.** Snapshots attached to a foreign reference before this change
  keep their rows; nothing re-parents or removes them, and nothing can list them.

## Verification

- RED, against the exact unrepaired store: the foreign submission succeeded
  (`200`, a fresh `snapshot_id`) while the absent id was refused;
  `48 passed; 1 failed`.
- GREEN: foreign and absent are refused in the **same words** with **0** rows
  attached; the registering tenant files normally; the second tenant registers
  the same locator, replays to the same reference id, files, and the submission
  replays to **one** shared snapshot row carrying **2** citations.
