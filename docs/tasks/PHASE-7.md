# PHASE-7: Internet-qualified operation

## Metadata

- Tree ID: `PHASE-7`
- Status: `active`
- Roadmap lane: Phase 7 (`ROADMAP.md` §20.9); Trust track (scoped)
- Created: `2026-09-05`
- Estimate: 18–32 engineer-weeks plus external review
- Depends on: Phase 1, applicable Phase 2 trust/recovery, and every feature-specific gate for the surface being exposed. Does **not** require Phases 3–6 for capabilities that remain disabled and unclaimed.
- Exit: G6–G7 for a named capability profile only.

## Goal

Internet capability is claimed only for a qualified feature set. Adding remote
discovery, arbitrary resources, governed policy, or another adapter later
reopens the applicable portions of G4–G7.

## Non-Goals

- Permissionless or anonymous public agent network.
- Exposing unfinished tracks behind an experimental default.

## Task Tree

- ID: `PHASE-7.1`
  Status: `done`
  Goal: hardened ingress/egress, mTLS workload identity, tenant isolation, quota/abuse, secret-manager integration, regional/data-class controls
  Roadmap: §16.2, §16.8, §16.11
  Children: `.1.1`–`.1.4` (decomposed `2026-09-07` at the census
    seams): `.1.1` ADR-034 + the census (the hardening
    contract: the qualified-surface rule — the Internet
    capability is claimed per the NAMED feature set, the
    transport-context rule, the defense-in-depth isolation,
    the quota/abuse vocabulary) → `.1.2` the mTLS workload
    identity (the transport hardening — the cert-fingerprint
    → the principal binding) → `.1.3` the tenant isolation +
    the quotas/abuse (the RLS defense-in-depth + the
    per-principal/tenant quotas riding the budget machinery)
    → `.1.4` the secret-manager integration + the
    regional/data-class controls.
  Done (`2026-09-07`): the census at the seams — the block is
    LIFTED (the frontier row's "blocked on Phase 1 +
    applicable Phase 2" resolves: the Phase-1 channel + the
    CA/certificate infra ship, the Phase-2 authority/budget
    ship). The SHIPPED halves: the authenticated channel
    with the CA-issued node certs (the mTLS groundwork —
    LAN-grade), the tenant_id on every aggregate key (the
    §16.8 shape, the RLS unshipped — the named
    defense-in-depth), the budget ceilings/reservations/
    breakers (Phase 2 — the per-thread spend; the
    per-principal/per-resolver quotas unshipped), the
    Classification (General/Confidential — the data-class
    shape, the regional controls unshipped). The GREENFIELD:
    no secret-manager integration, no regional controls, no
    per-principal quotas, no mTLS principal binding at the
    transport. The §23 queue has no hardening item — the
    lane opens ADR-034. Frontier → `.1.1`.

  - ID: `PHASE-7.1.1`
    Status: `done`
    Goal: ADR-034 + the census — the hardening contract:
      the QUALIFIED-SURFACE rule (the Internet capability is
      claimed per the named feature set — the G6/G7 gates
      reopen per surface), the transport-context rule (the
      overlay/VPN/address/token is the context, never the
      identity), the defense-in-depth isolation (the RLS is
      the second layer, never the only one), the
      quota/abuse vocabulary. No code.
    ADR: 034
    Roadmap: §16.2, §16.8, §16.11
    Done (`2026-09-07`): ADR-034 accepted (evidence-gated) —
      `docs/adr/034-internet-hardening.md` (top-level
      `answers:`): the Internet capability is claimed per
      the QUALIFIED SURFACE (the exit names the feature set;
      an unqualified surface exposed by default is the typed
      refusal — never "experimental-default"); the transport
      is CONTEXT, never identity (the certificate
      fingerprint is the principal; the reimaging/
      incarnation separation is structural — a changed host
      never inherits the history); the isolation is DEFENSE
      IN DEPTH (the tenant-scoped keys are the first layer,
      the RLS is the SECOND — never the only one; the
      cross-tenant recruitment stays the explicit opt-in);
      the quotas bound the abuse by the principal/tenant/
      resolver/destination (the vocabulary rides the
      Phase-2 budget machinery; the quarantine preserves the
      evidence); the secrets and the regions are DECLARED
      profiles (the external store is a configuration
      choice; a classification without the controls is the
      typed refusal). No code changed. Frontier → `.1.2`.

  - ID: `PHASE-7.1.2`
    Status: `done`
    Goal: the mTLS workload identity — the transport
      hardening: the cert-fingerprint → the principal
      binding at the channel (the TLS 1.3 default, the
      mutually authenticated node-to-control-plane), the
      reimaging/incarnation separation (a changed host never
      inherits the history).
    Roadmap: §16.2
    Done (`2026-09-08`): the mTLS transport shipped as the
      §16.2 config pair (the production serve wiring stays
      the deployment-profile concern — NAMED, not hidden):
      `ca::issue_serving_cert` (the ServerAuth-EKU serving
      leaf), `mtls::build_server_config` (TLS 1.3-only,
      ring-pinned, the `WebPkiClientVerifier` over the CA
      root — the client MUST chain to the deployment CA),
      `mtls::build_client_config` (the CA root + the node's
      leaf). The split holds per the census: the transport
      verifies the CA membership; the fingerprint → the
      principal binding stays the APPLICATION proof
      (`node_channel::verify_cert_proof`) — the layered
      defense, never the transport alone. The offline
      roundtrip proves both legs: the CA-issued client
      completes the mutual handshake + a byte; the
      cert-less client is refused at the transport (the
      server's accept errors + the client's first read
      delivers the alert/EOF, never data). The FIRST test
      draft failed on a real TLS 1.3 asymmetry — the
      client's connect() completes before the server's
      certificate_required alert arrives, so the refusal is
      read-side, never connect-side — promoted to
      `docs/decisions/2026-09-08_tls13-refusals-are-read-side.md`
      (top-level `answers:`) as the future TLS tests'
      contract. Frontier → `.1.3`.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the transport
      hardening is the mTLS config pair + the serving leaf:
      `rustls` 0.23 pinned ring (the workspace provider
      rule), TLS 1.3-only per §16.2; the client verifier is
      `WebPkiClientVerifier` over the deployment CA — the
      proof is the OFFLINE roundtrip (loopback TLS, no
      Postgres) in `tests/mtls.rs`: the issued leg
      completes, the cert-less leg is refused at the
      transport. Evidence: `cargo test -p reasonbraid-server
      --test mtls` → `test result: ok. 1 passed; 0 failed`.
    - [x] **ADDRESSED** — the certificate is CONTEXT, never
      identity (ADR-034): the transport checks the CA
      membership only; the principal binding (fingerprint →
      node) remains the application's `verify_cert_proof`
      — the two layers measured separately (the channel
      suite + this transport suite). Evidence: the client
      refused at the transport carries NO cert at all; the
      roundtrip's accepted client presents the CA-issued
      leaf. `cargo test --all` → the full offline sweep
      rc=0.
    - [x] **NO REGRESSION** — `cargo test -p
      reasonbraid-server --test mtls` → `test result: ok.
      1 passed; 0 failed`; `cargo test --all` → rc=0, 64
      offline suites (63 + the new mtls suite)
      (`target/mtls_offline.log`); `cargo clippy
      --all-targets -- -D warnings` → clean; `cargo fmt
      --all -- --check` → rc=0; `make gate` → 13/13.
    - [x] **LESSON PROMOTED** — the TLS 1.3
      connect-side-refusal trap: `docs/decisions/2026-09-08_tls13-refusals-are-read-side.md`
      (top-level `answers:`).

  - ID: `PHASE-7.1.3`
    Status: `done`
    Goal: the tenant isolation + the quotas/abuse — the RLS
      defense-in-depth (the second layer over the
      tenant-scoped keys), the per-principal/tenant/
      resolver quotas riding the Phase-2 budget machinery,
      the quarantine-preserving-evidence rule (§16.11).
    Roadmap: §16.8, §16.11
    Done (`2026-09-08`): the census at the seams —
      decomposed (the vocabulary ships in ADR-034; the
      machinery is the greenfield). Measured: `grep -c
      "tenant_id" migrations/*.sql` → the first-layer key
      on the identity/authority/budget/inbox/enrollment
      surface (0001/0003–0009/0012/0016/0017/0020); RLS →
      NOTHING (`grep -rn "ROW LEVEL SECURITY" migrations/`
      → rc=1 — the named defense-in-depth is unshipped);
      quotas → NOTHING (`grep -rln "quota" crates/
      migrations/` → one fixture word, no machinery); the
      quarantine rows preserve the evidence in-place
      (`quarantined_at` + `quarantine_reason`, migration
      0010) but the §16.11 rule is unarticulated. Children:
      `.1.3.1` the RLS layer (the DB-level cross-tenant
      refusal measured) → `.1.3.2` the quotas (the
      per-principal/tenant/resolver ceilings riding the
      Phase-2 budget machinery) → `.1.3.3` the
      quarantine-preserving-evidence rule (the §16.11
      contract articulated over the shipped quarantine).
    Children: `.1.3.1`–`.1.3.3`

  - ID: `PHASE-7.1.3.1`
    Status: `done`
    Goal: the RLS defense-in-depth — migration 0046: the
      per-table RLS policies over the tenant-scoped keys
      (the SECOND layer — the application scoping is the
      first), the tenant-claim connection wiring, and the
      measured DB-level cross-tenant refusal (a
      foreign-tenant query refuses at the DATABASE, not the
      application). The policy design + the claim pattern
      ride a decision record.
    Roadmap: §16.8
    Done (`2026-09-08`): the RLS layer lands on the COMMAND
      CORE (the enablement set matches the wired set):
      migration 0046 enables + FORCES the policies on
      `aggregate_state`/`event_log`/`idempotency` (the
      aggregate head, the audit trail, the replay
      protection) — `USING/WITH CHECK (tenant_id =
      current_setting('app.tenant_id', true))`, fail-closed
      (the unset claim matches no row). The claim is the
      TRANSACTION-LOCAL `app.tenant_id` GUC: `rls.rs`
      (`set_tenant_claim` + the boxed-future
      `with_tenant_claim` read wrapper); the two write
      entries (`agg::claim_in_tx`, `agg::apply_fresh_in_tx`)
      set it as the transaction's FIRST statement (the
      in-tx validation helpers inherit it); the three
      inspection reads in `api.rs` + the two policy-lane
      EXISTS checks in `lifecycle.rs` (now tenant-threaded
      through the handlers) route through the wrapper. The
      measured proof (`tests/rls.rs`, LIVE, joined the
      guard — 22 suites): the NON-superuser probe role
      (superusers bypass RLS unconditionally — the dev
      profile connects as postgres, so the layer is proven
      as it will bind) reads ZERO rows unset (fail-closed),
      sees only its tenant's rows under the claim, and a
      foreign-tenant INSERT is REFUSED by the WITH CHECK at
      the DATABASE (nothing stored). The NAMED deferrals:
      the deployment-profile role change that BINDS the
      layer (the non-superuser app role + the grants — the
      `.1.4` serve-wiring profile), the outbox exemption
      (the worker's cross-tenant queue), and the remaining
      19 tenant-keyed tables (the per-family wiring is
      follow-on) — each with the trigger, in the decision
      record.
    Decision: `docs/decisions/2026-09-08_rls-tenant-claim.md`
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the §16.8
      defense-in-depth was unshipped (the `.1.3` census:
      `grep -rn "ROW LEVEL SECURITY" migrations/` → rc=1);
      the layer lands on the command core because the
      enablement set must equal the wired set (a claim-less
      path sees NOTHING under the fail-closed policy — the
      wiring census: 4 modules touch the three tables, the
      worker's outbox stays exempt). Evidence: `cargo test
      -p reasonbraid-server --test rls` →
      `test result: ok. 1 passed; 0 failed` (the probe-role
      refusal legs).
    - [x] **ADDRESSED** — migration 0046 + the claim wiring
      (`agg`'s two entries set the transaction-local GUC
      first; the pool-direct reads route through
      `with_tenant_claim`); the measured refusal: the unset
      claim reads 0 rows, the foreign claim reads 0 of the
      other tenant's rows, the foreign INSERT is refused at
      the DATABASE. Evidence: the rls suite's legs (the
      guard log `target/pg_rls_guard4.log`); the demo stays
      green (the superuser connection bypasses RLS — the
      named dev-profile stance).
    - [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh`
      → rc=0, 22 suites + the demo `ALL acceptance checks
      passed` (`target/pg_rls_guard4.log`);
      `cargo test --all` → rc=0, 65 suites (64 + the rls
      suite's offline skip — `test result: ok. 0 passed`)
      (`target/rls_offline.log`); clippy/fmt clean;
      `make gate` → 13/13.
    - [x] **LESSON PROMOTED** — the superuser-bypass +
      transaction-local-claim design:
      `docs/decisions/2026-09-08_rls-tenant-claim.md`
      (top-level `answers:`).

  - ID: `PHASE-7.1.3.2`
    Status: `done`
    Goal: the quotas — the per-principal/tenant/resolver
      quota tables + the reservation-path check riding the
      Phase-2 budget machinery (the same in-transaction
      claim pattern; a quota denial is a recorded budget
      event, never silent), the §16.11 abuse vocabulary
      bound per ADR-034.
    Roadmap: §16.8, §16.11
    Done (`2026-09-08`): migration 0047 — `usage_quotas`
      (the per-key WINDOWED ceilings: `scope_kind` ∈
      tenant/principal/resolver/destination, the ceiling +
      the window, UNIQUE per key) + `quota_events` (the
      recorded uses AND denials — the audit). The check
      rides the budget-machinery pattern: `quota.rs`'s
      in-tx `check_in_tx` counts the window's uses, records
      a `use` under the ceiling, records a `denial` + the
      typed refusal at it; the events commit WITH the
      guarded action (a rolled-back invite rolls its use
      back). FAIL-CLOSED: a scope with no quota row is the
      typed `quota_unconfigured` (503) — the surface
      refuses until a bound exists (ADR-034's "never
      silent"); the migration BACKFILLS the dev default
      (1000 invites/hour) for existing tenants, and the
      enroll path creates it in the tenant's OWN
      transaction (a tenant exists with its bounds). The
      SHIPPED binding: the per-tenant INVITE bound (the
      invitation-storm surface) — `thread.invite` checks
      before dispatching; the quota refusal rides the
      authorization-denied pattern (the rejection stores +
      the denial row COMMITS — never a silent rollback);
      the wire gains `quota_exceeded` (429) +
      `quota_unconfigured` (503). The NAMED deferrals (each
      with its trigger): the principal/resolver/destination
      bindings (the Internet profile's fetch + the public
      enrollment surfaces). The guard's migration_upgrade
      seed now writes the pre-upgrade schema's OWN shape
      (the new enroll depends on the new schema — the
      honest upgrade boundary, recorded in the test).
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the abuse
      vocabulary was unbound (the `.1.3` census: no quota
      machinery; ADR-034 names the invitation storms); the
      bound lands on the invite dispatch — the only
      storm-capable shipped surface — riding the budget
      machinery's in-tx + recorded-denial pattern.
      Evidence: `cargo test -p reasonbraid-server --test
      quota` → `test result: ok. 1 passed; 0 failed` (the
      ceiling crossing, the recorded denial, the window
      slide, the fail-closed unconfigured stance).
    - [x] **ADDRESSED** — migration 0047 + `quota.rs` +
      the `thread.invite` gate + the enroll default + the
      backfill; the refusal is a recorded event (the
      denial row) and the stored rejection replays with
      the original 429/503. Evidence: the quota suite's
      legs (uses == the accepted invites; denials == 1;
      the slide releases; the unconfigured refusal
      records nothing) — `target/pg_quota_guard8.log`.
    - [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh`
      → rc=0, 23 suites + the demo `ALL acceptance checks
      passed` (`target/pg_quota_guard8.log`); the
      FK-purge ripple (the quota tables reference
      `tenants`) fixed across all 14 purge lists + the CLI
      e2e; `cargo test --all` → rc=0, 66 suites
      (`target/quota_offline.log`); clippy/fmt clean;
      `make gate` → 13/13.
    - [x] **LESSON PROMOTED** — the upgrade-boundary
      coupling (the new app's enroll depends on the new
      schema — the migration_upgrade seed rides the
      pre-upgrade schema's raw shape): recorded in the
      test's own header (the test IS the re-derivation
      point); the quota design itself rides the ADR-034
      contract, already accepted.
      `promotion: declined (the upgrade-boundary fact is per-slice history, recorded in the migration_upgrade test it shapes)`

  - ID: `PHASE-7.1.3.3`
    Status: `proposed`
    Goal: the quarantine-preserving-evidence rule (§16.11)
      — the articulated contract: the quarantine is a ROW
      FACT (the evidence stays; the preservation survives
      the disposition), the retention cleanup stays the
      explicit operator action (the `.1.2.3` shape), no
      quarantine path may delete the evidence it cites.
    Roadmap: §16.11

  - ID: `PHASE-7.1.4`
    Status: `proposed`
    Goal: the secret-manager integration + the regional/
      data-class controls — the external secret-store
      interface (the declared profiles), the
      classification-driven region/retention/export
      decisions (the §16.8 thread-classification controls).
    Roadmap: §16.8

- ID: `PHASE-7.2`
  Status: `proposed`
  Goal: public-node enrollment and quarantine, revocation propagation, signed software updates, SBOM/provenance, disclosure process
  Backlog: 40
  ADR: 027
  Roadmap: §16.10, §16.12

- ID: `PHASE-7.3`
  Status: `proposed`
  Goal: horizontally scalable coordinator workers only where measurements require them
  ADR: 002 (extraction criteria)

- ID: `PHASE-7.4`
  Status: `proposed`
  Goal: capacity/load tests, incident exercises, penetration-test remediation, production runbooks
  Roadmap: §16.12, §18.6

- ID: `PHASE-7.5`
  Status: `proposed`
  Goal: G6–G7 exit for a named capability profile; subtraction record; explicit unsupported matrix
  Gate: G6, G7
  Kill/pivot: do not expose remote enrollment if the qualification gate is incomplete (`ROADMAP.md` §25.1)
  ADR: 022

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-7.1.3.3` | `proposed` | `.1.3.2` done — the quota machinery ships (migration 0047 + the in-tx check + the recorded denials + the invite-storm binding + the measured suite); the quarantine-preserving-evidence rule executes next |

## Changelog

- `2026-09-08`: `.1.3.2` done — the quotas (migration
  0047's windowed per-key ceilings + the recorded
  use/denial events; the fail-closed unconfigured stance +
  the backfill + the enroll default; the per-tenant invite
  binding with the commit-on-refusal pattern; the
  measured storm/denial/slide/unconfigured legs); the
  principal/resolver/destination bindings named with their
  triggers; frontier → `.1.3.3`.
- `2026-09-08`: `.1.3.1` done — the RLS defense-in-depth
  (migration 0046: the fail-closed policies on the command
  core; the transaction-local claim; the measured
  probe-role refusal — the unset claim sees nothing, the
  foreign-tenant write is refused at the DATABASE); the
  role-change/outbox/19-table deferrals named with their
  triggers; frontier → `.1.3.2`.
- `2026-09-08`: `.1.3` done — the census at the seams
  (the RLS + the quota machinery are the greenfield; the
  quarantine rows already preserve the evidence) →
  decomposed `.1.3.1` (the RLS layer) → `.1.3.2` (the
  quotas) → `.1.3.3` (the quarantine-evidence rule);
  frontier → `.1.3.1`.
- `2026-09-08`: `.1.2` done — the mTLS workload identity
  (the §16.2 config pair + the serving leaf + the offline
  roundtrip: the CA-issued client connects, the cert-less
  client is refused at the transport; the TLS 1.3
  connect-side-refusal trap promoted to a decision record);
  frontier → `.1.3`.
- `2026-09-07`: `.1.1` done — ADR-034 accepted (the
  hardening contract: the per-surface qualification, the
  transport-context rule, the defense-in-depth isolation,
  the quota vocabulary); no code; frontier → `.1.2`.
- `2026-09-07`: `.1` decomposed at the census seams — the block
  is LIFTED (the channel/CA + the authority/budget ship; the
  quotas/secrets/regions are the greenfield); children `.1.1`
  (ADR-034 + the census) → `.1.2` (the mTLS identity) →
  `.1.3` (the isolation + the quotas) → `.1.4` (the secrets +
  the regions); frontier → `.1.1`.
- `2026-09-05`: Created from `ROADMAP.md` §20.9, §16.12, backlog 40.
