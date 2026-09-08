# Phase 7 gate record — G6–G7 (`PHASE-7.5.2`)

## Context

`PHASE-7.5.2` (the G6–G7 exit gate package, the Phase-1 `.1.8.2` /
Phase-4 `.7.2` / Phase-5 `.6.2` / Phase-6 `.7.2` pattern): Phase 7
ships the Internet-hardening programme — the `.1` hardening lane
(ADR-034, the mTLS workload identity, the RLS layer, the quotas,
the quarantine-evidence rule, the secret-store profiles, the
classification controls), the `.2` supply-chain lane (ADR-027,
SECURITY.md, the signed release manifests, the public-enrollment
contract), the `.3` extraction criteria, the `.4` capacity/incident
lane (the load harness, the thirteen-family runbook catalogue, the
game-day + the pen-test stance). The gate: `ROADMAP.md` §16.12's
**G6/G7** — the ten-line Internet qualification. The kill/pivot
line (`ROADMAP.md` §25.1): do not expose remote enrollment to the
Internet if the qualification gate is incomplete.

## Decision

- **G6–G7 outcome: NOT MET for the Internet exposure — Met as the
  hardening-machinery exit for the LAN profile; the exposure stays
  UNCLAIMED.** The `.5`/`.5.1` census mapped the ten lines: SEVEN
  ship with the measured evidence (the enrollment/rotation/
  revocation/isolation suites, the non-escalation legs, the
  SSRF/archive-bomb suites, the signing pipeline, the restore +
  the compromised-key exercise, the rate-limit/breaker/storm
  tests, the runbooks/disclosure), and THREE are the EXTERNAL
  preconditions that do not exist yet — (1) the externally
  reviewed threat model, (5) the prompt-injection
  action-boundary suite, (9) the penetration test. The §25.1
  kill/pivot therefore HOLDS — deliberately: the phase never
  exposed remote enrollment, and the exit does not claim it.
- **The named capability profile:** the Trusted-LAN/Developer
  profile (the shipped + measured profile). The Internet profile
  remains the UNCLAIMED future whose preconditions are the three
  gaps — each with its trigger (the external review + the
  pen-test precede the exposure; the prompt-injection suite
  lands with the exposure profile's first action-bearing
  surface).
- **The unsupported matrix (explicit):** the Internet exposure
  (the public enrollment endpoint), the multi-tenant production
  operation under a non-superuser app role (the `.1.3.1` named
  deferral — the RLS binds only when the role lands), the
  external secret stores (the `dev_database` profile ships), the
  confidential-thread evaluation (no confidential-qualified
  evaluator), the dependency-graph SBOM (the artifact manifest
  ships), the horizontal scaling (zero extractions — the `.3`
  criteria), and the multi-region controls. Each matrix row is
  the explicit NOT-SUPPORTED statement; nothing is silently
  implied.

## answers:

- **The exit is honest by construction**: the gate's own census
  names the three gaps, and the outcome refuses the exposure
  rather than claiming it against missing evidence — the §25.1
  discipline applied.
- **The machinery claim is the measured one**: the seven shipped
  lines cite the guard suites that run on every pass — the claim
  re-derives from the guard, not from the narration.
- **The phase is CLOSED on the machinery + the LAN profile**;
  the Internet profile's qualification is the named follow-on
  whose gate lines are the three gaps — no re-litigation needed,
  the census is the standing map.
