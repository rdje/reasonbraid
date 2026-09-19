---
answers:
  - Is the policy registry site-global by design or by omission?
  - Does a policy belong to a tenant, or is one site-wide library intended?
  - Why does the answer differ for policy_versions and for policy_proposals?
  - Where is the caller's tenant already derived in the policy lifecycle, and what happens to it?
  - Why does binding these reads not hide rows from their own author?
  - What happens to policy_publications and deployment_assignments, which the evidence decision deferred here by name?
  - How many SQL sites does this decision cover?
---
# The policy LIBRARY is shared by design; the policy LIFECYCLE is its tenants', by omission

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.6.1.5`
- **Date:** 2026-09-19
- **Answers by name:** `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`
  (DOC-0029), which deferred `policy_publications` and `deployment_assignments`
  here
- **Re-scoped by:** `docs/decisions/2026-09-19_the-policy-registry-is-a-shared-control-surface.md`
  (DOC-0066), from one table to ten
- **Precedent:** `SIGNOFF-REPAIR.7.1.2.1`, which settled the WORKFLOW registry

## The question, as `.6.1.5` posed it

> Is one site-wide policy library intended, or does a policy belong to a tenant?

`.6.1.5` was opened over **one table and six SQL sites** — `policy_versions`, and
`policy::register`, `list`, `resolve`, `impact`, `lifecycle::register_proposal`'s
existence probe, and the MCP read. DOC-0066 re-scoped it to **ten tables**. The
real population is larger again:

```bash
python3 -B scripts/census_registry_read_reach.py     # the readers, per table
python3 -B scripts/census_shared_registry_writes.py  # the writers, by admission
```

**32 read sites and 15 write sites — 47 in all — across ten tables.** The six the
leaf named are `policy_versions` alone.

## The answer: the question had one name and two subjects

⛔ **Neither blanket shape is right, and the code says why.**

| Subject | Verdict | Why |
| --- | --- | --- |
| `policy_versions` — **the library** | **site-wide BY DESIGN** | A policy is a governance document keyed `(policy_id, version)` with an `owning_authority`. Its ownership model is a GRANT, not a tenant. Nothing links it to a thread, and nothing should: a policy that only one tenant can read is not governance |
| `policy_proposals`, `policy_decisions`, `policy_approvals`, `policy_projections`, `policy_publications`, `policy_drift`, `policy_outcomes`, `policy_corrections`, `policy_reviews` — **the lifecycle** | **tenant-owned BY OMISSION** | Each starts, directly or transitively, in a TENANT'S THREAD — and the caller's tenant is already derived and enforced at write time, then discarded |

## The measurement that settles it

⭐ **THE LIFECYCLE ALREADY KNOWS THE TENANT. IT JUST DOES NOT KEEP IT.**

`lifecycle::register_proposal` takes a `tenant_id`, and its thread check runs
under that tenant's row-level-security claim:

```rust
let thread: Option<bool> = crate::rls::with_tenant_claim(pool, tenant_id, |tx| {
    Box::pin(async move {
        sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM aggregate_state \
             WHERE aggregate_id = $1 AND aggregate_type = 'thread')",
        )
```

Its own doc comment states the intent — *"a proposal may only name a thread of
the caller's tenant — the RLS layer enforces the read"* (`.1.3.1`). The HTTP
handler derives that tenant from the authenticated principal:
`reader_tenant(&state.pool, &principal)`. `lifecycle::record_decision` does the
same for its verdict event.

⛔ **And then `INSERT INTO policy_proposals (proposal_id, policy_id,
policy_version, thread_id, status)` stores no tenant at all**, so
`list_proposals`, `publications::stage`, `record_decision` and `record_approval`
all read site-wide.

⛔ **QUALIFIED THE SAME DAY BY `SIGNOFF-REPAIR.6.1.5.1`, which tried to reuse
that claim and watched it admit a foreign tenant.** `rls.rs`'s module doc states
the mechanism — *"The dev profile's superuser connection bypasses RLS regardless
… and binds the moment the app role lands"* — and `migrations/0046`'s `FORCE ROW
LEVEL SECURITY` does not reach a superuser either. So under the profile this
repository's suites and its dev deployment actually run, **both claims enforce
nothing**, and no control has ever observed either gate working or failing. That
is a live gap in both verbs, owned by `.6.1.5.1.1`.

⚠️ **It does not weaken the decision below, and saying why matters.** The argument
here is about INTENT: a system that did not mean proposals to be tenant work
would not have written an RLS claim to enforce it. An intent is not undone by the
mechanism failing to bind — if anything the gap sharpens the verdict, because it
means the lifecycle is site-wide in practice *today* while being tenant work by
design.

🔎 **So `.6.1.5`'s question — by design or by omission? — is ANSWERED, and the
answer is BOTH, for different tables.** The library is shared by design. The
lifecycle is shared by omission, and the omission is provable rather than
inferred: a system that did not intend proposals to be tenant work would not have
written an RLS claim to enforce it.

## Why this escapes the trap `.6.1.5` was opened around

That leaf states the trap in full, and it is the reason it forbade the cheap fix:

> ⛔ Do not answer this by adding a filter to one read. The registry has a WRITE
> surface admitting any enrolled principal, so a tenant-scoped read over an
> unscoped write would hide rows from their own author.

⭐ **The trap does not apply to the lifecycle, and the measurement above is
exactly why.** The write side is not unscoped — it already derives the caller's
tenant and already refuses a thread outside it. Storing that tenant is not a new
binding invented for the read; it is RECORDING one the write already performs and
throws away. The author of a row is, by construction, the tenant the write
validated it against.

⚠️ It DOES apply to `policy_versions`, which is why that table is not
tenant-scoped here.

## The weakest link, found while measuring

🔴 **`record_approval` does not take the tenant at all.** Its two siblings do:

```
register_policy_proposal  →  let Some(tenant_id) = reader_tenant(…)  →  register_proposal(pool, &tenant_id, …)
record_policy_decision    →  let Some(tenant_id) = reader_tenant(…)  →  record_decision(pool, &tenant_id, …)
record_policy_approval    →  let enrolled = reader_tenant(…).is_some()  →  record_approval(pool, &principal, …)
```

The third derives the tenant, tests it for existence, and discards it — the
`is_some()` shape this repair programme has now found at a dozen sites. So an
approval is the one lifecycle write with no tenant enforcement of any kind, and
`publications::stage` reads approvals to decide whether a publication may be
staged. ⛔ That is a control-surface consequence of a disclosure-shaped defect,
and it is why the implementation must start there rather than with the reads.

## What `policy_versions` gets instead

Site-wide by design is not the same as unguarded, and today it is unguarded:

- `POST /v1/policies` admits **any enrolled principal**, over a first-come
  `PRIMARY KEY (policy_id, version)` namespace.
- `policy::register` requires `owning_authority` to be a LIVE grant but — its own
  comment says so explicitly — **not one the registrar holds**. So any enrolled
  principal may take a policy id and attribute it to any live grant in the site.

⭐ **The remedy is `SIGNOFF-REPAIR.7.1.2.1`'s, and taking the same shape is the
point rather than a coincidence.** That leaf settled the WORKFLOW registry —
site-wide configuration, keyed without a tenant, whose write admitted on
enrolment — as a site-operator capability, following
`docs/decisions/2026-09-09_site-operator-authority.md`. The policy library is the
same kind of object. DOC-0066 required the two to be decided consistently or the
difference stated; they are consistent, and this sentence is the record of it.

⚠️ **The registrar-holds-the-grant question is NOT decided here.** `policy.rs`
routes it to `SIGNOFF-REPAIR.9.1` by name and this record does not pre-empt it: a
site capability to register and a rule about whose authority may be named are two
different questions, and the second is a semantic one about policy ownership.

## `policy_publications` and `deployment_assignments`, deferred here by name

DOC-0029 deferred exactly these two and this record discharges that deferral:

| Table | Verdict | Derivation |
| --- | --- | --- |
| `policy_publications` | **tenant-owned** — it joins the lifecycle | A publication is staged from a proposal, a decision, an approval and a projection; `publications::stage` reads the proposal first. Its tenant is the proposal's |
| `deployment_assignments` | **tenant-owned by its PUBLICATION, not by its target** | An assignment binds a publication to a `deployment_targets` row, and DOC-0029 rules a target *site-wide by design* — a deployment host belongs to the site operator. ⭐ The assignment crosses that boundary, and the tenant comes from the side that has one |

## The rejected alternatives

1. **Tenant-scope all ten, `policy_versions` included.** Rejected: it makes a
   governance library unreadable by the tenants it governs, and `owning_authority`
   already carries an ownership model that a tenant column would contradict.
2. **Leave all ten site-wide and gate every write on site-operator authority.**
   Rejected on the measurement: the lifecycle's writes ALREADY derive and enforce
   a tenant. Promoting them to site acts would discard a binding the code
   performs, and would mean a tenant could not deliberate its own policy adoption
   without an operator.
3. **Bind the reads and leave the writes.** Rejected for `policy_versions`, where
   it is `.6.1.5`'s named trap. Accepted for the lifecycle, where it is not a
   trap, because the write already knows the answer.
4. **One decision for the whole chain.** Rejected — it is what made this leaf look
   like one question for two years of commits. A registry and its lifecycle are
   different objects with different owners.

## What this decision does NOT do

It performs no repair. ⛔ 47 SQL sites, ten tables, two HTTP surfaces and the MCP
read are far more than one leaf, and `SIGNOFF-REPAIR.11.13`'s rule is that a child
without its own acceptance is not an owner. The work is split into children under
`.6.1.5`, each carrying one:

| Child | Owns |
| --- | --- |
| `.6.1.5.1` | `record_approval`'s discarded tenant — the weakest link, and a control-surface consequence |
| `.6.1.5.2` | the lifecycle migration and the write sites: store the tenant the write already derives |
| `.6.1.5.3` | the lifecycle reads, HTTP and MCP, bound with a foreign tenant's row proved absent from both |
| `.6.1.5.4` | `policy_versions` as a site-operator capability, following `.7.1.2.1` |
