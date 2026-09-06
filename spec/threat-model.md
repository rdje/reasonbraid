# Threat-model skeleton

**Status: draft — not normative.** The skeleton of the security model, distilled
from `ROADMAP.md` §16 (security and trust architecture) and §25 (risk register).
Backlog item 7: "enumerate trust boundaries, abuse cases, assets, mitigations,
and residual risks." A full, externally-reviewed threat model is a G6 gate
(§16.12) and is **not** produced here; this skeleton names the boundaries and
the asset/adversary/abuse/mitigation structure so the full model can be grown
without re-deriving them.

Security is part of the domain model, not an edge proxy (§16). Every request
answers four independent questions: who is this workload, on whose behalf is it
acting, what exact operation, and under which current grant and policy.

## Trust boundaries

A trust boundary is a line where untrusted input or a weaker-trust principal
meets a protected asset. The skeleton's boundaries:

| # | Boundary | Weaker side | Stronger side | Notes |
| --- | --- | --- | --- | --- |
| 1 | Ingress | unauthenticated Internet/client | control-plane API | deny-by-default; TLS 1.3; actor resolved server-side (§9.1) |
| 2 | Data store | control plane | PostgreSQL | row-level security as defense-in-depth, not the only barrier (§16.8) |
| 3 | Object store | control plane | content-addressed store | tenant namespace in every key (§16.8) |
| 4 | Node channel | node daemon | control plane | mutual-auth workload identity for Internet profiles (§16.2) |
| 5 | Adapter / sidecar | harness or vendor sidecar | node | supervised, minimal FS/network, no governance logic (§11.6–§11.7) |
| 6 | Provider API | node | external model provider | credentials stay local; ambiguity handled (§11.3) |
| 7 | Resolver egress | external resource | resolver worker | SSRF/DNS-rebinding/redirect/archive-bomb suite (§16.7) |
| 8 | Tenant isolation | tenant A | tenant B | tenant ID in every key; no cross-tenant path (§16.8) |
| 9 | Admin surface | human/admin | control plane | break-glass: scoped, expiring, alerted, audited (§16.11) |
| 10 | Publication | control plane | Git / policy repo | staged refs, signed manifest, CAS (§15.7–§15.8) |
| 11 | Untrusted content | fetched page/repo/model output | every consumer | prompt injection: typed separation, action-boundary validation (§16.6) |

## Assets (`ROADMAP.md` §16.1)

| Asset | Protection goal |
| --- | --- |
| Policy text, approval state, signing keys, authority grants, target credentials | confidentiality + integrity |
| Private thread content, resource snapshots, prompts, model outputs, cost data | confidentiality |
| Agent/human/host/tenant identities | integrity + availability |
| Ordering, provenance, audit, and outcome history | integrity (append-only, hash-chained) |
| Service availability and budget capacity | availability |

## Adversaries (`ROADMAP.md` §16.1)

The model includes: an unauthenticated Internet attacker; a malicious or
compromised node; a hostile tenant; an over-privileged administrator; a
compromised dependency or release pipeline; a prompt-injected resource; a
dishonest participant; a replaying client; a compromised model-provider account;
and ordinary operator error. It also models collusion and gradual authority
capture — "several agents agreed" is not an authorization proof.

## Abuse cases (`ROADMAP.md` §16.6, §16.11)

- Indirect prompt injection from fetched pages, repositories, comments, and
  model output, attempting to change instructions or trigger tools.
- Notification floods, invitation storms, expensive loops, and scraping —
  bounded by per-principal/tenant/thread/resolver/destination/provider quotas.
- Cross-tenant disclosure via a telemetry, object-namespace, or authorization
  mismatch.
- Confused-deputy delegation: a delegate attempting to widen authority.
- Provider ambiguity exploited to duplicate a charge or side effect.
- SSRF / DNS rebinding / redirect / archive-bomb against a resolver.

## Mitigations (`ROADMAP.md` §16, mapped)

| Concern | Mitigation | Source |
| --- | --- | --- |
| Identity & transport | short-lived workload certificates; one-time enrollment token; rotation/revocation as tested operations | §16.2 |
| Delegation | strict-subset attenuation; capability tokens attenuated, audience-bound, replay-resistant | §16.3 |
| Authorization | deny-by-default, versioned policy, enforcement at every boundary, cached-decision expiry/revocation | §16.4 |
| Secrets | reference-not-raw storage; JIT resolution in the constrained worker; scrubbing at ingress/egress | §16.5 |
| Untrusted content | typed separation of instructions/objectives/evidence; sandboxed parsing; action-boundary validation | §16.6 |
| Resolver isolation | separate constrained pool; DNS + destination checked together; loopback/private/metadata blocked | §16.7 |
| Tenant isolation | tenant ID everywhere; RLS as defense-in-depth; opt-in cross-tenant recruitment only | §16.8 |
| Audit integrity | append-only, hash-chained, periodically check-pointed; distinct from operational logs | §16.9 |
| Supply chain | pinned deps/toolchain; `cargo deny` + advisories + secret scan; SBOM/provenance; signed artifacts | §16.10 |
| Abuse & containment | quotas, circuit breakers, quarantine, break-glass with review | §16.11 |

## Residual risks

Residual (accepted/known-open) risks live in `docs/risks.md` (the live subset of
`ROADMAP.md` §25) and are owned there, not duplicated here. The G6 Internet
qualification gate (`ROADMAP.md` §16.12) lists the externally-reviewed evidence
required before any Internet-capable claim; this skeleton does not satisfy it.
