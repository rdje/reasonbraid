//! Bounded CLI transport, with recovery preserved
//! (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.3.2`).
//!
//! The CLI persists a bootstrap request key BEFORE it dispatches, and holds the
//! state lock across the response so no other writer can act on a store whose
//! outcome is unknown. Both are correct. Neither is bounded: with no connect or
//! whole-request deadline, a peer that accepts a connection and never answers
//! holds that lock for as long as the process lives.
//!
//! These controls drive the REAL public entrypoint against origins this test
//! owns — a socket that never answers, and one that answers with more bytes
//! than the reply bound allows — and assert what must survive each refusal: the
//! ORIGINAL request key, an exclusion that was actually released, and a
//! same-key recovery that still completes afterwards.
#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use reasonbraid_cli::{Config, StateFile};

/// The hang detector. This — not a tight margin on the bound — is what
/// distinguishes "bounded" from "unbounded": an absent ceiling does not return
/// at all, and any plausible WRONG ceiling (two minutes, five minutes) trips
/// this too.
const OUTER_BOUND: Duration = Duration::from_secs(120);

/// The upper ceiling the refusal must land under, deliberately generous over the
/// 60-second bound it is checking.
///
/// ⚠️ Measured, not guessed: with a `cargo clippy --all-targets` running
/// alongside it, this control observed a refusal at **75.03 s** for a 60 s
/// bound. The deadline governs the REQUEST; the elapsed wall clock also
/// contains the runtime's scheduling, and a saturated machine adds to it. A
/// margin tight enough to catch a 60-vs-75 difference is therefore measuring
/// the load on the host, not the bound in the client — and it fails in CI,
/// where saturation is normal. The assertion that carries the real weight is
/// `OUTER_BOUND` above.
const GENEROUS_CEILING: Duration = Duration::from_secs(100);

