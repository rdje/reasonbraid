# A citation is withdrawn by its tenant; a shared row is tombstoned by the site

- Date: 2026-09-18
- Status: accepted
- Owner: `SIGNOFF-REPAIR.7.4.4`
- Supersedes nothing; NARROWS `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`,
  which bound the READS of a shared evidence row and left the WRITE that
  destroys it reachable by any citer.
- Related: `migrations/0070_citation_withdrawal.sql`, `migrations/0062_evidence_citations.sql`,
  `docs/decisions/2026-09-09_site-operator-authority.md`, ROADMAP §12.9, §16.8.

## The decision

`DELETE /v1/snapshots/{id}` **withdraws the caller's citation**. It no longer
tombstones the snapshot row. Tombstoning a named row is a **site-operator act**,
`POST /v1/snapshots/{id}/tombstone`, under the `evidence_expire` authority that
already governs the retention sweep.

## The finding it answers

A snapshot two tenants cite is ONE row, by construction: `resource_references`
is UNIQUE on `(original_locator, expected_digest)` and `snapshot_objects` is
keyed by `digest` alone (ADR-011), so the second citer replays the first's row
and `0062` records it in the citer set. Sharing is the design.

`SIGNOFF-REPAIR.11.14.1` bound the delete verb to a CITING tenant. That stopped
a stranger reaching it and does not separate two citers, because both are
citers. So tenant A's deletion removed the evidence from tenant B's surface and
stamped B's receipt with A's reason — measured, before the repair, as B reading
`deleted_at: "2026-09-18T09:09:11Z"`, `deletion_reason: "tenant A no longer
relies on this"` on a row B had cited and A had deleted.

## Why two verbs and not a predicate

One verb carried two acts, and they do not share an authority.

*Withdrawing a citation* says **this tenant no longer relies on this evidence**.
It is a statement about one tenant's own relationship to a row, it touches no
other citer, and the tenant plainly owns it.

*Tombstoning the row* says **this evidence must not be relied upon by anyone**.
That is a statement about shared bytes, and it is the same authority the
retention sweep already needs: `snapshots::expire_due` carries no tenant
predicate and cannot, because `retention_class` is a column on the shared row,
which is why `SIGNOFF-REPAIR.7.4.3` made invoking it a site-operator capability.
Tombstoning one named row differs from tombstoning every due row only in the
`WHERE` clause.

## Rejected alternatives

**Refuse the delete when another tenant cites the row.** Rejected: it makes one
tenant's behaviour depend on another's, observably. A caller could learn whether
some other tenant cites a given snapshot by watching its own delete succeed or
fail — the cross-tenant existence leak §9.8 forbids and that `.11.14.3.2` already
retired `locator_digest_conflict` for.

**Tombstone only when the last citer leaves.** Rejected for the same reason plus
a worse one: it hands any tenant the shared-row authority this decision removes,
by the back door of happening to be the only citer. Whether a row dies would
depend on a fact the caller cannot see and did not choose. A row nobody cites is
simply read by nobody — exactly the state `0062` describes for every row written
before citations existed — and the retention sweep reaps it on its class TTL.

**Give each tenant its own snapshot row.** Rejected upstream and restated here:
`2026-09-16` weighed it for all twelve evidence tables and it either breaks the
content-addressed uniqueness or stores identical bytes once per tenant.

**Delete the citation row instead of marking it.** Rejected on §12.9 — *never a
silent disappearance*. A `DELETE FROM evidence_citations` leaves no answer to
*who stopped relying on this, when, and why*, which is the question an audit of a
deliberation's evidence trail asks. Three nullable columns cost less than that
answer. Re-citation clears them and deliberately leaves `cited_at`/`cited_by`
alone: the original citation time and actor are the durable fact, and a
withdrawal is an episode in that row's life rather than a new row.

**A new `GrantAction` variant for the named tombstone.** Rejected as scope and as
design. `permitted_actions` stores wire names, so a new action is a MIGRATION no
existing boundary can contain — that decision belongs to `SIGNOFF-REPAIR.9.3.4.1`
— and expiring a named row is the same authority over the same store as expiring
every due row, so an operator granted `evidence_expire` would expect it to cover
both.

## The irreversibility, addressed

The leaf recorded that a tombstone is first-wins and permanent: the `UPDATE`
carries `AND deleted_at IS NULL` and `git grep -n "SET deleted_at" -- crates`
returns three statements, none of which clears the column.

That is **accepted for the tombstone and removed for the tenant**. A tombstone is
now reachable only by an authorized, audited site act, so the irreversible write
has an operator behind it and a receipt in front of it. What a tenant can do is
now fully reversible: a withdrawn citation is restored by citing the evidence
again, which a control exercises. The asymmetry is deliberate — §12.9 wants
evidence deletion to be a recorded, deliberate act, and undoing one would make
the record of it a lie.

## Consequences

- `evidence_citations` gains `withdrawn_at`, `withdrawn_by`, `withdrawal_reason`
  and a partial index on the live citer set.
- Every disclosure surface says `withdrawn_at IS NULL`: `get_for_tenant`, the
  staleness list, `is_cited_by` (the gate the child reads apply) and the
  derivation join. A withdrawn citation is not a citation.
- The reply field changed from `tombstoned` to `withdrawn`. Reporting a tombstone
  that did not happen was judged the worse compatibility choice; the request body
  is unchanged and `reason` is still required.
- `the_snapshot_store_roundtrips_replays_and_tombstones` is renamed
  `…_and_withdraws`, with the three moved assertions named in its body.
