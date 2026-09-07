# PHASE-2: delivery, identity, and recovery hardening

## Metadata

- Tree ID: `PHASE-2`
- Status: `done`
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
  Status: `complete`
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
    incarnations). **`.1` is COMPLETE** — all six gaps closed: the
    project-local CA + cert lifecycle (GAP 1), the revocation write paths
    (GAP 2), the delegated authority context (GAP 3), the cached-decision
    semantics (GAP 4), the incarnation/run writers (GAP 5 — the Phase-1
    deferral #4 closes), the ledger identity row (GAP 6).

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
    Status: `active`
    Goal: incarnation/run writers — enroll records the incarnation row
      (harness + model/provider facts known at node start; the 0007
      hierarchy gets its writers), dispatches record run rows linked to
      the attempt; the Phase-1 gate-record deferral #4 closes.
    Backlog: —
    Acceptance: an enrolled node's incarnation row exists and is
      inspectable; a dispatch links its run + attempt; re-enroll/rotation
      do not duplicate incarnations; no regression.
    Note: gap census (`2026-09-07`, on pickup): the 0007 hierarchy is
      SCHEMA-ONLY — `grep -rn 'INSERT INTO incarnations\|INSERT INTO runs'
      crates/` → no matches (deferral #4: "the incarnation/run row writers
      are deferred to Phase 2 identity"); `incarnations` (role, provider/
      model/harness/config, validity window) + `runs` (→ incarnation)
      both exist. The node knows its harness at start (`rb-node` builds
      the adapter from its flags) but the ENROLL request carries none of
      the §8.1 facts (`provider`/`model`/`harness`/`config`); the dispatch
      boundary is node-local (the attempt row is journaled there, the
      server folds the result) — the run row needs a server-side write
      keyed on the result receipt.
    Children: `.1.6.1`–`.1.6.2` (decomposed `2026-09-07` at the
      incarnation-vs-run seam — the `.1.2.1`-first precedent: a coherent
      interim exists, incarnations without runs). **`.1.6` is COMPLETE** —
      the incarnation writer (enrollment's §8.1 facts) + the run writer
      (the result receipt's attempt→incarnation link) landed; the Phase-1
      gate-record deferral #4 CLOSES.

  - ID: `PHASE-2.1.6.1`
    Status: `done`
    Goal: the incarnation writer — the enroll request gains the §8.1
      facts the node knows at start (`provider`/`model`/`harness`/
      `config`; the dev profile's fake harness is one honest value), the
      enroll transaction writes the `incarnations` row (role, tenant, the
      facts, `valid_from` = now), `rb-node` gains the flags, and the row
      is inspectable (`rb inspect` or the enrollment surface); re-enroll/
      rotation do NOT duplicate incarnations (one ACTIVE incarnation per
      role — a re-enroll refreshes it or adds a new valid window, the
      decided contract below).
    Backlog: —
    Done (`2026-09-07`): the incarnation writer landed — the enroll
      request gains the §8.1 facts (all optional; `deny_unknown_fields`
      keeps the wire strict), the enroll transaction writes the
      `incarnations` row WHEN the node id is the agent ROLE wire id it
      serves (the dev wiring; a plain `nod_…` node serves no role and
      records no incarnation — the hierarchy's `role_id` is NOT NULL),
      the response returns the `incarnation_id`, `rb-node` gains
      `--provider`/`--model`/`--harness`/`--config`, and the tenant_admin
      inspection surface (`GET /v1/admin/incarnations` + `rb inspect
      incarnations`) shows the rows. No duplication is structural: the
      one-token-per-node index makes a second token unissuable, the
      consumed-token reuse refuses BEFORE the writer, and rotation has no
      incarnation writer. The live test (node_enrollment, +1: the row +
      its facts + the inspection + the refusal count + the role-less
      null) + the demo beat (33 checks) + the acceptance checklist below
      record the evidence — frontier → `.1.6.2`.
    Acceptance: an enrolled node's incarnation row exists with its §8.1
      facts and is inspectable; re-enroll/rotation do not duplicate
      incarnations; no regression.

  - ID: `PHASE-2.1.6.2`
    Status: `proposed`
    Goal: the run writer — a node-emitted result receipt records a `runs`
      row linked to the CURRENT incarnation + the attempt (the attempt id
      rides the result payload); the run is inspectable beside its
      attempt; deferral #4 closes.
    Backlog: —
    Done (`2026-09-07`): the run writer landed — migration 0014 adds
      `runs.attempt_id`; the result fold (`apply_node_result_in_tx`)
      writes the run row AFTER the idempotency claim (one result = one
      run, ever — a redelivered result replays the original application
      and writes no second run), linking the payload's attempt id to the
      role's CURRENT incarnation (valid_to IS NULL, latest valid_from);
      a result without an attempt id or an incarnation still folds (the
      linkage is best-effort, not a gate); the tenant_admin inspection
      (`GET /v1/admin/runs` + `rb inspect runs`) shows the run → attempt
      → incarnation → role chain. The live test (the result-fold test
      gained the three legs: the row + the chain + the inspection + the
      no-second-run count) + the demo beat (34 checks) + the acceptance
      checklist below record the evidence — **`.1.6` is COMPLETE** — the
      Phase-1 gate-record deferral #4 CLOSES.
    Acceptance: a dispatched attempt's result links its run + attempt +
      incarnation; inspection shows the chain; no regression.

- ID: `PHASE-2.2`
  Status: `complete`
  Goal: production-grade leases/fencing, retry policy, dead-letter/quarantine/replay
  ADR: 005 (transport choice if Phase 0 left it open)
  Note: gap census (`2026-09-07`, on pickup): EXISTS — the `.1.2.2`
    lease/fencing (60s TTL, token rotation per handshake, heartbeats
    renew, stale tokens refused; `grep -n LEASE_TTL node_channel.rs` →
    line 77) + the `.1.2.3` quarantine/prune (manual, operator verbs,
    reason-stored) + the no-silent-retry rule (outcome_unknown → proof or
    adjudication). MISSING — a capability-aware RETRY policy (§14.6: the
    provider-accepted-but-unproven class; `grep -rn 'retry' server/node
    src` → the reason code only), an automatic dead-letter surface (the
    quarantine is operator-driven, nothing auto-quarantines after N
    refusals), and replay (re-delivering a quarantined/dead-lettered
    command is impossible today — quarantine is one-way). ADR-005 is
    UNOPENED (`docs/adr/INDEX.md` has no 005); the Phase-0 `.2.2` outbox
    worker + the channel decisions are its evidence.
  Children: `.2.1`–`.2.4` (decomposed `2026-09-07` at the contract
    seams): `.2.1` ADR-005 (transport: the existing PG-queue evidence,
    accepted-with-evidence — the ADR-006 precedent) → `.2.2` lease/
    fencing hardening (renewal races, lease epochs, fencing on the
    dispatch path) → `.2.3` the retry policy (the §14.6 classes +
    `retry_requires_authorization` + the supervisor's retry gate) →
    `.2.4` dead-letter/replay (auto-quarantine after refusals + an
    operator replay verb). **`.2` is COMPLETE** — ADR-005 accepted, the
    lease epoch fenced the race, the retry policy landed, the quarantine
    became two-way.

  - ID: `PHASE-2.2.1`
    Status: `done`
    Goal: ADR-005 — the transport choice, accepted with evidence: the
      Phase-0 `.2.2` leased outbox worker IS the PostgreSQL-queue choice
      (ADR-004's machinery), the node channel is the pull surface, and
      the `.1` lane proved the delivery/replay/resume semantics over it;
      no broker experiment (the dev profile has no measured need — the
      NATS/JetStream trigger is named, ADR-006's precedent).
    ADR: 005
    Done (`2026-09-07`): ADR-005 accepted — the PostgreSQL queue is the
      event transport (the WP2 leased outbox worker for server jobs + the
      per-node inbox for the pull channel; the fencing/lease/ack
      semantics proven at kill points 3–5 are its production contract).
      The record names the broker revisit trigger (measured fan-out/push/
      replication need, with numbers). No code changed — the record
      promotes the shipped evidence (`docs/adr/INDEX.md` row added; the
      queue item closes). Frontier → `.2.2`.
    Acceptance: ADR-005 accepted (evidence-gated), the revisit trigger
      named; no code changes.

  - ID: `PHASE-2.2.2`
    Status: `done`
    Goal: lease/fencing hardening — the renewal race (a heartbeat
      concurrent with a handshake must not resurrect a fenced token:
      both write the lease row today, so the LAST writer wins — a stale
      heartbeat can extend a lease its own handshake just fenced), a
      lease EPOCH (every handshake bumps it; renewals/events carry the
      epoch they saw, and a renewal from a fenced epoch is refused), and
      the check-vs-commit window (the fencing check runs at request
      admission, then the transaction applies — a lease that lapses
      mid-transaction is not re-checked; the epoch column closes it).
    Done (`2026-09-07`): migration 0015 adds `node_leases.lease_epoch`;
      every handshake bumps it (the rotation's token AND epoch fence the
      old session); all four fenced writes (events/ack/poll/heartbeat)
      carry the epoch they saw and the fencing check requires the pair;
      `renew_lease` rides the epoch in its WHERE — a heartbeat racing a
      newer handshake matches no row (RowNotFound → the typed fencing
      refusal); the events handler re-verifies the pair INSIDE its
      transaction (`verify_fencing_in_tx`, the lease row FOR UPDATE) so a
      rotation landing between the admission check and the apply is
      observed; CHANNEL_VERSION 5; the node channel stores the epoch
      beside the token. The deterministic state-level test proves the
      race (a renewal from the fenced epoch is refused, the in-tx
      verifier refuses the stale pair) + the wire test re-proves every
      fenced surface with the epoch; the full guard green (12 suites +
      e2e + demo, `target/pg222c_guard.log`). The acceptance checklist
      below records the evidence — frontier → `.2.3`.
    Acceptance: the renewal race test (concurrent heartbeat + handshake
      → the old token stays fenced); a stale-epoch renewal is refused;
      no regression.

  - ID: `PHASE-2.2.3`
    Status: `done`
    Goal: the retry policy — the §14.6 classes as a pure decision (the
      provider-accepted-but-unproven class retries only with an explicit
      possible-duplicate authorization; the refused/failed_known classes
      never auto-retry; the transient pre-dispatch class retries
      bounded), the `retry_requires_authorization` reason code wired, and
      the supervisor's retry gate honoring it (a node-side journal fact,
      no silent retry).
    Done (`2026-09-07`): the pure `retry_decision` landed in the core
      crate (`retry.rs`: `None`/`prepared` re-dispatch unconditionally; a
      budget-denied item — no reservation — is terminal whatever the
      count; a reserved pre-dispatch refusal retries bounded
      (`MAX_DISPATCH_ATTEMPTS` 3); `outcome_unknown` retries ONLY with
      the delivery's `allow_possible_duplicate` flag — without it the
      refusal names `retry_requires_authorization`; dispatched →
      proof-or-adjudication; terminal states never; 5 tests — the core
      suite 49). The work payload gains the typed flag (false in the dev
      profile — nothing dispatches with duplicate risk); the worker's
      skip decision became the retry gate (the payload facts parsed
      first: reservation presence + the flag + the attempt count from
      the `.1.6.2` accessor; a refusal is logged with the reason and the
      item's journal status stays the visible fact). Four worker-level
      tests prove the legs (re-dispatch after a reserved refusal reaches
      the adapter and completes; the budget denial stays one attempt; the
      ambiguous refusal stays one attempt; the authorized ambiguous
      re-dispatch runs). The acceptance checklist below records the
      evidence — frontier → `.2.4`.
    Acceptance: the pure retry decision's tests (per-class allow/refuse);
      a would-be retry without the authorization is refused and visible;
      no regression.

  - ID: `PHASE-2.2.4`
    Status: `done`
    Goal: dead-letter/replay — auto-quarantine after N dispatch
      refusals (the `.1.5.2` gate's refusals count toward it) with the
      reason + the count, and the operator replay verb (a dead-lettered
      command re-enters the delivery tail with a fresh admission
      decision — quarantine becomes two-way).
    Done (`2026-09-07`): quarantine became TWO-WAY — the worker's
      terminal refusal (the `.2.3` retry gate's `Refuse`) reports a
      `work_dead_lettered` event ONCE per operation (the outgoing-events
      dedup; best-effort — journaled first, a failed send defers to the
      reconcile's re-emit), the server auto-quarantines the inbox row
      with the reason in the SAME transaction as the receipt, and
      `POST /v1/nodes/replay` + `rb node replay` reverse it: the
      quarantine clears, the admission decision REFRESHES (decided_at now
      + the CURRENT revocation epoch), and the row re-sequences to the
      delivery tail. The node's `record_command` refreshes the cached
      decision on a replayed redelivery, and the retry gate counts only
      attempts made under the CURRENT decision — the old refusals stop
      counting, so the replay re-arms the dispatch. The offline tests (2:
      the once-only report + the fresh-decision re-arm) + the LIVE
      end-to-end leg (the always-refusing worker dead-letters, the server
      auto-quarantines with the reason, the operator replays, the
      completing worker re-dispatches — exactly one contribution) record
      the evidence — **`.2` is COMPLETE** — frontier → `.3`.
    Acceptance: a repeatedly-refused command auto-quarantines with the
      refusal evidence; replay re-delivers it exactly once; inspection
      shows the dead-letter state; no regression.

- ID: `PHASE-2.3`
  Status: `complete`
  Goal: provider-attempt state machine, usage reconciliation, spend circuit breakers, ambiguous-outcome workflows
  Backlog: 23, 25
  ADR: 012, 013
  Note: gap census (`2026-09-07`, on pickup): EXISTS — the
    provider-attempt state machine (core: prepared/dispatched/completed/
    failed_before_dispatch/outcome_unknown/reconciled/failed_known with
    deterministic `apply`), the budget settlement (`.6.2`/
    `.1.5`-adjacent: `settle_reservation` records ACTUAL usage, overruns
    reported never clamped — `grep -n settle_reservation budget.rs`),
    and the ambiguous-outcome basics (outcome_unknown → proof or
    adjudication; the `.2.3` retry gate's `retry_requires_authorization`
    is the per-adapter retry policy's core). MISSING — spend CIRCUIT
    breakers (backlog 23; `grep -rn circuit crates/ migrations/` → no
    matches; the budget ceiling refuses per-reservation, but nothing
    stops NEW dispatches once a tenant's spend crosses a declared
    threshold), usage RECONCILIATION (backlog 25: the held-vs-settled
    picture, estimates vs receipts, uncertainty, pricing snapshots — the
    settlement records usage but no surface reconciles it), and
    ADR-012/013 are UNOPENED (the machinery they describe largely
    shipped — the promotion precedent, ADR-005/006).
  Children: `.3.1`–`.3.3` (decomposed `2026-09-07` at the contract
    seams): `.3.1` ADR-012/013 accepted-with-evidence (the shipped
    ambiguity machinery + budget invariants promote) → `.3.2` the spend
    circuit breakers (a declared per-tenant spend threshold refuses NEW
    dispatches at the reservation boundary) → `.3.3` the usage
    reconciliation surface (held vs settled vs overrun, the estimates vs
    receipts picture). **`.3` is COMPLETE** — ADR-012/013 accepted, the
    spend latch landed, the reconciliation surface shipped.

  - ID: `PHASE-2.3.1`
    Status: `done`
    Goal: ADR-012 + ADR-013, accepted with evidence: the ambiguity
      machinery (outcome_unknown → proof/adjudication, the §14.6
      no-silent-retry rule, the `.2.3` retry classes) promotes to
      ADR-012; the budget invariants (reserve before dispatch at both
      boundaries, settle with actual usage, overruns reported never
      clamped, denials recorded) promote to ADR-013 (the pricing-
      snapshot machinery is named as the Phase-4+ trigger). No code.
    ADR: 012, 013
    Done (`2026-09-07`): ADR-012 accepted — the ambiguity contract is the
      shipped machinery (the WP3 boundary-first journal, the WP4
      prove/adjudicate exits, the `.2.3` pure retry classes; the risky
      re-run requires the explicit possible-duplicate authorization);
      ADR-013 accepted — the budget contract is the shipped WP5 engine
      (reserve before dispatch at both boundaries, settle with actual
      usage, overruns reported never clamped, holds on indeterminate
      attempts; pricing snapshots deferred to Phase 4+ with the named
      trigger). Both promote the existing decision records
      (`docs/decisions/2026-09-06_node-journal.md`,
      `…fake-adapter.md`, `…real-adapter-codex.md`,
      `…budget-reservation.md`) — no code changed; the ADR INDEX gained
      both rows (012/013 close). Frontier → `.3.2`.
    Acceptance: both ADRs accepted (evidence-gated), the revisit
      triggers named; no code changes.

  - ID: `PHASE-2.3.2`
    Status: `done`
    Goal: the spend circuit breakers — a declared per-tenant spend
      threshold (a budget-ceiling extension or a sibling row): once the
      tenant's recorded spend (settled usage + held reservations) crosses
      it, NEW dispatch reservations are refused with a typed reason and
      the breaker state is inspectable + resettable (the operator verb);
      the refusal is audited (the denial-row pattern). Backlog 23's core.
    Backlog: 23
    Done (`2026-09-07`): the spend latch landed — migration 0016
      (`spend_breakers`: the tenant's threshold + tripped state + the
      reason); the reservation path checks it BEFORE any ceiling math
      (`check_spend_breaker_in_tx`): a tripped breaker refuses every new
      reservation with the typed reason, and an armed breaker trips IN
      the reservation transaction when the tenant's recorded spend
      (settled + active across all ceilings) plus the request crosses the
      threshold — the trip and the refusal commit with the denial's
      transaction (the latch never lags the ledger). The admin verbs
      (`POST /v1/admin/breakers` arm — re-arming clears a trip, `POST
      /v1/admin/breakers/reset`, `GET /v1/admin/breakers` +
      `rb breaker arm|reset` + `rb inspect breakers`). The live tests (2,
      the budget suite 7→9: the crossing trips + refuses with the typed
      reason + the latch refuses while tripped + the reset re-opens; the
      reset breaker re-trips on the next crossing) + the full guard green
      (12 suites + e2e + demo, `target/pg232b_guard.log`). The acceptance
      checklist below records the evidence — frontier → `.3.3`.
    Acceptance: a tenant over the threshold refuses NEW dispatches with
      the typed reason while existing reservations settle; the breaker
      resets; no regression.

  - ID: `PHASE-2.3.3`
    Status: `done`
    Goal: the usage reconciliation surface — the estimates-vs-receipts
      picture per thread/tenant: held (active reservations) vs settled
      vs overrun vs denied, with the per-dimension sums; the inspection
      is tenant_admin-gated and read-only (the budget engine's records,
      never a rewrite). Backlog 25's honest dev slice (pricing snapshots
      stay Phase 4+ — ADR-013's trigger).
    Backlog: 25
    Done (`2026-09-07`): `GET /v1/admin/usage` + `rb inspect usage`
      landed — the tenant's estimates-vs-receipts picture SUMMED over the
      ledger rows (held = active unexpired reservations; settled =
      actual usage; overrun = used minus reserved per dimension, floored;
      denied = the denials with their reasons; expired holds count
      nowhere) + the per-thread breakdown; tenant_admin-gated, read-only.
      The live test is MEASURED (the seeded rows' arithmetic recomputed
      in the test: held 2 / settled 8 / overrun 3 calls / denied 1, the
      expired hold excluded — command_api 17). The acceptance checklist
      below records the evidence — **`.3` is COMPLETE** — frontier →
      `.4`.
    Acceptance: the surface shows held/settled/overrun/denied per
      dimension for a thread + tenant; it matches the ledger rows
      (measured); no regression.

- ID: `PHASE-2.4`
  Status: `complete`
  Goal: backup, PITR, object/Git inventory groundwork, migrations, upgrade/rollback testing
  Roadmap: §17.5–17.6
  Note: gap census (`2026-09-07`, on pickup): NOTHING exists —
    `ls scripts/` has no backup/restore tooling (`grep -rn 'pg_dump|
    pg_basebackup|restore' scripts/ Makefile` → no matches); the
    migrations (0001–0016, sqlx) are applied to a FRESH database by every
    suite, but the upgrade-an-EXISTING-database path is never exercised
    (the guard re-creates the cluster each run); the demo's SIGKILL+
    restart beat proves durability, not restoration. The object/Git
    inventory + the signing-key recovery + the reconciliation have
    NOTHING to bind in the dev profile (no object store — Phase 4 — and
    no canonical Git mirror) — those are named deferrals, not build
    targets.
  Children: `.4.1`–`.4.3` (decomposed `2026-09-07` at the contract
    seams): `.4.1` the backup + restore automation with the measured
    restore exercise (a backup that has never been restored is not a
    recovery control) → `.4.2` the migration upgrade test (an existing
    database upgrades, its data survives) → `.4.3` the
    inventory-groundwork deferral record (object/Git inventory + key
    recovery + the reconciliation — named triggers). **`.4` is COMPLETE** —
    the restore exercise runs on every guard pass, the upgrade path is
    measured, the absent controls are named deferrals.

  - ID: `PHASE-2.4.1`
    Status: `done`
    Goal: backup + restore automation — `scripts/backup.sh` (pg_dump
      custom-format to a dated file, the dev profile's plaintext stance
      — encryption is the §17.5 note, deferred with the key story) +
      `scripts/restore.sh` (into an isolated database) + a guard leg: a
      live test backs up the seeded database, MUTATES it, restores into a
      fresh database, and asserts the state returned — the restore is the
      proof (§17.5's last line), run in CI.
    Roadmap: §17.5
    Done (`2026-09-07`): `scripts/backup.sh` + `scripts/restore.sh`
      landed, and the restore EXERCISE runs in the guard
      (`backup_restore` suite): the test seeds rows, takes a REAL pg_dump
      of the live database, MUTATES it, restores into an ISOLATED
      database (createdb → pg_restore → assert → dropdb), and asserts the
      restored state matches the pre-mutation state — the backup that has
      never been restored is not a recovery control, and this one is
      restored on every guard run. The suite skips offline (no
      DATABASE_URL) and when the pg tools are absent. The guard grew to
      14 suites + the demo 34/34 (`target/pg241f_guard.log`); the
      acceptance checklist below records the evidence — frontier →
      `.4.2`.
    Acceptance: the backup file restores into an isolated database and
      the restored state matches the pre-mutation state (measured); the
      guard leg is green.

  - ID: `PHASE-2.4.2`
    Status: `done`
    Goal: the migration upgrade test — a live suite applies migrations
      up to N-1 on a fresh database, SEEDS data through the real API,
      applies the remaining migrations, and asserts the seeded data +
      behavior survive (the expand/migrate/contract reality: the ALTERs
      are additive; the `.1.2.2` view lesson — Postgres view replacement
      appends columns — is the recorded caveat). Destructive contraction
      stays deferred (no rollback point exists in the dev profile — the
      §17.6 note).
    Roadmap: §17.6
    Done (`2026-09-07`): the upgrade-an-EXISTING-database path runs in
      the guard (`migration_upgrade` suite): all but the last migration
      apply to a clean schema, the REAL API seeds the tenant + boundary
      rows, the remaining migrations apply over the existing data, and
      the rows + the post-upgrade API behavior (the role enroll) survive
      — measured. The suite joins the guard (15 suites + the demo 34/34,
      `target/pg242b_guard.log`); the acceptance checklist below records
      the evidence — frontier → `.4.3`.
    Acceptance: the N-1 → N upgrade preserves the seeded rows + the API
      behavior (measured); no regression.

  - ID: `PHASE-2.4.3`
    Status: `done`
    Goal: the inventory-groundwork deferral record — the object/Git
      inventory, the signing-key recovery design, the post-restore
      reconciliation, and the backup encryption have nothing to bind in
      the dev profile (no object store, no canonical Git, no release
      signing); a decisions record names each with its trigger. No code.
    Roadmap: §17.5–§17.6
    Done (`2026-09-07`): `docs/decisions/2026-09-07_phase2-inventory-
      deferrals.md` landed (`answers:` present) — each absent control
      names the exact product surface whose arrival re-opens it (the
      object store → Phase 4, the Git mirror → Phase 6, the signing key →
      the first signed release, the multi-store reconciliation → the first
      multi-store restore, the encryption → the key story); no placeholder
      infrastructure (the subtraction doctrine). The `.4.1` restore
      exercise is the reconciliation that exists. No code changed.
      **`.4` is COMPLETE** — frontier → `.5`.
    Acceptance: the record lands with `answers:` naming each deferral's
      trigger; no code changes.

- ID: `PHASE-2.5`
  Status: `active`
  Goal: OpenTelemetry, operator dashboards, initial SLO baselines, game days
  Backlog: —
  ADR: 023
  Roadmap: §18
  Note: gap census (`2026-09-07`, on pickup): the observability is
    UNSTRUCTURED — the code's operational logs are `eprintln!` (16 sites:
    `grep -c 'eprintln!' server/src/*.rs node/src/*.rs` → api 8,
    node_channel 2, worker 6); no metrics, no traces, no SLO record, no
    runbook (`grep -rn 'tracing|opentelemetry|metrics'` → no matches).
    The four-record doctrine (§18.1) IS structurally true (the
    operational eprintln vs the durable event/audit tables are separate
    systems — a log is not an audit record by construction), but nothing
    measures, and ADR-023 is unopened.
  Children: `.5.1`–`.5.3` (decomposed `2026-09-07` at the contract
    seams): `.5.1` ADR-023 (the dev profile's four-record answer: no
    external telemetry sink yet — the trigger is named; the §18.2
    redaction rules pin the future sink) → `.5.2` the structured-log +
    metrics slice (JSON operational logs + a minimal admin metrics
    surface over the §18.3 minimums that APPLY: denials, attempt
    outcomes, command lag, dead letters) → `.5.3` the SLO record + the
    runbook slice (hypotheses from the existing guard/demo measurements;
    the demo's kill points ARE the game day, named).

  - ID: `PHASE-2.5.1`
    Status: `done`
    Goal: ADR-023 — telemetry storage/redaction, accepted with the dev
      profile's answer: the four records stay separate BY DESIGN (the
      operational eprintln/structured logs, the durable audit tables, and
      NO third-party telemetry sink yet); the §18.2 redaction rules pin
      the future sink (never prompt text, credentials, secret-bearing
      URLs, private evidence, or model output in span attributes;
      sensitive IDs tokenized at the sink boundary; a trace sampler can
      never decide whether a governance action remains provable — the
      audit record is the proof). No code.
    ADR: 023
    Done (`2026-09-07`): ADR-023 accepted — the four records remain
      separate systems (the shipped eprintln-vs-audit-tables design), the
      §18.2 redaction rules pin the FUTURE sink, and the OpenTelemetry
      dependency waits for the trigger (a non-loopback deployment or the
      G7 ops gate). No code changed; the ADR INDEX gained the row (023
      closes). Frontier → `.5.2`.
    Acceptance: ADR-023 accepted (the four-record separation is the
      shipped design; the sink trigger named); no code changes.

  - ID: `PHASE-2.5.2`
    Status: `done`
    Goal: the structured-log + metrics slice — the `eprintln!` sites
      become structured JSON lines (a `log_event!`-style helper: level,
      event, the correlation fields — no new dependencies, the
      OpenTelemetry sink stays the `.5.1` trigger) + a minimal metrics
      registry (counters for authorization denials, attempt outcomes,
      dead letters, idempotency replays, handshake refusals) exposed at
      `GET /v1/admin/metrics` (tenant_admin, read-only) — the §18.3
      minimums that apply to the dev profile.
    Backlog: —
    Done (`2026-09-07`): the observability slice landed — a process-wide
      in-memory registry (`telemetry.rs`: the seven counters), the
      increment sites on the real paths (the authorize denial arms, the
      read-gate refusals, the idempotency replay arm, the dead-letter
      auto-quarantine, the lease/proof refusals in the channel, the
      result fold/reject), the structured `log_event!` (JSON lines with
      the correlation fields — the rejected-result site converted), and
      `GET /v1/admin/metrics` (the gate: the caller HOLDS `tenant_admin`
      in any active grant — a process-global surface has no single
      tenant). The live test is MEASURED (the denial counter's delta
      matches the denied authorization row for the actor handle —
      command_api 18); the full guard green (15 suites + e2e + demo,
      `target/pg252d_guard.log`). The acceptance checklist below records
      the evidence — frontier → `.5.3`.
    Acceptance: the admin metrics surface answers with the counted
      signals; the counts match the records (measured); the structured
      logs carry the correlation fields; no regression.

  - ID: `PHASE-2.5.3`
    Status: `done`
    Goal: the SLO record + the runbook slice — the initial SLO
      HYPOTHESES (the §18.4 shape: population, window, statistic,
      target, error budget, owner) instantiated from the existing
      guard/demo measurements (the demo's 34 checks, the guard's 15
      suites, the restore exercise) as a decisions record; ONE runbook
      (node lost/replaced — the demo's SIGKILL beat is the exercise)
      with the §18.6 shape (detection, authority, safe first actions,
      diagnostics, containment, recovery, evidence). No code (docs).
    Backlog: —
    Done (`2026-09-07`): the SLO record
      (`docs/decisions/2026-09-07_phase2-slo-hypotheses.md`,
      `answers:` present) instantiates the §18.4 shape from the
      measurements that EXIST: SLO-1…SLO-4 (the guard's live
      assertions, the demo's 34 checks, the restore exercise, the
      reconcile-after-kill beats) target 100 % with a ZERO error
      budget (a red pass halts the frontier), SLO-5 records the one
      measured latency baseline (issuance p50 63 µs/p95 69 µs), and
      every unmeasured latency family is named with its trigger —
      never given a number. The runbook
      (`docs/runbooks/node-lost-replaced.md`) carries the full §18.6
      shape and its closure tests name the EXISTING exercises (the
      demo's SIGKILL beat + the revoke beat + the replay suites + the
      `.4.1` restore). No code changed. Frontier → `.6`.
    Acceptance: the SLO record + the runbook land; the runbook's
      closure test names the existing exercise; no code changes.

- ID: `PHASE-2.6`
  Status: `done`
  Goal: adapter conformance kit and permanent failure fixture corpus
  Roadmap: §19.4
  Children: `.6.1`–`.6.3` (decomposed `2026-09-07` at the contract
    seams): `.6.1` the conformance harness (the one mechanical suite
    every adapter passes against the contract) → `.6.2` the permanent
    failure-fixture corpus (the pinned replay oracle + the §19.4
    coverage map) → `.6.3` the qualification checklist + the named
    deferrals (the §19.4 items with nothing to build against in the
    dev profile). **`.6` is COMPLETE** — the five checkable §19.4
    items are mechanical (the harness + the pinned corpus), the manual
    half is the book's checklist, and the five absent-surface items
    are named deferrals with triggers.
  Done (`2026-09-07`): the census mapped §19.4's ten items against
    the shipped adapter surface: EXISTS — the capability declaration
    + unsupported-operation behavior (`AdapterCapabilities` +
    `StatusLookupOutcome::Unsupported`, `contract.rs`), the honest
    ambiguous-outcome reporting (the `outcome_unknown` contract + the
    lose-response fixtures), the usage accounting with confidence
    (`NormalizedUsage` + `UsageConfidence`), the sanitized fixture
    corpus (10 fixtures, the credential-scan + coverage tests in
    `fixtures.rs`), the cancellation/streaming semantics
    (`CancellationStrength` + `AttemptHandle`), the four per-adapter
    test files. ABSENT/partial — the conformance suite is FOUR
    SEPARATE files (no single contract harness); the corpus is not
    pinned as a replay oracle with a §19.4 coverage map; rate-limit/
    backoff normalization, output-size limits, tool-call validation,
    the provider error taxonomy, and prompt/policy projection
    fidelity have no machinery (or no surface to bind — projection
    fidelity is Phase 6's); the manual qualification checklist is
    not named as one artifact. Children at those seams — frontier →
    `.6.1`.

  - ID: `PHASE-2.6.1`
    Status: `done`
    Goal: the conformance harness — ONE mechanical suite every adapter
      (fake + codex + claude) passes against the CONTRACT (§19.4's
      checkable items): the capability-manifest agreement (the declared
      `AdapterCapabilities` match the adapter's behavior — e.g. a
      non-streaming declaration refuses the streaming path, an
      unsupported status lookup is declared as such), the
      unsupported-operation honesty (never a retry recommendation), the
      secret-containment invariant (the adapter struct carries no
      credential field; credentials arrive via env at the process
      boundary), the cancellation semantics (the `CancellationStrength`
      table), the ambiguous-outcome honesty (`outcome_unknown` — never a
      silent terminal), and the usage-accounting invariant
      (`NormalizedUsage.confidence` is never a lie: unknown dimensions
      are unknown, not zero). The existing four per-adapter test files
      fold under this harness (the census found them separate).
    Backlog: —
    Done (`2026-09-07`): the harness landed —
      `crates/reasonbraid-adapter/tests/conformance/` (the six
      invariant checks over a `ConformanceScenario`: the manifest
      agreement, the never-dispatched lookup's `Unsupported` honesty,
      the dispatch boundary, the lost-response honesty, the
      cancellation ceiling, the usage-accounting floor) +
      `tests/conformance/stubs.rs` (the shared provider stubs) +
      `tests/adapter_conformance.rs` (the three adapters register
      scenarios against ONE suite: the fake via its fixture corpus's
      OWN declarations, codex + claude via the stub binaries +
      the missing-binary refusal). The duplicated cross-adapter tests
      (the capability boundary, the unsupported lookup) left the two
      per-adapter files — they keep their provider-specific mechanics
      (prompt travel, stderr tails, the child kill, the receipt
      shapes). All three pass the suite; the evidence is in the
      acceptance checklist below — frontier → `.6.2`.
    Acceptance: the one harness runs against all three adapters; each
      §19.4-checkable item above has a named test; no regression.

  - ID: `PHASE-2.6.2`
    Status: `done`
    Goal: the permanent failure-fixture corpus — pin the ten fixtures
      (`fixtures/*.json`) as the REPLAY oracle: a versioned corpus
      manifest (the fixture set is permanent: no fixture is silently
      edited or dropped — changes are additive with a recorded reason),
      the §19.4-item → fixture coverage map (which fixture proves which
      conformance item), and the sanitized-replay guarantee as a test
      (the corpus replays through the fake adapter AND through every
      real adapter's fixture-mapping path — the credential scan stays
      the mechanical gate).
    Backlog: —
    Done (`2026-09-07`): `fixtures/MANIFEST.json` (version 1: one entry
      per fixture — file, the §19.4-item keys it proves, the adding
      leaf, the recorded reason) + the manifest machinery in
      `fixtures.rs` (`CorpusManifest`, `manifest()`, the
      `CONFORMANCE_ITEMS` key list) + the two mechanical guarantees:
      `manifest_matches_the_corpus_exactly` (the manifest and the
      corpus drift-check each other — additive changes only) and
      `every_conformance_item_is_covered_by_the_manifest` (every
      mapped item has a replay fixture; the four dev-profile deferrals
      are the ONLY unmapped keys). The real adapters replay the corpus
      semantics through the `.6.1` harness scenarios (their
      fixture-mapping path); the credential scan stays the mechanical
      gate. Frontier → `.6.3`.
    Acceptance: the manifest + the coverage map land; the replay
      guarantee holds for every adapter that maps the corpus; no
      regression.

  - ID: `PHASE-2.6.3`
    Status: `done`
    Goal: the manual qualification checklist + the named deferrals —
      the §19.4 last item (the sanitized replay fixture + the manual
      qualification checklist) becomes ONE artifact (the book's
      adapter-boundary chapter gains the checklist the env-gated live
      qualification already follows: `RB_LIVE_CODEX=1`/`RB_LIVE_CLAUDE=1`
      runs, the money-cost gate, the credential env handling); and the
      §19.4 items with no machinery or no surface to bind in the dev
      profile are NAMED with their triggers as a decisions record:
      rate-limit/backoff normalization (no provider rate-limit signal
      is parsed — trigger: the first real 429), output-size limits
      (no bound on streamed output — trigger: the first streamed run
      over the limit the policy sets), tool-call validation (the
      capability flag exists, the validator doesn't — trigger: the
      first tool-capable adapter), the provider error taxonomy (the
      adapters map exit/stderr to the honest outcomes — trigger: the
      first provider-specific error that needs a typed reason code),
      prompt/policy projection fidelity (Phase 6's projection
      machinery — trigger: the first policy projection).
    Backlog: —
    Done (`2026-09-07`): the book's adapter-boundary chapter gains the
      six-box qualification checklist (the conformance harness pass,
      the stub-mechanics pass, the env-gated bounded live dispatch,
      the credential containment, the manifest entry, the dependency-
      ledger row); the deferrals record
      (`docs/decisions/2026-09-07_phase2-adapter-conformance-deferrals.md`,
      `answers:`) names the five §19.4 items with the exact trigger
      that re-opens each. No code changed. **`.6` COMPLETE** — frontier
      → `.7`.
    Acceptance: the checklist artifact + the deferrals record land;
      each deferral names its trigger; no code changes.

- ID: `PHASE-2.7`
  Status: `done`
  Goal: exit — non-escalation properties; restore + node replacement; no false safe-retry of unknown attempts
  Gate: feeds G6–G7; subtraction record required
  ADR: 022 (audit hash-chain groundwork)
  Children: `.7.1`–`.7.4` (decomposed `2026-09-07` at the census
    seams): `.7.1` the non-escalation property suite (the §16.12
    adversarial line — cross-tenant, confused-deputy, delegation
    widening, revoked-reuse) → `.7.2` the node-replacement drill (the
    runbook's named gap, measured) → `.7.3` ADR-022 (the audit
    hash-chain groundwork) + the no-false-safe-retry inventory →
    `.7.4` the Phase-2 subtraction record + the G6–G7 feed.
    **`.7` is COMPLETE — Phase 2 is CLOSED**: the exit line's three
    properties are measured, not argued (the adversarial suite, the
    replacement drill, the six-leg inventory), the mandatory §19.8
    subtraction record ships, and the G6–G7 feed hands the evidence
    to the next deployment profile.
  Done (`2026-09-07`): the census mapped the exit lane's three
    properties + the gate items against the shipped surface: the
    non-escalation FOUNDATIONS exist (the deny-by-default evaluation,
    the subset checker, the freeze carve-out, the four
    escalation-adjacent authority tests — membership grants nothing,
    admin never implied, every decision recorded, grants can't exceed
    the boundary) but NO adversarial property suite names the
    §16.12 line (`grep -rn "escalat|cross.tenant"` over the server
    tests → 0 matches); the restore exercise + the replay machinery
    exist but the node-replacement DRILL is the runbook's own named
    gap; the no-false-safe-retry property has four measured legs
    (`worker_retry_policy`: the reserved refusal redispatch, the
    budget denial never redispatch, the ambiguous refusal without
    authorization, the authorized redispatch) — the exit lane records
    the inventory, it does not re-prove; ADR-022 is unopened (the
    roadmap's "audit hash-chain/checkpoint and verification policy").
    Children at those seams — frontier → `.7.1`.
  - ID: `PHASE-2.7.1`
    Status: `done`
    Goal: the non-escalation property suite — the §16.12 line
      "authorization non-escalation properties and confused-deputy
      tests" becomes a NAMED adversarial suite over the shipped
      surface: a cross-tenant access attempt refuses (every tenant
      boundary), a confused deputy cannot widen (a delegated caller
      acting on the principal's grant cannot exceed the delegated
      scope — the `.1.4` ladder), a revoked credential/epoch-stale
      decision refuses (the `.1.3` revocation + the `.1.5.2` cache),
      a freeze refuses the writes while the reads stay authorized
      (the `.1.3.2` carve-out), and a replayed/forged envelope is
      refused (the idempotency + the digest bindings). The existing
      four escalation-adjacent authority tests stay; this suite names
      the ADVERSARIAL surface as one file.
    Backlog: —
    Done (`2026-09-07`): the adversarial suite landed —
      `crates/reasonbraid-server/tests/escalation.rs` (4 measured
      tests, one per escalation surface): the cross-tenant attempt
      refuses at every boundary (the create 403 + audited denial, the
      thread read 404 with no existence leak, the key-replay 409 with
      the typed idempotency conflict); the delegation scope is the
      ceiling even for a deputy whose OWN grant covers the target
      (403 + the widening invariant named); the nuclear option cannot
      be re-armed (identity minting under a revoked boundary is 400
      at enrollment, no row left); the revocation fences future work
      but never rewrites history (the original replay returns its
      stored result with exactly ONE contribution event, the next new
      command 403 + audited). The first live run caught four REAL
      behaviors the tests then pinned (the conflict typing, the
      enrollment fence). The guard's live list gained the suite
      (16 suites now). Frontier → `.7.2`.
    Acceptance: the adversarial suite lands with a named test per
      escalation surface; all green; no regression.

  - ID: `PHASE-2.7.2`
    Status: `done`
    Goal: the node-replacement drill — the runbook's own named gap
      closes with a MEASURED exercise: destroy a node's journal (the
      total-machine-loss path), re-enroll a replacement incarnation
      (same role id), let the channel replay the inbox from the
      durable cursor, prove the in-flight attempt lands its honest
      terminal (`outcome_unknown` — never a silent retry), then
      `rb node replay` the dead-lettered one; the guard gains the
      drill as a live suite (or the demo gains the beat — whichever
      the census finds cheaper to hold). The runbook's closure tests
      gain the drill line (the Phase-7 game day then EXERCISES it at
      scale, it no longer builds it).
    Backlog: —
    Done (`2026-09-07`): the drill landed —
      `crates/reasonbraid-server/tests/node_replacement.rs` (the ONE
      measured ritual, in the guard) + the machinery the drill exposed
      and fixed: the replacement enroll path in `node_channel.rs`
      (the node row exists + every cert revoked → the fresh token +
      secret enroll a NEW incarnation with a 'replaced' audit decision
      + the dev-key swap; the state check runs BEFORE any insert), the
      0018 token-reissue migration (0008's unconditional UNIQUE made
      the first token permanent — the comment always said one UNUSED
      token; the partial index pins it), the 0017 presence migration
      (suspended = a revoked cert AND no active cert — the replacement
      reads NOT suspended while the old certs stay fenced). The
      measured chain: the lost node's attempt lands `outcome_unknown`
      (the honest terminal), the journal is destroyed, the revocation
      fences the old cert + bumps the epoch, the replacement
      reconciles + the inbox tail replays, and THE FENCE HOLDS — the
      re-delivered decision is epoch-stale, the dispatch refuses
      fail-closed (never a silent re-dispatch of the lost node's
      in-flight work), the row dead-letters, the operator replays, and
      exactly ONE contribution lands. The runbook's recovery section
      + closure tests carry the ritual. Frontier → `.7.3`.
    Acceptance: the drill runs measured on every guard pass; the
      runbook names it; no regression.

  - ID: `PHASE-2.7.3`
    Status: `done`
    Goal: ADR-022 (audit hash-chain/checkpoint and verification
      policy) — the groundwork record: the shipped audit-linkage
      design (the actor/subject/grant/digest bindings, the
      deterministically derived handles) is the chain's GROUNDWORK,
      accepted-with-evidence; the hash-chain itself is deferred WITH
      its trigger (the first non-loopback deployment or the G7 ops
      gate — the same trigger family ADR-023 named). Plus the
      no-false-safe-retry evidence inventory: the four measured
      retry-policy legs + the quarantine replay gate, recorded as the
      §16.12 line's dev-profile proof (no code).
    Backlog: —
    Done (`2026-09-07`): ADR-022 accepted (evidence-gated) —
      `docs/adr/022-audit-hash-chain-groundwork.md`: the shipped
      linkage (the actor/subject/grant/digest bindings, the
      deterministic handles, the per-thread monotonic sequence, the
      audit reconstruction) IS the chain's groundwork; the hash chain
      + checkpoint + verification policy are deferred WITH their
      trigger (the first non-loopback deployment / the G7 ops gate —
      the ADR-023 family). The retry-safety inventory landed —
      `docs/decisions/2026-09-07_phase2-retry-safety-inventory.md`
      (`answers:`): six MEASURED legs (the retry decision, the
      dispatch gate, the quarantine gate, the replacement fence, the
      reboot fence, the history fence) prove the machine refuses every
      silent re-dispatch by default; the operator's flags are the only
      human path. The `.7.2` drill's epoch fence is the inventory's
      R-4. No code changed. Frontier → `.7.4`.
    Acceptance: ADR-022 accepted-with-evidence + the inventory
      recorded; the chain's trigger named; no code changes.

  - ID: `PHASE-2.7.4`
    Status: `done`
    Goal: the Phase-2 subtraction record + the G6–G7 feed — the
      §19.8 mandatory record names what Phase 2 did NOT build (no
      Internet exposure, no external review, no SSRF suite, no
      prompt-injection suite, no release signing, no chaos injection
      beyond the kill beats, no rate-limit machinery — each with the
      profile that re-opens it); and the gate-feed record maps the
      §16.12/G7 lines to the Phase-2 evidence (the restore exercise,
      the drill, the SLO record, the runbook, the conformance kit)
      vs what stays open until the deployment profile exists. No
      code (docs).
    Backlog: —
    Done (`2026-09-07`): the Phase-2 subtraction record
      (`docs/decisions/2026-09-07_phase2-subtraction-record.md`,
      `answers:` — S-1…S-13: thirteen named not-built items, each
      with the profile that re-opens it) + the G6–G7 feed
      (`docs/decisions/2026-09-07_phase2-gate-feed.md`, `answers:` —
      every §16.12/G7 line mapped to its Phase-2 evidence or named
      OPEN with its profile). No code changed.
      **`PHASE-2.7` COMPLETE — Phase 2 is CLOSED** (the exit line:
      authority non-escalation proven by the adversarial suite, full
      restore + node replacement proven by the exercises, no known
      path that reports an ambiguous provider attempt as safely
      retryable — the six-leg inventory). The next executable work is
      `PHASE-3.1` (the `PHASE-3` tree).
    Acceptance: the subtraction record + the gate-feed record land;
      each deferred item names its re-opening profile; no code
      changes. **`PHASE-2.7` and Phase 2 close.**


## Current Frontier

**Tree complete.** Phase 2 is CLOSED — the exit line's properties are
measured (authority non-escalation: the `.7.1` adversarial suite; restore
and node replacement: the `.4.1` exercise + the `.7.2` drill; no false
safe-retry of unknown attempts: the `.7.3` six-leg inventory), the §19.8
subtraction record + the G6–G7 feed ship. The next executable work is
`PHASE-3.1` (the `PHASE-3` tree, now `active`).
 `.5.1` done — ADR-023 accepted (the four-record separation + the redaction rules pinning the future sink); the structured-log + metrics slice executes now |
 `.5` decomposed at the contract seams (the census: eprintln-only observability; the four-record doctrine is structurally true but nothing measures; ADR-023 unopened); the ADR-023 record executes now |
 `.4` is COMPLETE (the restore exercise, the measured upgrade path, the named deferrals); the observability lane executes now |
 `.4.2` done — the migration upgrade test (the existing-database path, measured); the inventory-groundwork deferral record executes now |
 `.4.1` done — the backup + restore automation with the measured restore exercise; the migration upgrade test executes now |
 `.4` decomposed at the contract seams (the census: no backup tooling, the upgrade path unexercised, the inventory has nothing to bind — named deferrals); the backup + restore automation executes now |
 `.3` is COMPLETE (ADR-012/013, the spend latch, the reconciliation surface); the backup/PITR + migrations lane executes now |
 `.3.2` done — the spend circuit breakers (the in-tx latch + the arm/reset/inspect verbs); the usage-reconciliation surface executes now |
 `.3.1` done — ADR-012/013 accepted (the shipped ambiguity + budget machinery promotes; no code); the spend circuit breakers execute now |
 `.2.2` done — the lease epoch hardened the fencing (the renewal race + the check-vs-commit window); the retry policy executes now |
 `.2.1` done — ADR-005 accepted (the PostgreSQL queue, evidence-gated; no code changes); the lease/fencing hardening executes now |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.4.
- `2026-09-07`: `.7.4` done — the Phase-2 subtraction record
  (S-1…S-13, each with its re-opening profile) + the G6–G7 feed
  (every gate line mapped to its evidence or named OPEN); no code
  changed. **`.7` COMPLETE — Phase 2 is CLOSED** (the exit line's
  three properties measured; the subtraction record ships).
- `2026-09-07`: `.7.3` done — ADR-022 accepted (the shipped
  audit linkage is the chain's groundwork; the chain + checkpoint +
  verification policy wait for the named trigger) + the retry-safety
  inventory (six measured legs: the machine refuses every silent
  re-dispatch by default); no code changed; frontier → `.7.4`.
- `2026-09-07`: `.7.2` done — the node-replacement drill: the
  measured ritual (outcome_unknown → journal destroyed → revoke →
  replacement enroll → inbox replay → the epoch-stale refusal fence
  → dead letter → operator replay → exactly one fold) + the three
  machinery gaps the drill exposed and fixed (the replacement enroll
  path, the 0018 token-reissue index, the 0017 presence derivation);
  the runbook carries the ritual; the guard grew to 17 live suites;
  frontier → `.7.3`.
- `2026-09-07`: `.7.1` done — the non-escalation property suite: 4
  measured adversarial tests (cross-tenant 403/404/409, the
  confused-deputy 403 despite the deputy's own grant, the
  re-arm fence at enrollment, the revocation fence around
  history); the guard gained the suite (16 live suites);
  frontier → `.7.2`.
- `2026-09-07`: `.7` decomposed at the census seams — the exit lane's
  three properties mapped: the non-escalation foundations exist (no
  adversarial suite), the replacement drill is the runbook's named
  gap, the no-false-safe-retry property has four measured legs,
  ADR-022 is unopened; children `.7.1` (the non-escalation suite) →
  `.7.2` (the replacement drill) → `.7.3` (ADR-022 + the retry
  inventory) → `.7.4` (the subtraction record + the G6–G7 feed);
  frontier → `.7.1`.
- `2026-09-07`: `.6.3` done — the manual qualification checklist
  (the book's six-box gate over the env-gated live runs) + the
  deferrals record (the five §19.4 items with no dev-profile
  machinery, each named with its trigger); **`.6` COMPLETE** —
  frontier → `.7`.
- `2026-09-07`: `.6.2` done — the permanent failure-fixture corpus:
  `fixtures/MANIFEST.json` (version 1, one entry per fixture with the
  §19.4-item keys + the adding leaf + the reason) + the manifest
  machinery + the two mechanical guarantees (the exact-match
  drift check, the item-coverage check); no fixture can be silently
  added, dropped, or edited; frontier → `.6.3`.
- `2026-09-07`: `.6.1` done — the conformance harness: one suite
  (`tests/conformance/` + `adapter_conformance.rs`) runs the six §19.4
  invariant checks against all three adapters (the fake via its
  fixture corpus's own declarations, codex + claude via the shared
  stubs + the missing-binary refusal); the duplicated cross-adapter
  tests left the per-adapter files; `test result: ok. 3 passed` for
  the suite, 48 offline suites green; frontier → `.6.2`.
- `2026-09-07`: `.6` decomposed at the contract seams — the census
  mapped §19.4's ten items against the shipped surface (capabilities,
  unsupported behavior, ambiguity, usage-confidence, the sanitized
  corpus exist; the harness is four separate files, the corpus is
  unpinned as a replay oracle, five items have no machinery — named);
  children `.6.1` (the conformance harness) → `.6.2` (the permanent
  corpus) → `.6.3` (the qualification checklist + the deferrals);
  frontier → `.6.1`.
- `2026-09-07`: `.5.3` done — the SLO record (the guard IS the dev
  profile's population: SLO-1…SLO-4 target 100 % with a zero error
  budget; SLO-5 is the one measured latency baseline; the unmeasured
  families are named with triggers) + the node lost/replaced runbook
  (the full §18.6 shape; the closure tests name the demo's SIGKILL
  beat, the revoke beat, the replay suites, the restore exercise); no
  code changed; frontier → `.6`.
- `2026-09-07`: `.5.2` done — the structured-log + metrics slice: the
  process-wide registry (seven counters on the real paths), the `log_event!`
  JSON lines, `GET /v1/admin/metrics` (the caller-holds-tenant_admin gate),
  the measured live test (the denial delta matches the record); command_api
  18; frontier → `.5.3`.
- `2026-09-07`: `.5.1` done — ADR-023 accepted: the four records are
  separate systems (the shipped design), the §18.2 redaction rules pin the
  future sink, the OpenTelemetry dependency waits for the trigger; no code
  changed; frontier → `.5.2`.
- `2026-09-07`: `.5` decomposed at the contract seams — the census found
  UNSTRUCTURED observability (16 `eprintln!` sites, no metrics/traces/SLO/
  runbook) while the four-record doctrine is structurally true; children
  `.5.1` (ADR-023) → `.5.2` (the structured-log + metrics slice) → `.5.3`
  (the SLO record + the runbook slice); frontier → `.5.1`.
- `2026-09-07`: `.4.3` done — the inventory-groundwork deferral record
  (`docs/decisions/2026-09-07_phase2-inventory-deferrals.md`, `answers:`): the
  object/Git inventory, the key recovery, the multi-store reconciliation, and
  the encryption are deferred WITH their triggers; no placeholder
  infrastructure. **`.4` COMPLETE**; frontier → `.5`.
- `2026-09-07`: `.4.2` done — the migration upgrade test: the guard's
  `migration_upgrade` suite applies all but the last migration, seeds through
  the REAL API, upgrades over the existing data, and asserts the rows + the
  behavior survive (measured); frontier → `.4.3`.
- `2026-09-07`: `.4.1` done — the backup + restore automation:
  `scripts/backup.sh`/`scripts/restore.sh` + the guard's restore exercise
  (seed → real pg_dump → mutate → restore into an isolated database →
  assert the pre-mutation state — the backup is restored on every guard
  run); frontier → `.4.2`.
- `2026-09-07`: `.4` decomposed at the contract seams — the census found
  NOTHING: no backup/restore tooling, the upgrade-an-existing-database path
  unexercised, and the object/Git inventory with nothing to bind in the dev
  profile (named deferrals); children `.4.1` (backup + restore with the
  measured exercise) → `.4.2` (the migration upgrade test) → `.4.3` (the
  inventory-groundwork deferral record); frontier → `.4.1`.
- `2026-09-07`: `.3.3` done — the usage-reconciliation surface:
  `GET /v1/admin/usage` + `rb inspect usage` (held vs settled vs overrun
  vs denied, summed over the ledger rows the engine enforces against; the
  live test is measured against the seeded arithmetic); **`.3` COMPLETE**;
  frontier → `.4`.
- `2026-09-07`: `.3.2` done — the spend circuit breakers: migration 0016's
  per-tenant latch (tripped refuses every new reservation; the crossing trips
  IN the reservation transaction — the latch never lags the ledger), the
  arm/reset/inspect admin verbs + `rb breaker arm|reset` + `rb inspect
  breakers`; the budget suite grew to 9; frontier → `.3.3`.
- `2026-09-07`: `.3.1` done — ADR-012 accepted (the ambiguity contract is
  the shipped machinery: the WP3 boundary-first journal + the WP4
  prove/adjudicate exits + the `.2.3` pure retry classes) and ADR-013
  accepted (the WP5 budget invariants; pricing snapshots deferred to Phase
  4+ with the named trigger); both promote the existing decision records,
  no code changed; frontier → `.3.2`.
- `2026-09-07`: `.3` decomposed at the contract seams — the census found
  the state machine + the settlement + the ambiguity basics EXIST while
  the circuit breakers (backlog 23), the usage-reconciliation surface
  (backlog 25), and ADR-012/013 are open; children `.3.1` (ADR-012/013
  accepted-with-evidence) → `.3.2` (the spend circuit breakers) → `.3.3`
  (the reconciliation surface); frontier → `.3.1`.
- `2026-09-07`: `.2.4` done — the two-way quarantine: the terminal refusal
  reports the dead letter ONCE (best-effort, journaled first), the server
  auto-quarantines in the receipt transaction, `POST /v1/nodes/replay` +
  `rb node replay` clear + refresh the admission decision + re-sequence the
  row, and the replayed redelivery refreshes the node's cached decision
  (the retry count is decision-scoped — the old refusals stop counting);
  the live end-to-end leg (refuse → dead-letter → replay → exactly one
  contribution). **`.2` COMPLETE**; frontier → `.3`.
- `2026-09-07`: `.2.3` done — the retry policy: the pure `retry_decision`
  (§14.6 classes: a budget denial is terminal, a reserved pre-dispatch
  refusal retries bounded, an ambiguous outcome needs the explicit
  possible-duplicate flag — the refusal names `retry_requires_authorization`),
  the work payload's typed flag, the worker's retry gate (the attempt count
  rides the `.1.6.2` accessor); core 49 + the four worker legs; frontier →
  `.2.4`.
- `2026-09-07`: `.2.2` done — lease/fencing hardening: migration 0015's
  lease epoch rides every fenced write (the token AND the epoch fence the old
  session), a stale-epoch renewal matches no row (the heartbeat loses the
  race), the events transaction re-verifies the pair FOR UPDATE (the
  check-vs-commit window), CHANNEL_VERSION 5; frontier → `.2.3`.
- `2026-09-07`: `.2.1` done — ADR-005 accepted: the PostgreSQL queue is the
  event transport (the WP2 leased outbox worker + ADR-004 + the ADR-006 pull
  channel — shipped across two phases; the broker trigger is named). No code
  changed. Frontier → `.2.2`.
- `2026-09-07`: `.2` decomposed at the contract seams — the census found
  the `.1.2.2` lease/fencing + the `.1.2.3` quarantine/prune EXIST in
  their Phase-1 forms while the retry policy, the dead-letter/replay
  surface, and ADR-005 are open; children `.2.1` (ADR-005
  accepted-with-evidence) → `.2.2` (lease/fencing hardening) → `.2.3`
  (the retry policy) → `.2.4` (dead-letter/replay); frontier → `.2.1`.
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
- `2026-09-07`: `.1.6.2` done — the run writer: migration 0014 adds
  `runs.attempt_id`; the result fold writes the run row AFTER the
  idempotency claim (one result = one run, ever) linking the attempt to
  the role's CURRENT incarnation; `GET /v1/admin/runs` + `rb inspect
  runs` show the chain; the result-fold test gained the legs + the demo
  beat (34 checks). **`.1.6` complete — deferral #4 closes — `.1` is
  COMPLETE** (all six census gaps closed); frontier → `.2`.
- `2026-09-07`: `.1.6.1` done — the incarnation writer: the enroll request
  gains the §8.1 facts, the enroll transaction writes the `incarnations`
  row when the node id is the role wire id it serves (a plain `nod_…` node
  records none — the hierarchy binds incarnations to roles), the response
  returns the `incarnation_id`, `rb-node` gains the four flags, and
  `GET /v1/admin/incarnations` + `rb inspect incarnations` inspect; no
  duplication is structural (one token per node, the refusal before the
  writer, no rotation writer); the demo gains the beat (33 checks);
  frontier → `.1.6.2`.
- `2026-09-07`: `.1.6` decomposed at the incarnation-vs-run seam — the
  census found the 0007 hierarchy SCHEMA-ONLY (`grep -rn 'INSERT INTO
  incarnations\|INSERT INTO runs' crates/` → no matches; deferral #4) and
  the enroll request carries none of the §8.1 facts; children `.1.6.1`
  (the incarnation writer at enrollment) → `.1.6.2` (the run writer keyed
  on the result receipt); frontier → `.1.6.1`.
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

## Acceptance Checklist (PHASE-2.7.2)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/node_channel.rs` (the replacement enroll
path + the dev-key swap + the pre-insert state check),
`migrations/0017_node_replacement_presence.sql` (NEW),
`migrations/0018_node_token_reissue.sql` (NEW),
`crates/reasonbraid-server/tests/node_replacement.rs` (NEW — the drill),
`scripts/run_pg_tests.sh` (the guard's live list gained the drill), and
`docs/runbooks/node-lost-replaced.md` (the ritual + the closure tests) —
`\.rs$` + `\.sh$` + `(^|/)migrations/` in `.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the runbook (`.5.3`) promised a
  replacement ritual the machinery did not ship: the FIRST drill run
  found the re-enrollment refused (the `nodes` primary key), the
  second found the token re-issue refused (0008's unconditional
  UNIQUE — the comment always said "one UNUSED token"), the third
  found the dev-key insert refused (`node_keys_pkey`), and the fourth
  found the presence derivation would keep the replacement suspended
  forever (0012's "any revoked cert").
- [x] **ROOT CAUSE (WHY + WHERE)** — no replacement path existed at
  all — `git grep -c "replacement" ad0468e --
  crates/reasonbraid-server/src/node_channel.rs` → rc=1 (zero
  matches); a destroyed credential is the ONE recovery case with no
  shipped machinery (the rotate exchange needs the current cert, the
  enroll primary key refuses the id). The fix point is the enroll
  boundary: a node whose certificates are ALL revoked is a declared
  loss, and a fresh token + secret enroll the replacement (a new
  incarnation) — with the state check BEFORE any insert (a
  unique-violation probe aborts the transaction).
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  re-enroll 409, the token re-issue 409, the key insert pkey
  violation. After:
  `DATABASE_URL=postgres://postgres@127.0.0.1:55432/reasonbraid_test
  cargo test -p reasonbraid-server --test node_replacement` →
  `test result: ok. 1 passed` — the whole ritual measured: the lost
  node's attempt lands `outcome_unknown`, the journal is destroyed,
  the revocation fences the old cert (proof refused), the replacement
  enroll returns the new cert + the second incarnation + the
  'replaced' audit row, the presence reads NOT suspended, the inbox
  tail replays, the re-delivered decision is epoch-stale and the
  dispatch REFUSES fail-closed (no silent re-dispatch), the row
  dead-letters, the operator replay refreshes the decision, and
  exactly ONE contribution lands.
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 17 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg272_guard.log`); `cargo test --all` → 50 offline suites
  green; `cargo clippy --all --all-targets -- -D warnings` → clean;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at commit.
- [x] **FIX** — `node_channel.rs` (the replacement branch + the key
  swap), `0017` (the presence derivation), `0018` (the partial
  one-unused-token index), `tests/node_replacement.rs` (the drill),
  the guard script, the runbook.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-2.7.1)

The CODE change owned by this leaf:
`crates/reasonbraid-server/tests/escalation.rs` (NEW — the adversarial
suite) and `scripts/run_pg_tests.sh` (the guard's live list gained the
suite) — `\.rs$` + `\.sh$` in `.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the `.7` census: the §16.12 line
  "authorization non-escalation properties and confused-deputy tests"
  has foundations but no NAMED adversarial suite.
- [x] **ROOT CAUSE (WHY + WHERE)** — every refusal path is exercised
  only by happy-path-adjacent tests; NO test asserts the headline §5
  property (no known cross-tenant escalation) — `git grep -c
  "cross.tenant\|escalat" ad0468e -- crates/reasonbraid-server/tests/`
  → rc=1 (zero matches before this leaf). The fix point is one
  adversarial suite staging each escalation attempt as a REAL
  envelope and asserting the refusal AND its audit row.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `DATABASE_URL=postgres://postgres@127.0.0.1:55432/reasonbraid_test
  cargo test -p reasonbraid-server --test escalation` → `test result:
  ok. 4 passed` — the four attacks: the cross-tenant create 403 +
  audited (read 404, replay 409 conflict-typed), the confused
  deputy's delegation widening 403 DESPITE its own grant, the
  re-arm 400 at the enrollment fence (no row left), the revocation
  fence (replay returns the stored result with exactly ONE
  contribution event; the next command 403 + audited). The first
  live run caught four real behaviors (the conflict typing, the
  enrollment fence) that the tests then pinned.
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 16 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg271b_guard.log`); `cargo test --all` → 49 offline
  suites green; `cargo clippy --all --all-targets -- -D warnings` →
  clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `tests/escalation.rs` (the four adversarial tests +
  the shared scaffolding); `scripts/run_pg_tests.sh` (the guard's
  live list gained `--test escalation` — the suite runs on every
  guard pass).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-2.6.2)

The CODE change owned by this leaf: `crates/reasonbraid-adapter/
fixtures/MANIFEST.json` (NEW — the versioned corpus manifest) and
`crates/reasonbraid-adapter/src/fixtures.rs` (the `CorpusManifest`
machinery + the two guarantee tests) — `(^|/)crates/` in
`.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the `.6` census: the corpus is the
  conformance oracle (§11.6) but NOTHING pins it — no manifest, no
  permanence rule, nothing stops a fixture from being silently edited
  or dropped (the coverage + credential tests would shrink with it).
- [x] **ROOT CAUSE (WHY + WHERE)** — the corpus lives as raw JSON +
  an include-list only; permanence is a DOCUMENTED property, not a
  mechanical one — `git grep -c "manifest" ad0468e --
  crates/reasonbraid-adapter/src/` → rc=1 (no manifest machinery
  existed). The fix point is a versioned manifest that the corpus
  drift-checks against: the two must match EXACTLY, so a silent
  add/drop/edit fails a test instead of shrinking the oracle.
- [x] **ADDRESSED (verified)** — measured before→after. Before: zero
  manifest machinery (the grep above). After:
  `cargo test -p reasonbraid-adapter --lib` → `test result: ok. 9
  passed` — the two new guarantees: `manifest_matches_the_corpus_exactly`
  (the manifest's entry files equal the corpus's fixtures in order;
  version pinned) and `every_conformance_item_is_covered_by_the_manifest`
  (every §19.4 item key is proven by a fixture, except the four named
  dev-profile deferrals; unknown keys are typos and fail).
- [x] **NO REGRESSION** — `cargo test --all` → 48 offline suites
  green; `bash scripts/run_pg_tests.sh` → 15 live suites + the demo
  `ALL acceptance checks passed` 34/34 (`target/pg262_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → clean;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at commit.
- [x] **FIX** — `fixtures/MANIFEST.json` (version 1, one entry per
  fixture: file, the §19.4-item keys, the adding leaf, the reason);
  `src/fixtures.rs` (`CorpusManifest`/`CorpusManifestEntry`,
  `manifest()`, `CONFORMANCE_ITEMS`, the two tests).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, the book's
  adapter-boundary chapter, KNOWLEDGE_MAP — same commit.

## Acceptance Checklist (PHASE-2.6.1)

The CODE change owned by this leaf:
`crates/reasonbraid-adapter/tests/conformance/mod.rs` (NEW — the
harness: `ConformanceScenario` + `Trigger` + the six invariant checks),
`crates/reasonbraid-adapter/tests/conformance/stubs.rs` (NEW — the
shared provider stubs), `crates/reasonbraid-adapter/tests/
adapter_conformance.rs` (NEW — the three adapters register against the
ONE suite), `crates/reasonbraid-adapter/tests/codex_adapter.rs` +
`crates/reasonbraid-adapter/tests/claude_adapter.rs` (the duplicated
cross-adapter tests removed; the shared stub replaces the local copy) —
`\.rs$` + `(^|/)crates/` in `.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the `.6` census: §19.4's checkable items
  live in FOUR separate per-adapter test files with no single
  contract harness, and the two real adapters carry their own copies
  of the shared stub + the shared invariants (the capability-boundary
  test, the unsupported-lookup test) — a drift risk the conformance
  item is meant to kill.
- [x] **ROOT CAUSE (WHY + WHERE)** — the contract already carries the
  declarations (`AdapterCapabilities`, `StatusLookupOutcome::Unsupported`,
  `CancellationStrength`, `UsageConfidence`) but NOTHING asserts them
  uniformly across adapters — `git grep -c "capabilities()" ad0468e
  -- crates/reasonbraid-adapter/tests/` → claude_adapter.rs:1 +
  codex_adapter.rs:1 + fake_adapter.rs:2 (each rc=0), `git grep -c
  "query_status" ad0468e -- crates/reasonbraid-adapter/tests/` →
  claude:2 + codex:2 + fake:4, `git grep -c "stub_binary" ad0468e`
  → the stub copied twice. The fix point is ONE harness over a
  scenario shape (name + trigger + declared caps + the tripping
  request) that every adapter registers against — the invariants
  live once.
- [x] **ADDRESSED (verified)** — measured before→after. Before: no
  shared harness (the census count above). After:
  `cargo test -p reasonbraid-adapter --test adapter_conformance` →
  `test result: ok. 3 passed` (fake, codex, claude each pass the six
  checks: the manifest agreement against the scenario's declared
  boundary, the never-dispatched lookup's `Unsupported` honesty, the
  dispatch boundary, the lost-response no-terminal honesty, the
  cancellation ceiling, the usage-accounting floor);
  `cargo test -p reasonbraid-adapter` → 8 `test result: ok` lines
  (the two per-adapter files keep their provider-specific mechanics —
  codex 7, claude 8).
- [x] **NO REGRESSION** — `cargo test --all` → 48 offline suites
  green; `bash scripts/run_pg_tests.sh` → 15 live suites + the demo
  `ALL acceptance checks passed` 34/34 (`target/pg261_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → clean;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at commit.
- [x] **FIX** — `tests/conformance/mod.rs` (the six checks + the
  scenario shape), `tests/conformance/stubs.rs` (the shared stubs —
  the per-adapter binaries include it via `#[path]` and use only
  their provider's stub), `tests/adapter_conformance.rs` (the three
  registrations), the two per-adapter files (duplicates removed).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-2.5.2)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/
telemetry.rs` (NEW — the process-wide registry + the `log_event!` macro),
`crates/reasonbraid-server/src/lib.rs` (the module),
`crates/reasonbraid-server/src/api.rs` (the increment sites + the
`/v1/admin/metrics` surface), `crates/reasonbraid-server/src/
node_channel.rs` (the channel-side increments + the structured rejected-
result line), `crates/reasonbraid-server/tests/command_api.rs` (the
measured test) — `\.rs$` + `(^|/)crates/` in `.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the `.5` census found UNSTRUCTURED
  observability: 16 `eprintln!` sites, no metrics, no structured lines
  (the `.5` decompose row in the changelog). Nothing measures the
  governance-relevant signals (denials, replays, dead letters,
  refusals), so no §18.3 surface can answer "did anything get refused
  since the last check".
- [x] **ROOT CAUSE (WHY + WHERE)** — the eprintln sites format by
  hand: they drop the correlation fields and carry no machine-readable
  event name, and no registry counts the refusal-family paths —
  `git grep -c "log_event!" bb42f65 -- crates/` → rc=1 (no structured
  emitter existed) and `git grep -c "admin/metrics" bb42f65 --
  crates/` → rc=1 (no metrics surface existed). The fix point is the
  boundary where each refusal is decided (the authorize arms in
  `api.rs`, the channel verifies in `node_channel.rs`, the result
  fold/reject tails) — each gains a counter increment and the log
  lines gain a JSON shape (`level`, `event`, the correlation fields —
  ADR-023's redaction rules hold: no prompt text, no credentials, no
  secret URLs in the attributes).
- [x] **ADDRESSED (verified)** — measured before→after. Before: zero
  counters, zero structured lines. After:
  `bash scripts/run_pg_tests.sh` → command_api `test result: ok. 18
  passed` — the new `the_metrics_surface_counts_match_the_records`
  test denies an authorization through the REAL API, then asserts the
  `authorization_denials` DELTA equals the denied record's row for the
  actor handle AND the `/v1/admin/metrics` surface returns the same
  value (the record-vs-surface agreement); the full guard log
  `target/pg252d_guard.log` shows 15 `test result: ok` lines (incl. the
  e2e suite) + `ALL acceptance checks passed` (the demo — the
  structured lines do not break its beat).
- [x] **NO REGRESSION** — `cargo test --all` → the 47 offline suites
  green; `cargo clippy --all --all-targets -- -D warnings` → clean;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at commit.
- [x] **FIX** — `crates/reasonbraid-server/src/telemetry.rs` (NEW:
  `Metrics`/`metrics()` on `OnceLock`, the seven counters, the
  `log_event!` macro); `src/lib.rs` (`mod telemetry;`); `src/api.rs`
  (the authorize deny + read-gate refuse arms, the replay outcome, the
  result fold tail, the dead-letter auto-quarantine, the
  `admin_metrics` handler gated on `authority_grants` HOLDING
  `tenant_admin` — a process-global surface has no single tenant);
  `src/node_channel.rs` (the lease/proof refusals, the
  `results_rejected` counter + the structured line);
  `tests/command_api.rs` (the measured test).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier,
  `docs/adr/INDEX.md` (023 stays the owner), KNOWLEDGE_MAP — same
  commit.

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

## Acceptance Checklist (PHASE-2.4.2)

The CODE change owned by this leaf: `crates/reasonbraid-server/tests/
migration_upgrade.rs` (`\.rs$`), `scripts/run_pg_tests.sh` (the suite joins
the guard) — all code paths.

- [x] **REPRODUCE / ISSUE** — §17.6: every suite migrates a FRESH
  database; the upgrade-an-EXISTING-database path — the one the section
  is about — had never run (the `.4` census).
- [x] **ROOT CAUSE (WHY + WHERE)** — sqlx's `migrate!` macro applies
  everything; the gap is a RUNTIME migrator that can apply a PREFIX. The
  fix uses `Migrator::new(path)` + a filtered `Migrator` (the public
  semver-exempt fields) to stage the pre-upgrade database, then the full
  migrator over the seeded data.
- [x] **ADDRESSED (verified)** — measured before→after. Before: no
  upgrade path. After: `bash scripts/run_pg_tests.sh` → `test result:
  ok. 1 passed` (`migration_upgrade` — the all-but-last migrations apply
  to a clean schema, the REAL API seeds the tenant + boundary rows, the
  remaining migrations apply over them, and the rows + the post-upgrade
  role-enroll behavior survive) + the full guard green (15 suites + e2e
  `2 passed` + the demo `ALL acceptance checks passed` (34 PASS, `rc=0`,
  `target/pg242b_guard.log`)).
- [x] **NO REGRESSION** — `cargo test --all` → 47 offline suites green;
  `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt
  --all -- --check` → rc=0; `make gate` → 13/13 at commit; `make book`
  builds.
- [x] **FIX** — `migration_upgrade.rs` (the prefix migrator + the real-
  API seed + the upgrade + the survival assertions); the guard list.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP — same
  commit.

## Acceptance Checklist (PHASE-2.4.1)

The CODE change owned by this leaf: `scripts/backup.sh` + `scripts/restore.sh`
(`(^|/)scripts/` + `\.sh$`), `crates/reasonbraid-server/tests/backup_restore.rs`
(`\.rs$`), `scripts/run_pg_tests.sh` (the suite joins the guard) — all code
paths.

- [x] **REPRODUCE / ISSUE** — §17.5: not one backup existed (`grep -rn
  'pg_dump|pg_basebackup|restore' scripts/ Makefile` → no matches before
  this leaf); the demo's SIGKILL beat proves durability, not restoration.
- [x] **ROOT CAUSE (WHY + WHERE)** — the recovery control is the
  RESTORE, not the dump (§17.5's last line). The fix is the exercise: a
  guard suite that runs the real pg_dump/pg_restore pair and asserts the
  restored state — a backup that is restored on every guard run is a
  control, not an artifact.
- [x] **ADDRESSED (verified)** — measured before→after. Before: no
  tooling, no exercise. After: `bash scripts/run_pg_tests.sh` →
  `test result: ok. 1 passed` (`backup_restore` — the seed → pg_dump →
  mutate → createdb → pg_restore → assert-the-pre-mutation-state →
  dropdb leg) + the full guard green (14 suites + e2e `2 passed` + the
  demo `ALL acceptance checks passed` (34 PASS, `rc=0`,
  `target/pg241f_guard.log`)).
- [x] **NO REGRESSION** — `cargo test --all` → 46 offline suites green;
  `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt
  --all -- --check` → rc=0; `make gate` → 13/13 at commit; `make book`
  builds.
- [x] **FIX** — `scripts/backup.sh` (custom-format pg_dump, dated file);
  `scripts/restore.sh` (pg_restore --clean --if-exists --exit-on-error
  into a caller-chosen database); the restore-exercise suite (the URL's
  user/host/port all ride the pg tools — the exercise's first runs
  caught each); the guard list.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, the book's
  deployment chapter, KNOWLEDGE_MAP — same commit.

## Acceptance Checklist (PHASE-2.3.3)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/api.rs`
(the `admin_usage` handler + the route), `crates/reasonbraid-cli/src/
{lib,main.rs}` (`rb inspect usage`), `crates/reasonbraid-server/tests/
command_api.rs` (the measured live test) — all code paths.

- [x] **REPRODUCE / ISSUE** — backlog 25's reconciliation slice: the
  `.1.6.1` budget surface was per-THREAD and row-level; nothing summed
  the tenant's estimates-vs-receipts picture (`grep -rn 'admin/usage'
  api.rs` → no matches before this leaf).
- [x] **ROOT CAUSE (WHY + WHERE)** — the rows already carry every fact
  (dimensions, usage, status, reason, expiry); the missing piece is the
  aggregation. The fix sums the SAME rows the budget engine enforces
  against (held = active unexpired; settled = actuals; overrun = used
  minus reserved floored; denied counted with reasons; expired holds
  count nowhere) — nothing computed or invented.
- [x] **ADDRESSED (verified)** — measured before→after. Before: no
  surface. After: `bash scripts/run_pg_tests.sh` → `test result: ok. 17
  passed` (`command_api`, +1: the MEASURED leg — the seeded rows'
  arithmetic recomputed in the test: held 2 / settled 8 / overrun 3
  calls / denied 1, the expired hold excluded, the denial reason rides
  the surface, the per-thread breakdown matches) + the full guard green
  (12 suites + e2e `2 passed` + the demo `ALL acceptance checks passed`
  (34 PASS, `rc=0`, `target/pg233_guard.log`)).
- [x] **NO REGRESSION** — `cargo test --all` → 45 offline suites green;
  `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt
  --all -- --check` → rc=0; `make gate` → 13/13 at commit; `make book`
  builds.
- [x] **FIX** — `api.rs` (the handler + the route); the CLI verb; the
  measured live test.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, the book's cli
  chapter, KNOWLEDGE_MAP — same commit.

## Acceptance Checklist (PHASE-2.3.2)

The CODE change owned by this leaf: migration `0016_spend_breakers.sql`,
`crates/reasonbraid-server/src/{budget.rs,api.rs}` (the breaker check + the
arm/reset/inspect verbs), `crates/reasonbraid-cli/src/{lib,main.rs}` (`rb
breaker arm|reset` + `rb inspect breakers`), `crates/reasonbraid-server/
tests/budget.rs` (the two live legs + the suite's purge/seed), the seven
tenant-purging suites' `spend_breakers` purge row — all code paths.

- [x] **REPRODUCE / ISSUE** — backlog 23: the ceiling refused only
  per-reservation; `grep -rn 'circuit' crates/ migrations/` → no matches
  before this leaf — nothing stopped NEW dispatches once a tenant's
  recorded spend crossed a declared threshold.
- [x] **ROOT CAUSE (WHY + WHERE)** — the ceiling is THREAD-scoped and
  checked per-reservation; a tenant-level latch needs a tenant-level fact
  + a check at the same reservation boundary. The fix: a `spend_breakers`
  row (threshold + tripped state), checked FIRST in the reservation
  transaction — the trip and the refusal commit with the denial's own
  transaction, so the latch never lags the ledger it guards.
- [x] **ADDRESSED (verified)** — measured before→after. Before: no
  breaker. After: `bash scripts/run_pg_tests.sh` → `test result: ok. 9
  passed` (`budget`, +2: the crossing trips + refuses with the typed
  reason + the latch refuses while tripped + the reset re-opens; the
  reset breaker re-trips on the next crossing) + the full guard green (12
  suites + e2e `2 passed` + the demo `ALL acceptance checks passed` (34
  PASS, `rc=0`, `target/pg232c_guard.log`)).
- [x] **NO REGRESSION** — `cargo test --all` → 45 offline suites green;
  `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt
  --all -- --check` → rc=0; `make gate` → 13/13 at commit; `make book`
  builds. (The FIRST guard re-run caught the FK leak: `spend_breakers`
  references tenants, so every tenant-purging suite's list gained the
  row — the suite-ownership doctrine, not a code bug.)
- [x] **FIX** — migration 0016; `budget.rs`
  (`check_spend_breaker_in_tx`: the tripped-latch refusal + the
  crossing-trip in the caller's transaction); `api.rs` (the three admin
  verbs + routes); the CLI (arm/reset/inspect); the live tests; the
  purge rows.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, the book's cli
  chapter, KNOWLEDGE_MAP — same commit.

## Acceptance Checklist (PHASE-2.2.4)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/api.rs`
(the dead-letter auto-quarantine arm + the replay endpoint + the route),
`crates/reasonbraid-node/src/{worker.rs,journal.rs}` (the once-only
best-effort report + the replayed-decision refresh + the dedup accessor),
`crates/reasonbraid-cli/src/{lib,main.rs}` (`rb node replay`),
`crates/reasonbraid-node/tests/worker_dead_letter.rs`,
`crates/reasonbraid-server/tests/node_work.rs` — all code paths.

- [x] **REPRODUCE / ISSUE** — quarantine was one-way: the `.1.2.3`
  verbs marked + pruned, nothing auto-quarantined after the `.2.3`
  terminal refusals and nothing re-delivered a dead-lettered command
  (`grep -rn 'replay' api.rs` → no verb before this leaf).
- [x] **ROOT CAUSE (WHY + WHERE)** — the terminal refusal is a NODE-side
  fact (the retry gate) the server never sees, and the quarantine is a
  SERVER-side row the node never touches. The fix rides the existing
  event channel: the refusal reports a `work_dead_lettered` event (once,
  best-effort), the server's result path auto-quarantines in the SAME
  transaction as the receipt, and the replay verb reverses it by
  REFRESHING the admission decision (decided_at + the current epoch) and
  re-sequencing the row — the node's decision refresh + the
  decision-scoped retry count re-arm the dispatch.
- [x] **ADDRESSED (verified)** — measured before→after. Before: no
  report, no replay, one-way quarantine. After: `bash
  scripts/run_pg_tests.sh` → `test result: ok. 8 passed` (`node_work`,
  +1: the LIVE end-to-end leg — the always-refusing worker dead-letters
  after the bounded retries, the server auto-quarantines with the
  terminal reason, the operator replay clears + refreshes + re-sequences,
  the completing worker re-dispatches — exactly one contribution) + the
  full guard green (12 suites + e2e `2 passed` + the demo
  `ALL acceptance checks passed` (34 PASS, `rc=0`,
  `target/pg224b_guard.log`)); `cargo test -p reasonbraid-node --test
  worker_dead_letter` → `test result: ok. 2 passed` (the once-only
  report + the fresh-decision re-arm).
- [x] **NO REGRESSION** — `cargo test --all` → 45 offline suites green;
  `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt
  --all -- --check` → rc=0; `make gate` → 13/13 at commit; `make book`
  builds.
- [x] **FIX** — `api.rs` (the `work_dead_lettered` arm + `replay_command`
  + the route); `worker.rs` (`report_dead_letter` — journaled first,
  best-effort send, the dedup); `journal.rs` (`has_dead_letter` + the
  replayed-decision refresh in `record_command`); the CLI verb; the
  offline + live tests.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, the book's cli
  chapter, KNOWLEDGE_MAP — same commit.

## Acceptance Checklist (PHASE-2.2.3)

The CODE change owned by this leaf: `crates/reasonbraid-core/src/{retry.rs,
lib.rs}` (the pure decision + exports), `crates/reasonbraid-server/src/
threads.rs` (the typed wire flag), `crates/reasonbraid-node/src/worker.rs`
(the retry gate), `crates/reasonbraid-node/tests/worker_retry_policy.rs` —
all code paths.

- [x] **REPRODUCE / ISSUE** — the worker's skip decision was binary:
  `None | prepared` re-dispatch, everything else skip — the §14.6 classes
  had no expression, the budget denial and the transient refusal shared
  one status, and `retry_requires_authorization` existed as a code but
  was wired nowhere.
- [x] **ROOT CAUSE (WHY + WHERE)** — the classes are distinguishable by
  facts the worker already holds: the reservation's presence (a budget
  denial means the server refused — retrying cannot change it) and the
  delivery's duplicate flag (a risky re-run must be AUTHORIZED). The fix
  is the pure decision over those facts, evaluated before any gate.
- [x] **ADDRESSED (verified)** — measured before→after. Before: no
  decision, no flag. After: `cargo test -p reasonbraid-core` →
  `test result: ok. 49 passed` (the five retry tests: the boundary-
  never-crossed rule, the terminal budget denial at ANY count, the
  bounded reserved retry, the authorization-required ambiguity, the
  terminal states); `cargo test -p reasonbraid-node --test
  worker_retry_policy` → `test result: ok. 4 passed` (a reserved refusal
  re-dispatches and reaches the adapter; the budget denial stays one
  attempt; the ambiguous refusal stays one attempt; the authorized
  ambiguous re-dispatch runs).
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → all twelve
  live server suites green + CLI e2e `2 passed` + the demo
  `ALL acceptance checks passed` (34 PASS — the budget-denied beat stays
  `failed_before_dispatch=1`: the denial is terminal by design, `rc=0`,
  `target/pg223_guard.log`); `cargo test --all` → 44 offline suites
  green; `cargo clippy --all --all-targets -- -D warnings` → clean;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at commit.
- [x] **FIX** — `retry.rs` (the decision + the five tests); the typed
  `allow_possible_duplicate` wire flag; the worker's retry gate (the
  payload facts first, the attempt count from the `.1.6.2` accessor, the
  refusal log naming the reason); the worker-level tests.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP — same
  commit.

## Acceptance Checklist (PHASE-2.2.2)

The CODE change owned by this leaf: migration `0015_lease_epoch.sql`,
`crates/reasonbraid-server/src/node_channel.rs` (the epoch on the four DTOs +
`issue_lease`/`renew_lease`/`verify_fencing` + the in-tx verifier + the
handlers), `crates/reasonbraid-node/src/channel.rs` (the epoch beside the
token), `crates/reasonbraid-server/tests/node_channel.rs` +
`tests/node_inbox.rs` + `tests/node_work.rs` (the wire bodies), the demo's
probes, `docs/book/src/node-channel.md` — all code paths.

- [x] **REPRODUCE / ISSUE** — the renewal race: `renew_lease`'s UPDATE had no
  token/epoch guard (`grep -n renew_lease node_channel.rs` → the WHERE was
  `node_id` only), so a stale heartbeat that verified before a concurrent
  handshake extended the NEW session's lease after it landed; the fencing
  check ran at admission, outside the events transaction.
- [x] **ROOT CAUSE (WHY + WHERE)** — the token is checked but never carried
  into the write, and the check and the apply are separate moments. The fix
  is the lease EPOCH: the handshake bumps it with the token, every fenced
  write carries the pair it saw, the renewal's WHERE requires the pair (0
  rows = the race lost), and the events transaction re-verifies the pair FOR
  UPDATE (a rotation between admission and apply is observed).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no epoch,
  the unguarded renewal. After: `bash scripts/run_pg_tests.sh` →
  `test result: ok. 22 passed` (`node_channel`, +1: the deterministic
  state-level race — rotation bumps the epoch, a renewal from the fenced
  epoch matches no row, a stale epoch with the CURRENT token is refused,
  the in-tx verifier refuses the stale pair) + the wire test re-proves
  every fenced surface carries the epoch + the full guard green (12 suites
  + e2e `2 passed` + the demo `ALL acceptance checks passed` (34 PASS,
  `rc=0`, `target/pg222c_guard.log`)).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green;
  `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt
  --all -- --check` → rc=0; `make gate` → 13/13 at commit; `make book`
  builds.
- [x] **FIX** — migration 0015; the epoch through the channel DTOs + the
  lease methods + the handlers + the in-tx verifier; the node channel's
  epoch storage; the wire bodies; the demo probes; the book's lease
  passages + CHANNEL_VERSION 5.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, the book,
  KNOWLEDGE_MAP — same commit.

## Acceptance Checklist (PHASE-2.1.6.1)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/
{node_channel.rs,api.rs}` (the enroll request/response + the incarnation
writer + the admin inspection route), `crates/reasonbraid-node/src/
{channel.rs,bin/rb-node.rs}` (the `enroll_with_facts` client + the flags),
`crates/reasonbraid-cli/src/{lib,main.rs}` (`run_inspect_incarnations` +
the verb), `crates/reasonbraid-server/tests/node_enrollment.rs`,
`scripts/demo_two_host.sh`, `docs/book/src/cli.md` — all code paths.

- [x] **REPRODUCE / ISSUE** — deferral #4's first half: the 0007
  hierarchy is schema-only — `grep -rn 'INSERT INTO incarnations'
  crates/` → no matches before this leaf — while the node KNOWS its
  harness at start (`rb-node` builds the adapter from its flags) and the
  enroll request carries none of the §8.1 facts.
- [x] **ROOT CAUSE (WHY + WHERE)** — the enrollment boundary discarded
  the facts it alone sees (the node declares them once, at start). The
  fix point is the enroll request/transaction: the facts ride the body,
  the writer lands in the SAME transaction as the node/key/cert rows (a
  refused enrollment writes nothing), and the role-vs-plain-node split
  falls out of the schema (the hierarchy binds incarnations to roles).
- [x] **ADDRESSED (verified)** — measured before→after. Before: no
  writer, no request fields, no inspection. After: `bash
  scripts/run_pg_tests.sh` → `test result: ok. 5 passed`
  (`node_enrollment`, +1: the role node's enrollment returns the branded
  `incarnation_id`, the row carries the declared facts verbatim, the
  tenant_admin list shows it, the consumed-token re-enrollment refuses
  and duplicates NOTHING, a plain `nod_…` node records no incarnation) +
  the full guard green (12 suites + e2e + the demo `ALL acceptance checks
  passed` — 33 checks, `rc=0`, `target/pg161c_guard.log`).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green;
  `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt
  --all -- --check` → rc=0; `make gate` → 13/13 at commit; `make book`
  builds.
- [x] **FIX** — `node_channel.rs` (the §8.1 request fields, the
  incarnation writer in the enroll transaction, the response's
  `incarnation_id`); `api.rs` (`GET /v1/admin/incarnations` +
  the route); the node's `enroll_with_facts` client; `rb-node`'s four
  flags; the CLI verb + the `rb inspect incarnations` text; the live
  test; the demo beat; the book's cli chapter.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, the book,
  KNOWLEDGE_MAP — same commit.

## Acceptance Checklist (PHASE-2.1.6.2)

The CODE change owned by this leaf: migration `0014_run_attempt_link.sql`,
`crates/reasonbraid-server/src/api.rs` (the run writer + `list_runs` +
the route), `crates/reasonbraid-cli/src/{lib,main.rs}`
(`run_inspect_runs` + the verb), `crates/reasonbraid-server/tests/
node_work.rs` (the result-fold test's three legs),
`scripts/demo_two_host.sh`, `docs/book/src/cli.md` — all code paths.

- [x] **REPRODUCE / ISSUE** — deferral #4's second half: `grep -rn
  'INSERT INTO runs' crates/` → no matches before this leaf — a result
  receipt folded its contribution and its reservation, but the
  incarnation→attempt linkage the §8.1 hierarchy promises did not exist.
- [x] **ROOT CAUSE (WHY + WHERE)** — the linkage facts are split across
  two boundaries: the incarnation is server-side (`.1.6.1`), the attempt
  is a NODE-local journal fact that only rides the result payload. The
  fix point is the result fold — the ONE server transaction that already
  sees the attempt id (the payload) and the current incarnation (the
  table): write the run row there, AFTER the idempotency claim, so
  redelivery can never duplicate it.
- [x] **ADDRESSED (verified)** — measured before→after. Before: no runs
  row, no attempt link. After: `bash scripts/run_pg_tests.sh` →
  `test result: ok. 7 passed` (`node_work` — the result-fold test gained
  the legs: the run row links the payload's attempt to the role's current
  incarnation, the tenant_admin list shows the chain, and the two
  duplicate transports write NO second run) + the full guard green (12
  suites + e2e `2 passed` + the demo `ALL acceptance checks passed` — 34
  checks, `rc=0`, `target/pg162_guard.log`).
- [x] **NO REGRESSION** — `cargo test --all` → every offline suite green;
  `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt
  --all -- --check` → rc=0; `make gate` → 13/13 at commit; `make book`
  builds.
- [x] **FIX** — `migrations/0014_run_attempt_link.sql` (the attempt
  column); `api.rs` (the writer in `apply_node_result_in_tx` + the
  `list_runs` inspection + the route); the CLI verb; the result-fold
  test's legs; the demo beat; the book's cli chapter.
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
| `2026-09-07` | `PHASE-2.3.1` | docs-only (no code paths changed): `make gate` → 13/13 at commit | ADR-012 + ADR-013 accepted (the shipped ambiguity + budget machinery promotes); frontier → `.3.2` |
| `2026-09-07` | `PHASE-2.5.1` | docs-only (no code paths changed): `make gate` → 13/13 at commit | ADR-023 accepted (the four-record separation + the redaction rules + the sink trigger); frontier → `.5.2` |
| `2026-09-07` | `PHASE-2.7.4` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the subtraction record + the G6–G7 feed — **Phase 2 is CLOSED** |
| `2026-09-07` | `PHASE-2.7.3` | docs-only (no code paths changed): `make gate` → 13/13 at commit | ADR-022 accepted (the linkage groundwork + the named chain trigger) + the retry-safety inventory (six measured legs); frontier → `.7.4` |
| `2026-09-07` | `PHASE-2.7.2` | `DATABASE_URL=… cargo test -p reasonbraid-server --test node_replacement` → `test result: ok. 1 passed` (the measured ritual); `bash scripts/run_pg_tests.sh` → 17 live suites + the demo 34/34 (`target/pg272_guard.log`); `cargo test --all` → 50 offline suites; clippy/fmt clean; `make gate` → 13/13 | the node-replacement drill (the ritual measured end to end; the epoch fence held); frontier → `.7.3` |
| `2026-09-07` | `PHASE-2.7.1` | `DATABASE_URL=… cargo test -p reasonbraid-server --test escalation` → `test result: ok. 4 passed` (the first live run caught four real behaviors the tests pinned); `bash scripts/run_pg_tests.sh` → 16 live suites + the demo 34/34 (`target/pg271b_guard.log`); `cargo test --all` → 49 offline suites; clippy/fmt clean; `make gate` → 13/13 | the non-escalation property suite (one named adversarial test per escalation surface); frontier → `.7.2` |
| `2026-09-07` | `PHASE-2.7` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the exit-lane census + the contract-seam decomposition (the foundations vs the four gaps); frontier → `.7.1` |
| `2026-09-07` | `PHASE-2.6.3` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the qualification checklist + the deferrals record (each of the five §19.4 items names its trigger); **`.6` COMPLETE** — frontier → `.7` |
| `2026-09-07` | `PHASE-2.6.2` | `cargo test -p reasonbraid-adapter --lib` → `test result: ok. 9 passed` (the two manifest guarantees); `cargo test --all` → 48 offline suites; `bash scripts/run_pg_tests.sh` → 15 live suites + the demo 34/34 (`target/pg262_guard.log`); clippy/fmt clean; `make gate` → 13/13 | the permanent failure-fixture corpus (the versioned manifest + the exact-match drift check + the item-coverage check); frontier → `.6.3` |
| `2026-09-07` | `PHASE-2.6.1` | `cargo test -p reasonbraid-adapter --test adapter_conformance` → `test result: ok. 3 passed` (the fake + codex + claude all pass the ONE harness); `cargo test --all` → 48 offline suites; `bash scripts/run_pg_tests.sh` → 15 live suites + the demo 34/34 (`target/pg261_guard.log`); clippy/fmt clean; `make gate` → 13/13 | the conformance harness (the six cross-adapter invariant checks over the scenario shape); frontier → `.6.2` |
| `2026-09-07` | `PHASE-2.6` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the §19.4 census + the contract-seam decomposition (`.6.1` harness → `.6.2` corpus → `.6.3` checklist + deferrals); frontier → `.6.1` |
| `2026-09-07` | `PHASE-2.5.3` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the SLO record + the node lost/replaced runbook (the guard is the population; the closure tests name the existing exercises); frontier → `.6` |
| `2026-09-07` | `PHASE-2.5.2` | `bash scripts/run_pg_tests.sh` → fifteen live server suites green (`test result: ok.` 4 + 5 + 9 + 1 + 7 + 18 + 3 + 4 + 1 + 22 + 5 + 3 + 8 + 7 + 2 `passed` — `command_api` grew to 18 with the measured metrics leg: the denied authorization's counter DELTA matches the denied record for the actor handle AND the `/v1/admin/metrics` surface agrees) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (34 PASS, `rc=0`, `target/pg252d_guard.log`); `cargo test --all` → 47 offline suites; clippy/fmt clean; `make gate` → 13/13 | the structured-log + metrics slice (the seven counters on the real paths, the `log_event!` JSON lines, the admin metrics surface); frontier → `.5.3` |
 docs-only (no code paths changed): `make gate` → 13/13 at commit | the inventory-groundwork deferral record (the absent controls named with their triggers); **`.4` COMPLETE** — frontier → `.5` |
 `bash scripts/run_pg_tests.sh` → fifteen live server suites green (`test result: ok.` 4 + 5 + 9 + 1 + 7 + 17 + 3 + 4 + 1 + 22 + 5 + 3 + 8 + 7 + 2 `passed` — the new `migration_upgrade` suite: the all-but-last migrations + the real-API seed + the upgrade + the survival assertions) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (34 PASS, `rc=0`, `target/pg242b_guard.log`); `cargo test --all` → 47 offline suites; clippy/fmt clean; `make gate` → 13/13 | the migration upgrade test (the existing-database path, measured); frontier → `.4.3` |
 `bash scripts/run_pg_tests.sh` → fourteen live server suites green (`test result: ok.` 4 + 5 + 9 + 1 + 7 + 17 + 3 + 4 + 22 + 5 + 3 + 8 + 7 + 2 `passed` — the new `backup_restore` suite: seed → pg_dump → mutate → createdb → pg_restore → assert the pre-mutation state → dropdb) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (34 PASS, `rc=0`, `target/pg241f_guard.log`); `cargo test --all` → 46 offline suites; clippy/fmt clean; `make gate` → 13/13 | the backup + restore automation (the restore EXERCISE is the recovery control); frontier → `.4.2` |
 `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 9 + 17 + 3 + 4 + 22 + 5 + 3 + 8 + 7 `passed` — `command_api` grew to 17 with the measured reconciliation leg) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (34 PASS, `rc=0`, `target/pg233_guard.log`); `cargo test --all` → 45 offline suites; clippy/fmt clean; `make gate` → 13/13 | the usage-reconciliation surface (`GET /v1/admin/usage` + `rb inspect usage` — the summed held/settled/overrun/denied picture); **`.3` COMPLETE** — frontier → `.4` |
 `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 9 + 16 + 3 + 4 + 22 + 5 + 3 + 8 + 7 `passed` — `budget` grew to 9 with the two breaker legs) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (34 PASS, `rc=0`, `target/pg232c_guard.log`); `cargo test --all` → 45 offline suites; clippy/fmt clean; `make gate` → 13/13 | the spend circuit breakers (migration 0016 + the in-tx latch + the arm/reset/inspect verbs + the CLI); frontier → `.3.3` |
 `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 16 + 3 + 4 + 22 + 5 + 3 + 8 + 7 `passed` — `node_work` grew to 8 with the live dead-letter/replay leg) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (34 PASS, `rc=0`, `target/pg224b_guard.log`); `cargo test -p reasonbraid-node --test worker_dead_letter` → `test result: ok. 2 passed`; `cargo test --all` → 45 offline suites; clippy/fmt clean; `make gate` → 13/13 | the two-way quarantine (the once-only dead-letter report + the server's auto-quarantine + `POST /v1/nodes/replay` + `rb node replay` + the decision-scoped retry re-arm); **`.2` COMPLETE** — frontier → `.3` |
 `cargo test -p reasonbraid-core` → `test result: ok. 49 passed` (the five retry tests); `cargo test -p reasonbraid-node --test worker_retry_policy` → `test result: ok. 4 passed`; `bash scripts/run_pg_tests.sh` → all twelve live suites + e2e + the demo 34 PASS (`rc=0`, `target/pg223_guard.log`); `cargo test --all` → 44 offline suites; clippy/fmt clean; `make gate` → 13/13 | the retry policy (the pure §14.6 decision + the typed wire flag + the worker's retry gate); frontier → `.2.4` |
 `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 16 + 3 + 4 + 22 + 5 + 3 + 7 + 7 `passed` — `node_channel` grew to 22 with the deterministic renewal-race test) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (34 PASS, `rc=0`, `target/pg222c_guard.log`); `cargo test --all` → every offline suite green; clippy/fmt clean; `make gate` → 13/13 | the lease epoch (migration 0015 + CHANNEL_VERSION 5): the token AND the epoch fence the old session, a stale-epoch renewal matches no row, the in-tx verifier closes the check-vs-commit window; frontier → `.2.3` |
 docs-only (no code paths changed): `make gate` → 13/13 at commit | ADR-005 accepted (the PostgreSQL queue — evidence-gated); frontier → `.2.2` |
 `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 16 + 3 + 4 + 21 + 5 + 3 + 7 + 7 `passed` — the result-fold test gained the run-writer legs) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (34 PASS, `rc=0`, `target/pg162_guard.log`); `cargo test --all` → every offline suite green; clippy/fmt clean; `make gate` → 13/13 | the run writer (the result receipt's attempt→incarnation link + the inspection chain); **`.1.6` complete — deferral #4 closes — `.1` COMPLETE**; frontier → `.2` |
 `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 16 + 3 + 4 + 21 + 5 + 3 + 7 + 7 `passed` — `node_enrollment` grew to 5 with the incarnation test) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (33 PASS, `rc=0`, `target/pg161c_guard.log`); `cargo test --all` → every offline suite green; clippy/fmt clean; `make gate` → 13/13 | the incarnation writer (the §8.1 request facts + the enroll transaction's row + the inspection surface + `rb-node`'s flags + the demo beat); frontier → `.1.6.2` |
 `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 16 + 3 + 4 + 21 + 4 + 3 + 7 + 7 `passed` — `node_work` grew to 7 with the measured live leg: the REAL node worker completes the fresh allow, the revocation bumps the epoch 0→1, the next dispatch refuses without a re-ask) + CLI e2e `2 passed` + the demo `ALL acceptance checks passed` (32 PASS, `rc=0`, `target/pg152b_guard.log`); `cargo test -p reasonbraid-node --test worker_cached_decision` → `test result: ok. 5 passed`; `cargo test --all` → 43 offline suites green; `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 | the cached-decision machinery (migration 0013 + the epoch-in-transaction + the delivery-carried decision + CHANNEL_VERSION 4 + the node-side dispatch gate); **`.1.5` complete** — frontier → `.1.6` |

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
| `PHASE-2.7.4` | `REASONBRAID-PHASE2-0043` | the Phase-2 subtraction record (S-1…S-13) + the G6–G7 feed — **Phase 2 is CLOSED** (docs-only) |
| `PHASE-2.7.3` | `REASONBRAID-PHASE2-0042` | ADR-022 (the audit-linkage groundwork, the chain deferred with its trigger) + the six-leg no-false-safe-retry inventory (docs-only) |
| `PHASE-2.7.2` | `REASONBRAID-PHASE2-0041` | the node-replacement drill (the measured ritual + the replacement enroll path + migrations 0017/0018 + the runbook ritual) |
| `PHASE-2.7.1` | `REASONBRAID-PHASE2-0040` | the non-escalation property suite (4 measured adversarial tests + the guard's live list gained the suite) |
| `PHASE-2.7` | `REASONBRAID-PHASE2-0039` | the exit-lane census + the contract-seam decomposition (`.7.1` adversarial suite → `.7.2` replacement drill → `.7.3` ADR-022 + retry inventory → `.7.4` subtraction + gate feed) |
| `PHASE-2.6.3` | `REASONBRAID-PHASE2-0038` | the qualification checklist + the named deferrals (docs-only: the book's six-box gate, the five triggers) — **`.6` COMPLETE** |
| `PHASE-2.6.2` | `REASONBRAID-PHASE2-0037` | the permanent failure-fixture corpus (the versioned manifest + the exact-match + item-coverage guarantees) |
| `PHASE-2.6.1` | `REASONBRAID-PHASE2-0036` | the conformance harness (one suite, three adapters, the six §19.4 invariant checks; the shared provider stubs) |
| `PHASE-2.6` | `REASONBRAID-PHASE2-0035` | the contract-seam decomposition (the §19.4 census: what the adapter surface already carries vs the five absent items — named) |
| `PHASE-2.5.3` | `REASONBRAID-PHASE2-0034` | the SLO record + the node lost/replaced runbook (docs-only: the guard is the population, the zero-error-budget halt rule, the §18.6 runbook shape) |
| `PHASE-2.5.2` | `REASONBRAID-PHASE2-0033` | the structured-log + metrics slice (the seven counters on the real paths + `GET /v1/admin/metrics` + the measured denial-vs-record test) |
| `PHASE-2.5.1` | `REASONBRAID-PHASE2-0032` | ADR-023 accepted (the four-record separation + the §18.2 redaction rules pinning the future sink; the OpenTelemetry trigger named — no code) |
| `PHASE-2.5` | `REASONBRAID-PHASE2-0031` | the contract-seam split (eprintln-only observability; the four-record doctrine structurally true, nothing measures) |
| `PHASE-2.4.3` | `REASONBRAID-PHASE2-0030` | the inventory-groundwork deferral record (the absent §17.5/§17.6 controls named with their triggers — no placeholder infrastructure) — **`.4` COMPLETE** |
| `PHASE-2.4.2` | `REASONBRAID-PHASE2-0029` | the migration upgrade test (the N-1 → N path: the real API seeds, the remaining migrations apply over the existing data, the rows + behavior survive — measured) |
| `PHASE-2.4.1` | `REASONBRAID-PHASE2-0028` | the backup + restore automation: `scripts/backup.sh`/`restore.sh` + the guard's restore exercise (seed → dump → mutate → restore → assert the pre-mutation state) |
| `PHASE-2.4` | `REASONBRAID-PHASE2-0027` | the contract-seam split (no backup tooling, the upgrade path unexercised, the inventory has nothing to bind) |
| `PHASE-2.3.3` | `REASONBRAID-PHASE2-0026` | the usage-reconciliation surface (`GET /v1/admin/usage` + `rb inspect usage` — the summed held/settled/overrun/denied picture, measured against the ledger rows) — **`.3` COMPLETE** |
| `PHASE-2.3.2` | `REASONBRAID-PHASE2-0025` | the spend circuit breakers: migration 0016 + the in-tx latch (tripped refuses everything new, the crossing trips with the denial's transaction) + the arm/reset/inspect verbs + the CLI |
| `PHASE-2.3.1` | `REASONBRAID-PHASE2-0024` | ADR-012 + ADR-013 accepted (the shipped ambiguity + budget machinery promotes — no code; the pricing-snapshot trigger named) |
| `PHASE-2.3` | `REASONBRAID-PHASE2-0023` | the contract-seam split (the state machine + settlement exist; circuit breakers, the reconciliation surface, ADR-012/013 open) |
| `PHASE-2.2.4` | `REASONBRAID-PHASE2-0022` | the two-way quarantine: the once-only best-effort dead-letter report, the server's auto-quarantine in the receipt transaction, `POST /v1/nodes/replay` + `rb node replay` (the decision refreshes + the row re-sequences), the replayed-decision refresh + the decision-scoped retry count — **`.2` COMPLETE** |
| `PHASE-2.2.3` | `REASONBRAID-PHASE2-0021` | the retry policy: the pure `retry_decision` (§14.6 classes) + the `allow_possible_duplicate` wire flag + the worker's retry gate (attempt-counted, budget-denials terminal, ambiguity authorization-required) |
| `PHASE-2.2.2` | `REASONBRAID-PHASE2-0020` | the lease epoch: migration 0015 + CHANNEL_VERSION 5 — every fenced write carries the epoch it saw, a stale-epoch renewal loses the race, the events transaction re-verifies FOR UPDATE |
| `PHASE-2.2.1` | `REASONBRAID-PHASE2-0019` | ADR-005 accepted (the PostgreSQL queue — evidence-gated; the WP2 outbox worker + ADR-004/006 promote; no code changes) |
| `PHASE-2.2` | `REASONBRAID-PHASE2-0018` | the contract-seam split (the Phase-1 lease/fencing + quarantine exist; retry policy, dead-letter/replay, ADR-005 open) |
| `PHASE-2.1.6.2` | `REASONBRAID-PHASE2-0017` | the run writer: migration 0014 + the result fold's run row (after the idempotency claim — one result = one run) + `GET /v1/admin/runs` + `rb inspect runs` + the demo beat — **`.1.6` complete, deferral #4 closes, `.1` COMPLETE** |
| `PHASE-2.1.6.1` | `REASONBRAID-PHASE2-0016` | the incarnation writer: the enroll request's §8.1 facts → the `incarnations` row (role nodes only) + `GET /v1/admin/incarnations` + `rb inspect incarnations` + `rb-node`'s four flags + the demo beat |
| `PHASE-2.1.6` | `REASONBRAID-PHASE2-0015` | the incarnation-vs-run split (the 0007 hierarchy is schema-only — deferral #4; the enroll request carries no §8.1 facts) |
| `PHASE-2.1.5.2` | `REASONBRAID-PHASE2-0014` | the cached-decision machinery: migration 0013 (the tenant epoch + the inbox decision columns), the revocation writes bump the epoch in-transaction, the delivery-carried admission decision + CHANNEL_VERSION 4, the node journal's cached decision + the dispatch gate (refuses stale/denied/absent — journaled, adapter never invoked); the measured live revocation-invalidation leg — **`.1.5` complete** |
