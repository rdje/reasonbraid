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

#### One transaction, from the admission to the evidence

Revocation used to run **two** guarded transactions: the handler admitted the
caller under a shared guard, then called a service that opened a second one under
an exclusive guard. Between them the admission could be a fact about authority
that had already stopped holding, and the submitted reason and the final outcome
were never recorded with the mutation at all.

Both routes now run one transaction under the tenant's exclusive guard, holding
the decision time (database time sampled **after** the guard wait), the
admission, the tenant-bound target selection and row lock, the status change, the
revocation-epoch bump, and the final effect record.

| Order | Result |
| --- | --- |
| A revocation arrives while the tenant's exclusive guard is held | It waits; nothing is applied and no epoch advances until it acquires. |
| The caller's own administration ends while its revocation queues | The post-wait admission sees it ended: 403, and the target and epoch are unchanged. |
| An eligible administrator revokes a live target | 200; status becomes revoked, the epoch advances once, and the effect records `applied` with the submitted reason. |
| The same target is revoked again | 409 and no further epoch increment, with the effect recording `no_op`. |
| The target is missing, or belongs to another tenant | One identical 404, with the effect recording `refused`/`not_found` in the CALLER's tenant. |

Every answer — including each refusal — carries an
`x-reasonbraid-authorization` header naming the admission this request
committed, which is also the effect record's id when one was written. Without it
the effect record would be unreachable, because there is no list endpoint.

What commits an effect record is narrower than what commits an admission. A 403
denial and a 400 malformed reason record their admission only: the request never
became an operation, and the admission record already says what happened. A 404
does record a refusal, and it leaks nothing — the two cases are indistinguishable
to the caller, and the record names only the id the caller itself supplied.

`revoked_at` is now the transaction's own database time rather than a process
clock read after the fact, so the response and the effect record report the same
instant.

**Wire changes.** The submitted reason was already required; it now also has to
be at most 1 024 UTF-8 bytes and free of control characters, the same contract
[site authority](site-authority.md) publishes. A reason exactly at the ceiling is
accepted; one byte over is `400 invalid_command`, before any effect is recorded.
And both responses gained the receipt header described above.

Serialization against revocation of the acting administrator's own authority is
part of this: the admission is evaluated inside the same exclusive guard the
mutation holds.

### Arming and resetting a spend breaker

The [spend circuit breaker](budget.md) is a per-tenant latch, and its two
administrative verbs were the weakest administrative path in the server. Each
admitted the caller in its own transaction under a *shared* guard, and then ran
a bare statement **on the connection pool** — outside any transaction, under no
guard, and recording nothing about what the operation finally did.

Both now run one transaction under the tenant's **exclusive** authority guard,
holding the decision time (database time sampled after the guard wait), the
admission, the tenant-bound breaker selection and row lock, the mutation, and the
final effect record.

The mode is exclusive for two measured reasons rather than by analogy with
revocation. Deciding whether an arm actually changed anything is a read-then-write
over a row that **may not exist**, and a row lock cannot cover an absent row. And
the reservation path evaluates this latch inside thread-command transactions that
hold the *shared* guard, so exclusive mode is what orders an arm against every
in-flight reservation in the tenant — the latch never lags the ledger it guards.

| Order | Result |
| --- | --- |
| Another tenant operation holds the authority guard when a breaker request arrives | It waits; nothing is armed or reset until it acquires. |
| The caller's own administration ends while its request queues | The post-wait admission sees it ended: 403, and no breaker changes. |
| An administrator arms a breaker that does not exist, or changes its threshold, or re-arms a tripped one | 200; the effect records `applied`. |
| An administrator re-arms the **same** threshold on an untripped breaker | 200, and the effect records `no_op` — no column of the row changes, because re-arming only rewrites the threshold and clears a trip. |
| An administrator resets a tripped breaker | 200; the latch re-opens and the effect records `applied`. |
| An administrator resets a breaker that is armed but not tripped | 409, with the effect recording `no_op`. |
| An administrator resets when no breaker is armed at all | The **same** 409, with the effect recording `refused`/`invalid_transition`. |

The last two rows are the point. The wire deliberately answers both idle states
identically — its message has always said `no tripped breaker to reset (none
armed, or none tripped)` — and the effect record is where they stop being the
same fact. An operator reading `refused` knows the tenant has no spend ceiling
armed at all; reading `no_op` knows it has one and it simply has not tripped.
This leaks nothing: the caller is an admitted administrator of that tenant and
`GET /v1/admin/breakers` already shows it the same state.

The missing-breaker case records `invalid_transition` rather than `not_found`
because a breaker operation's target **is the tenant** — which is why
`breaker_arm` and `breaker_reset` are the two operations that carry no target
field — and the tenant was found.

Neither verb advances the tenant's revocation epoch, in any outcome including the
successful one. The epoch invalidates cached authority decisions; a spend latch is
not authority.

**Wire changes.** Request bodies, success bodies, status codes and messages are
unchanged. Both routes gained the `x-reasonbraid-authorization` receipt header on
every answer, for the reason revocation did: without it the effect record would
be unreachable, because there is no list endpoint. Neither verb takes a caller
reason, so neither records one — `submitted_reason` is `null`, and adding a reason
requirement to either would be a wire change to be documented and tested as one.

### Issuing a node enrollment token

An operator issues a one-time token bound to a tenant, an expected node id, a
host claim, a nonce and an expiry; the node consumes it once at
`POST /v1/nodes/enroll`.

