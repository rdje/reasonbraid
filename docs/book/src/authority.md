# Authority

ReasonBraid's required model separates identity, enrollment boundaries and grants.
The development control API trusts the supplied principal header. A known identity
is not, by itself, authority to mutate a target.

The full source review found gaps between this contract and several current API
paths. [Current qualification and repairs](qualification-review.md) names those
limits and their repair owners. The description below distinguishes the existing
thread-command machinery from the selected administrative repair.

## Enrollment boundaries and grants

An enrollment boundary defines permitted actions, domains, risk, spend, delegation
and a validity window. A grant names a subject, action set, selector, validity
window and parent boundary. The core evaluator checks the supplied grant against
the supplied boundary and returns an allowed or denied decision.

The supplied grant must name the supplied boundary and belong to the same tenant.
Its actions, risk, explicit spend limits, delegation flag and validity window must
fit the boundary. The evaluator also requires the grant's subject to match the
principal being evaluated (the delegated subject for the authority-source check).
The caller's own permission remains a separate check in delegated requests.

Validity windows are nonempty and include the start but exclude expiration:
`valid_from <= decision_time < expires_at`. A grant ending at 12:00:00 has no
authority at 12:00:00. An active status alone cannot extend that window.

| Action | Supported target | Required selector coverage |
| --- | --- | --- |
| `thread_create`, `thread_create_auto`, `tenant_admin` | Tenant | `tenant_wide` |
| `thread_inspect` for listing all tenant threads | Tenant | `tenant_wide` |
| `thread_inspect` for one thread | Thread | `tenant_wide` or a set containing that thread |
| Invite, contribute, close, cancel, respond to an invitation, advance a round | Thread | `tenant_wide` or a set containing that thread |

For example, an inspection grant selecting only thread A can inspect A but cannot
list every thread in the tenant. Adding `tenant_admin` to that thread-scoped grant
does not grant tenant-wide administration. A tenant-wide selector still cannot
cross the grant's tenant boundary.

These evaluation corrections are implemented under `SIGNOFF-REPAIR.3.3.2`;
51 core unit + 3 subject tests, all six evaluator controls, 32 live authority/command API
tests and strict core/server lint pass. The owned verification cluster stopped
and was removed. Normal command candidate loading is verified under `.3.3.3.1`: all 37 live
authority/command API tests, six evaluator controls and strict focused lint pass. It reads active grants in pages of 32 rows, ordered by
newest validity start and then bytewise grant ID. Each candidate is checked against
its own parent; the first usable grant wins. A newer future, expired or narrower
grant does not itself remove older authority. Requested delegation scope is part
of selecting the delegated source, and the caller still needs its own usable grant.
There is no arbitrary total candidate cutoff; page size bounds buffered row count.

If no candidate is usable, the audit references the first refused candidate and
its actual parent. If the authority source has no candidate, both references are
absent and the policy version is `no-policy`; a delegated denial cannot borrow
the caller's grant. Malformed candidate storage returns a storage error rather than
a fabricated authority decision. That selection repair retained the audit schema;
the later evaluation-provenance field is described below. The digest input format
remains unchanged. These lookups do not establish transaction-wide revocation ordering.

The separate frozen-tenant administrative read path now uses the same candidate
selection with a boundary-status exception described below. Under `.3.3.3.2.1`,
all 40 live authority/command API tests, ten pure controls and strict focused lint
pass; all results are consumed and the owned cluster removed. The seven original
reads and the exact tenant-scoped receipt lookup now commit inspection admissions
and expose receipts as described below. Delegation depth, consent and cached-decision freshness remain `.3.4`;
tenant authority/effect transaction ordering remains `.3.3.4`.

## Removing a participant

`thread.remove_participant` requires `tenant_admin` on the tenant with a
`tenant_wide` selector, even though its effect changes one thread. The HTTP
handler now constructs that tenant target. Previously it supplied a thread target,
so the stricter evaluator rejected even an eligible administrator with 403.
The evaluator's action/target restriction remains intact.

For example, direct removal through the existing CLI is:

```bash
rb thread remove-participant --thread "$THREAD_ID" --participant "$ROLE_ID" --as organizer
```

The server still locks and selects the aggregate by both tenant and thread ID.
Authority in tenant A cannot remove a participant from tenant B's thread. A
thread-only grant containing `tenant_admin` is insufficient; so are missing action,
revoked/expired/future grants and suspended/revoked parent boundaries. A delegated
caller cannot borrow authority it lacks, and thread-only delegated scope cannot
cover tenant administration.

| Request outcome | Result and durable evidence |
| --- | --- |
| Eligible administrator, matching tenant/thread and removable participant | 200, revoked participant state, removal event and a tenant-target administrative allowance. |
| Authority or requested delegation scope is insufficient | 403, one tenant-target denial; domain, queue, budget and quota-event rows remain unchanged. |
| Eligible tenant administrator names a thread outside that tenant | 404; domain rows stay unchanged and the provisional admission rolls back. |
| Same key replays an already committed denial | The original refusal remains the result; restoring authority does not reinterpret that past request. Use a fresh key for new intent. |

The existing accepted-participant lifecycle and new live HTTP controls qualify
this target mapping. Concurrent authority/effect ordering and broader delegated
consent/depth remain `.3.3.4.4`/`.3.4`. The CLI companion `.11.4.3.1.2.10` now
requests tenant-wide delegation for removal only. Ordinary existing-thread verbs
retain single-thread scope; both caller and source authority are still required.
Direct HTTP/CLI requests do not carry that delegation context. Evidence:
`docs/tasks/artifacts/signoff_review/participant-removal-authority.md` and
`docs/tasks/artifacts/signoff_review/cli-removal-delegation.md`.

## Grant creation failures

Grant creation distinguishes a missing parent, a structural ceiling violation and
unavailable or malformed storage. A database failure does not prove that a grant
exceeds its boundary. Enrollment and card import use these outcomes:

| Condition | HTTP result | Effect |
| --- | --- | --- |
| The requested role actions exceed the actual parent ceiling | 400 `invalid_command`, with the enrollment/import violation details | No grant or enrollment/import effect from that refusal. |
| The named parent is missing, or no active enrollment boundary exists | 400 `invalid_command`, identifying the missing authority | No new grant. |
| A grant INSERT fails, or the stored boundary cannot be decoded | 500 `dependency_unavailable`, `internal server error` | The request refuses; internal storage details stay out of the response. |
| The boundary and requested grant are valid and storage succeeds | Existing successful enrollment/import response | The normal successful path continues. |

