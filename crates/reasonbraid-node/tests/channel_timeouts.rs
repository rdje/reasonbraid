//! The channel's bounds (`SIGNOFF-REPAIR.4.4.5.1`): a control plane that accepts
//! the connection and then goes silent fails the request within its bound, as a
//! transport timeout, instead of holding the node's work loop for ever.
//!
//! The RED this leaf was built from ran the DEFAULT client against the silent
//! server with an outer 45 s bound and never returned; after the repair the same
//! probe returned its error at 30.00 s, the default request bound. That probe is
//! not kept: it costs half a minute. What it proved is split across the two
//! controls below: every client is built by the one bounded builder, and the
//! defaults are finite and are the ones both constructors install.

mod support;

use std::time::{Duration, Instant};

use reasonbraid_node::{
    ChannelError, ChannelTimeouts, HandshakeRequest, NodeChannel, CHANNEL_VERSION,
};
use support::control_plane::StalledServer;

const NODE_ID: &str = "nod_00000000-0000-7000-8000-000000000001";

fn channel_at(base_url: &str) -> NodeChannel {
    let key = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).expect("keypair");
    NodeChannel::new(base_url, NODE_ID.to_string(), vec![0x00, 0x01, 0x02], key)
}

fn handshake_request() -> HandshakeRequest {
    HandshakeRequest {
        channel_version: CHANNEL_VERSION,
        node_id: NODE_ID.to_string(),
        last_acked_cursor: 0,
        pending_operations: Vec::new(),
        ambiguous_attempts: Vec::new(),
        cert_der: String::new(),
        proof_signature: String::new(),
        nonce: String::new(),
    }
}

/// THE failure: a silent server. The request fails at its bound, as a transport
/// timeout the node answers by reconciling; the outer bound only keeps a
/// regression from hanging the suite.
#[tokio::test]
async fn a_silent_server_fails_the_request_within_its_bound() {
    let server = StalledServer::start().await;
    let bounds = ChannelTimeouts {
        connect: Duration::from_secs(2),
        request: Duration::from_millis(300),
    };
    let channel = channel_at(server.base_url()).with_timeouts(bounds);
    assert_eq!(channel.timeouts(), bounds, "the channel reports its bounds");

    let started = Instant::now();
    let outcome = tokio::time::timeout(
        Duration::from_secs(10),
        channel.handshake(&handshake_request()),
    )
    .await
    .expect("the request returned on its own, before the outer bound");
    let elapsed = started.elapsed();
    match outcome {
        Err(ChannelError::Http(e)) => assert!(e.is_timeout(), "a transport timeout: {e}"),
        other => panic!("expected a transport timeout, got {other:?}"),
    }
    assert!(
        elapsed >= bounds.request && elapsed < Duration::from_secs(5),
        "it failed at its bound, not before and not long after: {elapsed:?}"
    );
}

/// The defaults are finite and positive, and they are what a channel starts
/// with: a zero bound would fail every request, an unbounded one is the defect.
#[test]
fn the_default_bounds_are_finite_and_installed() {
    let defaults = ChannelTimeouts::default();
    assert!(defaults.connect > Duration::ZERO && defaults.connect <= Duration::from_secs(30));
    assert!(defaults.request > Duration::ZERO && defaults.request <= Duration::from_secs(60));
    assert_eq!(channel_at("http://127.0.0.1:1").timeouts(), defaults);
    assert_eq!(
        NodeChannel::for_enrollment("http://127.0.0.1:1", NODE_ID.to_string()).timeouts(),
        defaults
    );
}
