//! The publication reconciliation matrix (`PHASE-6.4.3.3`, ADR-020, §15.8):
//! a PURE function over (the DB state, the Git observations, the expected
//! object id) → the reconciler action. The six matrix rows are exhaustive;
//! the actions are idempotent (the repeated observation yields the same
//! action); the failed/later-appearing pair is the QUARANTINE — never a
//! silent promote.

/// The publication's database state (the `.4.2` stage machine).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbState {
    Staged,
    Effective,
    Failed,
}

/// The observed Git state for one publication: the immutable ref, the
/// effective channel, and the staging branch (each `None` when absent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitState {
    pub immutable: Option<gix::ObjectId>,
    pub effective: Option<gix::ObjectId>,
    pub staging: Option<gix::ObjectId>,
}

/// The reconciler's action (the §15.8 matrix's right column).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// The staged/absent pair: retry the safe staged write (the same
    /// content commits identically — the idempotent retry).
    RetryStagedWrite,
    /// The staged/matching pair: verify the fetched-back content and
    /// advance to the effective.
    VerifyAndAdvance,
    /// The staged/conflicting pair: stop with the security alert + the
    /// human resolution (never a silent pick).
    StopSecurityAlert,
    /// The effective/missing-or-moved pair: freeze the deployment and
    /// restore only through the authorized repair.
    FreezeAndRepair,
    /// The failed/later-appearing pair: quarantine and adjudicate — NEVER
    /// a silent promote.
    QuarantineAndAdjudicate,
    /// The no-record/out-of-band pair: verify the signature and alert.
    OutOfBandAlert,
    /// The consistent pair: nothing to do.
    Consistent,
}

/// The six-row §15.8 reconciliation. `expected_immutable` is the object id
/// the record expects (the staged publication's digest-derived commit, or
/// the effective row's recorded id); a `None` database state is the
/// no-record row.
///
/// `expected_channel` is what `refs/rb/effective` must hold, and it is `Some`
/// only for the effective publication at the HEAD of its repository's chain
/// (`SIGNOFF-REPAIR.11.56`). Publications have no superseded state, so every
/// older effective publication legitimately sees the channel hold a later
/// commit; for those the channel is not theirs to judge and the caller passes
/// `None`. Until `.11.56` the channel was read by [`observe`] and compared by
/// nothing, so §15.8's *"effective / ref missing or moved"* could not fire for
/// a moved or deleted channel.
pub fn reconcile(
    db: Option<&DbState>,
    git: &GitState,
    expected_immutable: Option<&gix::ObjectId>,
    expected_channel: Option<&gix::ObjectId>,
) -> Action {
    let Some(db) = db else {
        // The no-record row: a ReasonBraid-looking ref without a DB record.
        if git.immutable.is_some() || git.staging.is_some() {
            return Action::OutOfBandAlert;
        }
        return Action::Consistent;
    };
    match db {
        DbState::Staged => match (&git.immutable, expected_immutable) {
            (None, _) => Action::RetryStagedWrite,
            (Some(found), Some(expected)) if found == expected => Action::VerifyAndAdvance,
            (Some(_), _) => Action::StopSecurityAlert,
        },
        DbState::Effective => match (&git.immutable, expected_immutable) {
            (Some(found), Some(expected)) if found == expected => match expected_channel {
                Some(channel) if git.effective.as_ref() != Some(channel) => Action::FreezeAndRepair,
                _ => Action::Consistent,
            },
            _ => Action::FreezeAndRepair,
        },
        DbState::Failed => {
            // The later-appearing write: the immutable ref appeared AFTER
            // the failure — quarantine and adjudicate, never promote.
            if git.immutable.is_some() || git.staging.is_some() {
                Action::QuarantineAndAdjudicate
            } else {
                Action::Consistent
            }
        }
    }
}

/// Read a publication's three refs from its repository
/// (`SIGNOFF-REPAIR.9.3.5.1.2`) — the observation [`reconcile`] consumes.
/// An absent ref is `None`; a repository that does not open is an error, never
/// an "everything absent" observation, which would read as `RetryStagedWrite`.
pub fn observe(repo_path: &std::path::Path, publication_id: &str) -> Result<GitState, String> {
    let repo = gix::open(repo_path).map_err(|e| e.to_string())?;
    let read = |name: String| -> Result<Option<gix::ObjectId>, String> {
        repo.try_find_reference(name.as_str())
            .map(|found| found.map(|r| r.id().detach()))
            .map_err(|e| e.to_string())
    };
    Ok(GitState {
        immutable: read(format!("refs/rb/publications/{publication_id}"))?,
        effective: read("refs/rb/effective".to_string())?,
        staging: read(format!("refs/rb/staging/{publication_id}"))?,
    })
}
