//! Repository-derived private storage (`SIGNOFF-REPAIR.7.2.1`).
//!
//! Two owners in this crate need the same thing: a private place to put bytes
//! that belongs to the repository's own volume, is created exclusively rather
//! than adopted, and is removed only after proving the path still names what
//! the owner created. `extraction_input` established the shape for a FILE;
//! this module holds the shared parts and adds the DIRECTORY owner the git
//! acquisition needs.
//!
//! Deliberately absent: any temporary-directory or home fallback. A name
//! proposes storage; only exclusive creation proves it.

use std::fs::{DirBuilder, File, Metadata};
use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};

/// How many private names may be tried before giving up. A v7 UUID does not
/// collide in practice; the bound exists so an occupied or hostile directory
/// ends in a refusal rather than an unbounded loop.
pub(crate) const MAX_CANDIDATES: usize = 64;

/// The repository root discovered at RUNTIME from the current directory.
/// Nothing persists an absolute path, so moving the checkout changes these
/// locations without an edit.
///
/// The test is deliberately stricter than a `Cargo.toml` + `migrations` pair:
/// `crates/reasonbraid-node` satisfies that pair, so a process whose working
/// directory sat there would stop at the crate and place private storage
/// inside it. `rust-toolchain.toml` exists only at the real root.
pub fn repository_root() -> io::Result<PathBuf> {
    let current = std::env::current_dir()?;
    current
        .ancestors()
        .find(|path| {
            path.join("Cargo.toml").is_file()
                && path.join("migrations").is_dir()
                && path.join("rust-toolchain.toml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("run this from within the repository"))
}

/// `<root>/.project-data/<area>`, created 0700. The parents are checked
/// against a created entry's own ownership afterwards, so creating them here
/// is not by itself a trust decision.
pub(crate) fn storage(root: &Path, area: &str) -> io::Result<PathBuf> {
    let mut path = root.to_path_buf();
    for component in [".project-data", area] {
        path.push(component);
        match DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => (),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
            Err(error) => return Err(error),
        }
    }
    Ok(path)
}

/// A directory this process owns, on the expected volume, not writable by
/// group or other, and not a symbolic link (`symlink_metadata` never follows).
pub(crate) fn owned_directory(metadata: &Metadata, uid: u32, device: u64) -> bool {
    metadata.is_dir()
        && metadata.dev() == device
        && metadata.uid() == uid
        && metadata.mode() & 0o022 == 0
}

/// Every parent of `path` up to and including `root` is an owned on-volume
/// directory. `uid` and `device` come from an entry this process just created,
/// so the reference never reaches outside the standard library for `geteuid`.
pub(crate) fn check_parents(
    path: &Path,
    root: &Path,
    subject: &str,
    uid: u32,
    device: u64,
) -> io::Result<()> {
    let mut parent = path.parent().map(Path::to_path_buf);
    while let Some(current) = parent {
        let metadata = std::fs::symlink_metadata(&current)?;
        if !owned_directory(&metadata, uid, device) {
            return Err(io::Error::other(format!(
                "the {subject} parent is not an owned on-volume directory"
            )));
        }
        if current == root {
            return Ok(());
        }
        parent = current.parent().map(Path::to_path_buf);
    }
    Err(io::Error::other(format!(
        "the {subject} escaped the repository root"
    )))
}

/// The repository-relative form of `path`, for messages that must never carry
/// a checkout-specific absolute path.
pub(crate) fn relative<'a>(path: &'a Path, root: &Path) -> &'a Path {
    path.strip_prefix(root).unwrap_or(path)
}

/// One exclusively created working directory on the repository's own volume.
///
/// The superseded git acquisition built its name from the process id and a
/// nanosecond field in an ambient temporary directory, then called
/// `create_dir_all`, which ADOPTS an existing directory instead of refusing.
/// The `.7.2.1` probe measured that naming at 501 distinct values in 2000
/// calls, every collision between adjacent calls — so two concurrent
/// acquisitions in one process, which share the process id by construction,
/// could clone into one directory, and either one's error cleanup would
/// recursively delete the other's objects.
#[derive(Debug)]
pub struct OwnedDirectory {
    root: PathBuf,
    path: PathBuf,
    /// An open descriptor on the directory THIS owner created, held for the
    /// owner's whole life. It pins the inode, so a successor at the same path
    /// cannot present the same `(device, inode)` pair and be deleted in its
    /// place — measured at 0 reuses in 200 create/remove/recreate cycles on
    /// the repository volume.
    handle: File,
    subject: String,
    released: bool,
}

