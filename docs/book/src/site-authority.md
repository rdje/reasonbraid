# Site authority for shared registries

The shared adapter and region HTTP registries now use explicit site authority.
The service and protected `rb-site` CLI are verified under
`SIGNOFF-REPAIR.3.2.1` and `.3.2.2`. HTTP enforcement is implemented under `.3.2.3`;
all eight focused HTTP controls and the selected adjacent checks pass. Tenant enrollment conveys no site
capability. The development principal header remains a deployment trust assumption.

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
| `evidence_expire` | Expire due evidence, or tombstone one named snapshot. |
| `workflow_register` | Register a workflow profile version in the site-wide registry. |

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

## Registry HTTP commands

Each request presents the human or role named by an explicit site grant using
`x-reasonbraid-principal`. That header is trusted only in the development profile;
issuing a site grant does not authenticate the network caller. Use the protected
operator CLI below to issue the grant. There is no HTTP site-issuance endpoint.

| Method and path | Site action | Required JSON body |
| --- | --- | --- |
| `GET /v1/admin/adapters` | `registry_inspect` | None |
| `GET /v1/admin/regions` | `registry_inspect` | None |
| `POST /v1/admin/adapters` | `adapter_allow` | `{"adapter_id":"vendor-4","reason":"qualified release"}` |
| `POST /v1/admin/adapters/{adapter_id}/revoke` | `adapter_revoke` | `{"reason":"retired release"}` |
| `POST /v1/admin/regions` | `region_declare` | `{"region":"eu-site","reason":"qualified site"}` |
| `POST /v1/admin/regions/{from}/pair/{to}` | `region_pair` | `{"reason":"approved route"}` |
| `POST /v1/admin/regions/{from}/unpair/{to}` | `region_unpair` | `{"reason":"retire route"}` |
| `POST /v1/workflow-profiles` | `workflow_register` | `{"profile_id":"reviewed","steps":["solicit","critique","decide"],"reason":"the review lane needs a critique step"}` |

The JSON schemas reject unknown fields. Use `Content-Type: application/json` for
mutations. Path names are percent-encoded URL components; the decoded names obey
the same 256-byte contract as names in JSON. Invalid UTF-8 path components return
the same safe JSON error envelope as malformed request bodies. A write grant does not grant list
access; a read grant does not grant writes. Lists require no caller reason and
record the service reason `inspect registry`.

For example, after issuing the matching site grant to an enrolled human:

```bash
# Choose the development listener and that grant's exact human/role subject.
RB_API='http://127.0.0.1:4310'
RB_SITE_SUBJECT='hpr_<human-UUID>'
curl --fail-with-body -i -H "x-reasonbraid-principal: $RB_SITE_SUBJECT" \
  -H 'Content-Type: application/json' \
  --data '{"region":"eu-site","reason":"qualified site"}' \
  "$RB_API/v1/admin/regions"
curl --fail-with-body -i -H "x-reasonbraid-principal: $RB_SITE_SUBJECT" \
  -H 'Content-Type: application/json' \
  --data '{"reason":"approved route"}' \
  "$RB_API/v1/admin/regions/dev-local/pair/eu-site"
curl --fail-with-body -i -H "x-reasonbraid-principal: $RB_SITE_SUBJECT" \
  "$RB_API/v1/admin/regions"
```

Both regions must already be declared before pairing. Pairs are directed:
`dev-local` → `eu-site` conveys no reverse route. An ordinary tenant administrator
receives 403 for all seven operations, even if its tenant boundary is active.
Revoking that tenant boundary cannot confer site access. A separately issued site
grant remains governed by its own actual site boundary.

Successful responses retain the existing JSON bodies, such as
`{"region":"eu-site","declared":true}` or
`{"from":"dev-local","to":"eu-site","paired":true}`. The response header
`x-reasonbraid-site-audit` identifies the committed audit record. Adapter lists
retain `adapter_id`, `added_by`, `reason`, `added_at`; region lists retain `regions`
and `pairs` with `from`/`to`. Inspect the audit record through `rb-site audit list`.
An already satisfied mutation returns 200 with the same response body and its own
`noop` audit. Repeating an allow preserves the registry's original reason while
recording the repeat request's reason separately.

