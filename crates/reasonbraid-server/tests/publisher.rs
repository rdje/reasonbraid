//! The publisher's Git-half proofs (`.4.3.2`, ADR-020 + the store contract):
//! the ref writes, the fetch-back verification, the immutable written-once
//! rule, and the effective channel's compare-and-swap. OFFLINE — the local
//! bare repository under `CARGO_TARGET_TMPDIR` (the §13 on-volume locality).

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use reasonbraid_server::publisher::{publish, PublishError};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn repo_dir() -> PathBuf {
    // The §13 on-volume locality: the bare repos live under the workspace
    // target dir (never /tmp). The per-call counter keeps the parallel
    // tests on distinct dirs.
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/publisher-tests");
    let dir = base.join(format!("pub-{n}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the dir creates");
    dir
}

fn effective_id(repo: &PathBuf) -> Option<gix::ObjectId> {
    let repo = gix::open(repo).expect("opens");
    repo.find_reference("refs/rb/effective")
        .ok()
        .map(|r| r.id().detach())
}

#[test]
fn the_publisher_writes_the_refs_and_verifies_the_fetch_back() {
    let dir = repo_dir();
    gix::init_bare(&dir).expect("the bare repo inits");
    let refs = publish(
        &dir,
        "pub-1",
        "{\"publication_id\":\"pub-1\"}",
        "# bundle",
        None,
    )
    .expect("the publish succeeds");
    assert!(!refs.publication_ref_id.is_empty());

    let repo = gix::open(&dir).expect("opens");
    let immutable = repo
        .find_reference("refs/rb/publications/pub-1")
        .expect("the immutable ref exists");
    assert_eq!(immutable.id().to_string(), refs.publication_ref_id);
    let effective = repo
        .find_reference("refs/rb/effective")
        .expect("the effective channel exists");
    assert_eq!(effective.id().to_string(), refs.effective_ref_id);
    // The fetch-back: the blobs are readable + hash to what was written.
    let blob = repo
        .rev_parse_single("refs/rb/publications/pub-1:manifest.json")
        .expect("the manifest blob resolves");
    let object = repo.find_object(blob.detach()).expect("the blob loads");
    assert_eq!(
        reasonbraid_server::fetcher::digest_sha256_hex(object.data.as_slice()),
        reasonbraid_server::fetcher::digest_sha256_hex(b"{\"publication_id\":\"pub-1\"}"),
        "the fetch-back matches"
    );

    // The second publication CASes onto the first's effective id.
    let old = effective_id(&dir).expect("the effective exists");
    let second = publish(
        &dir,
        "pub-2",
        "{\"publication_id\":\"pub-2\"}",
        "# b2",
        Some(old),
    )
    .expect("the CAS publish succeeds");
    assert_ne!(
        second.effective_ref_id, refs.effective_ref_id,
        "the channel advanced"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_stale_cas_expectation_refuses_and_the_immutable_never_moves() {
    let dir = repo_dir();
    gix::init_bare(&dir).expect("the bare repo inits");
    publish(&dir, "pub-1", "{}", "# b", None).expect("the first publish succeeds");
    let real_old = effective_id(&dir).expect("the effective exists");

    // The STALE expectation (the channel has already advanced beyond it)
    // refuses — never a force-push.
    let stale = gix::ObjectId::empty_tree(gix::hash::Kind::Sha1);
    let error =
        publish(&dir, "pub-2", "{}", "# b2", Some(stale)).expect_err("the stale CAS refuses");
    assert!(matches!(error, PublishError::CasMismatch(_)), "{error}");
    // The correct expectation succeeds.
    let _ = publish(&dir, "pub-3", "{}", "# b3", Some(real_old)).expect("the correct CAS succeeds");

    // The immutable ref never moves: the re-publish with the SAME
    // publication id refuses.
    let error = publish(&dir, "pub-1", "{}", "# b4", None).expect_err("the immutable refuses");
    assert!(matches!(error, PublishError::ImmutableExists), "{error}");
    let _ = std::fs::remove_dir_all(&dir);
}