/// A repository-local, same-volume fixture directory (§13): the root is found
/// at runtime from the current directory, and nothing reaches for a system
/// temporary directory.
struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::current_dir()
            .expect("cwd")
            .ancestors()
            .find(|p| p.join("crates/reasonbraid-cli/Cargo.toml").is_file())
            .expect("run inside the repository")
            .to_path_buf();
        let base = root.join("target/cli-http-bounds");
        std::fs::create_dir_all(&base).expect("the control base is created");
        let path = base.join(format!("{name}-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir(&path).expect("the control fixture is created");
        Self(path)
    }

    fn config(&self, server: &SocketAddr) -> Config {
        Config {
            server_base: format!("http://{server}"),
            state_dir: self.0.join("store"),
        }
    }

    fn state(&self) -> StateFile {
        StateFile::load(&self.0.join("store")).expect("the store loads")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // A failed control keeps its evidence; a passing one removes only the
        // directory it created.
        if !std::thread::panicking() {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

/// An origin that completes the TCP handshake and then says nothing, ever. This
/// is the peer the bound exists for: a refused connection fails fast on its own,
/// and only a silent accepted connection can hold a caller open indefinitely.
async fn silent_origin() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the silent origin");
    let addr = listener.local_addr().expect("the origin's address");
    let handle = tokio::spawn(async move {
        let mut held = Vec::new();
        while let Ok((stream, _)) = listener.accept().await {
            held.push(stream); // accepted, never answered, never closed
        }
    });
    (addr, handle)
}

/// The whole-request deadline, measured through the real entrypoint.
///
/// Against the unrepaired client this control does not fail — it does not
/// RETURN, which is the defect stated precisely. The outer bound is what turns
/// a hang into a measurement.
#[tokio::test(flavor = "multi_thread")]
async fn a_silent_peer_cannot_hold_the_cli_open() {
    let fixture = Fixture::new("silent-peer");
    let (addr, _origin) = silent_origin().await;
    let cfg = fixture.config(&addr);

    let started = Instant::now();
    let outcome = tokio::time::timeout(
        OUTER_BOUND,
        reasonbraid_cli::run_enroll(&cfg, "human", "stalled", None, None, true),
    )
    .await;
    let elapsed = started.elapsed();

    let result = outcome.unwrap_or_else(|_| {
        panic!(
            "the CLI did not return within {OUTER_BOUND:?} against a silent peer \
             — the whole-request bound is absent or ineffective"
        )
    });
    let error = result.expect_err("a silent peer cannot produce an enrollment");
    // Two assertions, each doing the job it can do RELIABLY. Wall clock cannot
    // separate a 60-second bound from a 90-second one on a loaded host, so it is
    // not asked to: the declared value is pinned deterministically, and the
    // runtime observation proves that declared value is actually enforced rather
    // than merely written down.
    assert_eq!(
        reasonbraid_cli::REQUEST_TIMEOUT,
        Duration::from_secs(60),
        "the declared whole-request bound changed; the book documents 60s and \
         `.3.3.4.3.3.3.3.2.3.2` derives it from the server's own 15s budget"
    );
    assert!(
        elapsed < GENEROUS_CEILING,
        "the refusal took {elapsed:?}, past the {GENEROUS_CEILING:?} ceiling"
    );
    let rendered = error.to_string();
    assert!(
        rendered.contains("transport error"),
        "a deadline is a transport refusal, named as one: {rendered}"
    );
}

/// What must survive that refusal: the ORIGINAL key, and a released lock.
///
/// A bound that discarded the pending request would be worse than the hang it
/// replaced — the server may have committed a tenant this caller can no longer
/// name.
#[tokio::test(flavor = "multi_thread")]
async fn a_timed_out_request_keeps_its_key_and_releases_the_store() {
    let fixture = Fixture::new("timeout-recovery");
    let (addr, _origin) = silent_origin().await;
    let cfg = fixture.config(&addr);

    let first = tokio::time::timeout(
        OUTER_BOUND,
        reasonbraid_cli::run_enroll(&cfg, "human", "keeper", None, None, true),
    )
    .await
    .expect("the first attempt returns within the outer bound");
    first.expect_err("a silent peer cannot produce an enrollment");

    let state = fixture.state();
    let recovery = state
        .bootstrap
        .as_ref()
        .expect("the pending request survives a transport refusal");
    let pending = recovery
        .pending
        .as_ref()
        .expect("the pending request is retained, not discarded");
    assert_eq!(pending.name, "keeper");
    assert!(
        pending.request_id.starts_with("req_"),
        "the retained key is the canonical request id: {}",
        pending.request_id
    );
    let original_key = pending.request_id.clone();

    // The lock is genuinely released: a second attempt opens the same store
    // rather than refusing it as busy. It reuses the SAME key, which is the
    // whole point of retaining it.
    let second = tokio::time::timeout(
        OUTER_BOUND,
        reasonbraid_cli::run_enroll(&cfg, "human", "keeper", None, None, true),
    )
    .await
    .expect("the second attempt returns within the outer bound");
    let error = second.expect_err("the peer is still silent");
    assert!(
        error.to_string().contains("transport error"),
        "the second attempt reached the transport, so the store was not locked: {error}"
    );
    assert_eq!(
        fixture
            .state()
            .bootstrap
            .expect("recovery record")
            .pending
            .expect("pending")
            .request_id,
        original_key,
        "the retained key is reused, not replaced — a new key would be a second \
         logical bootstrap against a server that may already hold the first"
    );
}

/// An origin that answers, but with more than the reply bound allows, and one
/// that redirects to somewhere the operator never configured.
async fn answering_origin() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the answering origin");
    let addr = listener.local_addr().expect("the origin's address");
    let handle = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut seen = Vec::new();
                let mut buf = [0u8; 4096];
                // Read until the request head is complete; the body follows and
                // is not needed to decide the reply.
                while !seen.windows(4).any(|w| w == b"\r\n\r\n") {
                    match stream.read(&mut buf).await {
                        Ok(0) | Err(_) => return,
                        Ok(n) => seen.extend_from_slice(&buf[..n]),
                    }
                }
                let head = String::from_utf8_lossy(&seen).to_string();
                if head.contains("/redirect") {
                    let _ = stream
                        .write_all(
                            b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:9/elsewhere\r\n\
                              Content-Length: 0\r\n\r\n",
                        )
                        .await;
                    return;
                }
                // One byte past the 8 MiB reply ceiling, streamed so the bound
                // has to refuse mid-read rather than on a declared length.
                let oversize = reasonbraid_cli::MAX_REPLY_BYTES + 1;
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                     Content-Length: {oversize}\r\n\r\n"
                );
                if stream.write_all(header.as_bytes()).await.is_err() {
                    return;
                }
                let block = vec![b'x'; 64 * 1024];
                let mut sent = 0usize;
                while sent < oversize {
                    let take = block.len().min(oversize - sent);
                    if stream.write_all(&block[..take]).await.is_err() {
                        return;
                    }
                    sent += take;
                }
            });
        }
    });
    (addr, handle)
}

