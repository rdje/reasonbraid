//! The `PHASE-2.1.1` issuance-model experiment — run with `--nocapture`:
//!
//! ```bash
//! cargo test -p reasonbraid-cert-spike -- --nocapture
//! ```
//!
//! Every `cert-spike:` line is a verdict or a measurement. The test fails if any
//! refusal case does not refuse or any acceptance case does not complete, so the
//! suite result IS the experiment result.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use rcgen::{
    BasicConstraints, Certificate, CertificateParams, CertifiedIssuer, DnType, IsCa, KeyPair,
    KeyUsagePurpose,
};
use rustls::client::danger::HandshakeSignatureValid;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::server::WebPkiClientVerifier;
use rustls::{
    ClientConfig, ClientConnection, DigitallySignedStruct, DistinguishedName, RootCertStore,
    ServerConfig, ServerConnection, SignatureScheme,
};

// The experiment's shape, matching the §16.2 contract the product will inherit:
// short-lived workload leaves from a server-held CA, riding a durable node id.
const CA_NAME: &str = "reasonbraid-spike-ca";
const LEAF_SECS: i64 = 600; // 10-minute leaves
const LATENCY_N: usize = 200;

/// The CA: a self-signed issuer (params + key + cert in one handle).
struct Ca {
    issuer: CertifiedIssuer<'static, KeyPair>,
}

fn now_offset() -> time::OffsetDateTime {
    time::OffsetDateTime::from(SystemTime::now())
}

fn issue_ca() -> Ca {
    let key = KeyPair::generate().expect("CA key generation");
    let mut params = CertificateParams::new(vec![]).expect("CA params");
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.distinguished_name.push(DnType::CommonName, CA_NAME);
    params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::DigitalSignature,
    ];
    let now = now_offset();
    params.not_before = now;
    params.not_after = now + time::Duration::days(30);
    let issuer = CertifiedIssuer::self_signed(params, key).expect("CA self-sign");
    Ca { issuer }
}

fn issue_leaf(
    ca: &Ca,
    node_id: &str,
    validity: Option<(time::OffsetDateTime, time::OffsetDateTime)>,
) -> (KeyPair, Certificate) {
    let key = KeyPair::generate().expect("leaf key generation");
    // The SAN rides the constructor (rcgen 0.14's `Vec<String>` interface).
    let mut params = CertificateParams::new(vec!["node.local".to_string()]).expect("leaf params");
    params
        .distinguished_name
        .push(DnType::CommonName, format!("node:{node_id}"));
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    let now = now_offset();
    let (nb, na) = validity.unwrap_or((
        // Slight backdate so a fresh cert validates immediately.
        now - time::Duration::seconds(5),
        now + time::Duration::seconds(LEAF_SECS),
    ));
    params.not_before = nb;
    params.not_after = na;
    let cert = params.signed_by(&key, &ca.issuer).expect("leaf sign");
    (key, cert)
}

/// The revocation primitive: sha256 over the leaf DER. The product binds the
/// node id → current fingerprint in the server's status table; this spike proves
/// the handshake-level refusal the table will back.
fn fingerprint(cert: &CertificateDer<'_>) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(cert.as_ref());
    h.finalize().into()
}

/// A client-cert verifier that COMPOSES the two product gates: rustls's webpki
/// verifier (chain to the CA + the validity window) and the fingerprint
/// allowlist (the node id → current fingerprint binding; unregistered = the
/// revoked/unknown-cert refusal).
#[derive(Debug)]
struct AllowlistVerifier {
    inner: Arc<dyn ClientCertVerifier>,
    allowed: Vec<[u8; 32]>,
}

