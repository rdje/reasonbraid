# Authority

The control plane is model-neutral: Rust — not an LLM — decides what is
authorized (`ROADMAP.md` §4). This chapter describes the Phase 0 development
authority model: the enrollment ceiling, the scoped grants under it, and the
audit record every command leaves.

## The ceiling and the subset rule

An **enrollment boundary** (`§4.4`) is the root/parent-granted ceiling for a
tenant: permitted actions, domains, a risk ceiling, a spend ceiling, whether
delegation is allowed, and a validity window. A **grant** is a scoped mandate
under one boundary.

The subset rule is deterministic and enforced twice:

- at **grant creation** — a grant whose actions, risk, spend, delegation, or
  window exceeds its boundary is refused and never stored;
- at **every evaluation** — the stored grant is re-checked against the stored
  boundary before any decision.

Tenant membership alone grants **nothing**: a principal without an active
grant is denied, and administrative authority (`tenant_admin`) is never
implied by other actions.

## Scoped commands

A grant's selector is tenant-wide or a named thread set, and each action has a
target shape: `thread_create` targets the tenant; `thread_invite`,
`thread_contribute`, `thread_inspect`, `thread_close`, and `thread_cancel`
target a thread that must be inside the selector. Out-of-scope targets are
denied with the reason recorded.

## The audit record

Every command — accepted or refused — writes an `authorization_records` row:

```text
record_id · tenant · actor · subject (if delegated) · boundary · grant
action · target · decision (allowed | denied + reason)
policy_digest · policy_version · decided_at
```

The digest is SHA-256 over a documented field-order input (boundary id,
charter digest, policy version, grant id, subject, action, target, decision),
so the record names exactly which policy produced it — and identical inputs
always digest identically. An accepted command's audit record commits in the
SAME transaction as its state, event, idempotency, and outbox writes; a
denied command commits its denial record and writes nothing else.

## Honest limits (Phase 0)

- Development credentials: no certificate issuer; the caller supplies a
  resolved actor. Authentication arrives with the real identity work.
- Direct grants only — delegation chains are Phase 2 (the boundary records
  the permitted depth).
- The dev trust store is the control plane's own PostgreSQL; policy engines
  (OPA/Cedar, §16.4) remain Phase 2 candidates.
- `.5.2` adds the other half of WP5: no provider dispatch without an
  applicable budget reservation.
