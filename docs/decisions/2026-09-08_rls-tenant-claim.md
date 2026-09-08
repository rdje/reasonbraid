# The RLS tenant claim — a transaction-local GUC, fail-closed, scoped to the wired surface (`PHASE-7.1.3.1`)

- Date: 2026-09-08 · Leaf: `PHASE-7.1.3.1` · Decision record (the §16.8 defense-in-depth contract)

## Context

ADR-034 fixed the stance: the tenant-scoped keys are the FIRST layer and the
row-level security is the SECOND — never the only one. The `.1.3` census
measured the greenfield (no RLS anywhere; `tenant_id` is the direct key on
the 0001–0020 core) and the wiring reality: ~130 query sites across twelve
modules, a pooled connection that must never leak a session claim, an outbox
worker that legitimately crosses tenants, and a dev profile that connects as
the postgres SUPERUSER — which bypasses row-level security unconditionally
(FORCE included). Each of these shaped the design below.

## Decision

- **The claim is the `app.tenant_id` GUC, TRANSACTION-LOCAL.** The server
  sets it with `set_config('app.tenant_id', $1, true)` (is_local) as the
  FIRST statement of every transaction touching a protected table; a pooled
  connection therefore never carries a claim past its transaction. A
  session-level claim on a pooled connection would leak the tenant across
  requests — the exact class of bug the layer exists to stop.
- **Fail-closed.** Every policy is
  `USING/WITH CHECK (tenant_id = current_setting('app.tenant_id', true))` —
  the `missing_ok` form reads NULL when unset, and `tenant_id = NULL` matches
  NO row. A path that forgets the claim sees nothing, loudly.
- **`FORCE ROW LEVEL SECURITY`** on each protected table: the future app role
  OWNS the tables, and owners bypass RLS unless forced. The dev profile's
  superuser bypasses regardless — the force declaration is the binding intent.
- **The enablement set matches the wired set — the COMMAND CORE only:**
  `aggregate_state`, `event_log`, `idempotency` (the aggregate head, the
  audit trail, the replay protection). The claim rides the two write entries
  (`agg::claim_in_tx`, `agg::apply_fresh_in_tx`), the in-transaction
  validation helpers inherit it, and the pool-direct inspection reads route
  through the `with_tenant_claim` wrapper.
- **The `outbox` stays EXEMPT (named):** the worker's queue legitimately
  spans tenants — a tenant claim would blind the worker. It stays on the
  first-layer WHERE clauses.
- **The measured proof is a NON-superuser probe role.** The live suite
  creates `rls_probe` (LOGIN, grants on the three tables), connects AS it,
  and measures the DATABASE-level refusal: a foreign-tenant claim reads zero
  rows and a foreign-tenant INSERT is refused; an unset claim reads zero
  rows; the correct claim reads the tenant's rows.

answers:

- **The dev profile stays on the superuser connection — the layer is INERT
  there and HONESTLY SO:** superusers bypass RLS unconditionally, so the
  migration alone would be decoration. The probe-role proof measures the
  layer as it will bind; the app wiring (the claims) is exercised end-to-end
  by the live guard. The deployment-profile change that binds it — the
  non-superuser app role + the grants + the per-profile connection URLs —
  is the NAMED deferral with its trigger (the Internet profile's serve
  wiring, `PHASE-7.1.4`).
- **The claim is APP-SET — the honest limit stands:** a role that can issue
  the GUC sees any tenant's rows, so the layer stops application scoping
  bugs (the second layer per ADR-034), never a database-credentialed actor.
- **The remaining tenant-keyed tables (authority/budget/identity/enrollment/
  inbox/recruitment — 19 tables) stay on the application layer** with the
  same trigger: their claim wiring is per-family retrofit work, named not
  hidden.
- **The purge stays DELETE-as-postgres** (superuser bypasses RLS) — no test
  harness change; TRUNCATE was rejected as the larger cross-suite change.
