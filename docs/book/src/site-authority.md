# Site authority for shared registries

The server now has a separate site-authority service for the shared adapter and
region registries. Its schema and library entry points are implemented under
`SIGNOFF-REPAIR.3.2.1`. Operator CLI commands and HTTP enforcement are the following
children, `.3.2.2` and `.3.2.3`; the existing HTTP handlers still use the legacy
tenant-admin check until that work lands. The service's ten live controls and
strict focused lint pass; this does not qualify the pending HTTP integration.

## Who controls site authority

A deployment operator issues a **site boundary**, then a **site grant** naming
that exact boundary and a human or agent-role subject. A boundary is the maximum
permitted action set and validity window. A grant can narrow those permissions.
Tenant enrollment creates neither object and existing tenant grants receive no
automatic upgrade. Existing adapter and region entries confer no authority.

Issuance and disabling derive their issuer from PostgreSQL's session role. They
require a database superuser or direct/indirect membership in the deployment-managed
`reasonbraid_site_operator` role. The migration creates no database role or initial
site grant. A role membership is an application permission gate; the selected
database login must also have the table privileges needed by the operation.
Database administrators remain the trust root and can change database contents
outside the supported service.

This identity is the database session user, whose distinction from the current
effective role is described in PostgreSQL's
[session information functions](https://www.postgresql.org/docs/16/functions-info.html).
It is never accepted as a caller-supplied issuer string. Credential strength is a
deployment responsibility: the disposable tests use loopback trust authentication,
and the development HTTP principal header remains a development assumption.

## Capabilities and examples

| Capability | Authorized service operation |
| --- | --- |
| `registry_inspect` | List shared adapters, regions and directed region pairs. |
| `adapter_allow` | Add an adapter to the shared allowlist. |
| `adapter_revoke` | Remove an adapter from that allowlist. |
| `region_declare` | Declare a shared region. |
| `region_pair` | Add a directed pair between two declared regions. |
| `region_unpair` | Remove a directed pair. |

For example, Alice may administer tenant A but have no site grant. Her tenant
grant does not authorize any of these service operations. A deployment operator
can issue Alice a site grant containing only `registry_inspect`; she can then
inspect the shared registries while a region declaration is refused. A separate
grant containing `region_declare` permits declarations within its actual parent's
ceiling. A newer read-only grant does not hide that usable write grant.

A site grant's subject has an explicit kind and ID in issuance receipts and audit
targets. These are illustrative payload fragments, not a new HTTP request format:

```json
{"kind": "human", "id": "hpr_<human-UUID>"}
```

```json
{"kind": "role", "id": "rol_<role-UUID>"}
```

Registry names preserve their exact spelling, including Unicode and punctuation;
they must be nonblank, contain no control characters and fit within 256 UTF-8
bytes. Every mutation requires a nonblank reason, without control characters,
within 1,024 UTF-8 bytes. Blank or overlong input is rejected before a command is
formed. Action names are a closed set, so `tenant_admin` is not a site capability.

## Expiry, suspension and revocation

Both boundary and grant must be active and current. Validity includes `valid_from`
and excludes `expires_at`. Timestamps are normalized to PostgreSQL microseconds
before checking that the grant's window fits its parent. This permits an identical
parent/child window without an accidental nanosecond-rounding refusal. Explicit
future provisioning is supported, but it grants no access before validity begins.

Issuance facts are immutable. In particular, an existing grant cannot be moved
to a different boundary. Revoking an old boundary and issuing an unrelated new
boundary does not rearm its grants. Restoring access requires explicit issuance
of a new usable grant and, where necessary, a new boundary.

Suspension and revocation only remove authority. A suspended record cannot become
active again; it can be revoked. A revoked record remains revoked even if a later
request asks to suspend it. Repeating an already satisfied disable operation
returns an attributable no-op rather than rewriting issuance history.

## Transaction and audit behavior

Each supported site operation takes the same database guard before reading
authority or changing the registry. This serializes the small configuration
control plane, including site grant/boundary issuance and disabling. Transactions
contain database work only, use READ COMMITTED, and set five-second lock and
ten-second statement timeouts. Contention or database errors fail the operation;
they do not become an authorization success or a fabricated domain refusal.

Decision time comes from the database after the guard wait. A grant that expires
while a request is queued cannot be used at its earlier transaction-start time.
The operator's database permission is also rechecked after the wait, using a
freshly parsed query: a repeated cached prepared query reproduced stale role
membership on PostgreSQL 16.15 during verification.

| Order under the site guard | Result |
| --- | --- |
| A region declaration commits, then its grant is revoked | The declaration remains, with its allowed audit; subsequent declarations are refused. |
| The grant or actual boundary is revoked, then a declaration acquires the guard | The declaration leaves the registry unchanged and commits a denied audit. |
| Registry SQL succeeds but the audit insert fails | The transaction rolls back the registry change. |
| An adapter is already allowed and is allowed again | The original allowlist row remains; the new attempt has its own reason and no-op audit. |
| Pairing names an undeclared region | The service records an `undeclared_region` refusal with the authority references used for evaluation. |

Receipts identify a durable `audit_id`. Audit records contain actor kind and ID,
action, target, requested reason, grant and actual boundary references where
applicable, decision, outcome, evaluation details and database decision time.
Outcomes distinguish `applied`, `noop`, `inspected` and `denied`. A permission denial
commits its own audit; a database failure rolls back the attempted transaction.
An unprivileged database caller without INSERT permission on the audit table is
refused without attempting an audit write. This limitation is explicit rather
than a claim that a record exists when it could not be stored.

The service has no history-delete or reactivation operation. These controls do
not make records tamper-proof against the deployment database administrator.
They also do not repair the separate tenant-admin effect-audit gaps owned by
`SIGNOFF-REPAIR.3.3` or qualify Internet exposure under G6/G7.

The focused live controls are executable in a repository-owned disposable cluster:

```bash
RB_DEMO=0 bash scripts/run_pg_tests.sh site_authority
```

The decision and durable verification record are
`docs/decisions/2026-09-09_site-operator-authority.md` and
`docs/tasks/SIGNOFF-REPAIR.md`.
