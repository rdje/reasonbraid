# Certificate-proof verification — ring verifies against the EC point, not rcgen's SPKI (`PHASE-2.1.2.2`)

- Date: 2026-09-07 · Leaf: `PHASE-2.1.2.2` · Decision record (the ADR-007 model's verification leg)

## Context

The channel v3 proof verifies the node's ECDSA P-256 signature over the
canonical coverage JSON against the leaf's public key. The natural first
choice — feed the certificate's SPKI DER to `ring::signature::UnparsedPublicKey`
with `ECDSA_P256_SHA256_ASN1` — refused EVERY valid proof (16 channel tests
401). The probe ladder isolated the legs: the webpki chain check passed, the
SPKI extracted cleanly (91 bytes, well-formed DER), the certificate's public
key matched the signing key byte-for-byte, a pure-ring control round-trip
passed — and yet the verification failed, in all four formats (ASN.1 over
SPKI, ASN.1 over digest, FIXED over point, FIXED over digest).

## Decision

- **Verify the proof signature against the bare EC point** extracted from the
  certificate (`x509_parser` → `public_key().subject_public_key.data`), not
  against the SPKI DER. ring's `UnparsedPublicKey` sniffs the bare
  uncompressed point for ECDSA — the same path webpki uses internally (which
  is why the chain check always worked).
- The signature format is ASN.1 DER (70–72 bytes for P-256) — `ECDSA_P256_SHA256_ASN1`,
  matching rcgen's `SigningKey::sign` output.
- The ladder order stays: chain-to-CA + validity (webpki) → node-id fingerprint
  status (the row) → signature (ring over the point) — all before any ledger read.

## answers:

- **rcgen's SPKI DER is well-formed but ring's SPKI parser refuses it** — the
  refusal reproduces with a FRESH rcgen key and zero round-trips, while the
  same ring verifies a bare-point control; the point-based path is the
  measured fix, recorded in `crates/reasonbraid-server/src/ca.rs`.
- **The FIXED-format probes were structurally invalid** — `ECDSA_P256_SHA256_FIXED`
  expects a 64-byte raw r‖s signature, and rcgen emits ASN.1; a failing
  FIXED leg proves nothing about the ASN.1 leg.
- **The probe discipline paid for itself** — the ladder probe (chain / SPKI /
  self-SPKI / ring-only control / digest variants) turned a "signature refused"
  mystery into one named interop fact in three runs; the probe was removed
  before commit, the fix and the record remain.
