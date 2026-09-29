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
use std::sync::Arc;

/// How long an issued workload leaf stays valid (ADR-007: 10 minutes).
pub const LEAF_TTL_SECS: i64 = 600;

/// The server's CA handle: the certified issuer (params + signing key + the
/// CA's own certificate) + the DER material persisted in `server_ca` (the
/// dev-profile placement — ADR-007's consequences).
pub struct ServerCa {
    pub issuer: Issuer<'static, KeyPair>,
    pub cert_der: Vec<u8>,
    pub key_der: Vec<u8>,
    /// The CA certificate's own expiry, read out of it (`SIGNOFF-REPAIR.4.1.7`):
    /// no leaf it signs may outlive it.
    pub not_after: chrono::DateTime<chrono::Utc>,
}

/// The expiry a CA certificate carries. The certificates here are the server's
/// own, generated or stored by it, so an unparsable one is a corrupted store.
fn ca_not_after(cert_der: &[u8]) -> chrono::DateTime<chrono::Utc> {
    validity_window(cert_der)
        .expect("the server's own CA certificate parses")
        .1
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
    let cert_der = cert.der().to_vec();
    ServerCa {
        issuer: Issuer::new(params, key),
        not_after: ca_not_after(&cert_der),
        cert_der,
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
        not_after: ca_not_after(&ca_der),
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

/// Why [`issue_node_leaf`] issued nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeafRefused {
    /// The caller's host claim cannot be a SAN (`SIGNOFF-REPAIR.4.1.6`).
    HostClaim(HostClaimRefused),
    /// The issuer has no more than the node's rotation margin left
    /// (`SIGNOFF-REPAIR.4.1.7`): a leaf capped at its expiry would be due for
    /// rotation the instant it was issued. The CA must be renewed (`.4.1.8`).
    IssuerExhausted {
        issuer_not_after: chrono::DateTime<chrono::Utc>,
    },
}

impl std::fmt::Display for LeafRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeafRefused::HostClaim(refusal) => refusal.fmt(f),
            LeafRefused::IssuerExhausted { issuer_not_after } => write!(
                f,
                "the certificate authority expires at {}, too soon to issue a leaf a node \
                 would not have to rotate at once; the authority must be renewed",
                issuer_not_after.to_rfc3339()
            ),
        }
    }
}

impl std::error::Error for LeafRefused {}

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
/// Returns [`LeafRefused::HostClaim`] rather than panicking when the library
/// will not accept the claim (`SIGNOFF-REPAIR.4.1.6`): the claim is a caller
/// string, and a function that can only report a bad one by unwinding leaves
/// its caller's typed error path unreachable.
///
/// The leaf never outlives its issuer (`SIGNOFF-REPAIR.4.1.7`): its `not_after`
/// is the earlier of `now + LEAF_TTL_SECS` and the CA's own, and an issuer with
/// no more than [`reasonbraid_core::LEAF_ROTATE_REMAINING_SECS`] left issues
/// nothing ([`LeafRefused::IssuerExhausted`]).
pub fn issue_node_leaf(
    ca: &ServerCa,
    node_id: &str,
    host_claim: &str,
) -> Result<IssuedLeaf, LeafRefused> {
    let now = now_offset();
    let issuer_left = ca.not_after.timestamp() - now.unix_timestamp();
    if issuer_left <= reasonbraid_core::LEAF_ROTATE_REMAINING_SECS {
        return Err(LeafRefused::IssuerExhausted {
            issuer_not_after: ca.not_after,
        });
    }
    let key = KeyPair::generate().expect("leaf key generation");
    let mut params = CertificateParams::new(vec![host_claim.to_string()]).map_err(|e| {
        LeafRefused::HostClaim(HostClaimRefused {
            host_claim: host_claim.to_owned(),
            reason: e.to_string(),
        })
    })?;
    params
        .distinguished_name
        .push(DnType::CommonName, format!("node:{node_id}"));
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    params.not_before = now - time::Duration::seconds(5);
    params.not_after = now + time::Duration::seconds(LEAF_TTL_SECS.min(issuer_left));
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
    verify_leaf_against(&[ca], cert_der, chrono::Utc::now())
}

