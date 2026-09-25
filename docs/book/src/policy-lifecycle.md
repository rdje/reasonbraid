# The policy lifecycle

A policy does not go from written to enforced in one step. It is **proposed**
against a deliberation thread, **decided** by a frozen electorate, **approved**,
**projected** into a target's own dialect, and **published**. Once it is live,
what actually happens comes back: **drift** between what was published and what a
target is running, **outcomes** that record real effects, and **reviews** that
those outcomes and drift observations schedule automatically.

This chapter covers the stages that had no chapter of their own. The grants
approval and publication require are in [Authority](authority.md) and the publish verbs are in
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
POST   /v1/policies/resolve                        resolve a policy set for one target
GET    /v1/policies/{policy_id}/{version}/impact   the impact map for one version
POST   /v1/policy-proposals                        register a proposal (draft)
GET    /v1/policy-proposals                        the proposals, newest first
POST   /v1/policy-decisions                        record a decision (draft → decided)
GET    /v1/policy-decisions                        the decisions
POST   /v1/policy-projections                      project a resolved set for a target
GET    /v1/policy-projections                      the projections
GET    /v1/policy-bundles/{manifest_digest}        a published bundle, verified on the way out
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

**A publication leaves `staged` exactly once** (`SIGNOFF-REPAIR.9.2.2`). Marking
it `effective` and marking it `failed` are both terminal, and each write now
carries the condition that the publication is still `staged`. When two arrive
together, the first to reach the row wins and the other is refused with the
stage it found:

```json
{
  "code": "invalid_command",
  "message": "publication `pb-2` is at stage `effective` — the transition does not apply"
}
```