For example, an enrollment INSERT failure returns:

```json
{"code":"dependency_unavailable","message":"internal server error"}
```

It no longer says that the grant exceeds its boundary. An active boundary with
malformed stored actions produces the same safe error instead of dropping the HTTP
connection through a handler panic. Correcting the storage fault permits a fresh
request; this error classification does not introduce automatic retries.

**Rust API migration:** `create_grant` now returns `Result<(), GrantCreateError>`.
Code that accessed `error.violations` must first match
`GrantCreateError::Refused(refusal)` and then inspect `refusal.violations`.
`GrantRefused` remains exported for actual structural ceiling failures.

```rust
use reasonbraid_server::{create_grant, GrantCreateError};

// Within an async caller, with an owned pool and a candidate AuthorityGrant:
match create_grant(&pool, &candidate).await {
    Ok(()) => { /* the repository call succeeded */ }
    Err(GrantCreateError::Refused(refusal)) => {
        for violation in refusal.violations {
            eprintln!("{violation}");
        }
    }
    Err(GrantCreateError::MissingBoundary { boundary_id }) => {
        eprintln!("missing parent: {boundary_id}");
    }
    Err(GrantCreateError::Storage(error)) => {
        // Preserve diagnostics in the trusted caller; do not return them over HTTP.
        eprintln!("storage unavailable: {error}");
    }
    Err(error) => {
        // The enum is non-exhaustive; handle future failures without claiming success.
        eprintln!("grant creation failed: {error}");
    }
}
```

Storage failures also retain the original SQLx error through
`std::error::Error::source`, including unique violations, closed pools and malformed
data. Missing parents are a distinct outcome, with no invented structural
violation or SQL unavailability cause.

All 56 live authority/HTTP/card controls and focused strict lint pass under
`SIGNOFF-REPAIR.3.3.4.3.1`; all results and shutdown are consumed, and the four
owned verification clusters are absent.

The matched grant-insertion fault controls check complete enrollment/import table
snapshots and recovery. They do not establish atomicity for every later card-import
step: complete import transaction integration remains `SIGNOFF-REPAIR.3.3.4.11`.
Standalone writers and complete development enrollment now use the tenant guard
and issuance-time parent liveness described below; development issuer policy is
unchanged. Card import retains its separately owned transaction integration.

## Subject JSON and delegation inputs

Core authority payloads represent the subject explicitly. For example:

```json
{"kind":"human","id":"hpr_00000000-0000-7000-8000-000000000001"}
```

```json
{"kind":"role","id":"rol_00000000-0000-7000-8000-000000000001"}
```

The kind must match the typed ID prefix. Missing, duplicate or unknown fields,
unknown kinds, malformed IDs, bare strings and arrays are rejected. Object field
order does not matter. This representation applies to a core grant's `subject`,
a delegated authorization record's `subject`, and core delegation constraints'
`on_behalf_of`. The correction under `SIGNOFF-REPAIR.3.3.1` passes the core tests,
including direct/enclosing payloads and invalid input. All 40 live
authority/command API/site-receipt compatibility controls pass. The old serialization failure is reproduced.

The existing public command envelope remains a separate compatibility contract:

```json
{
  "authority_context": {
    "on_behalf_of": "rol_00000000-0000-7000-8000-000000000001",
    "purpose": "delegated contribution",
    "scope": {"kind":"tenant_wide"}
  }
}
```

This is a fragment of a command, not a complete command or a capability grant.
The HTTP/MCP principal input also remains a prefixed string. Database records
retain separate `subject_kind` and `subject_id` columns; no migration or automatic
new authority follows from the core JSON correction. Site issuance already uses
the explicit kind/id form described in [site authority](site-authority.md).

The development choice remains delegation through the command envelope. The old
ADR-009 test added 64 to a hand-built JSON length; it did not measure a token
implementation or multi-hop depths. Its size-advantage claim is withdrawn. The
replacement control uses the actual public envelope type, and `.3.4` owns a real
comparison before any comparative claim is restored.

## Thread commands and audit

Thread creation targets a tenant. Invitation, contribution, close, cancel,
invitation-response and round-advancement actions target a thread. Inspection
supports either a selected thread or the tenant-wide listing described above. The normal command path
combines authorization with state, event, idempotency and outbox writes in a
PostgreSQL transaction. Rejected commands preserve their rejection/audit according
to that path's transaction handling. Exact committed replays preserve historical
results; they are not a fresh authorization to mutate another target.

The authorization record contains:

```text
record_id · tenant · actor · delegated subject · boundary · grant
action · target · decision · reason · policy_digest · policy_version · time
```

The current digest covers selected identifiers and decision fields. It is not a
content digest of every field of the boundary and grant. The review must reconcile
that limitation with any stronger policy-binding claim. Administrative handlers do
not all use the thread-command transaction; their audit and revocation serialization
are explicit corrective work, not guarantees inferred from the command core.

### Evaluation provenance and historical records

Authorization records now carry an explicit `evaluation` object, including the
existing thread-audit response. Under `SIGNOFF-REPAIR.3.3.3.2.2.1`, final code
passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade
tests and strict lint. Final fixed/latest-upgrade controls pass; all results and
shutdown are consumed and the owned cluster removed. Qualification closure
REPAIR-0017 completes implementation 305ed26.

| `evaluation.kind` | Meaning |
| --- | --- |
| `legacy_unspecified` | The producer did not record which evaluation path it used. |
| `boundary_checked` | Ordinary evaluation required an active actual boundary; this includes ordinary reads. |
| `tenant_admin_inspection` | Admission for a named direct own-tenant read using the boundary-status exception, including authority denials. |

For example, new ordinary records contain:

```json
{"evaluation":{"kind":"boundary_checked"}}
```

Historical rows and old writers receive `legacy_unspecified` through migration
0055. Historical core JSON that omits evaluation decodes with that same meaning;
no action is used to guess its prior intent. Newly serialized records include the
field. Older strict readers that reject unknown fields must upgrade before
consuming them. The migration preserves the other historical fields unchanged.

