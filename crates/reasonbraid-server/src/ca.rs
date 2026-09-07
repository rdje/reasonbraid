//! The control plane's workload-identity CA (`PHASE-2.1.2.1`, ADR-007): ONE
//! self-signed CA per deployment, persisted in `server_ca` (migration 0011),
//! issuing short-lived leaves whose CN is the durable node id and whose SAN is
//! the enrollment token's host claim — the certificate rides the identity
//! (§16.2), it does not replace it.
//!
//! Honest limits (ADR-007's consequences, restated at the point of use):
//! - the CA key lives with the control plane (dev/Trusted-LAN profile);
//! - the node's leaf key is server-generated and dev-escrowed (the `.1.2.1`
//!   trust-store stance — the Internet profile re-evaluates both, the ADR's
//!   revisit trigger);
//! - no OCSP/CRL: revocation is the status table + the short validity window.

use rcgen::{BasicConstraints, CertificateParams, DnType, IsCa, Issuer, KeyPair, KeyUsagePurpose};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

/// How long an issued workload leaf stays valid (ADR-007: 10 minutes).
pub const LEAF_TTL_SECS: i64 = 600;

/// The server's CA handle: the certified issuer (params + signing key + the
/// CA's own certificate) + the DER material persisted in `server_ca` (the
/// dev-profile placement — ADR-007's consequences).
pub struct ServerCa {
    pub issuer: Issuer<'static, KeyPair>,
    pub cert_der: Vec<u8>,
    pub key_der: Vec<u8>,
}

impl std::fmt::Debug for ServerCa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerCa")
            .field("cert_der_bytes", &self.cert_der.len())
            .finish_non_exhaustive()
    }
}

fn now_offset() -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp(chrono::Utc::now().timestamp())
        .expect("system clock before the Unix epoch")
}

/// Generate a fresh CA (first boot).
fn generate_ca() -> ServerCa {
    let key = KeyPair::generate().expect("CA key generation");
    let mut params = CertificateParams::new(vec![]).expect("CA params");
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params
        .distinguished_name
        .push(DnType::CommonName, "reasonbraid-server-ca");
    params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::DigitalSignature,
    ];
    let now = now_offset();
    params.not_before = now;
    params.not_after = now + time::Duration::days(365);
    let cert = params.self_signed(&key).expect("CA self-sign");
    let key_der = key.serialize_der();
    ServerCa {
        issuer: Issuer::new(params, key),
        cert_der: cert.der().to_vec(),
        key_der,
    }
}

/// Load the persisted CA (subsequent boots — the demo kills and restarts the
/// server, so previously issued leaves must still chain).
fn load_ca(ca_der: Vec<u8>, key_der: Vec<u8>) -> ServerCa {
    let key = PrivateKeyDer::try_from(key_der.clone()).expect("stored CA key is a valid DER");
    let key = KeyPair::from_der_and_sign_algo(&key, &rcgen::PKCS_ECDSA_P256_SHA256)
        .expect("stored CA key parses");
    let cert_der: CertificateDer<'static> = ca_der.clone().into();
    let issuer = Issuer::from_ca_cert_der(&cert_der, key).expect("stored CA cert parses");
    ServerCa {
        issuer,
        cert_der: ca_der,
        key_der,
    }
}

/// Ensure the deployment's CA exists (row id 1), loading it if it does and
/// generating + persisting it if it does not. Race-safe: two concurrent first
/// boots race the INSERT, the loser reloads the winner's row.
pub async fn ensure_server_ca(pool: &PgPool) -> Result<ServerCa, sqlx::Error> {
    let existing: Option<(Vec<u8>, Vec<u8>)> =
        sqlx::query_as("SELECT ca_der, key_der FROM server_ca WHERE ca_id = 1")
            .fetch_optional(pool)
            .await?;
    if let Some((ca_der, key_der)) = existing {
        return Ok(load_ca(ca_der, key_der));
    }
    let fresh = generate_ca();
    sqlx::query(
        "INSERT INTO server_ca (ca_id, ca_der, key_der) VALUES (1, $1, $2) \
                 ON CONFLICT (ca_id) DO NOTHING",
    )
    .bind(&fresh.cert_der)
    .bind(&fresh.key_der)
    .execute(pool)
    .await?;
    let (ca_der, key_der): (Vec<u8>, Vec<u8>) =
        sqlx::query_as("SELECT ca_der, key_der FROM server_ca WHERE ca_id = 1")
            .fetch_one(pool)
            .await?;
    Ok(load_ca(ca_der, key_der))
}

