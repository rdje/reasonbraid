# The policy lifecycle

A policy does not go from written to enforced in one step. It is **proposed**
against a deliberation thread, **decided** by a frozen electorate, **approved**,
**projected** into a target's own dialect, and **published**. Once it is live,
what actually happens comes back: **drift** between what was published and what a
target is running, **outcomes** that record real effects, and **reviews** that
those outcomes and drift observations schedule automatically.

This chapter covers the stages that had no chapter of their own. Approval and
publication are in [Authority](authority.md) and the publish verbs are in
[The CLI](cli.md); [Site authority](site-authority.md) covers who may hold the
grants publication requires.

## The stages, in order

```text
  proposal ──▶ decision ──▶ approval ──▶ projection ──▶ publication
                                                            │
                                                            ▼
                                                       deployment
                                                    (target + wave)
                                                            │
                                                            ▼
                                                        receipt
                                                 (the observed digest)
                                                            │
                          ┌─────────────────────────────────┤
                          ▼                                 ▼
                        drift                            outcomes
                          │                                 │
                          └──────────▶ reviews ◀────────────┘
```

A review is never scheduled by hand: `POST /v1/policy-reviews/schedule` derives
the due reviews from the drift and outcome rows that already exist.

## The routes

```text
GET    /v1/policies/{policy_id}/{version}/impact   the impact map for one version
POST   /v1/policy-proposals                        register a proposal (draft)
GET    /v1/policy-proposals                        the proposals, newest first
POST   /v1/policy-decisions                        record a decision (draft → decided)
GET    /v1/policy-decisions                        the decisions
POST   /v1/policy-projections                      project a resolved set for a target
GET    /v1/policy-projections                      the projections
POST   /v1/deployments                             assign a publication to a target
GET    /v1/deployments                             the assignments, desired vs observed
POST   /v1/deployments/{target_id}/{publication_id}/receipt   attest the observed digest
POST   /v1/policy-drift                            record one drift observation
GET    /v1/policy-drift                            the drift observations
POST   /v1/policy-outcomes                         record one outcome
GET    /v1/policy-outcomes                         the outcomes
POST   /v1/policy-reviews/schedule                 evaluate and create the due reviews
GET    /v1/policy-reviews                          the reviews
POST   /v1/policy-reviews/{review_id}/done         the due → done transition
```

## Who may call them

Every route on this page admits an **enrolled principal** and binds the result to
that principal's own tenant. There is no separate grant: enrolment is the
admission, and the tenant is taken from the enrolment rather than from the
request.

- An unenrolled principal is refused — `an unenrolled principal registers no
  proposal`, `… reads no proposals`, and the equivalent for each verb.
- Every read is filtered to the caller's tenant. Before
  `SIGNOFF-REPAIR.6.1.5.3` the list verbs returned **every** tenant's governance
  trail to any enrolled caller; they now pass the caller's tenant into the query
  itself.

⚠️ **The grant requirement begins at publication, not here.** Recording a
decision or an outcome takes enrolment; *staging, publishing and marking a
publication effective or failed* require an `owning_authority` the caller holds
(`SIGNOFF-REPAIR.9.2.1.2`). So this trail records what a tenant decided — it is
not itself the authority to put a policy into force.

⚠️ **The impact map is the one site-level read.** `policy_versions` carries no
tenant column: the policy register is a deployment-wide register whose rows are
owned by an `owning_authority`, not by a tenant. `GET
/v1/policies/{policy_id}/{version}/impact` therefore checks enrolment and then
reads by `(policy_id, version)` alone. That is deliberate and matches the
schema; the lifecycle rows above are the tenant-scoped ones.

Send the principal as a header, as everywhere else in the dev profile:

```bash
curl -s localhost:4310/v1/policy-proposals \
  -H 'x-reasonbraid-principal: hpr_0192…'
```

## The impact map

`GET /v1/policies/{policy_id}/{version}/impact` returns one entry per clause,
each carrying the version's declared applicability and non-applicability:

```bash
curl -s "localhost:4310/v1/policies/pol_retention/2.1.0/impact" \
  -H 'x-reasonbraid-principal: hpr_0192…'
```

