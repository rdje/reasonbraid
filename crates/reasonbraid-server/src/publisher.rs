//! The Git publication half (`PHASE-6.4.3.2`, ADR-020 + the
//! `.4.3.1` store contract): the publisher writes the bundle + the
//! manifest into the LOCAL bare repository — the staging branch, the
//! fetch-back verification (the digest re-derived from the written blobs),
//! the IMMUTABLE publication ref (written once), and the EFFECTIVE channel
//! via the compare-and-swap (the expected old id). No git CLI — the gix
//! plumbing, the pure-Rust doctrine.

use std::path::{Path, PathBuf};

use gix::bstr::BStr;

/// The published refs: the immutable publication ref's object id + the
/// effective channel's new object id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedRefs {
    pub publication_ref_id: String,
    pub effective_ref_id: String,
}

/// The typed failure reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishError {
    Open(String),
    Write(String),
    FetchBack(String),
    ImmutableExists,
    CasMismatch(String),
}

impl std::fmt::Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublishError::Open(e) => write!(f, "the repository does not open: {e}"),
            PublishError::Write(e) => write!(f, "the write failed: {e}"),
            PublishError::FetchBack(e) => {
                write!(f, "the fetch-back verification failed: {e}")
            }
            PublishError::ImmutableExists => {
                write!(
                    f,
                    "the immutable publication ref already exists — written once, never moved"
                )
            }
            PublishError::CasMismatch(found) => {
                write!(f, "the effective channel moved underneath: found `{found}` — the compare-and-swap failed, never a force-push")
            }
        }
    }
}

/// Why a caller-supplied publication repository location was refused
/// (`SIGNOFF-REPAIR.9.2.1.1`). Before that leaf the publish verb handed
/// `gix::open` whatever the request body said, so any enrolled principal
/// named any path on the server's filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryRefusal {
    /// The deployment declares no publication repository root, so the verb is
    /// CLOSED. Fail-closed is the contract rather than a detail: a deployment
    /// that never named a root must not publish into a caller-named path.
    Unconfigured,
    /// The configured root does not resolve to a usable directory — an
    /// operator's problem, never the caller's.
    RootUnusable { root: String, reason: String },
    /// The requested location does not resolve: nothing is there, or a
    /// component of the path is not a directory.
    Unresolvable { requested: String, reason: String },
    /// It resolves OUTSIDE the configured root.
    Outside { requested: String },
}

impl std::fmt::Display for RepositoryRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryRefusal::Unconfigured => write!(
                f,
                "this deployment declares no publication repository root — the verb is closed until an operator configures one"
            ),
            RepositoryRefusal::RootUnusable { root, reason } => write!(
                f,
                "the configured publication repository root `{root}` is not usable: {reason}"
            ),
            RepositoryRefusal::Unresolvable { requested, reason } => write!(
                f,
                "the repository location `{requested}` does not resolve inside the configured publication repository root: {reason}"
            ),
            RepositoryRefusal::Outside { requested } => write!(
                f,
                "the repository location `{requested}` resolves outside the configured publication repository root"
            ),
        }
    }
}

/// A publication repository location that has been RESOLVED inside the
/// deployment's configured root (`SIGNOFF-REPAIR.9.2.1.1`).
///
/// It has no public constructor. [`resolve_repository`] is the only way to
/// obtain one, so a verb that takes this type cannot be reached with a path
/// that has not passed containment — the binding is structural rather than a
/// convention a later caller can forget. `SIGNOFF-REPAIR.9.2.1.3` is why it is
/// a type at all: a second verb needed the same guarantee, and two surfaces
/// sharing a rule want the rule in one place they both have to go through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationRepository(PathBuf);

impl PublicationRepository {
    /// The canonical path the repository is opened at.
    pub fn path(&self) -> &Path {
        &self.0
    }
}

/// Canonicalize a DECLARED publication repository root, or say why it is not
/// usable (`SIGNOFF-REPAIR.9.2.1.1`).
///
/// `rb-server` runs this at boot, before anything mutates, and
/// [`resolve_repository`] runs it again on every request — one definition, so
/// the boot check and the request check cannot drift into disagreeing about
/// what a usable root is.
pub fn validate_root(declared: &Path) -> Result<PathBuf, RepositoryRefusal> {
    let root = declared
        .canonicalize()
        .map_err(|e| RepositoryRefusal::RootUnusable {
            root: declared.display().to_string(),
            reason: e.to_string(),
        })?;
    if !root.is_dir() {
        return Err(RepositoryRefusal::RootUnusable {
            root: declared.display().to_string(),
            reason: "it is not a directory".to_string(),
        });
    }
    Ok(root)
}

