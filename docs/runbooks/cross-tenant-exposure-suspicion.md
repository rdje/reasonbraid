# Runbook: cross-tenant exposure suspicion

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN)
- Scope: a suspected cross-tenant read or write — one tenant seeing or
  touching another tenant's records.

## Detection

- **The RLS proof:** the probe-role measurement (the `rls` suite) — the
  fail-closed policies read zero rows under a foreign claim. A suspicion is
  checked by RE-DERIVATION: run the same query under the foreign claim and
  verify it sees nothing.
- **The non-escalation signals:** the guard's adversarial suite (the
  cross-tenant 403/404/409 refusals) — a NEW cross-tenant acceptance is the
  alarm.
- **The operator's suspicion:** a reported observation, a code review
  finding.

## Authority

- Freezing the suspect tenant: `tenant_admin` (the boundary-revocation
  freeze — the reads stay inspectable, the writes stop).
- Reading: the audit surfaces (read-only, the freeze never hides them).

## Safe first actions

1. **Freeze, then verify.** The boundary revocation stops every write while
   the investigation reads (the `.1.3.2` freeze semantics).
2. **Do not "fix" the data.** The suspected exposure is proven or refuted
   by the re-derivation — never by editing rows.

## Diagnostic queries

- The foreign-claim probe (the rls suite's own legs, re-run against the
  suspect surface).
- `GET /v1/admin/grants` + the boundary (what the suspect tenant could
  reach).
- The audit rows for the suspect window (who read/wrote what).

## Containment

- The freeze + the grant revocation (the tenant's writes stop).
- The defense-in-depth holds: the tenant-scoped keys (the first layer) +
  the RLS (the second) — a proven exposure names WHICH layer failed, and
  the fix lands in that layer.

## Recovery

- **The exposure is refuted:** the freeze lifts (the operator's recorded
  decision), the re-derivation evidence stands.
- **The exposure is proven:** the affected layer fixes (the missing
  tenant-scoped key, the unwired claim — the `.1.3.1` named deferrals are
  the candidate list), the regression test lands (the non-escalation
  suite's next row), the affected tenants are told.
- **The honest dev stance:** the app role stays the superuser today (the
  RLS binds only when the non-superuser role lands — the `.1.3.1` named
  deferral); the suspicion handling is therefore the APPLICATION-layer
  proof until then.

## Evidence preservation

- The probe's re-derivation output (the before/after rows).
- The audit rows for the suspect window.
- The freeze/revocation records.

## Communication

- The operator reports the suspicion (the re-derivation verdict) to the
  accountable owner; a proven exposure is a declared incident.

## Closure tests

- The non-escalation suite (the four adversarial legs) + the rls suite
  (the probe-role refusal) + the escalation-freeze tests on every guard
  pass.
