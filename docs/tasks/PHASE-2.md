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
    Children: `.1.2.1`–`.1.2.2`.

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
    Status: `proposed`
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

  - ID: `PHASE-2.1.3`
    Status: `proposed`
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

  - ID: `PHASE-2.1.4`
    Status: `proposed`
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

  - ID: `PHASE-2.1.5`
    Status: `proposed`
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
| 1 | `PHASE-2.1.2.2` | `proposed` | `.1.2.1` done (enroll issues + the node stores the leaf; the CA survives restarts — the rebuild test proves it); the channel v3 cert-proof handshake + rotation execute now |

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
| `2026-09-07` | `PHASE-2.1.2.1` | `bash scripts/run_pg_tests.sh` → all twelve live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 13 + 3 + 4 + 17 + 4 + 3 + 6 + 7 `passed` — `node_enrollment` grew to 4 with the CA-persistence test) + CLI e2e `2 passed` + the two-host demo `ALL acceptance checks passed` (30 PASS, `rc=0`, `target/pg121b_guard.log`); `cargo test --all` → 42 offline suites green (rc=0, `target/pg121_offline.log`); `cargo clippy --all --all-targets -- -D warnings` → clean; `cargo fmt --all -- --check` → rc=0; `make deny` → rc=0 (rcgen's `x509-parser` feature entered the server graph without a ban); `make gate` → 13/13 | cert issuance at enrollment: the persisted `ServerCa` (generated on first boot, loaded thereafter — the rebuild test proves the same key + cert), the enroll response carries the leaf + dev-escrowed key + fingerprint, `rb-node` stores `cert.der`/`key.der`; the HMAC channel untouched (the coherent interim) — frontier → `.1.2.2` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-2.1` | `REASONBRAID-PHASE2-0001` | the census-seam decomposition (six tool-backed gaps → `.1.1`–`.1.6`) |
| `PHASE-2.1.1` | `REASONBRAID-PHASE2-0002` | the ADR-006/007 spike + records: the test-only experiment crate, the two accepted ADRs, the ledger row; `make deny`'s ban caught the base64 split — fixed by dropping rcgen's unused `pem` feature |
| `PHASE-2.1.2` | `REASONBRAID-PHASE2-0003` | the issuance-vs-channel split (the `.1.2.1`-first precedent): `.1.2.1` cert issuance at enrollment → `.1.2.2` the channel v3 swap |
| `PHASE-2.1.2.1` | `REASONBRAID-PHASE2-0004` | cert issuance at enrollment: migration 0011 + `ca.rs` (the persisted CA) + the enroll response's cert + escrowed key + the node's `cert.der`/`key.der` persistence; the HMAC channel untouched |