| Failure | Status and receipt |
| --- | --- |
| Missing development principal | 401; no site audit is claimed |
| Malformed JSON, missing/unknown fields, invalid name or reason | 400 `invalid_command`; no site audit |
| Wrong content type or oversized body | 415 or 413 `invalid_command`; no site audit |
| No usable site grant and actual boundary for the action | 403 `site_authority_required`, with committed `audit_id` |
| A caller that cannot even resolve `public.site_audit` | 403 `site_authority_required` — the privilege probe reads the catalogue by OID, so a missing schema `USAGE` is still answered as "not an operator", never as a dependency failure |
| Authorized pairing names an undeclared region | 400 `undeclared_region`, with committed `audit_id` |
| Database, lock-timeout or audit-insert failure | 500 `dependency_unavailable`; no receipt is fabricated |

Refusal JSON contains `code`, a safe `message`, and `audit_id` only when that refusal
record committed. No supplied malformed body or database diagnostic is echoed.
A lost response can leave a committed effect: inspect history before deciding to
retry. These registry requests do not use the thread-command idempotency-key store.

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
RB_DEMO=0 bash scripts/run_pg_tests.sh site_registry_http allowlist regions site_authority
```

The decision and durable verification record are
`docs/decisions/2026-09-09_site-operator-authority.md` and
`docs/tasks/SIGNOFF-REPAIR.md`.

## Deployment-local operator CLI

Build and run the dedicated executable from within the repository:

```bash
python3 -B scripts/project_env.py cargo build --locked -p reasonbraid-server --bin rb-site
python3 -B scripts/project_env.py target/debug/rb-site --help
```

`rb-site` neither starts a server nor applies migrations. Prepare the database
schema through the deployment workflow first. The current SQLx build has no TLS
transport, so this tool accepts only explicit numeric loopback hosts, an explicit
port, login and database, and the optional `sslmode=disable` URL parameter. Host
names, remote addresses, other URL options and implicit targets are refused.
Run on the database host; the data directory must exist on the repository's
filesystem volume. The tool reads that setting and checks the database identity
before any authority or inspection write. Unix deployment is currently required.

Select the connection with `RB_SITE_DATABASE_URL` in the operator's protected
environment. This separate variable avoids treating an ordinary development
DATABASE_URL as operator intent. The following is a shape example; replace the
login, credential, port and database for the deployment:

```bash
export RB_SITE_DATABASE_URL='postgres://operator:<percent-encoded-password>@127.0.0.1:55432/reasonbraid?sslmode=disable'
```

The tool does not print the selected URL, including in help and target errors.
It discards ambient PGHOST, PGUSER, PGPASSWORD, PGOPTIONS and other PostgreSQL
overrides before starting its runtime. It constructs options without reading
passfiles or certificate files. URL components are decoded explicitly, preserving
literal plus signs and percent-encoded credential characters. No operator state
file or home cache is written. Protect the chosen environment and local database
authentication according to the deployment's operating-system access policy.
Production operator logins need verified database authentication; loopback trust
is reserved for the disposable test setup.

### Deployment permissions

Provision a dedicated login through the database's normal authentication workflow.
Then a deployment administrator can create the site-operator group and grant the
required privileges. These SQL statements are deployment setup, not actions
performed automatically by `rb-site`:

```sql
CREATE ROLE reasonbraid_site_operator NOLOGIN;
GRANT reasonbraid_site_operator TO operator;
GRANT pg_read_all_settings TO reasonbraid_site_operator;
GRANT USAGE ON SCHEMA public TO reasonbraid_site_operator;
GRANT SELECT, UPDATE ON public.site_authority_guard TO reasonbraid_site_operator;
GRANT SELECT, INSERT ON public.site_boundaries, public.site_grants,
    public.site_audit TO reasonbraid_site_operator;
GRANT UPDATE (status) ON public.site_boundaries, public.site_grants
    TO reasonbraid_site_operator;
```

Use the actual login identifier in place of `operator`. If the group already
exists, review its configuration instead of recreating it. Normal inherited
privileges are assumed. `pg_read_all_settings` permits the data-directory check;
it is not site authority by itself. The deployment operator's direct database
privileges form part of the trust root: supported commands maintain the audit
contract, but those credentials must not be given to tenant-facing processes or
treated as incapable of direct SQL. No DELETE or audit UPDATE privilege is needed
for the supported operator commands.

### Issue and retire access

Choose explicit RFC3339 times and the existing human's full `hpr_…` ID. An agent
role uses `role:rol_…` instead. The following flow gives only region-declaration
authority. Commands return JSON; the extraction commands retain the IDs for the
next step, while every successful operation also has a durable audit receipt.

```bash
set -euo pipefail
RB_SITE_FROM='2026-09-09T00:00:00Z'       # choose the intended inclusive start
RB_SITE_UNTIL='2026-09-10T00:00:00Z'      # choose a future exclusive expiry
RB_SITE_SUBJECT='hpr_<existing-human-UUID>'