**Who "an operator" is, exactly** (`SIGNOFF-REPAIR.4.1.4`): whoever holds the
`tenant_admin` grant in that tenant. That is a grant, not a kind of principal —
so **an agent role granted `tenant_admin` can issue an enrollment token, and can
revoke a node**, exactly as a human administrator can. This is deliberate and it
follows the authority model on this page: authorization is over typed actions
and resources, deny-by-default, and nothing in this system decides anything by
asking whether a principal is a person. The route's documentation used to say
"an authorized human", which was never what the code did; the sentence was the
mistake, not the behaviour.

⚠️ The practical consequence is worth stating plainly, because it is a
governance choice rather than an implementation detail: **granting
`tenant_admin` to an agent lets that agent extend the node population.** If a
deployment does not want that, the answer is to withhold the grant — not to
expect the route to refuse agents, which it will not. Narrowing it would be a
change to the grant model, not a check added to one endpoint. The issuing route used to admit the caller in its own
transaction and then insert **on the connection pool**, with nothing recording
what the request finally did.

It now runs one transaction under the tenant's **shared** authority guard,
holding the decision time, the admission, the token insert and the final effect
record.

**The guard is shared here, and that is a different answer from the breaker's on
purpose.** Deciding whether a token was issued or refused is not a
read-then-write over a row that may not exist — a single insert that does nothing
on conflict returns either the new token or no row at all, which is the whole
decision. Contention over the same node id is already settled by the partial
unique index that allows one *live* unused token per node. And issuance touches nothing
the reservation path reads. What the guard still has to do is fence the request
against an authority change, and a revocation's exclusive mode does that against
a shared holder. Taking the exclusive guard would have bought no invariant and
blocked every concurrent thread command in the tenant.

| Request | Result |
| --- | --- |
| An eligible administrator issues a token for a node with none outstanding | 200 with the token id, nonce and expiry; the effect records `applied`. |
| A **live** unused token for that node is already outstanding | 409 `invalid_command` with the established message; the effect records `refused`/`invalid_command`, and no token is created. |
| An earlier token for that node **expired** unused | 200: the lapsed token is stamped superseded and a new one is issued. |
| The caller does not administer the named tenant | 403; no token, and no effect — the admission record already says denied. |
| `node_id` is not a valid node identity | 400, **before any admission exists** — and therefore with no receipt, because there is no record to name. |
| `ttl_seconds` is outside `1 … 86 400` | 400 `invalid_command`, **before any admission exists** and before any arithmetic — same shape, same reason. |

`expires_at` is now the transaction's own database time plus the requested TTL
(3 600 seconds by default), so a token's lifetime runs from the instant the
decision was made rather than from a process clock read before the guard wait.
Every answer that reached an admission carries the
`x-reasonbraid-authorization` receipt; the validation refusals above deliberately
do not.

#### The lifetime has a range, and the range is enforced first

`ttl_seconds` must be between **1 and 86 400** (one day). A request outside that
range is refused `400 invalid_command` before anything computes with the value.

The upper bound exists because the token is a **bearer credential**: whoever
holds it can enroll that one node id from that one host claim, once. Everything
that makes that exposure acceptable rests on the token expiring soon. One day is
enough to prepare a node ahead of a working day; past that the answer is to issue
a fresh token, not to hold a long-lived one.

The lower bound exists because a token issued already-expired can never be
redeemed, and it is exactly the row that used to lock its node out for good (see
below). An operator error is refused rather than stored.

**The host claim is checked here too** (`SIGNOFF-REPAIR.4.1.6`). The claim
becomes the subject alternative name of the node's workload certificate, so a
claim the certificate library will not accept can never produce one. Such a
request is refused `400 invalid_command`, naming what is wrong, and no token is
written.

The check runs at issuance rather than at enrollment for a measured reason. It
used to run nowhere: the claim was stored as given, and the certificate library
was first asked about it when the *node* redeemed the token — where it raised,
so the node received a dropped connection rather than an answer, no node row was
written, and the token was left unconsumed. Because an outstanding unused token
refuses a second issuance for the same node id, that node id could then not be
enrolled until the token lapsed. Refusing at issuance costs the operator one
corrected field and makes that state unreachable.

The rule is the certificate library's own verdict, not a host-name grammar
written alongside it. Nothing that works today stops working: the check accepts
exactly what issuance would have accepted, and refuses only what could never
have produced a certificate. Whether a stricter grammar *should* bind — the
library accepts a good deal that is not a host name — is an open question with a
compatibility cost, recorded in
`docs/decisions/2026-09-14_host-claim-checked-at-issuance.md` and deliberately
not settled by that repair.

A stored host name that predates this check can still reach enrollment or
certificate rotation. Both now answer with a typed `400 invalid_command` instead
of dropping the connection.

⚠️ **This narrows the contract.** The field previously accepted any integer, and
three behaviours were measured before the bound was added: a century was issued
without complaint, expiring in 2126; a value past the date range aborted the
request *while holding the tenant's authority guard*; and `i64::MAX` aborted it
**before any authorization ran at all**, because the principal header is only
parsed at that point, not checked. The range check sits beside the `node_id`
shape check so that no caller-chosen value reaches the arithmetic.

#### A token does not outlive the authority that issued it

Revoking a grant or a boundary **voids** the enrollment tokens that authority
issued and has not yet had redeemed. Redeeming one afterwards is refused with its
own reason — *the authority that issued this token has been revoked* — which is
deliberately not "expired": waiting does not help, and neither does re-issuing
under the same withdrawn authority.

The reasoning is the one applied twice elsewhere in this chapter and in the node
channel: withdrawn authority stops producing effects. A redeemed token produces
an enrolled node, and an enrolled node is an effect.

