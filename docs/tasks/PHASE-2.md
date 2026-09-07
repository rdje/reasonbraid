# PHASE-2: delivery, identity, and recovery hardening

## Metadata

- Tree ID: `PHASE-2`
- Status: `active`
- Roadmap lane: Phase 2 (`ROADMAP.md` §20.4); Trust track
- Created: `2026-09-05`
- Estimate: 12–20 engineer-weeks
- Depends on: Phase 1 slice
- Exit: authority non-escalation; full restore and node replacement; no known path that reports an ambiguous provider attempt as safely retryable

## Goal

Harden identity, delivery, recovery, and observability so a later Internet
slice can reuse the same control plane without rewriting it.

## Task Tree

- ID: `PHASE-2.1`
  Status: `active`
  Goal: workload certificate lifecycle, scoped grants, delegated authority context, revocation, cached-decision rules
  Backlog: 11
  ADR: 007, 008, 009
  Note: gap census (`2026-09-07`, on pickup — `grep -rn` over the authority
    engine + api.rs + the node + migrations + Cargo.tomls + the ADR index):
    - EXISTS: one-time enrollment tokens bound to tenant+node+host-claim+
      nonce+expiry, single-use (0008 — §16.2's offline-bootstrap shape);
      scoped grants (actions + `TargetSelector` + risk ceiling + spend
      limits + delegable + validity window + status); the enrollment
      boundary as the ceiling; the authenticated HMAC channel
      (`CHANNEL_VERSION` 2).
    - GAP 1 — no workload certificate machinery at all: `grep -rn
      'rustls|rcgen|x509|Certificate' Cargo.toml crates/*/Cargo.toml
      crates/*/src` → zero matches; the channel comment defers mTLS to
      ADR-006/007. ADR-006/007/008/009 are UNOPENED (`docs/adr/INDEX.md`
      has 001/002/004 only).
    - GAP 2 — no revocation WRITE path: `GrantStatus::Revoked` +
      `BoundaryStatus::Revoked` exist as types + tests only (`grep -rn
      'revoke' crates/reasonbraid-server/src/api.rs
      crates/reasonbraid-cli/src/main.rs` → no verb, no route).
    - GAP 3 — no delegated authority context: the boundary refuses
      delegation (`max_delegation_depth`=1, "chains are Phase 2" comment);
      no `AuthorityContext` (§16.3).
    - GAP 4 — no cached-decision semantics: fresh evaluation everywhere
      (`grep -rn 'cache' authority.rs api.rs` → no decision cache).
    - GAP 5 — incarnation/run rows have NO writer (inherited from the
      Phase-1 gate record deferral #4 — `grep -rn 'INSERT INTO
      incarnations' crates/` → no matches; the 0007 hierarchy is
      schema-only).
    - GAP 6 — the dependency ledger has no identity-issuer row (the §16.2
      candidates SPIFFE/SPIRE, step-ca, rcgen are unrecorded).
  Children: `.1.1`–`.1.6` (decomposed `2026-09-07` at the census seams:
    ADRs+spike → cert lifecycle → revocation → delegation → caching →
    incarnations).

  - ID: `PHASE-2.1.1`
    Status: `done`
    Goal: the ADR-006/007 spike + records — reconcile ADR-006 (node
      transport/reconnect — the existing channel decisions
      `docs/decisions/2026-09-06_node-channel*.md` are the evidence; the
      ADR is written as accepted-with-evidence) and choose the workload
      identity issuance model for ADR-007 with an operational spike: a
      small on-volume experiment crate (`crates/reasonbraid-cert-spike`,
      dev-only, not wired into the product) measuring the candidates —
      project-local CA (rcgen: issue a CA + a short-lived leaf with a SAN,
      validate the chain, measure issuance latency, prove rotation +
      revocation-by-expiry/status refusal) vs step-ca (external service
      cost) vs SPIFFE/SPIRE (operational weight) — against the Trusted LAN
      profile (§16.2: TLS 1.3 transport, short-lived rotated workload
      certs, the cert identity rides the durable node id, not replacing
      it). `cargo deny` stays green over the new deps. Deliverables: the
      spike evidence on-volume + ADR-006 + ADR-007 + the dependency-ledger
      identity row.
    Backlog: 11 (the identity sliver)
    ADR: 006, 007
    Acceptance: the spike's numbers are recorded (issuance latency,
      chain-validation proof, rotation/revocation refusals); ADR-007 names
      the chosen model + the honest limits (no OCSP/CRL unless the spike
      proves the need — short expiry + server-side status is the default
      candidate); ADR-006 is accepted-with-evidence; the ledger row is
      filled; the guard set stays green.
    Done (`2026-09-07`): the spike passed 6/6 verdicts on a real TLS 1.3
      handshake (issuance p50 63 µs / p95 69 µs, N=200); ADR-006 +
      ADR-007 accepted (evidence-gated), the ledger identity row filled;
      the acceptance checklist below records the evidence — frontier →
      `.1.2`.

  - ID: `PHASE-2.1.2`
    Status: `active`
    Goal: the certificate lifecycle core — the server issues a short-lived
      X.509 workload cert at enrollment (project-local CA key held by the
      server, per the `.1.1` model), the node stores its key + cert
      beside its journal, rotation is automatic (re-issue on expiry via
      the authenticated channel), and the channel's handshake upgrades to
      the cert proof (CHANNEL_VERSION 3 — the HMAC dev path retires for
      enrolled nodes; the demo moves to the new contract and stays green).
      The cert identity RIDES the durable node id (§16.2: reimaging does
      not inherit identity).
    Backlog: 11
    ADR: 007 (the `.1.1` choice, implemented)
    Acceptance: enroll issues a cert exactly once (replay refused); the
      channel refuses an expired/unknown cert with a typed error; rotation
      heals the channel without a re-enroll; the two-host demo passes on
      the new contract; the existing channel suites move and stay green.
    Note: decomposed further (`2026-09-07`, the `.1.2.1`-first precedent —
      a coherent interim exists: the cert is issued and stored while the
      HMAC channel stays live until the v3 swap):
    Children: `.1.2.1`–`.1.2.2`. **`.1.2` is COMPLETE** — the certificate
      lifecycle rides the channel (v3 proof + rotation), backlog 11's cert
      sliver is closed.

  - ID: `PHASE-2.1.2.1`
    Status: `done`
    Goal: cert issuance at enrollment — migration 0011 (`server_ca` +
      `node_certificates`), the server generates/loads its CA at startup
      and persists it (the demo kills and restarts the server — the CA
      must survive), `POST /v1/nodes/enroll` issues a short-lived leaf
      (CN = the node id, SAN = the token's host claim — the cert rides
      the durable identity) and returns it WITH the (server-generated,
      dev-escrowed) key; the node persists `cert.der`/`key.der` beside its
      journal; the one-time token path keeps the replay refusal. The HMAC
      channel is UNTOUCHED (coherent interim — the cert exists, unused,
      until `.1.2.2`).
    Backlog: 11
    Acceptance: the enroll response carries the cert + key; the
      `node_certificates` row lands; the CA row survives a server
      rebuild/restart (same CA key — previously issued certs still chain);
      replay is still refused; the existing suites + demo stay green
      (the demo stores the files but does not use them yet).
    Done (`2026-09-07`): migration 0011 + `ca.rs` (the persisted CA:
      generated on first boot, loaded thereafter — the rebuild test
      proves the same key/cert) + the enroll response carries the leaf +
      dev-escrowed key + fingerprint; `rb-node` persists `cert.der`/
      `key.der` beside the journal; the HMAC channel untouched (the demo
      passes with the files stored, unused); the acceptance checklist
      below records the evidence — frontier → `.1.2.2`.

  - ID: `PHASE-2.1.2.2`
    Status: `done`
    Goal: the channel v3 cert-proof handshake + rotation —
      `CHANNEL_VERSION` 3: the handshake body carries the cert DER + a
      signature over the SAME canonical coverage JSON (the private key's
      proof replaces the HMAC dev secret), and the server verifies
      chain-to-the-CA + validity window + fingerprint ∈
      `node_certificates` + the signature before ANY ledger read; the
      lease/fencing machinery rides it unchanged. Rotation: a
      cert-proof-authenticated rotate endpoint issues a fresh key + cert
      (additive fingerprint), and the node rotates at ≤50% remaining
      lifetime — no re-enroll. The channel suites (17) move to the new
      contract; the two-host demo enrolls → stores → handshakes with the
      cert (its psql fencing-token oracle stays — fencing is unchanged);
      the book's node-channel + two-host-demo chapters carry the new
      auth; the deployment chapter's honest limits stay true (transport
      TLS is not claimed — the proof rides the HTTP/1 channel per
      ADR-006/007).
    Backlog: 11
    Acceptance: a handshake with a foreign/expired/unregistered cert is a
      typed 401; a rotated cert heals the channel without re-enrollment;
      the demo passes on v3; all channel suites green; the book names the
      new contract.
    Done (`2026-09-07`): CHANNEL_VERSION 3 landed — the handshake signs the
      canonical coverage with the workload certificate's key, the server
      verifies chain-to-CA + validity + the node-id fingerprint + the
      signature BEFORE any ledger read; the rotate endpoint issues a fresh
      key + cert (additive); the node rotates at ≤50% lifetime; the 19
      channel tests + the demo (31 checks) pass on v3; the acceptance
      checklist below records the evidence — frontier → `.1.3`.
    Note: the verification leg uncovered a real interop fact — ring's
      `UnparsedPublicKey` refuses rcgen's well-formed SPKI DER and accepts
      the bare EC point (the path webpki uses internally); the proof
      verifies against the extracted point. Measured by the temporary
      ladder probe (chain / SPKI / self-SPKI / ring-only control /
      digest variants); recorded in
      `docs/decisions/2026-09-07_cert-proof-verification.md`.

  - ID: `PHASE-2.1.3`
    Status: `active`
    Goal: revocation surfaces — `node revoke`/cert status (the server
      refuses a revoked cert at the handshake; a revoked node goes
      `suspended` with visible presence), `grant revoke` + `boundary
      revoke` verbs (the `Revoked` statuses get their write paths, CLI +
      audited), and revocation FRESHNESS propagates (a revoked grant/cert
      refuses within the decision path, not eventually).
    Backlog: 11
    Acceptance: a revoked cert/grant/boundary is refused at the next
      boundary crossing with an audit row; revocation is observable
      through the inspection surfaces; no existing suite regresses.
    Note: gap census (`2026-09-07`, on pickup): the REFUSAL paths already
      exist — the handshake ladder checks `revoked_at IS NOT NULL`
      (`.1.2.2`), and the grant/boundary evaluation filters
      `status = 'active'` (`grep -n "status = 'active'"
      crates/reasonbraid-server/src/authority.rs` → lines 407/538) — what
      is MISSING is every write path (`grep -n 'revoke'
      crates/reasonbraid-server/src/api.rs
      crates/reasonbraid-cli/src/main.rs` → no verbs) + the suspended
      presence state (the 0009 view derives online/offline only).
    Children: `.1.3.1`–`.1.3.2` (decomposed `2026-09-07` at the
      cert-vs-grant seam — two independent contracts). **`.1.3` is
      COMPLETE** — the revocation surfaces are live (cert + grant +
      boundary), the refusals ride the existing ladders.

  - ID: `PHASE-2.1.3.1`
    Status: `done`
    Goal: node/cert revocation — `POST /v1/nodes/revoke` (the
      tenant_admin surface, the issue-token pattern): sets `revoked_at`
      on the node's ACTIVE certificates (zero rows = 404; the refusal is
      audited by the authorization record); the handshake ladder ALREADY
      refuses revoked leaves (the `.1.2.2` row check — the test proves the
      next handshake is 401); migration 0012 extends the `node_presence`
      view with `suspended` (a node with a revoked certificate reads
      suspended, whatever its lease); `rb node revoke --node <id>`
      [--reason] --as/--tenant; the demo gains the revoke beat (node B,
      after its thread closes — presence `suspended:true`). The book's
      node-channel + cli chapters carry the surface.
    Backlog: 11
    Done (`2026-09-07`): `POST /v1/nodes/revoke` (tenant_admin-audited)
      sets `revoked_at` on the node's active certificates; migration 0012
      extends the presence view with `suspended` (Postgres view-replacement
      appends columns only — the column sits LAST); `rb node revoke`; the
      suite proves the next handshake is 401 + presence suspended + the
      typed refusals (404 unknown, 403 non-admin + audit, 409 re-revoke);
      the demo gains the beat (32 checks) — the acceptance checklist below
      records the evidence — frontier → `.1.3.2`. (Follow-up `REASONBRAID-PHASE2-0009`:
      the bootstrap assert now prints the enroll body — the diagnostic that
      found the seed-tenant boundary gap.)
    Acceptance: revoking a node refuses its next handshake (401) and
      flips presence to `suspended`; an unknown node is 404; a non-admin
      caller is the typed 403 + audit row; the demo passes with the new
      beat; no regression.

  - ID: `PHASE-2.1.3.2`
    Status: `done`
    Goal: grant/boundary revocation — `POST
      /v1/admin/grants/{grant_id}/revoke` + `POST
      /v1/admin/boundaries/{boundary_id}/revoke` (tenant_admin-audited):
      the `Revoked` statuses get their write paths; the evaluation's
      existing `status = 'active'` filters refuse them at the next
      decision (the test proves a revoked grant loses its authority while
      the tenant's other grants keep working); `rb grant revoke` + `rb
      boundary revoke`; the inspection surfaces show the status.
    Backlog: 11
    Done (`2026-09-07`): the `Revoked` statuses got their write paths —
      `POST /v1/admin/grants/{id}/revoke` + `POST
      /v1/admin/boundaries/{id}/revoke` (tenant_admin-audited, typed 404/
      409 refusals) + `rb grant revoke` / `rb boundary revoke` + the
      admin inspection lists (`GET /v1/admin/grants|boundaries`, `rb
      inspect grants|boundaries`); the tests prove the next authorization
      refuses (403 + audit) while other grants work, and the boundary
      revocation freezes the tenant's writes while the reads stay open
      (the freeze carve-out — recorded in
      `docs/decisions/2026-09-07_boundary-revocation-freeze.md`); the
      acceptance checklist below records the evidence — **`.1.3` is
      COMPLETE** — frontier → `.1.4`.
    Acceptance: a revoked grant is refused at the next authorization
      (with the audit row) while other grants evaluate; a revoked boundary
      refuses its ceiling checks; unknown ids are 404; non-admin callers
      are 403; no regression.

  - ID: `PHASE-2.1.4`
    Status: `active`
    Goal: the delegated authority context — `AuthorityContext` (actor,
      subject, tenant, scopes, selectors, purposes, constraints, issuer
      chain, validity) rides the command envelope + the authorization
      record; the §16.3 invariants hold mechanically (a delegate cannot
      widen a grant/duration/tenant/target/cost/approval; forwarding
      preserves the chain; both caller and subject permission are
      evaluated; revocation checks at irreversible boundaries). ADR-009
      records the representation (attenuated capability tokens vs
      chain-in-envelope — the spike decides).
    Backlog: 11
    ADR: 009
    Acceptance: a delegated request with a narrower subset succeeds; a
      widening attempt is a typed refusal naming the invariant; the
      decision record carries the chain; race/revocation tests green.
    Note: gap census (`2026-09-07`, on pickup): the plumbing is
      PRE-SHAPED — `CommandAuthz.delegate_subject` exists (always `None`)
      and the authorization_records INSERT already writes the subject
      split when delegation applies (`grep -n "delegate_subject"
      crates/reasonbraid-server/src/authority.rs` → lines 45/579); the
      envelope carries no delegation field and the dual evaluation +
      the widening check do not exist.
    Children: `.1.4.1`–`.1.4.2` (decomposed `2026-09-07` at the
      ADR-vs-implementation seam). **`.1.4` is COMPLETE** — the delegated
      authority context rides the envelope + the dual evaluation (ADR-009,
      the §16.3 invariants held mechanically).

  - ID: `PHASE-2.1.4.1`
    Status: `done`
    Goal: ADR-009 + the representation spike — the decision between
      chain-in-envelope (the request carries the structured delegation)
      and attenuated capability tokens (a separately issued credential)
      for the dev profile, decided by an executable prototype: a pure
      `DelegationConstraints` type + the subset check (the widening
      invariant) with offline tests in the core crate, plus the measured
      wire-size comparison (the envelope delta vs a token blob at depth
      1–3). ADR-009 records the choice + the honest limits (the dev
      profile's chain rides the existing grants for expiry/revocation —
      the `.1.3` filters apply unchanged).
    Backlog: 11
    ADR: 009
    Done (`2026-09-07`): ADR-009 accepted — chain-in-envelope for the dev
      profile (the plumbing was pre-shaped; expiry/revocation ride the
      `.1.3` grant filters; no token lifecycle). The spike landed the
      pure `DelegationConstraints` + `delegation_scope_is_subset` in the
      core crate with the offline tests (narrower/equal/empty pass;
      widening refused per-dimension; the wire-size leg asserts the
      envelope form beats a token blob) — `cargo test -p reasonbraid-core`
      → `test result: ok. 39 passed`. Wire note for `.1.4.2`:
      `GrantSubject` is a serde TAGGED newtype — its wire form is a plain
      string, so the envelope's `authority_context` must carry the subject
      as a string field, not the enum (the size probe proved the
      serialization refusal). The acceptance checklist below records the
      evidence — frontier → `.1.4.2`.
    Acceptance: the subset prototype's tests are green (narrower passes,
      widening refused per-dimension); the ADR names the representation
      + the measurement; the ledger row for the chosen shape is filled
      or explicitly declined with the reason.

  - ID: `PHASE-2.1.4.2`
    Status: `done`
    Goal: the implementation — the envelope gains the optional
      `authority_context` (the `.1.4.1` choice: `on_behalf_of` +
      `purpose` + `scope` constraints, deny-unknown); the authorize path
      evaluates BOTH the caller's and the subject's grants (the dual
      check) and applies the subset rule (a widening request is a typed
      403 naming the invariant); the authorization record carries the
      chain; the CLI gains `--on-behalf-of` (+ `--purpose`); the tests
      prove narrower-succeeds / widening-refused / revocation-freshness
      (a revoked subject grant refuses the delegated request at the next
      decision).
    Backlog: 11
    Done (`2026-09-07`): the envelope gained the optional
      `authority_context` (the ADR-009 shape — the subject rides a STRING
      field); the authorize path runs the DUAL evaluation (the caller's
      own grant AND the subject's grant, the latter as the authority
      source the record + digest bind) + the scope ladder (the request's
      target within the requested scope, the scope within the subject's
      grant selector — a widening request is a typed 403 naming the
      invariant); the CLI gained `--on-behalf-of`/`--purpose` on the
      thread verbs (the scope = the command's own target); the acceptance
      test proves narrower-succeeds (the audit carries the subject),
      widening-refused, the caller check, and the `.1.3` revocation
      freshness; the audit test moved to the dual semantics; the
      acceptance checklist below records the evidence — **`.1.4` is
      COMPLETE** — frontier → `.1.5`.
    Acceptance: a delegated request within the subject's grant succeeds
      and audits the chain; a widening attempt is a typed refusal; a
      revoked subject grant refuses the delegation at the next decision;
      no regression.

  - ID: `PHASE-2.1.5`
    Status: `active`
    Goal: cached-decision semantics — ADR-008 (which decisions are
      cacheable, the freshness/expiry rule, the revocation-epoch
      invalidation, the fail-closed rule when the authority store is
      unreachable) + the node-side cache honoring it.
    Backlog: 11
    ADR: 008
    Acceptance: a cached allow expires/refreshes on the declared rule; a
      revocation invalidates the cache (measured); an unreachable
      authority store fails closed for irreversible writes; the suites
      stay green.
    Note: gap census (`2026-09-07`, on pickup): the ROADMAP rule exists
      (§16.4 — cache only explicitly cacheable decisions, honor expiry +
      revocation freshness, per-action-class fail-open/fail-closed) but
      NO machinery: `grep -rn 'cache'
      crates/reasonbraid-node/src/ crates/reasonbraid-server/src/` → 0
      matches (fresh in-tx evaluation everywhere); no revocation epoch
      exists (`grep -rn 'epoch'` over the node/server/migrations → the
      `.1.3` write paths bump none); the poll payload carries no decision
      metadata (`ReplayCommand` = cursor/id/tenant/thread/payload —
      channel.rs:132-138) — but the plumbing is PRE-SHAPED: the node
      journal's `authz_ref` column exists with every writer binding
      `None` (journal.rs:144/437/444; writers at worker.rs:149 +
      node.rs:208) and `authorization_records` already holds
      `policy_digest`/`policy_version`/`decided_at` (migration 0004; the
      version is the hardcoded `"dev-authz-1"`).
    Children: `.1.5.1`–`.1.5.2` (decomposed `2026-09-07` at the
      ADR-vs-implementation seam — the `.1.4` precedent). **`.1.5` is
      COMPLETE** — ADR-008 accepted + the delivery-carried decision +
      the tenant epoch + the node-side dispatch gate (a revocation refuses
      the next dispatch without a re-ask, measured).

  - ID: `PHASE-2.1.5.1`
    Status: `done`
    Goal: ADR-008 + the semantics spike — which decisions are cacheable
      (the dev profile's answer: the server's ADMISSION decision rides
      the delivery and the node caches ONLY that — §11.1's minimum
      state), the freshness/expiry rule, the revocation-epoch
      invalidation (a per-tenant epoch counter bumped by every revocation
      write; a cached decision records the epoch it was decided under, a
      bump invalidates it), and the per-action-class fail-closed rule
      (irreversible writes fail closed when the authority store is
      unreachable; the dev-profile classes declared). The spike is pure
      core-crate types + offline tests (the `.1.4.1` precedent):
      `CachedDecision` (decision, decided_at, expires_at,
      revocation_epoch, policy_digest) with the freshness/expiry/
      invalidation checks + the fail-closed classifier. The engine half
      of ADR-008 is recorded accepted-with-evidence (the Phase-0 `.5.1`
      in-tx evaluator IS the chosen engine — no OPA/Cedar experiment for
      the dev profile; ADR-006's precedent).
    Backlog: 11
    ADR: 008
    Done (`2026-09-07`): ADR-008 accepted — the shipped in-tx evaluator
      stays (accepted with evidence; the OPA/Cedar comparison parks
      behind a measured trigger) and the node caches ONLY the admission
      decisions riding its delivery (the §17.1 store-authority table:
      the journal can never locally re-evaluate a grant). The spike
      landed the pure `CachedDecision` + `CacheVerdict` + the
      `ActionClass` fail table in the core crate with the five offline
      tests (fresh + epoch-current allow dispatches; expired → stale;
      an epoch bump invalidates a fresh entry; a deny is never widened;
      irreversible/admin writes fail closed, reads fail open) —
      `cargo test -p reasonbraid-core` → `test result: ok. 44 passed`.
      Verification found a REAL drift: the checked-in
      `command-envelope.schema.json` golden predated `.1.4.2`'s envelope
      change (its NO REGRESSION set ran the live suites only — the
      core crate's own offline suite, home of the golden-drift test, was
      not re-run). The golden is regenerated here (`write_schema_goldens`)
      and the core suite is green again; the acceptance checklist below
      records the evidence — frontier → `.1.5.2`.
    Acceptance: the spike's offline tests are green (a fresh, unexpired,
      epoch-current cached allow passes; an expired or epoch-stale one
      fails; the irreversible classes fail closed); ADR-008 names the
      cacheable decisions + the declared rules + the honest limits; no
      product code changes (pure core types consumed by `.1.5.2`).

  - ID: `PHASE-2.1.5.2`
    Status: `done`
    Goal: the implementation — the delivery carries the decision: the
      poll payload's commands gain the admission-decision metadata
      (authz_ref + policy_digest + decided_at + the epoch at decision
      time) so a dispatched command names its admitting record; the
      server gains the tenant revocation epoch (migration 0013) bumped in
      the same transaction as every `.1.3` revocation write; the node
      journals the cached decision (the pre-shaped `authz_ref` finally
      gains a value) and the dispatch boundary honors the `.1.5.1` rules
      (a fresh, epoch-current cached allow dispatches; an expired or
      epoch-stale one refuses the irreversible write — fail closed — and
      refreshes at the next poll); the tests prove expiry/refresh,
      revocation invalidation (measured), fail-closed.
    Backlog: 11
    ADR: 008
    Done (`2026-09-07`): the cached-decision machinery landed —
      migration 0013 (the per-tenant `revocation_epoch` + the inbox's
      decision columns), the three `.1.3` revocation writes bump the epoch
      IN their transaction, the delivery carries the admission decision
      (authz_ref + digest + decided_at + epoch-at-decision; the handshake/
      poll responses carry the CURRENT epoch; CHANNEL_VERSION 4), the
      journal stores the cached decision (migration 0003; the pre-shaped
      `authz_ref` finally gains a value), and the worker's dispatch
      boundary evaluates it (`.1.5.1` rules: fresh + epoch-current
      dispatches; expired/epoch-stale/denied/absent refuses, journaled
      `failed_before_dispatch` with the reason — the adapter is never
      invoked). THE measured acceptance leg: the live test drives the REAL
      node worker — the fresh allow completes (the contribution lands),
      the grant revocation bumps the epoch 0→1, and the next dispatch (of
      work decided under epoch 0) is refused WITHOUT a re-ask (the
      staleness is in the journaled evidence; no second contribution). The
      five offline gate tests (expired/epoch-bumped/absent/fresh-allow/
      budget-still-gates) + the live test passed; the full guard green
      (12 suites + e2e + demo 32/32, `target/pg152b_guard.log`). The
      acceptance checklist below records the evidence — **`.1.5` is
      COMPLETE** — frontier → `.1.6`.
    Acceptance: a cached allow expires/refreshes on the declared rule; a
      revocation invalidates the cache (measured — the next dispatch
      refuses without a re-ask); an unreachable authority store fails
      closed for irreversible writes; the suites stay green.

  - ID: `PHASE-2.1.6`
    Status: `proposed`
    Goal: incarnation/run writers — enroll records the incarnation row
      (harness + model/provider facts known at node start; the 0007
      hierarchy gets its writers), dispatches record run rows linked to
      the attempt; the Phase-1 gate-record deferral #4 closes.
    Backlog: —
    Acceptance: an enrolled node's incarnation row exists and is
      inspectable; a dispatch links its run + attempt; re-enroll/rotation
      do not duplicate incarnations; no regression.

- ID: `PHASE-2.2`
  Status: `proposed`
  Goal: production-grade leases/fencing, retry policy, dead-letter/quarantine/replay
  ADR: 005 (transport choice if Phase 0 left it open)

- ID: `PHASE-2.3`
  Status: `proposed`
  Goal: provider-attempt state machine, usage reconciliation, spend circuit breakers, ambiguous-outcome workflows
  Backlog: 23, 25
  ADR: 012, 013

- ID: `PHASE-2.4`
  Status: `proposed`
  Goal: backup, PITR, object/Git inventory groundwork, migrations, upgrade/rollback testing
  Roadmap: §17.5–17.6

- ID: `PHASE-2.5`
  Status: `proposed`
  Goal: OpenTelemetry, operator dashboards, initial SLO baselines, game days
  Backlog: —
  ADR: 023
  Roadmap: §18

- ID: `PHASE-2.6`
  Status: `proposed`
  Goal: adapter conformance kit and permanent failure fixture corpus
  Roadmap: §19.4

- ID: `PHASE-2.7`
  Status: `proposed`
  Goal: exit — non-escalation properties; restore + node replacement; no false safe-retry of unknown attempts
  Gate: feeds G6–G7; subtraction record required
  ADR: 022 (audit hash-chain groundwork)

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-2.1.6` | `proposed` | `.1.5` is COMPLETE (ADR-008 + the delivery-carried decision + the tenant epoch + the node-side dispatch gate — the measured revocation invalidation); the incarnation/run writers execute now |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.4.
- `2026-09-07`: Unblocked — the Phase-1 G2 close (`PHASE-1.8.2`, gate record
  **Met**) releases the frontier; `.1` (workload certificate lifecycle,
  scoped grants, delegated authority context, revocation, cached-decision
  rules — backlog 11, ADR 007/008/009) executes after its pickup census.
- `2026-09-07`: `.1` decomposed at the census seams — the tool-backed census
  found the scoped-grant core + enrollment tokens EXIST while six gaps own
  the lane: no cert machinery (ADR-006/007/008/009 unopened), no revocation
  write path, no delegation context, no decision cache, no incarnation/run
  writers, no ledger issuer row; children `.1.1` (the ADR-006/007 spike) →
  `.1.2` (the cert lifecycle + channel v3) → `.1.3` (revocation surfaces) →
  `.1.4` (delegation context) → `.1.5` (cached decisions) → `.1.6`
  (incarnation/run writers); frontier → `.1.1`.
- `2026-09-07`: `.1.1` done — the ADR-006/007 spike: the experiment crate
  `crates/reasonbraid-cert-spike` (test-only — no bin, so `make release`
  stays four binaries) drove a real rustls TLS 1.3 client-cert handshake and
  passed 6/6 verdicts (trusted completes; foreign-CA, expired, and
  unregistered-fingerprint certificates refused on BOTH sides; rotation
  additive; issuance latency N=200 p50 63 µs / p95 69 µs); ADR-006
  (accepted-with-evidence) + ADR-007 (the project-local CA model) land; the
  `make deny` first run caught a real dependency split (two base64 versions
  via rcgen's optional `pem` feature) — fixed by dropping the unused feature,
  not a skip entry; the ledger gains the identity-stack row; frontier →
  `.1.2`.
- `2026-09-07`: `.1.2` decomposed further at the issuance-vs-channel seam
  (the Phase-1 `.1.2.1`-first precedent — a coherent interim exists): `.1.2.1`
  cert issuance at enrollment (migration 0011, the persisted server CA, the
  enroll response gains cert + dev-escrowed key, the node stores
  `cert.der`/`key.der`; the HMAC channel UNTOUCHED) → `.1.2.2` the channel v3
  cert-proof handshake + rotation (the 17 channel suites move, the demo
  enrolls → stores → handshakes with the cert); frontier → `.1.2.1`.
- `2026-09-07`: `.1.2.1` done — cert issuance at enrollment: migration 0011
  (`server_ca` + `node_certificates`), `ca.rs` (the CA is generated on first
  boot and LOADED thereafter — the rebuild test proves the same key + cert
  survive), the enroll response carries the leaf + dev-escrowed key +
  fingerprint, `rb-node` persists `cert.der`/`key.der` beside the journal;
  the HMAC channel untouched (the demo passes with the files stored, unused —
  the coherent interim); all guards green; frontier → `.1.2.2`.
- `2026-09-07`: `.1.2.2` done — the channel v3 cert-proof handshake +
  rotation: `CHANNEL_VERSION` 3 (the handshake signs the canonical coverage
  with the workload certificate's key; the server verifies chain-to-CA +
  validity + the node-id fingerprint + the signature before ANY ledger read);
  the rotate endpoint issues a fresh key + cert (additive fingerprints); the
  node rotates at ≤50% lifetime; the 19 channel tests + the demo (31 checks)
  pass on v3; the verification leg's interop discovery (ring refuses rcgen's
  SPKI DER, accepts the bare EC point) is recorded in
  `docs/decisions/2026-09-07_cert-proof-verification.md`; the book's
  node-channel + two-host-demo chapters carry the new contract; **`.1.2` is
  COMPLETE**; frontier → `.1.3`.
- `2026-09-07`: `.1.3.1` done — node/cert revocation: `POST /v1/nodes/revoke`
  (tenant_admin-audited: the unknown-node 404, the role 403 + audited denial,
  the re-revoke 409) sets `revoked_at` on the active certificates; migration
  0012 appends `suspended` to the presence view (Postgres view-replacement
  appends columns at the END only — the first attempt proved it); the next
  handshake is refused and presence reads suspended while the live lease is
  untouched; `rb node revoke`; the demo gains the beat (32 checks); the
  channel suite grew to 21; frontier → `.1.3.2`.
- `2026-09-07`: `.1.5.2` done — the cached-decision machinery: migration
  0013 (the per-tenant revocation epoch + the inbox's decision columns), the
  `.1.3` revocation writes bump the epoch IN their transaction, the delivery
  carries the admission decision (authz_ref + digest + decided_at +
  epoch-at-decision; the handshake/poll responses carry the current epoch;
  CHANNEL_VERSION 4), the journal stores the cached decision (the pre-shaped
  `authz_ref` finally gains a value), and the worker's dispatch boundary
  evaluates it — fresh + epoch-current dispatches, expired/epoch-stale/
  denied/absent refuses (journaled `failed_before_dispatch`, the adapter
  never runs). The measured live leg: the real worker completes the fresh
  allow, the grant revocation bumps the epoch 0→1, the next dispatch refuses
  WITHOUT a re-ask. **`.1.5` is COMPLETE**; frontier → `.1.6`.
- `2026-09-07`: `.1.5.1` done — ADR-008 accepted: the shipped in-tx
  evaluator stays (accepted with evidence — the OPA/Cedar comparison parks
  behind a measured trigger) and the node caches ONLY the admission
  decisions riding its delivery (§17.1: the journal can never locally
  re-evaluate a grant); the spike landed the pure `CachedDecision` +
  `CacheVerdict` + the `ActionClass` fail table (five offline tests:
  fresh+epoch-current allow, expiry, the epoch bump, the deny-is-never-
  widened rule, the §16.4 fail table); `cargo test -p reasonbraid-core` →
  44 passed. The verification found a REAL drift — the checked-in
  command-envelope schema golden predated `.1.4.2` (its NO REGRESSION set
  was live-suites-only; the core crate's own offline suite, home of the
  golden-drift test, was not re-run) — the golden is regenerated here and
  the discipline lesson recorded; frontier → `.1.5.2`.
- `2026-09-07`: `.1.5` decomposed at the ADR-vs-implementation seam — the
  census found the ROADMAP rule (§16.4) with NO machinery: no decision
  cache in the node or server (`grep -rn 'cache'` → 0 matches), no
  revocation epoch (the `.1.3` writes bump none), the poll payload carries
  no decision metadata — but the journal's `authz_ref` column is
  PRE-SHAPED (every writer binds `None`) and the authorization record
  holds the digest/version/decided_at; children `.1.5.1` (ADR-008 + the
  pure semantics spike) → `.1.5.2` (the delivery-carried decision + the
  tenant epoch + the node-side cache); frontier → `.1.5.1`.
- `2026-09-07`: `.1.4.2` done — the delegation implementation: the
  envelope's optional `authority_context` (the subject rides a string —
  the tagged-newtype wire fact), the DUAL evaluation (the caller's own
  grant AND the subject's grant — the record + digest bind the subject),
  the scope ladder (widening = a typed 403 naming the invariant), the CLI
  flags, and the acceptance test (narrower-succeeds / widening-refused /
  caller-check / revocation-freshness); command_api grew to 16; **`.1.4`
  is COMPLETE**; frontier → `.1.5`.
- `2026-09-07`: `.1.4.1` done — ADR-009 accepted: chain-in-envelope for
  the dev profile (the plumbing was pre-shaped; expiry/revocation ride the
  `.1.3` grant filters; no token lifecycle); the spike landed the pure
  `DelegationConstraints` + `delegation_scope_is_subset` with the offline
  tests (narrower/equal/empty pass, widening refused per-dimension, the
  wire-size leg); the core suite grew to 39; frontier → `.1.4.2`.
- `2026-09-07`: `.1.4` decomposed at the ADR-vs-implementation seam — the
  census found the plumbing PRE-SHAPED (`CommandAuthz.delegate_subject` +
  the audit subject split exist, always `None`) while the envelope field,
  the dual evaluation, and the widening check do not; children `.1.4.1`
  (ADR-009 + the representation spike) → `.1.4.2` (the implementation);
  frontier → `.1.4.1`.
- `2026-09-07`: `.1.3.2` done — the grant/boundary revocation write paths:
  the `Revoked` statuses got their verbs (`POST /v1/admin/grants/{id}/revoke`
  + `/v1/admin/boundaries/{id}/revoke`, tenant_admin-audited, typed 404/409
  refusals), the admin inspection lists (`GET /v1/admin/grants|boundaries` +
  `rb inspect grants|boundaries`), and the CLI verbs (`rb grant revoke` / `rb
  boundary revoke`); the tests prove the next authorization refuses with the
  audit row while other grants keep working, and the boundary revocation
  freezes the tenant's WRITES while the READ surfaces stay open (the freeze
  carve-out — `docs/decisions/2026-09-07_boundary-revocation-freeze.md`);
  command_api grew to 15; **`.1.3` is COMPLETE**; frontier → `.1.4`.
- `2026-09-07`: `.1.3` decomposed at the cert-vs-grant seam — the census
  found the REFUSAL paths already exist (the `.1.2.2` handshake checks
  `revoked_at`, the evaluation filters `status = 'active'`) while NO write
  path exists (`grep -n 'revoke' api.rs main.rs` → no verbs) and presence
  has no suspended state; children `.1.3.1` (node/cert revocation + the
  suspended presence + the demo beat) → `.1.3.2` (grant/boundary revoke
  verbs); frontier → `.1.3.1`.

## Acceptance Checklist (PHASE-2.1.5.1)

The CODE change owned by this leaf: `crates/reasonbraid-core/src/authority.rs`
(the `CachedDecision` + `CacheVerdict` + `ActionClass`/`FailMode` types, the
`CACHED_ALLOW_TTL_SECONDS` rule, and the five tests),
`crates/reasonbraid-core/src/lib.rs` (the re-exports), and
`crates/reasonbraid-core/schema/command-envelope.schema.json` (the
regenerated golden — a drift fix; see below) — `\.rs$` + `(^|/)crates/` in
`.doctrine/code_paths.txt`. ADR-008 is the record.

- [x] **REPRODUCE / ISSUE** — backlog 11's cached-decision sliver: the
  §16.4 rule has no machinery — `grep -rn 'cache'
  crates/reasonbraid-node/src/ crates/reasonbraid-server/src/` → 0 matches
  before this leaf; the poll payload carries no decision metadata and no
  revocation epoch exists (the `.1.3` writes bump none).
- [x] **ROOT CAUSE (WHY + WHERE)** — the dev profile evaluates every
  decision fresh in-transaction at admission; the node then dispatches at
  an irreversible boundary with no re-check (a revocation between
  admission and dispatch would NOT refuse). The fix point is the
  SEMANTICS first (pure, tested functions) so the `.1.5.2` wiring has a
  decided contract; the engine question is settled with evidence (the
  shipped evaluator covers the profile — a re-platform has no measured
  trigger).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no cache
  types, no ADR. After: `cargo test -p reasonbraid-core` → `test result:
  ok. 44 passed` (the five new tests: a fresh, epoch-current cached allow
  dispatches; an expired one is stale; an epoch bump invalidates a fresh
  entry; a cached deny is never widened by time; irreversible/admin
  writes fail closed, reads fail open); ADR-008 accepted
  (evidence-gated).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green
  (the core suite is the changed surface; the full offline re-run is the
  selected set, §16); `cargo clippy --all --all-targets -- -D warnings`
  → clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `crates/reasonbraid-core/src/authority.rs` (the cache
  semantics + the five tests); `src/lib.rs` (the exports);
  `crates/reasonbraid-core/schema/command-envelope.schema.json` (the
  regenerated golden — the `.1.4.2` envelope change had NOT regenerated
  it and its NO REGRESSION set never re-ran the core crate's own offline
  suite, so the drift sat undetected for one leaf;
  `cargo test -p reasonbraid-core -- --ignored write_schema_goldens`
  regenerates it); `docs/adr/008-authorization-engine-and-cached-decisions.md`
  + the INDEX row.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, `docs/adr/INDEX.md`,
  `docs/decisions/INDEX.md` (the drift-lesson record), KNOWLEDGE_MAP —
  same commit. DEV_NOTES: promoted →
  `docs/decisions/2026-09-07_verification-set-coverage.md` gained
  `answers:`.

## Acceptance Checklist (PHASE-2.1.5.2)

The CODE change owned by this leaf: migration `0013_cached_decisions.sql`
(`(^|/)migrations/` is a code path), `crates/reasonbraid-server/src/
{authority.rs,node_channel.rs,api.rs}` (the epoch + the delivery metadata +
the Allowed outcome's digest/decided_at), `crates/reasonbraid-node/src/
{channel.rs,journal.rs,node.rs,worker.rs}` + `migrations/0003_cached_decisions.sql`
(the cached decision + the dispatch gate + the epoch store),
`crates/reasonbraid-node/tests/worker_cached_decision.rs` +
`crates/reasonbraid-server/tests/node_work.rs` + `scripts/demo_two_host.sh`
(the version bump) + `docs/book/src/node-channel.md` — all code paths.

- [x] **REPRODUCE / ISSUE** — the §16.4 rule has no machinery (the `.1.5`
  census): `grep -rn 'cache' crates/reasonbraid-node/src/
  crates/reasonbraid-server/src/` → 0 matches before this leaf; the poll
  payload carried no decision metadata and no epoch existed to bump.
- [x] **ROOT CAUSE (WHY + WHERE)** — the dev profile evaluates fresh at
  admission and the node dispatches at an irreversible boundary with NO
  re-check: a revocation between admission and dispatch would NOT refuse.
  The fix point is the dispatch boundary (`.1.5.1`'s semantics wired to the
  real surfaces): the delivery carries the decision, the revocation writes
  bump the epoch, the node evaluates the cache before any provider contact.
- [x] **ADDRESSED (verified)** — measured before→after. Before: fresh
  evaluation everywhere, no cache, no epoch. After: `bash
  scripts/run_pg_tests.sh` → `test result: ok. 7 passed` (`node_work`, +1:
  the live measured leg — the REAL node worker completes the fresh allow
  (the contribution lands), the grant revocation bumps the tenant epoch 0→1
  (`SELECT revocation_epoch`), the NEXT dispatch of work decided under epoch
  0 is refused without a re-ask: `failed_before_dispatch` with the staleness
  in the evidence, no second contribution) + the full guard green (12 suites
  + e2e `2 passed` + the demo `ALL acceptance checks passed` (32 PASS,
  `rc=0`, `target/pg152b_guard.log`)); `cargo test -p reasonbraid-node
  --test worker_cached_decision` → `test result: ok. 5 passed` (expired /
  epoch-bumped / absent decision refuse; the fresh + epoch-current allow
  reaches the adapter and completes; the budget gate still refuses after
  the cached decision allows).
- [x] **NO REGRESSION** — `cargo test --all` → 43 offline suites green
  (rc=0); `cargo clippy --all --all-targets -- -D warnings` → clean;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at commit;
  `make book` builds.
- [x] **FIX** — migration 0013 (the epoch column + the inbox's decision
  columns); `authority.rs` (the `Allowed` outcome's digest/decided_at, the
  `bump_revocation_epoch` helper, the revoke helpers' transactions);
  `node_channel.rs` (the decision-carrying `enqueue_in_tx` + `ReplayCommand`
  + the current-epoch accessor + the response fields + CHANNEL_VERSION 4);
  `api.rs` (the `AdmissionDecision` capture + threading); the node's
  `channel.rs`/`journal.rs`/`node.rs`/`worker.rs` (the DTO fields, journal
  migration 0003, the cached-decision reader + epoch store, the dispatch
  gate + `refuse_dispatch`); the tests + the demo's version bump + the
  book's cached-decisions section.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, the book,
  KNOWLEDGE_MAP — same commit.

## Acceptance Checklist (PHASE-2.1.2.2)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/ca.rs`
(the verification legs), `src/node_channel.rs` (v3 + rotate),
`Cargo.toml` (rustls-webpki/ring/x509-parser), `crates/reasonbraid-node/src/channel.rs`
+ `src/node.rs` + `src/bin/rb-node.rs` (the v3 client + rotation), the test
files, and `scripts/demo_two_host.sh` — all code paths.

- [x] **REPRODUCE / ISSUE** — backlog 11's channel sliver is open: the
  handshake authenticates with the dev secret (HMAC) while the workload
  certificate exists but proves nothing — `grep -n 'key_proof'
  crates/reasonbraid-server/src/node_channel.rs` (before this leaf) →
  the v2 DTO + `verify_handshake_proof` read `node_keys`; no rotate
  surface (`grep -n 'rotate' crates/reasonbraid-server/src/node_channel.rs`
  → no matches).
- [x] **ROOT CAUSE (WHY + WHERE)** — `.1.2.1` deliberately stopped at
  issuance (the coherent interim); the channel swap is the `.1.2.2`
  contract itself. The fix point is the handshake boundary (the proof
  replaces the HMAC in the SAME canonical-coverage shape, so the
  replay/cursor semantics are untouched) + a rotate endpoint reusing the
  same verification ladder + the node-side identity install.
- [x] **ADDRESSED (verified)** — measured before→after. Before: HMAC v2,
  19 channel tests on the secret. After: `bash scripts/run_pg_tests.sh` →
  `test result: ok. 19 passed` (`node_channel`: the 17 migrated tests +
  the rotation pair — fresh fingerprint ≠ old, additive rows, both
  identities handshake, forged rotate 401) + the demo
  `ALL acceptance checks passed` (31 checks incl. the cert-file beat,
  `rc=0`, `target/pg122e_guard.log`); the handshake ladder refuses
  foreign/expired/unregistered/wrongly-signed certs with the typed 401
  (`handshake_without_a_valid_certificate_proof_is_refused`).
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → all twelve live
  server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 13 + 3 + 4 + 19
  + 4 + 3 + 6 + 7 `passed`) + CLI e2e `test result: ok. 2 passed` + the
  two-host demo `ALL acceptance checks passed` (31 PASS, `rc=0`,
  `target/pg122e_guard.log`); `cargo test --all` → 42 offline suites green
  (rc=0, `target/pg122c_offline.log`); `cargo clippy --all --all-targets
  -- -D warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make
  deny` → rc=0 (rustls-webpki with the `ring` feature + ring + x509-parser
  entered the server graph without a ban); `make gate` → 13/13 at commit;
  `make book` builds.
- [x] **FIX** — `ca.rs` (`verify_leaf_chain` — webpki chain + validity;
  `extract_point`; `verify_signature` — ring over the POINT, the measured
  interop fix); `node_channel.rs` (v3 DTOs, `verify_cert_proof`/
  `verify_rotate_proof`, the rotate endpoint + route); the node (`compute_cert_proof`,
  the v3 client with the installable identity, rotate-before-handshake at
  ≤50% lifetime, `Node::open` with the cert + key, the bin's identity
  load); the migrated suites (node_channel 19, node_work, node_inbox);
  the demo (v3 literals + the cert-file beat).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted →
  `docs/decisions/2026-09-07_cert-proof-verification.md` gained
  `answers:`), MEMORY, LIVE_STATUS, this tree's logs below,
  `docs/TASK_TREE.md` frontier, the book (node-channel + two-host-demo),
  `docs/decisions/INDEX.md`, KNOWLEDGE_MAP — same commit.

