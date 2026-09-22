---
answers:
  - Why is GET /v1/health unauthenticated when every other operator read is authorized?
  - What may the health read disclose, and what does it withhold?
  - Why are dependencies probed on an interval rather than when the route is read?
  - Which dependencies does rb-server probe, and what does each probe mean?
  - Why do the probes of one round run concurrently?
---
# Health is read without authority, and says only state and age

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.4.6.1.4`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §18.5 (its first bullet: *service and dependency health with
  freshness*), §6.6 (deployment profiles);
  `docs/decisions/2026-09-22_three-of-the-five-missing-operator-surfaces-have-no-stored-fact.md`
  (which named the conflict this record resolves)

## Context

Nothing probed a dependency after `rb-server` booted. DOC-0131 named the design conflict
to resolve before building anything: every operator read authorizes against PostgreSQL,
so a health read gated the same way could never report PostgreSQL down, which is the one
fact it most exists to report.

## Decision

1. **`GET /v1/health` requires no authority.** Its availability cannot depend on the
   store whose availability it reports.
2. **It discloses only what an unauthenticated read justifies:** each dependency's name
   (`postgres`, `secret_store`, `server_ca`, `publication_root`, all documented in the
   book), its state (`up`, `down`, `stale`, `unobserved`), and the instants
   `observed_at`, `last_up_at` and `age_ms`. ⛔ A failed probe's error text is never in
   the response, because it can carry a host, a database name, a path or a driver
   message. It goes to the structured log (`dependency_health_changed`) on every change
   of state, where the operator holding the terminal reads it. The live control asserts
   that the response names neither the scratch database nor the publication directory it
   removed.
3. **Probes run on an interval, and reads never probe.** A read is then cheap and cannot
   drive load onto a dependency. And *freshness*, §18.5's own word, becomes observable: an
   observation that stops being refreshed ages, and once it is older than the staleness
   bound (three probe intervals, 30 s as shipped) it reads `stale` whatever it said. A
   stalled prober is itself a fault, and a check made at request time could never show it.
4. **Each probe means the dependency does its job, not merely that it exists.**
   - `postgres`: `SELECT 1` answers.
   - `secret_store`: the declared store answers for the CA material, which is the read
     the boot makes.
   - `server_ca`: the certificate is inside the validity window it carries.
   - `publication_root`: the boot's own `validate_root` still passes. It is probed only
     when a root is declared.

   Each probe is bounded at 2 s, and a slower answer counts as down.
5. **One round's probes run concurrently.** A real `rb-server` run with PostgreSQL
   stopped showed that sequential probing held every later dependency's observation for
   the whole timeout. The control `one_hung_dependency_does_not_delay_the_others` was
   observed RED on the sequential shape (4.01 s for two hung probes) and GREEN on the
   concurrent one (2.01 s).
6. **The status code carries the verdict:** 200 when every dependency is up and fresh,
   503 otherwise, with the same body either way.

## Consequences

- ✅ §18.5's first bullet is exposed. The book's *What an operator cannot see yet* table
  loses its health row.
- ⚠️ **An unauthenticated route widens what an unauthenticated caller learns** — which
  dependencies exist, and whether they are up. At the Developer profile's loopback bind
  that caller is the operator. ⛔ **At a non-loopback bind this disclosure must be
  re-judged together with the OpenTelemetry trigger DOC-0124 set on the same condition**:
  a deployment reachable by other hosts may want the route behind a network boundary,
  or reduced to the overall status alone.
- ⚠️ Probe outcomes are process-local. With a second control-plane process each process
  reports its own view. That is the same trigger again, and the OpenTelemetry sink is
  where a combined view would come from.

## What would make this wrong

- ⛔ If any probe's error text ever reaches the response body, decision 2 is broken. The
  live control checks that the response contains neither the database name nor the path.
- ⛔ If the prober task dies without the process dying, every observation turns `stale`
  within 30 s. That is the designed signal, not a defect. But a restart policy that
  masks it would be.
