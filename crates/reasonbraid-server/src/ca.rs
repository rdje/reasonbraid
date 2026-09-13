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
///
/// The convenience form resolves the shipped `dev_database` store (the
/// tests' default); the boot path uses [`ensure_server_ca_with_store`] with
/// the DEPLOYMENT's resolved profile — the store is the only seam (`.1.4.2`).
pub async fn ensure_server_ca(pool: &PgPool) -> Result<ServerCa, sqlx::Error> {
    ensure_server_ca_with_store(pool, &crate::secret_store::SecretStore::dev()).await
}

/// The store-routed form: the CA material reads THROUGH the declared
/// profile (the configuration choice, never an ambient dependency).
pub async fn ensure_server_ca_with_store(
    pool: &PgPool,
    store: &crate::secret_store::SecretStore,
) -> Result<ServerCa, sqlx::Error> {
    if let Some((ca_der, key_der)) = store.load_ca_material(pool).await? {
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
    let (ca_der, key_der): (Vec<u8>, Vec<u8>) = store
        .load_ca_material(pool)
        .await?
        .expect("the CA row just written loads back");
    Ok(load_ca(ca_der, key_der))
}

/// An issued workload leaf, with the expiry the CERTIFICATE actually carries.
///
/// `SIGNOFF-REPAIR.3.4.3.1.1`: `not_after` is returned rather than left for the
/// caller to re-derive. Both call sites used to compute `Utc::now() + LEAF_TTL_SECS`
/// separately from the value baked into the certificate here — two independent
/// derivations of one quantity, agreeing only because they read the same clock
/// and used the same constant. On the enrollment path the caller's `now` is
/// sampled BEFORE its database work, so the stored expiry under-reported the
/// certificate's real validity by however long that work took.
pub struct IssuedLeaf {
    pub cert_der: Vec<u8>,
    pub key_der: Vec<u8>,
    /// The instant in the certificate's own validity window — the only expiry a
    /// verifier will ever enforce.
    pub not_after: chrono::DateTime<chrono::Utc>,
}

/// A host claim the certificate library will not put in a SAN
/// (`SIGNOFF-REPAIR.4.1.6`).
///
/// ⛔ This is the ONLY caller-supplied input `issue_node_leaf` takes, and it was
/// the only one of that function's five `expect`s that a caller could reach. The
/// others (key generation, the validity window, signing) take server-controlled
/// values and stay as they are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostClaimRefused {
    pub host_claim: String,
    pub reason: String,
}

impl std::fmt::Display for HostClaimRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the host claim cannot be a certificate subject alternative name: {}",
            self.reason
        )
    }
}

impl std::error::Error for HostClaimRefused {}

/// Would `issue_node_leaf` accept this host claim?
///
/// The rule is the certificate library's OWN verdict, asked here rather than
/// restated — so this narrows nothing and cannot drift from what the issuance
/// will actually do. `SIGNOFF-REPAIR.4.1.6` measured that rcgen 0.14.10's accept
/// set is very wide (an empty string, 300 characters, `*` and `..` all pass, and
/// of eight probed claims only a non-ASCII one was refused); whether a STRICTER
/// grammar should bind is a separate decision with a compatibility question,
/// recorded in that leaf and deliberately not taken here.
pub fn check_host_claim(host_claim: &str) -> Result<(), HostClaimRefused> {
    CertificateParams::new(vec![host_claim.to_string()])
        .map(|_| ())
        .map_err(|e| HostClaimRefused {
            host_claim: host_claim.to_owned(),
            reason: e.to_string(),
        })
}

/// Issue one short-lived workload leaf: CN `node:<node_id>` (the durable
/// identity), SAN = the enrollment token's host claim. The key is
/// server-generated (dev escrow, see the module note), and the returned
/// `not_after` is the one signed into the certificate.
///
/// Returns [`HostClaimRefused`] rather than panicking when the library will not
/// accept the claim (`SIGNOFF-REPAIR.4.1.6`): the claim is a caller string, and
/// a function that can only report a bad one by unwinding leaves its caller's
/// typed error path unreachable.
pub fn issue_node_leaf(
    ca: &ServerCa,
    node_id: &str,
    host_claim: &str,
) -> Result<IssuedLeaf, HostClaimRefused> {
    let key = KeyPair::generate().expect("leaf key generation");
    let mut params =
        CertificateParams::new(vec![host_claim.to_string()]).map_err(|e| HostClaimRefused {
            host_claim: host_claim.to_owned(),
            reason: e.to_string(),
        })?;
    params
        .distinguished_name
        .push(DnType::CommonName, format!("node:{node_id}"));
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    let now = now_offset();
    params.not_before = now - time::Duration::seconds(5);
    params.not_after = now + time::Duration::seconds(LEAF_TTL_SECS);
    let not_after = chrono::DateTime::from_timestamp(params.not_after.unix_timestamp(), 0)
        .expect("the leaf validity window is representable");
    let cert = params.signed_by(&key, &ca.issuer).expect("leaf sign");
    Ok(IssuedLeaf {
        cert_der: cert.der().to_vec(),
        key_der: key.serialize_der(),
        not_after,
    })
}

