# Workload identity issuance — the ADR-007 decision record (`PHASE-2.1.1`)

- Date: 2026-09-07 · Leaf: `PHASE-2.1.1` · Mirror of `docs/adr/007-workload-identity-issuance.md`

## Context

Phase 2's identity lane needed the §16.2 issuance decision before any
certificate code could land. The roadmap forbids choosing a technology by name
("selection requires an ADR and operational experiment"), so the leaf built the
spike first (`crates/reasonbraid-cert-spike`) and decided from its verdicts.

## Decision

- **A project-local CA (rcgen) issues short-lived (10-minute) workload
  certificates** from the control plane; the handshake gates on chain-to-the-CA
  + the node id → current fingerprint binding; rotation re-issues additively;
  revocation is the server-side status table + short expiry (no OCSP/CRL at the
  LAN profile).
- **ADR-006 is accepted-with-evidence**: the shipped outbound channel (WP3 +
  `.1.2.2`) is the transport decision — the record now exists before `.1.2`
  changes the wire again.
- **The spike crate stays a test-only experiment** (no bin target), so
  `make release`'s four-binary contract is untouched.
- **`rcgen` ships without its `pem` feature** (DER only): the bans doctrine
  flagged two base64 versions; dropping the unused feature is the fix, not a
  skip entry.

answers:

- **Issuance is effectively free at LAN scale** — the spike measured p50 63 µs /
  p95 69 µs (N=200) for a leaf issuance; rotation can therefore be aggressive
  without an operational cost.
- **The refusal primitives work end-to-end before any product code exists** —
  foreign-CA, expired, and unregistered-fingerprint certificates are refused on
  BOTH sides of a real TLS 1.3 handshake; rotation is additive (the old cert
  stays valid until removed or expired).
- **The supply-chain gate chose the dependency shape, not me** — `make deny`
  rejected the two-base64 split and the resolution was removing the unused
  `pem` feature, keeping the dependency tree minimal and the ban list honest.
- **The spike's WouldBlock lesson is load-bearing for `.1.2`** — a rustls
  blocking reader yields WouldBlock until `complete_io` decrypts; the channel
  upgrade must drive IO before reading, or it will reproduce the spike's first
  failing iteration in production code.
