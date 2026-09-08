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
pub fn reconcile(
    db: Option<&DbState>,
    git: &GitState,
    expected_immutable: Option<&gix::ObjectId>,
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
            (Some(found), Some(expected)) if found == expected => Action::Consistent,
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