/// Resolve a caller-supplied publication repository location INSIDE the
/// deployment's configured root (`SIGNOFF-REPAIR.9.2.1.1`).
///
/// A relative location is joined to the root; an absolute one is taken as
/// given. Either way BOTH sides are canonicalized before they are compared,
/// and that is what lets one question answer two different escapes: `..` and a
/// symlink are two spellings of "somewhere else", and canonicalization resolves
/// each into the real path it names. Comparing the strings would catch neither.
///
/// The location must already exist: a publication repository is opened, never
/// created, and canonicalization is only defined over a path that resolves.
///
/// ⚠️ An honest limit, stated rather than implied: the containment decision is
/// made once, against the filesystem as it is at the moment of the request, and
/// the resolved path is then handed to [`publish`]. A symlink swapped between
/// the two would not be seen. Closing that needs a directory handle the
/// repository is opened relative to, which `gix::open` does not accept.
pub fn resolve_repository(
    configured_root: Option<&Path>,
    requested: &str,
) -> Result<PublicationRepository, RepositoryRefusal> {
    let root = validate_root(configured_root.ok_or(RepositoryRefusal::Unconfigured)?)?;
    let asked = Path::new(requested);
    let joined = if asked.is_absolute() {
        asked.to_path_buf()
    } else {
        root.join(asked)
    };
    let resolved = joined
        .canonicalize()
        .map_err(|e| RepositoryRefusal::Unresolvable {
            requested: requested.to_string(),
            reason: e.to_string(),
        })?;
    if !resolved.starts_with(&root) {
        return Err(RepositoryRefusal::Outside {
            requested: requested.to_string(),
        });
    }
    Ok(PublicationRepository(resolved))
}

/// Which of the DECLARED Git object ids do not name an object that exists in
/// the publication repository (`SIGNOFF-REPAIR.9.2.1.3`).
///
/// `mark_effective` used to write whatever ids the caller sent, so the record
/// asserted the existence of objects nobody had looked for — and the project's
/// own fixtures drove it with `abc123`, which is not even a well-formed id.
/// An id that does not PARSE and one that parses and resolves to nothing are
/// the same answer here: it is not in the repository.
///
/// Returns the missing ids in the order they were declared, empty when every
/// one resolves. The repository failing to open is a different thing from an
/// id being absent, so it comes back as [`PublishError::Open`] rather than as
/// a verdict about the ids.
/// Whether the repository opens — checked before a publish RECORDS its Git
/// operation (`SIGNOFF-REPAIR.9.3.5.1.1`): a location that is not a repository
/// was never written to, so recording an operation against it would pin every
/// later, correct publish to a repository that cannot exist.
pub fn opens(repository: &PublicationRepository) -> Result<(), PublishError> {
    gix::open(repository.path())
        .map(|_| ())
        .map_err(|e| PublishError::Open(e.to_string()))
}

/// The repository's location RELATIVE to the configured publication root —
/// what a publication records (§12: persisted paths are root-relative, so the
/// root may move without stranding the record).
pub fn root_relative(
    configured_root: Option<&Path>,
    repository: &PublicationRepository,
) -> Result<String, RepositoryRefusal> {
    let root = validate_root(configured_root.ok_or(RepositoryRefusal::Unconfigured)?)?;
    let relative =
        repository
            .path()
            .strip_prefix(&root)
            .map_err(|_| RepositoryRefusal::Outside {
                requested: repository.path().display().to_string(),
            })?;
    Ok(relative.to_string_lossy().into_owned())
}

pub fn missing_objects(
    repository: &PublicationRepository,
    declared: &[String],
) -> Result<Vec<String>, PublishError> {
    let repo = gix::open(repository.path()).map_err(|e| PublishError::Open(e.to_string()))?;
    Ok(declared
        .iter()
        .filter(|declared| {
            !declared
                .parse::<gix::ObjectId>()
                .is_ok_and(|oid| repo.find_object(oid).is_ok())
        })
        .cloned()
        .collect())
}

/// The raw signature header value (the `name <email> seconds +HHMM` shape —
/// the CommitRef's raw author/committer fields).
fn signature(now: gix::date::Time) -> String {
    let sign = if now.offset < 0 { '-' } else { '+' };
    let offset = now.offset.abs();
    format!(
        "reasonbraid-publisher <publisher@reasonbraid.local> {} {sign}{:02}{:02}",
        now.seconds,
        offset / 3600,
        (offset % 3600) / 60,
    )
}

