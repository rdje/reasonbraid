//! The `PHASE-2.1.1` workload-identity issuance spike (ROADMAP §16.2, ADR-007).
//!
//! This crate is an EXPERIMENT, not product code: it measures the project-local
//! CA candidate (rcgen + rustls) so ADR-007 is an evidence-gated decision rather
//! than a technology preference. The experiment is the integration test in
//! `tests/issuance_model.rs` — run it with
//! `cargo test -p reasonbraid-cert-spike -- --nocapture`.
//!
//! What the experiment proves, against the §16.2 contract:
//! - a server-held CA key issues short-lived workload leaves (measured latency);
//! - the leaf rides a durable node id (the CN/SAN), never replacing it;
//! - the handshake refuses: an untrusted chain (a different CA), an expired
//!   leaf, and an unregistered fingerprint (the revocation-by-allowlist
//!   primitive the server-side status table will back);
//! - rotation = a fresh key + cert for the SAME node id (both handshakes pass
//!   until the old fingerprint leaves the allowlist or the cert expires).