Until the repair each verb read the stage, checked it, and then wrote without
that condition, so both could pass the check and the second overwrote the first.
Measured before the repair: both callers were told they had won, and the row
ended `effective` while still carrying the failed transition's reason. Two
related answers were corrected with it: a publication that does not exist (or
is another tenant's) is refused as *"publication `…` does not exist"*, where it
used to say *proposal*, and a failure of the database during a transition is
the server's `500`, where it used to be answered as though the publication did
not exist.

The same correction reaches every governance record (`SIGNOFF-REPAIR.9.2.3`).
Registering a proposal, recording a decision or an approval, and staging a
publication each look records up before writing, and each of those lookups used
to answer a database failure as *"… does not exist"* or *"… is not registered"*
for the record it was reading, and the approval's authority check as an invalid
proof. A caller told that could fairly re-create the record or give up on it. A
database failure is now `500 dependency_unavailable` everywhere on these routes,
and a missing decision is named as one (*"decision `…` does not exist"*, where it
read *"proposal `decision `…`` does not exist"*).

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

## Resolving a policy set

`POST /v1/policies/resolve` answers which clauses bind one target, given the policy
versions you name. It is a read: it records nothing, and any enrolled principal may
call it, because the library is shared.

```bash
curl -s -X POST localhost:4310/v1/policies/resolve \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "policies": [
          {"policy_id": "org-baseline", "version": "1.0.0"},
          {"policy_id": "project-x", "version": "1.0.0"}
        ],
        "target": {"layer": "project", "target": "prj-x"}
      }'
```

The answer names, for each clause, the policy that won it and the path it took,
and adds an explanation with one line per step:

```json
{
  "target": {"layer": "project", "target": "prj-x"},
  "resolved": [
    {"policy_id": "project-x", "version": "1.0.0", "clause_id": "c1",
     "statement": "the project requires the evidence gate",
     "path": ["authority: active grant", "lifecycle: draft",
              "applicability: matched", "precedence: the winner over the carriers"]}
  ],
  "explanation": ["loaded 2 policies: org-baseline, project-x", "step 1: …", "…"],
  "conflicts": []
}
```

The steps run in this order, and anything they refuse answers `400`, naming why.
Each refusal says what happened: a reference that is not registered says so, and a
store that could not answer is the server's `500`, never a refusal
(`SIGNOFF-REPAIR.9.1.7`). Both used to be reported as *"… already exists"*, the
second even during an outage.


1. **One version of each policy.** Naming a policy twice, or two of its versions,
   is refused.
2. **Registered and owned.** Every named version must be registered, and its
   owning grant must be live.
3. **Applicability.** A policy applies when one of its applicability selectors
   matches the target, or it has none, and none of its non-applicability selectors
   does. A `suspended` or `retracted` version never applies. The other lifecycle
   labels (`draft`, `active`, `superseded`, `deprecated`) are shown in each
   clause's path and not acted on: the label is the registrar's declaration, set
   once and never changed, and a policy is put into force by approval and
   publication, not by its label.
4. **Dependencies and conflicts, among the policies that apply.** Every dependency
   must apply here at the exact version it names, and no explicit conflict may
   apply here. A policy that does not apply needs nothing and satisfies nothing, so
   a conflict with it is moot.
5. **Precedence, among the policies that apply.** The `over` hints must not form a
   cycle of any length; the refusal names the cycle. A hint naming the policy itself
   does nothing.
6. **Waivers.** Each requested waiver must appear in some named policy's exception
   schema. A waiver does not yet change the result; see
   [the qualification review](qualification-review.md).
7. **Collisions.** Where applying policies carry the same clause id, the one that
   wins over all the others by precedence takes it; with no single winner, the
   resolution fails closed.

Each list in a policy has one entry shape, checked at registration (`400`, naming
the list and the entry). A stored entry that does not parse fails the resolution
closed, naming its policy:

| List | Each entry is exactly |
| --- | --- |
| `applicability`, `non_applicability` | `{"layer": …, "target": …}` |
| `dependencies` | `{"policy": …, "version": …}` |
| `conflicts` | `{"policy": …}` |
| `precedence_hints` | `{"over": …}` |

The example above is the resolution the control `the_seven_step_resolution_fails_closed`
sends, and `the_resolution_steps_refuse_what_the_design_refuses` covers each refusal
in the list (`SIGNOFF-REPAIR.9.1.6`).

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
    "applicability": [{"layer": "organization", "target": "*"}],
    "non_applicability": [{"layer": "organization", "target": "eu-restricted"}]
  }
]
```

A selector is exactly `{"layer": …, "target": …}`, both non-empty strings, and it
matches a resolution target whose layer and target equal its own; `"*"` is the
wildcard, written out per field. An empty applicability list means the policy
applies everywhere. Registration refuses any other shape, such as a missing
field, a bare string, an unknown key, a non-string or an empty value, and names
the list and the entry. A version stored before that rule, whose selector does
not parse, makes a resolution that names it fail with `400`, naming the policy
(`SIGNOFF-REPAIR.9.1.5`). A malformed selector used to be read as a wildcard, so
a typo in an applicability applied a policy everywhere and a typo in a
non-applicability removed it everywhere.

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

`POST /v1/policy-decisions` performs the **draft → decided** transition. A
policy decision is not a second vote. It is the record of the proposal thread's
**counted close** — the *deterministic decision* step of ROADMAP §15.6. The
server reads it from the thread; the request only names the proposal:

```bash
curl -s -X POST localhost:4310/v1/policy-decisions \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{ "decision_id": "dec_0192…", "proposal_id": "prp_0192…" }'
```

```json
{
  "decision_id": "dec_0192…",
  "proposal_id": "prp_0192…",
  "rule": "majority_of_electorate",
  "electorate": { "participants": ["hpr_a", "rol_b", "rol_c"], "denominator": 3, "abstentions": [] },
  "verdict_event_id": null,
  "derivation": {
    "thread_id": "thr_0192…",
    "outcome": "accepted_with_recorded_objections",
    "approval_threshold": 0.6,
    "charter_digest": "sha256:…",
    "tally": { "electorate": 3, "approve": 2, "reject": 1, "abstain": 0 }
  }
}
```

For that to work, the proposal's thread must:

- **declare a decision rule** when it is created — see
  [Deciding a thread](decision-rules.md). A thread with no rule closes on the
  closer's word, and a binding policy decision may not rest on a claim;
- be **closed**, with an outcome the server **derived**;
- have ended in a **binding acceptance**: `accepted_unanimously`,
  `accepted_with_recorded_objections` or `accepted_by_rule`. A `deadlocked` or
  `no_quorum` thread has nothing to approve, and `advisory_synthesis` binds
  nothing.

Each of these is refused with `400 invalid_command`, and the message says which.

`rule` and `electorate` may still be sent. They are **assertions**: each must
equal the derived value, or the request is refused with both values named. An
electorate may name `participants`, `denominator` and `abstentions` only.
Under `owner_decides`, which counts no ballot, the electorate is the thread's
owner alone.

`verdict_event_id` is optional. If named, it must be a `verdict` contribution
of the proposal's thread; it is kept as supporting evidence, not as the
decision.

The electorate is stored as the thread fixed it, so the decision keeps the
membership it was taken under even after the roster changes. A decision cannot
be recorded twice against the same proposal.

⚠️ **Decisions recorded before this rule** carry `"derivation": null`. Their
`rule` and `electorate` are what the caller sent, and they are kept as written
rather than rewritten.

## Approvals

`POST /v1/policy-approvals` performs the **decided → approved** transition. The
approver cites a grant covering `policy_proposal_approve` (see
[Authority](authority.md)), and must be the caller.

The approval's **quorum snapshot is copied from the decision** it approves — the
electorate that decision derived from its thread. The approver supplies nothing
the record keeps:

```bash
curl -s -X POST localhost:4310/v1/policy-approvals \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "approval_id": "app_0192…",
        "proposal_id": "prp_0192…",
        "decision_id": "dec_0192…",
        "approver": "hpr_0192…",
        "grant_id": "grt_hpr_0192…"
      }'
