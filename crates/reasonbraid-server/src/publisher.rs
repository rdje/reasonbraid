//! The Git publication half (`PHASE-6.4.3.2`, ADR-020 + the
//! `.4.3.1` store contract): the publisher writes the bundle + the
//! manifest into the LOCAL bare repository — the staging branch, the
//! fetch-back verification (the digest re-derived from the written blobs),
//! the IMMUTABLE publication ref (written once), and the EFFECTIVE channel
//! via the compare-and-swap (the expected old id). No git CLI — the gix
//! plumbing, the pure-Rust doctrine.

use std::path::Path;

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
) -> Result<PublishedRefs, PublishError> {
    let repo = gix::open(repo_path).map_err(|e| PublishError::Open(e.to_string()))?;

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
    let raw_signature = signature(gix::date::Time::now_local_or_utc());
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
            new: gix::refs::Target::Object(commit_id.detach()),
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
            new: gix::refs::Target::Object(commit_id.detach()),
        },
        name: publication_ref,
        deref: false,
    })
    .map_err(|e| {
        if e.to_string().contains("not supposed to exist") {
            PublishError::ImmutableExists
        } else {
            PublishError::Write(e.to_string())
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
            new: gix::refs::Target::Object(commit_id.detach()),
        },
        name: effective_ref,
        deref: false,
    })
    .map_err(|e| {
        if e.to_string().contains("should have content") {
            PublishError::CasMismatch(e.to_string())
        } else {
            PublishError::Write(e.to_string())
        }
    })?;

    Ok(PublishedRefs {
        publication_ref_id: commit_hex.clone(),
        effective_ref_id: commit_hex,
    })
}
