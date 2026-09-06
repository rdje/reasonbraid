# The development authority engine: a boundary is a ceiling, a grant is a subset, and every decision is audited

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.5.1` (WP5 development `EnrollmentAuthorityBoundary` + scoped commands + authorization audit record)
answers: how does the development profile prove that tenant membership alone grants no mandate or administrative authority, that every command records actor / delegated subject / grant and boundary references / decision / policy digest and version, and that a grant can never exceed the enrollment ceiling?

## The fact / decision

`reasonbraid-core/src/authority.rs` lands the authority MODEL and
`crates/reasonbraid-server/src/authority.rs` (+ `migrations/0004_authority.sql`) the
authority ENGINE:

- **The subset rule is a pure, deterministic checker** (`grant_exceeds_boundary`):
  actions, risk ceiling, spend limits, delegation, and the validity window of a grant
  must each fit inside the applicable [`EnrollmentAuthorityBoundary`]. It is enforced
  TWICE: at grant creation (`create_grant` refuses and stores nothing) and at every
  evaluation (defense in depth, §4.4).
- **Membership grants nothing.** Evaluation requires an ACTIVE grant for the
  principal; without one the decision is `denied: "no applicable grant: tenant
  membership alone grants no authority"` — and `TenantAdmin` is never implied by other
  actions.
- **Every command records its authorization.** `authorize` writes an
  `authorization_records` row — actor, delegated subject when present, boundary +
  grant references, the decision with its denial reason, and the policy digest +
  version — for ALLOWANCES AND DENIALS. `apply_authorized_command` runs that record
  and the WP2 durability writes in ONE transaction (the `.2.1` body was extracted to
  `apply_command_in_tx` for this): an accepted command implies its audit record, and
  a denied command commits its denial record and applies NOTHING.
- **The policy digest** is SHA-256 over a documented field-order input (boundary id,
  charter digest, policy version, grant id, subject, action, target, decision) — the
  record names exactly WHICH policy it was decided under, and identical inputs always
  digest identically (tested).
- **Scoping is typed:** a grant's [`TargetSelector`] is tenant-wide or a named thread
  set; `thread_create` targets the tenant; a thread outside the selector is denied.
- **Development-profile limits, stated:** no certificate issuer (development
  credentials; the caller supplies a resolved actor); delegation CHAINS out of scope
  (direct grants only; depth is recorded on the boundary); the dev trust store is the
  control plane's own PostgreSQL. Policy engines (OPA/Cedar) remain §16.4 Phase 2
  candidates.

## Why

KICKOFF WP5 acceptance: "tenant membership alone does not grant mandate or
administrative authority"; "every command records actor, subject if delegated,
grant/boundary reference, decision, and policy digest/version"; "a grant cannot
exceed the enrollment ceiling" (`ROADMAP.md` §4.2–§4.5). This is the experiment for
kill-risk question 5 (are the identity/authority contracts small enough to evolve
beside working code?) — and the foundation `.5.2`'s budget reservation builds on.

## Measured behavior (legs: re-derive · falsify · durable)

- Re-derived live (PostgreSQL 16.15): `bash scripts/run_pg_tests.sh` →
  `test result: ok. 9 passed` (`authority`) — membership-without-grant denied AND
  audited with no domain effect; admin authority never implied; an accepted command's
  record carries actor + delegated subject + grant/boundary + allowed decision + a
  64-hex digest that RE-DERIVES from the same inputs; overreaching grants refused at
  creation (nothing stored); scope denials outside the named thread set; expired
  grants denied; a boundary-less tenant denied; identical evaluations digest
  identically. `cargo test -p reasonbraid-core` → `test result: ok. 31 passed`
  (subset checker per dimension, liveness windows, digest binding, wire-name
  parsers).
- Falsified (the checker was MORE precise than the fixtures): the first live run
  failed 6/9 — every grant "outlived its boundary" because each fixture helper read
  its own `Utc::now()`, so a grant built microseconds after its boundary exceeded the
  window by those microseconds. The fixtures now use wide boundary windows; the
  failure itself is evidence the temporal subset rule really binds. A second failure
  taught a serde rule: an externally-tagged newtype variant cannot wrap a sequence,
  so `TargetSelector::Threads` is struct-like (`{ threads: [...] }`).
- Durable: model + engine + migration + tests are tracked; the proof command is
  `scripts/run_pg_tests.sh` (the `pg-tests` CI job now runs all four suites).

## Rejected designs

- **Membership-implies-authority shortcuts** — rejected by the acceptance itself; a
  principal with no grant is denied, full stop.
- **Enforcement in the database only** — constraints can't express the subset rule or
  the digest; the checker is Rust, evaluated at both boundaries, with the DB as the
  durable store.
- **Records only for allowances** — a refused command is an audited event too;
  denial records carry the reason and the policy digest.
- **OPA/Cedar now** — §16.4 names them candidates; the dev profile needs ONE
  deterministic evaluator, and a policy engine is scope for Phase 2 (recorded).
- **Digesting only the policy, not the decision** — the record must prove what was
  DECIDED under that policy; the decision (including the denial reason) is part of
  the hashed input.
- **Timestamps as strings here** — unlike the wire envelopes (ADR-010 still open for
  those), these are internal domain types persisted via sqlx chrono bindings, not a
  wire contract; no schema claim is made (`JsonSchema` deliberately absent).

## How to apply

- Add an action/ceiling by extending the checker FIRST — a grant that exceeds the
  boundary must be unrepresentable, not merely unapproved.
- Never evaluate from memory of a grant: load the stored row, re-check the subset
  rule, then decide — every time.
- Keep the digest input field order documented; changing it invalidates stored
  digests (bump `policy_version` when the input shape changes).
- `.5.2`'s reservation check goes AFTER this evaluation (no dispatch without an
  applicable reservation — and no reservation without an applicable grant).
