---
answers:
  - Who may mutate shared adapter and region registries?
  - Can tenant enrollment issue site-operator authority?
  - How must registry mutations serialize with revocation?
---
# Explicit site-operator authority for shared registries

- Status: accepted design; implementation pending `SIGNOFF-REPAIR.3.2`.
- Owner: repo-local engineering, acting on the director's explicit delegation.
- Date: 2026-09-09.
- Sources: `ROADMAP.md` §§4.4–4.5, 16.4, 20.10; ADR-027 and ADR-035.
- Supersedes: tenant-admin authority for global mutations in `PHASE-8.4.4` and `PHASE-8.5.2`; their historical test results remain historical evidence.

## Context and source evidence

At baseline `9c2d2ba`, `require_admin_any_tenant` in `crates/reasonbraid-server/src/api.rs` authorizes shared adapter and region registry handlers using an active tenant-admin grant from any tenant. Its SQL omits enrollment-boundary validation. The region and allowlist fixtures expect an ordinary newly enrolled tenant administrator to mutate global rows.

This conflicts with tenant scope and the administrative freeze contract in `docs/decisions/2026-09-07_boundary-revocation-freeze.md`. This is source inspection; runtime reproduction is pending in `SIGNOFF-REPAIR.3.2`.

## Decision

1. Shared registry mutations require a distinct, explicitly issued **site-operator grant**. Tenant administrator authority has no implicit site capability. Existing tenant grants receive no automatic upgrade.
2. Site authority is rooted in an operator-controlled boundary, with explicit permitted actions, validity window, subject and revocation state. Evaluation resolves the grant's actual parent boundary. Tenant-facing enrollment and tenant-admin APIs cannot create, widen or restore site authority.
3. Issuance and revocation use protected operator tooling under deployment control. It records issuer, subject, boundary, actions, expiry and reason. Enrollment alone cannot bootstrap site authority through a network endpoint.
4. Each mutation validates current grant and boundary liveness and scope in the transaction that writes the registry and its audit. A consistent lock order serializes it with revocation. Revocation committed first prevents later mutations; previously committed mutations remain attributable history.
5. Allowed and denied attempts leave durable audit records naming actor, action, target, authority references, reason, decision and time. Audit failure prevents an allowed mutation from committing. No-op operations remain attributable.
6. Tenant administrators retain approved tenant-scoped inspection and frozen-boundary read behavior. Global inspection gets an explicit read policy and conveys no mutation rights. Shared resolver/workflow writes receive the same scope census in `SIGNOFF-REPAIR.7.1` and `SIGNOFF-REPAIR.8.1`.
7. Site authority does not strengthen development authentication. The principal header remains a development trust assumption. Internet authentication and external G6/G7 qualification remain required.

The schema and CLI wire shape belong to the implementation leaf and receive tested examples when shipped. This decision adds no unimplemented public command to the manual.

## Required proof

Demonstrate operator success; tenant-admin refusal across multiple tenants; refusal after grant or boundary revocation, suspension, expiry or before validity; rejection of enrollment-based escalation; unchanged victim state on refusals; audit persistence; and deterministic revocation/mutation races in both commit orders. HTTP status alone is insufficient.

## Consequences

Existing success fixtures must obtain explicit site authority. Deployments will need an operator-issued grant for these mutations after the repair. Existing registry entries confer no authority. Correct historical qualification claims through `docs/tasks/SIGNOFF-REPAIR.md`; preserve source evidence under `docs/tasks/artifacts/signoff_review/`.