## Acceptance Checklist (PHASE-2.1.4.2)

The CODE change owned by this leaf: `crates/reasonbraid-core/src/{envelope.rs,authority.rs,lib.rs}`
(the envelope field + the re-exports), `crates/reasonbraid-server/src/{authority.rs,api.rs}`
(the dual evaluation + the delegation parse), `crates/reasonbraid-cli/src/{lib,main.rs}`
(the flags + threading), and the test files — all code paths.

- [x] **REPRODUCE / ISSUE** — the delegation plumbing is pre-shaped but
  UNWIRED: `grep -n "delegate_subject" crates/reasonbraid-server/src/authority.rs`
  → the field + the audit subject split exist while the envelope carries
  no delegation field (`grep -n "authority_context" crates/reasonbraid-core/src/envelope.rs`
  → no matches before this leaf) and the dual check does not exist.
- [x] **ROOT CAUSE (WHY + WHERE)** — `.1.4.1` decided the representation
  (chain-in-envelope); this leaf is the wiring: the envelope boundary +
  the authorize_in_tx dual evaluation + the scope ladder, riding the
  `.1.3` grant filters for freshness.
- [x] **ADDRESSED (verified)** — measured before→after. Before: no
  delegation on the wire, `delegate_subject` dormant. After: `bash
  scripts/run_pg_tests.sh` → `test result: ok. 16 passed` (`command_api`,
  +1: the delegation test — the narrower scope succeeds with the subject
  audited, the widening scope is a typed 403 naming the invariant, the
  caller's own authority gates, and the revoked subject grant refuses
  the next delegation) + the audit test's dual semantics (the record +
  digest bind the SUBJECT's grant); the full guard green (12 suites + e2e
  + the demo 32/32, `rc=0`, `target/pg142e_guard.log`).
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → all twelve live
  server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 16 + 3 + 4 + 21
  + 4 + 3 + 6 + 7 `passed`) + CLI e2e `test result: ok. 2 passed` + the
  demo `ALL acceptance checks passed` (32 PASS, `rc=0`,
  `target/pg142e_guard.log`); `cargo clippy --all --all-targets -- -D
  warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make gate` →
  13/13 at commit; `make book` builds.