RB_SITE_BOUNDARY="$(python3 -B scripts/project_env.py target/debug/rb-site boundary issue \
  --action region_declare --valid-from "$RB_SITE_FROM" --expires-at "$RB_SITE_UNTIL" \
  --reason 'approved regional administration' \
  | python3 -B -c 'import json,sys; print(json.load(sys.stdin)["result"]["boundary_id"])')"

RB_SITE_GRANT="$(python3 -B scripts/project_env.py target/debug/rb-site grant issue \
  --boundary "$RB_SITE_BOUNDARY" --subject "human:$RB_SITE_SUBJECT" \
  --action region_declare --valid-from "$RB_SITE_FROM" --expires-at "$RB_SITE_UNTIL" \
  --reason 'assign regional administration' \
  | python3 -B -c 'import json,sys; print(json.load(sys.stdin)["result"]["grant_id"])')"

python3 -B scripts/project_env.py target/debug/rb-site grant list --reason 'review assignments'
python3 -B scripts/project_env.py target/debug/rb-site grant suspend "$RB_SITE_GRANT" --reason 'stop access'
python3 -B scripts/project_env.py target/debug/rb-site grant revoke "$RB_SITE_GRANT" --reason 'retire access'
python3 -B scripts/project_env.py target/debug/rb-site boundary revoke "$RB_SITE_BOUNDARY" --reason 'retire ceiling'
```

The site actions are `registry_inspect`, `adapter_allow`, `adapter_revoke`,
`region_declare`, `region_pair`, `region_unpair`, `evidence_expire` and
`workflow_register` — `evidence_expire` being the evidence retention sweep (see
[Deployment](deployment.md)) and `workflow_register` the workflow-profile
registry described below. Repeat `--action` to grant additional capabilities. A request to issue beyond the
parent's action set or window is refused and audited. Suspension cannot be undone
by this tool: issue replacement authority when access must be restored. Repeated
revocation returns `changed: false` and creates a no-op audit. There is no automatic
retry of issuance and no issuance idempotency key; a deliberate repeated issuance
creates a different record. After a timeout or lost output, inspect history before
trying to issue again. Lost shell variables can be recovered through the protected
boundary/grant listings and their reasons.

The issued grant is the authority evaluated by the registry HTTP commands above.
The library and CLI controls prove issuance and disabling; `.3.2.3` owns live HTTP
qualification. The ordinary `rb` CLI has no dedicated shared-registry commands.

### Inspect bounded history

```bash
python3 -B scripts/project_env.py target/debug/rb-site boundary list --limit 20 --reason 'review ceilings'
python3 -B scripts/project_env.py target/debug/rb-site audit list --limit 20 --reason 'review site history'
python3 -B scripts/project_env.py target/debug/rb-site audit list --limit 20 \
  --before 'sau_<next_before-UUID>' --reason 'continue site history review'
