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
| `policy_register` | Register a policy version in the site-wide governance library. |
| `evaluation_record` | Set the evaluation standard: register a corpus, record a run, create a trial, record its results, record a calibration, record a gate. |
| `gate_evaluate` | Measure against that standard: run a gate. Separate from `evaluation_record` because a party that measures must not be able to move the standard. |
| `charter_register` | Register a [governance charter](governance-charter.md) version for a tenant. |
| `resolver_register` | Register a new resolver in the site-wide resolver registry that every tenant's acquisition ranks. |

Every action in this table can be held by a boundary and a grant. That was not
always true. `charter_register` was added to the code without the migration that
lets the database store it, so no grant could carry it and the charter verb
refused every caller until `SIGNOFF-REPAIR.9.1.1` (migration `0109`). A live
control now lists the actions from the code itself and issues a boundary for
each one, so a new action that the database cannot store fails a test. The next
action to arrive, `resolver_register` (`SIGNOFF-REPAIR.7.1.3.1`, migration
`0110`), came with its migration in the same change, and that control passed on
it.

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
| `POST /v1/policies` | `policy_register` | The §15.1 policy document, plus `"reason":"the organization baseline is amended"` |
| `POST /v1/resolvers` | `resolver_register` | `{"advertise":{"resolver_id":"r1-git-mirror","schemes":["git"],"egress_class":"listed","sandbox_level":"process","version":"0.1.0"},"reason":"the mirror serves the air-gapped site"}` |

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
| An AUTHORIZED caller's act refused on its own terms — an undeclared region, a taken coordinate, a corpus that is not registered | 400 with that reason as `code` and a committed `audit_id`; never counted as an authorization denial |
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