- [x] **FIX** — `envelope.rs` (`AuthorityContext` + the envelope field);
  `authority.rs` (the `delegation_scope` field, `target_to_selector`,
  the dual evaluation with the caller check + the scope ladder, the
  record/digest binding the subject); `api.rs`
  (`delegation_from_envelope` + the handler wiring); the CLI (the two
  flags on every thread verb, the scope = the command's own target, the
  envelope threading); the tests (the delegation acceptance test; the
  audit test moved to the dual semantics with both grants).
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's logs
  below, `docs/TASK_TREE.md` frontier, the book's cli chapter,
  KNOWLEDGE_MAP — same commit. DEV_NOTES: `promotion: declined (the dual-evaluation semantics and the record-binds-the-subject rule are per-slice engine facts recorded here — no new cross-cutting decision)`.

## Acceptance Checklist (PHASE-2.1.4.1)

The CODE change owned by this leaf: `crates/reasonbraid-core/src/authority.rs`
(the `DelegationConstraints` type + `delegation_scope_is_subset` + the three
tests — `\.rs$` in `.doctrine/code_paths.txt`). ADR-009 is the record.

- [x] **REPRODUCE / ISSUE** — backlog 11's delegation sliver: the dev
  engine has no delegation representation — `grep -rn "on_behalf_of\|
  DelegationConstraints" crates/` → no matches before this leaf — while
  `CommandAuthz.delegate_subject` sits unused (always `None`).