impl ClientCertVerifier for AllowlistVerifier {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        self.inner.root_hint_subjects()
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        now: UnixTime,
    ) -> Result<ClientCertVerified, rustls::Error> {
        // Chain + validity first (the inner verifier), THEN the allowlist.
        let verified = self
            .inner
            .verify_client_cert(end_entity, intermediates, now)?;
        if self.allowed.contains(&fingerprint(end_entity)) {
            Ok(verified)
        } else {
            Err(rustls::Error::General(
                "unregistered workload certificate".into(),
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }

    fn client_auth_mandatory(&self) -> bool {
        true
    }

    fn offer_client_auth(&self) -> bool {
        true
    }
}

/// One server handshake + a 4-byte ping/pong, on a listener bound in advance.
fn server_once(listener: TcpListener, config: Arc<ServerConfig>) -> Result<(), String> {
    let (mut sock, _) = listener.accept().map_err(|e| format!("accept: {e}"))?;
    sock.set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("set_read_timeout: {e}"))?;
    let mut conn = ServerConnection::new(config).map_err(|e| format!("server conn: {e}"))?;
    loop {
        match conn.complete_io(&mut sock) {
            Ok(_) => {
                if conn.is_handshaking() {
                    continue;
                }
                break;
            }
            Err(e) => return Err(format!("server io: {e}")),
        }
    }
    // Drive IO + read until the client's application data is decrypted
    // (a blocking rustls reader yields WouldBlock until complete_io decrypts).
    let mut buf = [0u8; 4];
    let mut filled = 0;
    while filled < 4 {
        match conn.reader().read(&mut buf[filled..]) {
            Ok(0) => return Err("server read: eof".to_string()),
            Ok(n) => filled += n,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                conn.complete_io(&mut sock)
                    .map_err(|e| format!("server io (data): {e}"))?;
            }
            Err(e) => return Err(format!("server read: {e}")),
        }
    }
    conn.writer()
        .write_all(&buf)
        .map_err(|e| format!("server write: {e}"))?;
    conn.complete_io(&mut sock)
        .map_err(|e| format!("server flush: {e}"))?;
    Ok(())
}

/// One client handshake + the ping/pong roundtrip.
fn client_once(addr: std::net::SocketAddr, config: Arc<ClientConfig>) -> Result<(), String> {
    let mut sock = TcpStream::connect(addr).map_err(|e| format!("connect: {e}"))?;
    sock.set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("set_read_timeout: {e}"))?;
    let mut conn = ClientConnection::new(
        config,
        ServerName::try_from("node.local").map_err(|e| format!("sni: {e}"))?,
    )
    .map_err(|e| format!("client conn: {e}"))?;
    loop {
        match conn.complete_io(&mut sock) {
            Ok(_) => {
                if conn.is_handshaking() {
                    continue;
                }
                break;
            }
            Err(e) => return Err(format!("client io: {e}")),
        }
    }
    conn.writer()
        .write_all(b"ping")
        .map_err(|e| format!("client write: {e}"))?;
    // Drive IO + read until the server's pong is decrypted.
    let mut buf = [0u8; 4];
    let mut filled = 0;
    while filled < 4 {
        match conn.reader().read(&mut buf[filled..]) {
            Ok(0) => return Err("client read: eof".to_string()),
            Ok(n) => filled += n,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                conn.complete_io(&mut sock)
                    .map_err(|e| format!("client io (data): {e}"))?;
            }
            Err(e) => return Err(format!("client read: {e}")),
        }
    }
    if &buf != b"ping" {
        return Err(format!("pong mismatch: {buf:?}"));
    }
    Ok(())
}

/// Build a server config: chain verification to the CA + the fingerprint allowlist.
fn server_config(
    ca: &Ca,
    server_cert: CertificateDer<'static>,
    server_key: PrivateKeyDer<'static>,
    allowed: Vec<[u8; 32]>,
) -> Arc<ServerConfig> {
    let mut roots = RootCertStore::empty();
    roots.add(ca.issuer.der().clone()).expect("root store add");
    let inner = WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .expect("client verifier build");
    let verifier: Arc<dyn ClientCertVerifier> = Arc::new(AllowlistVerifier { inner, allowed });
    Arc::new(
        ServerConfig::builder()
            .with_client_cert_verifier(verifier)
            .with_single_cert(vec![server_cert, ca.issuer.der().clone()], server_key)
            .expect("server config"),
    )
}

