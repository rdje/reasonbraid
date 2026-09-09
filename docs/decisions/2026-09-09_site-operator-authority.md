---
answers:
  - Who may mutate shared adapter and region registries?
  - Can tenant enrollment issue site-operator authority?
  - How must registry mutations serialize with revocation?
---
# Explicit site-operator authority for shared registries

- Status: accepted; service/schema implemented under `SIGNOFF-REPAIR.3.2.1`, ten live controls and strict focused lint pass. Operator CLI is implemented and verified under `.3.2.2`; HTTP enforcement is implemented and verified under `.3.2.3`: eight focused HTTP controls, the selected 58-test security run and the final 12-test registry run pass.
- Owner: repo-local engineering, acting on the director's explicit delegation.
- Date: 2026-09-09.
- Sources: `ROADMAP.md` §§4.4–4.5, 16.4, 20.10; ADR-027 and ADR-035.
- Supersedes: tenant-admin authority for global mutations in `PHASE-8.4.4` and `PHASE-8.5.2`; their historical test results remain historical evidence.

## Context and source evidence

At baseline `9c2d2ba`, `require_admin_any_tenant` in `crates/reasonbraid-server/src/api.rs` authorizes shared adapter and region registry handlers using an active tenant-admin grant from any tenant. Its SQL omits enrollment-boundary validation. The region and allowlist fixtures expect an ordinary newly enrolled tenant administrator to mutate global rows.

This conflicts with tenant scope and the administrative freeze contract in `docs/decisions/2026-09-07_boundary-revocation-freeze.md`. An owned legacy-server probe runtime-confirmed all four shared writes from two tenant admins, including adapter/region writes after the second admin revoked its tenant boundary (all returned HTTP 200). SQL recorded one revoked boundary, two new adapters, two new regions and only the boundary-revocation admission audit. Full evidence is in `SIGNOFF-REPAIR.3.2.1`.

## Decision

1. Shared registry mutations require a distinct, explicitly issued **site-operator grant**. Tenant administrator authority has no implicit site capability. Existing tenant grants receive no automatic upgrade.
2. Site authority is rooted in an operator-controlled boundary, with explicit permitted actions, validity window, subject and revocation state. Evaluation resolves the grant's actual parent boundary. Tenant-facing enrollment and tenant-admin APIs cannot create, widen or restore site authority.
3. Issuance and revocation use protected operator tooling under deployment control. It records issuer, subject, boundary, actions, expiry and reason. Enrollment alone cannot bootstrap site authority through a network endpoint.
4. Each mutation validates current grant and boundary liveness and scope in the transaction that writes the registry and its audit. A consistent lock order serializes it with revocation. Revocation committed first prevents later mutations; previously committed mutations remain attributable history.
5. Allowed and denied attempts leave durable audit records naming actor, action, target, authority references, reason, decision and time. Audit failure prevents an allowed mutation from committing. No-op operations remain attributable.
6. Tenant administrators retain approved tenant-scoped inspection and frozen-boundary read behavior. Global inspection gets an explicit read policy and conveys no mutation rights. Shared resolver/workflow writes receive the same scope census in `SIGNOFF-REPAIR.7.1` and `SIGNOFF-REPAIR.8.1`.
7. Site authority does not strengthen development authentication. The principal header remains a development trust assumption. Internet authentication and external G6/G7 qualification remain required.

## Service implementation contract

Migration `0054` adds separate site boundaries, grants, audit history and one
serialization guard. It seeds no authority and creates no database role. Immutable
issuance facts include the actual parent and scope/window; a trigger refuses
rebinding or reactivation. A replacement requires explicit new issuance. Candidate
selection checks every relevant grant until it finds a usable one, so a later
unusable grant cannot shadow an eligible grant.

Global reads require the explicit `registry_inspect` capability and a live grant
and actual boundary. Writes use `adapter_allow`, `adapter_revoke`, `region_declare`,
`region_pair` and `region_unpair`. Human and role subjects have explicit kind/id
objects in site issuance/audit payloads. The separately discovered core
`GrantSubject` serde defect remains owned by `.3.3`.

