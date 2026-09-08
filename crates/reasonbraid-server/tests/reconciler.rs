//! The reconciliation matrix's kill-point proofs (`.4.3.3`, ADR-020,
//! §15.8): each of the six rows maps to its action; the idempotency holds
//! (the same pair yields the same action); the failed/later-appearing pair
//! is the QUARANTINE — never a silent promote. OFFLINE — the pure function.

use reasonbraid_server::reconciler::{reconcile, Action, DbState, GitState};

fn id(byte: u8) -> gix::ObjectId {
    let mut bytes = [0u8; 20];
    bytes[19] = byte;
    gix::ObjectId::from(bytes)
}

fn git(immutable: Option<u8>, effective: Option<u8>, staging: Option<u8>) -> GitState {
    GitState {
        immutable: immutable.map(id),
        effective: effective.map(id),
        staging: staging.map(id),
    }
}

#[test]
fn the_six_matrix_rows_map_to_their_actions() {
    // 1. staged/absent → the idempotent retry.
    assert_eq!(
        reconcile(Some(&DbState::Staged), &git(None, None, None), None),
        Action::RetryStagedWrite
    );
    // 2. staged/matching → the verify-and-advance.
    assert_eq!(
        reconcile(
            Some(&DbState::Staged),
            &git(Some(7), None, None),
            Some(&id(7))
        ),
        Action::VerifyAndAdvance
    );
    // 3. staged/conflicting → the stop + the alert (never a silent pick).
    assert_eq!(
        reconcile(
            Some(&DbState::Staged),
            &git(Some(9), None, None),
            Some(&id(7))
        ),
        Action::StopSecurityAlert
    );
    // 4. effective/missing-or-moved → the freeze + the authorized repair.
    assert_eq!(
        reconcile(
            Some(&DbState::Effective),
            &git(None, None, None),
            Some(&id(7))
        ),
        Action::FreezeAndRepair
    );
    assert_eq!(
        reconcile(
            Some(&DbState::Effective),
            &git(Some(9), None, None),
            Some(&id(7))
        ),
        Action::FreezeAndRepair
    );
    // 5. failed/later-appearing → the quarantine — NEVER a silent promote.
    assert_eq!(
        reconcile(
            Some(&DbState::Failed),
            &git(Some(7), None, None),
            Some(&id(7))
        ),
        Action::QuarantineAndAdjudicate
    );
    // 6. no-record/out-of-band → the verify + the alert.
    assert_eq!(
        reconcile(None, &git(Some(7), None, None), Some(&id(7))),
        Action::OutOfBandAlert
    );
}

#[test]
fn the_consistent_pairs_are_quiet() {
    // The effective + the matching immutable: nothing to do.
    assert_eq!(
        reconcile(
            Some(&DbState::Effective),
            &git(Some(7), Some(8), None),
            Some(&id(7))
        ),
        Action::Consistent
    );
    // The failed + nothing appearing: the failure stands.
    assert_eq!(
        reconcile(Some(&DbState::Failed), &git(None, None, None), None),
        Action::Consistent
    );
    // The no-record + nothing: quiet.
    assert_eq!(
        reconcile(None, &git(None, None, None), None),
        Action::Consistent
    );
}

#[test]
fn the_reconciler_is_idempotent_over_the_unchanged_pair() {
    // The repeated observation of the same pair yields the same action —
    // the reconciler converges, it never flips.
    let cases: Vec<(Option<DbState>, GitState, Option<gix::ObjectId>)> = vec![
        (Some(DbState::Staged), git(None, None, None), None),
        (Some(DbState::Staged), git(Some(7), None, None), Some(id(7))),
        (Some(DbState::Staged), git(Some(9), None, None), Some(id(7))),
        (Some(DbState::Failed), git(Some(7), None, None), Some(id(7))),
        (
            Some(DbState::Effective),
            git(Some(7), Some(8), None),
            Some(id(7)),
        ),
        (None, git(Some(7), None, None), Some(id(7))),
    ];
    for (db, git, expected) in &cases {
        let first = reconcile(db.as_ref(), git, expected.as_ref());
        let second = reconcile(db.as_ref(), git, expected.as_ref());
        assert_eq!(first, second, "the action repeats");
    }
}
