---
answers:
  - Should a contribution's EvidenceRef register a resource reference, or be refused when none exists?
  - What is the identity of a resource reference — the locator, or the locator and the digest?
  - Why was `locator_digest_conflict` retired?
  - May two tenants cite the same URL at different digests?
  - Is a digest-less evidence citation acceptable?
  - Where does a contribution's citation get its `scheme` from?
  - What bounds how many references one contribution may register?
---
# A citation registers the reference it names, and the (locator, digest) pair is the key

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.14.3.2`
- **Date:** 2026-09-16

## Context

`docs/decisions/2026-09-16_the-deliberation-flow-owns-the-evidence-chain.md`
decided that the deliberation flow owns the evidence chain and named two join
points from ROADMAP §13.2. `SIGNOFF-REPAIR.11.14.3.1` built the second one —
step 6, the `assess` step. This record settles the first: step 2, *"register
context and resource references"*.

The census behind "nothing joins it", classified rather than counted:

```bash
# Pinned at 69b6374, the commit before this record's repair, which adds hits of
# its own — the working tree measures the repair, not the defect.
git grep -n evidence_refs 69b6374 -- crates/reasonbraid-server/src   # -> 6 hits, 1 file
```

All six are in `threads.rs` and every one is inert with respect to the store:
two doc comments, the struct field, two emptiness checks in the contribution
validator, and one write of the value verbatim into the event body. **Zero**
read a digest, resolve a locator, or touch `resource_references`.

## The decision

⭐ **A contribution's `EvidenceRef` registers the §12.1 reference it names, in
the contribution's own transaction, and the event carries the resulting
`resource_id`.** Registering is §13.2's own verb for step 2; step 6 acquires.

⭐ **The identity of a reference is the `(original_locator, expected_digest)`
PAIR** — the key `migrations/0023_resource_references.sql:21` already declares.
`resources::submit`'s locator-alone pre-check is corrected to that key, and the
`locator_digest_conflict` refusal is **retired**.

Three clauses follow from those two:

1. **A digest-less citation registers with no pin.** §12.1 makes
   `expected_digest?` optional, and a contributor who has not yet acquired the
   bytes cannot know the digest — step 6 is where acquisition happens. The
   uniqueness holds for the unpinned row too: migration `0065` rebuilds the
   constraint `NULLS NOT DISTINCT`, so two unpinned citations of one locator are
   one row rather than two.
2. **The scheme is derived from the locator, never claimed beside it.** The
   citation carries only a URI, so the scheme is parsed from it (RFC 3986 §3.1)
   and a URI without one is refused by name. ⚠️ This makes the citation path
   stricter than `POST /v1/resources`, whose `scheme` is caller-supplied and
   unvalidated — owned as `SIGNOFF-REPAIR.11.14.3.5`, not fixed here.
3. **The digest is validated as ADR-011 `sha256:<64 hex>`,** by the same
   `ResourceReference::digest_error` the route uses. A citation's digest was
   previously free text.

## Why the pair, and not the locator

Three independent clauses of the frozen roadmap say the same thing, and the
schema already agrees with all three.

**§12.1.** *"The original locator is immutable. Canonicalization for
cache/deduplication is separate and scheme-specific; it must not erase
security-relevant distinctions."* Two different expected digests for one locator
IS a security-relevant distinction — it is the difference between two versions
of a page. Collapsing them onto one row erases it.

**§12.6.** *"A live Web page or branch can change."* The whole reason
`EvidenceSnapshot` exists. A store that cannot hold two pins for one locator
cannot record the change its own snapshot layer is built for. `snapshots::submit`
already replays on `(reference_id, raw_digest)`, so the versions were always
expected to be plural.

**§9.8.** *"Internal secrets, policy internals, and cross-tenant existence are
not leaked."* `locator_digest_conflict` is exactly a cross-tenant existence
leak: submitting `(L, D2)` and receiving a 409 tells the caller that some other
principal registered `L` at a digest they were not shown. It is also a denial —
under the old pre-check, the first principal to pin a locator made that locator
**uncitable by every other tenant, in any form**, including digest-less. On a
public network that is an abuse primitive: pin the popular URLs and nobody can
cite them.

⛔ The refusal was not §12.1's rule. §12.1 makes the *locator* immutable — no
row's locator is ever updated, and that remains true. The stricter
"one digest per locator, first writer wins" was invented in the pre-check and
contradicted by the table's own `UNIQUE (original_locator, expected_digest)`,
which the pre-check made unreachable for a differing digest.

⚠️ Stated at its real width rather than at its convenient one: the pre-check and
the insert were never atomic, so two concurrent submissions of one locator at two
digests could both pass it and both insert. The rule it enforced was therefore
"one digest per locator, *usually*" — which is a weaker property than the
constraint it was overriding, and another reason to let the declared key win.

## The prior ruling this contradicts, named

`docs/CLAIM_VERIFICATION.md` leg 2: *"the cheapest such oracle is your own
project's history … if it was ruled the other way, your finding must name the
difference — and if you cannot name one, the earlier ruling wins."* There is such
a ruling, and it ruled the other way.

`docs/tasks/PHASE-4.md`'s `PHASE-4.1.2` record states the contract as shipped:
*"the same locator with a DIFFERENT digest is the typed `locator_digest_conflict`
— the immutability is the update-refusal + the conflict, not a convention"*, and
it was measured by a control that asserted the 409.

⭐ **The difference is what that record measured and what it did not.** It
measured the refusal's *behaviour* — that the route returns 409 with that code —
and it was right: the behaviour was as designed. It did not ask **whose data
decides the refusal**, and the answer is *another principal's row*, which turns
the 409 into a cross-tenant existence disclosure under §9.8 and the refusal into
a cross-tenant denial. Nothing in that record is falsified; a question it never
asked is now answered, and the answer moves the verdict. `PHASE-4.1.2` carries a
superseded-in-part note saying exactly that, and everything else it measured
stands.

## The rejected alternatives

1. **Refuse a citation that matches no registered reference.** Rejected on
   §13.2: step 2 *registers* and step 6 *acquires*. Requiring registration
   before citation inverts the flow, makes the already-wired `evidence_request`
   step pointless, and contradicts §12.2's `resource_unresolvable_now`, which
   exists to *preserve the reference for later*.
2. **Record the citation as a dangling link acquisition later fills.** Rejected:
   that is what exists today — a string — only with a column added. The
   reference row IS the registration; there is nothing to defer.
3. **A thread-keyed citation table.** Rejected on the same schema argument
   `2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md` uses for the
   evidence tables: the reference is shared by construction, and the thread link
   already exists because the contribution event names the `resource_id`.
4. **Keep the locator-alone pre-check and let a citation inherit its refusal.**
   Rejected on the abuse and disclosure analysis above. It was the smallest
   change and it would have amplified a cross-tenant denial from a
   rarely-used route onto the highest-volume write path in the product.
5. **Give the citation path its own registration semantics and leave
   `resources::submit` alone.** Rejected because it produces an incoherent
   store: a citation registering `(L, D2)` beside an existing `(L, D1)` would
   leave the route refusing to replay a row that demonstrably exists. Two
   writers with two keys into one table is the asymmetry `.11.14.3.3` exists to
   clean up; this record declines to create a second instance of it.

## What this decision does NOT say

It does not make the pin enforced: nothing checks an acquired snapshot's bytes
against its reference's `expected_digest`, and that gap is older than this
record — owned as `SIGNOFF-REPAIR.11.14.3.6`. It does not bind the reference
READ to a tenant; `resource_references` remains *site-wide by design* per
`2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`, and the residual
oracle that registration makes reachable from the contribution path is owned as
`SIGNOFF-REPAIR.11.14.3.4`. It does not settle what becomes of
`claim_assessments`'s invented identifiers — that stays `.11.14.3.3`. And it
does not bound what a contribution may cite: `thread.contribute` carries no
quota and no length limit, so registering turns an unbounded citation list from
unbounded INPUT into O(n) work inside the transaction holding the thread's
`FOR UPDATE` row — measured and owned as `SIGNOFF-REPAIR.11.14.3.7`, with a cap
deliberately not invented here.

## How to apply

- A new writer into `resource_references` uses `resources::submit` and inherits
  the pair key; do not add a second lookup shape.
- A refusal that depends on a row another tenant wrote is a §9.8 disclosure
  question before it is a validation question. Ask it in that order.
- Related: [[2026-09-16_the-deliberation-flow-owns-the-evidence-chain]],
  [[2026-09-16_evidence-is-shared-the-read-is-tenant-bound]].