/// Verify a leaf against ANY of `cas` (`SIGNOFF-REPAIR.4.1.8.1`): the trust
/// set while one generation issues and an older one's leaves are still live.
fn verify_leaf_against(
    cas: &[&ServerCa],
    cert_der: &[u8],
    at: chrono::DateTime<chrono::Utc>,
) -> Result<(), String> {
    let cert: CertificateDer<'static> = cert_der.to_vec().into();
    let ca_certs: Vec<CertificateDer<'static>> =
        cas.iter().map(|ca| ca.cert_der.clone().into()).collect();
    let anchors = ca_certs
        .iter()
        .map(|ca_cert| {
            webpki::anchor_from_trusted_cert(ca_cert).map_err(|e| format!("CA anchor invalid: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let end_entity =
        webpki::EndEntityCert::try_from(&cert).map_err(|e| format!("leaf unparsable: {e}"))?;
    end_entity
        .verify_for_usage(
            &[webpki::ring::ECDSA_P256_SHA256],
            &anchors,
            &[],
            rustls_pki_types::UnixTime::since_unix_epoch(std::time::Duration::from_secs(
                u64::try_from(at.timestamp()).map_err(|_| "a time before 1970".to_string())?,
            )),
            webpki::KeyUsage::client_auth(),
            None,
            None,
        )
        .map(|_| ())
        .map_err(|e| format!("chain/validity verification failed: {e}"))
}

/// The deployment's CA generations (`SIGNOFF-REPAIR.4.1.8.1`, decided by
/// `docs/decisions/2026-09-24_the-ca-rotates-itself-with-an-overlapping-trust-set.md`):
/// the NEWEST issues; a leaf is trusted if it chains to ANY generation inside
/// its own validity window. Held behind a lock so a successor can be adopted
/// in-process (`.4.1.8.2`) without a restart.
pub struct CaSet {
    /// Oldest first; never empty.
    generations: std::sync::RwLock<Vec<Arc<ServerCa>>>,
}

impl std::fmt::Debug for CaSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaSet")
            .field("generations", &self.read().len())
            .finish()
    }
}

impl CaSet {
    /// One generation: every caller that built a single CA keeps its meaning.
    pub fn single(ca: Arc<ServerCa>) -> Self {
        Self::from_generations(vec![ca])
    }

    /// Generations oldest first. Panics on an empty list: a deployment always
    /// has its first CA (`ensure_server_ca_with_store` creates it).
    pub fn from_generations(generations: Vec<Arc<ServerCa>>) -> Self {
        assert!(
            !generations.is_empty(),
            "a CA set has at least one generation"
        );
        Self {
            generations: std::sync::RwLock::new(generations),
        }
    }

    fn read(&self) -> std::sync::RwLockReadGuard<'_, Vec<Arc<ServerCa>>> {
        // A poisoned lock means a writer panicked mid-swap; the vector is still
        // a whole value (the swap is one assignment), so reading it is sound.
        self.generations
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The generation that signs new leaves: the newest.
    pub fn issuer(&self) -> Arc<ServerCa> {
        self.read().last().expect("never empty").clone()
    }

    /// How many generations the set holds.
    pub fn len(&self) -> usize {
        self.read().len()
    }

    /// Never true: a set always holds its first generation.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Adopt the stored generations, oldest first (`SIGNOFF-REPAIR.4.1.8.2`).
    /// One assignment under the write lock, so a reader sees the old set or the
    /// new one, never a mixture.
    fn replace(&self, generations: Vec<Arc<ServerCa>>) {
        assert!(
            !generations.is_empty(),
            "a CA set has at least one generation"
        );
        *self
            .generations
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = generations;
    }

    /// Verify a node leaf against every generation inside its own window now.
    /// The window is checked HERE because a trust anchor's own validity is not
    /// something webpki checks: an expired generation must stop vouching.
    pub fn verify_leaf(&self, cert_der: &[u8]) -> Result<(), String> {
        self.verify_leaf_at(cert_der, chrono::Utc::now())
    }

    /// [`CaSet::verify_leaf`] at a stated instant, so the window's edges can be
    /// tested exactly rather than raced against the clock.
    fn verify_leaf_at(
        &self,
        cert_der: &[u8],
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), String> {
        let generations = self.read();
        let trusted: Vec<&ServerCa> = generations
            .iter()
            .filter(|ca| {
                validity_window(&ca.cert_der)
                    .is_ok_and(|(not_before, not_after)| not_before <= now && now < not_after)
            })
            .map(|ca| ca.as_ref())
            .collect();
        if trusted.is_empty() {
            return Err("no CA generation is inside its validity window".to_string());
        }
        verify_leaf_against(&trusted, cert_der, now)
    }
}

/// Is `ca` due for a successor at `now`? When a THIRD or less of its lifetime
/// remains (`SIGNOFF-REPAIR.4.1.8.2`; cert-manager's default `renewBefore`, per
/// `docs/decisions/2026-09-24_the-ca-rotates-itself-with-an-overlapping-trust-set.md`).
/// An unparsable CA is due: it cannot be shown to have time left.
pub fn renewal_due(ca: &ServerCa, now: chrono::DateTime<chrono::Utc>) -> bool {
    let Ok((not_before, not_after)) = validity_window(&ca.cert_der) else {
        return true;
    };
    (not_after - now) * 3 <= not_after - not_before
}

/// Mint the issuing CA's successor if it is due, and adopt whatever the store
/// then holds (`SIGNOFF-REPAIR.4.1.8.2`). Returns whether the set changed.
///
/// Race-safe across servers: the insert happens only while the newest STORED
/// generation is still this server's issuer, and `ca_id` is the primary key,
/// so of several servers noticing at once exactly one mints; every one then
/// reloads and adopts it. A server that finds a successor already stored by
/// another adopts it without minting.
pub async fn renew_if_due(
    cas: &CaSet,
    pool: &PgPool,
    store: &crate::secret_store::SecretStore,
) -> Result<bool, sqlx::Error> {
    let issuer = cas.issuer();
    if !renewal_due(&issuer, chrono::Utc::now()) {
        return Ok(false);
    }
    let successor = generate_ca();
    sqlx::query(
        "INSERT INTO server_ca (ca_id, ca_der, key_der) \
         SELECT (SELECT max(ca_id) + 1 FROM server_ca), $1, $2 \
         WHERE (SELECT ca_der FROM server_ca ORDER BY ca_id DESC LIMIT 1) = $3 \
         ON CONFLICT (ca_id) DO NOTHING",
    )
    .bind(&successor.cert_der)
    .bind(&successor.key_der)
    .bind(&issuer.cert_der)
    .execute(pool)
    .await?;
    let generations: Vec<Arc<ServerCa>> = store
        .load_ca_generations(pool)
        .await?
        .into_iter()
        .map(|(ca_der, key_der)| Arc::new(load_ca(ca_der, key_der)))
        .collect();
    let changed = generations.len() != cas.len()
        || generations.last().map(|ca| &ca.cert_der) != Some(&issuer.cert_der);
    if !generations.is_empty() {
        cas.replace(generations);
    }
    Ok(changed)
}

/// Load every stored CA generation through the declared store, creating the
/// first on a fresh database (`SIGNOFF-REPAIR.4.1.8.1`).
pub async fn load_ca_set(
    pool: &PgPool,
    store: &crate::secret_store::SecretStore,
) -> Result<CaSet, sqlx::Error> {
    ensure_server_ca_with_store(pool, store).await?;
    let generations = store
        .load_ca_generations(pool)
        .await?
        .into_iter()
        .map(|(ca_der, key_der)| Arc::new(load_ca(ca_der, key_der)))
        .collect();
    Ok(CaSet::from_generations(generations))
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

/// A certificate's validity window, `[not_before, not_after)`, as signed into
/// it (`SIGNOFF-REPAIR.4.6.1.4`: the health probe of the server CA reads the
/// window the certificate itself carries, like [`leaf_not_after`]).
pub fn validity_window(
    cert_der: &[u8],
) -> Result<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>), String> {
    let (_, x509) = x509_parser::parse_x509_certificate(cert_der)
        .map_err(|e| format!("certificate unparsable: {e}"))?;
    let validity = x509.validity();
    let instant = |t: i64| {
        chrono::DateTime::from_timestamp(t, 0)
            .ok_or_else(|| "the validity window is not representable".to_string())
    };
    Ok((
        instant(validity.not_before.timestamp())?,
        instant(validity.not_after.timestamp())?,
    ))
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
    /// A CA like [`generate_ca`]'s, but with only `secs` of validity left.
    fn ca_valid_for(secs: i64) -> ServerCa {
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
        params.not_before = now - time::Duration::days(364);
        params.not_after = now + time::Duration::seconds(secs);
        let cert = params.self_signed(&key).expect("CA self-sign");
        load_ca(cert.der().to_vec(), key.serialize_der())
    }

    /// A leaf valid for ten minutes from now, signed by `ca` whatever the CA's
    /// own window: the shape `issue_node_leaf` refuses to produce, built here so
    /// the trust set's own filter is what is tested.
    fn leaf_signed_by(ca: &ServerCa) -> Vec<u8> {
        let key = KeyPair::generate().expect("leaf key");
        let mut params = CertificateParams::new(vec!["host-a".to_string()]).expect("params");
        params.distinguished_name.push(
            DnType::CommonName,
            "node:nod_00000000-0000-7000-8000-000000000001",
        );
        params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        let now = now_offset();
        params.not_before = now - time::Duration::seconds(5);
        params.not_after = now + time::Duration::seconds(600);
        params
            .signed_by(&key, &ca.issuer)
            .expect("sign")
            .der()
            .to_vec()
    }

    /// `SIGNOFF-REPAIR.4.1.8.1` — the set issues from its NEWEST generation,
    /// trusts a leaf of ANY generation inside its window, and stops trusting a
    /// generation whose own window has closed, even for a leaf still in date.
    #[test]
    fn the_ca_set_issues_from_the_newest_and_trusts_every_live_generation() {
        let older = Arc::new(ca_valid_for(30 * 86_400));
        let newer = Arc::new(ca_valid_for(365 * 86_400));
        let set = CaSet::from_generations(vec![older.clone(), newer.clone()]);
        assert_eq!(set.issuer().cert_der, newer.cert_der, "the newest issues");
        set.verify_leaf(&leaf_signed_by(&older))
            .expect("an older generation's leaf is still trusted");
        set.verify_leaf(&leaf_signed_by(&newer))
            .expect("the issuer's own leaf is trusted");
        assert!(
            verify_leaf_chain(&newer, &leaf_signed_by(&older)).is_err(),
            "one CA alone trusts only its own leaves: the set is what adds the older one"
        );

        let expired = Arc::new(ca_valid_for(-60));
        let set = CaSet::from_generations(vec![expired.clone(), newer]);
        assert!(
            set.verify_leaf(&leaf_signed_by(&expired)).is_err(),
            "an expired generation vouches for nothing"
        );
        let only_expired = CaSet::single(expired.clone());
        assert!(only_expired
            .verify_leaf(&leaf_signed_by(&expired))
            .is_err_and(|e| e.contains("no CA generation")));
        assert_eq!(format!("{set:?}"), "CaSet { generations: 2 }");
    }

    /// The window's edges, exactly (`SIGNOFF-REPAIR.4.1.8.1`): a generation
    /// vouches through the second BEFORE its `not_after` and not at it, which
    /// is the instant the chain verifier itself treats as expired.
    #[test]
    fn a_generation_stops_vouching_at_its_not_after_exactly() {
        let ca = Arc::new(ca_valid_for(3_600));
        let leaf = leaf_signed_by(&ca);
        let set = CaSet::single(ca.clone());
        let (_, leaf_not_after) = validity_window(&leaf).expect("leaf parses");
        // Keep the instant inside the leaf's own window, so only the CA's edge
        // is being tested.
        assert!(ca.not_after > leaf_not_after);
        let inside = leaf_not_after - chrono::Duration::seconds(1);
        set.verify_leaf_at(&leaf, inside)
            .expect("inside both windows");
        let early = CaSet::single(Arc::new(ca_valid_for(-1)));
        assert!(early.verify_leaf_at(&leaf, inside).is_err());
        // The CA's own edge: a set whose only generation ends at `edge`.
        let edge_ca = Arc::new(ca_valid_for(400));
        let edge_leaf = leaf_signed_by(&edge_ca);
        let edge_set = CaSet::single(edge_ca.clone());
        let edge = edge_ca.not_after;
        assert!(
            edge_set
                .verify_leaf_at(&edge_leaf, edge)
                .is_err_and(|e| e.contains("no CA generation")),
            "at not_after the generation no longer vouches"
        );
        edge_set
            .verify_leaf_at(&edge_leaf, edge - chrono::Duration::seconds(1))
            .expect("one second before, it does");
    }

    /// `SIGNOFF-REPAIR.4.1.8.2` — adopting a reloaded set replaces it whole,
    /// and the set reports its size truthfully before and after.
    #[test]
    fn a_set_adopts_a_reloaded_generation_list_whole() {
        let first = Arc::new(ca_valid_for(30 * 86_400));
        let set = CaSet::single(first.clone());
        assert_eq!(set.len(), 1);
        assert!(!set.is_empty());
        let successor = Arc::new(generate_ca());
        set.replace(vec![first, successor.clone()]);
        assert_eq!(set.len(), 2, "both generations held");
        assert_eq!(
            set.issuer().cert_der,
            successor.cert_der,
            "the adopted newest issues"
        );
        assert!(!set.is_empty());
    }

    /// `SIGNOFF-REPAIR.4.1.8.2` — a CA is due for a successor when a THIRD or
    /// less of its lifetime remains, exactly: 300 days long, it is not due with
    /// 101 days left and is due with 100.
    #[test]
    fn renewal_is_due_at_a_third_of_the_lifetime_exactly() {
        let key = KeyPair::generate().expect("key");
        let mut params = CertificateParams::new(vec![]).expect("params");
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let now = now_offset();
        params.not_before = now - time::Duration::days(200);
        params.not_after = now + time::Duration::days(100);
        let cert = params.self_signed(&key).expect("self-sign");
        let ca = load_ca(cert.der().to_vec(), key.serialize_der());
        let now = chrono::DateTime::from_timestamp(now.unix_timestamp(), 0).expect("instant");
        assert!(
            renewal_due(&ca, now),
            "100 of 300 days left is a third: due"
        );
        assert!(
            !renewal_due(&ca, now - chrono::Duration::days(1)),
            "a day earlier, 101 of 300 left: not yet"
        );
        assert!(!renewal_due(&generate_ca(), now), "a fresh CA is not due");
    }

    /// `SIGNOFF-REPAIR.4.1.7` — a leaf never outlives its issuer. With 400 s
    /// left, the CA signs a leaf that ends when the CA does (not at the usual
    /// 600 s); with 200 s left, which is inside the node's rotation margin, a
    /// leaf would be due for rotation the instant it was issued, so issuance is
    /// refused.
    #[test]
    fn a_leaf_never_outlives_its_issuer() {
        let short = ca_valid_for(400);
        let (_, issuer_not_after) = validity_window(&short.cert_der).expect("the CA parses");
        let leaf = issue_node_leaf(&short, "nod_00000000-0000-7000-8000-000000000001", "host-a")
            .expect("400 s is past the rotation margin, so it issues");
        let (_, leaf_not_after) = validity_window(&leaf.cert_der).expect("the leaf parses");
        assert_eq!(
            leaf_not_after, issuer_not_after,
            "the leaf ends when its issuer does"
        );
        assert_eq!(leaf.not_after, leaf_not_after, "and says so");

        let spent = ca_valid_for(200);
        assert!(
            matches!(
                issue_node_leaf(&spent, "nod_00000000-0000-7000-8000-000000000001", "host-a")
                    .err(),
                Some(LeafRefused::IssuerExhausted { issuer_not_after })
                    if issuer_not_after == spent.not_after
            ),
            "an issuer inside the rotation margin issues nothing, and says when it ends"
        );
        // The operator reads why: the issuer's end, and that it needs renewing.
        let refusal = issue_node_leaf(&spent, "nod_00000000-0000-7000-8000-000000000001", "host-a")
            .err()
            .expect("refused")
            .to_string();
        assert!(
            refusal.contains(&spent.not_after.to_rfc3339()) && refusal.contains("renewed"),
            "the refusal names the expiry and the remedy: {refusal}"
        );
        // An ordinary issuer still issues the full 600 s.
        let ca = generate_ca();
        let leaf = issue_node_leaf(&ca, "nod_00000000-0000-7000-8000-000000000001", "host-a")
            .expect("a year-long issuer issues");
        let (not_before, not_after) = validity_window(&leaf.cert_der).expect("parses");
        assert_eq!((not_after - not_before).num_seconds(), LEAF_TTL_SECS + 5);
    }

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

/// `SIGNOFF-REPAIR.11.4.7.2.1.1` — the fuzz baseline: a node's proof hands the
/// server a certificate in DER, which three parsers read before anything trusts
/// it. Every mutation of a real leaf answers a result, never a panic.
#[cfg(test)]
mod fuzz_certificate {
    use super::*;

    #[test]
    fn the_certificate_parsers_answer_every_mutation() {
        let ca = generate_ca();
        let leaf = issue_node_leaf(&ca, "nod_00000000-0000-7000-8000-000000000001", "host-a")
            .expect("a leaf");
        let set = CaSet::single(std::sync::Arc::new(ca));
        assert!(
            extract_point(&leaf.cert_der).is_ok(),
            "the seed's key reads"
        );
        assert!(set.verify_leaf(&leaf.cert_der).is_ok(), "the seed verifies");
        crate::fuzz_support::survive(
            "certificate parsers",
            std::slice::from_ref(&leaf.cert_der),
            |der| {
                (
                    extract_point(der).is_ok(),
                    leaf_not_after(der).is_ok(),
                    set.verify_leaf(der).is_ok(),
                )
            },
        );
    }
}