The inspection metadata names the direct human/role principal and inspection,
retaining the selected parent's actual status and grant selector. Source references
remain in the enclosing record. Absent authority sources carry absent references
and absent status/scope evidence. The exact receipt lookup records the requested
typed record ID in its inspection name, separately from its own admission ID.

The metadata for an inspection admission is explicit, for example:

```json
{
  "kind": "tenant_admin_inspection",
  "principal": {"kind": "human", "id": "hpr_00000000-0000-7000-8000-000000000141"},
  "inspection": {"kind": "grants"},
  "boundary_status": "revoked",
  "grant_selector": {"kind": "tenant_wide"}
}
```

This is an evaluation value, not a complete record or a grant. A missing-source
refusal uses null status and selector instead
of borrowing evidence from another tenant boundary or the caller's other grant.

Record readback validates typed IDs, nullable field pairs, decision and target
discriminants, and complete evaluation metadata. Malformed records return a
storage failure; the reader does not guess a denial, drop a subject or panic.
Evaluation, inspection names and shared target selectors require JSON objects.
For example, `{"kind":"tenant_wide","threads":["thr_…"]}` is refused rather than
discarding the named threads; `["tenant_wide"]` is also refused. Canonical
tenant-wide and named-thread selector objects retain their existing shape and
schema. Duplicate and unknown metadata members are errors.
The existing thread-audit view uses the same decoder. Its inventory remains the
existing unpaginated thread view; this change adds no general audit list.

Evaluation provenance describes admission. It does not prove an effect or that
the entire response reached the caller. The existing policy digest does not bind
this additional field or every source fact. The eight named HTTP reads return
admission receipts, with exact tenant-scoped lookup described below. Ordering and
effect audits remain `.3.3.4`.

## Administrative authority

Tenant administration is intended to remain tenant-scoped. The approved frozen
boundary carve-out allows an otherwise eligible tenant administrator to inspect its
own tenant after boundary revocation. It does not grant authority over other tenants.

The exception ignores **only the actual boundary's status** (active, suspended or
revoked). The direct caller still needs an active tenant_admin grant selecting
the whole tenant, bound to that parent and tenant and within every parent ceiling.
Both windows must be nonempty and currently valid: the start is inclusive and
expiration is exclusive. Revoking the grant, expiring either window or narrowing
the parent below the grant removes eligibility. No delegation enters this path.

These eight named GET routes use the exception:

| Route under `/v1/admin/` | Own-tenant inspection |
| --- | --- |
| `nodes/presence` | Known nodes and derived presence |
| `grants` | Grant inventory and status |
| `boundaries` | Enrollment boundaries and status |
| `incarnations` | Recorded role incarnations |
| `runs` | Recorded runs |
| `breakers` | Spend-breaker state |
| `usage` | Usage ledger summary |
| `authorization-records/{record_id}` | One known authorization record in this tenant |

Each takes `?tenant_id=ten_…`; the seven original routes retain their response
shapes, and the new exact lookup is described below. For example,
after freezing Alice's boundary, Alice can still call
`GET /v1/admin/boundaries?tenant_id=ten_…` with a structurally valid unexpired
grant. A grant limited to one thread cannot list these tenant-wide inventories.
A newer future-dated grant does not hide Alice's eligible older grant. Bob's
administrator grant in another tenant cannot inspect Alice's inventory.

Each attributable allow or authority denial commits a `tenant_admin_inspection`
record before returning protected data. The record keeps the actual selected
parent status and grant scope, so a read allowed under a revoked parent remains
distinguishable from ordinary boundary-checked admission.

That record is evidence, so the admission takes the tenant's **shared authority
guard** before reading any authority row, and its decision time is database time
sampled **after** that wait. A revocation holds the guard exclusively, so it
fences every admission that has not already selected its evidence; admissions
never fence each other. Without it, the selected parent's status and the grant's
selector could be read either side of the writer that changes them, and the
record would carry a mixture.

The same applies to the ordinary standalone admission — thread inspection,
node-token issuance, automatic thread creation, recruitment calls and every
`tenant_admin` gate — which now authorizes through the guarded entry point. The
standalone `authorize` API keeps its explicit timestamp and takes no guard: it
is the evaluation-time surface a caller drives with a chosen instant, not the
path a live request uses.

| Order | Result |
| --- | --- |
| A revocation holds the exclusive guard when a read arrives | The read waits, then admits against the authority as it then stands. |
| Authority ends while a read waits | The post-wait database time sees it ended; the read is refused 403. |
| A boundary is frozen while a read waits | The read is still ADMITTED — the carve-out ignores boundary status, and the ordering does not change that. |
| Two reads in the same tenant | Both hold the shared guard; neither blocks the other. |

The response queries still run after the admission commits, so a receipt proves
admission and not a shared snapshot or delivery.

Use `curl -i` to retain the receipt, for example:

```bash
curl -i "$RB_URL/v1/admin/grants?tenant_id=$TENANT_ID" \
  -H "x-reasonbraid-principal: $PRINCIPAL_ID"
```

Both successful reads and authority denials carry
`x-reasonbraid-authorization: authz_…`, naming their committed admission. Successful
JSON bodies retain their existing shape. A denied read remains HTTP 403 with code
`unauthorized`, and its message also names the record. These are trusted dev-profile
principal headers; they do not establish Internet authentication.

| Failure point | Response and receipt |
| --- | --- |
| Request/principal extraction | Existing extraction refusal; no admission receipt. |
| Authority selection or audit persistence | HTTP 500 `dependency_unavailable`; no protected response or unconfirmed receipt. |
| Response query after the admission commits | HTTP 500 `dependency_unavailable` with the real committed admission receipt. |

The response queries run after the admission transaction commits. A receipt proves
that admission was recorded; it does not promise a shared response snapshot,
revocation serialization or delivery of the response. Under
`SIGNOFF-REPAIR.3.3.3.2.2.2`, 45 live authority/API tests, ten pure evaluator tests
and strict lint pass, including all seven routes, human/role admissions, denials,
audit-insert failure and a later query failure with recovery. All results and
owned-cluster shutdown are consumed. Thread/audit inspection, cross-domain
receipts and process metrics retain their separate gates; the exception does not
extend to them or to site registries.

