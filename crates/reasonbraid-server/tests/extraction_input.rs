//! The owned extraction input, through the real spawner
//! (`SIGNOFF-REPAIR.7.3.3.3.1`).
//!
//! The storage mechanics — exclusive creation, occupied and linked candidates,
//! replacement, retention, thirty-two simultaneous creators — are controls in
//! the module itself, where the single-candidate seam lives. What is proved
//! here is the join: an input this process owns, read by the ACTUAL extraction
//! worker, comes back described by its own digest, and is released only once
//! that worker is known to be finished.

#![cfg(unix)]

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Duration;

use reasonbraid_server::extraction::{
    run_extraction_reporting, worker_path, ExtractionError, WorkerLimits, WorkerResponse,
};
use reasonbraid_server::extraction_input::{extract_acquired_bytes, OwnedInput};

const FEED: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Owned Input Feed</title>
  <entry><title>One</title><summary>the first</summary></entry>
</feed>"#;

/// `R2_WORKER_BIN` is process-wide. EVERY scenario holds this across worker
/// selection AND the spawner call — including the one that sets no override.
/// A control that reads ambient selection without the lock can be handed
/// another control's worker: the probe preserved under
/// `target/extraction-owned-input-controls` shows exactly that, an
/// unsynchronized caller receiving the stub's `sha256:0000…` parent digest.
fn worker_selection() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = LOCK.get_or_init(|| Mutex::new(()));
    lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::current_dir()
        .expect("cwd")
        .ancestors()
        .find(|path| {
            path.join("Cargo.toml").is_file() && path.join("rust-toolchain.toml").is_file()
        })
        .expect("run inside the repository")
        .to_path_buf();
    let dir = root.join("target/extraction-input-controls-fixtures");
    std::fs::create_dir_all(&dir).expect("the control scratch is created");
    dir.join(format!("{name}-{}", std::process::id()))
}

/// The whole point of owning the input: the worker's receipt describes the
/// exact bytes this caller supplied, so a response can be bound to its source
/// instead of trusted because it arrived.
#[test]
fn the_real_worker_describes_the_owned_input_by_its_own_digest() {
    let binary = {
        let guard = worker_selection();
        let binary = worker_path();
        drop(guard);
        binary
    };
    if !binary.exists() {
        println!("SKIP: the extraction worker is absent at {binary:?} — run `cargo test --all`");
        return;
    }
    let mut input = OwnedInput::create(FEED).expect("the input is created");
    let run = {
        let guard = worker_selection();
        let run = run_extraction_reporting(
            input.path(),
            "application/atom+xml",
            WorkerLimits::default(),
            Duration::from_secs(30),
        );
        drop(guard);
        run
    };
    let response = run.result.expect("the real worker responds");
    assert_eq!(
        response.parent_digest,
        input.digest(),
        "the worker's parent digest is the digest of the bytes this caller owns"
    );
    let joined: String = response
        .chunks
        .iter()
        .map(|chunk| chunk.text.as_str())
        .collect::<Vec<_>>()
        .join("|");
    assert!(joined.contains("Owned Input Feed"), "{joined}");

    assert!(
        run.completion.reader_finished(),
        "the reader must be finished before the input is released: {}",
        run.completion
    );
    let path = input.path().to_path_buf();
    input
        .release(&run.completion)
        .expect("the input is released");
    assert!(!path.exists(), "the released input is gone");
}