- [x] **ROOT CAUSE (WHY + WHERE)** — the plumbing was built with the
  delegation in mind (the field + the audit subject split) but no wire
  shape was decided, and §16.3 forbids choosing by name; the fix point is
  a PURE prototype of the hardest invariant (the widening rule) + the
  measured representation comparison, so ADR-009 is evidence-gated.
- [x] **ADDRESSED (verified)** — measured before→after. Before: no type,
  no invariant, no ADR. After: `cargo test -p reasonbraid-core` →
  `test result: ok. 39 passed` (the three delegation tests: narrower/
  equal/empty subsets pass; a foreign thread and a tenant-wide request
  over a thread-scoped grant are refused; the envelope form beats a
  token blob on the wire — the size leg asserts the ordering);
  ADR-009 accepted (chain-in-envelope).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green
  (the core suite is the changed surface — the full offline re-run is
  the selected set, §16); `cargo clippy --all --all-targets -- -D
  warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make gate` →
  13/13 at commit.
- [x] **FIX** — `crates/reasonbraid-core/src/authority.rs` (the
  `DelegationConstraints` wire shape + the pure subset decision + the
  three tests); `docs/adr/009-delegated-authority-representation.md` +
  the INDEX row.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's logs
  below, `docs/TASK_TREE.md` frontier, `docs/adr/INDEX.md`,
  KNOWLEDGE_MAP — same commit (CHANGELOG.md rotated at the README-STABILITY
  threshold: the Phase-0 RB-SEED history → git history; 96,460 → 95,295
  bytes). DEV_NOTES: `promotion: declined (the GrantSubject tagged-newtype wire note is a per-slice serialization fact for .1.4.2, recorded in the leaf — no new cross-cutting decision)`.