/// Build a client config trusting ONLY the CA and presenting the leaf.
fn client_config(
    ca: &Ca,
    leaf: CertificateDer<'static>,
    leaf_key: PrivateKeyDer<'static>,
) -> Arc<ClientConfig> {
    let mut roots = RootCertStore::empty();
    roots.add(ca.issuer.der().clone()).expect("root store add");
    Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_client_auth_cert(vec![leaf], leaf_key)
            .expect("client config"),
    )
}

/// Run one roundtrip; return BOTH sides' verdicts (each refusal must be visible
/// on both ends, not just one).
fn roundtrip(
    server_cfg: Arc<ServerConfig>,
    client_cfg: Arc<ClientConfig>,
) -> (Result<(), String>, Result<(), String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("local addr");
    let server_cfg2 = server_cfg.clone();
    let server = std::thread::spawn(move || server_once(listener, server_cfg2));
    let client = client_once(addr, client_cfg);
    let server = server.join().expect("server thread join");
    (client, server)
}

fn leaf_and_key_der(
    ca: &Ca,
    node_id: &str,
    validity: Option<(time::OffsetDateTime, time::OffsetDateTime)>,
) -> (CertificateDer<'static>, PrivateKeyDer<'static>) {
    let (key, cert) = issue_leaf(ca, node_id, validity);
    (cert.der().clone(), PrivateKeyDer::from(key))
}