### Read an authorization receipt

An eligible tenant administrator can retrieve a known own-tenant authorization
record, including a denial or a record made before provenance was recorded:

```bash
curl -i "$RB_URL/v1/admin/authorization-records/$RECORD_ID?tenant_id=$TENANT_ID" \
  -H "x-reasonbraid-principal: $PRINCIPAL_ID"
```

Set `RECORD_ID` to the complete `authz_…` value from an earlier receipt. A successful
response contains `tenant_id` and `authorization`, the complete canonical record.
The lookup preserves its record ID, actor, delegated subject, actual source
references, action, target, decision/reason, evaluation, policy digest/version
and decision time. Historical `evaluation.kind` remains `legacy_unspecified`;
an inspection does not relabel or rewrite the earlier record.

| Location | Meaning |
| --- | --- |
| Response body `authorization.record_id` | The record requested in the URL. |
| Response body `authorization.decision` | That earlier request's decision, which can be denied even when this lookup succeeds. |
| Response header `x-reasonbraid-authorization` | A new admission for this lookup. |
| New admission's `evaluation.inspection` | `{"kind":"authorization_record","record_id":"authz_…"}` naming the requested record. |

Each lookup creates one new admission. It returns the requested evidence without
automatically fetching the new admission; inspecting that separate receipt takes
another explicit request. The same grant/parent/tenant/scope/window checks apply
to direct human and role administrators, including the boundary-status exception.
Possessing a receipt ID alone grants no access to its tenant.

| Request outcome | Response |
| --- | --- |
| Eligible administrator, known record in the requested tenant | HTTP 200, complete record and a new admission receipt. |
| Eligible administrator, absent or foreign record ID | Identical HTTP 404 `not_found`, message `authorization record not found`, with its new admission receipt. |
| Ineligible principal for the requested tenant | HTTP 403 and a committed denial receipt before evidence lookup. |
| Malformed own-tenant record | Safe HTTP 500 `dependency_unavailable` with the lookup's committed admission receipt. |
| Malformed record/tenant ID, unknown or duplicate query field | HTTP 400 extraction refusal without an admission receipt. |

Tenant and record ID are filtered together before decoding. Even malformed foreign
evidence has the same missing-record response. Failed audit persistence uses the
same no-data/no-unconfirmed-receipt rule as the other seven reads. This endpoint
retrieves one ID; it adds no list, pagination or query language. The `rb` CLI has no
receipt command yet; use the HTTP call above. Admission remains separate from
response delivery and revocation serialization. Under
`SIGNOFF-REPAIR.3.3.3.2.2.3`, 18 live authority tests and 30 HTTP tests pass;
the final HTTP rerun also verifies retrieval of an outsider's denied attempt by
the tenant administrator. Strict lint passes, all results are consumed and the
owned clusters are removed.

### Shared registries

The shared adapter and region HTTP handlers use the separate
[site-authority service](site-authority.md): explicit operator-issued grants,
actual-parent liveness, and atomic registry/audit operations serialized with site
revocation. Tenant enrollment cannot mint those grants. The protected `rb-site`
CLI is verified under `SIGNOFF-REPAIR.3.2.2`; HTTP enforcement is implemented under
`.3.2.3`, with eight HTTP controls and the selected adjacent checks passed. Shared inspection requires the
explicit `registry_inspect` action; tenant-scoped inspection retains its own policy.

### Grant and boundary revocation

Both revocation services now bind the target id to the authorized tenant inside
PostgreSQL before changing status. A missing or foreign target returns HTTP 404.
The matching target is locked until status and the tenant's revocation epoch commit
together. An already revoked target causes no further epoch increment.

For example, suppose Alice administers tenant A and Bob administers tenant B:

| Request | Result and protected state |
| --- | --- |
| Alice supplies A's tenant id while targeting B's role grant | 404; B's grant, epoch and authorization records remain unchanged. |
| Alice supplies A's tenant id while targeting B's boundary | 404; B's boundary, epoch and authorization records remain unchanged. |
| Bob revokes B's role grant | 200; status becomes revoked and B's epoch increases once. |
| Bob repeats that role-grant revocation | 409; no further epoch increment. |
| Two admitted requests revoke the same active grant | One succeeds; the other sees the revoked status and returns 409; one epoch increment. |
| Bob revokes B's boundary, then attempts another administrative write | The next write is refused by the frozen boundary; eligible own-tenant inspection remains available. |

The prior implementation returned 404 only after committing the foreign target's
revocation. Both live baseline controls reproduced status active→revoked and epoch
0→1 despite that response. A rejected repeated grant revocation also advanced the
epoch to 2. `SIGNOFF-REPAIR.3.1` owns the correction and its focused verification;
the corrected API/authority/escalation run passed 34 tests. The contention control
observed both requests waiting on database locks before releasing the target.

The existing tenant-admin audit records the caller's admission decision. It does
not yet persist the submitted revocation reason and final effect outcome atomically
with the mutation. That effect audit, and serialization against revocation of the
acting administrator's own authority, remain `.3.3`. The target-row lock described
here does not establish those separate guarantees.

### The final administrative effect record

An admission record says a caller **was allowed to ask**. It says nothing about
what the local mutation then did. Collapsing the two would let an operator read
`allowed` as `applied`, so the final effect is a separate, additive record.

⛔ **No route writes one yet.** `SIGNOFF-REPAIR.3.3.4.7.1` defines the
representation and `.7.2` gives it durable storage; grant/boundary revocation is
the first route to produce one, under `.8`. This section describes a contract the
integration children fill, exactly as the tenant guard was qualified as a
primitive before any route used it.

An effect record names four things:

| Part | Meaning |
| --- | --- |
| The admission | The `authz_…` record that permitted this request. It IS the effect's own identifier, so an effect with no admission cannot be represented and there is no second key to keep in step with the first. |
| The operation | One of a closed set of fourteen administrative mutations, each carrying its own tenant-bound target. |
| The submitted reason | What the caller wrote, when the operation takes a reason. Four of the fourteen do today; the rest record `null`, and adding a reason requirement to any of them is a wire change that has to be documented and tested as one. |
| The outcome | `applied`, `no_op`, or `refused`. |

