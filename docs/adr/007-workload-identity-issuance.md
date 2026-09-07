# ADR-007 — Workload identity issuance: a project-local CA with short-lived certificates

- **Status:** `accepted` (evidence-gated — the `PHASE-2.1.1` spike measured the
  candidate before this decision)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-2.1.1`
- **Requirements:** `ROADMAP.md` §23 queue item 007; §16.2 (identity and
  transport); §2.2.16 (operational simplicity first)

## Context

Phase 1 proved enrollment with one-time tokens + a dev signing key (migration
0008) and an HMAC key-proof channel — the loopback dev trust store. Phase 2
needs the §16.2 contract: TLS 1.3 as the default external transport,
short-lived rotated workload certificates, and a certificate identity that
RIDES the durable node id (reimaging a host must not inherit identity). The
roadmap names the candidates but forbids choosing by name alone: "selection
requires an ADR and operational experiment."

## Options

1. **Project-local CA (rcgen)** — the control plane holds a CA key, issues
   short-lived X.509 leaves at enrollment/rotation over the authenticated
   channel, and gates the handshake on chain-to-the-CA + a server-side
   fingerprint/status table. No external service.
2. **step-ca** — a separate ACME-ish CA service: standard CLI issuance, OCSP/
   CRL, but a new long-running service, binary, and key-management story to
   operate on every deployment.
3. **SPIFFE/SPIRE** — the full workload-identity framework: SVIDs, agent
   daemons per host, registration API. The strongest ecosystem story, the
   heaviest operational footprint — designed for the cloud/Kubernetes world
   the Trusted LAN profile does not inhabit.

## Evidence

The spike (`crates/reasonbraid-cert-spike`, `PHASE-2.1.1`; run:
`cargo test -p reasonbraid-cert-spike -- --nocapture`, log
`target/spike81.log`) measured option 1 against the §16.2 contract over a REAL
rustls TLS 1.3 handshake (client-cert authenticated, ping/pong roundtrip):

- trusted allowlisted leaf completes;
- foreign-CA leaf refused on BOTH sides (chain validation);
- expired leaf refused on BOTH sides (validity window);
- unregistered fingerprint refused (the revocation primitive the status table
  will back);
- rotation = a fresh key + cert for the same node id — the new cert completes,
  the old cert stays valid until removed/expired (additive, no silent
  invalidation);
- issuance latency N=200: **p50 = 63 µs, p95 = 69 µs** — cert issuance is
  effectively free at LAN scale (re-derive: the command above; the number is a
  one-shot measurement on this machine, not a cross-platform claim).

Options 2–3 were evaluated on their published operational model, not installed
(recorded asymmetry): both move the trust root into a second service/agent the
LAN profile does not need yet; nothing in the spike contradicts their later
adoption because the wire contract depends only on verifiable identity +
rotation semantics (§16.2). The chosen stack passes the supply-chain gates:
`make deny` → advisories/bans/licenses/sources ok (rcgen 0.14.10, rustls
0.23.43, rustls-pki-types 1.15.1 — versions pinned by `Cargo.lock`).

## Choice

Option 1 for the Trusted LAN profile: the control plane issues short-lived
(10-minute) workload certificates from a project-local CA, gated by
chain-to-the-CA + the node id → current fingerprint binding; rotation re-issues
additively, revocation is the status table + short expiry.

## Consequences

- One less service to operate (subtraction: no CA daemon, no per-host agent).
- Honest limits: no OCSP/CRL distribution (revocation = server-side status +
  short expiry — acceptable while the control plane is the single gate);
  the CA key lives with the control plane (its compromise = re-issue, a
  documented recovery operation); the cert CN carries the node id, the SAN
  carries the host name.
- `.1.2` implements it: enroll issues the cert, the channel upgrades to the
  cert proof (CHANNEL_VERSION 3), rotation rides the authenticated channel.
- A later step-ca/SPIRE migration changes the issuer, not the wire contract.

## Rollback / revisit trigger

- Any measured revocation-freshness requirement shorter than the status-table
  propagation (OCSP/CRL becomes justified).
- Internet qualification (G6): re-evaluate the CA key's placement and the
  issuance path before any non-LAN exposure.
