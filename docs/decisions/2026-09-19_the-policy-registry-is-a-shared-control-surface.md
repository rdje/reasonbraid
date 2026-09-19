---
answers:
  - Which of the thirty site-global tables did the evidence decision never name?
  - Is a table with no tenant_id column necessarily an ownerless table?
  - What is the difference between shared evidence and a shared control surface?
  - Why does binding the read not repair the policy registry?
  - Can one tenant change another tenant's default deliberation workflow?
  - How many evaluation tables are there, and why did the earlier record say six?
  - Is .3.2's closed set of six site actions still the closed set?
  - Which of the seventeen tables are a live defect and who owns each?
---
# The policy registry is a shared CONTROL surface; the evidence decision's remedy does not reach it

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.7.1.2`
- **Date:** 2026-09-19
- **Extends:** `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md` (DOC-0029, `SIGNOFF-REPAIR.11.14`)
- **Defers to by name:** `SIGNOFF-REPAIR.6.1.5`

## Why this record exists at all

`SIGNOFF-REPAIR.7.1.1` censused the WRITE side and published a population: **42 of
72 mutating routes write a table carrying no tenant dimension, 33 of them
admitted on enrolment alone, across 30 distinct tables.** It deliberately judged
none of them and routed the adjudication here.

⛔ **It should first have asked whether the question was already answered, and it
was — for thirteen of the thirty.** DOC-0029 is `done` and decides the site-global
data model: the evidence chain is content-addressed **by design**, no table is
decided tenant-owned, and what ROADMAP §16.8 requires is that the DISCLOSURE
decision names the tenant. `docs/CLAIM_VERIFICATION.md` leg 2 says the cheapest
oracle is the project's own history and that an earlier ruling wins unless the
difference is named.

⭐ **The difference is named, and it is a number.** DOC-0029 decided **twelve**
tables. A census derived from the producers finds **thirty**, and **seventeen are
not named anywhere in that record**:

```bash
python3 -B scripts/census_shared_registry_writes.py --json   # the 30, under `identity only`, arm `walk`
grep -oE '`[a-z_]+`' docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md
```

⛔ **And the cause is a shape this repair keeps finding.** DOC-0029's census was
*"of the twelve policy, evaluation, deployment and evidence tables"* — a
population scoped by **family name**, taken as given. A population derived from
the producers (every mutating route, every table its call walk writes, every
migration) is thirty. A family-scoped census answers a question about the family,
not about the surface.

## The instrument

`scripts/census_registry_read_reach.py`, which supplies the two facts an
adjudication needs and that a second reading cannot be trusted to reproduce:

```bash
python3 -B scripts/census_registry_read_reach.py             # 40 tables, readers and derivations
python3 -B scripts/census_registry_read_reach.py --check      # the pinned set
python3 -B scripts/census_registry_read_reach.py --self-test
```

It reuses `census_get_route_binding.tenant_dimensioned_tables()` and
`census_shared_registry_writes.collect()` rather than re-deriving either, so
there is one answer to *does this table carry a tenant dimension*, not two that
can drift. The set is pinned in `.doctrine/registry_read_reach_baseline.tsv`.

⛔ **It classifies nothing.** Whether a row is evidence or control is a judgement
about what a decision MEANS; a script that guessed it would repeat `.11.14`'s
defect one layer down. The census reports readers and derivations; the verdicts
are below.

## The two facts the adjudication turns on

### Fact 1 — a column-less table is not necessarily an ownerless one

⭐ **This is the correction DOC-0029 most needs, and it is not a disagreement with
its reasoning — it is a limit on its reach.** That record's argument is about
CONTENT-ADDRESSED rows: `UNIQUE (original_locator, expected_digest)` makes two
tenants share one row *by construction*, so no scalar column can hold an owner.
That argument is correct and it is why `evidence_snapshots` and `derivations` get
no `tenant_id`.

⛔ **It never reached a table whose primary key is a FOREIGN KEY into a
tenant-dimensioned table.** One join recovers the tenant there, and four of the
seventeen are exactly that shape:

| Table | Derivation the census walks | Recovers |
| --- | --- | --- |
| `agent_profiles` | `agent_profiles.role_id → agent_roles` | `agent_roles.tenant_id` |
| `profile_versions` | `profile_versions.role_id → agent_roles` | `agent_roles.tenant_id` |
| `quota_events` | `quota_events.quota_id → usage_quotas` | `usage_quotas.tenant_id` |
| `recruitment_responses` | `recruitment_responses.call_id → recruitment_calls` | `recruitment_calls.tenant_id` |

So "site-global by data model" — the phrase `tenant_dimensioned_tables()`'s own
doc comment uses, correctly, about the schema — must never be read as
"ownerless". Nine of the forty site-global tables carry a derivation.

### Fact 2 — who reads it decides what it is

**The question, stated so the classification is not a matter of taste:** does the
table only describe its writer, or does another tenant's resolution, routing or
policy decision READ it?

- A row nobody else's decision consults is **shared evidence**, and DOC-0029's
  remedy fits it exactly: bind the READ.
- A row that binds another tenant's outcome is a **shared CONTROL surface**, and
  DOC-0029's remedy does not touch it. ⛔ A control surface's problem is the
  WRITE. Binding its read would hide rows from their own author — the trap
  `.6.1.5` already states in full — while leaving the defect entirely in place.

## The decision, per table — all seventeen

### A. Tenant-owned by derivation — four tables, no migration owed

⛔ **None of these four gains a `tenant_id` column**, for a different reason from
DOC-0029's: not because no column could hold the owner, but because a column
would DUPLICATE one the schema already derives.

| Table | Verdict | The reading path |
| --- | --- | --- |
| `quota_events` | **tenant-owned, and the read is ALREADY bound** | `quota::check_in_tx` selects `count(*) … WHERE quota_id = $1`, and that `quota_id` comes from the statement immediately above it: `SELECT quota_id … FROM usage_quotas WHERE tenant_id = $1 AND scope_kind = $2 AND scope_id = $3`. ⚠️ The census marks the second statement `NO tenant predicate` and it is right to — the binding is upstream, in a different statement, which is precisely why a per-statement predicate scan must never be read as a verdict |
| `recruitment_responses` | **tenant-owned by derivation**; the read is bound by its call | `recruitment::responses` selects `WHERE call_id = $1`; `recruitment_calls.tenant_id` is `NOT NULL REFERENCES tenants`. The control `a_response_is_bound_to_the_calls_own_tenant` already asserts it |
| `agent_profiles` | **tenant-owned row, site-wide read BY DESIGN, bound at FIELD level** | see below |
| `profile_versions` | same | see below |

⭐ **The directory pair is the interesting one, and it is a fourth shape neither
record had.** `api::directory_match` and `api::directory_presence` read
`profile_versions JOIN agent_profiles` over `node_presence` with **no tenant
predicate on the rows** — deliberately. The product premise is that an authorized
caller asks a durable network a question *without knowing who is online*; a
directory that returned only one tenant's roles would not be a directory. What
binds the disclosure is not the row set but the FIELDS: both handlers derive a
`ReaderClass` (`Full` for a tenant admin, else `Tenant` or `Network`), clamp the
request's scope against it, and pass every profile through
`profiles::filter_profile(&profile, reader)`.

⛔ **So §16.8 is satisfied here, not violated — by field-level disclosure rather
than by row-level scoping**, and that is a design position this project had taken
in code without ever writing down. It is written down now. It is consistent with
`docs/decisions/2026-09-18_node-presence-is-read-by-its-own-tenant.md`, which
bound `GET /v1/nodes/presence` — that route answers about ONE named node, where a
foreign answer is an existence oracle; the directory answers about the network,
where the set is the point and the fields are the secret.

### B. Within DOC-0029's decision — one table the record undercounted

| Table | Verdict | Why |
| --- | --- | --- |
| `evaluation_trial_results` | **site-wide by design, gated by SITE-OPERATOR grants** — DOC-0029's `evaluation_*` verdict, unchanged | It is read only by `evaluation::list_trial_results`, keyed on its trial |

🔎 **DOC-0029 says "the six `evaluation_*` tables". There are SEVEN.** It names
`evaluation_corpora`, `evaluation_runs`, `evaluation_trials`,
`evaluation_calibrations`, `evaluation_gates`, `evaluation_gate_results` — and
misses `evaluation_trial_results`, which `migrations/0034_evaluation_trials.sql`
creates in the same file as `evaluation_trials`. ⛔ A record that counted tables
per FILE would see six; the seventh is the second `CREATE TABLE` in a file named
for the first. The verdict is unchanged and extends to it by the same reasoning,
and `.8.2`'s attached clause — *they admit any enrolled principal today* — extends
to it too.

### C. Shared journal, disclosure-only — two tables, DOC-0029's remedy fits

| Table | Verdict | The reading path |
| --- | --- | --- |
| `routing_resolutions` | **site-wide row, TENANT-BOUND READ owed** | Written by `routing::record_resolution` as an append-only audit row carrying `caller` and `surface`. Read by exactly one function — `routing::list_resolutions`, `ORDER BY resolved_at DESC` with **no predicate of any kind**. ⛔ `routing::resolve` does NOT read it; the routing decision reads `routing_rules` and `workflow_profiles`, so this table binds nobody's outcome |
| `routing_recommendations` | **site-wide row, TENANT-BOUND READ owed** | Written by `routing::record_recommendation`; read only by `routing::list_recommendations`, no predicate. `evidence_ref` and `case_class` describe who was routed where |

⚠️ **Stated at its real width:** this is a DISCLOSURE path, exactly the shape
DOC-0029 measured on `snapshots::stale` — any enrolled principal in any tenant
reads every tenant's routing trail. It is not a write defect and not an oracle.
Owner: `SIGNOFF-REPAIR.7.1.2.2`.

### D. Shared CONTROL surfaces — ten tables

⛔ **These are the seventeen's sharp end, and DOC-0029's remedy reaches none of
them.** Nine are the policy lifecycle; the tenth is the workflow registry.

#### D1 — the policy lifecycle is ONE control chain, not nine tables

`policy_versions` → `policy_proposals` → `policy_decisions` → `policy_approvals` →
`policy_projections` → `policy_publications` → `policy_drift` / `policy_outcomes`
/ `policy_corrections` → `policy_reviews`.

Every link is keyed on an opaque string id with no tenant dimension; every write
admits on enrolment alone; and **each stage READS the previous stage's row in
order to decide**. Two sites make that concrete:

- `publications::stage` reads `policy_proposals.status`, then
  `policy_decisions.proposal_id`, then `policy_approvals.proposal_id`, then
  `EXISTS(policy_projections)`. It checks that the decision and approval belong to
  the named PROPOSAL — `ForeignRecord` is a real, typed refusal — and it never
  checks that the proposal belongs to the caller. ⛔ So the chain is internally
  consistent and externally unowned.
- `reviews::schedule_reviews` reads `policy_outcomes`, `policy_drift` and
  `policy_corrections` **with no predicate at all**, across every publication in
  the site, and INSERTS a `policy_reviews` row per `(publication, trigger)` pair.
  ⛔ One tenant's drift row schedules a review on another tenant's publication.

And the registry's own write is unowned by construction. `policy::register`
requires `owning_authority` to be a live grant — the comment in that function is
explicit that it does **not** require the registrar to hold it:

> ⛔ What this does NOT decide: whether the owning authority must be a grant the
> REGISTRAR holds. […] that binding is a semantic question and stays
> `SIGNOFF-REPAIR.9.1`'s.

With `PRIMARY KEY (policy_id, version)` over a site-global namespace, that makes
the registry first-come: any enrolled principal may take `(policy_id, version)`
and attribute it to any live grant in the site.

| Table | Verdict | The decision that reads it |
| --- | --- | --- |
| `policy_versions` | **shared control surface** | `policy::resolve` — the seven-step fail-closed resolution |
| `policy_proposals` | **shared control surface** | `publications::stage`, `lifecycle::record_decision`, `lifecycle::record_approval` |
| `policy_decisions` | **shared control surface** | `publications::stage`, `lifecycle::record_approval` |
| `policy_approvals` | **shared control surface** | `publications::stage` |
| `policy_projections` | **shared control surface** | `publications::stage`, `projections::load` |
| `policy_drift` | **shared control surface** | `reviews::schedule_reviews` (unpredicated) |
| `policy_outcomes` | **shared control surface** | `reviews::schedule_reviews` (unpredicated) |
| `policy_corrections` | **shared control surface** | `reviews::schedule_reviews` (unpredicated) |
| `policy_reviews` | **shared control surface** | `reviews::schedule_reviews`, `reviews::mark_done` |
| `policy_publications` | *already deferred to `.6.1.5` by DOC-0029* | `deployments::assign`, `corrections::publication_exists`, `publications::mark_effective` / `mark_failed` |

⭐ **This is how `.6.1.5` is answered: by re-scoping it, not by pre-empting it.**
DOC-0029 deferred `policy_publications` and `deployment_assignments` to `.6.1.5`
**by name**. The chain shows that deferral was right and too narrow — the surface
`.6.1.5` must decide is a **registry of ten tables**, not `policy_versions` and
its six SQL sites. `.6.1.5`'s own text already forbids the shortcut ("Do not
answer this by adding a filter to one read"); this record supplies the population
that shortcut would have missed, and `.6.1.5` remains the owner of the design
decision. ⛔ This record does NOT decide whether a policy belongs to a tenant.

#### D2 — `workflow_profiles`: a live cross-tenant control defect

🔴 **The sharpest finding in the seventeen, and it is not a disclosure question.**

- `workflows::resolve(pool, profile_id)` selects
  `SELECT version, steps FROM workflow_profiles WHERE profile_id = $1 ORDER BY version DESC LIMIT 1`
  — the highest version, site-wide, with no `built_in` filter and no tenant predicate.
- `workflows::register(pool, profile_id, steps)` computes
  `COALESCE(MAX(version), 0) + 1` for **any** `profile_id` and inserts. It accepts
  the id of an existing profile, including one of the eight §13.1 built-ins.
- `api::register_workflow_profile` (`POST /v1/workflow-profiles`) admits on
  `reader_tenant(&state.pool, &principal).await?.is_some()` — **enrolment alone**.
- `workflows::resolve` is called from `api::create_thread` (`POST /v1/threads`)
  and `api::create_thread_auto` (`POST /v1/threads/auto`), and
  `DEFAULT_PROFILE_ID` is `quick_advice` — the profile a bare thread runs.

⛔ **So any enrolled principal in any tenant can register `quick_advice` at a
higher version and change the steps every other tenant's next bare thread will
execute.** The vocabulary is closed (`validate_steps` admits only the thirteen
step kinds and requires a terminal last), so this is not a capability escape — it
is a control-plane override: the attacker chooses the deliberation shape, and can
for instance drop `blind_solicit` or `critique` from every tenant's default.

⚠️ **SOURCE-MEASURED; runtime reproduction is owed before the repair, not after**
— `.7.2.2`'s standard, which observed the unrepaired client actually dial. This
record claims the mechanism at four sites; it does not claim an end-to-end
observation. Owner: `SIGNOFF-REPAIR.7.1.2.1`, whose acceptance requires the RED
control first.

⭐ **And this instance is not new — it was known by name and its significance was
not.** `.7.1`'s attached clause 2 already cited *"`POST /v1/workflow-profiles` on
enrolment alone"* as one of the two examples proving `.3.2`'s closed set was
designed without a census. It was read as an admission gap. It is a
control-plane takeover, and the difference is that nobody had read
`workflows::resolve` next to `workflows::register`.

## `.3.2`'s closed set of six site actions — SUPERSEDED IN SCOPE

`SIGNOFF-REPAIR.3.2` pinned six: `registry_inspect`, `adapter_allow`,
`adapter_revoke`, `region_declare`, `region_pair`, `region_unpair`.

**Upheld as a set of ACTIONS; superseded as THE set of site actions.** Each of the
six remains correct, explicitly issued and audited, and nothing here reopens the
site-authority service. ⛔ But the six cover **none** of the thirty site-global
tables, and this record names ten of them as control surfaces that bind other
tenants' outcomes. A seventh and eighth action — over the policy registry and the
workflow registry — are the missing members, and naming them is `.6.1.5`'s and
`.7.1.2.1`'s respectively, because an action is only definable once the surface
it guards has been decided.

⛔ **The general lesson is `.7.1`'s clause verbatim, now with its evidence:** a
rigorous closed set is still the wrong closed set if its scope was never
measured.

## What this record does NOT say

- It does not say a policy belongs to a tenant. That is `.6.1.5`'s to decide, and
  this record hands it the population instead of the answer.
- It does not claim the workflow takeover has been observed at runtime. It has
  been measured at four source sites; `.7.1.2.1` owes the control.
- It does not re-open the thirteen DOC-0029 already decided, except to record
  that the evaluation family is seven and not six.
- It does not repair anything. Every finding names an owner, and the owners are
  leaves, not this file.

## Owners

| Finding | Owner |
| --- | --- |
| `workflow_profiles` — the cross-tenant default-workflow override | `SIGNOFF-REPAIR.7.1.2.1` |
| `routing_resolutions` / `routing_recommendations` — the unpredicated journal read | `SIGNOFF-REPAIR.7.1.2.2` |
| the nine-table policy lifecycle — the design decision | `SIGNOFF-REPAIR.6.1.5`, re-scoped by this record |
| `evaluation_trial_results` — the seventh evaluation table's site-operator gate | `SIGNOFF-REPAIR.8.2`, by its existing attached clause |
| the directory's field-level disclosure — now stated rather than implied | this record; `docs/book/src/site-authority.md` |
