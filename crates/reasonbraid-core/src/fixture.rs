//! A generated test fixture that removes itself when its test PASSES
//! (`SIGNOFF-REPAIR.11.2.1.3.2.2`).
//!
//! ⭐ WHY THIS EXISTS. `SIGNOFF-REPAIR.11.2.1.1` replaced fixture ADOPTION with
//! exclusive creation, which is correct: a name does not establish ownership and
//! a reused directory belongs to an earlier run. The cost is that every run now
//! creates a NEW directory and nothing ever removed one — 351,000 KiB across
//! 3,979 fixtures when `.11.2.1.3.2.1` measured it, 2,375 of them in
//! `journal-tests` alone. The accumulation is not a regression; it is the bill
//! for a property the suites bought deliberately, and this module is how it gets
//! paid without giving the property back.
//!
//! ⛔ A SWEEPER WOULD BE A WORKAROUND. The producer is what knows whether its
//! fixture still matters, and it knows it at exactly one moment: when the test
//! ends. A passing test's fixture is evidence of nothing. A FAILING test's
//! fixture is the diagnostic, and `scripts/census_retained_fixtures.py` exists to
//! REDUCE those rather than delete them. So cleanup is conditioned on
//! `std::thread::panicking()` — the same discriminator
//! `crates/reasonbraid-extract/tests/support/mod.rs` and
//! `crates/reasonbraid-cli/tests/http_bounds.rs` already use — and never on a
//! clock, a size or a sweep.
//!
//! ⛔ AND `cargo clean` IS NOT THE ANSWER. It also destroys the build cache,
//! which costs over two hours on the machine this was measured on
//! (`docs/decisions/2026-09-12_checkpoint-cost-model.md`), so a cleanup only
//! available at that price is one nobody runs.
//!
//! ⚠️ THE HAZARD THIS SHAPE INTRODUCES, STATED RATHER THAN DISCOVERED. A guard
//! is a value, and a value used as a TEMPORARY is dropped at the end of its
//! statement:
//!
//! ```ignore
//! let journal = Journal::open(fixture("profile").join("node.db")).await?; // ⛔ WRONG
//! ```
//!
//! That removes the directory while the journal still holds a file inside it.
//! Every call site therefore BINDS the guard to a local that outlives what it
//! contains. `#[must_use]` catches a guard thrown away entirely and does NOT
//! catch this shape, so the real bound is that the failure is LOUD rather than
//! silent: the next database operation fails on a file that has vanished. A
//! confusing failure is recoverable; a silent pass is not.

use std::io;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, MetadataExt};

use crate::paths::repository_root;

/// Where a family's fixtures live, relative to the repository root.
///
/// ⛔ Joined from `"target"` and the family name IN THIS FUNCTION, because §13
/// requires every byte a suite generates to land on the repository's own volume
/// and §12 requires the root to be found at RUNTIME. A caller that built its own
/// absolute base would be free to get either wrong.
fn family_root(root: &Path, family: &str) -> PathBuf {
    root.join("target").join(family)
}

/// An exclusively created fixture directory that removes itself on success.
///
/// Create one per test. Read its path with [`Fixture::path`] or [`Fixture::join`],
/// and let it drop at the end of the test: it removes itself if the test passed,
/// and retains itself — printing where — if the test is unwinding from a panic,
/// which includes every failed assertion.
#[must_use = "a fixture guard must be BOUND to a local; a temporary is dropped \
              at the end of its statement, removing the directory while the test \
              is still using it"]
#[derive(Debug)]
pub struct Fixture {
    path: PathBuf,
    /// The identity the directory had when this guard created it. Removal
    /// refuses if it has changed, so a guard can never delete something that is
    /// no longer the thing it made.
    identity: (u64, u64),
    retained: bool,
}

impl Fixture {
    /// Create `target/<family>/<name>-<uuid>` on the repository volume.
    ///
    /// The family directory is SHARED by design and is adopted if present; the
    /// per-test directory is created with `DirBuilder::create`, which refuses an
    /// existing path rather than adopting it (`SIGNOFF-REPAIR.11.2.1.1`). The
    /// uuid is v7, so the name sorts by creation time when a failure retains it.
    pub fn create(family: &str, name: &str) -> io::Result<Self> {
        let root = repository_root()?;
        let parent = family_root(&root, family);
        std::fs::create_dir_all(&parent)?;

        let path = parent.join(format!("{name}-{}", uuid::Uuid::now_v7()));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        builder.mode(0o700);
        builder.create(&path)?;

        let metadata = std::fs::symlink_metadata(&path)?;
        #[cfg(unix)]
        let identity = (metadata.dev(), metadata.ino());
        #[cfg(not(unix))]
        let identity = (0, 0);
        Ok(Self {
            path,
            identity,
            retained: false,
        })
    }

    /// The fixture directory itself.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// A path inside the fixture — `fixture.join("node.db")`.
    pub fn join(&self, name: impl AsRef<Path>) -> PathBuf {
        self.path.join(name)
    }

    /// Keep this fixture even though the test passed.
    ///
    /// For a test whose PASS produces the evidence — a control that proves what
    /// a run leaves behind. Retention is announced on stderr so a kept fixture
    /// is never a mystery.
    pub fn retain(&mut self) {
        self.retained = true;
    }