The outcome is the fact an admission cannot carry:

| Outcome | What it asserts |
| --- | --- |
| `applied` | The protected target changed. |
| `no_op` | The request was already satisfied — the repeated-revocation shape. No protected state and no revocation epoch changed, and the detail says what was already true. |
| `refused` | A domain rule refused AFTER admission. No protected state and no revocation epoch changed. It carries the exact code the request's own response used — `invalid_command`, `invalid_transition` or `not_found` — so the record and the response cannot disagree about which refusal happened. |

The fourteen operations are the administrative mutations this repair family owns:
grant and boundary revocation; breaker arm and reset; node enrollment-token
issuance, node revocation, inbox command replay, quarantine and prune; profile
card import and capability-claim attestation; and the three federation direction
verbs. They were not chosen by taste — the population was measured first, and the
four other `tenant_admin`-gated mutations (resolver registration, automatic thread
creation, recruitment open and close) are deliberately absent because they are
owned elsewhere and this family cannot certify their gates.

Every variant names a target the **caller supplied**, so the target exists when
the outcome is `refused` just as it does when the mutation applied. Breaker
administration names no target field at all: its target is the tenant, which the
record already carries.

For example, a repeated grant revocation would record:

```json
{
  "record_id": "authz_00000000-0000-7000-8000-000000000001",
  "tenant_id": "ten_00000000-0000-7000-8000-000000000002",
  "operation": {"kind": "grant_revoke", "grant_id": "grt_hpr_…"},
  "submitted_reason": "role retired",
  "outcome": {"kind": "no_op", "detail": "the grant was already revoked"},
  "effected_at": "2026-09-12T10:30:00Z"
}
```

Decoding is strict, because this is evidence:

- an operation, an outcome and a record must each be a JSON **object** — the
  array and bare-string forms the pinned decoder would otherwise accept are
  refused, as they already are for evaluation provenance;
- unknown, duplicate and missing members are errors, and a target identifier is
  nonblank, at most 256 UTF-8 bytes and free of control characters, while a
  reason has the same shape with a 1 024-byte limit — the bounds the site
  registry already publishes, so an operator meets one rule rather than two;
- a `refused` outcome naming a code outside those three is a decode **failure**,
  not a guessed outcome. This build writes these codes, so a code it cannot name
  means the row did not come from a build it understands;
- `submitted_reason` must be **present**, as `null` when the operation takes no
  reason. This record has no history, so a missing member is malformed evidence
  rather than an absent reason.

Two things this deliberately does not do. It does not relabel anything: the
authorization record's JSON is byte-identical, and an effect is a separate row
rather than a new field on the admission. And it makes no historical claim — an
absent effect record means the outcome was never recorded, never that an
operation succeeded or failed.

#### Where the effect is written, and what its absence means

Migration 0058 adds `administrative_effects`, keyed by the admission's own
`authz_…` id. It is additive in the strict sense: no existing table, column or
row changes, nothing is backfilled, and the one index it adds to
`authorization_records` exists only so an effect can reference both an admission
and that admission's tenant in a single foreign key. That composite key is why an
effect **cannot** cite another tenant's admission — a structural bound rather
than a rule the writer is trusted to follow.

The writer runs on the caller's **own** transaction, and both halves of that
matter:

| Property | What provides it |
| --- | --- |
| Evidence and mutation commit together | They are the same transaction, so there is no window in which one exists without the other. |
| An evidence failure rolls the protected write back | Same transaction again: a rejected effect aborts the mutation rather than leaving it unexplained. |
| The whole thing is ordered against an authority change | The caller's tenant authority guard, which it already holds. The writer takes no guard of its own — that would be a second acquisition on a connection that already has one. |

Reading is an **exact own-tenant lookup** by admission id: the tenant filter is
applied before the row is decoded, so a foreign tenant's malformed evidence is
indistinguishable from an absent record and cannot become an existence oracle.
There is no list, no pagination and no query language.

Two failure answers are deliberately different, and an operator acts on them
differently:

- **absent** — no outcome was ever recorded for that admission. Every request
  admitted before this table existed reads that way, and so does every route
  that has not yet been migrated onto the writer. It does **not** mean the
  operation did nothing, and it does not mean the operation succeeded.
- **a storage failure** — the row exists and this build cannot read it. The
  reader never guesses `applied` and never downgrades an outcome it does not
  recognise into one it does.

The database constrains the `kind` discriminant of both JSON columns to the
closed vocabularies above; everything below the discriminant is the codec's to
enforce, which is why a row can pass the column constraint and still be refused
on the way out.

The three refusal codes are their own small vocabulary rather than the §9.8
reason-code registry, and the reason is worth stating because the first attempt
went the other way. Reusing §9.8 looked like exactly the right move — one
registry, no drift. Measuring it said otherwise: the registry publishes 20 codes,
the product emits 19 distinct ones, **10 of which the registry does not contain**,
including the `not_found` that a revocation's 404 returns. So a record typed
against §9.8 could not say what the response said. These three are instead what
the administrative handlers actually refuse an admitted operation with, and their
wire names are literally the strings in the response body. The registry's own
reconciliation is tracked separately as `SIGNOFF-REPAIR.11.7`; it is a wider
finding than this chapter.

### Ordering a thread command against an authority change

A thread command now takes the tenant's authority guard **before** it claims its
idempotency key, locks the aggregate row, or touches quota and inbox state. The
mode is *shared*, so commands still run concurrently with one another; a
revocation takes the *exclusive* mode and therefore fences every command that
has not already passed that point.

This is an ordering that previously did not exist at all, rather than a race
window that has been narrowed. Before it, the command path took no guard, so a
revocation and a command had no defined order between them.

The decision time is now **database** time, sampled after the guard and the
idempotency claim have both waited, rather than the process clock read before
them. A grant that expires while a command is queued is evaluated as expired,
not as it stood when the request arrived. The standalone `authorize` API keeps
its explicit timestamp parameter, so its documented evaluation-time behaviour is
unchanged; only the live effect entrypoints choose their own current time.