/// The object a ref points at, or `None` when it is absent or unreadable.
fn current(repo: &gix::Repository, name: &str) -> Option<gix::ObjectId> {
    repo.try_find_reference(name)
        .ok()
        .flatten()
        .map(|r| r.id().detach())
}

/// The publication's objects — its two blobs, its tree and its root commit —
/// written into `repo`, returning the commit id and the manifest blob id.
///
/// ⭐ ONE definition, shared by [`publish`] and [`expected_commit`], so the
/// reconciler's expected id and the id a publish writes cannot diverge. Git
/// objects are content-addressed, so writing them again is harmless.
fn write_commit(
    repo: &gix::Repository,
    publication_id: &str,
    manifest: &str,
    bundle: &str,
    staged_at_seconds: i64,
) -> Result<(gix::ObjectId, gix::ObjectId), PublishError> {
    // The blobs + the tree + the root commit.
    let manifest_blob = repo
        .write_object(gix::objs::BlobRef {
            data: manifest.as_bytes(),
        })
        .map_err(|e| PublishError::Write(e.to_string()))?;
    let bundle_blob = repo
        .write_object(gix::objs::BlobRef {
            data: bundle.as_bytes(),
        })
        .map_err(|e| PublishError::Write(e.to_string()))?;
    let tree_id = repo
        .write_object(gix::objs::TreeRef {
            // The tree serialization requires the filename-sorted entries.
            entries: vec![
                gix::objs::tree::EntryRef {
                    mode: gix::objs::tree::EntryKind::Blob.into(),
                    filename: "bundle.txt".into(),
                    oid: &bundle_blob,
                },
                gix::objs::tree::EntryRef {
                    mode: gix::objs::tree::EntryKind::Blob.into(),
                    filename: "manifest.json".into(),
                    oid: &manifest_blob,
                },
            ],
        })
        .map_err(|e| PublishError::Write(e.to_string()))?;

    let tree_hex = tree_id.to_string();
    // `SIGNOFF-REPAIR.9.3.5.1.1`: the commit's time is the publication's
    // STAGING time, not the clock — so the commit id is a pure function of the
    // record and its content, a retry writes the identical commit, and the
    // reconciler can recompute the id it expects. A wall-clock stamp made
    // §15.8's `RetryStagedWrite` (*commits identically*) false.
    let raw_signature = signature(gix::date::Time::new(staged_at_seconds, 0));
    let message = format!("publication {publication_id}");
    let commit_ref = gix::objs::CommitRef {
        tree: BStr::new(tree_hex.as_bytes()),
        parents: Default::default(),
        author: BStr::new(raw_signature.as_bytes()),
        committer: BStr::new(raw_signature.as_bytes()),
        encoding: None,
        message: BStr::new(message.as_bytes()),
        extra_headers: Vec::new(),
    };
    let commit: gix::objs::Commit =
        gix::objs::Commit::try_from(commit_ref).map_err(|e| PublishError::Write(e.to_string()))?;
    let commit_id = repo
        .write_object(&commit)
        .map_err(|e| PublishError::Write(e.to_string()))?;
    Ok((commit_id.detach(), manifest_blob.detach()))
}

/// The commit a publication's content commits to (`SIGNOFF-REPAIR.9.3.5.1.2`):
/// what §15.8's matrix compares the observed immutable ref against. Writes the
/// objects (content-addressed, so harmless) and NO ref.
pub fn expected_commit(
    repo_path: &Path,
    publication_id: &str,
    manifest: &str,
    bundle: &str,
    staged_at_seconds: i64,
) -> Result<gix::ObjectId, PublishError> {
    let repo = gix::open(repo_path).map_err(|e| PublishError::Open(e.to_string()))?;
    write_commit(&repo, publication_id, manifest, bundle, staged_at_seconds).map(|(id, _)| id)
}