## Acceptance Checklist (PHASE-2.1.3.2)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/authority.rs`
(the revoke helpers), `src/api.rs` (the four admin endpoints + the read
carve-out), `crates/reasonbraid-cli/src/{lib,main}.rs` (the verbs + runners),
and `crates/reasonbraid-server/tests/command_api.rs` — all code paths.

- [x] **REPRODUCE / ISSUE** — backlog 11's revocation sliver: the `Revoked`
  statuses exist as types + tests only — `grep -n "UPDATE authority_grants\|
  UPDATE enrollment_boundaries" crates/reasonbraid-server/src/authority.rs`
  → no matches before this leaf; no revoke verb (`grep -n "revoke"
  crates/reasonbraid-cli/src/main.rs` → only the node revoke from `.1.3.1`).
- [x] **ROOT CAUSE (WHY + WHERE)** — the evaluation filters
  `status = 'active'` (the refusal path is free), but no operator surface
  writes the status; the fix point is the tenant_admin surface (the
  `.1.3.1` pattern) + the two UPDATE helpers. The boundary case forced a
  REAL semantics decision mid-leaf: the admin authorization itself rides
  the boundary, so a boundary revocation would refuse even the inspection
  lists — the freeze carve-out (reads authorize grant-directly) is
  recorded in `docs/decisions/2026-09-07_boundary-revocation-freeze.md`.