| Order | Result |
| --- | --- |
| A revocation holds the exclusive guard when a command arrives | The command waits, then evaluates against the revoked authority. |
| A command holds its shared guard when a revocation arrives | The command commits; the revocation applies to what follows. |
| A grant expires while a command waits | The post-wait database time sees it expired. |
| Two commands in the same tenant | Both hold the shared guard; neither blocks the other. |
| A command in an unrelated tenant | Unaffected — the guard is per tenant. |

Each row is a live control in the `command_ordering` suite rather than a
consequence read off the design: the wait, the shared-mode bound (which covers
both the concurrent-commands row and the unrelated-tenant row), and the
post-wait evaluation of authority that ended underneath a blocked command. The
last of those was argued rather than measured until `SIGNOFF-REPAIR.3.3.4.4.1`
added it, and it is falsified rather than merely green — with the guard
acquisition removed, the command completes before the authority changes at all.

This orders a command against an authority change. It is not a claim about
replay-hash or consent semantics, which remain `SIGNOFF-REPAIR.3.4`, nor about
automatic-initiation preflight, which remains `.5.2`.

### Tenant transaction foundation and remaining integration

The selected contract in `SIGNOFF-REPAIR.3.3.4.1` keeps a shared tenant-authority
guard through an ordinary protected local transaction and an exclusive guard
through authority issuance/revocation. A dedicated full-tenant-key table preserves
the standalone authority API's ability to operate without a tenant identity row.
The guard comes before lease, idempotency and domain locks; live decision time is
sampled after earlier waits. Final administrative effect evidence will remain
distinct from admission, with the mutation and its required evidence in one commit.

Migration 0056 and the private guard owner are implemented under
`SIGNOFF-REPAIR.3.3.4.2`. The isolated tests compile that exact source. Under
`.3.3.4.3.2`, five standalone service paths now use the guard: boundary creation,
grant creation, active-boundary lookup, grant revocation and boundary revocation.
All 85 selected controls (84 live / one pure), focused strict lint and book
checks pass, including both HTTP commit-failure routes. All results/shutdown are
consumed and the three owned clusters are absent.
The source census covers 42 direct named-call locations across 101 tracked Rust
source files, plus transitive HTTP/MCP/node/state-service and authority mutations.
Command/node transactions, inspection admissions, final effect records and the
remaining administrative families retain their separate integration children.
The exact scope and remaining policy owners are recorded in
`docs/tasks/artifacts/signoff_review/tenant-authority-paths.md` and
`docs/decisions/2026-09-09_tenant-authority-transaction-order.md`.

### Standalone issuance and status services

The four standalone writers hold the tenant's exclusive guard through their
transaction; active-boundary lookup holds a shared guard through its read. The
same full tenant key coordinates issuance with grant/boundary revocation. The
services also retain their target-row locks and exact tenant predicates, so a
foreign target cannot be mutated and an already revoked target does not advance
the epoch. Unknown stored target status now produces safe HTTP 500 with the
original target and epoch preserved, rather than being overwritten as `revoked`.

| Standalone issuance scenario | Behavior |
| --- | --- |
| Parent-boundary revocation obtains the tenant guard first | Issuance waits, then reads the revoked parent and refuses. |
| Issuance obtains the guard first | Revocation waits until issuance finishes, then revokes the attributed parent. |
| The parent expires while issuance waits for the guard | The fresh database-time evaluation refuses the grant. |
| The parent is active but starts tomorrow | Standalone issuance refuses until the parent is live. |
| The parent is live now and the grant starts tomorrow within its window | The scheduled grant remains supported. |
| A grant names a different tenant's parent | Refuse the binding without locking or decoding the foreign policy. |

The time check occurs after guard acquisition and the own-tenant parent lookup,
immediately before INSERT. It does not promise that the parent remains live at
commit acknowledgment or response delivery. Every later use of the grant still
checks the parent and grant validity windows normally.

`create_boundary` now returns `Result<(), AuthorityTransactionError>`. This public,
non-exhaustive error keeps ordinary `Storage(sqlx::Error)`, pre-commit `Deadline`,
and unconfirmed `Commit(sqlx::Error)` / `CommitDeadline` distinct. Storage and
commit errors retain their original SQLx source. Boundary creation inserts a new
row; it does not re-activate or replace an existing boundary.

`GrantCreateError` adds `BoundaryNotLive { boundary_id }` for the new issuance-time
refusal and `Transaction(AuthorityTransactionError)` for guarded transaction
failures. Ordinary storage failures still expose the original SQLx error directly
through `Storage` and `Error::source`; a commit failure preserves its transaction
phase rather than being collapsed into that variant. Code matching the
non-exhaustive enum must continue to handle unrecognized future failures safely.

A standalone grant refusal now aborts its guarded transaction. For example,
`MissingBoundary`, `Refused` and `BoundaryNotLive` insert neither a grant nor a
first-use coordination anchor. An anchor that already existed stays intact. A
deferred constraint on a provisional anchor cannot replace the established
refusal, because this path does not attempt COMMIT. The successful grant path
still commits normally and preserves the commit-error contract below.

The private transaction API supports typed callback errors through the same
bounded guard and connection owner. A returned error aborts preceding work; a
successful callback value commits that value and its local effects. For example,
a recorded authorization denial may deliberately be a successful transaction
value, while an enrollment policy refusal must roll back its provisional rows.
These are explicit choices at each call site. Converting a domain refusal into a
SQL protocol error would lose that distinction. The complete enrollment handler now uses this entrypoint under
`SIGNOFF-REPAIR.3.3.4.3.3.2`. The typed-error
prerequisite passes 89 selected controls (88 live / one pure), focused strict lint
and rendered book checks. All results/shutdown are consumed; both owned clusters
are absent. Evidence is recorded in
`docs/tasks/artifacts/signoff_review/typed-rollback-errors.md`.

HTTP routes using these guarded services return a distinct error if a commit was
attempted but its outcome is unconfirmed:

```json
{"code":"commit_outcome_unconfirmed","message":"transaction outcome is unconfirmed; inspect the target before retrying"}
```

The status is HTTP 500. The response exposes neither SQL details nor a claimed
rollback. Inspect the relevant tenant state before deciding whether another
request is appropriate. This is an additive error-code contract; ordinary
pre-commit storage/decoding failures retain `dependency_unavailable`. A deferred
constraint failure may be shown by a controlled test to have rolled back, while
an acknowledgment timeout can leave a committed row. The public error must
preserve uncertainty across both situations.