```json
[
  {
    "clause_id": "c1",
    "statement": "evidence is retained for 90 days",
    "applicability": ["tenant:*", "resource_kind:evidence"],
    "non_applicability": ["region:eu-restricted"]
  }
]
```

⛔ **This is derivable coverage, never an achievement claim.** It is the clauses
multiplied by what the version *declares* it applies to. It does not say the
policy is being obeyed anywhere, and nothing on this page measures compliance.

## Proposals

`POST /v1/policy-proposals` registers the draft stage: a reference to the policy
version, and the deliberation thread the proposal belongs to.

```bash
curl -s -X POST localhost:4310/v1/policy-proposals \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "proposal_id": "prp_0192…",
        "policy_id": "pol_retention",
        "policy_version": "2.1.0",
        "thread_id": "thr_0192…"
      }'
```

```json
{
  "proposal_id": "prp_0192…",
  "policy_id": "pol_retention",
  "policy_version": "2.1.0",
  "thread_id": "thr_0192…",
  "status": "draft"
}
```

Both the policy version and the thread must already exist; a proposal cannot
reference something that does not.

## Decisions

`POST /v1/policy-decisions` performs the **draft → decided** transition. It
carries the decision rule, the **frozen electorate** the decision was taken
under, and the id of the verdict event in the thread:

```bash
curl -s -X POST localhost:4310/v1/policy-decisions \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "decision_id": "dec_0192…",
        "proposal_id": "prp_0192…",
        "rule": "supermajority",
        "electorate": {"members": ["rol_a", "rol_b", "rol_c"]},
        "verdict_event_id": "evt_0192…"
      }'
```

The electorate is stored as given, so the decision keeps the membership it was
taken under even after the roster changes. An **empty** electorate is refused,
and so is a proposal that is not in the `draft` stage — a decision cannot be
recorded twice against the same proposal.

## Projections

`POST /v1/policy-projections` resolves a policy set for one target and renders
it through the hermetic compiler. The stored row keeps the rendered bytes, their
digest, and anything the compiler declared it **could not represent** in that
target's dialect:

```bash
curl -s -X POST localhost:4310/v1/policy-projections \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "projection_id": "prj_0192…",
        "target": "gateway-v2",
        "resolution": {"subject": {"tenant_id": "ten_0192…"}}
      }'
```

```json
{
  "projection_id": "prj_0192…",
  "target": "gateway-v2",
  "digest": "sha256:…",
  "bytes": "…",
  "unrepresentable": [
    {"clause_id": "c4", "reason": "no equivalent construct in gateway-v2"}
  ],
  "resolved_policies": [{"policy_id": "pol_retention", "version": "2.1.0"}]
}
```

⛔ **`unrepresentable` is part of the record, not a warning to be discarded.** A
projection that could not express a clause says so, and staging a publication
reads `resolved_policies` to require that the publication carries the policy its
proposal was approved for.

⚠️ `resolved_policies` may be absent on rows written before `migrations/0082`.
Absent means *the set was never recorded* — it does **not** mean the set was
empty — and staging fails closed on it, naming which of the two it is.

## Deployment and receipts

A publication does not reach a target by itself. `POST /v1/deployments` assigns
one publication to one target in a **canary wave**, recording the *desired* pair
— the ref and its digest:

```bash
curl -s -X POST localhost:4310/v1/deployments \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "target_id": "gateway-v2",
        "publication_id": "pub_0192…",
        "wave": 1,
        "desired_ref": "refs/heads/main",
        "desired_digest": "sha256:…"
      }'
```

The target must be registered — `/v1/deployment-targets`, in
[Authority](authority.md) — the
publication must exist, and — the one that matters — **the publication must be
effective**. Assigning one that is not is refused; a target is never pointed at
something the deployment has not put into force.

The target then reports back. `POST
/v1/deployments/{target_id}/{publication_id}/receipt` is the **attestation**: the
digest the target says it is actually running, and the state it reached.

```bash
curl -s -X POST "localhost:4310/v1/deployments/gateway-v2/pub_0192…/receipt" \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{"observed_digest": "sha256:…", "observed_state": "applied"}'
```

`observed_state` is one of `pending`, `applied`, `waived` or `rejected`; anything
else is refused with the vocabulary. A receipt for an assignment that was never
made is refused, as is a malformed digest.

