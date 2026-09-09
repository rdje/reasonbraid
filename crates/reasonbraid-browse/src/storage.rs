//! Private invocation storage. Only the explicit, confirmed shutdown path deletes it.

use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

pub struct Workspace {
    root: PathBuf,
    pub path: PathBuf,
    identity: (u64, u64),
    removed: bool,
}

impl Workspace {
    pub fn discover() -> io::Result<Self> {
        let current = std::env::current_dir()?;
        let root = current
            .ancestors()
            .find(|p| p.join("Cargo.toml").is_file() && p.join("migrations").is_dir())
            .ok_or_else(|| io::Error::other("run the browser worker from within the repository"))?;
        Self::create(root)
    }

    fn create(root: &Path) -> io::Result<Self> {
        let device = std::fs::metadata(root)?.dev();
        let mut parent = root.to_path_buf();
        for component in [".project-data", "browser"] {
            parent.push(component);
            match std::fs::DirBuilder::new().mode(0o700).create(&parent) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e),
            }
            let metadata = std::fs::symlink_metadata(&parent)?;
            if !metadata.is_dir()
                || metadata.dev() != device
                || metadata.uid() != rustix::process::geteuid().as_raw()
                || metadata.mode() & 0o022 != 0
            {
                return Err(io::Error::other(
                    "browser storage parent is not an owned on-volume directory",
                ));
            }
        }
        let path = parent.join(format!("run-{}", uuid::Uuid::now_v7()));
        std::fs::DirBuilder::new().mode(0o700).create(&path)?;
        let metadata = std::fs::symlink_metadata(&path)?;
        let owner = Self {
            root: root.to_path_buf(),
            path,
            identity: (metadata.dev(), metadata.ino()),
            removed: false,
        };
        owner.verify()?;
        for component in [
            "profile", "cache", "tmp", "config", "data", "state", "crashes",
        ] {
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(owner.path.join(component))?;
        }
        Ok(owner)
    }

    pub fn relative(&self) -> &Path {
        self.path
            .strip_prefix(&self.root)
            .expect("workspace is within its root")
    }

    pub fn write_new(&self, name: &str, bytes: &[u8]) -> io::Result<()> {
        use std::io::Write;
        self.verify()?;
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(self.path.join(name))?;
        file.write_all(bytes)?;
        file.sync_all()
    }

    fn verify(&self) -> io::Result<()> {
        let metadata = std::fs::symlink_metadata(&self.path)?;
        if !metadata.is_dir() || (metadata.dev(), metadata.ino()) != self.identity {
            return Err(io::Error::other(
                "browser workspace identity changed; retain data",
            ));
        }
        Ok(())
    }

    /// Call only after every owned process and task has demonstrably stopped.
    pub fn remove(&mut self) -> io::Result<()> {
        self.verify()?;
        verify_volume(&self.path, self.identity.0)?;
        std::fs::remove_dir_all(&self.path)?;
        match std::fs::symlink_metadata(&self.path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                self.removed = true;
                Ok(())
            }
            _ => Err(io::Error::other("browser workspace removal not confirmed")),
        }
    }
}

fn verify_volume(path: &Path, device: u64) -> io::Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.dev() != device {
        return Err(io::Error::other(
            "browser workspace contains a foreign volume; retain data",
        ));
    }
    // Chromium's SingletonSocket/Lock links are unlinked, never traversed.
    if metadata.is_dir() {
        for entry in std::fs::read_dir(path)? {
            verify_volume(&entry?.path(), device)?;
        }
    }
    Ok(())
}

impl Drop for Workspace {
    fn drop(&mut self) {
        if !self.removed {
            eprintln!("browser workspace retained: {}", self.relative().display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{symlink, PermissionsExt};

    fn root() -> PathBuf {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let device = std::fs::metadata(&root).unwrap().dev();
        for component in ["target", "browser-production-controls"] {
            root.push(component);
            match std::fs::DirBuilder::new().mode(0o700).create(&root) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                Err(e) => panic!("fixture parent refuses: {e}"),
            }
            let metadata = std::fs::symlink_metadata(&root).unwrap();
            assert!(metadata.is_dir());
            assert_eq!(metadata.dev(), device);
        }
        let path = root.join(format!("storage-{}", uuid::Uuid::now_v7()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .unwrap();
        path
    }

    #[test]
    fn independent_private_workspaces_preserve_each_other_and_external_links() {
        let root = root();
        let mut first = Workspace::create(&root).unwrap();
        let mut second = Workspace::create(&root).unwrap();
        assert_ne!(first.path, second.path);
        assert!(!first.relative().is_absolute());
        for owner in [&first, &second] {
            let metadata = std::fs::metadata(&owner.path).unwrap();
            assert_eq!(metadata.dev(), std::fs::metadata(&root).unwrap().dev());
            assert_eq!(metadata.mode() & 0o777, 0o700);
            owner.write_new("witness", b"owned").unwrap();
            assert!(owner.write_new("witness", b"overwrite").is_err());
        }
        symlink(second.path.join("witness"), first.path.join("link")).unwrap();
        first.remove().unwrap();
        assert_eq!(
            std::fs::read(second.path.join("witness")).unwrap(),
            b"owned"
        );
        second.remove().unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unfinished_and_replaced_workspaces_are_retained() {
        let root = root();
        let owner = Workspace::create(&root).unwrap();
        let path = owner.path.clone();
        drop(owner);
        assert!(path.is_dir());
        let mut owner = Workspace::create(&root).unwrap();
        let moved = root.join("original");
        std::fs::rename(&owner.path, &moved).unwrap();
        std::fs::create_dir(&owner.path).unwrap();
        std::fs::write(owner.path.join("witness"), b"replacement").unwrap();
        assert!(owner
            .remove()
            .unwrap_err()
            .to_string()
            .contains("identity changed"));
        assert_eq!(
            std::fs::read(owner.path.join("witness")).unwrap(),
            b"replacement"
        );
        assert!(moved.is_dir());
        drop(owner);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn symlinked_or_shared_writable_parents_refuse_without_touching_targets() {
        let root = root();
        let target = root.join("foreign");
        std::fs::create_dir(&target).unwrap();
        symlink(&target, root.join(".project-data")).unwrap();
        assert!(Workspace::create(&root).is_err());
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 0);
        std::fs::remove_file(root.join(".project-data")).unwrap();
        std::fs::create_dir(root.join(".project-data")).unwrap();
        std::fs::set_permissions(
            root.join(".project-data"),
            std::fs::Permissions::from_mode(0o777),
        )
        .unwrap();
        assert!(Workspace::create(&root).is_err());
        assert!(!root.join(".project-data/browser").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
