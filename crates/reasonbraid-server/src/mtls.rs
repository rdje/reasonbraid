//! The mTLS serving config (`PHASE-7.1.2`, ADR-034, §16.2): the TLS 1.3-only
//! acceptor with the MUTUAL authentication — the server presents the
//! CA-signed serving leaf; the client must present a certificate chaining to
//! the deployment CA (the transport-level half). The fingerprint → the
//! principal binding stays the APPLICATION proof (the channel's
//! `verify_cert_proof` — the layered defense: the transport verifies the CA
//! membership, the application verifies the principal). The ring provider is
//! pinned explicitly (the workspace single-provider rule).

use std::sync::Arc;

/// Build the mTLS server config (the §16.2 TLS 1.3-only, the CA-verified
/// client certs).
pub fn build_server_config(
    ca: &crate::ca::ServerCa,
    serving_cert_der: Vec<u8>,
    serving_key_der: Vec<u8>,
) -> Arc<rustls::ServerConfig> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut roots = rustls::RootCertStore::empty();
    let ca_der: rustls_pki_types::CertificateDer<'static> = ca.cert_der.clone().into();
    roots.add(ca_der).expect("the CA root adds");
    let verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .expect("the client verifier builds");
    let chain: Vec<rustls_pki_types::CertificateDer<'static>> =
        vec![serving_cert_der.clone().into(), ca.cert_der.clone().into()];
    let key: rustls_pki_types::PrivateKeyDer<'static> =
        rustls_pki_types::PrivatePkcs8KeyDer::from(serving_key_der).into();
    let config = rustls::ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("TLS 1.3 is available")
        .with_client_cert_verifier(verifier)
        .with_single_cert(chain, key)
        .expect("the serving chain installs");
    Arc::new(config)
}

/// Build the CLIENT config for one node (the test + the node-side wiring):
/// the CA root + the node's leaf.
pub fn build_client_config(
    ca_der: &[u8],
    leaf_der: Vec<u8>,
    leaf_key_der: Vec<u8>,
) -> Arc<rustls::ClientConfig> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut roots = rustls::RootCertStore::empty();
    let ca_cert: rustls_pki_types::CertificateDer<'static> = ca_der.to_vec().into();
    roots.add(ca_cert).expect("the CA root adds");
    let chain: Vec<rustls_pki_types::CertificateDer<'static>> = vec![leaf_der.into()];
    let key: rustls_pki_types::PrivateKeyDer<'static> =
        rustls_pki_types::PrivatePkcs8KeyDer::from(leaf_key_der).into();
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("TLS 1.3 is available")
        .with_root_certificates(roots)
        .with_client_auth_cert(chain, key)
        .expect("the client chain installs");
    Arc::new(config)
}