/// A reply larger than the local store could ever hold is REFUSED, by name,
/// rather than buffered to the end or silently truncated. A prefix of a JSON
/// outcome is not an outcome.
#[tokio::test(flavor = "multi_thread")]
async fn an_oversized_reply_is_refused_by_name() {
    let fixture = Fixture::new("oversized-reply");
    let (addr, _origin) = answering_origin().await;
    let cfg = fixture.config(&addr);

    let started = Instant::now();
    let result = tokio::time::timeout(
        OUTER_BOUND,
        reasonbraid_cli::run_enroll(&cfg, "human", "flooded", None, None, true),
    )
    .await
    .expect("the reply bound returns well inside the outer bound");
    let error = result.expect_err("an oversized reply cannot produce an enrollment");
    let rendered = error.to_string();
    assert!(
        rendered.contains(&reasonbraid_cli::MAX_REPLY_BYTES.to_string()),
        "the refusal names the limit it enforced: {rendered}"
    );
    // Well inside the whole-request deadline, which proves this refusal is the
    // REPLY bound rather than the timeout wearing its name. The margin is large
    // on purpose: an 8 MiB loopback read takes seconds, so anything approaching
    // sixty means a different mechanism answered.
    assert!(
        started.elapsed() < reasonbraid_cli::REQUEST_TIMEOUT,
        "the refusal took {:?}, at or past the whole-request deadline — that is \
         the timeout answering, not the reply bound",
        started.elapsed()
    );
    // The key still survives: an oversized reply says nothing about whether the
    // server committed.
    assert!(
        fixture
            .state()
            .bootstrap
            .expect("recovery record")
            .pending
            .is_some(),
        "an unusable reply retains the pending request"
    );
}

/// A redirect from the CONFIGURED endpoint is reported, not followed. Following
/// it would carry the dev principal header — and a bootstrap request key — to a
/// host the operator never named.
#[tokio::test(flavor = "multi_thread")]
async fn a_redirect_off_the_configured_endpoint_is_not_followed() {
    let fixture = Fixture::new("redirect");
    let (addr, _origin) = answering_origin().await;
    let cfg = Config {
        server_base: format!("http://{addr}/redirect"),
        state_dir: fixture.0.join("store"),
    };

    let result = tokio::time::timeout(
        OUTER_BOUND,
        reasonbraid_cli::run_enroll(&cfg, "role", "redirected", Some("ten_x"), None, true),
    )
    .await
    .expect("the redirect is answered, not chased");
    let error = result.expect_err("a redirect is not an enrollment");
    let rendered = error.to_string();
    assert!(
        rendered.contains("302") || rendered.contains("server error"),
        "the 3xx is reported as the server response it is: {rendered}"
    );
    assert!(
        !rendered.contains("elsewhere"),
        "the redirect target was never contacted: {rendered}"
    );
}

// ── the configured endpoint (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.3.3`) ──────────

/// An origin that records what it was actually sent, so a control can assert on
/// the REQUEST rather than on the client's own account of it.
async fn recording_origin() -> (SocketAddr, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the recording origin");
    let addr = listener.local_addr().expect("the origin's address");
    let sink = std::sync::Arc::clone(&seen);
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let sink = std::sync::Arc::clone(&sink);
            tokio::spawn(async move {
                let mut buf = [0u8; 8192];
                if let Ok(n) = stream.read(&mut buf).await {
                    sink.lock()
                        .unwrap()
                        .push(String::from_utf8_lossy(&buf[..n]).into_owned());
                }
                let _ = stream
                    .write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n")
                    .await;
            });
        }
    });
    (addr, seen)
}

/// An ordinary verb must refuse a credential-bearing configured base BEFORE any
/// socket opens.
///
/// `canonical_server` already refuses URL credentials, a query and a fragment —
/// but only on the bootstrap path. `ApiClient::new` has 22 call sites in
/// `lib.rs` that validate nothing, so 1 of 23 construction paths checks the
/// base, and the transport forwards userinfo as Basic credentials.
///
/// The assertion is on what the ORIGIN received, because the point is that
/// nothing was sent at all.
#[tokio::test(flavor = "multi_thread")]
async fn an_ordinary_verb_refuses_a_credential_bearing_base_before_dispatch() {
    let fixture = Fixture::new("endpoint-credentials");
    let (addr, seen) = recording_origin().await;
    let cfg = Config {
        server_base: format!("http://operator:secret@{addr}"),
        state_dir: fixture.0.join("store"),
    };

    let result = tokio::time::timeout(
        OUTER_BOUND,
        reasonbraid_cli::run_enroll(&cfg, "role", "reviewer", Some("ten_x"), None, true),
    )
    .await
    .expect("the refusal returns promptly");
    let error = result.expect_err("a credential-bearing base is not a usable endpoint");

    assert!(
        seen.lock().unwrap().is_empty(),
        "the endpoint was contacted despite carrying URL credentials: {:?}",
        seen.lock().unwrap()
    );
    let rendered = error.to_string();
    assert!(
        !rendered.contains("secret"),
        "the refusal must not echo the credential it refused: {rendered}"
    );
}
