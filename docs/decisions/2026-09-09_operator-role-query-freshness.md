---
answers:
  - Why must the site operator recheck use a fresh PostgreSQL query?
  - What reproduced stale database role membership after revocation?
  - Does a passing psql control qualify a cached prepared authorization query?
---
# Fresh database-role checks after waiting

- Owner: `SIGNOFF-REPAIR.3.2.1`.
- Applies to the site operator gate in `crates/reasonbraid-server/src/site_authority/operator.rs`.
- Extends `docs/decisions/2026-09-09_site-operator-authority.md`.

The first live site-authority suite accepted a second boundary issuance after the
operator's database membership was revoked while issuance waited for the site
guard. The test reused one physical operator connection. An ordinary outsider
was refused, so the failure was specific to permission freshness after waiting.
A diagnostic rerun independently checked membership was false from another
connection before releasing the guard; the queued request still succeeded.

## Independent control

Two minimal probes ran in verified disposable PostgreSQL 16.15 clusters. Both
created a member role, checked permission, blocked that member on a guard row,
revoked membership from another session and released the row. No project or
external production data was used.

| Query path on the same member connection | Observed result after the wait |
| --- | --- |
| psql simple protocol, with SQL PREPARE/EXECUTE | False membership; zero direct membership edges. |
| Native libpq cached prepared query, followed by cached clock and identity queries | True membership despite the committed revocation. |
| A fresh simple query on that libpq connection, then the prepared identity query again | Zero edges; the subsequent prepared query returned false. |

The native libpq probe reproduces the mechanism independently of Rust and SQLx.
PostgreSQL's [16.15 role-membership implementation](https://github.com/postgres/postgres/blob/REL_16_15/src/backend/utils/adt/acl.c)
caches expanded role lists and invalidates them through syscache callbacks.
Our measured conclusion is that repeated prepared execution did not refresh that
permission in this transaction. It is not a general claim that READ COMMITTED
ordinary table reads retain an old transaction snapshot.

## Implementation and regression obligation

The operator gate now passes a static SQL string to SQLx's text Executor path,
which uses the simple query protocol. It performs that query before taking the
guard and again after waiting. There is no caller interpolation. The selected
gate still uses SESSION_USER and explicit superuser or operator-role membership;
it does not fall back to a tenant grant or trust the preliminary result.

The live regression keeps a single physical member connection, confirms a real
guard-lock wait, proves committed membership loss independently before release,
and requires an audited denial with no second boundary. Ordinary member issuance
and outsider refusal are independent positive and negative controls. Preserve
this test when changing the driver, protocol path or PostgreSQL version.

The first `raw_sql(...).fetch_one(...)` formulation encountered an async
Executor/Send lifetime compilation error in spawned callers. The direct text
Executor call preserves the required protocol and compiles with those callers.
This is an implementation detail, not a reason to weaken the concurrency test.

Raw deployment-role administration remains outside the site lifecycle API. The
post-wait check is its permission decision point; site grant/boundary revocation
and registry effects have the separate shared-guard transaction ordering.
Verification transcripts are summarized durably in the owning task leaf, with
local probe sources/logs under `target/site-authority-controls/`.