Complete development enrollment now retains its exclusive guard through all
local writes, as described below. Card import still uses the separate guarded
active-boundary lookup followed by its existing insertion bridge, which does not
retain that guard or apply the guarded service's time check. Its complete
transaction owner is `.3.3.4.11`. Node-certificate epoch writes retain `.3.3.4.10`.
HTTP revocation admission, submitted reason and final effect are still separate
until `.3.3.4.8`; the service lock does not join the acting administrator's earlier
admission to that mutation.

### Development enrollment transactions

`POST /v1/enrollments` and `rb enroll` use one exclusive tenant guard from the
replay lookup through commit. New-tenant identity, tenant quotas, boundary, grant,
principal identity, principal quota and enrollment rows commit together. Existing
tenants use their active boundary on the same connection. A typed pre-commit
refusal rolls back every provisional row, including a new coordination anchor;
pre-existing anchors remain intact. Concurrent requests for the same existing
(tenant, kind, name) produce one new principal and a replay of that principal.

A new human request creates a tenant and its development boundary:

```json
{"kind":"human","name":"alice"}
```

The successful response has `replayed: false`, a tenant ID, a human principal ID,
and both `boundary_id` and `grant_id`. Supply that tenant ID when enrolling a role
(the abbreviated IDs below are placeholders for actual response IDs):

```json
{"tenant_id":"ten_…","kind":"role","name":"reviewer"}
```

A new role response includes `grant_id` and omits `boundary_id`. Omitting `actions`
grants both `thread_contribute` and `thread_invitation_respond`. To request a
specific action set on first enrollment, use for example:

```json
{"tenant_id":"ten_…","kind":"role","name":"observer","actions":["thread_inspect"]}
```

Repeating the same tenant, kind and name returns the original principal, with
neither a new grant nor a changed action set:

```json
{"tenant_id":"ten_…","principal_id":"rol_…","kind":"role","name":"reviewer","replayed":true}
```

Replay precedes action parsing and authority lookup, so a replay with different
or invalid action names still returns that existing principal. It also remains
available after boundary revocation; it grants no new permission. Unsupported
kind and malformed tenant IDs are rejected before replay. A new role needs an
explicit tenant and active boundary. Missing active authority is checked before
parsing role action names. A new human request without `tenant_id` creates a new
tenant each time; matching names across separate tenants do not imply replay.

| Enrollment scenario | Result |
| --- | --- |
| Enrollment obtains the tenant guard first | Revocation waits until enrollment's complete local transaction finishes. |
| Boundary revocation obtains the guard first | New enrollment waits, then refuses without a new grant or identity. |
| The parent expires while enrollment waits | Fresh database-time evaluation returns 400 `invalid_command`, identifying the non-live parent. |
| The active parent starts in the future | New enrollment refuses until the parent is live. |
| Role actions exceed the current parent ceiling | 400 `invalid_command` retains the structural violation details. |
| Identity, quota, grant or enrollment storage fails before commit | Safe 500 `dependency_unavailable`; provisional local writes roll back. |
| A commit acknowledgment fails or times out | Safe 500 `commit_outcome_unconfirmed`; inspect state before deciding whether to retry. |
| A standalone boundary exists without a tenant identity | Enrollment fails the identity FK; it cannot silently manufacture a tenant. |

A new bootstrap boundary starts at database time sampled after guard acquisition
and replay lookup. Grant issuance checks its actual parent's live window again
immediately before INSERT, after preceding authority waits. This is an evaluation
instant; the guard does not stop time or guarantee response delivery before
expiry. Commit failures retain uncertainty even when a controlled deferred fault
can prove that its particular transaction rolled back.

For a new bootstrap whose success response is lost or whose commit is
unconfirmed, a caller that omitted the request key may not know the generated
tenant ID. Operator database reconciliation can still be needed. Repeating the
same human name without a tenant ID or request key is intentionally a new request
and can create another tenant. A live control observes HTTP commit uncertainty,
then the original committed bootstrap, then a distinct tenant from a repeated
no-key request. Keyed callers can now recover through the protocol below; the CLI
has not yet implemented durable request-key persistence.

Enrollment remains a development trust mechanism. A human receives the nine
explicit dev admin actions regardless of an `actions` field, and its grant names
that human as issuer. A role's grant records a newly generated human issuer handle;
that handle is not authenticated issuer provenance. Caller/issuer policy remains
owned by `.3.5`; enrollment cannot issue site-operator grants. Qualification and
exact rollback/race evidence for this integration are tracked in
`docs/tasks/artifacts/signoff_review/enrollment-transaction.md`. All 97 selected
controls (96 live / one pure), final focused strict lint and rendered book checks
pass. Every result/shutdown is consumed; all three owned clusters are absent.

#### Recover one bootstrap with a persisted request ID

`POST /v1/enrollments` accepts an optional `bootstrap_request_id` for a human
creating a new tenant. Generate a fresh RequestId once and **durably persist it
with the exact request and server identity before the first send**. Its wire form
is exactly 40 characters: `req_` followed by a lowercase, hyphenated RFC UUIDv7.
The server continues generating the tenant and principal IDs. This key cannot
select an existing tenant for new authority.
Rust callers constructing EnrollRequest or EnrollResponse struct literals must
provide the new bootstrap_request_id field; None preserves the legacy wire shape.

The following IDs are illustrative; clients must generate their own request ID:

```json
{
  "kind": "human",
  "name": "operator",
  "bootstrap_request_id": "req_00000000-0000-7000-8000-000000000001"
}
```

A successful original response contains the complete creation outcome:

```json
{
  "tenant_id": "ten_00000000-0000-7000-8000-000000000002",
  "principal_id": "hpr_00000000-0000-7000-8000-000000000003",
  "kind": "human",
  "name": "operator",
  "boundary_id": "bnd_ten_00000000-0000-7000-8000-000000000002",
  "grant_id": "grt_hpr_00000000-0000-7000-8000-000000000003",
  "bootstrap_request_id": "req_00000000-0000-7000-8000-000000000001",
  "replayed": false
}
```