/// Issue one short-lived workload leaf: CN `node:<node_id>` (the durable
/// identity), SAN = the enrollment token's host claim. Returns (cert DER, key
/// DER) — the key is server-generated (dev escrow, see the module note).
pub fn issue_node_leaf(ca: &ServerCa, node_id: &str, host_claim: &str) -> (Vec<u8>, Vec<u8>) {
    let key = KeyPair::generate().expect("leaf key generation");
    let mut params = CertificateParams::new(vec![host_claim.to_string()]).expect("leaf params");
    params
        .distinguished_name
        .push(DnType::CommonName, format!("node:{node_id}"));
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    let now = now_offset();
    params.not_before = now - time::Duration::seconds(5);
    params.not_after = now + time::Duration::seconds(LEAF_TTL_SECS);
    let cert = params.signed_by(&key, &ca.issuer).expect("leaf sign");
    (cert.der().to_vec(), key.serialize_der())
}

/// The cert fingerprint: sha256 over the leaf DER (the node-id → fingerprint
/// binding the handshake gates on — the same shape the `.1.1` spike proved).
pub fn cert_fingerprint(cert_der: &[u8]) -> String {
    Sha256::digest(cert_der)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The server-side hex helpers (no hex crate — the codebase hand-rolls hex).
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn from_hex(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("odd-length hex".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

// ── The `.1.2.2` channel-proof verification (chain → status → signature) ──────

/// Verify a presented leaf: it chains to THIS deployment's CA within its
/// validity window (webpki), its fingerprint is registered for the node and
/// not revoked/expired (the caller's row check), and `signature` verifies over
/// `message` against the leaf's SPKI (ring, ECDSA P-256 ASN.1 — rcgen's
/// default algorithm). Each refusal is a distinct `Err` reason so the caller
/// maps it to the typed channel refusal.
pub fn verify_leaf_chain(ca: &ServerCa, cert_der: &[u8]) -> Result<(), String> {
    let cert: CertificateDer<'static> = cert_der.to_vec().into();
    let ca_cert: CertificateDer<'static> = ca.cert_der.clone().into();
    let anchor = webpki::anchor_from_trusted_cert(&ca_cert)
        .map_err(|e| format!("CA anchor invalid: {e}"))?;
    let anchors = [anchor];
    let end_entity =
        webpki::EndEntityCert::try_from(&cert).map_err(|e| format!("leaf unparsable: {e}"))?;
    end_entity
        .verify_for_usage(
            &[webpki::ring::ECDSA_P256_SHA256],
            &anchors,
            &[],
            rustls_pki_types::UnixTime::now(),
            webpki::KeyUsage::client_auth(),
            None,
            None,
        )
        .map(|_| ())
        .map_err(|e| format!("chain/validity verification failed: {e}"))
}

/// Extract the leaf.s SPKI DER (the public key the proof signature binds to).
pub fn extract_point(cert_der: &[u8]) -> Result<Vec<u8>, String> {
    let (_, x509) = x509_parser::parse_x509_certificate(cert_der)
        .map_err(|e| format!("leaf unparsable: {e}"))?;
    Ok(x509.public_key().subject_public_key.data.to_vec())
}

/// Verify an ECDSA P-256 (ASN.1) signature over `message` against the leaf.s
/// EC point (see extract_point).
pub fn verify_signature(point: &[u8], message: &[u8], signature: &[u8]) -> Result<(), String> {
    use ring::signature::UnparsedPublicKey;
    let key = UnparsedPublicKey::new(&ring::signature::ECDSA_P256_SHA256_ASN1, point);
    key.verify(message, signature)
        .map_err(|_| "the proof signature did not verify".to_string())
}
