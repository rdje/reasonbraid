//! Exclusively owned extraction inputs. Test files never use an ambient temp dir.

use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};

static NEXT_INPUT: AtomicU64 = AtomicU64::new(0);

fn refuse(message: &str) -> io::Error {
    io::Error::other(message)
}

fn same_volume(left: &Metadata, right: &Metadata) -> bool {
    #[cfg(unix)]
    {
        left.dev() == right.dev()
    }
    #[cfg(not(unix))]
    {
        let _ = (left, right);
        false // These filesystem ownership controls require a POSIX host.
    }
}

fn same_file(left: &Metadata, right: &Metadata) -> bool {
    #[cfg(unix)]
    {
        same_volume(left, right) && left.ino() == right.ino() && left.nlink() == 1
    }
    #[cfg(not(unix))]
    {
        let _ = (left, right);
        false
    }
}

fn repository_root() -> io::Result<PathBuf> {
    std::env::current_dir()?
        .ancestors()
        .find(|path| {
            path.join("Cargo.toml").is_file()
                && path.join("scripts/project_env.py").is_file()
                && path.join("rust-toolchain.toml").is_file()
        })
        .ok_or_else(|| refuse("run extraction tests inside the repository"))?
        .canonicalize()
}

fn storage(root: &Path, create: bool) -> io::Result<PathBuf> {
    let volume = fs::metadata(root)?;
    let mut path = root.to_path_buf();
    for part in ["target", "extract-tests"] {
        path.push(part);
        if create {
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            builder.mode(0o700);
            match builder.create(&path) {
                Ok(()) => (),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
                Err(error) => return Err(error),
            }
        }
        let metadata = fs::symlink_metadata(&path)?;
        if !metadata.is_dir() || !same_volume(&metadata, &volume) {
            return Err(refuse(
                "extraction fixture storage must be local and unlinked",
            ));
        }
    }
    Ok(path)
}

pub struct Input {
    root: PathBuf,
    path: PathBuf,
    file: File,
}

impl Input {
    pub fn new(bytes: &[u8]) -> Self {
        for _ in 0..512 {
            let sequence = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
            let name = format!("input-{}-{sequence}", std::process::id());
            match Self::create_named(&name, bytes) {
                Ok(input) => return input,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
                Err(error) => panic!("extraction input creation failed: {error}"),
            }
        }
        panic!("extraction input names exhausted; existing files remain untouched");
    }

    /// Exposed only to tests, including deliberate existing-file/link controls.
    pub fn create_named(name: &str, bytes: &[u8]) -> io::Result<Self> {
        let mut components = Path::new(name).components();
        if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
            return Err(refuse("extraction input name must be one normal component"));
        }
        let root = repository_root()?;
        let path = storage(&root, true)?.join(name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let file = options.open(&path)?;
        let mut input = Self { root, path, file };
        input.file.write_all(bytes)?;
        Ok(input)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        let relative = self
            .path
            .strip_prefix(&self.root)
            .expect("owned input path");
        if std::thread::panicking() {
            eprintln!("retained failed extraction input: {}", relative.display());
            return;
        }
        let cleanup = || -> io::Result<()> {
            if storage(&self.root, false)? != self.path.parent().expect("owned input parent") {
                return Err(refuse("extraction input parent changed"));
            }
            let metadata = fs::symlink_metadata(&self.path)?;
            if !metadata.is_file() || !same_file(&metadata, &self.file.metadata()?) {
                return Err(refuse("extraction input identity changed; retained"));
            }
            fs::remove_file(&self.path)
        };
        if let Err(error) = cleanup() {
            panic!(
                "extraction input cleanup refused ({}): {error}",
                relative.display()
            );
        }
    }
}