**The check is at revocation, not at redemption, and that is the load-bearing
choice.** Redemption takes no tenant authority guard; re-reading a grant's status
there would read authority state outside the guard a revocation holds
exclusively. A revocation already holds that guard, and the token row is already
redemption's declared serialization point — so the row lock alone orders them. A
redemption that reaches the row first commits, and the node it creates is
separately revocable; one that arrives second reads a voided row and is refused.

The selection is exact. Each token records the authorization that issued it, and
that record already names the grant and the boundary it selected — so revoking
one administrator's grant leaves a colleague's outstanding tokens alone, even in
the same tenant. Revoking a *boundary* also voids tokens issued under the grants
beneath it, because the record names both.

⚠️ **A voided token releases its node's one-live-token slot**, so revoking a
compromised administrator never closes the node's enrollment path — the surviving
operator issues a fresh token immediately. Without that, this feature would
recreate the lockout described next.

**A token that lapses no longer locks its node out.** The partial unique index
allows one unused token per node, and an expired token is still unused — so a
token that was issued and never consumed used to occupy that index for ever. Every
later issuance for that node id answered 409, and the 409's own message told the
operator to *consume or expire it before issuing another*: expiring it was the
advertised recovery and the one thing that could not work, because the lapsed
token could not be consumed either. The node's ordinary enrollment path was closed
permanently.

The liveness cannot go in the index — a partial index predicate must be immutable
and `now()` is not — so it is written down instead. An issuance that finds a
lapsed token for its node stamps that token `superseded_at` inside the same
transaction and then inserts, and the index keys on the stamp as well as on use.
Two live unused tokens still collide, which is the property the index was narrowed
to buy in the first place. The lapsed row is kept rather than deleted, and it is
never stamped *used*: it was never redeemed, and recording it as used would make
the token store misdescribe what happened. The stamp is scoped to the admitted
tenant, so an issuance never supersedes a row belonging to someone else.

**What this route does not check, and who does.** `node_enrollment_tokens` has no
foreign key to `nodes`, because a token is issued *before* the node it names
exists. There is therefore no target row to bind to the caller's tenant at
issuance: the row is stamped with the admitted tenant, and whether a node may
later enroll into that tenant is lineage enforced at **redemption**, owned by
`SIGNOFF-REPAIR.4.1`. One related limit is recorded rather than implied: the
one-unused-token index is global rather than per tenant, so two tenants cannot
hold outstanding tokens for the same node identity at once. `SIGNOFF-REPAIR.3.5`
owns that.

### Revoking a node's certificates

Revoking a node marks its active workload certificates revoked and advances the
tenant's revocation epoch, so the handshake ladder refuses the certificates at the
next crossing and every cached node-side admission decision is invalidated.

The route used to do that in three pieces: an admission in its own transaction, a
tenant-bound existence probe as a **separate pool query**, and then a third
transaction whose `UPDATE` matched on `node_id` alone. The check and the act read
two different snapshots, and nothing recorded what the request finally did.

All of it is now one transaction under the tenant's **exclusive** authority guard.
The mode is exclusive — unlike token issuance, which is shared — because this
advances the revocation epoch: it is a revocation in the sense the guard contract
means, and the exclusive mode is what fences every admission that has not already
selected its evidence. The node row is selected `FOR UPDATE` bound to the admitted
tenant, and the `UPDATE` carries its own tenant predicate rather than trusting the
probe that preceded it, so the binding is visible in the statement that writes.

| Request | Result |
| --- | --- |
| An eligible administrator revokes a node with active certificates | 200; the certificates are revoked, the epoch advances once, and the effect records `applied` with the submitted reason. |
| The same node is revoked again | 409 and no further epoch increment, with the effect recording `no_op`. |
| The node exists but has never had a certificate | The **same** 409, with the effect recording `refused`/`invalid_transition`. |
| The node is missing, or belongs to another tenant | One identical 404, with the effect recording `refused`/`not_found` in the CALLER's tenant. |
| The caller does not administer the named tenant | 403; nothing changes and no effect is recorded. |
| The reason is blank, over 1 024 bytes, or carries a control character | 400 before any effect is recorded; the admission still commits. |

**Wire changes.** The reason was already required; it now also has to be at most
1 024 UTF-8 bytes and free of control characters — the same contract
[site authority](site-authority.md) and grant revocation publish — and it is now
**persisted** with the effect rather than checked for blankness and discarded.
`revoked_at` is the transaction's own database time, so the response, the
certificate rows and the effect record report the same instant. Every answer
carries the `x-reasonbraid-authorization` receipt.

What a revoked node then experiences — the refused handshake, the suspended
presence, the lease that is not cut — is unchanged and documented in
[the node channel](node-channel.md).

### Administering a node's inbox

Three operator verbs act on one node's durable inbox: **quarantine** marks one
command so it is never re-delivered, **replay** reverses that quarantine and
re-sequences the command to the tail, and **prune** deletes finished rows older
than a window. Each now runs one transaction under the tenant's **shared**
authority guard, holding the admission, a tenant-bound row selection, the
mutation and the final effect record.

**The repair here is a cross-tenant one.** `node_inbox` has carried a `tenant_id`
column since the migration that created it, and none of the three verbs used it.
Measured against the superseded routes, with an administrator of tenant A acting
on a node whose inbox rows belong to tenant B:

| Verb | Answer before the repair |
| --- | --- |
| Quarantine a foreign row | `200` — the row was quarantined |
| Replay a foreign row | `200` — the row was un-quarantined and re-sequenced |
| Prune a foreign inbox | `200 {"deleted":2,"before":2,"after":0}` — **both of the other tenant's rows were destroyed** |

Every selection is now bound to the admitted tenant, so a foreign target is
indistinguishable from an absent one and mutates nothing. A foreign prune reports
`deleted: 0` with `before: 0`, because the counts describe the inbox the caller is
allowed to see rather than every row the node happens to hold.

The guard is shared: none of the three writes authority or advances the
revocation epoch, so none needs to fence other operations. What they need is to be
fenced BY a revocation, which the shared mode provides. Exactness against another
administrator comes from a row lock, at the granularity that actually conflicts.

⚠️ **The fourth verb, and why it was late.** `GET /v1/nodes/inbox` — the
inspection — is not a mutation, and the census behind the repair above scoped
itself to the node administrative *mutations*. It was correct about its own scope
and silent about the read, which therefore kept selecting by node id alone until
`SIGNOFF-REPAIR.3.5.3`. Measured against that superseded route, an administrator
of tenant A naming its OWN tenant and tenant B's node received both of B's rows
in full — command ids, thread ids, delivery state and payloads.

⭐ The general shape is worth carrying: **the caller supplies two independent
identifiers, and the handler checked one of them.** The admission proves you
administer the tenant you *named*; nothing tied that tenant to the node. Compare
`GET /v1/calls/{call_id}`, which loads the call first and authorizes against the
call's OWN tenant — there the tenant is derived from the target, so the two
cannot disagree. The inspection keeps its caller-supplied tenant (removing a
required parameter is a wire change on a public route) and now carries it into
the select, so a node outside your tenant returns an empty row list rather than a
refusal. It takes no transaction and no guard, deliberately: a read has no
check-then-act, so an older snapshot cannot produce an unauthorized effect.

| Request | Result |
| --- | --- |
| Quarantine a live command | 200; the effect records `applied` with the submitted reason. |
| Quarantine a command that is already quarantined | 409; the effect records `no_op`. |
| Replay a dead-lettered command | 200; the quarantine is reversed and the effect records `applied`. |
| Replay a command that is not dead-lettered | 409; the effect records `refused`/`invalid_transition`. |
| Prune, with rows old enough | 200 with the measured `before`/`deleted`/`after`; the effect records `applied`. |
| Prune, with nothing old enough | 200 with the same measured receipt and `deleted: 0`; the effect records `no_op`. |
| Any verb naming a command or node outside the caller's tenant | The same answer an absent one gets, and nothing changes. |

**Wire changes.** The quarantine reason was already required; it now also has to
be at most 1 024 UTF-8 bytes and free of control characters, and it is persisted
with the effect as well as on the row. Timestamps in the responses
(`quarantined_at`, `replayed_at`, `cutoff_at`) are the transaction's own database
time. Every answer carries the `x-reasonbraid-authorization` receipt. One message
is retired: a replay that lost a concurrent race used to say so, and the row lock
now makes that state indistinguishable from any other command that is not
dead-lettered, so it receives that same typed answer instead of a claim about a
race the lock no longer permits.

The retention rule is unchanged: prune never deletes a **quarantined** row, so an
age-based sweep cannot destroy quarantine evidence.

**Prune removes two classes of row, and its receipt names them separately.**
Rows the node acknowledged holding are aged by that acknowledgement. Rows that
were never delivered because the grant admitting them passed its own expiry —
§10.6's `expired` — are aged by that expiry, which is the exact instant the row
entered the terminal, so the window measures time *in* the state being retained.
The response carries `deleted_delivered` and `deleted_expired` beside the total,
because a receipt saying *delivered* about work that was never handed over is a
false retention statement.

⛔ **A `revoked` row is not prunable, and that is a measured omission rather than
an oversight.** Revoking a grant writes `status = 'revoked'` and nothing else:
`authority_grants` has no `revoked_at`, so nothing records *when* the authority
was withdrawn. A window aged by any other column would delete work the operator
was told they could still see. Those rows are retained until that column exists.

Which decision facts a replay refreshes is deliberately not changed here — that
remains `SIGNOFF-REPAIR.3.4`. Only the transaction they are refreshed in, the
tenant binding, and the clock they read have moved.

### The final administrative effect record

An admission record says a caller **was allowed to ask**. It says nothing about
what the local mutation then did. Collapsing the two would let an operator read
`allowed` as `applied`, so the final effect is a separate, additive record.