The protected service derives its issuer from SESSION_USER and requires a database
superuser or MEMBER of the deployment-managed `reasonbraid_site_operator` role.
Membership alone does not provision table privileges. PostgreSQL documents
[session versus current identity and membership semantics](https://www.postgresql.org/docs/16/functions-info.html).
Database administration is the trust root; this gate does not authenticate HTTP
callers or make a trust-authenticated test session credential-authenticated.

Short configuration transactions take the singleton guard, use READ COMMITTED,
and read database clock time after waiting. Lock and statement timeouts are five
and ten seconds. Only database operations run under the guard. All supported
site grant/boundary status changes take that same guard; PostgreSQL's
[row-lock rules](https://www.postgresql.org/docs/16/explicit-locking.html)
provide the mutual exclusion until transaction completion. Registry effects and
their audit insert commit together; audit failure rolls back the effect.
No-ops preserve the original registry entry and receive a separate attempt record.

Operator membership is checked again after the guard wait. Native libpq controls
on PostgreSQL 16.15 reproduced stale membership from a cached prepared query after
a concurrent committed REVOKE; fresh simple-protocol SQL saw the revocation.
The implementation therefore sends the static identity statement through SQLx's
text Executor path before and after waiting. There is no caller interpolation.
PostgreSQL's [membership cache and invalidation implementation](https://github.com/postgres/postgres/blob/REL_16_15/src/backend/utils/adt/acl.c)
supports the observed cache mechanism; the independent probe and queued-issuance
regression, rather than an assumed protocol guarantee, are the qualification evidence.
Raw database-role administration is outside the site's lifecycle API: the recheck
is the role decision point, not a claim to serialize arbitrary privileged SQL.

A database caller without operator authority is refused. If it has audit INSERT
privilege, refusal commits an attributable denial; otherwise it receives
`OperatorRequired` without attempting an audit write. Malformed inputs fail before
forming a command. Database failures prevent completion and must never be reported
as stored audit records. The service exposes no history-deletion operation; database
administrators can still modify their database directly.

## Protected operator command

`rb-site` now issues/lists/disables boundaries and grants, and lists durable audit
history. Every inspection uses the operator gate and guard, records its reason,
and returns a bounded newest-ID-first page with a same-collection continuation
cursor. Pages are current views, not a repeatable snapshot across concurrent
updates. Issuance is deliberately not automatically retried; lost output or a
timeout requires history inspection before another issuance.

The binary requires an explicit `RB_SITE_DATABASE_URL` (or `--database-url`) with
a numeric loopback host, port, database and login. It refuses remote plaintext
connections because the existing SQLx build has no TLS transport. Before any
authority or audit write, it verifies the actual database and that its data
directory resides on the repository volume. The operator needs read access to
`data_directory`, in addition to the site role and operation's table privileges.
It performs no migrations, creates no roles and grants no permissions.

The process discards ambient PG overrides before creating its runtime and builds
explicit decoded options through `new_without_pgpass`. Help/errors do not print
the URL or driver connection details. No home credential file is an implicit
input. The runner's credential fallback correction and driver evidence are
recorded in `docs/decisions/2026-09-09_disposable-postgresql-runner.md`.

The book chapter `docs/book/src/site-authority.md` describes the command flow,
deployment permissions, exit statuses and HTTP registry contract.

## Required proof

Demonstrate operator success; tenant-admin refusal across multiple tenants; refusal after grant or boundary revocation, suspension, expiry or before validity; rejection of enrollment-based escalation; unchanged victim state on refusals; audit persistence; and deterministic revocation/mutation races in both commit orders. HTTP status alone is insufficient.

## Consequences

Existing success fixtures must obtain explicit site authority. Deployments will need an operator-issued grant for these mutations after the repair. Existing registry entries confer no authority. Correct historical qualification claims through `docs/tasks/SIGNOFF-REPAIR.md`; preserve source evidence under `docs/tasks/artifacts/signoff_review/`.

## HTTP integration contract

All seven shared adapter/region HTTP operations call the same site service.
Successful JSON bodies are preserved and the committed audit reference is exposed
in x-reasonbraid-site-audit. Every mutation now requires a bounded Reason;
registry identifiers are bounded RegistryName values. The old any-tenant gate and
private unaudited region mutation helpers are removed. Registry inspection requires
its own site action. Tenant-admin grants have no site interpretation.

Malformed requests return safe typed JSON without an authority write. Domain and
authority refusals include audit_id only after the refusal commits. A database or
audit-write failure remains 500 dependency_unavailable with no fabricated receipt;
allowed effects roll back if their audit cannot commit. Site issuance remains
restricted to the database operator surface; HTTP principal authentication is
still the development deployment assumption.

The HTTP tests replace the former escalation-positive fixtures and exercise all
verbs, distinct tenants and freeze, scoped human/role grants, current actual parents,
no-op receipts, malformed/domain/storage refusals, both revocation orders and
expiry during a database-guard wait. Exact runtime results belong to the owning
leaf; implementation and test source alone are not qualification.