Resubmit the same request ID and name to the same authoritative store after a
lost response or `commit_outcome_unconfirmed`. A committed result returns all
these original fields with `replayed: true`. If the original transaction rolled
back, the same key can complete creation once. A concurrent request may still hit
a bounded lock/deadline error; retain the key for subsequent recovery. A new key
means a distinct logical bootstrap, even when the name is identical.

| Request or stored condition | Result |
| --- | --- |
| Same key and exact name | Original tenant, human, boundary and grant IDs; replay creates no authority. |
| Same key with a different name | 409 `idempotency_conflict`; existing binding/outcome remains unchanged. |
| Changed human `actions` list | Ignored by this dev endpoint; same-key/name replay still succeeds. |
| Noncanonical, malformed or non-v7 key | 400 `invalid_command` before database effects. |
| Key combined with a role or a non-null tenant_id | 400 `invalid_command`; keyed recovery is only for new-human bootstrap. |
| Wrong JSON type for the key | JSON extraction refuses with 422; no enrollment effects. |
| Omitted or null key | Existing no-key semantics; no keyed recovery guarantee. |
| Unsupported outcome version, malformed IDs/shape or broken source binding | Safe 500 `dependency_unavailable`; no replacement bootstrap or fabricated result. |
| Revoked or expired original authority | Historical keyed outcome remains recoverable; replay does not reactivate authority. |

Migration 0057 adds an outcome table with a canonical request key, unique tenant
mapping and tenant identity foreign key. It invents no receipts for legacy rows.
The request binding and versioned complete result commit in the same transaction
as all bootstrap rows. Service paths only insert/read these bindings. Database
maintenance, backups and retention must preserve the binding while the original
tenant is recoverable; deleting or reassigning it destroys that recovery guarantee.
Database-owner corruption is refused where detected, not treated as a supported
mutation path or a reason to mint another tenant.

A retry may read only the request-to-tenant route before knowing the original
tenant. It aborts its provisional attempt, then acquires that tenant's exclusive
guard before decoding the full outcome and checking identity/grant/parent
bindings. A concurrent loser likewise rolls back every provisional row and anchor.
At most one redirect shares the original fifteen-second budget; COMMIT errors keep
their unconfirmed phase. Replays validate historical identity bindings without
re-evaluating current grant liveness. This does not add caller authentication or
site-operator credentials.

Server qualification under `SIGNOFF-REPAIR.3.3.4.3.3.3.2` passes 73 selected
controls (72 live / one pure), a final eleven-control fixture rerun and strict
lint. All results/shutdown are consumed; all four owned clusters are absent.
`docs/tasks/artifacts/signoff_review/bootstrap-server.md` records the matched
baseline, observed contention, rollback, real commit/response-loss recovery,
malformed storage, migration and fifteen-second shared-budget controls. The next CLI child will persist pending requests,
validate replies and durably publish state before clearing pending recovery.
Until then, the existing CLI still needs operator reconciliation after an
unrecoverable no-key response loss. The complete selected contract is
`docs/decisions/2026-09-09_bootstrap-recovery.md`.

### Guard lifetime and failure handling

The guard accepts one to eight predeclared tenant/mode entries. Duplicate entries
take the strongest mode; full keys are sorted before locking. It cannot upgrade
an existing shared guard. A guarded executor checks its tenant and required mode,
while the caller remains responsible for actual permission and tenant-bound SQL.
Backfilled and newly created anchors confer no identity or authority.

| Example within the qualified primitive | Behavior |
| --- | --- |
| Two shared operations for tenant A | Both can hold the guard together. |
| An exclusive operation for A while a shared operation holds A | Wait until the shared operation finishes; a second remaining shared holder still excludes it. |
| An exclusive operation for B while A is occupied | B can progress independently. |
| An operation declares B shared, A shared, then A exclusive | Acquire A exclusive before B shared, irrespective of input order. |
| A callback returns a refusal as a successful value, with evidence intended to persist | Commit that value and its local evidence together. |
| A callback returns a typed domain error after provisional writes | Roll back those writes and any new anchor; preserve the exact error. |
| A callback encounters a SQL error or is cancelled | Do not return success; discard the connection and roll back unfinished work. Cleanup is asynchronous. |

The default lock limit is 5 seconds, the statement limit 10 seconds, and the whole
operation limit 15 seconds, including pool acquisition and commit. Internal limits
may be shorter, never longer or disabled. An operation samples database time when
it evaluates authority, after preceding waits. The guard itself does not freeze
time or prove that a later response was delivered before expiry.

The cancellation control found a setup gap before SQLx acknowledged BEGIN: the
same backend was returned to the pool still inside a transaction. The owner now
holds the connection before that await and permits reuse only after an acknowledged
successful commit. Tests check replacement of the cancelled backend and normal
reuse/reset after healthy commit. A commit acknowledgment deadline remains
**unconfirmed**, because PostgreSQL can still finish a COMMIT already sent. The
control observes that exact case and reads back the one original row; the runner
does not retry or invent a success receipt. This primitive creates no new HTTP
receipt or final-effect API; those interfaces retain their separately owned work.

All four crates that embed migrations now declare their migration directory as a
build dependency. This addresses the observed cached executable that lacked 0056
after an ordinary incremental build. The guard migration preserves every original
identity, boundary, grant and authorization row; even an invalid historical
grant/parent relationship is preserved as evidence, without inferring permission.
Qualification and exact failure/cleanup records are in
`docs/tasks/artifacts/signoff_review/tenant-guard-qualification.md`.
The final foundation gate passes 35 controls (13 live guard, one pure limits,
three upgrades and 18 existing authority), 28 migration rebuild/cache checks and
four-crate strict lint. All results and shutdown are consumed; owned clusters and
temporary probes are removed. This is qualification of the named primitive and
migration behavior, with application integration still pending.
Seven additional commit-checker controls ensure nested evidence files can accompany
their real owning task tree and cannot replace that tree. They also preserve
refusals for missing/unchecked boxes and unrelated evidence. This corrects file
selection; broader doctrine accuracy remains under `.11.2`.

Foreign inbox operations and shared registry mutations require their own corrected
authority checks and unchanged-state controls under `.3.5` and `.3.2`.

Site authority and workload certificate checks do not replace production caller
authentication. The development header trust and the open external G6/G7 gates remain
material deployment limits.