/// A worker that names bytes this caller never supplied is DETECTABLE at the
/// spawner's own boundary. The refusal that consumes this observation belongs
/// to the API wiring (`.7.3.3.3.2`); what is proved here is that the two
/// digests disagree and the input is still owned when they do.
#[test]
fn a_response_describing_other_bytes_disagrees_with_the_owned_input() {
    let stub = scratch("wrong-digest-worker.sh");
    std::fs::write(
        &stub,
        "#!/bin/sh\ncat > /dev/null\nprintf '%s\\n' '{\"parent_digest\":\"sha256:0000000000000000000000000000000000000000000000000000000000000000\",\"chunks\":[{\"digest\":\"sha256:aa\",\"text\":\"another document\"}],\"excluded\":[],\"extractor_version\":\"0.1.0\"}'\n",
    )
    .expect("the stub is written");
    let mut permissions = std::fs::metadata(&stub)
        .expect("stub metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o700);
    std::fs::set_permissions(&stub, permissions).expect("the stub is executable");

    let mut input = OwnedInput::create(FEED).expect("the input is created");
    let run = {
        let guard = worker_selection();
        std::env::set_var("R2_WORKER_BIN", &stub);
        let run = run_extraction_reporting(
            input.path(),
            "application/atom+xml",
            WorkerLimits::default(),
            Duration::from_secs(30),
        );
        std::env::remove_var("R2_WORKER_BIN");
        drop(guard);
        run
    };
    let response = run.result.expect("the stub responds");
    assert_ne!(
        response.parent_digest,
        input.digest(),
        "a response for other bytes must not match the owned input's digest"
    );
    assert!(
        input.path().exists(),
        "the input is still owned when its response is unusable"
    );
    input
        .release(&run.completion)
        .expect("the input is released");
    std::fs::remove_file(&stub).expect("the control removes its own stub");
}

// ---------------------------------------------------------------------------
// The whole R2 boundary in one call (`.7.3.3.3.2`): the bytes become an owned
// input, the worker reads it, the response is bound to those bytes, and the
// input is released only once no reader can hold it. This is the entrypoint the
// R2 API now uses, so these controls cover the API's extraction leg without a
// database or an HTTP origin.
// ---------------------------------------------------------------------------

fn run_bound(media_type: &str, bytes: &[u8]) -> Result<WorkerResponse, ExtractionError> {
    let guard = worker_selection();
    let result = extract_acquired_bytes(
        bytes,
        media_type,
        WorkerLimits::default(),
        Duration::from_secs(30),
    );
    drop(guard);
    result
}

fn run_bound_against(
    stub: &std::path::Path,
    bytes: &[u8],
) -> Result<WorkerResponse, ExtractionError> {
    let guard = worker_selection();
    std::env::set_var("R2_WORKER_BIN", stub);
    let result = extract_acquired_bytes(
        bytes,
        "application/atom+xml",
        WorkerLimits::default(),
        Duration::from_secs(30),
    );
    std::env::remove_var("R2_WORKER_BIN");
    drop(guard);
    result
}

fn executable_stub(name: &str, body: &str) -> PathBuf {
    let path = scratch(name);
    std::fs::write(&path, body).expect("the stub is written");
    let mut permissions = std::fs::metadata(&path)
        .expect("stub metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o700);
    std::fs::set_permissions(&path, permissions).expect("the stub is executable");
    path
}

/// The ordinary success path, end to end through the real worker.
#[test]
fn the_bound_extraction_returns_a_receipt_for_the_supplied_bytes() {
    if !worker_path().exists() {
        println!("SKIP: the extraction worker is absent — run `cargo test --all`");
        return;
    }
    let response = run_bound("application/atom+xml", FEED).expect("the real worker responds");
    assert_eq!(
        response.parent_digest,
        reasonbraid_server::fetcher::digest_sha256_hex(FEED),
        "the receipt describes exactly the supplied bytes"
    );
    assert!(response
        .chunks
        .iter()
        .any(|chunk| chunk.text.contains("Owned Input Feed")));
}

/// A named worker refusal survives the boundary verbatim.
#[test]
fn a_named_refusal_survives_the_bound_extraction() {
    if !worker_path().exists() {
        println!("SKIP: the extraction worker is absent — run `cargo test --all`");
        return;
    }
    match run_bound("application/atom+xml", b"not a feed") {
        Err(ExtractionError::WorkerRefused { kind, .. }) => assert_eq!(kind, "feed_unreadable"),
        other => panic!("the named refusal must survive: {other:?}"),
    }
}

/// THE refusal this child exists for: a response describing bytes the caller
/// never supplied is rejected, with both digests named, so nothing derived from
/// it can be persisted.
#[test]
fn a_response_for_other_bytes_is_refused_before_it_can_be_persisted() {
    let stub = executable_stub(
        "mismatch-worker.sh",
        "#!/bin/sh\ncat > /dev/null\nprintf '%s\\n' '{\"parent_digest\":\"sha256:0000000000000000000000000000000000000000000000000000000000000000\",\"chunks\":[{\"digest\":\"sha256:aa\",\"text\":\"another document\"}],\"excluded\":[],\"extractor_version\":\"0.1.0\"}'\n",
    );
    match run_bound_against(&stub, FEED) {
        Err(ExtractionError::SourceMismatch { expected, received }) => {
            assert_eq!(
                expected,
                reasonbraid_server::fetcher::digest_sha256_hex(FEED)
            );
            assert_eq!(
                received,
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            );
        }
        other => panic!("a response for other bytes must be refused: {other:?}"),
    }
    std::fs::remove_file(&stub).expect("the control removes its own stub");
}

/// A worker failure keeps its own classification through the boundary.
#[test]
fn a_worker_failure_keeps_its_classification_through_the_boundary() {
    let stub = executable_stub("failing-worker.sh", "#!/bin/sh\ncat > /dev/null\nexit 3\n");
    match run_bound_against(&stub, FEED) {
        Err(ExtractionError::RequestFailed(detail)) => {
            assert!(detail.contains("the worker exited with"), "{detail}");
        }
        other => panic!("a worker failure must surface: {other:?}"),
    }
    std::fs::remove_file(&stub).expect("the control removes its own stub");
}

/// Concurrent requests never see each other's document: each receipt describes
/// the bytes its own caller supplied. This is the property the superseded
/// process-id-and-nanoseconds input could not provide.
#[test]
fn concurrent_bound_extractions_each_describe_their_own_document() {
    if !worker_path().exists() {
        println!("SKIP: the extraction worker is absent — run `cargo test --all`");
        return;
    }
    let callers: Vec<_> = (0..8u32)
        .map(|index| {
            std::thread::spawn(move || {
                let bytes = format!(
                    "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\">\n  <title>Caller {index}</title>\n  <entry><title>Entry {index}</title><summary>body {index}</summary></entry>\n</feed>"
                )
                .into_bytes();
                let response = run_bound("application/atom+xml", &bytes)
                    .unwrap_or_else(|error| panic!("caller {index} failed: {error}"));
                (index, bytes, response)
            })
        })
        .collect();
    for handle in callers {
        let (index, bytes, response) = handle.join().expect("the caller finished");
        assert_eq!(
            response.parent_digest,
            reasonbraid_server::fetcher::digest_sha256_hex(&bytes),
            "caller {index} received a receipt for its own document"
        );
        let joined: String = response
            .chunks
            .iter()
            .map(|chunk| chunk.text.as_str())
            .collect::<Vec<_>>()
            .join("|");
        assert!(
            joined.contains(&format!("Caller {index}")),
            "caller {index} received another caller's text: {joined}"
        );
    }
}