- [x] **ADDRESSED (verified)** — measured before→after. Before: no verbs,
  no lists. After: `bash scripts/run_pg_tests.sh` → `test result: ok. 15
  passed` (`command_api`, +2: the grant revocation refuses the subject's
  NEXT command 403 + audited denial while the human keeps working, the
  list shows `revoked`, unknown 404, re-revoke 409; the boundary
  revocation freezes the tenant's writes 403 while the admin list stays
  200) + the demo stays 32/32 (`rc=0`, `target/pg132c_guard.log`).
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → all twelve live
  server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 15 + 3 + 4 + 21
  + 4 + 3 + 6 + 7 `passed`) + CLI e2e `test result: ok. 2 passed` + the
  demo `ALL acceptance checks passed` (32 PASS, `rc=0`,
  `target/pg132c_guard.log`); `cargo clippy --all --all-targets -- -D
  warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make gate` →
  13/13 at commit; `make book` builds.
- [x] **FIX** — `authority.rs` (`revoke_grant`/`revoke_boundary` — the
  row-first UPDATEs); `api.rs` (`RevokeAuthorityRequest`, the two revoke
  endpoints with the 404/409 ladder, `AdminListQuery` + the two list
  endpoints, `authorize_tenant_admin_read` — the freeze carve-out, the
  routes); the CLI (`GrantCommand::Revoke`/`BoundaryCommand::Revoke` +
  `InspectCommand::Grants|Boundaries` + the dispatch arms + the four
  runners + `post_admin`/`get_admin`); the two `command_api` tests.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted →
  `docs/decisions/2026-09-07_boundary-revocation-freeze.md` gained
  `answers:`), MEMORY, LIVE_STATUS, this tree's logs below,
  `docs/TASK_TREE.md` frontier, the book's cli chapter,
  `docs/decisions/INDEX.md`, KNOWLEDGE_MAP — same commit.

## Acceptance Checklist (PHASE-2.1.3.1)

The CODE change owned by this leaf: `migrations/0012_node_presence_suspended.sql`
(schema), `crates/reasonbraid-server/src/api.rs` (the revoke endpoint + the
not_found constructor + the route), `src/node_channel.rs` (the presence
response + query), `crates/reasonbraid-cli/src/{lib,main}.rs` (the verb),
the test files, and `scripts/demo_two_host.sh` — all code paths.

- [x] **REPRODUCE / ISSUE** — backlog 11's revocation sliver: NO revoke
  surface exists — `grep -n .revoke. crates/reasonbraid-server/src/api.rs
  crates/reasonbraid-cli/src/main.rs` → no matches before this leaf —
  while the REFUSAL paths already exist (the `.1.2.2` handshake ladder
  checks `revoked_at IS NOT NULL`; the presence view derives online only).
