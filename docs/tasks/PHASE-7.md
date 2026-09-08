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
    Status: `done`
    Goal: the quarantine-preserving-evidence rule (§16.11)
      — the articulated contract: the quarantine is a ROW
      FACT (the evidence stays; the preservation survives
      the disposition), the retention cleanup stays the
      explicit operator action (the `.1.2.3` shape), no
      quarantine path may delete the evidence it cites.
    Roadmap: §16.11
    Done (`2026-09-08`): the census measured every
      quarantine path: the explicit + the dead-letter
      auto-quarantine are UPDATE-only row facts
      (`quarantined_at` + `quarantine_reason`), the replay
      re-arm clears the MARK (the delivery state), never
      the row — and the RETENTION path held the gap: the
      prune deleted EVERY old acknowledged row, and a
      dead-lettered row is acknowledged BY DEFINITION —
      the disposition destroyed the evidence. FIXED: the
      prune's DELETE gains `AND quarantined_at IS NULL`
      (the preservation survives the disposition). The
      contract articulates the rule in
      `docs/decisions/2026-09-08_quarantine-preserves-evidence.md`
      (top-level `answers:`): the quarantine is a ROW FACT;
      the retention never deletes a quarantined row; the
      re-arm is a disposition (the mark clears), never a
      destruction (the row + the payload + the events
      stay); the prune stays the ONLY age-based removal,
      operator-invoked, with the measured receipt. The
      measured suite (`tests/quarantine.rs`, LIVE — the
      guard's 24th): the old-acked quarantined row (with
      its reason) SURVIVES the prune; the old-acked
      non-quarantined row prunes; the recent row stays;
      the receipt matches (before 3 / deleted 1 / after 2).
      The archive/export disposition (moving the evidence,
      never deleting it) is the named deferral with the
      Internet-profile trigger.
    Decision: `docs/decisions/2026-09-08_quarantine-preserves-evidence.md`
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the §16.11 rule
      was unarticulated and the prune lacked the
      quarantine exclusion (a dead-lettered row is
      acknowledged by definition → the age sweep deleted
      the evidence). Evidence: the quarantine suite —
      `cargo test -p reasonbraid-server --test quarantine`
      → `test result: ok. 1 passed; 0 failed`.
    - [x] **ADDRESSED** — the prune's `AND quarantined_at
      IS NULL` + the articulated contract (the decision
      record). Evidence: the suite's legs (the
      quarantined row + its reason survive; the sweep
      still removes its target; the receipt matches) —
      `target/pg_quarantine_guard.log`.
    - [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh`
      → rc=0, 24 suites + the demo `ALL acceptance checks
      passed` (`target/pg_quarantine_guard.log`);
      `cargo test --all` → rc=0, 67 suites
      (`target/quarantine_offline.log`); clippy/fmt clean;
      `make gate` → 13/13.
    - [x] **LESSON PROMOTED** — the retention-vs-evidence
      gap (the disposition destroyed the evidence the rule
      exists to protect):
      `docs/decisions/2026-09-08_quarantine-preserves-evidence.md`
      (top-level `answers:`).

  - ID: `PHASE-7.1.4`
    Status: `done`
    Goal: the secret-manager integration + the regional/
      data-class controls — the external secret-store
      interface (the declared profiles), the
      classification-driven region/retention/export
      decisions (the §16.8 thread-classification controls).
    Roadmap: §16.8
    Done (`2026-09-08`): the census at the seams —
      decomposed. Measured: the dev profile's secrets are
      the plaintext rows (`server_ca`'s ca_der/key_der,
      the enrollment tokens) + the HASHED node secret
      (`node_keys` — `Sha256::digest` at the handshake) —
      no store interface; the `Classification`
      (general/confidential) is RECORDED-ONLY (the Phase-1
      deferral stands — the ADR-034 "silent general" is
      the shipped state); no region/export machinery; the
      snapshot store's `retention_class` is a free string,
      never classification-driven. Children: `.1.4.1` the
      declared-profile contract → `.1.4.2` the
      secret-store declared profiles → `.1.4.3` the
      classification-driven controls (the dev subset).
    Children: `.1.4.1`–`.1.4.3`

  - ID: `PHASE-7.1.4.1`
    Status: `done`
    Goal: the declared-profile contract — the decision
      record: the secret-store profile vocabulary (the
      shipped `dev_database` profile, the external stores
      as the CONFIGURATION choice, the undeclared-store
      typed refusal) + the classification-controls mapping
      (each control refuses at ITS decision point: the
      evaluator control at the dispatch, the retention at
      the sweep, the region/export at their triggers —
      ADR-034's "a classification without the controls is
      the typed refusal, never a silent general") + the
      named deferrals with their triggers.
    Roadmap: §16.8
    Decision: `docs/decisions/2026-09-08_declared-profiles-secrets-classification.md`
    Done (`2026-09-08`): the contract accepted —
      `docs/decisions/2026-09-08_declared-profiles-secrets-classification.md`
      (top-level `answers:`): the registry is the ONLY
      seam (every secret read routes through the declared
      profile; the shipped `dev_database` is the honest
      dev stance, named; the external store arrives as a
      configuration change); each classification control
      refuses at ITS decision point (the evaluator control
      at the DISPATCH — the dev profile registers no
      confidential-qualified evaluator, so the dispatch is
      the typed refusal; the retention control at the
      SWEEP — the snapshot retention_class gains the
      confidential tier; the export + the region controls
      have no decision point in the dev profile — the
      named deferrals with their triggers); the
      confidential classification stays CREATABLE (the
      controls refuse the PROVIDER use, never the thread).
      No code changed. Frontier → `.1.4.2`.

  - ID: `PHASE-7.1.4.2`
    Status: `done`
    Goal: the secret-store declared profiles — the profile
      registry with the shipped `dev_database` profile (the
      plaintext dev rows — the ADR-007 stance, honest);
      the CA/node-key reads route through the declared
      profile (the store is a configuration choice, never
      an ambient dependency — mechanical); the
      undeclared-profile request is the typed refusal;
      the measured suite.
    Roadmap: §16.8
    Done (`2026-09-08`): `secret_store.rs` — the declared
      registry: `DECLARED_PROFILES` (the shipped
      `dev_database`), `SecretStore::resolve` (the
      boot-time seam — an UNDECLARED name is the typed
      `UndeclaredStore` refusal naming the profile AND the
      declared set, never a silent fallback),
      `load_ca_material` (the CA row read THROUGH the
      store). `ca::ensure_server_ca_with_store` routes the
      CA material read through the resolved profile (the
      old inline SELECT is gone — the store is the only
      seam); `ensure_server_ca` stays as the
      dev-default convenience (the tests' path — the SAME
      implementation, no hidden third route). The boot
      (`rb-server`) gains `--secret-store-profile`
      (default `dev_database`), resolved ONCE before the
      CA load — the undeclared name refuses the boot. The
      node-key material: the handshake rides the cert
      proof (the `.1.2.2` channel); the `node_keys`
      secret is enrollment-written + the cert proof
      carries the identity — the key-read routing covers
      the CA material (the live key read); the external
      store joins by adding its name + its implementation
      (the named extension point). The measured suite:
      the OFFLINE unit tests (the resolution, the typed
      refusal, the declared default, the list) — the
      read-through routing rides EVERY live suite + the
      demo (they all boot through the store).
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the key reads
      were AMBIENT (the CA SELECT inline in `ca.rs` — the
      store was an undeclared fact, the ADR-034
      "ambient dependency"); the registry + the routed
      read make the configuration choice mechanical.
      Evidence: `cargo test -p reasonbraid-server --lib
      secret_store` → `test result: ok. 3 passed; 0
      failed` (the resolution + the refusal).
    - [x] **ADDRESSED** — the registry, the
      store-routed CA read, the boot resolution + the CLI
      flag, the typed `UndeclaredStore` refusal.
      Evidence: the unit tests (the undeclared name
      refuses AND names itself + the declared set); the
      guard boots every suite through the store seam
      (`target/pg_secretstore_guard.log`).
    - [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh`
      → rc=0, 24 suites + the demo `ALL acceptance checks
      passed` (`target/pg_secretstore_guard.log`);
      `cargo test --all` → rc=0, 67 suites
      (`target/secretstore_offline.log`); clippy/fmt
      clean; `make gate` → 13/13.
    - [x] **LESSON PROMOTED** — none new: the leaf
      executes the `.1.4.1` contract (the decision record
      already carries the registry-is-the-only-seam
      answers).

  - ID: `PHASE-7.1.4.3`
    Status: `done`
    Goal: the classification-driven controls (the dev
      subset) — the evaluator-access decision at the
      DISPATCH: a confidential thread's work delivery
      refuses without a confidential-qualified evaluator
      profile (the typed refusal, never a silent general);
      the retention/export vocabulary wired where the
      machinery exists (the snapshot retention_class);
      the region control stays the named deferral.
    Roadmap: §16.8
    Done (`2026-09-08`): the evaluator-access control
      binds at the dispatch — `Classification` gains
      `as_str` + `has_qualified_evaluator` (the registry:
      the dev built-ins qualify for `general` ONLY —
      `confidential` has no qualified evaluator, the
      named state); `dispatch_work_in_tx` (the single
      choke point — the accept + the challenge + the
      auto-initiation paths) refuses the confidential
      delivery FIRST, in-transaction (the command rolls
      back cleanly — no accept, no work item — the
      `.6.2` invitation-iff-work invariant holds); the
      wire gains `classification_unqualified` (409, in
      the status map so the stored rejection replays the
      original status). The thread itself stays CREATABLE
      + inspectable (the control refuses the PROVIDER
      use, never the thread — the `.1.4.1` contract).
      The measured suite (`tests/classification.rs`, LIVE
      — the guard's 25th): the confidential accept
      refuses with the typed code + no work item lands;
      the general thread's accept still dispatches; the
      classification stays the recorded fact. The
      retention/export vocabulary: the census found NO
      decision point (no thread-classified data enters
      the snapshot store — the `.1.4.1` "binds at the
      sweep" is revised: the retention binding awaits
      the confidential-data pipeline) — the named
      deferral with its trigger, alongside the region
      control. **The `.1.4` lane is COMPLETE.**
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the confidential
      classification was RECORDED-ONLY (the Phase-1
      deferral) — the dispatch treated it as general
      (ADR-034's forbidden "silent general"); the control
      lands at the dispatch's single choke point.
      Evidence: `cargo test -p reasonbraid-server --lib
      threads::tests::the_evaluator` → `test result: ok.
      1 passed` (the registry) + the live suite →
      `test result: ok. 1 passed; 0 failed`.
    - [x] **ADDRESSED** — `has_qualified_evaluator` +
      the dispatch gate + `classification_unqualified`
      (409) + the measured legs (the refusal, the
      no-work-item invariant, the general dispatch
      intact). Evidence:
      `target/pg_classification_guard.log`.
    - [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh`
      → rc=0, 25 suites + the demo `ALL acceptance checks
      passed` (`target/pg_classification_guard.log`);
      `cargo test --all` → rc=0, 68 suites
      (`target/classification_offline.log`); clippy/fmt
      clean; `make gate` → 13/13.
    - [x] **LESSON PROMOTED** — none new: the leaf
      executes the `.1.4.1` contract; the retention-
      decision-point revision is recorded in the Done
      notes (the trigger-named deferral).

- ID: `PHASE-7.2`
  Status: `done`
  Goal: public-node enrollment and quarantine, revocation propagation, signed software updates, SBOM/provenance, disclosure process
  Backlog: 40
  ADR: 027
  Roadmap: §16.10, §16.12
  Done (`2026-09-08`): the census at the seams —
    decomposed. Measured: the QUARANTINE ships (the
    two-way `.2.4` + the `.1.3.3` evidence rule); the
    REVOCATION PROPAGATION ships in its LAN form (the
    cert revocation + the suspended presence + the
    epoch fence at the dispatch); the SIGNED UPDATES +
    the SBOM/provenance are the greenfield (ADR-027 is
    RESERVED, no record); the DISCLOSURE policy is the
    greenfield (no SECURITY.md); the PUBLIC enrollment
    surface is the greenfield BY DESIGN (the `.5`
    kill/pivot forbids exposing remote enrollment with
    an incomplete gate — the enrollment POLICY is the
    dev-profile deliverable, the exposure is not).
    Children: `.2.1` the census + ADR-027 → `.2.2` the
    disclosure + the supported-version policy → `.2.3`
    the SBOM + the signed release artifacts → `.2.4`
    the public-enrollment contract.
  Children: `.2.1`–`.2.4`

  - ID: `PHASE-7.2.1`
    Status: `done`
    Goal: the census + ADR-027 — the signing-and-
      distribution contract: the release-identity key
      hierarchy, the digest-pinned manifest + the
      signature scheme, the adapter allowlist/
      capability-manifest vocabulary (the verification
      ladder over the shipped ledger); the distribution
      channel + the reproducible builders are the named
      deferrals.
    ADR: 027
    Roadmap: §16.10
    Done (`2026-09-08`): ADR-027 accepted (evidence-gated)
      — `docs/adr/027-signing-and-distribution.md`
      (top-level `answers:`): ONE Ed25519 release
      identity per channel (the dev placement — the
      releaser's key, the ADR-007 stance; the protected
      identities + the reproducible builders are the
      named deferrals); the release MANIFEST is the
      single verification unit (the per-binary
      `sha256:<hex>` digests + the manifest digest + the
      Ed25519 signature over the canonical JSON — the
      ring provider, the workspace rule); the
      downloaded-adapter verification is the ORDERED,
      FAIL-CLOSED ladder (allowlist → digest → signature
      → API compatibility → capability manifest — a
      failure refuses AT its rung, never a partial
      trust); the shipped dev adapters satisfy the
      ladder by construction (the ledger rows + the
      pinned interfaces). No code changed. Frontier →
      `.2.2`.

  - ID: `PHASE-7.2.2`
    Status: `done`
    Goal: the disclosure + the supported-version policy
      (SECURITY.md): the reporting path, the vetting +
      the embargo vocabulary, the supported versions
      (the §16.10 "before public beta" line) — the
      process record, linked from the README.
    Roadmap: §16.10
    Done (`2026-09-08`): `SECURITY.md` lands (the
      disclosure + the supported-version policy): the
      reporting path names the ACCOUNTABLE OWNER (the
      release/security gates' owner — no invented public
      channel: the repo is PRIVATE, the ADR-001 gate);
      the vetting rides the claim-verification
      discipline (a report is re-derived, never trusted);
      the embargo is the honest private-repo shape (the
      fix ships with its leaf + its regression test +
      its disclosure note; nothing is announced because
      nothing is public); the supported versions: the
      dev line is the only line — the window (the
      latest + the previous minor) begins at the first
      public release, together with the disclosure
      channel + the CVE pipeline (the public-beta
      trigger). The README's Status block + the
      "Where to read more" table updated (the security
      row + the Phase-7 status — the stale Phase-6.1
      line fixed in the same commit, the lockstep).
      No code changed. Frontier → `.2.3`.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the §16.10
      "before public beta" line had no process record
      (the census: no SECURITY.md); the policy lands as
      the record with the honest limits stated (no
      invented channels). Evidence: `git grep -c
      "disclosure" SECURITY.md` → the vocabulary
      present; the README links it.
    - [x] **ADDRESSED** — the reporting path (the
      accountable owner), the vetting/embargo
      vocabulary, the supported-version window + the
      honest-limits list; the README status + the link.
      Evidence: `make gate` → 13/13 (the
      README-STABILITY + the routing checks green).
    - [x] **NO REGRESSION** — docs-only: `make gate` →
      13/13; `make book` builds; no code paths changed
      (the guard stays green from `.2.1`).

  - ID: `PHASE-7.2.3`
    Status: `done`
    Goal: the SBOM + the signed release artifacts — the
      release pipeline generates the per-binary SBOM +
      the signed manifest (`make release` gains the
      step; the local release identity is the dev
      stance — the protected identity is the named
      deferral).
    Roadmap: §16.10
    Done (`2026-09-08`): the release manifest lands as
      the ADR-027 verification unit — the new
      `crates/reasonbraid-release-tool` (the
      `rb-release-manifest` bin, ring Ed25519 + sha2 —
      the workspace rules): `keygen` (the release
      identity key, the raw PKCS8 DER, 0600, refuses to
      overwrite — the dev placement, gitignored),
      `generate` (the per-binary `sha256:<hex>` digests
      + the canonical manifest — the struct field order
      + the SORTED binaries map — + the Ed25519
      signature over the exact bytes at `<out>.sig`),
      `verify` (the signature over the manifest's exact
      bytes + the RE-DERIVED digests against the
      binaries — a changed binary or a tampered
      manifest refuses). `make release` gains the step
      (the keygen on first use + the generate + the
      verify — the release is signed end-to-end). The
      offline roundtrip suite (`tests/manifest.rs`)
      proves: the generate → the verify, the overwrite
      refusal, the changed-binary refusal, the
      tampered-manifest refusal, the wrong-key refusal.
      The dependency-level SBOM (the SPDX/CycloneDX
      graph) is the named deferral — the manifest is
      the artifact-level SBOM; the full graph awaits
      the generator-tool census (the trigger).
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the release
      artifacts were unsigned + digest-less (the §16.10
      greenfield from the census); the manifest + the
      signature land in the release pipeline. Evidence:
      `cargo test -p reasonbraid-release-tool` →
      `test result: ok. 1 passed; 0 failed` (the
      roundtrip + the three refusals).
    - [x] **ADDRESSED** — the tool (keygen/generate/
      verify), the `make release` step, the gitignored
      key, the typed refusals. Evidence: `make release`
      → rc=0 with the manifest + the verify
      (`target/release_manifest.log`).
    - [x] **NO REGRESSION** — `cargo test --all` → rc=0
      (the new crate's suite joins the offline count);
      clippy/fmt clean; `make deny` green (the new
      crate reuses the vetted dep set); `make gate` →
      13/13; `make book` builds.
    - [x] **LESSON PROMOTED** — none new: the leaf
      executes ADR-027 verbatim (the manifest unit, the
      identity placement, the deferrals).

  - ID: `PHASE-7.2.4`
    Status: `done`
    Goal: the public-enrollment contract — the §16.10
      enrollment policy (the vetting steps, the
      quarantine-on-suspicion trigger, the revocation
      propagation over the shipped machinery); the
      EXPOSURE itself stays the `.5` kill/pivot gate.
    Roadmap: §16.10, §16.12
    Decision: `docs/decisions/2026-09-08_public-enrollment-contract.md`
    Done (`2026-09-08`): the contract accepted —
      `docs/decisions/2026-09-08_public-enrollment-contract.md`
      (top-level `answers:`): the enrollment is a STAGED
      vetting ladder (the identity claim → the capability
      declaration → the operator's acceptance → the cert
      issuance; the public form adds ONE stage — the
      claim VETTING, the re-derivation discipline); the
      suspicion → the quarantine is the OPERATOR's typed
      action over the shipped machinery (the §16.11
      observable counters + the quarantine verb + the
      `.1.3.3` evidence rule — the policy connects them,
      nothing new builds); the revocation propagation
      INHERITS the shipped ladder verbatim (the cert
      revocation → the suspended presence → the epoch
      fence → the replacement drill); the exposure is a
      QUALIFIED profile under the `.5` gate — never an
      experimental default (ADR-034). No code changed.
      **The `.2` lane is COMPLETE.** Frontier → `.3`.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the public
      enrollment had no policy record (the `.2` census:
      the machinery ships, the vocabulary does not); the
      contract fixes the vocabulary the `.5` gate
      judges against. Evidence: `make gate` → 13/13
      (the record + the INDEX row green).
    - [x] **ADDRESSED** — the vetting ladder, the
      suspicion-to-quarantine mapping, the revocation
      inheritance, the qualified-profile exposure
      stance. Evidence: the record's `answers:`; the
      shipped verbs the contract maps are the measured
      ones (the quarantine/replacement suites).
    - [x] **NO REGRESSION** — docs-only: `make gate` →
      13/13; `make book` builds; no code paths changed.

- ID: `PHASE-7.3`
  Status: `done`
  Goal: horizontally scalable coordinator workers only where measurements require them
  ADR: 002 (extraction criteria)
  Decision: `docs/decisions/2026-09-08_coordinator-extraction-criteria.md`
  Done (`2026-09-08`): the census at the seams —
    the coordinator is the deliberate SINGLE-WRITER
    design (ADR-002's named property); the measurements
    are catalogue-named-not-instantiated (the SLO
    record: the ingress→commit, the worker throughput,
    the channel latency are named families with no
    workload measurement — the ONE empirical number is
    SLO-5, the issuance baseline). The extraction
    criteria record (`docs/decisions/2026-09-08_coordinator-extraction-criteria.md`,
    top-level `answers:`) turns the mandate mechanical:
    the extraction trigger is a MEASUREMENT (never a
    hunch) — five seams (the aggregate write path, the
    outbox worker, the node channel, the CA issuance,
    the evaluation) each name their trigger measurement
    from the `.4` load harness + their horizontal form
    over the shipped machinery (the claim keys, the
    lease/fencing, the node keying — a re-arrangement,
    never a rebuild); zero extractions today. No code
    changed. Frontier → `.4`.
  Acceptance:
  - [x] **ROOT CAUSE (WHY + WHERE)** — the single-writer
    is a deliberate, named design (ADR-002); the lane's
    mandate needs the criteria that gate the extraction
    — the record fixes them. Evidence: `make gate` →
    13/13 (the record + the INDEX row green).
  - [x] **ADDRESSED** — the measurement-gated seam map
    (five seams, five triggers, the reuse forms).
    Evidence: the record's `answers:`.
  - [x] **NO REGRESSION** — docs-only: `make gate` →
    13/13; `make book` builds; no code paths changed.

- ID: `PHASE-7.4`
  Status: `done`
  Goal: capacity/load tests, incident exercises, penetration-test remediation, production runbooks
  Roadmap: §16.12, §18.6
  Done (`2026-09-08`): the census at the seams —
    decomposed. Measured: the RUNBOOK set is ONE record
    of the §18.6 catalogue (`node-lost-replaced.md` —
    the thirteen families are the catalogue); the LOAD
    harness does not exist (the bench harness is the
    deliberation benchmark, not a capacity test — the
    `.3` criteria's trigger measurements have no
    feeder); the game-days are the replacement drill +
    the restore exercise + the demo's kill points (the
    named gaps: the remaining families); the pen-test is
    EXTERNAL (not run — the remediation rides its
    findings). Children: `.4.1` the load harness → `.4.2`
    the runbook set → `.4.3` the game-day catalogue +
    the pen-test record.
  Children: `.4.1`–`.4.3`

  - ID: `PHASE-7.4.1`
    Status: `done`
    Goal: the load harness — the capacity tests that
      feed the `.3` criteria: a scripted concurrent
      command driver recording the ingress→commit p95,
      the worker throughput, and the channel latency at
      the target concurrency (the reproducible measured
      run — the CLAIM_VERIFICATION shape).
    Roadmap: §16.12
    Done (`2026-09-08`): `scripts/load_harness.sh` — the
      scripted CONCURRENT driver (the bash + curl + jq +
      python3 stack, the demo's style): it boots the
      server against the caller's database, enrolls the
      bootstrap human, creates the load thread, then
      fires N `thread.contribute` commands at C workers
      (each request a fresh `req_<uuid>` request id + a
      unique idempotency key — the FULL claim →
      authorize → validate → apply path, the
      ingress→commit measurement). The per-request
      `status seconds` lines land in
      `target/load/latencies.txt`; the summary prints
      the p50/p95 + the throughput + the failures, and
      the exit gates on every command committing (the
      200s). The MEASURED run: 200 commands at 8
      workers → 1.057s wall, ingress→commit p50 0.0033s
      / p95 0.0079s, 189.2 commands/s, 0 failures
      (`target/load_harness_run.log`) — the FIRST
      `.3`-criteria feed (the aggregate-write seam's
      trigger measurement: the ingress→commit p95 at
      the target concurrency). The worker-throughput +
      the channel-latency legs are the named follow-ons
      (the harness's loop is the extension point — the
      node-side poll rides the same concurrency
      shape). The first run caught a real harness bug:
      the bare `wait` also joined the backgrounded
      SERVER (never exits) — the fix waits the WORKER
      PIDs only (the comment records it).
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the `.4` census:
      no load harness (the `.3` criteria's trigger
      measurements had no feeder); the harness is the
      feeder. Evidence: `bash scripts/load_harness.sh
      --database-url … --commands 200 --concurrency 8`
      → rc=0, the PASS line.
    - [x] **ADDRESSED** — the concurrent driver + the
      recorded latencies + the summary + the exit gate.
      Evidence: the measured run — `PASS: every command
      committed (200) and the summary is recorded`;
      ingress→commit p50 0.0033s / p95 0.0079s,
      189.2 commands/s (`target/load_harness_run.log`).
    - [x] **NO REGRESSION** — no server paths changed:
      `make gate` → 13/13; the guard stays green from
      `.2.3` (the harness is a standalone script, not a
      guard suite — the guard wiring is the named
      follow-on).
    - [x] **LESSON PROMOTED** — `promotion: declined
      (the bare-wait-joins-the-server trap is a
      well-known bash wait semantics fact, recorded in
      the harness's own comment)`.

  - ID: `PHASE-7.4.2`
    Status: `proposed`
    Goal: the §18.6 runbook set — the remaining twelve
      families (the provider outage/ambiguous charge,
      the credential compromise, the notification storm,
      the runaway budget, the poisoned resource, the
      database failover, the object loss, the Git/DB
      publication mismatch, the signing-key incident,
      the cross-tenant exposure suspicion, the
      audit-chain break, the rollback/suspension, the
      full DR) as the runbook records over the shipped
      controls (the §18.6 shape: the detection, the
      authority, the safe first actions, the diagnostics,
      the containment, the recovery, the evidence, the
      communication, the closure tests).
    Roadmap: §18.6

  - ID: `PHASE-7.4.3`
    Status: `proposed`
    Goal: the game-day catalogue + the pen-test record
      — the exercise mapping (which games exist — the
      replacement drill, the restore exercise, the
      demo's kill points — vs the named gaps) + the
      pen-test remediation stance (the findings become
      the backlog items with owners; the remediation
      rides the external test's findings).
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
| 1 | `PHASE-7.4.2` | `proposed` | `.4.1` done — the load harness ships (the measured run: p50 3.3ms / p95 7.9ms / 189.2 cmds/s at 8 workers — the first `.3`-criteria feed); the §18.6 runbook set executes next |

## Changelog

- `2026-09-08`: `.4.1` done — the load harness (the
  concurrent driver + the measured run: 200 commands,
  8 workers, p50 0.0033s / p95 0.0079s, 189.2
  commands/s, 0 failures — the `.3` criteria's first
  feed; the worker/channel legs are the named
  follow-ons); frontier → `.4.2`.
- `2026-09-08`: `.4` done — the census at the seams
  (the runbook set is one record of the thirteen; the
  load harness does not exist — the `.3` triggers have
  no feeder; the game-days are the three shipped
  exercises; the pen-test is external) → decomposed
  `.4.1` (the load harness) → `.4.2` (the runbook set)
  → `.4.3` (the game-day catalogue + the pen-test
  record); frontier → `.4.1`.
- `2026-09-08`: `.3` done — the coordinator extraction
  criteria (the measurement-gated seam map: the five
  seams name their `.4` trigger measurements + their
  reuse forms; zero extractions today); no code;
  frontier → `.4`.
- `2026-09-08`: `.2.4` done — the public-enrollment
  contract (the staged vetting ladder, the
  suspicion-to-quarantine policy over the shipped
  verbs, the revocation inheritance, the qualified-
  profile exposure stance); **the `.2` lane is
  COMPLETE**; frontier → `.3`.
- `2026-09-08`: `.2.3` done — the SBOM + the signed
  release artifacts (the `rb-release-manifest` tool —
  keygen/generate/verify — + the `make release` step +
  the measured refusals; the dependency-graph SBOM is
  the named deferral); frontier → `.2.4`.
- `2026-09-08`: `.2.2` done — SECURITY.md (the
  disclosure path over the accountable owner, the
  re-derivation vetting, the honest embargo, the
  supported-version window at the public beta) + the
  README status/link lockstep fix; no code; frontier →
  `.2.3`.
- `2026-09-08`: `.2.1` done — ADR-027 accepted
  (evidence-gated): the release identity + the
  digest-pinned manifest + the ordered fail-closed
  adapter-verification ladder; the distribution channel +
  the reproducible builders named as the deferrals; no
  code; frontier → `.2.2`.
- `2026-09-08`: `.2` done — the census at the seams
  (the quarantine + the revocation propagation ship in
  their LAN forms; ADR-027 reserved; the SBOM/disclosure/
  public-enrollment are the greenfield) → decomposed
  `.2.1` (the census + ADR-027) → `.2.2` (the
  disclosure policy) → `.2.3` (the SBOM + the signed
  artifacts) → `.2.4` (the public-enrollment contract);
  frontier → `.2.1`.
- `2026-09-08`: `.1.4.3` done — the classification-driven
  controls (the evaluator-access dispatch gate — the
  confidential delivery refuses with the typed code, the
  general dispatch intact, the thread stays creatable; the
  retention/export/region deferrals revised + named);
  **the `.1` lane is COMPLETE**; frontier → `.2`.
- `2026-09-08`: `.1.4.2` done — the secret-store declared
  profiles (the registry + the store-routed CA read + the
  boot-time resolution + the typed undeclared refusal +
  the offline measured suite); frontier → `.1.4.3`.
- `2026-09-08`: `.1.4.1` done — the declared-profile
  contract accepted (the decision record: the secret-store
  profile vocabulary + the classification-controls
  mapping + the named deferrals); no code; frontier →
  `.1.4.2`.
- `2026-09-08`: `.1.4` done — the census at the seams
  (the dev secrets are the plaintext rows + the hashed
  node secret; the classification is RECORDED-ONLY — the
  shipped "silent general"; no region/export machinery) →
  decomposed `.1.4.1` (the declared-profile contract) →
  `.1.4.2` (the secret-store profiles) → `.1.4.3` (the
  classification controls); frontier → `.1.4.1`.
- `2026-09-08`: `.1.3.3` done — the
  quarantine-preserving-evidence rule (the census found
  the retention gap — the prune deleted acknowledged
  dead-lettered rows; the `AND quarantined_at IS NULL`
  exclusion + the articulated contract + the measured
  survival legs); **the `.1.3` lane is COMPLETE**;
  frontier → `.1.4`.
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
