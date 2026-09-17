# A reference's `expected_digest` names its bytes; an unpinned reference holds the versions

- Date: 2026-09-17
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.14.3.6`
- Related: `2026-09-16_a-citation-registers-the-reference-it-names.md` (which made
  the pin part of the reference's identity and therefore made this a question
  with an answer), `2026-09-17_the-reference-read-is-bound-to-its-registrants.md`.

## The finding, censused rather than counted

```bash
git grep -n expected_digest -- crates/reasonbraid-server/src
```

Every hit is `resources.rs`'s own store — the struct field, the ADR-011
validator, the SQL, the row mapping — plus two throwaway `ResourceReference`
values built *only* to reuse `digest_error()`, and two doc comments about the
uniqueness constraint. **Zero read a STORED pin.**

`snapshots::submit` verified that the bytes hash to the *submission's own*
`raw_digest` — content-addressing verified, not trusted — and then asked the
reference table one question:

```sql
SELECT EXISTS (SELECT 1 FROM resource_references WHERE resource_id = $1)
```

So the §12.1 field a caller supplied to say *"these are the bytes I expect"*
constrained nothing. Reproduced against the unrepaired store: a reference pinned
to one digest accepted a snapshot of entirely different bytes, `200
{"replay":false,"snapshot_id":"snp_01a0aec5…"}`.

## The decision

**A pinned reference accepts only the bytes it names. An unpinned reference is
unchanged.**

`snapshots::submit`'s lookup now returns `expected_digest` instead of asking
`EXISTS`: the outer `Option` is existence, the inner one is the pin. A mismatch
is `SnapshotError::PinMismatch`, before anything is written.

⭐ **Enforcement is what makes `.11.14.3.2`'s pair key mean something.** That leaf
made `(original_locator, expected_digest)` the reference's identity so that
§12.6's changed page would be a SECOND reference rather than an erased
distinction. With no checkpoint, both rows accepted any bytes — so the
distinction the key was created to preserve was preserved nowhere.

⚠️ **The plural is relocated, not forbidden**, and that is the whole shape of the
rule rather than an exemption. `evidence_snapshots` replays on `(reference_id,
raw_digest)`, so one reference holds many versions — which is exactly what a
living page needs, and which stays true of an **unpinned** reference. A pin says
the opposite about its own reference: these bytes, this row. The two compose
because the changed page has its own reference to attach to.

⛔ **The refusal carries NEITHER digest.** The caller already holds the actual one
— it hashed the bytes it sent — and the pinned one belongs to a reference this
route does not check the caller may read, so quoting it would make the refusal an
oracle over a `res_…` id. The message says what to do instead, which is the part
a legitimate caller does not already have: *register the locator at the new
digest and acquire against that.*

## The alternatives, and why each was rejected

1. **Record a MISMATCH on the snapshot and let the reader see it** — ❌ it stores
   a false record: a snapshot filed under a reference whose pin it violates,
   with a flag asking every future reader to notice. §12.7's standard for this
   chain is that *citation existence alone never satisfies an evidence gate*, and
   this project refuses rather than annotates wherever the refusal is cheap. The
   refusal here costs one named error and one documented next step.
2. **State that the pin is a caller's note** — ❌ it contradicts
   `.11.14.3.2`, which made the pin part of the reference's IDENTITY. A key
   column that constrains nothing is `.11.14.3.3`'s finding in another costume:
   *every column of a replay key must be a value the submitter is entitled to
   assert* — and a column the submitter asserts, that nothing ever compares
   against, is worse than absent, because its name promises a check.
3. **Enforce at the resolver instead of in the store** — ❌ same reasoning as
   `.11.14.3.8` one slice earlier: the store is what every writer reaches. There
   are four `snapshots::submit` call sites and a fifth would inherit the hole.

## The consequence, stated rather than discovered

⛔ **A pinned reference whose page has changed now fails to persist a snapshot on
the acquisition paths, and two of those paths DISCARD that failure.**
`api.rs`'s R0 and R5 arms call `let _ = crate::snapshots::submit(…)` — a
pre-existing discard, documented in place as *"a persistence failure leaves the
receipt returned (the acquisition succeeded)"*. With the pin enforced, a caller
resolving a pinned reference whose bytes drifted receives an acquisition receipt
and no snapshot, silently.

⚠️ That is the same discard that was already there for a storage fault; what
changed is that it now has a likely, caller-meaningful cause. It is
`SIGNOFF-REPAIR.11.14.3.12`'s, opened with this measurement — the R2 arm already
captures its result, so the census is two sites, not four.

## What it does NOT close

- ⚠️ **Nothing binds `POST /v1/snapshots` to a registrant of the reference it
  names.** The route is enrolment-gated, so a caller holding a `res_…` id can
  submit bytes against a reference it never registered and learn — from
  `ReferenceMissing` versus any other answer — that the id exists.
  `SIGNOFF-REPAIR.11.14.3.11` owns it. ⭐ It is recorded here because it is a gap
  in `.11.14.3.4`'s own census one commit earlier: that census enumerated the
  routes under `/v1/resources` and this route names a reference in its **body**.
  The same blind spot `.3.5.3` had, in a different family.
- ⚠️ **Nothing re-checks a pin after the fact.** A snapshot stored before this
  change against a pin it violates keeps its row; there is no sweep, and none is
  invented here.

## Verification

- RED, against the exact unrepaired store (the production file reverted and
  restored): `200 {"replay":false,"snapshot_id":"snp_01a0aec5…"}`;
  `46 passed; 1 failed`.
- GREEN: the pinned reference refuses other bytes `400`, quoting neither digest,
  with **0** rows written; it accepts its own bytes; an **unpinned** reference
  still holds **2** versions; and the same locator at the other digest is a
  second reference that acquires normally. `47 passed; 0 failed`.
