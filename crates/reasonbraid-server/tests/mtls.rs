//! The mTLS transport proofs (`.1.2`, ADR-034, §16.2): the TLS 1.3-only
//! acceptor with the MUTUAL authentication — the CA-issued client presents
//! and connects; the cert-less client is refused at the transport (the
//! fingerprint → the principal binding stays the application proof — the
//! layered defense). OFFLINE — the loopback TLS.

use std::sync::Arc;

use reasonbraid_server::{ca::ServerCa, mtls};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn ca() -> ServerCa {
    let key = rcgen::KeyPair::generate().expect("CA key generation");
    let mut params = rcgen::CertificateParams::new(vec![]).expect("CA params");
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "mtls-test-ca");
    params.key_usages = vec![rcgen::KeyUsagePurpose::KeyCertSign];
    let now = time::OffsetDateTime::from_unix_timestamp(chrono::Utc::now().timestamp())
        .expect("the clock");
    params.not_before = now;
    params.not_after = now + time::Duration::days(30);
    let cert = params.self_signed(&key).expect("CA self-sign");
    ServerCa {
        issuer: rcgen::Issuer::new(params, key),
        cert_der: cert.der().to_vec(),
        key_der: Vec::new(),
    }
}

#[test]
fn the_ca_issued_client_connects_and_the_certless_client_refuses() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("the runtime builds");
    rt.block_on(async move {
        let ca = ca();
        let (serving_cert, serving_key) =
            reasonbraid_server::ca::issue_serving_cert(&ca, "localhost");
        let server_config = mtls::build_server_config(&ca, serving_cert, serving_key);
        let leaf = reasonbraid_server::ca::issue_node_leaf(&ca, "node-a", "host-a");
        let (node_cert, node_key) = (leaf.cert_der, leaf.key_der);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binds");
        let addr = listener.local_addr().expect("the addr");

        // The server task: two accepts — the CA-issued client completes the
        // mutual handshake; the cert-less client is refused at the transport.
        let acceptor: tokio_rustls::TlsAcceptor = server_config.into();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accepts the issued client");
            let mut tls = acceptor
                .accept(stream)
                .await
                .expect("the mutual handshake completes");
            let mut buf = [0u8; 1];
            let _ = AsyncReadExt::read(&mut tls, &mut buf).await;
            let (stream, _) = listener
                .accept()
                .await
                .expect("accepts the cert-less client");
            let refused = acceptor.accept(stream).await.is_err();
            assert!(refused, "the transport refuses the cert-less client");
        });

        // The CA-issued node: presents the leaf → the handshake + a byte.
        let client_config = mtls::build_client_config(&ca.cert_der, node_cert, node_key);
        let connector: tokio_rustls::TlsConnector = client_config.into();
        {
            let stream = tokio::net::TcpStream::connect(addr)
                .await
                .expect("connects");
            let mut tls = connector
                .connect(
                    rustls_pki_types::ServerName::try_from("localhost").expect("the SNI"),
                    stream,
                )
                .await
                .expect("the mutual handshake completes");
            AsyncWriteExt::write_all(&mut tls, b"x")
                .await
                .expect("writes");
        }

        // The cert-less client: no client_auth_cert → the transport refuses.
        {
            let provider = Arc::new(rustls::crypto::ring::default_provider());
            let mut roots = rustls::RootCertStore::empty();
            roots
                .add(ca.cert_der.clone().into())
                .expect("the root adds");
            let bare_config = rustls::ClientConfig::builder_with_provider(provider)
                .with_protocol_versions(&[&rustls::version::TLS13])
                .expect("tls13")
                .with_root_certificates(roots)
                .with_no_client_auth();
            let connector: tokio_rustls::TlsConnector = Arc::new(bare_config).into();
            let stream = tokio::net::TcpStream::connect(addr)
                .await
                .expect("connects");
            // TLS 1.3: the client's Finished can leave the client-side connect
            // "successful" before the server's certificate_required alert
            // arrives — the refusal surfaces on the server's accept AND on the
            // client's first read (the alert/EOF, never a data byte).
            let refused = match connector
                .connect(
                    rustls_pki_types::ServerName::try_from("localhost").expect("the SNI"),
                    stream,
                )
                .await
            {
                Err(_) => true,
                Ok(mut tls) => {
                    let mut buf = [0u8; 1];
                    matches!(AsyncReadExt::read(&mut tls, &mut buf).await, Ok(0) | Err(_))
                }
            };
            assert!(refused, "the cert-less client is refused at the transport");
        }

        server.await.expect("the server task joins");
    });
}
