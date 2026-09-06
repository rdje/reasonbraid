# 2026-09-06_node-channel-auth.md

## Context

`PHASE-1.2.2` (backlog 13's remainder) landed the authenticated node channel:
the handshake carries a key-proof, heartbeats renew a server-side lease, expiry
leaves the node observably `offline`, and a fencing token guards renewal. The
existing channel (`.3.2`, proven by Phase 0) moved wholesale to the new
contract — `CHANNEL_VERSION` is now **2**.

## Decision

- **The handshake proves the key, and the fence guards everything after.**
  The proof is HMAC-SHA256 over the canonical channel fields (a mirrored
  `ProofCoverage` struct serialized as JSON on both sides — the struct shape
  IS the canonicalization contract), keyed with the `.1.2.1` dev secret. The
  server verifies in constant time and refuses `401 unauthorized` BEFORE any
  ledger fact is read; a missing key and a wrong proof fail identically (no
  existence leak). A missing `key_proof` field is a malformed request (422) at
  the strict wire boundary — distinct from a wrong proof (401).
- **A successful handshake issues a lease with a FRESH fencing token**
  (`fnc_<uuid>`, generated in PostgreSQL in the same statement that writes the
  row): the only token that renews the lease or guards `events`/`ack`/`poll`.
  Every handshake rotates it, so a stale process is fenced the moment a newer
  handshake lands — staleness is visible, never silently accepted.
- **Expiry is a database fact, presence is derived, never a stored flag.**
  `lease_expires_at` (60 s dev TTL) + `node_presence` (a VIEW computing
  `online` from the clock): a crashed process cannot leave a stale `online`
  row. A heartbeat renews only a LIVE lease; once expired, channel traffic is
  refused until a NEW key-proof (a fresh handshake) restores it.
- **`poll` became a POST.** The fencing token is a credential and never rides a
  query string (access logs).
- **The dev node-id space is `nod_…` OR the `rol_…` role wire id** (the dev
  wiring collapses node == role). The `.1.2.1` surfaces accepted only `NodeId`;
  the authenticated handshake looks up `node_keys` for whatever id the node
  reports, so issuance + enrollment now accept both (a superset — the `.1.2.1`
  suites never asserted `nod`-only).

## Consequences

- The channel is authenticated end to end on the dev trust-store stance
  (`.6.1`); mTLS/X.509 stays ADR-006/ADR-007 (Phase 2).
- The node client keeps the fencing token in shared state: the worker's poll,
  the heartbeat task, and `reconcile`'s rotation all observe one lease. The
  `rb-node` process runs a 15 s heartbeat loop; a refused heartbeat only logs
  because the poll failure path re-handshakes.
- Migration 0009 (`node_leases` + `node_presence`) joined every suite's purge
  list in FK order; the demo enrolls its nodes, re-POSTs the duplicate with the
  live fencing token, probes polls with the CURRENT token (re-read each
  attempt — the restart re-handshake rotates it), and asserts observable
  presence before and after the server restart.
- Two live processes for one node id fence each other by design — that is the
  contract, not a bug (documented in the book).

answers:

- **A missing field and a wrong credential are different refusals.** `serde`
  `deny_unknown_fields` + required fields reject a MISSING `key_proof`/
  `fencing_token` as 422 before the handler runs; a WRONG value reaches the
  verifier and gets 401. Tests must assert both, or the wire contract is
  under-specified.
- **Fencing tokens rotate only at the handshake; renewal never rotates.**
  Rotation and renewal are different operations with different authority: the
  handshake owns identity re-proof, the heartbeat owns liveness. Conflating
  them would let a stolen heartbeat credential escalate.
- **Cross-side crypto must be mirrored, not shared.** The server and node each
  serialize their own `ProofCoverage`; a test computing proofs with the node's
  public `compute_key_proof` against the server's verifier catches any drift
  between the two canonicalizations — a shared wire crate would have hidden it.
- **Test closures that build async requests must own their captures.** A
  closure returning an `async move` block moves captured locals into the future
  (making the closure `FnOnce`); clone owned values inside the non-move
  closure, and prefer owned parameters over borrowed ones in the signature.
- **Suite purge lists are all-or-nothing.** Purging `tenants` in the channel
  suite failed until the list covered EVERY tenant-referencing table (e.g.
  `human_principals` left by an earlier suite in the same run). A suite that
  owns a table owns everything referencing it, in FK order.
