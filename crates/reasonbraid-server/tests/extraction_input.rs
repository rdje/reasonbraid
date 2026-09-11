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

use reasonbraid_server::extraction::{run_extraction_reporting, worker_path, WorkerLimits};
use reasonbraid_server::extraction_input::OwnedInput;

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
