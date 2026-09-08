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

The server's current grant selection and boundary lookup require correction:
the actual referenced boundary must be used, a later unrelated grant must not
hide another applicable grant, and tenant-target actions must reject thread-only
selectors. These are owned by `SIGNOFF-REPAIR.3.3`. Delegation and cached admission
decisions exist; their depth, consent and freshness constraints are under `.3.4`.

## Thread commands and audit

Thread creation targets a tenant. Invitation, contribution, inspection, close,
cancel and invitation-response actions target a thread. The normal command path
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

The shared adapter and region registries currently use an any-tenant-admin check.
The accepted replacement is a distinct site-operator grant, issued through protected
deployment tooling, with a live operator-controlled parent boundary. Tenant enrollment
cannot mint site authority. Each mutation and its audit must serialize with revocation.
Implementation is pending `SIGNOFF-REPAIR.3.2`; the decision is
`docs/decisions/2026-09-09_site-operator-authority.md`.

For example, an administrator of tenant A must not revoke tenant B's grant, prune
B's inbox or declare a shared site region merely by supplying A's tenant identifier.
These refusal cases must prove unchanged protected rows and epochs, alongside a
successful correctly authorized control. The current source does not enforce that
uniformly; `.3.1`, `.3.2` and `.3.5` own the repairs.

Site authority and workload certificate checks do not replace production caller
authentication. The development header trust and the open external G6/G7 gates remain
material deployment limits.