impl OwnedDirectory {
    /// Create a private working directory under `<root>/.project-data/<area>`.
    pub fn create(area: &str, prefix: &str) -> io::Result<Self> {
        let root = repository_root()?;
        let directory = storage(&root, area)?;
        for _ in 0..MAX_CANDIDATES {
            let name = format!("{prefix}-{}", uuid::Uuid::now_v7());
            match Self::create_named(&root, &directory, &name, area) {
                Ok(owner) => return Ok(owner),
                // An occupied candidate is SKIPPED, never adopted or emptied:
                // whatever is there belongs to someone else.
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::other(format!(
            "the {area} workspace names are exhausted; existing directories are untouched"
        )))
    }

    /// The single-candidate path, exposed to the controls that must force an
    /// occupied name — a v7 UUID cannot be made to collide on demand.
    pub(crate) fn create_named(
        root: &Path,
        directory: &Path,
        name: &str,
        area: &str,
    ) -> io::Result<Self> {
        let path = directory.join(name);
        // `create`, NOT `create_all`: an existing path must REFUSE.
        DirBuilder::new().mode(0o700).create(&path)?;
        let handle = File::open(&path)?;
        let created = handle.metadata()?;
        let owner = Self {
            root: root.to_path_buf(),
            path,
            handle,
            subject: format!("{area} workspace"),
            released: false,
        };
        if let Err(error) = check_parents(
            &owner.path,
            &owner.root,
            &owner.subject,
            created.uid(),
            created.dev(),
        ) {
            // Our own directory, created moments ago under a parent we now
            // distrust: take it back out rather than leaving it in an unsafe
            // place. It is empty and provably ours, so this cannot reach
            // anyone else's bytes.
            let _ = std::fs::remove_dir(&owner.path);
            return Err(error);
        }
        Ok(owner)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The repository-relative path, for messages that must never carry a
    /// checkout-specific absolute path.
    pub fn relative(&self) -> &Path {
        relative(&self.path, &self.root)
    }

    /// Remove the directory and everything this owner put in it — but only
    /// once the path is proved to still name the directory this owner created.
    /// A changed identity means something replaced it; that successor is never
    /// deleted.
    pub fn release(&mut self) -> io::Result<()> {
        match self.remove() {
            Ok(()) => {
                self.released = true;
                Ok(())
            }
            Err(error) => {
                crate::log_event!(
                    "project_workspace_retained",
                    "workspace" => self.relative().display().to_string(),
                    "reason" => error.to_string(),
                );
                Err(error)
            }
        }
    }

    fn remove(&self) -> io::Result<()> {
        match self.verify() {
            Ok(()) => (),
            // Already gone: nothing to delete, and nothing was wrongly
            // deleted. The fact is recorded, not treated as a failure.
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                crate::log_event!(
                    "project_workspace_absent",
                    "workspace" => self.relative().display().to_string(),
                    "reason" => "the workspace was already removed by something else",
                );
                return Ok(());
            }
            Err(error) => return Err(error),
        }
        std::fs::remove_dir_all(&self.path)?;
        match std::fs::symlink_metadata(&self.path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            _ => Err(io::Error::other(format!(
                "the {} {} removal is unconfirmed",
                self.subject,
                self.relative().display()
            ))),
        }
    }

    /// Prove the path still names the directory this owner created.
    ///
    /// For a FILE, `extraction_input` also checks the descriptor's link count,
    /// because Linux reuses an inode number as soon as it is freed. That check
    /// is deliberately NOT used here: a directory's link count grows as
    /// subdirectories appear inside it, and macOS was measured still reporting
    /// `nlink == 2` on a held descriptor AFTER the directory was removed, so
    /// it carries no signal for this case. The held descriptor does the work
    /// instead — it pins the inode, so no successor can be handed the same
    /// `(device, inode)` pair while this owner is alive.
    fn verify(&self) -> io::Result<()> {
        let created = self.handle.metadata()?;
        let metadata = std::fs::symlink_metadata(&self.path)?;
        if !metadata.is_dir() || (metadata.dev(), metadata.ino()) != (created.dev(), created.ino())
        {
            return Err(io::Error::other(format!(
                "the {} {} identity changed; it is retained",
                self.subject,
                self.relative().display()
            )));
        }
        Ok(())
    }
}

impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        if !self.released {
            // The whole point of owning the directory is that it does not
            // outlive its owner. A failure here is already reported by
            // `release`, so the result is deliberately discarded.
            let _ = self.release();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// The name must be proved by creation, not proposed by a clock. The
    /// superseded git acquisition named its directory from the process id and
    /// a nanosecond field; a probe measured 501 distinct values in 2000 calls,
    /// and two concurrent acquisitions in one process share the process id by
    /// construction.
    #[test]
    fn simultaneous_creators_never_share_a_workspace() {
        let owners: Vec<OwnedDirectory> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..16)
                .map(|_| {
                    scope.spawn(|| {
                        OwnedDirectory::create("git", "control-concurrent")
                            .expect("the workspace creates")
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("the creator finishes"))
                .collect()
        });

        let mut paths: Vec<_> = owners
            .iter()
            .map(|owner| owner.path().to_path_buf())
            .collect();
        // Every one exists AT THE SAME TIME, so distinctness is a property of
        // the live set rather than of names taken in turn.
        for path in &paths {
            assert!(path.is_dir(), "{} is live", path.display());
        }
        let live = paths.len();
        paths.sort();
        paths.dedup();
        assert_eq!(paths.len(), live, "every concurrent workspace is distinct");
    }

    /// An occupied name is REFUSED. The superseded call was `create_dir_all`,
    /// which returns `Ok` for a directory that is already there — so it
    /// adopted another acquisition's working directory, content and all.
    #[test]
    fn an_occupied_name_is_refused_rather_than_adopted() {
        let root = repository_root().expect("the repository root");
        let directory = storage(&root, "git").expect("the store");
        let name = format!("control-occupied-{}", uuid::Uuid::now_v7());

        let mut first = OwnedDirectory::create_named(&root, &directory, &name, "git")
            .expect("the first creation succeeds");
        let foreign = first.path().join("another-acquisition.pack");
        let mut file = std::fs::File::create(&foreign).expect("the foreign content creates");
        file.write_all(b"objects belonging to a different acquisition")
            .expect("the foreign content writes");
        drop(file);

        let second = OwnedDirectory::create_named(&root, &directory, &name, "git");
        let error = second.expect_err("the occupied name refuses");
        assert_eq!(
            error.kind(),
            io::ErrorKind::AlreadyExists,
            "the refusal names the occupancy"
        );
        assert!(
            foreign.is_file(),
            "the first owner's content is untouched by the refused creation"
        );
        first.release().expect("the first owner releases");
    }

    /// Removal deletes the directory this owner created, or nothing. The
    /// superseded error path called `remove_dir_all` on a NAME, so it deleted
    /// whatever had taken that name.
    #[test]
    fn a_replaced_workspace_is_retained_and_its_successor_survives() {
        let mut owner =
            OwnedDirectory::create("git", "control-replaced").expect("the workspace creates");
        let path = owner.path().to_path_buf();

        // Something else removes it and puts its own directory in its place.
        std::fs::remove_dir_all(&path).expect("the original removes");
        std::fs::create_dir(&path).expect("the successor creates");
        let successor_content = path.join("successor.pack");
        std::fs::write(&successor_content, b"the successor's objects").expect("content writes");

        let error = owner.release().expect_err("the replaced workspace refuses");
        assert!(
            error.to_string().contains("identity changed"),
            "the refusal names the cause: {error}"
        );
        assert!(
            successor_content.is_file(),
            "the successor's content is never deleted"
        );

        std::fs::remove_dir_all(&path).ok();
    }

    /// Storage is repository-derived and on the repository's own volume. The
    /// superseded acquisition used the ambient temporary directory, which the
    /// `.7.2.1` probe measured as a DIFFERENT volume from the checkout.
    #[test]
    fn the_workspace_is_repository_derived_and_on_the_repository_volume() {
        let root = repository_root().expect("the repository root");
        let mut owner =
            OwnedDirectory::create("git", "control-located").expect("the workspace creates");

        assert!(
            owner
                .path()
                .starts_with(root.join(".project-data").join("git")),
            "the workspace is under the repository's own store: {}",
            owner.path().display()
        );
        assert_eq!(
            owner.relative(),
            std::path::Path::new(".project-data/git")
                .join(owner.path().file_name().expect("a name")),
            "the relative form carries no checkout-specific prefix"
        );
        let workspace_device = std::fs::metadata(owner.path())
            .expect("workspace metadata")
            .dev();
        let root_device = std::fs::metadata(&root).expect("root metadata").dev();
        assert_eq!(
            workspace_device, root_device,
            "the workspace shares the repository's volume"
        );
        owner.release().expect("the workspace releases");
    }

    /// Dropping the owner removes the directory, so a SUCCESSFUL run leaves
    /// nothing behind. The superseded acquisition only cleaned up on error,
    /// so every success leaked a bare repository into the ambient store.
    #[test]
    fn dropping_the_owner_removes_the_workspace() {
        let path = {
            let owner =
                OwnedDirectory::create("git", "control-dropped").expect("the workspace creates");
            let path = owner.path().to_path_buf();
            std::fs::write(path.join("objects.pack"), b"acquired objects").expect("content writes");
            assert!(path.is_dir(), "the workspace is live while the owner is");
            path
        };
        assert!(
            !path.exists(),
            "the workspace does not outlive its owner: {}",
            path.display()
        );
    }
}
