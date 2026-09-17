# A snapshot names the locator its reference names; `final_locator` stays free

- Date: 2026-09-17
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.14.3.13`
- Related: `2026-09-17_the-snapshot-write-is-bound-to-the-reference-it-names.md`
  (which bound the writer and measured that the binding does not reach this),
  `2026-09-16_a-citation-registers-the-reference-it-names.md` (§12.1's immutable
  locator).

## The finding

`SnapshotSubmission` carries `original_locator`, and `snapshots::submit` bound it
straight into the `evidence_snapshots` insert. The only things it read from
`resource_references` were the pin and the registration — never the locator. So a
snapshot could say it was an acquisition of one document while its reference
named another. Reproduced against the unrepaired store: a submission naming
`https://example.org/some-other-document`, filed against a reference registered
as `https://example.org/the-registered-report`, was **stored** — `200
{"replay":false,"snapshot_id":"snp_01a0af0ef6…"}`.

§12.6 asks an `EvidenceSnapshot` to carry *"original reference and resolved final
locator"*. Those were two facts that need not agree.

## The census that shaped the disposition

```bash
grep -rn "original_locator" crates/reasonbraid-server/src/*.rs
```

Every hit outside `resources.rs` is either `reference.original_locator` — the
**reference's** field, which the resolvers read to fetch — or the snapshot
column being written, mapped into `StoredSnapshot`, and listed in the read
surfaces' column set.

⭐ **Nothing in the product reads `evidence_snapshots.original_locator` to make a
decision.** No resolver, no gate, no routing rule. That is what makes this an
evidence-integrity defect rather than a routing one, and it is why a refusal is
the whole repair: there is no behaviour to correct downstream, only a record that
could be false.

## The decision

**The submission's `original_locator` must equal the reference's, byte for byte.
A disagreement is `SnapshotError::LocatorMismatch`, before anything is written.
`final_locator` is untouched.**

⚠️ **Byte equality on purpose, and this answers the leaf's own warning.** The
leaf recorded: ⛔ *"Do NOT assume the answer is an equality check … §12.1 says
canonicalization is separate and scheme-specific and must not erase
security-relevant distinctions, so an equality check is a canonicalization
decision wearing a different name."* That is true of a **normalising** check — one
that lower-cases a host or strips a trailing slash to decide two spellings are
the same. A **strict** check normalises nothing and decides nothing: it refuses a
disagreement and leaves both spellings exactly as they are. §12.1's immutable
locator is precisely what makes the reference's stored string the identity to
compare against.

⭐ **`final_locator` stays free, and the control asserts it.** A redirect
legitimately ends somewhere else, which is why §12.6 records both. A repair that
compared them too would have been wrong, not merely stricter.

The refusal **names both values**. This check runs after the registration
predicate `.11.14.3.11` added, so the caller has already proved it registered the
reference and may read that locator; quoting it discloses nothing it does not
hold, and the diagnosis is worth more than the symmetry.

## The alternatives, and why each was rejected

1. **Derive the stored value from the reference and ignore the submission's** —
   ❌ it silently discards a field the caller stated. This project has ruled
   against exactly that shape before: `SIGNOFF-REPAIR.3.4.4` found a recruitment
   response whose extra member was silently dropped, recording a *join* for a
   payload that plainly said *decline*. A stated field is checked or refused,
   never quietly overwritten.
2. **A normalising comparison** — ❌ the leaf's warning, and it holds: choosing
   which differences are insignificant is a canonicalization decision, it is
   scheme-specific, and §12.1 defers it. A strict check needs none of that.
3. **Document it as the acquisition's own record** — ❌ that would be defensible
   if something distinguished the two, but nothing does: the column has the same
   name and the same meaning as the reference's, is never read, and the read
   surfaces return it beside `reference_id` with no hint that it may differ. A
   reader cannot act on a distinction the product does not express.

## What it does NOT close

- ⚠️ **No sweep.** A snapshot stored before this change whose locator disagrees
  with its reference keeps its row. Nothing lists a reference's snapshots
  (measured in `.11.14.3.11`), so nothing can enumerate them either.
- ⚠️ **`resource_references.scheme` is still a caller field validated against
  nothing**, which is the same family one level up and remains
  `SIGNOFF-REPAIR.11.14.3.5`'s.

## Verification

- RED, against the exact unrepaired store: the disagreeing submission was stored;
  `49 passed; 1 failed`.
- GREEN: the disagreeing submission is refused `400` naming **both** locators,
  with **0** rows written; the agreeing submission is accepted **with a different
  `final_locator`**, and both stored values are asserted.
