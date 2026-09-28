//! The §15.8 reconciler, operated (`SIGNOFF-REPAIR.9.3.5.1.2`).
//!
//! [`crate::reconciler::reconcile`] is the pure matrix; this module gives it a
//! caller. For one publication it reads the Git operation the row recorded
//! before its write (`SIGNOFF-REPAIR.9.3.5.1.1`), observes the three refs,
//! recomputes the commit the content commits to, asks the matrix what to do,
//! and does it — but only for the two actions §15.8 lets a machine take.
//!
//! ⛔ **Two actions are never automatic.** `StopSecurityAlert` (a conflicting
//! immutable ref) and `QuarantineAndAdjudicate` (a `failed` row whose write
//! appeared later — *never silently promoted*) are REPORTED and nothing is
//! changed; so are `FreezeAndRepair` and `OutOfBandAlert`, whose remedies
//! §15.8 gives to an authorized repair. The reconciler recovers; it never
//! adjudicates.
//!
//! ⭐ **Idempotent by construction.** The commit is reproducible and a ref edit
//! to an identical value is a no-op (measured in `tests/publisher.rs`), so
//! re-running the reconciler over a reconciled publication observes
//! `Consistent` and writes nothing.

use std::path::Path;

use sqlx::PgPool;

use crate::publications::{self, PublicationError};
use crate::publisher::{self, PublishError};
use crate::reconciler::{self, Action, DbState};

/// What reconciling one publication did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The action was applied; the publication is now `effective`.
    Applied(Action),
    /// Database and Git agree; nothing was written.
    Consistent,
    /// An action §15.8 does not let a machine take — reported, nothing changed.
    RequiresHuman { action: Action, why: String },
    /// There is nothing to observe: the row records no Git operation.
    Unreconcilable(String),
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::Applied(action) => write!(f, "applied {action:?}"),
            Outcome::Consistent => write!(f, "consistent"),
            Outcome::RequiresHuman { action, why } => {
                write!(f, "requires a human: {action:?} — {why}")
            }
            Outcome::Unreconcilable(why) => write!(f, "cannot be reconciled: {why}"),
        }
    }
}

/// A fault in the reconciler's own inputs — the store, the repository, Git.
#[derive(Debug)]
pub enum ReconcileError {
    Store(String),
    Repository(String),
    Git(String),
}

impl std::fmt::Display for ReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReconcileError::Store(e) => write!(f, "the publication store failed: {e}"),
            ReconcileError::Repository(e) => write!(f, "the repository is unusable: {e}"),
            ReconcileError::Git(e) => write!(f, "the Git operation failed: {e}"),
        }
    }
}

impl std::error::Error for ReconcileError {}

fn store(e: impl std::fmt::Display) -> ReconcileError {
    ReconcileError::Store(e.to_string())
}