    /// Remove the directory, refusing anything that is not what this guard made.
    fn remove(&self) -> io::Result<()> {
        let metadata = std::fs::symlink_metadata(&self.path)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(io::Error::other(
                "fixture is no longer a directory; retained",
            ));
        }
        #[cfg(unix)]
        if (metadata.dev(), metadata.ino()) != self.identity {
            return Err(io::Error::other("fixture identity changed; retained"));
        }
        std::fs::remove_dir_all(&self.path)?;
        match std::fs::symlink_metadata(&self.path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            _ => Err(io::Error::other("fixture removal not confirmed")),
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // ⛔ THE ONE DISCRIMINATOR. A failing test's fixture IS the diagnostic,
        // and this is the only moment anything knows the difference.
        if std::thread::panicking() {
            eprintln!("fixture retained (test failed): {}", self.path.display());
            return;
        }
        if self.retained {
            eprintln!("fixture retained (requested): {}", self.path.display());
            return;
        }
        if let Err(error) = self.remove() {
            // Not already unwinding, so this panics the test rather than
            // aborting the process. A cleanup that silently failed would
            // reintroduce the accumulation this module exists to end.
            panic!("fixture cleanup refused ({}): {error}", self.path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixture lands under the repository's own `target/<family>/`, on the
    /// same filesystem as the repository (§13), and is a real directory rather
    /// than a link to one.
    #[test]
    fn a_fixture_is_created_on_the_repository_volume() {
        let root = repository_root().expect("the tests run inside the repository");
        let fixture = Fixture::create("fixture-guard-controls", "placement").unwrap();

        assert!(fixture.path().is_dir());
        assert_eq!(
            fixture.path().parent().unwrap(),
            root.join("target").join("fixture-guard-controls")
        );
        let metadata = std::fs::symlink_metadata(fixture.path()).unwrap();
        assert!(!metadata.file_type().is_symlink());
        #[cfg(unix)]
        assert_eq!(metadata.dev(), std::fs::metadata(&root).unwrap().dev());
    }

    /// Two fixtures asking for the same NAME get different directories, so a
    /// name never has to carry uniqueness on its own.
    #[test]
    fn two_fixtures_of_one_name_do_not_collide() {
        let first = Fixture::create("fixture-guard-controls", "collide").unwrap();
        let second = Fixture::create("fixture-guard-controls", "collide").unwrap();
        assert_ne!(first.path(), second.path());
        assert!(first.path().is_dir() && second.path().is_dir());
    }

    /// ⭐ THE PROPERTY, PRODUCED RATHER THAN ASSERTED: a guard dropped while the
    /// thread is NOT panicking removes its directory, contents and all.
    #[test]
    fn a_passing_test_removes_its_own_fixture() {
        let path = {
            let fixture = Fixture::create("fixture-guard-controls", "passing").unwrap();
            std::fs::write(fixture.join("payload.txt"), b"reproducible").unwrap();
            std::fs::create_dir(fixture.join("nested")).unwrap();
            std::fs::write(fixture.join("nested/deep.txt"), b"also reproducible").unwrap();
            let path = fixture.path().to_path_buf();
            assert!(path.is_dir());
            path
        };
        assert!(!path.exists(), "a passing test's fixture must be removed");
    }

    /// ⭐ THE OTHER HALF, AND IT IS THE ONE THAT MATTERS: a guard dropped during
    /// a panic keeps everything. The panic is REAL — produced in a spawned
    /// thread and caught by its join handle — rather than simulated by setting a
    /// flag, because `std::thread::panicking()` is the thing under test.
    #[test]
    fn a_failing_test_retains_its_fixture_and_everything_in_it() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let handle = std::thread::spawn(move || {
            let fixture = Fixture::create("fixture-guard-controls", "failing").unwrap();
            std::fs::write(fixture.join("evidence.log"), b"why it failed").unwrap();
            sender.send(fixture.path().to_path_buf()).unwrap();
            panic!("the control's deliberate failure");
        });
        let path = receiver
            .recv()
            .expect("the fixture path arrives before the panic");
        assert!(handle.join().is_err(), "the thread must have panicked");

        assert!(path.is_dir(), "a failed test's fixture is its evidence");
        assert_eq!(
            std::fs::read(path.join("evidence.log")).unwrap(),
            b"why it failed",
            "the evidence must survive byte for byte"
        );
        // This control's own retained fixture is the only one the suite leaves,
        // and it is removed here because the assertions above have consumed it.
        std::fs::remove_dir_all(&path).unwrap();
    }

    /// An explicit `retain()` keeps the fixture even though nothing panicked —
    /// the escape hatch for a control whose PASS is what produces the evidence.
    #[test]
    fn an_explicit_retain_keeps_a_passing_fixture() {
        let path = {
            let mut fixture = Fixture::create("fixture-guard-controls", "retained").unwrap();
            fixture.retain();
            fixture.path().to_path_buf()
        };
        assert!(path.is_dir(), "an explicitly retained fixture stays");
        std::fs::remove_dir_all(&path).unwrap();
    }

    /// Removal refuses a directory that is no longer the one the guard created,
    /// so a guard can never delete a replacement that took its path.
    #[cfg(unix)]
    #[test]
    fn removal_refuses_a_directory_that_is_no_longer_the_one_created() {
        let fixture = Fixture::create("fixture-guard-controls", "swapped").unwrap();
        let path = fixture.path().to_path_buf();
        std::fs::remove_dir_all(&path).unwrap();
        std::fs::DirBuilder::new().create(&path).unwrap();
        std::fs::write(path.join("not-ours.txt"), b"another run's data").unwrap();

        let error = fixture
            .remove()
            .expect_err("a replaced directory is refused");
        assert!(
            error.to_string().contains("identity changed"),
            "got: {error}"
        );
        assert!(
            path.join("not-ours.txt").is_file(),
            "the replacement survives"
        );

        std::mem::forget(fixture);
        std::fs::remove_dir_all(&path).unwrap();
    }
}