/// Publish one publication into the bare repository (the §15.7 steps 5–8's
/// Git half):
///
/// 1. write the manifest + the bundle as the blobs (`manifest.json` +
///    `bundle.txt`), the tree, and the ROOT commit (the message = the
///    publication id);
/// 2. the staging branch `refs/rb/staging/<id>` (the idempotent re-write);
/// 3. the fetch-back verification (the re-derived digest must match);
/// 4. the IMMUTABLE ref `refs/rb/publications/<id>` (the written-once rule);
/// 5. the EFFECTIVE channel `refs/rb/effective` via the compare-and-swap
///    (the expected old id — `None` means the channel must not exist yet).
pub fn publish(
    repo_path: &Path,
    publication_id: &str,
    manifest: &str,
    bundle: &str,
    expected_effective: Option<gix::ObjectId>,
    staged_at_seconds: i64,
) -> Result<PublishedRefs, PublishError> {
    let repo = gix::open(repo_path).map_err(|e| PublishError::Open(e.to_string()))?;
    let (commit_id, manifest_blob) =
        write_commit(&repo, publication_id, manifest, bundle, staged_at_seconds)?;
    let commit_hex = commit_id.to_string();

    // The fetch-back verification: the re-read bytes must hash identically.
    let expected = crate::fetcher::digest_sha256_hex(manifest.as_bytes());
    let object = repo
        .find_object(manifest_blob)
        .map_err(|e| PublishError::FetchBack(e.to_string()))?;
    let fetched: &[u8] = object.data.as_slice();
    let actual = crate::fetcher::digest_sha256_hex(fetched);
    if actual != expected {
        return Err(PublishError::FetchBack(format!(
            "the digest mismatch: expected {expected}, fetched {actual}"
        )));
    }

    // The staging branch (the idempotent re-write: the same content commits
    // identically).
    let staging_name = format!("refs/rb/staging/{publication_id}");
    let staging_ref = gix::refs::FullName::try_from(staging_name.as_str())
        .map_err(|e| PublishError::Write(e.to_string()))?;
    repo.edit_reference(gix::refs::transaction::RefEdit {
        change: gix::refs::transaction::Change::Update {
            log: Default::default(),
            expected: gix::refs::transaction::PreviousValue::Any,
            new: gix::refs::Target::Object(commit_id),
        },
        name: staging_ref,
        deref: false,
    })
    .map_err(|e| PublishError::Write(e.to_string()))?;

    // The IMMUTABLE publication ref (the written-once rule).
    let publication_ref_name = format!("refs/rb/publications/{publication_id}");
    let publication_ref = gix::refs::FullName::try_from(publication_ref_name.as_str())
        .map_err(|e| PublishError::Write(e.to_string()))?;
    repo.edit_reference(gix::refs::transaction::RefEdit {
        change: gix::refs::transaction::Change::Update {
            log: Default::default(),
            expected: gix::refs::transaction::PreviousValue::MustNotExist,
            new: gix::refs::Target::Object(commit_id),
        },
        name: publication_ref,
        deref: false,
    })
    .map_err(|e| {
        // Classified by observed state, like the compare-and-swap below: the
        // ref holding ANOTHER commit is the written-once rule refusing; anything
        // else is a write failure. (The same commit is a no-op, not an error.)
        match current(&repo, &publication_ref_name) {
            Some(existing) if existing != commit_id => PublishError::ImmutableExists,
            _ => PublishError::Write(e.to_string()),
        }
    })?;

    // The EFFECTIVE channel via the compare-and-swap.
    let effective_ref = gix::refs::FullName::try_from("refs/rb/effective")
        .map_err(|e| PublishError::Write(e.to_string()))?;
    let previous = match expected_effective {
        Some(old) => {
            gix::refs::transaction::PreviousValue::MustExistAndMatch(gix::refs::Target::Object(old))
        }
        None => gix::refs::transaction::PreviousValue::MustNotExist,
    };
    repo.edit_reference(gix::refs::transaction::RefEdit {
        change: gix::refs::transaction::Change::Update {
            log: Default::default(),
            expected: previous,
            new: gix::refs::Target::Object(commit_id),
        },
        name: effective_ref,
        deref: false,
    })
    .map_err(|e| {
        // ⛔ CLASSIFIED BY THE REF'S OBSERVED STATE, not by the error's words
        // (`SIGNOFF-REPAIR.9.3.5.1.2`). This matched the phrase "should have
        // content", which gix uses only when the ref EXISTS with another value;
        // an expected ref that is ABSENT fails with different words, so that
        // compare-and-swap failure was reported as a write failure.
        let found = current(&repo, "refs/rb/effective");
        if found != expected_effective {
            PublishError::CasMismatch(format!(
                "expected `{}`, found `{}`",
                expected_effective.map_or("no effective ref".to_string(), |id| id.to_string()),
                found.map_or("no effective ref".to_string(), |id| id.to_string()),
            ))
        } else {
            PublishError::Write(e.to_string())
        }
    })?;

    Ok(PublishedRefs {
        publication_ref_id: commit_hex.clone(),
        effective_ref_id: commit_hex,
    })
}