/// Reconcile one publication against its recorded repository, inside the
/// deployment's configured publication `root`.
pub async fn reconcile_publication(
    pool: &PgPool,
    root: Option<&Path>,
    publication_id: &str,
) -> Result<Outcome, ReconcileError> {
    let publication = publications::load(pool, publication_id)
        .await
        .map_err(store)?;
    let db = match publication.state.as_str() {
        "staged" => DbState::Staged,
        "effective" => DbState::Effective,
        "failed" => DbState::Failed,
        other => {
            return Ok(Outcome::Unreconcilable(format!(
                "the publication is at stage `{other}`, which the matrix has no row for"
            )))
        }
    };
    let Some(recorded) = publication.repository.as_deref() else {
        return Ok(Outcome::Unreconcilable(
            "no recorded Git operation — the row predates `migrations/0086` or was never \
             published, so there is no repository to observe"
                .to_string(),
        ));
    };
    let repository = publisher::resolve_repository(root, recorded)
        .map_err(|e| ReconcileError::Repository(e.to_string()))?;
    let git = reconciler::observe(repository.path(), publication_id)
        .map_err(ReconcileError::Repository)?;

    // What the record says the immutable ref should hold.
    let projection = crate::projections::load(pool, &publication.projection_id)
        .await
        .map_err(store)?;
    let manifest = publications::manifest(
        &publication.publication_id,
        &publication.proposal_id,
        &publication.decision_id,
        &publication.approval_id,
        &publication.projection_id,
        &projection.digest,
    );
    let expected = match db {
        DbState::Staged => {
            // ⛔ The compiled inputs must still be the ones staged — the same
            // check the publish verb makes. A moved input is not something to
            // recover by writing it.
            if publications::manifest_digest(&manifest) != publication.manifest_digest {
                return Ok(Outcome::RequiresHuman {
                    action: Action::StopSecurityAlert,
                    why: "the compiled inputs moved under the staged publication — its \
                          manifest no longer digests to what was staged"
                        .to_string(),
                });
            }
            Some(
                publisher::expected_commit(
                    repository.path(),
                    publication_id,
                    &manifest,
                    &projection.bytes,
                    publication.staged_at_seconds,
                )
                .map_err(|e| ReconcileError::Git(e.to_string()))?,
            )
        }
        DbState::Effective => publication
            .git_object_ids
            .first()
            .and_then(|id| id.parse::<gix::ObjectId>().ok()),
        DbState::Failed => None,
    };

    // The channel is judged for the head of the repository's chain only
    // (`SIGNOFF-REPAIR.11.56`). `mark_effective` records the channel commit
    // this publication set as `git_object_ids[1]`, and a later publication
    // records the channel it expected to replace, so this one is superseded
    // exactly when another staged or effective publication in the same
    // repository expected its channel commit. A failed one never wrote, so it
    // supersedes nothing. The read is repository-wide on purpose: the channel
    // is a fact about the repository, not about one tenant.
    let expected_channel = match db {
        DbState::Effective => match publication
            .git_object_ids
            .get(1)
            .and_then(|id| id.parse::<gix::ObjectId>().ok())
        {
            Some(channel) => {
                let superseded: bool = sqlx::query_scalar(
                    "SELECT EXISTS (SELECT 1 FROM policy_publications \
                     WHERE repository = $1 AND expected_effective = $2 \
                     AND publication_id <> $3 AND state IN ('staged', 'effective'))",
                )
                .bind(recorded)
                .bind(channel.to_string())
                .bind(publication_id)
                .fetch_one(pool)
                .await
                .map_err(store)?;
                (!superseded).then_some(channel)
            }
            None => None,
        },
        DbState::Staged | DbState::Failed => None,
    };

    match reconciler::reconcile(
        Some(&db),
        &git,
        expected.as_ref(),
        expected_channel.as_ref(),
    ) {
        Action::Consistent => Ok(Outcome::Consistent),
        action @ (Action::RetryStagedWrite | Action::VerifyAndAdvance) => {
            // Both finish the operation the row recorded: the retry writes it,
            // the verify re-applies it — idempotently, since the commit is
            // identical — and either way the effective CAS is the RECORDED one.
            let expected_effective = publication
                .expected_effective
                .as_deref()
                .map(str::parse::<gix::ObjectId>)
                .transpose()
                .map_err(|e| ReconcileError::Store(e.to_string()))?;
            let refs = match publisher::publish(
                repository.path(),
                publication_id,
                &manifest,
                &projection.bytes,
                expected_effective,
                publication.staged_at_seconds,
            ) {
                Ok(refs) => refs,
                Err(PublishError::CasMismatch(found)) => {
                    return Ok(Outcome::RequiresHuman {
                        action,
                        why: format!(
                            "the effective channel is not what the recorded operation \
                             expected ({found}) — advancing it would overwrite a publication \
                             the record does not know about"
                        ),
                    })
                }
                Err(e) => return Err(ReconcileError::Git(e.to_string())),
            };
            let tenant: Option<String> = sqlx::query_scalar(
                "SELECT tenant_id FROM policy_publications WHERE publication_id = $1",
            )
            .bind(publication_id)
            .fetch_one(pool)
            .await
            .map_err(store)?;
            let tenant = tenant.ok_or_else(|| {
                ReconcileError::Store("the publication has no owning tenant".to_string())
            })?;
            match publications::mark_effective(
                pool,
                &tenant,
                publication_id,
                vec![refs.publication_ref_id, refs.effective_ref_id],
                &repository,
            )
            .await
            {
                Ok(_) => Ok(Outcome::Applied(action)),
                Err(PublicationError::Storage(e)) => Err(ReconcileError::Store(e)),
                Err(e) => Err(store(e)),
            }
        }
        action @ (Action::StopSecurityAlert
        | Action::QuarantineAndAdjudicate
        | Action::FreezeAndRepair
        | Action::OutOfBandAlert) => Ok(Outcome::RequiresHuman {
            why: match action {
                Action::StopSecurityAlert => {
                    "the immutable ref holds a commit this publication's content does not \
                     commit to — never pick a side"
                }
                Action::QuarantineAndAdjudicate => {
                    "the publication failed and its write appeared later — quarantine and \
                     adjudicate, never promote"
                }
                Action::FreezeAndRepair => {
                    "the effective publication's ref is missing or moved — freeze and repair \
                     through the authorized path"
                }
                _ => "a ReasonBraid ref exists with no record — verify and alert",
            }
            .to_string(),
            action,
        }),
    }
}

/// Every publication with a recorded Git operation, oldest first — what one
/// reconciler pass visits.
pub async fn candidates(pool: &PgPool) -> Result<Vec<String>, ReconcileError> {
    sqlx::query_scalar(
        "SELECT publication_id FROM policy_publications \
         WHERE repository IS NOT NULL ORDER BY created_at",
    )
    .fetch_all(pool)
    .await
    .map_err(store)
}