#[test]
fn issuance_model_verdicts() {
    // ── the CA + the server's own certificate ────────────────────────────────
    let ca = issue_ca();
    let (server_key, server_cert) = issue_leaf(&ca, "server", None);
    let server_der = server_cert.der().clone();
    let server_key_der = PrivateKeyDer::from(server_key);
    eprintln!("cert-spike: CA `{CA_NAME}` self-issued (30 d)");
    eprintln!("cert-spike: server cert issued (SAN node.local) — CN = node:server");

    // ── 1. a trusted, allowlisted leaf completes the handshake ───────────────
    let (leaf_a, key_a) = leaf_and_key_der(&ca, "rol_aaa", None);
    let fp_a = fingerprint(&leaf_a);
    let (client, server) = roundtrip(
        server_config(
            &ca,
            server_der.clone(),
            server_key_der.clone_key(),
            vec![fp_a],
        ),
        client_config(&ca, leaf_a.clone(), key_a.clone_key()),
    );
    assert!(client.is_ok(), "trusted leaf must complete: {client:?}");
    assert!(
        server.is_ok(),
        "server must accept the allowlisted leaf: {server:?}"
    );
    eprintln!("cert-spike: OK — trusted allowlisted leaf completes (ping/pong)");

    // ── 2. a foreign CA's leaf is refused (chain validation, both sides) ─────
    let foreign = issue_ca();
    let (leaf_f, key_f) = leaf_and_key_der(&foreign, "rol_aaa", None);
    // Allowlisted (fingerprint-level) but NOT chain-trusted: the webpki
    // verifier must refuse it regardless of the allowlist.
    let (client, server) = roundtrip(
        server_config(
            &ca,
            server_der.clone(),
            server_key_der.clone_key(),
            vec![fingerprint(&leaf_f)],
        ),
        client_config(&ca, leaf_f, key_f),
    );
    assert!(
        client.is_err(),
        "foreign-CA leaf must fail the client: {client:?}"
    );
    assert!(
        server.is_err(),
        "server must refuse the foreign chain: {server:?}"
    );
    eprintln!("cert-spike: OK — foreign-CA leaf refused on both sides");

    // ── 3. an EXPIRED leaf is refused (validity window, both sides) ──────────
    let now = now_offset();
    let expired = (
        now - time::Duration::hours(2),
        now - time::Duration::hours(1),
    );
    let (leaf_e, key_e) = leaf_and_key_der(&ca, "rol_bbb", Some(expired));
    let (client, server) = roundtrip(
        server_config(
            &ca,
            server_der.clone(),
            server_key_der.clone_key(),
            vec![fingerprint(&leaf_e)],
        ),
        client_config(&ca, leaf_e, key_e),
    );
    assert!(
        client.is_err(),
        "expired leaf must fail the client: {client:?}"
    );
    assert!(
        server.is_err(),
        "server must refuse the expired leaf: {server:?}"
    );
    eprintln!("cert-spike: OK — expired leaf refused on both sides");

    // ── 4. an unregistered fingerprint (the revocation primitive) ────────────
    let (leaf_u, key_u) = leaf_and_key_der(&ca, "rol_ccc", None);
    let (client, server) = roundtrip(
        server_config(
            &ca,
            server_der.clone(),
            server_key_der.clone_key(),
            vec![fp_a], // leaf_u is NOT allowlisted
        ),
        client_config(&ca, leaf_u, key_u),
    );
    assert!(
        client.is_err(),
        "unregistered leaf must fail the client: {client:?}"
    );
    assert!(
        server.is_err(),
        "server must refuse the unregistered leaf: {server:?}"
    );
    eprintln!("cert-spike: OK — unregistered fingerprint refused (the revocation primitive)");

    // ── 5. rotation: a fresh key + cert for the SAME node id ─────────────────
    let (leaf_r, key_r) = leaf_and_key_der(&ca, "rol_aaa", None);
    let fp_r = fingerprint(&leaf_r);
    assert_ne!(fp_a, fp_r, "rotation must produce a fresh fingerprint");
    // The new cert completes once allowlisted…
    let (client, server) = roundtrip(
        server_config(
            &ca,
            server_der.clone(),
            server_key_der.clone_key(),
            vec![fp_r],
        ),
        client_config(&ca, leaf_r, key_r),
    );
    assert!(client.is_ok(), "rotated leaf must complete: {client:?}");
    assert!(
        server.is_ok(),
        "server must accept the rotated leaf: {server:?}"
    );
    // …and the old cert STILL completes while its fingerprint stays allowlisted
    // (rotation is additive until expiry/removal — no silent invalidation).
    let (client, server) = roundtrip(
        server_config(
            &ca,
            server_der.clone(),
            server_key_der.clone_key(),
            vec![fp_a, fp_r],
        ),
        client_config(&ca, leaf_a, key_a),
    );
    assert!(
        client.is_ok(),
        "the pre-rotation cert must stay valid until expiry/removal: {client:?}"
    );
    assert!(
        server.is_ok(),
        "server must keep accepting the not-yet-removed cert: {server:?}"
    );
    eprintln!(
        "cert-spike: OK — rotation = fresh fingerprint for the same node id; the old cert stays valid until removed/expired"
    );

    // ── 6. issuance latency (the operational-experiment number) ──────────────
    let mut durations = Vec::with_capacity(LATENCY_N);
    for _ in 0..LATENCY_N {
        let t0 = Instant::now();
        let _ = issue_leaf(&ca, "rol_lat", None);
        durations.push(t0.elapsed());
    }
    durations.sort();
    let p50 = durations[LATENCY_N / 2];
    let p95 = durations[(LATENCY_N * 95) / 100];
    eprintln!(
        "cert-spike: issuance latency N={LATENCY_N} p50={:?} p95={:?}",
        p50, p95
    );
    // A soft sanity ceiling (an issuance cost of >10 ms would change the
    // enrollment/rotation design; the spike records the real number).
    assert!(
        p95 < Duration::from_millis(10),
        "leaf issuance p95 must stay in the single-digit-ms range: p95={p95:?}"
    );

    eprintln!("cert-spike: ALL VERDICTS PASS");
}
