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
distinguishable from ordinary boundary-checked admission. Decision time comes
from the database clock after acquiring the transaction connection.

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

Foreign inbox operations and shared registry mutations require their own corrected
authority checks and unchanged-state controls under `.3.5` and `.3.2`.

Site authority and workload certificate checks do not replace production caller
authentication. The development header trust and the open external G6/G7 gates remain
material deployment limits.