**Three families write one today.** `SIGNOFF-REPAIR.3.3.4.7.1` defines the
representation, `.7.2` gives it durable storage, `.8` makes grant and boundary
revocation its first producer — described under [one transaction, from the
admission to the evidence](#one-transaction-from-the-admission-to-the-evidence) —
`.9` adds [spend-breaker administration](#arming-and-resetting-a-spend-breaker),
and `.10` adds the five node administrative operations — [token
issuance](#issuing-a-node-enrollment-token), [certificate
revocation](#revoking-a-nodes-certificates) and [inbox
administration](#administering-a-nodes-inbox). `.11` and `.12` follow; until each
does, its operations have no effect record, which reads as an absence and never
as a success.

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
against §9.8 could not say what the response said. These are instead what the
administrative handlers actually refuse an admitted operation with, and their
wire names are literally the strings in the response body. The registry's own
reconciliation is tracked separately as `SIGNOFF-REPAIR.11.7`; it is a wider
finding than this chapter.

⚠️ The set was **three** until `SIGNOFF-REPAIR.3.3.4.7.4`, and the correction has
the same shape as the original mistake. That census classified every
`unauthorized` in the fourteen handlers as the admission's own denial — which is
already the authorization record's job and writes no effect — and it was right
thirteen times. The fourteenth is the card import's **allowlist rung**, which
refuses a caller who WAS admitted, because the importing tenant holds no
effective federation agreement with the card's origin: a precondition about the
two tenants, not about the caller's grant. Re-measured across all fourteen
handlers, there is exactly one such site. A denied admission still writes no
effect record at all, so a `refused` outcome carrying `unauthorized` always means
the request was allowed and the operation was not.

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

### Acting on behalf of someone else

A command envelope may carry an `authority_context`: the actor keeps its own
identity, and names a **subject** whose grant is the authority source, with the
scope it is claiming. §16.3's invariants bound it, and four gates enforce them —
all four, on every delegated request:

| Gate | What it refuses |
| --- | --- |
| the actor's own authority | an actor whose own grant does not cover this action **and this target**. The caller is evaluated against the same target as the subject, so a delegation reaches nothing the actor could not reach alone. |
| the subject's authority | a subject whose grant does not cover the action or target |
| the widening invariant | a claimed scope wider than the subject's grant, or narrower than the request's own target. A delegated request always carries a scope — the subject and the scope are one value, so there is no request shape in which this gate is skipped and the subject's full grant selector applies instead (`SIGNOFF-REPAIR.3.4.1.1`). |
| participation | an actor who is not a participant of the thread — **naming a well-placed subject does not launder an outsider in**, and the refusal names the actor, not the subject |

The audit record names the **subject** as the authority source, and the actor as
the actor; forwarding preserves both.

#### The subject is not asked, and that is deliberate

**A delegation here is trusted impersonation inside one tenant.** The subject
does not consent to it and cannot refuse it; its grant supplies the authority and
the audit names it alongside the actor.

Consent was never a delegation requirement in this project, which is worth saying
plainly because the word appears elsewhere and means something else. The
roadmap's consent is §4.4's — an *enrollment* act, carried on the enrollment
authority boundary as a disclosure shown to the target owner before enrollment
completes. §16.3, where the delegation invariants live, states six of them and
subject consent is not among them.

Because the four gates above hold, **a delegation reaches nothing the actor could
not reach alone**. What the subject loses is not access but narrative: an
authorization record — which §16.9 treats as high-impact evidence — names it as
the authority behind an act it never agreed to. That cost is accepted rather than
dismissed, and the reasoning is recorded in ADR-009: a subject cannot express
consent without something to issue and something to verify, which is the
capability-token option that ADR subtracted, wearing a different name.

It is revisited if a delegation can ever cross a tenant boundary, if a subject
need not be an enrolled principal of the same tenant, or if any of the four gates
is relaxed — each of which fails a named control rather than needing a judgement.

#### The idempotency key is bound to the authority context and the target

A command's idempotency hash covers the operation, the actor, the request body,
**the authority context when the request carries one** (`SIGNOFF-REPAIR.3.4.2`)
and **the target the command acts on** (`SIGNOFF-REPAIR.3.4.6`). It has to,
because the idempotency claim is made *before* authorization and a replay
returns the stored result without evaluating anything:

| | Superseded hash | Now |
| --- | --- | --- |
| same key and body, delegating to a **revoked** subject | `200 replayed=true`, `ok: true` | `409 idempotency_mismatch` |
| authorization records written by that second request | **zero** | — |
| genuine replay: same key, body and subject | the original result | unchanged |

An unauthorized delegation was being told it had succeeded, with nothing in the
audit showing it was ever attempted. `authority_context` is a sibling of `body`
in the envelope, so hashing the body alone could not see it.

⚠️ The hash is a **stored** value, so changing its inputs is a wire change. A
request with no authority context hashes exactly as before — the delegated suffix
is appended only when there is one — so every historical undelegated key keeps
replaying. A historical *delegated* key now conflicts instead of replaying, which
is the safe direction: it refuses rather than returning a result decided under a
different authority.

The **target** half closed a second way for a replay to answer the wrong
question. A thread command's thread arrives as a path segment and no typed body
carries it, while `idempotency` is keyed on `(tenant_id, idempotency_key)` — a
*tenant-wide* key. So the same actor, body and key against a **different thread
in the same tenant** hashed identically, and the second request replayed the
first thread's stored result without thread two ever being looked at:

| | Superseded hash | Now |
| --- | --- | --- |
| same actor, body and key, against a **different thread** | `200 replayed=true`, carrying thread **one**'s `thread_id` and `event_id` | `409 idempotency_mismatch` |
| genuine replay: same actor, body, key and thread | the original result | unchanged |

The target is bound exactly where a caller can vary it independently of the key —
the eleven operations of `POST /v1/threads/{thread_id}/commands`. Three other
callers pass no target and each has its own structural reason: a **creation** has
no thread yet and its target is the tenant, which is already the first column of
the idempotency primary key; the **MCP respond tool** derives its key as
`mcp_respond_{thread}_{hash}`, so the thread is fixed inside the key; and a
**node result** is keyed by the server-assigned `command_id`, which belongs to
exactly one thread. Their historical keys therefore hash byte-identically and
keep replaying.

#### What `delegable` and `max_delegation_depth` do not do

`AuthorityGrant.delegable` and `EnrollmentAuthorityBoundary.max_delegation_depth`
describe a different mechanism: **grant chains** — issuing a further grant *from*
an existing one. Nothing issues such a grant. Every grant's parent is an
enrollment boundary, never another grant, so the machinery has no producer and
there is no depth to bound. The flag is read in exactly one place, at issuance,
to refuse a grant that claims to be delegable under a boundary that is not.

⚠️ They do **not** gate the on-behalf-of path above, and the names invite the
opposite reading. Every dev-profile grant is issued `delegable: false`, under a
boundary that is `delegable: false` with `max_delegation_depth: 0` — so if the
flag gated that path, delegation could never have worked at all. Two controls pin
this, so a reader who notices `evaluate()` never consulting `delegable` finds the
answer rather than re-deriving a wrong one. `SIGNOFF-REPAIR.3.4`'s own census
recorded it as a defect before `.3.4.1` measured what the flag is for.

⛔ One thing is genuinely not required: the subject's **consent** to this
particular actor. Its consequence is bounded to attribution — the actor must
already hold a covering grant and already be a participant, so it gains no reach,
only the audit naming the subject alongside it. Chains, and the depth bound that
would come with them, are a later phase.

### Administering a federation direction

A federation agreement is **both-sides**: each tenant records its own direction
row, and the agreement is EFFECTIVE only when both are `accepted` and both carry
the capability in question. A one-sided proposal widens nothing, and a revoked
direction falls back to the network pseudonym.

The three verbs — `POST /v1/federation-agreements`, `…/accept` and `…/revoke` —
each run ONE transaction under the LOCAL tenant's **exclusive** authority guard
(`SIGNOFF-REPAIR.3.3.4.12`), holding the admission, the direction mutation, the
acceptance's cross-domain receipt and the final effect record together. Before
this, each admitted through a shared-guard transaction that had already
committed and then mutated on the connection pool, with nothing recording what
the request did.

**One guard key, not two.** Each verb writes only the local tenant's own row —
the pairing is both-sides precisely so that neither side mutates the other's —
and the acceptance's receipt is a local row naming a remote reference. A proposal
additionally reads `tenants` to check its counterparty exists, which is the
minimal foreign-ID existence probe the guard contract permits.

**Why exclusive.** Not because a direction is a revocation in the epoch sense —
none of these advances the tenant's revocation epoch. It is the other reason: all
three are a classify-then-write over a row that **may not exist**, deciding
`applied` against `no_op` or a refusal from how many rows their statement
touched, and a row lock cannot cover an absent row. Under `READ COMMITTED` a
concurrent proposal can insert the direction between another verb's look and its
write.

The wire is unchanged, and three answers that were previously indistinguishable
are now distinguished in the record:

| Request | Answer (unchanged) | What the record says |
| --- | --- | --- |
| propose a new direction, or change its terms | `200 {"agreement_id":…,"status":"proposed"}` | `applied` |
| re-propose on **identical** terms | the same `200` | `no_op` — nothing changed |
| accept a proposed direction | `200 {"status":"accepted"}` | `applied`, with the cross-domain receipt in the same commit |
| accept one **already accepted** | `409 invalid_transition` | `no_op` — already satisfied |
| accept one **never proposed**, or revoked | the same `409` | `refused` / `invalid_transition` |
| revoke a live direction | `200 {"revoked":1}` | `applied` |
| revoke one already revoked or never recorded | `200 {"revoked":0}` | `no_op` |

One refusal is **new**, and it replaces a `500`.
`federation_agreements.remote_tenant_id` references `tenants`, so proposing to a
tenant that does not exist used to raise a foreign-key violation — which, once
the transaction carries the admission and the effect record, would make the
refusal unrecordable. The counterparty is now checked before the insert and the
answer is `404 not_found`, recorded. A proposal has to name a real counterparty,
so distinguishing a real tenant from an absent one is intrinsic to the operation
rather than an incidental disclosure; tenant ids are unguessable, so it is an
existence check on an id the caller already holds.

`accepted_at` is now the transaction's own database time rather than `now()`,
which is `BEGIN` time — the instant *before* the guard and admission waits the
acceptance queued behind.

**What this fences, including the other side.** A card import declares BOTH
tenants in one predeclared sorted set (`SIGNOFF-REPAIR.3.3.4.12.1`): the
importing tenant exclusive, because it issues a grant, and the origin tenant
shared, because it reads the origin's agreement row. Since each direction verb
takes its own tenant's key exclusively, an import is now fenced by a revocation
from **either** side.

The set is declared to the runner, which sorts it, and that is what prevents lock
inversion — two single acquisitions could not, and the guard API deliberately
offers no later upgrade. An origin id that does not parse as a tenant id declares
no second key: there is no such tenant, so there is no state to order against,
and the allowlist rung refuses inside the transaction exactly as it always has.

⛔ Re-proposing with **different** terms resets an accepted direction to
`proposed` and clears its acceptance. That is preserved exactly as it was — it is
how terms are changed — but it means a direction can stop being effective without
anyone calling revoke. What the effective-agreement consumers are guaranteed
across such a change is `SIGNOFF-REPAIR.5.3`'s, not this chapter's.

### Importing a portable agent card

`POST /v1/profiles/cards/import` runs the ADR-027 ladder over a card and, if
every rung passes, creates a fresh LOCAL role under the importing tenant's own
boundary. The card's self-asserted capabilities never confer authority — the
local grant is the only authority that acts, which is the ADR-026 invariant.

It runs ONE transaction under the importing tenant's **exclusive** authority
guard (`SIGNOFF-REPAIR.3.3.4.11.3`), and the mode is derived rather than chosen:
the import **issues a grant**, and authority issuance takes the exclusive guard,
so a shared acquisition could not create the grant at all.

Everything is now inside that transaction, in this order:

| Step | What it does |
| --- | --- |
| Admission | `tenant_admin` for the importing tenant, evaluated on the transaction's own connection at database time sampled after the guard wait. |
| Digest and compatibility rungs | Pure. The digest must re-derive from the card's own canonical bytes, and the schema version must be the supported one. |
| Allowlist rung | The EFFECTIVE recruitment agreement with the origin — both sides accepted, both carrying `recruitment`. |
| Boundary | The importing tenant's active enrollment boundary, read in the same transaction that then issues against it. |
| Grant, identity, quota, enrollment, receipt | The default local grant, the `agent_roles` row, the per-principal quota row, the enrollment row and the cross-domain receipt. |
| **Profile** | The card's profile, written as the local role's first version. |
| Effect record | `profile_card_import`, with the outcome. |

🔴 The last two lines are the repair. The superseded route committed everything
above the profile and then wrote the profile **on the connection pool**:

| | Superseded route | Now |
| --- | --- | --- |
| A profile write that fails | `500`, with the role, its grant, its quota row, its enrollment and its receipt already durable | `500`, and nothing was written |
| What the directory then held | an identity with no profile to describe it | — |

The boundary and the agreement were read outside the transaction that used them
too, so a boundary revoked in between produced a grant checked against authority
that had already ended.

⭐ **What moving the agreement read inside buys, and what closed the rest.**
The read inside the transaction buys one consistent snapshot with the writes that
depend on it. When `.3.3.4.11.3` landed that was ALL it bought: the three
direction verbs took no tenant authority guard at all, so no guard set here could
fence a concurrent agreement revocation. `SIGNOFF-REPAIR.3.3.4.12` then put each
verb under its own tenant's exclusive guard, and `.3.3.4.12.1` made this import
declare both tenants' keys in one sorted set — so the import **is** now fenced by
a revocation from either side, as [administering a federation
direction](#administering-a-federation-direction) describes.

⛔ That paragraph said the opposite for two leaves after it stopped being true,
and so did the same sentence in two source files. Both sentences were accurate
when written and neither was revisited by the repairs that falsified them; the
correction is `SIGNOFF-REPAIR.3.3.4.12.2`, which also deleted the three
superseded services the sentence named. It is recorded rather than quietly fixed
because nothing mechanical could have caught it: each sentence was well-formed,
and only their conjunction was wrong.

The effect record's target is the digest the server **re-derives** from the
submitted card, not the one the caller presented. On every path but one they are
the same string; when the digest rung refuses, the presented value is by
definition not a digest of that card, while the re-derived one still names the
card the request actually carried.

The allowlist rung is the reason the refusal vocabulary has four codes rather
than three: it refuses a caller who *was* admitted, answering `403` with body
code `unauthorized`, and the record says the same word.

The wire is unchanged — every status, body code and message, including the
grant-refusal message, which is now composed by one renderer shared with the
effect record so the two cannot drift apart. One addition: every answer that
reached an admission carries the `x-reasonbraid-authorization` receipt.

One refusal is **new**, and it replaces a `500`. `agent_roles` carries
`UNIQUE (tenant_id, name)` and `enrollments` carries
`UNIQUE (tenant_id, kind, name)`, while the import writes the card's
`display_label` into both — so a card whose label is already taken in the
importing tenant used to raise a constraint violation and answer
`500 dependency_unavailable`, recording nothing. Re-importing the same card was
the commonest way to reach it; a card from a different origin sharing a label is
another. Both inserts now use `ON CONFLICT … DO NOTHING RETURNING`, so the
collision is a value rather than a raise, and it answers `400 invalid_command`
with an effect record that says the same thing (`SIGNOFF-REPAIR.3.3.4.11.5`).

⛔ That refusal says the **label is taken**. It deliberately does not decide what
a repeated import *ought* to do — the ordinary enrollment route answers a
`(tenant, kind, name)` collision with a replay, returning the original principal
id, and whether a card import should do the same is card replay semantics owned
by `SIGNOFF-REPAIR.5.3`. Making the import atomic is also what turned this from a
survivable mess into an unrecordable one: once the admission and the effect record
share the transaction, a raised constraint takes them down with it.

With this, the three unguarded bridges that existed only for this route are
**deleted** rather than left beside their replacements: the unordered grant
creator, the pool-taking boundary loader whose own comment called itself "an
explicit temporary bridge until its complete integration", and the pool-taking
profile writer.

### Attesting a capability claim

`POST /v1/profiles/{role_id}/attest` is the owner's path: §10.1 lets a role
declare its own capabilities, but only as `self_asserted`, and the upgrade to
`owner_attested` rides this audited verb instead. It is gated on `tenant_admin`
for the **role's own** tenant — an administrator of another tenant is denied,
because the admission is evaluated against the tenant the role belongs to, not
the one the caller administers.

It runs ONE transaction under that tenant's **shared** authority guard
(`SIGNOFF-REPAIR.3.3.4.11.2`), holding database time sampled after the guard
wait, the admission on that connection, the role's version anchor taken
`FOR UPDATE`, the read of the current profile, the single claim's upgrade, the
new version, and the final effect record.

The mode is shared and the reason is measured rather than inherited: an
attestation writes no authority and advances no revocation epoch, so it is not a
revocation in the sense the guard contract means — what it needs is to be fenced
**by** one, which shared mode gives, since a revocation takes the guard
exclusively. Exactness against a concurrent attestation comes from the anchor
lock, at the granularity that actually conflicts.

🔴 What that repaired is not a theoretical window. The superseded route read the
current profile **on the pool** and wrote it back through a different
transaction. Two administrators attesting two different capabilities of one role
each read version N, and each wrote a profile carrying only their own upgrade:

| | Superseded route | Now |
| --- | --- | --- |
| Both callers' status | `200` and `200` | `200` and `200` |
| Claims upgraded in the published profile | **one of the two** | both |
| Error reported to the losing caller | none | — |

Serializing the writes alone would not have fixed it, which is why the read moved
too: the losing writer's version *number* was correct while its *content* was
stale.

The wire is unchanged. The `404` still answers "no profile for this role" and "no
capability by that taxonomy id" with the same message, and the effect record is
where they stop being the same fact — both record `refused` with `not_found`,
alongside the `applied` an upgrade records. One addition: every answer that
reached an admission carries the `x-reasonbraid-authorization` receipt naming it,
which is also the effect record's id. The pre-admission `404` for an unknown role
deliberately carries none — no record exists yet to name.

A repeat attestation of an already-attested claim still writes a new version, as
it always has, and records `applied` rather than `no_op`: a new version row is
protected state changing, so `no_op` would be the wrong claim about it.

### Serializing a profile write, where no authority is being decided

Not every write that needs a transaction needs a **guard**, and the profile
writer is the worked example of the difference (`SIGNOFF-REPAIR.3.3.4.11.1`).

`PUT /v1/profiles/{role_id}` is gated on identity — only the role itself writes
its own profile, the owner's path being the attestation verb — so it evaluates no
grant and produces no authorization record. There is no authority decision for a
tenant guard to order it against, and taking one would block every concurrent
operation in the tenant to buy nothing. It takes no guard, deliberately. The two
admitted routes in the same family, capability attestation and card import, do
take one, because theirs are admissions.

What it does need is **serialization against itself**. Every write is a new
content-addressed version, and the version number is per role, so two concurrent
writers for one role must not choose the same number. `profile_versions` carries
`UNIQUE (role_id, version)`, and leaving that constraint to decide was measured
to be the wrong answer twice over:

- It refuses rather than queues. Two writers read the current version, both add
  one, and the second receives a `500` carrying `duplicate key value violates
  unique constraint`, where it should have received the next version.
- A constraint violation **aborts the transaction**, so once a write has to
  record anything about itself — an admission, an administrative effect — the
  refusal cannot commit alongside the evidence at all.

So the writers serialize at the role's own anchor row in `agent_profiles`, and
the anchor is created **inside** the acquisition rather than before it:

```sql
INSERT INTO agent_profiles (role_id, current_version, updated_at)
VALUES ($1, 0, $2) ON CONFLICT (role_id) DO NOTHING;
SELECT current_version FROM agent_profiles WHERE role_id = $1 FOR UPDATE;
```

The order matters and is not cosmetic: `SELECT … FOR UPDATE` over a row that does
not exist yet matches nothing, locks nothing and returns immediately, so a
lock-only fix would protect every write except a role's **first** — the one case
where the anchor has not been created. This is the same two-step the tenant
authority guards themselves use for first-use acquisition. The `UNIQUE`
constraint stays as the backstop that proves the lock is working; it should
never fire.

One wire-visible value changes and nothing else: `written_at` is now the write
transaction's own database time rather than a process clock read taken before
the anchor wait, so a writer that queued behind another records when it actually
wrote. Status codes, the response fields and the content addressing — identical
content still hashes identically — are unchanged.

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
The exact scope and remaining policy owners are recorded in
`docs/tasks/artifacts/signoff_review/tenant-authority-paths.md` and
`docs/decisions/2026-09-09_tenant-authority-transaction-order.md`.

#### What is on the shape, and what is not

`SIGNOFF-REPAIR.3.3.4.13` re-derived that census after `.8`–`.12.1` landed, with
instruments rather than a re-reading. The direct named-call surface fell from 42
locations to **27** while the corpus grew from 101 files to 118 — the shape the
integration predicted, since callers stopped naming an authority function
themselves and started entering a guarded service that names it once.

A second instrument classifies each of the **118 registered routes** by the gate
it actually reaches, following delegation rather than reading one handler body:

| Gate reached | Routes | Of which mutate |
| --- | --- | --- |
| **guarded transaction** | **18** | **18** |
| pool tenant-admin inspection | 9 | 0 |
| pool tenant-admin | 11 | 3 |
| pool authorize | 6 | 1 |
| identity only | 74 | 42 |

⛔ **Read the last three rows carefully.** 46 mutating routes do not run on a
guarded transaction. Every one has a named repair owner — policy (`.9.1`),
evaluation and routing (`.8.2`), evidence (`.7.4`), recruitment (`.5.2`),
resolvers and resources (`.7.1`), deployment (`.9.3`), regions (`.3.2`), adapters
(`.10.1`/`.10.2`), workflow (`.8.1`), directory matching (`.5.1`) — and being
owned is not being repaired. This chapter certifies the eighteen and says nothing
about the forty-six beyond naming who owns them.

The one route deliberately excluded rather than deferred is
`PUT /v1/profiles/{role_id}`: it is gated on identity rather than on a grant, so
it produces no authorization record and there is no admission for a guard to
order. It is [serialized instead](#serializing-a-profile-write-where-no-authority-is-being-decided).

No obsolete bypassing executor remains: 84 functions are declared across the
authority modules, 10 are never called, and all 10 are `#[test]` functions the
harness reaches by name.

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
HTTP revocation admission, submitted reason and final effect are now ONE
transaction under `.3.3.4.8`, described above; the two superseded revocation
services were removed rather than left as a second, unordered path.

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