- [x] **ROOT CAUSE (WHY + WHERE)** — the `.1.2.2` row check was built with
  the `.1.3` write path in mind (`revoked_at` on the cert row), but the
  operator verb was never wired; the fix point is the tenant_admin
  surface (the issue-token/quarantine pattern — the authorization record
  IS the audit) + the presence view (0012).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no verb,
  no suspended state. After: `bash scripts/run_pg_tests.sh` → `test
  result: ok. 21 passed` (`node_channel`: the revocation pair — the next
  handshake after revoke is the typed 401; presence reads
  `"suspended":true` while the live lease stays `"online":true`; the
  unknown-node 404, the role 403 + the audited denial, the 409
  re-revoke) + the demo `ALL acceptance checks passed` (32 checks incl.
  the revoke beat, `rc=0`, `target/pg131f_guard.log`).
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → all twelve live
  server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 13 + 3 + 4 + 21
  + 4 + 3 + 6 + 7 `passed`) + CLI e2e `test result: ok. 2 passed` + the
  demo `ALL acceptance checks passed` (32 PASS, `rc=0`,
  `target/pg131f_guard.log`); `cargo clippy --all --all-targets -- -D
  warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make gate` →
  13/13 at commit; `make book` builds.
- [x] **FIX** — migration 0012 (`suspended` APPENDED — Postgres
  view-replacement adds columns at the end only, the first attempt
  inserted mid-list and the migrate step refused it: the suite caught
  it); `api.rs` (`RevokeNodeRequest`/`revoke_node` — the existence 404,
  the zero-active-cert 409, the `not_found` constructor, the route); the
  presence DTO + query; the CLI verb + runner; the two tests; the demo
  beat + summary row.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree.s logs
  below, `docs/TASK_TREE.md` frontier, the book (node-channel + cli),
  KNOWLEDGE_MAP — same commit. DEV_NOTES: `promotion: declined (the Postgres view-replacement append-only column rule is a per-slice SQL fact recorded here — no new cross-cutting decision)`.

## Acceptance Checklist (PHASE-2.1.2.1)

