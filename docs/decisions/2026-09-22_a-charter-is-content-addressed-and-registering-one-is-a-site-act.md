---
answers:
  - Where do ROADMAP §4.1's allowed decision rules and approval thresholds live?
  - Why is a charter content-addressed rather than carrying a version number?
  - Why may a tenant not register its own charter?
  - Why are there seven decision-rule wire names and not nine?
  - Which families take an approval threshold, and why only those?
  - What does the read path answer for a boundary issued before the charter store existed?
---
# A charter is content-addressed, and registering one is a site act

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.2.1`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §4.1 (the governance charter), §4.4 (the enrollment boundary as
  ceiling), §13.3 (decision rules);
  `docs/decisions/2026-09-22_a-decision-rule-is-a-charter-scoped-vocabulary-and-never-ships-without-its-tally.md`
  (the contract this implements);
  `docs/decisions/2026-09-09_site-operator-authority.md` (the shape the gate takes);
  `SIGNOFF-REPAIR.9.2.1.3.1` (the server-derived digest rule)

## Context

§4.1 says each tenant has a *versioned `GovernanceCharter`* defining, among other
things, *allowed decision rules and approval thresholds*. Before this leaf the charter had
a NAME and no CONTENT: `enrollment_boundaries.charter_digest` was a `TEXT` label nothing
resolved — the shipped values are `dev-charter-digest`, `dev-charter-000` and
`fixture-charter` — and §13.3's seven rule families existed only as prose.

This is the prerequisite `.11.4.7.2.1.2` found: without it a `decision_rule` create field
would validate against nothing and degrade into a global enum, which §4.1 does not say.

## Decision

### 1. The store is CONTENT-ADDRESSED, and that is what makes it *versioned*

`governance_charters` is keyed by the digest of its own canonical content. Changing the
allowed set does not edit a charter — it writes a different row, and the old one stays
readable forever.

⭐ **This is what discharges the acceptance's hardest clause** — *changing the set must not
rewrite what past decisions were taken under* — **without any bookkeeping to get wrong.**
`reasonbraid_core::authority` already folds `charter_digest` into every authorization
decision record, so a decision taken on Monday names Monday's charter by digest and that
row is immutable by construction. The alternative — a `version` column with
`valid_from`/`superseded_at` — makes the same guarantee depend on range queries nobody
would re-derive.

⛔ **`tenant_id` is INSIDE the digested content.** Two tenants that allow the same rules
hold two rows, because a charter is a tenant's governance document and not a shared
template. The `snapshot_objects` precedent (identical bytes = one object) deliberately does
not apply.

⚠️ **Re-registering byte-identical content is IDEMPOTENT rather than a duplicate refusal.**
The row's identity is its content, so a second write asserts nothing new. That differs from
`evaluation_corpora`, where `(corpus_id, version)` is a first-come namespace and a duplicate
is a real conflict — the difference is that one key is chosen by a caller and the other is
derived from the thing itself.

### 2. Registration is a SITE act, by derivation rather than caution

§4.1 makes the charter the document that **constrains** a tenant. §4.4 makes the enrollment
boundary that names its digest a **root/parent-granted ceiling**. ⇒ A tenant that could
rewrite its own allowed decision rules would hold the ceiling it is bound by, which is the
escalation the boundary exists to prevent.

So registration takes the `charter_register` site capability, in the shape
`docs/decisions/2026-09-09_site-operator-authority.md` defines and that
`workflows::register_profile`, `policies::register_policy` and the `evaluation` family
already carry — authorization, effect and audit as one ordered transaction.

### 3. Seven wire names, not nine

§13.3's second family is *simple **or** supermajority of a defined electorate* and its
fifth is *role-weighted **or** chambered approval*. Splitting either into two wire names
would be this project choosing a vocabulary the roadmap did not state. ⭐ **And the
threshold is what the roadmap already supplies for the distinction** — which is exactly why
§4.1 names *allowed decision rules **and** approval thresholds* in one breath: the
thresholds parameterize the families rather than multiplying them.

### 4. Only `majority_of_electorate` takes a threshold, and it must have one

| family | threshold | why |
| --- | --- | --- |
| `majority_of_electorate` | **required**, in `(0.5, 1.0]` | it is the one family whose bar is a number; `0.5` alone is not a majority |
| `unanimity` | refused | unanimity IS `1.0`; a threshold on it is a contradiction, not a stricter rule |
| `advisory_synthesis` | refused | §13.3 defines it as having *no binding decision*, so there is nothing to gate |
| the other four | refused | their bar is defined by the family, or (for `role_weighted`) by a weighting §13.3 assigns to the charter |

⚠️ **`role_weighted`'s weighting has no schema and that is stated rather than implied.** The
family can be allowed; its weights have no home yet. Inventing one here would be a
vocabulary decision this leaf has no roadmap sentence for.

### 5. The read path resolves through the BOUNDARY, and FAILS CLOSED

`GET /v1/decision-rules/{rule}` answers *may this tenant decide under this rule* by
resolving the caller's tenant → its **active** enrollment boundary → the charter digest that
boundary was ISSUED under. ⛔ Not by `tenant_id` directly: §4.4 makes the boundary the
ceiling, so the charter a tenant is bound by is the one its issuer named, never the newest
one anybody registered for that tenant id.

⛔ **A boundary whose digest resolves to nothing answers *the question cannot be answered*.**
Every boundary issued before `migrations/0084` carries a label, so this is the live case, not
a hypothetical. A rule is never allowed by default.

⛔ **A refusal names the rule and never the charter's set.** Enumerating it would answer a
question the caller did not ask, and a unit control asserts that no other family's wire name
appears in the rendered refusal.

## Consequences

- ✅ **`.8.1.1` is unblocked for its charter-scoped validation.** It can now ask the charter
  whether a rule is allowed instead of accepting a string.
- ⚠️ **Nothing binds a thread to a rule yet, and the book says so.** A thread carries no
  `decision_rule` and a close still names its rule as free text beside a declared outcome.
  That pair lands in `.8.1.1`, because a rule nothing evaluates would be a declaration
  rather than a control — the decision `.11.4.7.2.1.2` took.
- ⚠️ **Existing deployments read *not answerable* until a charter is registered and a
  boundary reissued naming it.** Nothing breaks, because nothing consumed decision rules
  before this commit — but the fail-closed answer is a behaviour, not an absence.
- ⭐ **The site-authority capability table in the book gained two rows it was already owed** —
  `evaluation_record` and `gate_evaluate` shipped in `SIGNOFF-REPAIR.8.2.5` and were
  documented only in the evaluation chapter. Fixed in passing because this commit edits that
  exact table, and stated here so it is not mistaken for scope creep.

## What would make this wrong

- ⛔ If a charter must be amendable in place — a correction to a typo that should NOT create a
  new governance version — then content-addressing is too strict and an amendment path is
  owed. Nothing asks for that today, and §4.1's word is *versioned*.
- ⛔ If a tenant legitimately owns some charter provisions (its own conflict-of-interest list,
  say) while the root owns others, decision 2 is too coarse and the charter splits into a
  root-granted part and a tenant-granted part. §4.1 lists nine provision families and this
  leaf implements one; the split question returns with the others.
- ⛔ If `role_weighted` turns out to need its weights before `.8.1.1` can count anything, the
  gap named in decision 4 becomes a blocker rather than a note.

## Alternatives considered

1. **A `version` integer column with `valid_from`/`superseded_at`.** Rejected: it makes the
   *past decisions stay interpretable* guarantee depend on range queries, where the
   content-addressed form makes it true by construction and the decision record already
   carries the digest.
2. **Tenant-admin registration.** Rejected on decision 2's argument — it hands a tenant the
   ceiling it is bound by.
3. **Nine wire names, splitting *simple/supermajority* and *role-weighted/chambered*.**
   Rejected: the roadmap says seven families, and the threshold already carries the first
   split.
4. **Store the rules on `enrollment_boundaries` directly.** Rejected: the boundary is reissued
   for reasons that have nothing to do with governance rules, and every reissue would then
   silently restate the charter. Separating them is what lets one charter outlive many
   boundaries and one boundary name a charter that predates it.
