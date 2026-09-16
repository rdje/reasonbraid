---
answers:
  - Should the policy, evaluation, deployment and evidence tables carry a tenant_id column?
  - Why do twelve tables have no tenant column when ROADMAP §16.8 says tenant id is everywhere?
  - Is the evidence store site-wide by design or by omission?
  - What does §16.8 actually require to be tenant-scoped?
  - How is a snapshot's citing tenant derived, and why can it not be a column?
  - Why can a snapshot written before the tenant binding be read by nobody?
  - Why does claim_assessments get a tenant column when the evidence tables do not?
  - Can two tenants citing the same snapshot read each other's assessments of it?
---
# The evidence chain is shared by design; what must be tenant-bound is the authorization decision, not the row

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.14`; the live defect it found is `SIGNOFF-REPAIR.11.14.1`
- **Date:** 2026-09-16

## Context

`SIGNOFF-REPAIR.11.9.1.3.3` measured `R-86-1` clause 6 and found that **twelve of
twelve** policy, evaluation, deployment and evidence tables carry zero
`tenant_id` columns:

```bash
# per table, the CREATE TABLE block only — a whole-file grep is wrong here,
# because several of these files also create tenant-scoped tables
awk "/CREATE TABLE[^\n]*<table>/,/\);/" migrations/<file> | grep -c tenant_id
```

`evaluation_corpora`, `evaluation_runs` (`0033`), `evaluation_trials` (`0034`),
`evaluation_calibrations`, `evaluation_gates`, `evaluation_gate_results`
(`0035`), `policy_publications` (`0042`), `deployment_targets`,
`deployment_assignments` (`0043`), `claim_assessments` (`0030`), `derivations`
(`0029`), `evidence_snapshots` (`0028`) — all zero.

ROADMAP §16.8 reads: *"Tenant ID is part of every aggregate key, authorization
decision, object-store namespace, encryption context, queue subject, and trace
access rule."* The obvious reading is that twelve migrations are owed.

## The measurement that changed the question

**The evidence chain is content-addressed, and the schema says so in a
constraint.** `migrations/0023_resource_references.sql:21` declares
`UNIQUE (original_locator, expected_digest)`. Two tenants that cite the same URL
at the same digest **share one row by construction**. `snapshot_objects` is keyed
by `digest` alone. That is ADR-011's design, not an oversight: adding a tenant
column would either break the uniqueness constraint or duplicate identical bytes
per tenant, destroying the deduplication the digest exists to provide.

So "add `tenant_id` to the evidence tables" is not a repair. It is a rewrite of a
deliberate content-addressed store, and it would make the store worse.

⭐ **And §16.8 does not ask for it.** Read the sentence again: tenant id is part
of every *aggregate key* and every *authorization decision*. An evidence snapshot
is not an aggregate — it is an immutable receipt over shared bytes. What §16.8
requires of it is that the **decision to disclose it** is tenant-aware. That
decision is exactly what is missing.

## The decision, per table

⛔ **None of the twelve gains a `tenant_id` column.**

| Table(s) | Verdict | Why |
| --- | --- | --- |
| `snapshot_objects`, `resource_references` | **site-wide by design** | content-addressed; `UNIQUE (original_locator, expected_digest)` makes sharing structural. A tenant column breaks the constraint or duplicates the bytes |
| `evidence_snapshots`, `derivations`, `claim_assessments` | **site-wide row, TENANT-BOUND READ** | the row is a shared receipt; the disclosure decision must name the tenant that cited it. This is where §16.8 is unimplemented — owned by `.11.14.1` |
| the six `evaluation_*` tables | **site-wide by design, gated by SITE-OPERATOR grants** | a benchmark corpus is more useful shared than copied, and `docs/decisions/2026-09-09_site-operator-authority.md` already describes the shape: explicit site-operator grants, never tenant enrolment. ⛔ Today they admit any enrolled principal — `.8.2`'s attached clause from tranche 4c |
| `deployment_targets` | **site-wide by design** | a deployment host belongs to the site operator, not to a tenant |
| `policy_publications`, `deployment_assignments` | **DEFERRED to `.6.1.5`, by name** | a publication derives from a policy and an assignment binds one to a target; whether the policy registry is tenant-scoped is `.6.1.5`'s open decision and this record does not pre-empt it |

## What the decision found

🔴 **A live cross-tenant enumeration path**, measured while taking the decision
rather than inferred from it. `snapshots::stale` is:

```sql
SELECT … FROM evidence_snapshots
WHERE deleted_at IS NULL AND fresh_until IS NOT NULL AND fresh_until < $1
ORDER BY fresh_until
```

No tenant predicate, no principal, no limit. `GET /v1/snapshots/stale` gates on
`reader_tenant(…).is_some()` — enrolment — and returns every row. A
`StoredSnapshot` carries `original_locator`, `final_locator`, `resolver_id`,
`auth_class`, `provider_receipt`, `raw_digest`, `media_type` and `byte_length`.

⛔ So **any enrolled principal in any tenant can enumerate every other tenant's
evidence trail** — which documents they acquired, when, through which resolver,
under which credential class. Not the bytes; the research trail. `GET
/v1/snapshots/{id}` is the same gate for a single row.

⚠️ Stated at its real width: this is a DISCLOSURE path, not a write path, and the
bytes themselves are reached through a separate route this record does not
claim to have measured. It is an enumeration rather than an oracle, because
`stale` returns a list.

The risk register's own trigger for the cross-tenant row is *"any unresolved
cross-tenant read/write path"*. This is one.

## The rejected alternatives

1. **Add `tenant_id` to all twelve.** Rejected on the schema: it breaks
   content-addressing for three tables and answers `.6.1.5`'s open question for
   two without deciding it.
2. **Add a tenant predicate to the reads and leave the writes alone.**
   Rejected — `.6.1.5` states the trap in full: a tenant-scoped read over an
   unscoped write hides rows from their own author.
3. **Treat §16.8 as satisfied because the bytes are content-addressed.**
   Rejected: content-addressing explains why the ROW is shared. It says nothing
   about who may enumerate the receipts, which is the authorization decision
   §16.8 names.

## What this decision does NOT say

It does not say the current behaviour is safe. It says the repair is an
authorization decision on three read surfaces plus a site-operator gate on six
tables — not twelve migrations — and that two tables wait on `.6.1.5`.

## Outcome (2026-09-16, `SIGNOFF-REPAIR.11.14.1`, `REASONBRAID-REPAIR-0213`)

The read binding this record calls for is implemented, and implementing it
established one durable fact this record could not have stated: **the citing
tenant is not derivable from the schema at all.** Every path was measured and
every one failed.

- `evidence_snapshots` carries no principal column; its only link out is
  `reference_id`.
- `resource_references`'s only actor column is `submitted_by`, which
  `api.rs::submit_resource` fills from `actor_handle_for_subject(&principal)` —
  `Uuid::new_v5(NAMESPACE_OID, subject.describe())`. It joins to neither
  `human_principals` nor `agent_roles`.
- It would be the wrong answer even if it joined: `resources::submit` replays on
  the **locator alone**, so the second tenant to cite a URL receives the first
  tenant's row and `submitted_by` never moves.
- Nothing else links a snapshot to anything: `git grep -ln snapshot_id --
  migrations` returns three files, and all three are the evidence tables.

So a citation is many-to-many by construction — one shared row, N citing
tenants — and no scalar on either end can hold it. It is recorded at the moment
of citation instead: `migrations/0062_evidence_citations.sql`,
`PRIMARY KEY (snapshot_id, tenant_id)`, written by `snapshots::submit` on the
fresh insert **and on the replay**. The replay write is the part that matters:
it is what stops a tenant-scoped read from hiding a row from its own author,
which is the trap this record's rejected alternative 2 names.

Applies to the three tables this record marks *site-wide row, tenant-bound
read*. `evidence_snapshots` and the snapshot-keyed half of `derivations` and
`claim_assessments` are bound. The claim-keyed read
(`GET /v1/claims/{claim_id}/assessments`) is **not**, and is owned by
`SIGNOFF-REPAIR.11.14.2`: it is keyed on a claim, and no `claims` table exists
for it to be scoped by.

Two limits, published rather than implied: a snapshot written before the
migration has no recoverable citer and is read by no tenant until it is cited
again; and the site-wide retention sweep remains unauthorized — any enrolled
principal can tombstone every tenant's live evidence on a clock it supplies
itself, owned by `SIGNOFF-REPAIR.7.4.3`.

## Correction (2026-09-16, `SIGNOFF-REPAIR.11.14.2`, `REASONBRAID-REPAIR-0215`)

🔎 **The per-table verdict above is wrong for `claim_assessments`, and the
schema is what shows it.** The table places `evidence_snapshots`, `derivations`
and `claim_assessments` together under *site-wide row, tenant-bound read*, and
justifies "none of the twelve gains a column" from content-addressing: a row
several tenants share cannot carry an owner.

Compare the two replay keys the record did not read:

| Index | Key | Shared between tenants? |
| --- | --- | --- |
| `derivations_replay_idx` | `(parent_snapshot_id, derived_kind, derived_digest)` | **yes** — content-addressed, one row serves every tenant |
| `claim_assessments_replay_idx` | `(claim_id, snapshot_id, assessment, author)` | **no** — the key carries the AUTHOR |

Two tenants asserting the same thing about the same evidence already hold two
separate rows. An assessment is an **authored opinion**, not a shared receipt,
so the argument that correctly forbids a `tenant_id` column on
`evidence_snapshots` and `derivations` never reached it. `claim_assessments`
takes a server-derived `authored_by_tenant` column (`migrations/0064`), and the
verdict for that one row of the table is superseded:

| Table | Superseded verdict | Verdict |
| --- | --- | --- |
| `claim_assessments` | site-wide row, tenant-bound read | **tenant-owned row**, authored by one tenant, read by that tenant |

⭐ **This also closes a residual `.11.14.1` measured and could not close.** That
leaf bound the child reads on their PARENT snapshot's citation, so two tenants
citing one shared snapshot each read the other's assessments of it — the gate
admitted on the very citation they both held. Binding on the author closes it
for assessments. ⛔ It stays open for `derivations`, which are content-addressed
and shared by exactly the construction this record describes, and that limit is
published in the book rather than implied.

⛔ **The eleven other verdicts stand.** The error was one of grouping, not of
reasoning: content-addressing does make a row unable to carry an owner, and the
question that was not asked per table is whether each row is actually shared.
`author` and `verifier` remain unauthenticated caller strings and are NOT what
the binding reads — that clause belongs to `SIGNOFF-REPAIR.7.4`.