The CODE change owned by this leaf: `migrations/0011_workload_certificates.sql`
(schema), `crates/reasonbraid-server/src/ca.rs` (new), `src/node_channel.rs`
(state + router + enroll), `src/bin/rb-server.rs` (the boot), the test files,
and `crates/reasonbraid-node/src/bin/rb-node.rs` — all match `(^|/)crates/`,
`\.rs$` in `.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — backlog 11's cert-issuance portion is open:
  `grep -rn 'rcgen\|node_certificates\|server_ca' crates/reasonbraid-server/src
  migrations/` → no matches before this leaf (the `.1.1` spike is a separate
  crate, wired into nothing); the enroll response is `{node_id, host_id}`
  only (`grep -n 'pub struct NodeEnrollResponse'
  crates/reasonbraid-server/src/node_channel.rs` → 2 fields).
- [x] **ROOT CAUSE (WHY + WHERE)** — the issuance model was decided by
  ADR-007 but nothing consumes it: the server has no CA handle and the
  enroll path signs nothing. The fix point is the enroll transaction (the
  token-row serialization already guarantees exactly-one issuance) + a
  persisted CA the server can reload — the demo kills and restarts the
  server, so an in-memory CA would orphan every issued leaf.
  `git show HEAD:scripts/demo_two_host.sh | sed -n '385p'` →
  `kill -9 "$SERVER_PID" >/dev/null 2>&1 || true` (the restart kill point
  the CA must survive).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no CA, no
  cert rows, a 2-field enroll response. After: `bash
  scripts/run_pg_tests.sh` → `test result: ok. 4 passed; 0 failed`
  (`node_enrollment`, +1: the CA-persistence test — two `ensure_server_ca`
  passes return the SAME `cert_der` + `key_der`) and the happy-path test
  asserts the response's `cert_der`/`key_der`/`cert_fingerprint` (64 hex) +
  the `node_certificates` row + the `server_ca` row + the replay refusal
  issues no second cert (`target/pg121b_guard.log`).
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → all twelve live
  server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 13 + 3 + 4 + 17
  + 4 + 3 + 6 + 7 `passed`) + CLI e2e `test result: ok. 2 passed` + the
  two-host demo `ALL acceptance checks passed` (30 PASS, `rc=0`,
  `target/pg121b_guard.log` — the demo's `rb-node` now stores the cert
  files but the v2 channel does not use them: the coherent interim);
  `cargo test --all` → 42 offline suites green (rc=0,
  `target/pg121_offline.log`); `cargo clippy --all --all-targets -- -D
  warnings` → clean (rc=0); `cargo fmt --all -- --check` → rc=0; `make
  deny` → rc=0 (advisories/bans/licenses/sources ok — rcgen's
  `x509-parser` feature entered the server graph without a ban);
  `make gate` → 13/13 at commit.
- [x] **FIX** — `migrations/0011_workload_certificates.sql`;
  `crates/reasonbraid-server/src/ca.rs` (the `ServerCa` handle:
  generate-on-first-boot / load-from-the-row, `issue_node_leaf` with
  CN = node id + SAN = host claim, `cert_fingerprint`, hex helpers) +
  `src/lib.rs` (`pub mod ca`) + `Cargo.toml` (rcgen crypto+ring+
  x509-parser, rustls-pki-types, time); `node_channel.rs`
  (`NodeChannelState` gains the CA; `node_router(pool, ca)`; the enroll
  transaction issues + persists the leaf and the response carries it);
  `src/bin/rb-server.rs` (the CA boot step); `crates/reasonbraid-node/src/bin/rb-node.rs`
  (persist `cert.der`/`key.der` beside the journal, log the fingerprint);
  the test files (router signatures, purge lists, the two enrollment
  assertions).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (`promotion: declined (the server-generated dev-escrowed node key is the .1.2.1 trust-store stance recorded here + in ADR-007's honest limits — the Internet profile re-evaluates; no new cross-cutting decision)`),
  MEMORY, LIVE_STATUS, this tree's logs below, `docs/TASK_TREE.md`
  frontier — same commit.

## Acceptance Checklist (PHASE-2.1.1)

The CODE change owned by this leaf: `crates/reasonbraid-cert-spike/` (new —
matches `(^|/)crates/` in `.doctrine/code_paths.txt`). ADR-006/ADR-007 +
the ledger row are the record deliverables.

- [x] **REPRODUCE / ISSUE** — backlog 11's identity sliver is open and the
  §16.2 issuance contract has no implementation: `grep -rn
  'rustls|rcgen|x509|Certificate' Cargo.toml crates/*/Cargo.toml
  crates/*/src` → zero matches; the channel defers mTLS to ADR-006/007
  (`crates/reasonbraid-node/src/channel.rs` line 10); ADR-006/007/008/009
  are unopened (`grep -c '006\|007' docs/adr/INDEX.md` → 0 rows).
- [x] **ROOT CAUSE (WHY + WHERE)** — the roadmap forbids choosing the
  issuance technology by name, and nothing had measured any candidate, so
  the decision was un-makeable. Tool-backed census: `git show HEAD:crates/reasonbraid-node/src/channel.rs | sed -n '10p'` →
  `//! mTLS workload identity) arrives with ADR-006/ADR-007's formal
  records; the` (the channel's own deferral comment — the smoking gun);
  `grep -rn 'rcgen\|rustls' Cargo.toml crates/*/Cargo.toml` → no matches
  (no candidate even present); `grep -c '006\|007' docs/adr/INDEX.md` →
  `0` (both ADRs unopened). The fix point is a measured spike (the
  `rcgen` + `rustls` project-local-CA candidate — the only model that
  runs inside the monolith) + the ADR records the other candidates'
  operational comparison from published docs (recorded asymmetry).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no cert
  machinery, no ADRs, no ledger row. After: `cargo test -p
  reasonbraid-cert-spike -- --nocapture` → `test result: ok. 1 passed`
  with the verdicts — trusted allowlisted leaf completes (ping/pong),
  foreign-CA/expired/unregistered-fingerprint refused on BOTH sides,
  rotation additive, `issuance latency N=200 p50=63µs p95=69.042µs`
  (`target/spike81.log`); ADR-006 + ADR-007 accepted; the ledger's
  identity-stack row records the pinned versions (rcgen 0.14.10, rustls
  0.23.43, rustls-pki-types 1.15.1).
- [x] **NO REGRESSION** — `cargo test -p reasonbraid-cert-spike` → green;
  `cargo test --all` → all 39 offline suites + the spike green (rc=0,
  `target/spike81_all.log`); `cargo clippy -p reasonbraid-cert-spike
  --all-targets -- -D warnings` → clean (`target/spike81_clippy.log`);
  `cargo fmt --all -- --check` → rc=0; `make deny` → rc=0
  (advisories/bans/licenses/sources ok — the FIRST run failed on the
  two-base64 ban: rcgen's optional `pem` feature pulled base64 0.23; the
  fix is `default-features = false, features = ["crypto", "ring"]`, the
  spike consumes DER only — no skip entry added,
  `target/spike81_deny.log`); `make gate` → 13/13 at commit. No product
  code changed — the spike suite + the offline workspace + the gates are
  the selected set (§16).
- [x] **FIX** — `crates/reasonbraid-cert-spike/{Cargo.toml,src/lib.rs,
  tests/issuance_model.rs}` (the experiment: CA + leaf issuance, the
  composed chain+validity+allowlist verifier, the five handshake verdicts,
  the latency sweep); `docs/adr/006-node-transport-reconnect.md` +
  `007-workload-identity-issuance.md` + INDEX rows;
  `docs/decisions/2026-09-07_workload-identity-issuance.md` (the mirror
  with `answers:`); the ledger identity-stack row.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES (promoted →
  `docs/decisions/2026-09-07_workload-identity-issuance.md` gained
  `answers:`), MEMORY, LIVE_STATUS, this tree's logs below,
  `docs/TASK_TREE.md` frontier, `docs/adr/INDEX.md`,
  `docs/decisions/INDEX.md`, the dependency ledger, KNOWLEDGE_MAP — same
  commit.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-09-07` | `PHASE-2.1.1` | `cargo test -p reasonbraid-cert-spike -- --nocapture` → `test result: ok. 1 passed` (6/6 verdicts incl. the three refusal pairs + additive rotation; issuance N=200 p50=63µs p95=69µs, `target/spike81.log`); `cargo test --all` → 39 offline suites + the spike green (rc=0, `target/spike81_all.log`); `cargo clippy -p reasonbraid-cert-spike --all-targets -- -D warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make deny` → rc=0 (the first run caught the base64 split → rcgen ships without `pem`); `make gate` → 13/13 | the ADR-006/007 spike: the project-local CA model measured and adopted (ADR-007), the transport decision recorded (ADR-006), the ledger row filled — frontier → `.1.2` |
| `2026-09-07` | `PHASE-2.1.2.1` | `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 13 + 3 + 4 + 17 + 4 + 3 + 6 + 7 `passed` — `node_enrollment` grew to 4 with the CA-persistence test) + CLI e2e `2 passed` + the two-host demo `ALL acceptance checks passed` (30 PASS, `rc=0`, `target/pg121b_guard.log`); `cargo test --all` → 42 offline suites green (rc=0, `target/pg121_offline.log`); `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make deny` → rc=0; `make gate` → 13/13 | cert issuance at enrollment: the persisted `ServerCa` (generated on first boot, loaded thereafter — the rebuild test proves the same key + cert), the enroll response carries the leaf + dev-escrowed key + fingerprint, `rb-node` stores `cert.der`/`key.der`; the HMAC channel untouched (the coherent interim) — frontier → `.1.2.2` |
| `2026-09-07` | `PHASE-2.1.2.2` | `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 13 + 3 + 4 + 19 + 4 + 3 + 6 + 7 `passed`) + CLI e2e `2 passed` + the demo 31 PASS rc=0 (`target/pg122e_guard.log`); 42 offline suites; clippy/fmt clean; `make deny` rc=0; `make gate` 13/13 | the channel v3 cert-proof handshake + rotation landed; the ring-SPKI interop discovery recorded; **`.1.2` complete** — frontier → `.1.3` |
| `2026-09-07` | `PHASE-2.1.3.1` | `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 13 + 3 + 4 + 21 + 4 + 3 + 6 + 7 `passed` — `node_channel` grew to 21 with the revocation pair) + CLI e2e `2 passed` + the two-host demo `ALL acceptance checks passed` (32 PASS, `rc=0`, `target/pg131f_guard.log`); `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 | node/cert revocation: `POST /v1/nodes/revoke` (tenant_admin-audited, the typed refusals), the suspended presence (migration 0012), `rb node revoke`, the demo beat — frontier → `.1.3.2` |
| `2026-09-07` | `PHASE-2.1.3.2` | `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 15 + 3 + 4 + 21 + 4 + 3 + 6 + 7 `passed` — `command_api` grew to 15 with the revocation pair) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (32 PASS, `rc=0`, `target/pg132c_guard.log`); `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 | the grant/boundary revocation write paths + the admin inspection lists + the freeze carve-out; **`.1.3` complete** — frontier → `.1.4` |
| `2026-09-07` | `PHASE-2.1.4.1` | `cargo test -p reasonbraid-core` → `test result: ok. 39 passed` (the three delegation tests: subset narrowing/equality/emptiness pass, widening refused per-dimension, the wire-size leg); `cargo test --all` → every offline suite green; `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 | ADR-009 accepted (chain-in-envelope) + the pure subset prototype; frontier → `.1.4.2` |
| `2026-09-07` | `PHASE-2.1.4.2` | `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 16 + 3 + 4 + 21 + 4 + 3 + 6 + 7 `passed` — `command_api` grew to 16 with the delegation test) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (32 PASS, `rc=0`, `target/pg142e_guard.log`); `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 | the delegation implementation (the envelope field + the dual evaluation + the scope ladder + the CLI flags); **`.1.4` complete** — frontier → `.1.5` |
| `2026-09-07` | `PHASE-2.1.5.1` | `cargo test -p reasonbraid-core` → `test result: ok. 44 passed` (the five cache tests: fresh+epoch-current allow dispatches, expiry → stale, an epoch bump invalidates a fresh entry, a deny is never widened, the §16.4 fail table); `cargo test --all` → 42 offline suites green (rc=0 — the FIRST run failed the golden-drift test: the `.1.4.2` envelope change never regenerated `command-envelope.schema.json` and its live-suites-only NO REGRESSION set never re-ran the core crate's own suite; `write_schema_goldens` regenerated, the lesson recorded in `docs/decisions/2026-09-07_verification-set-coverage.md`); `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 | ADR-008 accepted (the shipped evaluator stays; the node caches ONLY the admission decisions riding its delivery) + the pure cache semantics landed; frontier → `.1.5.2` |
| `2026-09-07` | `PHASE-2.1.5.2` | `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 16 + 3 + 4 + 21 + 4 + 3 + 7 + 7 `passed` — `node_work` grew to 7 with the measured live leg: the REAL node worker completes the fresh allow, the revocation bumps the epoch 0→1, the next dispatch refuses without a re-ask) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (32 PASS, `rc=0`, `target/pg152b_guard.log`); `cargo test -p reasonbraid-node --test worker_cached_decision` → `test result: ok. 5 passed`; `cargo test --all` → 43 offline suites green; `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 | the cached-decision machinery (migration 0013 + the epoch-in-transaction + the delivery-carried decision + CHANNEL_VERSION 4 + the node-side dispatch gate); **`.1.5` complete** — frontier → `.1.6` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-2.1` | `REASONBRAID-PHASE2-0001` | the census-seam decomposition (six tool-backed gaps → `.1.1`–`.1.6`) |
| `PHASE-2.1.1` | `REASONBRAID-PHASE2-0002` | the ADR-006/007 spike + records: the test-only experiment crate, the two accepted ADRs, the ledger row; `make deny`'s ban caught the base64 split — fixed by dropping rcgen's unused `pem` feature |
| `PHASE-2.1.2` | `REASONBRAID-PHASE2-0003` | the issuance-vs-channel split (the `.1.2.1`-first precedent): `.1.2.1` cert issuance at enrollment → `.1.2.2` the channel v3 swap |
| `PHASE-2.1.2.1` | `REASONBRAID-PHASE2-0004` | cert issuance at enrollment: migration 0011 + `ca.rs` (the persisted CA) + the enroll response's cert + escrowed key + the node's `cert.der`/`key.der` persistence; the HMAC channel untouched |
| `PHASE-2.1.2.2` | `REASONBRAID-PHASE2-0005` | the channel v3 cert-proof handshake + rotation (as recorded — see the leaf's checklist); the 19 channel tests + the demo 31/31 |
| `PHASE-2.1.3` | `REASONBRAID-PHASE2-0006` | the cert-vs-grant split (the refusal paths exist; the write paths don't) |
| `PHASE-2.1.3.1` | `REASONBRAID-PHASE2-0007` | node/cert revocation: `POST /v1/nodes/revoke` + the suspended presence (migration 0012) + `rb node revoke` + the demo beat; the channel suite grew to 21 |
| `PHASE-2.1.3.2` | `REASONBRAID-PHASE2-0008` | the grant/boundary revoke verbs + the admin inspection lists + the freeze carve-out (reads survive the boundary revocation); `command_api` grew to 15 — **`.1.3` complete** |
| `PHASE-2.1.4` | `REASONBRAID-PHASE2-0009b` | the ADR-vs-implementation split (the delegation plumbing is pre-shaped) |
| `PHASE-2.1.4.1` | `REASONBRAID-PHASE2-0010` | ADR-009 (chain-in-envelope) + the pure `DelegationConstraints`/`delegation_scope_is_subset` prototype with the offline tests |
| `PHASE-2.1.4.2` | `REASONBRAID-PHASE2-0011` | the delegation implementation: the envelope's `authority_context`, the dual evaluation (caller + subject; the record binds the subject), the scope ladder, the CLI flags — **`.1.4` complete** |
| `PHASE-2.1.5` | `REASONBRAID-PHASE2-0012` | the ADR-vs-implementation split (no cache machinery; the journal's `authz_ref` is pre-shaped) |
| `PHASE-2.1.5.1` | `REASONBRAID-PHASE2-0013` | ADR-008 (the shipped evaluator stays; the node caches ONLY the admission decisions riding its delivery) + the pure `CachedDecision`/`CacheVerdict`/fail-table prototype (44 core tests); the verification caught + fixed the `.1.4.2` schema-golden drift (recorded in `docs/decisions/2026-09-07_verification-set-coverage.md`) |
| `PHASE-2.1.5.2` | `REASONBRAID-PHASE2-0014` | the cached-decision machinery: migration 0013 (the tenant epoch + the inbox decision columns), the revocation writes bump the epoch in-transaction, the delivery-carried admission decision + CHANNEL_VERSION 4, the node journal's cached decision + the dispatch gate (refuses stale/denied/absent — journaled, adapter never invoked); the measured live revocation-invalidation leg — **`.1.5` complete** |
