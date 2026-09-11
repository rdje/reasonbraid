//! Exclusively owned R2 extraction inputs (`SIGNOFF-REPAIR.7.3.3.3.1`).
//!
//! The acquired bytes reach the extraction worker through a file. A NAME
//! proposes an input; only exclusive creation proves one. The superseded span
//! built its name from the process id and a nanosecond field in an ambient
//! temporary directory and then truncated whatever was there — which `.7.3.3.1`
//! reproduced as six colliding paths and eight worker responses describing
//! another caller's document.
//!
//! This owner creates one private file per request, on the repository's own
//! volume under a checked parent, and holds it until no worker can still be
//! reading it — either none was ever started, or the spawner observed its exit.
//! Removal additionally proves the file is still the exact one that was created
//! — same device, same inode, one link. Anything else is retained with its
//! repository-relative path named, because an input whose fate is unknown is
//! evidence, not garbage.
//!
//! Deliberately absent: any temporary-directory or home fallback, and any
//! recursive deletion. The store removes one file it created, or nothing.

use std::fs::{DirBuilder, Metadata, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use crate::extraction::WorkerCompletion;

/// How many private names may be tried before giving up. A v7 UUID does not
/// collide in practice; the bound exists so an occupied or hostile directory
/// ends in a refusal rather than an unbounded loop.
const MAX_CANDIDATES: usize = 64;

/// One request's private input file.
#[derive(Debug)]
pub struct OwnedInput {
    root: PathBuf,
    path: PathBuf,
    /// (device, inode) of the file THIS owner created.
    identity: (u64, u64),
    digest: String,
    released: bool,
}

impl OwnedInput {
    /// Create a private input holding exactly these bytes.
    pub fn create(bytes: &[u8]) -> io::Result<Self> {
        let root = repository_root()?;
        for _ in 0..MAX_CANDIDATES {
            let name = format!("input-{}", uuid::Uuid::now_v7());
            match Self::create_named(&root, &name, bytes) {
                Ok(input) => return Ok(input),
                // An occupied candidate is SKIPPED, never opened or truncated:
                // whatever is there belongs to someone else.
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::other(
            "the extraction input names are exhausted; existing files are untouched",
        ))
    }

    /// The single-candidate path, exposed to the controls that must force an
    /// occupied name — a v7 UUID cannot be made to collide on demand.
    pub(crate) fn create_named(root: &Path, name: &str, bytes: &[u8]) -> io::Result<Self> {
        let directory = storage(root)?;
        let path = directory.join(name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true).mode(0o600);
        let mut file = options.open(&path)?;

        // A file this process just created carries this process's effective
        // uid, so it is also the reference for checking that every parent is
        // OURS — without reaching outside the standard library for geteuid.
        let created = file.metadata()?;
        let owner = Self {
            root: root.to_path_buf(),
            path,
            identity: (created.dev(), created.ino()),
            digest: crate::fetcher::digest_sha256_hex(bytes),
            released: false,
        };
        if let Err(error) = owner.check_parents(created.uid(), created.dev()) {
            // Our own file, created moments ago under a parent we now distrust:
            // take it back out rather than leaving it in an unsafe place.
            let _ = std::fs::remove_file(&owner.path);
            return Err(error);
        }
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(owner)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The digest of the bytes written here, in the worker's `sha256:<hex>`
    /// form, so a caller can bind a response to the source it supplied.
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// The repository-relative path, for messages that must never carry a
    /// checkout-specific absolute path.
    pub fn relative(&self) -> &Path {
        self.path
            .strip_prefix(&self.root)
            .unwrap_or(self.path.as_path())
    }

    /// Remove the input — but only when no reader can still hold it and the
    /// file is still the exact one this owner created.
    ///
    /// An unconfirmed reader may still hold the path open, so the input is
    /// retained and the completion evidence is named. A changed identity means
    /// something replaced the file; that successor is never deleted.
    pub fn release(&mut self, completion: &WorkerCompletion) -> io::Result<()> {
        if !completion.reader_finished() {
            return Err(io::Error::other(format!(
                "the extraction input {} is retained: {completion}",
                self.relative().display()
            )));
        }
        self.verify()?;
        std::fs::remove_file(&self.path)?;
        match std::fs::symlink_metadata(&self.path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.released = true;
                Ok(())
            }
            _ => Err(io::Error::other(format!(
                "the extraction input {} removal is unconfirmed",
                self.relative().display()
            ))),
        }
    }

    fn verify(&self) -> io::Result<()> {
        let metadata = std::fs::symlink_metadata(&self.path)?;
        if !metadata.is_file()
            || (metadata.dev(), metadata.ino()) != self.identity
            || metadata.nlink() != 1
        {
            return Err(io::Error::other(format!(
                "the extraction input {} identity changed; it is retained",
                self.relative().display()
            )));
        }
        Ok(())
    }

    fn check_parents(&self, uid: u32, device: u64) -> io::Result<()> {
        let mut parent = self.path.parent().map(Path::to_path_buf);
        while let Some(current) = parent {
            let metadata = std::fs::symlink_metadata(&current)?;
            if !owned_directory(&metadata, uid, device) {
                return Err(io::Error::other(
                    "the extraction input parent is not an owned on-volume directory",
                ));
            }
            if current == self.root {
                return Ok(());
            }
            parent = current.parent().map(Path::to_path_buf);
        }
        Err(io::Error::other(
            "the extraction input escaped the repository root",
        ))
    }
}

impl Drop for OwnedInput {
    fn drop(&mut self) {
        if !self.released {
            eprintln!(
                "retained extraction input: {} (its reader was never confirmed finished)",
                self.relative().display()
            );
        }
    }
}

/// A directory this process owns, on the expected volume, not writable by
/// group or other, and not a symbolic link (`symlink_metadata` never follows).
fn owned_directory(metadata: &Metadata, uid: u32, device: u64) -> bool {
    metadata.is_dir()
        && metadata.dev() == device
        && metadata.uid() == uid
        && metadata.mode() & 0o022 == 0
}

/// The repository root discovered at RUNTIME from the current directory.
/// Nothing persists an absolute path, so moving the checkout changes these
/// locations without an edit.
///
/// The test is deliberately stricter than the browser worker's `Cargo.toml` +
/// `migrations` pair: `crates/reasonbraid-node` satisfies that pair, so a
/// process whose working directory sat there would stop at the crate and place
/// private storage inside it. `rust-toolchain.toml` exists only at the real
/// root. (The browser worker's own predicate is not changed here; it is routed
/// to `SIGNOFF-REPAIR.7.3.2`, which owns that worker's storage boundary.)
fn repository_root() -> io::Result<PathBuf> {
    let current = std::env::current_dir()?;
    current
        .ancestors()
        .find(|path| {
            path.join("Cargo.toml").is_file()
                && path.join("migrations").is_dir()
                && path.join("rust-toolchain.toml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("run the extraction pipeline from within the repository"))
}

/// `<root>/.project-data/extraction`, created 0700. The parents are checked
/// against the created file's own ownership afterwards, so creating them here
/// is not by itself a trust decision.
fn storage(root: &Path) -> io::Result<PathBuf> {
    let mut path = root.to_path_buf();
    for component in [".project-data", "extraction"] {
        path.push(component);
        match DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => (),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
            Err(error) => return Err(error),
        }
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_digest_is_the_worker_s_own_form_over_the_written_bytes() {
        let input = OwnedInput::create(b"the acquired document").expect("the input is created");
        assert_eq!(
            input.digest(),
            crate::fetcher::digest_sha256_hex(b"the acquired document")
        );
        assert!(input.digest().starts_with("sha256:"));
        let mut input = input;
        input
            .release(&WorkerCompletion::Consumed {
                success: true,
                status: "exit status: 0".to_owned(),
            })
            .expect("a confirmed reader releases the input");
    }

    #[test]
    fn an_unconfirmed_reader_retains_the_input_and_names_its_evidence() {
        let mut input = OwnedInput::create(b"retained").expect("the input is created");
        let error = input
            .release(&WorkerCompletion::Unconfirmed {
                pid: 4242,
                detail: "the worker did not exit".to_owned(),
            })
            .expect_err("an unconfirmed reader must not release the input");
        let message = error.to_string();
        assert!(message.contains("is retained"), "{message}");
        assert!(message.contains("pid 4242"), "{message}");
        assert!(input.path().exists(), "the input stays on disk");
        assert!(
            !message.starts_with('/'),
            "the message must not carry an absolute path: {message}"
        );
        input
            .release(&WorkerCompletion::Consumed {
                success: false,
                status: "signal: 9 (SIGKILL)".to_owned(),
            })
            .expect("a later confirmed completion releases it");
    }

    fn consumed() -> WorkerCompletion {
        WorkerCompletion::Consumed {
            success: true,
            status: "exit status: 0".to_owned(),
        }
    }

    fn unique_name(label: &str) -> String {
        format!(
            "control-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::now_v7()
        )
    }

    /// The root is found by walking ancestors, so the store is the REPOSITORY's
    /// even though a test binary runs with the crate directory as its cwd.
    #[test]
    fn the_store_resolves_to_the_repository_root_not_the_crate() {
        let mut input = OwnedInput::create(b"root discovery").expect("the input is created");
        let relative = input.relative().to_path_buf();
        assert!(
            relative.starts_with(".project-data/extraction"),
            "the input lives in the repository store: {}",
            relative.display()
        );
        assert!(
            std::env::current_dir()
                .unwrap()
                .ends_with("crates/reasonbraid-server"),
            "this control is only meaningful from the crate directory"
        );
        input.release(&consumed()).expect("released");
    }

    /// An occupied candidate is skipped whole: not opened, not truncated, not
    /// adopted. Its bytes and its identity survive untouched.
    #[test]
    fn an_occupied_candidate_name_is_never_opened_or_truncated() {
        let root = repository_root().expect("the repository root");
        let name = unique_name("occupied");
        let squatter = storage(&root).expect("the store").join(&name);
        std::fs::write(&squatter, b"another owner's document").expect("the squatter is written");
        let before = std::fs::symlink_metadata(&squatter).expect("the squatter exists");

        let error = OwnedInput::create_named(&root, &name, b"mine")
            .expect_err("an occupied name must refuse");
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);

        let after = std::fs::symlink_metadata(&squatter).expect("the squatter survives");
        assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
        assert_eq!(
            std::fs::read(&squatter).expect("readable"),
            b"another owner's document",
            "the occupant's bytes are unchanged"
        );
        std::fs::remove_file(&squatter).expect("the control cleans up its own squatter");
    }

    /// A symlink sitting at a candidate name is refused at creation, so nothing
    /// is written THROUGH it: the target is never reached.
    #[test]
    fn a_symlinked_candidate_name_is_refused_and_its_target_untouched() {
        let root = repository_root().expect("the repository root");
        let store = storage(&root).expect("the store");
        let target = store.join(unique_name("link-target"));
        std::fs::write(&target, b"the link target").expect("the target is written");
        let link = store.join(unique_name("link"));
        std::os::unix::fs::symlink(&target, &link).expect("the link is created");

        let error =
            OwnedInput::create_named(&root, link.file_name().unwrap().to_str().unwrap(), b"mine")
                .expect_err("a linked name must refuse");
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            std::fs::read(&target).expect("readable"),
            b"the link target",
            "the link target is never written through"
        );
        std::fs::remove_file(&link).expect("cleanup");
        std::fs::remove_file(&target).expect("cleanup");
    }

    /// A replaced input is a different file. The owner refuses to delete the
    /// successor, whatever the reader's completion says.
    #[test]
    fn a_replaced_input_is_retained_and_its_successor_survives() {
        let mut input = OwnedInput::create(b"original").expect("the input is created");
        let path = input.path().to_path_buf();
        std::fs::remove_file(&path).expect("the original is removed out from under the owner");
        std::fs::write(&path, b"a successor").expect("a successor takes the name");

        let error = input
            .release(&consumed())
            .expect_err("a changed identity must refuse");
        let message = error.to_string();
        assert!(message.contains("identity changed"), "{message}");
        assert_eq!(
            std::fs::read(&path).expect("readable"),
            b"a successor",
            "the successor is never deleted"
        );
        std::fs::remove_file(&path).expect("the control removes the successor it planted");
    }

    /// Dropping an owner that was never released retains the file and names it
    /// relatively — the operator can find it; nothing silently disappears.
    #[test]
    fn dropping_an_unreleased_owner_retains_the_input() {
        let path = {
            let input = OwnedInput::create(b"dropped").expect("the input is created");
            input.path().to_path_buf()
        };
        assert!(path.exists(), "the dropped input is retained");
        std::fs::remove_file(&path).expect("the control removes what it created");
    }

    /// Thirty-two simultaneous creators: thirty-two distinct files, each with
    /// exactly its own bytes. This is the property the superseded
    /// process-id-and-nanoseconds name could not provide.
    #[test]
    fn simultaneous_creators_never_share_a_path_or_a_document() {
        let owners: Vec<_> = (0..32u32)
            .map(|index| {
                std::thread::spawn(move || {
                    let bytes = format!("document {index}").into_bytes();
                    let input = OwnedInput::create(&bytes).expect("the input is created");
                    (index, bytes, input)
                })
            })
            .collect();
        let mut results: Vec<_> = owners
            .into_iter()
            .map(|handle| handle.join().expect("the creator finished"))
            .collect();

        let mut paths: Vec<_> = results
            .iter()
            .map(|(_, _, input)| input.path().to_path_buf())
            .collect();
        paths.sort();
        paths.dedup();
        assert_eq!(paths.len(), 32, "every creator owns a distinct path");

        for (index, bytes, input) in &results {
            assert_eq!(
                &std::fs::read(input.path()).expect("readable"),
                bytes,
                "creator {index} still holds exactly its own document"
            );
            assert_eq!(input.digest(), crate::fetcher::digest_sha256_hex(bytes));
        }
        for (_, _, input) in results.iter_mut() {
            input.release(&consumed()).expect("released");
        }
    }

    /// A worker that never started cannot be holding the input, so this is the
    /// one failure that releases: refusing here would litter a private store on
    /// every absent binary and every failed spawn.
    #[test]
    fn a_never_started_worker_releases_the_input_it_never_read() {
        let mut input = OwnedInput::create(b"never started").expect("the input is created");
        let path = input.path().to_path_buf();
        input
            .release(&WorkerCompletion::NeverStarted)
            .expect("no reader ever existed");
        assert!(!path.exists(), "the input is removed");
    }
}