/// The cert fingerprint: sha256 over the leaf DER (the node-id → fingerprint
/// binding the handshake gates on — the same shape the `.1.1` spike proved).
pub fn cert_fingerprint(cert_der: &[u8]) -> String {
    Sha256::digest(cert_der)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The server-side hex ENCODER (no hex crate — the codebase hand-rolls hex).
///
/// There is deliberately no decoder here (`SIGNOFF-REPAIR.4.2.7`). This module
/// carried a `pub fn from_hex` that indexed `&s[i..i + 2]` on a `&str` and so
/// panicked on a slice splitting a multi-byte character — and it had NO caller:
/// `git grep` found none, and making it private turned that into a compiler
/// error, `function `from_hex` is never used`. The server's live decoder is
/// `node_channel::decode_hex`, which works over `as_bytes().chunks(2)` and has
/// no such failure mode; it is the one on the untrusted wire path. A dead,
/// well-named public decoder beside a private correct one is what the next
/// caller reaches for, so it is removed rather than repaired in place.
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
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

/// The expiry a verifier will actually enforce: the leaf's own `not_after`.
///
/// `SIGNOFF-REPAIR.3.4.3.1.1` — this is the same extraction the NODE performs
/// when it decides whether to rotate, so a stored `expires_at` that disagrees
/// with it is a claim about a certificate that the certificate does not make.
pub fn leaf_not_after(cert_der: &[u8]) -> Result<chrono::DateTime<chrono::Utc>, String> {
    let (_, x509) = x509_parser::parse_x509_certificate(cert_der)
        .map_err(|e| format!("leaf unparsable: {e}"))?;
    chrono::DateTime::from_timestamp(x509.validity().not_after.timestamp(), 0)
        .ok_or_else(|| "the leaf validity window is not representable".to_string())
}

/// Verify an ECDSA P-256 (ASN.1) signature over `message` against the leaf.s
/// EC point (see extract_point).
pub fn verify_signature(point: &[u8], message: &[u8], signature: &[u8]) -> Result<(), String> {
    use ring::signature::UnparsedPublicKey;
    let key = UnparsedPublicKey::new(&ring::signature::ECDSA_P256_SHA256_ASN1, point);
    key.verify(message, signature)
        .map_err(|_| "the proof signature did not verify".to_string())
}

/// Issue the SERVER's serving leaf (`.1.2`, §16.2): a CA-signed cert with
/// the server-auth EKU — the mTLS acceptor presents it. The leaf is
/// long-lived (the serving identity, not the workload identity).
pub fn issue_serving_cert(ca: &ServerCa, dns_name: &str) -> (Vec<u8>, Vec<u8>) {
    let key = KeyPair::generate().expect("serving key generation");
    let mut params = CertificateParams::new(vec![dns_name.to_string()]).expect("serving params");
    params
        .distinguished_name
        .push(DnType::CommonName, format!("server:{dns_name}"));
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
    let now = now_offset();
    params.not_before = now - time::Duration::seconds(5);
    params.not_after = now + time::Duration::days(365);
    let cert = params.signed_by(&key, &ca.issuer).expect("serving sign");
    (cert.der().to_vec(), key.serialize_der())
}

#[cfg(test)]
mod issued_leaf_binding {
    use super::*;

    /// `SIGNOFF-REPAIR.3.4.3.1.1` — the expiry a caller stores must be the one
    /// the certificate carries, because `not_after` is the only expiry a
    /// verifier will ever enforce. It used to be re-derived at the call site
    /// from a second clock read, which agreed with this one by coincidence of
    /// reading the same clock with the same constant, and on the enrollment
    /// path did not agree at all: that `now` is sampled BEFORE the enrollment
    /// transaction's database work.
    #[test]
    fn the_returned_expiry_is_the_one_signed_into_the_certificate() {
        let ca = generate_ca();
        let leaf = issue_node_leaf(&ca, "nod_00000000-0000-7000-8000-000000000001", "host-a")
            .expect("the unit fixture host claim is a valid SAN");
        let (_, x509) =
            x509_parser::parse_x509_certificate(&leaf.cert_der).expect("the issued leaf parses");
        assert_eq!(
            leaf.not_after.timestamp(),
            x509.validity().not_after.timestamp(),
            "the returned not_after must be the certificate's own"
        );
        // And it is the declared window, so the constant has not drifted from
        // what the certificate says.
        assert_eq!(
            x509.validity().not_after.timestamp() - x509.validity().not_before.timestamp(),
            LEAF_TTL_SECS + 5,
            "the leaf window is LEAF_TTL_SECS plus the 5 s not_before skew allowance"
        );
    }
}