A timestamp the server reports is the one it stored. Instants that are written to
the database and may also be returned are sampled from the database clock rather
than from the process, so the value in a response and the value in the row are the
same microsecond — a returned `decided_at` or `expires_at` can be compared against
a later read without an apparent difference that is only rounding. Values supplied
by a caller are normalized to the same precision before they are used in a
comparison.



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
`region_declare`, `region_pair`, `region_unpair`, `evidence_expire`,
`workflow_register` and `policy_register` — `evidence_expire` being the evidence
retention sweep (see [Deployment](deployment.md)), and the last two the two
site-wide registries described below: the workflow profiles and the governance
library. Repeat `--action` to grant additional capabilities. A request to issue beyond the
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
producers rather than from a family name found that **thirty tables carried no
tenant dimension and were written by routes admitted on tenant enrolment
alone**. The repairs since then leave **9 routes across 10 tables** admitted on
enrolment alone, every one of them adjudicated (measured 2026-09-25; both
censuses now run on every commit, `SIGNOFF-REPAIR.7.1.6`):

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
enrolled role's current profile regardless of the caller's tenant — including
each imported identity bound to its origin's node, listed under the importing
tenant ([where an imported identity runs](profiles.md#where-an-imported-identity-runs)). That is deliberate: the
product premise is that an authorized caller asks a durable network a question
*without knowing who is online*, and a directory restricted to one tenant would
not answer it.

What bounds the disclosure is not the row set but the fields, and the bound is
decided **per candidate tenant** (`SIGNOFF-REPAIR.5.1.1`). Toward its own tenant
the reader is `Full` (a tenant administrator) or `Tenant` (a member); toward a
tenant that holds the effective directory-visibility agreement with the reader's
it is `Tenant`; toward every other tenant it is `Network`. The match clamps the
request's declared scope against the reader's own class, then judges, ranks and
renders each candidate at the lower of that scope and the class toward the
candidate's tenant — so a foreign tenant-only claim neither satisfies a
requirement nor appears in the answer. A field declared at a higher visibility
class than the reader holds toward that candidate is absent from the response,
not redacted in place. ⚠️ Until that repair the match applied ONE class — the
reader's own — to every tenant's candidate, which this paragraph described as
the design. The presence listing reads each other tenant at the **same** class,
from the same code (`SIGNOFF-REPAIR.5.1.6`): until then it split own tenant
from every other correctly but read all the others at `Network`, so a partner
under an effective directory agreement saw the network view there and the
tenant view in the match.

This differs from `GET /v1/nodes/presence`, which answers about one *named* node
and is therefore bound to the caller's own tenant: there, a foreign answer would
be an existence oracle. Set-valued directory questions and identity-valued
presence questions are bounded differently, and on purpose.

**The network view names each role by its own id.** An entry another tenant
reads carries the role's `role_id`, and its `node_id` when the two are the same:
a stable identifier, not a pseudonym, although this book used to call the view
*the network pseudonym*. Knowing the id takes that tenant no further, measured
(`SIGNOFF-REPAIR.11.46`): inviting the role is refused (`400`, it is not a role
enrolled in that tenant), its node presence by id answers `404`, its profile
read by id is the same network view, and exporting its card is `403`.
Identifiers that are pseudonymous toward other tenants, which the roadmap's
directory design allows for, are not built (`.11.46.1`).

### The policy registry: one name, two subjects

The ten policy tables form one chain — a version, a proposal, a decision, an
approval, a projection, a publication, then drift, outcomes, corrections and
reviews — and each stage reads the previous stage's row in order to decide. The
question of whether a policy belongs to a tenant or to the site was open for a
long time, and the answer turned out to be *both*, for different tables.

**`policy_versions` — the library — is site-wide by design.** A policy is a
governance document keyed by `(policy_id, version)` and owned by a *grant*, not
by a tenant. A policy that only one tenant can read is not governance, so the
library stays readable by every enrolled principal and nothing here will change
that. Its *write* is a different question, and it used to admit any enrolled
principal into a first-come identifier namespace. It now requires the
`policy_register` site capability — the same repair the workflow registry took,
described in full below.

**The nine lifecycle tables belong to tenants.** The write side always knew it:
registering a proposal refuses a thread outside the caller's tenant, and
recording a decision refuses a verdict from one. The row simply did not store
what the write had already checked. It does now — every lifecycle row carries the
tenant that owns it.

Owning is not the same as writing, and the two differ for five of the nine. A
proposal, a decision, an approval and a projection belong to the caller. A
publication belongs to its **proposal's** tenant, and drift, corrections,
outcomes and reviews belong to their **publication's** — because a record about
someone else's publication concerns that someone else, and stamping it with its
author's tenant would hide it from the only party it is about.

**Every mutating verb over a publication now requires the caller to own it.**
Staging a publication, marking it effective or failed, publishing it, recording
drift, a correction or an outcome against it, deploying it, filing its receipt,
and closing one of its reviews are all refused for a caller in another tenant.
Scheduling reviews names no identifier at all, so it is scoped rather than
refused: a request materialises review rows for the caller's own publications and
for nobody else's.

A foreign record answers exactly as an absent one. That is deliberate: a refusal
that distinguished *not yours* from *no such thing* would let any enrolled
principal enumerate every other tenant's publication identifiers.

Three of those verbs already required the caller to hold a publication authority.
That check asks whether a principal may act on publications at all; it never
asked whose publication this was, so a principal holding their own grant could
advance another tenant's publication. Holding an authority and owning a record
are two different questions and both are now asked.

> **Operator note — an unattributable publication is frozen.** The migration that
> gave these rows an owner derives it from the publication's own lineage and
> leaves it empty where that lineage is broken. A publication with no owner can
> now be advanced by nobody: not marked effective, not failed, not published, not
> deployed, not corrected. This is deliberate — an unowned governance record that
> anyone may advance is worse than one that is frozen — but it means an upgraded
> deployment should check for rows with no owner before relying on them.

**Every lifecycle read is bound to the caller's tenant.** Listing proposals,
decisions, approvals, projections, publications, deployments, drift,
corrections, outcomes or reviews returns the caller's own rows and nobody
else's. A row the upgrade could not attribute is returned to nobody, which is
the same disposition as the frozen publication above and for the same reason.

**The governance library stays open, deliberately.** `GET /v1/policies`,
resolving a policy set, the impact map and the MCP policy bundle all still answer
any enrolled principal, because a policy only its author can read is not
governance. Only the *write* changed, and it is described under
[The governance library](#the-governance-library-the-same-repair-twice) below.

Staging a publication also now requires the projection to be the caller's own.
The check used to ask only whether the projection existed, so a publication
could be assembled from another tenant's compiled bytes — the output of a
resolution request that tenant made, not this one.

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

### The governance library: the same repair twice

`POST /v1/policies` used to admit any enrolled principal into a first-come
identifier namespace. `policy_versions` is keyed `(policy_id, version)` with no
tenant column, so the first caller to name a coordinate owned it: the rightful
author of an organization baseline could find `org-baseline 1.0.0` already taken
by someone in another tenant, be refused as a duplicate, and then read that
stranger's text as the baseline — because the library is shared, and shared is
what makes it governance. The same principal could append a further version to a
document an operator had published.

It was reproduced in all four of those steps against the unrepaired server before
anything was changed, and it now takes the `policy_register` site capability.
Like every other site act, the request body gained a required `reason`: a caller
who chooses what the whole site reads as governance is exactly the caller who
must be able to explain it afterwards.

**Two refusals moved behind the gate, and they answer `400` with an audit
identifier.** Naming an owning authority that is not a live grant you hold, and naming a
`(policy_id, version)` that already exists, are both questions about the database
rather than about the submitted document. Answering either one *before* the
capability is checked would tell a principal with no site authority which grants
the site holds and which coordinates the registry has taken — so the gate runs
first, and an unauthorized caller still receives `403` having learned nothing.

⚠️ **They briefly answered `403` too, and that was wrong.** A caller who passes
the gate holds the grant, so telling them *a current site grant is required*
stated something false and counted them as an authorization denial.
`SIGNOFF-REPAIR.16` split the two: `403` means the caller lacks the authority,
`400` with an `audit_id` means the caller held it and the act was refused on its
own terms. The ordering that closes the oracle is unchanged.

The seven rules that ask only about the *document* — the digest shape, that a
declared digest is the document's own, a SemVer 2.0.0 version, the lifecycle
vocabulary, a non-empty clause list, clause identifiers that do not repeat, and
selectors that are exactly `{"layer": …, "target": …}` —
still answer `400` to anyone, because each is a rule over the submission itself
or a constant this book publishes.

**Reading the library did not change and deliberately will not.** `GET
/v1/policies`, `POST /v1/policies/resolve`, the impact map and the MCP policy
bundle all still answer any enrolled principal. A policy only its author can read
is not governance, and the test suite asserts that openness rather than leaving
it implicit, so a future change that bound one of these reads fails a control
instead of quietly reversing the decision behind it.

**The owning authority must be a grant you hold** (`SIGNOFF-REPAIR.9.1.2`). The
`policy_register` capability lets you write the library; it does not let you speak
with someone else's authority. A registration names an `owning_authority`, and the
publication verbs later treat that grant as the policy's owner, so the grant must be
live, cover `policy_version_register`, and be held by the caller. Naming another
principal's grant, or a grant that does not exist, is refused with one text for
both, so a registrar learns nothing about grants it does not hold:

```json
{"code": "the named owning authority is not a live grant the caller holds that covers policy_version_register",
 "audit_id": "…"}
```

That answer is `400`, because the caller did pass the site gate. A policy owned by
someone other than the site operator is registered by its owner, once the owner
holds `policy_register`. Delegating the registration is not possible yet, because
nothing issues a grant from another grant (see *What `delegable` and
`max_delegation_depth` do not do* in [Authority](authority.md)). The decision is
`docs/decisions/2026-09-25_a-policy-registrar-holds-the-authority-it-names.md`.

**The server derives a policy's digest** (`SIGNOFF-REPAIR.9.1.3`). §15.1 gives every
policy version an immutable digest, and the registry used to store whatever
`sha256:<64 hex>` the caller typed: two versions with different clauses could share
one, and a digest could hash nothing. The server now computes it as `sha256` over
the canonical document:

- every field of the request except `reason`, `digest` and `lifecycle`; an omitted
  text field counts as `""` and an omitted list as `[]`;
- written as compact JSON, with object keys sorted by byte order at every depth and
  arrays kept in the order submitted.

`lifecycle` is left out because it is a status, not content. `digest` is optional in
the request. Send it to pin the content you mean. If it is not the document's own
digest, the request is refused before the site gate, naming both:

```json
{"code": "invalid_command",
 "message": "the declared digest `sha256:aaaa…` is not the document's digest `sha256:3f1c…`: the server derives it from the canonical document"}
```

Every read of the library (`GET /v1/policies` and the MCP policy bundle) carries
`digest_verified`, which says whether the stored document still hashes to the stored
digest. A version registered before this rule keeps the digest its registrar
declared, which nothing derived, so it reads `false`.

**A version is a SemVer 2.0.0 version** (`SIGNOFF-REPAIR.9.1.7`): `MAJOR.MINOR.PATCH`
with no leading zeros, an optional `-` pre-release and an optional `+` build, so
`1.2.3`, `1.2.3-rc.1` and `1.2.3+build.5` register, and `1`, `1.0` and `01.2.3` are
refused. The rule used to accept one to three digit groups under the name of
semantic versioning, so `1` and `007.8` passed and `1.2.3-rc.1` was refused.
Versions already stored are unchanged.

**Appending a version is still possible for a capability holder**, exactly as it
is for a workflow profile. The registry is versioned by design; the defect was
never that a policy can gain a version, only that anyone could give it one.

This is the third time the same template has been applied — the evidence
retention sweep, the workflow-profile registry, and now the governance library —
and the sameness is the point rather than a coincidence. Each is a store with no
tenant column whose write was admitted on enrolment alone. The decision record
that settled the policy library required it and the workflow registry to be
decided consistently or for the difference to be stated; they are consistent, and
this paragraph is the record of it.

### The resolver registry: the same template again

`POST /v1/resolvers` used to admit on the caller's own **tenant administrator**
grant and write a row into `resolver_capabilities`, which has no tenant column
and which every tenant's resolution ranks. Before `SIGNOFF-REPAIR.7.1.3`, one
such row advertised as fast outranked the built-in packs and made every
tenant's acquisition an empty success. Since that repair a row this server
cannot execute is skipped and named, so a tenant's row could no longer acquire
anything; what it could still do was appear in every other tenant's resolution
answer. The failing control was a tenant administrator registering a row, answered
`200`, before anything changed.

A resolver describes what this **server** can acquire through, which is site
configuration, so registering one now takes the `resolver_register` site
capability (`SIGNOFF-REPAIR.7.1.3.1`). The body nests the advertisement beside
the reason, `{"advertise": {...}, "reason": "..."}`, because the advertisement
refuses unknown fields and so cannot carry the reason itself. A tenant
administrator without the capability receives `403`; the old bare advertisement
is a malformed site request (`400`).

**An id that already exists is refused behind the gate, as the policy library
refuses a taken coordinate.** The answer is `400` with an `audit_id`, and the
audit record names the resolver the caller tried to replace. The insert inside
the act is insert-only, so two concurrent registrations of one new id cannot
replace each other either. The ADR-018 vocabulary check (a sandbox level on the
ladder, an egress class in the vocabulary) still answers `400` to anyone before
the gate, because it asks only about the submitted document, against a constant
this book publishes.

Reading the resolution answer did not change: every tenant still sees which
resolvers rank, because it has to know which one acquired its evidence.

### The rule table and the deterministic resolution

Routing is a **deterministic table lookup**, not a model decision. A case class
resolves to exactly one arm through one rule, and the resolution is journalled.

```text
GET  /v1/routing/rules      the deterministic rule table
POST /v1/routing/resolve    resolve one case class to its arm
```

Both admit any enrolled principal.

The seven case classes are a closed vocabulary: `simple`, `factual`,
`uncertain`, `design_policy`, `governed`, `correlated` and `diminishing`. A
class outside it is refused.

```bash
curl -s -X POST localhost:4310/v1/routing/resolve \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{"case_class": "governed"}'
```

```json
{"case_class": "governed", "arm": "wf_governed_review", "rule_id": "rul_0192…"}
```

⭐ **A rule may not point at an arm that does not exist.** After finding the
rule, the resolution checks that its arm is a registered workflow profile and
refuses with a *phantom arm* error if it is not — so a rule table that has
drifted away from the profile registry fails loudly at the lookup rather than
routing a case into nothing.

Every successful resolution is recorded in the routing journal described below;
the lookup itself reads the rule table and the profile registry only, never the
journals.

A resolution made while **creating a thread** (a `thread.create` naming a
`routing_class` and no profile, journalled with `surface: "create_boundary"`) is
recorded by the create command itself, after its authorization, and commits
exactly when the thread does (`SIGNOFF-REPAIR.8.2.7`). A create that is refused
leaves no journal row, and replaying a create with its idempotency key adds none.
Until the repair the row was written before the authorization, so a refused
create left a row describing a resolution for a thread that never existed, and
every replay added another. The `POST /v1/routing/resolve` verb's own row is
unchanged: recording the resolution is what that verb does.

### The routing journals: bound to their own tenant

`GET /v1/routing/resolutions` and `GET /v1/routing/recommendations` used to
return every tenant's routing trail to any enrolled caller: which case classes a
tenant submitted, which arm each resolved to, which principal asked, and which
evaluation trial or gate a shadow recommendation rested on. That was a disclosure
limit rather than a control one — routing decisions read the rule table and the
profile registry, never these journals — but it was a real one.

Both are now bound to the caller's own tenant, derived from the authenticated
principal and never accepted on the wire. A caller reads its own journal in full
and sees nothing of anyone else's.

Two limits, published rather than implied. Rows written before the binding
migration are attributed where the schema allows it and are read by nobody where
it does not: `routing_resolutions` recorded the calling principal, so its
historical rows are derived back to that principal's tenant, while
`routing_recommendations` never recorded an actor at all, so every row predating
the migration is invisible to every tenant. Inventing an owner for an audit row
that asserts who did something would be worse than losing its visibility.
