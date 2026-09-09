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
a fabricated authority decision. The audit schema and digest input format remain
unchanged. These lookups do not establish transaction-wide revocation ordering.

The separate frozen-tenant administrative read helper remains `.3.3.3.2`; it
needs structural and candidate checks while preserving its approved boundary-status
exception. Delegation depth, consent and cached-decision freshness
remain `.3.4`; tenant authority/effect transaction ordering remains `.3.3.4`.

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

## Administrative authority

Tenant administration is intended to remain tenant-scoped. The approved frozen
boundary carve-out allows an otherwise eligible tenant administrator to inspect its
own tenant after boundary revocation. It does not grant authority over other tenants.

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
