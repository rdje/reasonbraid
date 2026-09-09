//! The publisher's Git-half proofs (`.4.3.2`, ADR-020 + the store contract):
//! the ref writes, the fetch-back verification, the immutable written-once
//! rule, and the effective channel's compare-and-swap. OFFLINE — the local
//! bare repositories in exclusively owned `target/publisher-tests` directories.
//! Retain uncompleted fixtures; explicitly finish only after repository handles close.

#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};

use reasonbraid_server::publisher::{publish, PublishError};
use uuid::Uuid;

struct Fixture {
    path: PathBuf,
    identity: (u64, u64),
    finished: bool,
}

impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root exists");
        let device = std::fs::metadata(&root).unwrap().dev();
        let mut parent = root;
        for component in ["target", "publisher-tests"] {
            parent.push(component);
            match std::fs::DirBuilder::new().mode(0o700).create(&parent) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                Err(e) => panic!("cannot create publisher fixture parent: {e}"),
            }
            let meta = std::fs::symlink_metadata(&parent).unwrap();
            assert!(meta.is_dir() && !meta.file_type().is_symlink());
            assert_eq!(meta.dev(), device, "fixture parent must stay on volume");
        }
        let fixture = Self::create(parent.join(format!("pub-{}", Uuid::now_v7())))
            .expect("exclusive private fixture creates without replacing anything");
        assert_eq!(fixture.identity.0, device);
        eprintln!("publisher fixture created: {}", fixture.path.display());
        fixture
    }

    // Private callers supply a validated parent owned by this test or fixture.
    // A collision is an error, never permission to remove the existing entry.
    fn create(path: PathBuf) -> io::Result<Self> {
        std::fs::DirBuilder::new().mode(0o700).create(&path)?;
        let metadata = std::fs::symlink_metadata(&path)?;
        Ok(Self {
            path,
            identity: (metadata.dev(), metadata.ino()),
            finished: false,
        })
    }

    // Call only after all repository handles and readers have left scope.
    fn finish(mut self) -> io::Result<()> {
        let metadata = std::fs::symlink_metadata(&self.path)?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || (metadata.dev(), metadata.ino()) != self.identity
        {
            return Err(io::Error::other(
                "publisher fixture identity changed; retain data",
            ));
        }
        std::fs::remove_dir_all(&self.path)?;
        match std::fs::symlink_metadata(&self.path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                self.finished = true;
                Ok(())
            }
            _ => Err(io::Error::other("publisher fixture cleanup not confirmed")),
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if !self.finished {
            eprintln!("publisher fixture retained: {}", self.path.display());
        }
    }
}

fn effective_id(repo: &Path) -> Option<gix::ObjectId> {
    let repo = gix::open(repo).expect("opens");
    repo.find_reference("refs/rb/effective")
        .ok()
        .map(|r| r.id().detach())
}

#[test]
fn the_publisher_writes_the_refs_and_verifies_the_fetch_back() {
    let fixture = Fixture::new();
    let dir = &fixture.path;
    gix::init_bare(dir).expect("the bare repo inits");
    let refs = publish(
        dir,
        "pub-1",
        "{\"publication_id\":\"pub-1\"}",
        "# bundle",
        None,
    )
    .expect("the publish succeeds");
    assert!(!refs.publication_ref_id.is_empty());

    {
        let repo = gix::open(dir).expect("opens");
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
    }

    // The second publication CASes onto the first's effective id.
    let old = effective_id(dir).expect("the effective exists");
    let second = publish(
        dir,
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
    fixture.finish().expect("owned fixture cleanup completes");
}

#[test]
fn the_stale_cas_expectation_refuses_and_the_immutable_never_moves() {
    let fixture = Fixture::new();
    let dir = &fixture.path;
    gix::init_bare(dir).expect("the bare repo inits");
    publish(dir, "pub-1", "{}", "# b", None).expect("the first publish succeeds");
    let real_old = effective_id(dir).expect("the effective exists");

    // The STALE expectation (the channel has already advanced beyond it)
    // refuses — never a force-push.
    let stale = gix::ObjectId::empty_tree(gix::hash::Kind::Sha1);
    let error =
        publish(dir, "pub-2", "{}", "# b2", Some(stale)).expect_err("the stale CAS refuses");
    assert!(matches!(error, PublishError::CasMismatch(_)), "{error}");
    // The correct expectation succeeds.
    let _ = publish(dir, "pub-3", "{}", "# b3", Some(real_old)).expect("the correct CAS succeeds");

    // The immutable ref never moves: the re-publish with the SAME
    // publication id refuses.
    let error = publish(dir, "pub-1", "{}", "# b4", None).expect_err("the immutable refuses");
    assert!(matches!(error, PublishError::ImmutableExists), "{error}");
    fixture.finish().expect("owned fixture cleanup completes");
}

#[test]
fn an_existing_fixture_is_never_replaced() {
    let fixture = Fixture::new();
    let witness = fixture.path.join("witness");
    std::fs::write(&witness, b"preserve this owner").unwrap();
    let error = Fixture::create(fixture.path.clone())
        .err()
        .expect("collision refuses");
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read(&witness).unwrap(), b"preserve this owner");
    fixture.finish().unwrap();
}

#[test]
fn uncompleted_fixtures_retain_their_evidence() {
    let owner = Fixture::new();
    let path = owner.path.join("incomplete");
    let fixture = Fixture::create(path.clone()).unwrap();
    std::fs::write(path.join("witness"), b"diagnostic evidence").unwrap();
    drop(fixture);
    assert_eq!(
        std::fs::read(path.join("witness")).unwrap(),
        b"diagnostic evidence"
    );
    // This successful outer control owns both its data and its retained child.
    owner.finish().unwrap();
}

#[test]
fn cleanup_refuses_a_replaced_directory() {
    let owner = Fixture::new();
    let path = owner.path.join("child");
    let fixture = Fixture::create(path.clone()).unwrap();
    let original = owner.path.join("original");
    std::fs::rename(&path, &original).unwrap();
    std::fs::create_dir(&path).unwrap();
    std::fs::write(path.join("witness"), b"replacement").unwrap();
    let error = fixture.finish().unwrap_err();
    assert!(error.to_string().contains("identity changed"));
    assert_eq!(std::fs::read(path.join("witness")).unwrap(), b"replacement");
    assert!(original.is_dir());
    owner.finish().unwrap();
}