```

`quorum` may still be sent, as an **assertion**: it must equal the decision's
electorate, or the approval is refused with both named. An approver cannot
widen the quorum their approval rests on.

The approval threshold reaches an approval by one route only: the charter
supplies it to the thread when the thread is created, the thread's count uses
it, and the decision records it. A request never supplies it.

⚠️ A decision recorded before decisions were derived (`"derivation": null`) has
no derived quorum to copy, so it **cannot be approved**. Its electorate is what
its caller typed.

⚠️ **Recusal is not implemented**, so no member is ever excluded from a quorum;
the rule *a recused participant does not count* holds only because nobody can
be recused yet.

## Projections

`POST /v1/policy-projections` resolves a policy set for one target and renders
it through the hermetic compiler. `resolution` names the policies to resolve and
the layer and target they are resolved for; the top-level `target` picks one of
the compiler's four dialects: `generic`, `lock`, `codex` or `claude`. The stored
row keeps the rendered bytes, their digest, the resolved set, and anything the
compiler declared it **could not represent** in that dialect:

```bash
curl -s -X POST localhost:4310/v1/policy-projections \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "projection_id": "prj_0192…",
        "target": "generic",
        "resolution": {
          "policies": [{"policy_id": "pol_retention", "version": "2.1.0"}],
          "target": {"layer": "organization", "target": "org-acme"}
        }
      }'
```

```json
{
  "projection_id": "prj_0192…",
  "target": "generic",
  "digest": "sha256:…",
  "bytes": "# Policy bundle (deterministic projection)\n## c1 [pol_retention 2.1.0]\nevidence is retained for 90 days\n",
  "unrepresentable": [],
  "resolved_policies": [{"policy_id": "pol_retention", "version": "2.1.0"}]
}
```

A clause the dialect cannot carry is listed rather than silently dropped. The
compiler has two reasons: a statement with a control character a line-based
bundle cannot express, and a statement over the 8,192-character limit:

```json
"unrepresentable": [
  {"clause_id": "c4", "policy_id": "pol_retention",
   "reason": "the statement exceeds the target's 8192-character limit"}
]
```

**The `lock` target is written by the server** (`SIGNOFF-REPAIR.9.1.4`). It
renders the policy.lock: one line per policy the request named, with its id,
version, digest and owning authority, all read from the registry:

```text
# policy.lock (deterministic projection)
pol_retention 2.1.0 sha256:… grt_hpr_0192…
```

The request used to carry its own `lock` rows, which were rendered verbatim, so a
published lock could name a policy that was never resolved, a digest nobody
registered, or another principal's grant. The field is gone: a request that still
sends `lock` is refused by the body decoder with `422`, and nothing is stored. A
policy whose stored digest does not verify, meaning a version registered before
the server derived digests (`digest_verified` in
[Site authority](site-authority.md)), is refused by the `lock` target with `400`,
naming the policy. A lock never publishes a digest that identifies nothing.

Both examples above are sent as written, identifiers aside, by the control
`the_books_projection_example_runs`, so a change that breaks them fails a test.

⛔ **`unrepresentable` is part of the record, not a warning to be discarded.** A
projection that could not express a clause says so, and staging a publication
reads `resolved_policies` to require that the publication carries the policy its
proposal was approved for.

⚠️ `resolved_policies` may be absent on rows written before `migrations/0082`.
Absent means *the set was never recorded* — it does **not** mean the set was
empty — and staging fails closed on it, naming which of the two it is.

## Reading a published bundle

`GET /v1/policy-bundles/{manifest_digest}` returns what an **effective**
publication actually wrote, looked up by its manifest digest:

```json
{
  "publication_id": "pub_…",
  "manifest_digest": "sha256:…",
  "commit": "51746fd…",
  "manifest": "{\"approval_id\":\"…\",…,\"projection_digest\":\"sha256:…\"}",
  "bundle": "# Policy bundle (deterministic projection)\n## c1 …"
}
```

The server **checks the content before sending it**. The bytes are read from
the publication's repository through its immutable ref, and two checks run
against the record, not against what the stored bytes claim about themselves:

1. the stored manifest must hash to the digest in the request;
2. the stored bundle must hash to the projection digest that manifest names.

If either fails, the answer is `409 publication_conflict`, naming the file that
failed. The content is refused, never served with a warning. Before this route
existed, the digest was only checked when the publication was **written**;
nothing ever read it back.

| Answer | When |
| --- | --- |
| `200` | the content is effective, belongs to the caller's tenant, and both hashes match |
| `400 invalid_command` | the digest is not `sha256:<64 hex>` |
| `404 not_found` | no effective publication in the caller's tenant has this digest — a publication of another tenant gets the same answer — or it was published before its repository was recorded |
| `409 publication_conflict` | the stored content does not match the record |

## Deployment and receipts

A publication does not reach a target by itself. `POST /v1/deployments` assigns
one publication to one target in a **canary wave**, recording the *desired* pair
— the ref and its digest. Both come from the publication; the caller names them
and the server checks them against it:

```bash
curl -s -X POST localhost:4310/v1/deployments \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "target_id": "gateway-v2",
        "publication_id": "pub_0192…",
        "wave": 1,
        "desired_ref": "b45ef6f…",
        "desired_digest": "sha256:…"
      }'