`GET /v1/deployments` returns the assignments with **both halves side by side**:

```json
[
  {
    "target_id": "gateway-v2",
    "publication_id": "pub_0192…",
    "wave": 1,
    "desired_ref": "refs/heads/main",
    "desired_digest": "sha256:aaa…",
    "observed_digest": "sha256:bbb…",
    "observed_state": "applied"
  }
]
```

⭐ **That pair is what drift is computed from.** A row whose `observed_digest`
differs from its `desired_digest` — or whose `observed_digest` is still `null`
because no target has reported — is exactly the situation the next section
records, and `unauthorized_modification` and `pending_rollout` are the two
categories for those two shapes.

⛔ **A receipt is the target's claim, not the deployment's verification.** The
server stores the digest the caller reported; nothing here re-reads the target to
confirm it. The receipt makes the disagreement *visible* and recordable — it does
not adjudicate it.

## Drift

`POST /v1/policy-drift` records one categorized observation that a target is not
running what was published: the desired digest, the observed digest (absent when
nothing could be read), and a category from a closed vocabulary.

```bash
curl -s -X POST localhost:4310/v1/policy-drift \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "drift_id": "drf_0192…",
        "target_id": "gateway-v2",
        "publication_id": "pub_0192…",
        "category": "pending_rollout",
        "desired_digest": "sha256:…",
        "observed_digest": "sha256:…"
      }'
```

The six categories are `expected_override`, `pending_rollout`,
`unauthorized_modification`, `unsupported_target`, `unverifiable_load` and
`stale_agent_incarnation`. Anything else is refused, and the refusal lists the
vocabulary.

## Outcomes

`POST /v1/policy-outcomes` records a real effect of a live publication, and may
name the review trigger that effect should raise:

```bash
curl -s -X POST localhost:4310/v1/policy-outcomes \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "outcome_id": "out_0192…",
        "publication_id": "pub_0192…",
        "kind": "incident",
        "review_trigger": "adverse_threshold",
        "note": "retention deleted evidence a dispute still needed"
      }'
```

The six kinds are `observation`, `measurement`, `incident`, `complaint`,
`reversal` and `unintended_effect`. `review_trigger` is optional; when present it
must be one of the seven triggers below, and an unknown one is refused with the
list.

## Reviews

`POST /v1/policy-reviews/schedule` does not take a body. It derives the reviews
that are **due** from rows that already exist, for the caller's tenant only:

- every outcome carrying a `review_trigger`, under that trigger;
- every drift observation, under the `drift` trigger;
- every correction whose operation is `waiver`, under `repeated_waiver`.

The seven triggers are `elapsed_interval`, `dependency_change`,
`adverse_threshold`, `external_standard_change`, `repeated_waiver`, `drift` and
`evaluator_regression`.

The pairs are deduplicated, and a `(publication, trigger)` that already has a
`due` review is skipped — so **the verb is idempotent** and can be run on a timer
without accumulating duplicates. It returns the reviews it created:

```bash
curl -s -X POST localhost:4310/v1/policy-reviews/schedule \
  -H 'x-reasonbraid-principal: hpr_0192…'
```

```json
[
  {
    "review_id": "rev_0192…",
    "publication_id": "pub_0192…",
    "trigger": "drift",
    "status": "due"
  }
]
```

`POST /v1/policy-reviews/{review_id}/done` performs the **due → done**
transition once the review has been carried out, and `GET /v1/policy-reviews`
lists the caller's tenant's reviews with their current status.

## What this machinery does not claim

⛔ **Governance is qualified as machinery, not as binding use.** The Phase 6 gate
(G3) was met for the mechanism — the stages exist, the transitions are typed, the
records are durable and tenant-bound — and it remains **blocked for binding
use**. Nothing on this page enforces a policy on a running system; it records the
decisions, projections and observations from which enforcement would be argued.

⛔ **A recorded outcome is a claim by its author.** The server checks that the
kind and trigger are in the vocabulary and that the publication exists. It does
not verify that the effect described actually occurred.

⛔ **The review trail proves a review was marked done, not that one happened.**
The `due → done` transition is a record of an assertion, and no evidence is
required to make it.
