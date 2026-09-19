# A schema pattern nobody reads is not a contract

- Date: 2026-09-20
- Status: accepted
- Leaf: `SIGNOFF-REPAIR.11.24.1.1.2.1.1.1.1`
- Follows `2026-09-20_revoking-an-authority-records-when.md`, whose census found
  this table and whose framing this record corrects.

## The question

`authority/federation_admin.rs` revokes a direction with
`UPDATE federation_agreements SET status = 'revoked' …` — the status alone, the
same shape `SIGNOFF-REPAIR.11.24.1.1.2.1.1.1` repaired on `authority_grants`.
`migrations/0048_federation_agreements.sql` declares the table with
`proposed_at NOT NULL DEFAULT now()` and `accepted_at`, so it records two of its
three lifecycle instants and leaves the third undated beside them.

The leaf opened calling that **sharper** than the instance that found it: where
`authority_grants` recorded no transition instants at all — a uniform omission —
here the schema itself establishes a pattern that the missing column breaks, and
*a reader looking at the columns would reasonably conclude the instant is there*.

## The decision: no column, and the opening argument is withdrawn

**There is no such reader.** `git grep -n "accepted_at\|proposed_at" -- crates`
🔴 **CORRECTED by `SIGNOFF-REPAIR.13.4.3`: this paragraph first said the command
returned "exactly one hit outside the two writes", and that number is FALSE.**
The unfiltered command returns **2** hits, both in `federation_admin.rs` — line
304 (the accept `UPDATE`) and line 231 (the re-proposal resetting
`accepted_at = NULL`) — and both are writes. The *1* came from a `grep -v` I
applied to my own command and did not carry into the sentence. **A measurement's
scope is part of the number.**

The finding is unchanged, and stands on the corrected count: **both** hits are
writes, neither column appears in any `SELECT`, and the two instants this table
records are **written and read by nothing**.

So the pattern is a writing habit, not a contract anyone depends on, and the
argument that a reader would infer the third column from the other two is refuted
by there being no reader of the other two. That argument was mine, written one
commit earlier, and it is withdrawn rather than quietly dropped.

## The reader census the acceptance required

Every read of `federation_agreements` in the server:

| Site | Reads | Filters on |
| --- | --- | --- |
| `federation.rs` `has_effective_directory_agreement` | `directory_visibility` | `status = 'accepted'` |
| `federation.rs` `has_effective_recruitment_agreement` | `recruitment` | `status = 'accepted'` |
| `federation.rs` `has_effective_recruitment_agreement_in_tx` | `recruitment` | `status = 'accepted'` |
| `federation_admin.rs` (the revoke verb) | `status` | the target pair |

Their two callers — the directory read gate in `api.rs` and the card-import
allowlist rung in `authority/profile_admin.rs` — are **decision-time liveness
gates**. A revoked direction widens nothing, and `status` answers that
completely. None of them asks *when*.

**And nothing durable is aged by an agreement's revocation.**
`cross_domain_receipts`, the one artifact a federated act leaves behind, carries
its own `created_at NOT NULL DEFAULT now()`, so it is aged by itself.

## Why the grant's answer does not transfer

`authority_grants` gained `revoked_at` because a **second reader that is not the
audit log** needed to age something by the instant: the inbox prune had to retain
a `revoked` delivery row for a window measured in time-in-terminal, and no other
column answered that. That reader exists there and does not exist here.

A column is not missing until something needs it. That was true of the grant's
column until this week, and it is true of this one now.

## The trigger

Add the column when a reader must **age or order** something by when a federation
direction was withdrawn — for example if an imported card, a cross-domain receipt
or a directory projection ever needs retention keyed to the revocation rather
than to its own creation. At that point
`SIGNOFF-REPAIR.11.24.1.1.2.1.1.1`'s two rules apply unchanged: write it in the
same statement that changes the status, and let it answer *when* and never
*whether*.

## One observation recorded, and deliberately not graded a defect

`accepted_at` and `proposed_at` are written and never read. That is **not** the
shape `SIGNOFF-REPAIR.11.24.1.2` grades — an advertised wire value that can never
be produced — because these have a producer and no consumer, which is the
opposite arrangement. Latent data on a row an operator can inspect directly is a
legitimate thing for a schema to carry. It is recorded because it is the evidence
this decision rests on, not because it is owed a repair.

## Evidence

- The reader census above, produced by `git grep -n "federation_agreements" --
  crates/reasonbraid-server/src crates/reasonbraid-cli/src` and read site by site
  rather than counted.
- `git grep -n "accepted_at\|proposed_at" -- crates` → **2** hits, both writes
  (`federation_admin.rs:231` and `:304`), and no `SELECT` among them. 🔴 First
  published as *1 hit outside the two writes* — a `grep -v` artifact, corrected
  by `SIGNOFF-REPAIR.13.4.3`.
- No code changed, so there is nothing to falsify and nothing to regress. The
  discriminating evidence is the census, and it was capable of the other answer:
  a single `SELECT … accepted_at` would have shipped the column.