```

Each result contains `collection`, `items`, and `next_before`. Pass the exact
returned `next_before` value to the same collection; null means that page reached
the end. Pages contain 1–100 requested items at most, ordered by descending ID.
They are current views rather than a repeatable multi-page snapshot of concurrent
changes. An inspection creates a new audit record after reading its page; its
own newer records do not extend an ordinary descending history walk. The record
includes the requested collection, cursor, limit, database actor and reason.

Exit status 0 means a committed receipt was returned; 2 means invalid arguments or
input; 3 means an authority/domain refusal. Status 1 covers connection, storage
verification, operation or output failures. A denial includes an `audit_id` only
when its record committed. An unverified storage location is refused before any
operation audit (`storage_unverified`), and an outsider without audit INSERT permission cannot create
an audit record. Database errors are reported without driver connection details.

Run the real binary controls alongside the existing service suite:

```bash
RB_DEMO=0 bash scripts/run_pg_tests.sh site_operator_cli site_authority
```

## What site authority does NOT cover yet

The site actions above are each explicitly issued and audited. They are not,
however, the complete set of site-global surfaces. A census derived from the
producers rather than from a family name finds that **thirty tables carry no
tenant dimension and are written by routes admitted on tenant enrolment
alone**:

```bash
python3 -B scripts/census_shared_registry_writes.py   # the writers, by admission
python3 -B scripts/census_registry_read_reach.py      # the readers, and the tenant derivations
```

`SIGNOFF-REPAIR.7.1.2` adjudicated them. Four distinct positions came out of it,
and the distinction between the first two is the one an operator needs.

### A table without a `tenant_id` column may still have an owner

`agent_profiles`, `profile_versions`, `quota_events` and `recruitment_responses`
carry no tenant column, yet each keys into a table that does — `agent_roles`,
`usage_quotas` and `recruitment_calls` respectively — so a single join recovers
the owning tenant. They are tenant-owned records, not shared ones. A quota check,
for instance, resolves its `quota_id` from `usage_quotas WHERE tenant_id = …`
before counting events against it, so the count is tenant-bound even though the
counting statement never mentions a tenant.

### The evidence chain and the evaluation corpora are shared by design

Evidence snapshots, derivations, resource references and snapshot objects are
content-addressed rows that several tenants share by construction, so what must
name the tenant is the decision to DISCLOSE the row rather than the row itself.
A citation is recorded at the moment a tenant cites a snapshot, and the read is
bound to that citation. The seven `evaluation_*` tables are
shared for the same practical reason a benchmark corpus is: it is more useful
shared than copied.

### The directory is read across tenants on purpose, and bounded by FIELD

`POST /v1/directory/match` and `GET /v1/directory/presence` return every
enrolled role's current profile regardless of the caller's tenant. That is deliberate: the
product premise is that an authorized caller asks a durable network a question
*without knowing who is online*, and a directory restricted to one tenant would
not answer it.

What bounds the disclosure is not the row set but the fields. Each request derives
a reader classification — `Full` for a tenant administrator over their own tenant,
otherwise `Tenant` or `Network` — clamps the request's declared scope against it,
and filters every profile through that classification before returning it. A field
declared at a higher visibility class than the reader holds is absent from the
response, not redacted in place.

This differs from `GET /v1/nodes/presence`, which answers about one *named* node
and is therefore bound to the caller's own tenant: there, a foreign answer would
be an existence oracle. Set-valued directory questions and identity-valued
presence questions are bounded differently, and on purpose.

### The policy and workflow registries are shared CONTROL surfaces

These are the open items, and they are recorded here rather than implied.

The ten policy lifecycle tables form one chain — a version, a proposal, a
decision, an approval, a projection, a publication, then drift, outcomes,
corrections and reviews. Each stage reads the previous stage's row in order to
decide, every row is keyed on an opaque identifier with no tenant, and every write
admits on enrolment alone. The review scheduler in particular reads drift,
outcomes and corrections across the entire site with no predicate, so a record
written by one tenant can schedule a review against another tenant's publication.
Whether one site-wide policy library is intended, or a policy belongs to a tenant,
is an open decision owned by `SIGNOFF-REPAIR.6.1.5`.

### The workflow-profile registry: repaired, and why it needed to be

`POST /v1/workflow-profiles` used to admit any enrolled principal and append a
new version under any profile identifier, including the eight built-in profiles.
Thread creation resolves a profile to its **highest** version site-wide, and a
bare thread resolves `quick_advice`. So one enrolled principal in any tenant
could choose the steps every other tenant's next bare thread executed — not by
introducing a new capability, because the step vocabulary is a closed set of
thirteen kinds, but by choosing the deliberation's shape: dropping
`blind_solicit` from the panel, or `critique` from the review.

It now takes the `workflow_register` site capability, and the request body gained
a required `reason` like every other site act. Reading the registry
(`GET /v1/workflow-profiles`) deliberately still requires only enrolment: a
tenant must be able to see which profiles exist in order to name one on a thread,
and the rows carry no tenant data. The defect was the write.

Appending a version to an existing profile is still possible for an operator who
holds the capability — the registry is versioned by design, and forbidding that
would have removed the feature rather than repaired the authority.

The routing journals (`routing_resolutions`, `routing_recommendations`) are
append-only audit rows read without a tenant predicate, so a caller sees every
tenant's routing trail. That is a disclosure limit, not a control one — routing
decisions do not read those tables — and `SIGNOFF-REPAIR.7.1.2.2` owns it.
