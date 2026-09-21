//! Where the repository is, discovered at RUNTIME (§12).
//!
//! A path baked in by `env!("CARGO_MANIFEST_DIR")` names the checkout the
//! binary was BUILT in, not the one it is running in. §12 requires the
//! opposite — the repository root may be moved, even onto another filesystem,
//! without the project noticing — so every base that reaches project data is
//! derived from the current directory instead.
//!
//! This is `project_storage::repository_root()`, which has derived the server's
//! private storage this way since `SIGNOFF-REPAIR.7.2.1`, moved to the crate the
//! others already depend on so that there is ONE of it. Before the move the
//! workspace held three different predicates for the same question
//! (`SIGNOFF-REPAIR.11.2.1.2.1`).

use std::io;
use std::path::{Path, PathBuf};

/// The repository root, found by walking the current directory's ancestors.
///
/// The predicate is deliberately stricter than `Cargo.toml` alone, and stricter
/// than a `Cargo.toml` + `migrations` pair: `crates/reasonbraid-node` satisfies
/// the first, so a process whose working directory sat there would stop at the
/// crate and place project data inside it. `rust-toolchain.toml` exists only at
/// the real root.
///
/// Returns an error rather than a guess when the caller is outside a checkout:
/// a wrong root is how project data lands off the repository volume (§13), and
/// there is no safe default to fall back to.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The root this returns is the real one: it carries all three of the
    /// files the predicate names, and it is an ANCESTOR of the directory the
    /// test runs in rather than some other checkout on the machine.
    #[test]
    fn the_root_is_the_one_this_process_is_running_inside() {
        let root = repository_root().expect("the tests run inside the repository");
        assert!(root.join("Cargo.toml").is_file());
        assert!(root.join("migrations").is_dir());
        assert!(root.join("rust-toolchain.toml").is_file());

        let current = std::env::current_dir().expect("cwd");
        assert!(
            current.ancestors().any(|path| path == root),
            "the root {root:?} is not an ancestor of {current:?}"
        );
    }

    /// The property the compile-time form cannot have: the answer follows the
    /// PROCESS, not the build. Walking the same predicate from a different
    /// directory inside the same checkout returns the same root — so moving
    /// the checkout moves the answer with it, which is what §12 asks for.
    #[test]
    fn the_answer_follows_the_working_directory_not_the_build() {
        let root = repository_root().expect("the tests run inside the repository");
        let deeper = root.join("crates").join("reasonbraid-core").join("src");
        assert!(deeper.is_dir(), "the fixture directory {deeper:?} exists");

        let from_deeper = deeper
            .ancestors()
            .find(|path| {
                path.join("Cargo.toml").is_file()
                    && path.join("migrations").is_dir()
                    && path.join("rust-toolchain.toml").is_file()
            })
            .expect("the same predicate resolves from a deeper directory");
        assert_eq!(from_deeper, root);
    }

    /// Falsified: outside a checkout the predicate matches nothing, and the
    /// function REFUSES instead of returning a plausible parent. A root that
    /// is merely plausible is how project data leaves the repository volume.
    #[test]
    fn a_directory_outside_any_checkout_is_refused() {
        let outside = Path::new("/");
        assert!(
            !outside.ancestors().any(|path| {
                path.join("Cargo.toml").is_file()
                    && path.join("migrations").is_dir()
                    && path.join("rust-toolchain.toml").is_file()
            }),
            "the filesystem root must not satisfy the repository predicate"
        );
    }
}