```

The target must be registered — `/v1/deployment-targets`, in
[Authority](authority.md) — the
publication must exist, and — the one that matters — **the publication must be
effective**. Assigning one that is not is refused; a target is never pointed at
something the deployment has not put into force.

**The desired pair must be the publication's own** (ADR-021). `desired_digest`
must equal the digest of the publication's projection, and `desired_ref` must be
one of the Git object ids the publication recorded when it became effective.
`GET /v1/policy-publications` gives both halves: `git_object_ids` and
`projection_id`, whose `digest` `GET /v1/policy-projections` returns. Anything
else is refused with `400`, and the message names the field:

```json
{"code": "invalid_command",
 "message": "desired_digest `sha256:aaa…` is not the publication's projection digest `sha256:3f1…` — a target is assigned what its publication deploys"}
```

Until 2026-09-25 only the digest's *shape* was checked, so a target could be
assigned content its publication never had, and drift then compared what the
target reported against a value nobody published (`SIGNOFF-REPAIR.9.3.3.1`).

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
    "desired_ref": "b45ef6f…",
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
- **repeated waivers**, under `repeated_waiver`: at least **two** waivers of the
  same publication that are still in force (not past their `expires_at`) and
  were recorded within the last **90 days**. One waiver is an exception; the
  second is the repetition §15.11 names.

The seven triggers are `elapsed_interval`, `dependency_change`,
`adverse_threshold`, `external_standard_change`, `repeated_waiver`, `drift` and
`evaluator_regression`.

A `(publication, trigger)` gets a review when it has an occurrence recorded
**after its latest review**, and no review of it is `due`:

- run on a timer, **the verb is idempotent**: an occurrence already covered by a
  review, due or done, schedules nothing;
- the lifecycle **recurs**: an occurrence after a completed review schedules a
  new review, with its own `review_id`;
- occurrences that arrive while a review is due fold into it. At most one review
  per pair is ever due, which the database enforces, so two schedules racing
  cannot both create one.

If a review cannot be stored the verb answers `500`; it never answers an empty
list for a schedule it failed to write. It returns the reviews it created:

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

⚠️ **Until `SIGNOFF-REPAIR.9.3.2` a pair could be reviewed once, for ever.** A
review's id was built from the publication and the trigger and was the table's
key, so after the first review of a pair was done every later one collided with
it, and the collision was discarded. The same discard turned a failing database
into a successful, empty schedule. And a single waiver, even one that had lapsed,
counted as a repeated waiver.

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
