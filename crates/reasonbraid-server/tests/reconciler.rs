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
        reconcile(Some(&DbState::Staged), &git(None, None, None), None, None),
        Action::RetryStagedWrite
    );
    // 2. staged/matching → the verify-and-advance.
    assert_eq!(
        reconcile(
            Some(&DbState::Staged),
            &git(Some(7), None, None),
            Some(&id(7)),
            None
        ),
        Action::VerifyAndAdvance
    );
    // 3. staged/conflicting → the stop + the alert (never a silent pick).
    assert_eq!(
        reconcile(
            Some(&DbState::Staged),
            &git(Some(9), None, None),
            Some(&id(7)),
            None
        ),
        Action::StopSecurityAlert
    );
    // 4. effective/missing-or-moved → the freeze + the authorized repair.
    assert_eq!(
        reconcile(
            Some(&DbState::Effective),
            &git(None, None, None),
            Some(&id(7)),
            None
        ),
        Action::FreezeAndRepair
    );
    assert_eq!(
        reconcile(
            Some(&DbState::Effective),
            &git(Some(9), None, None),
            Some(&id(7)),
            None
        ),
        Action::FreezeAndRepair
    );
    // 5. failed/later-appearing → the quarantine — NEVER a silent promote.
    assert_eq!(
        reconcile(
            Some(&DbState::Failed),
            &git(Some(7), None, None),
            Some(&id(7)),
            None
        ),
        Action::QuarantineAndAdjudicate
    );
    // 6. no-record/out-of-band → the verify + the alert.
    assert_eq!(
        reconcile(None, &git(Some(7), None, None), Some(&id(7)), None),
        Action::OutOfBandAlert
    );
}

#[test]
fn the_consistent_pairs_are_quiet() {
    // The effective + the matching immutable, with the channel on a later
    // commit: a SUPERSEDED publication (no channel expectation), nothing to do.
    assert_eq!(
        reconcile(
            Some(&DbState::Effective),
            &git(Some(7), Some(8), None),
            Some(&id(7)),
            None
        ),
        Action::Consistent
    );
    // The failed + nothing appearing: the failure stands.
    assert_eq!(
        reconcile(Some(&DbState::Failed), &git(None, None, None), None, None),
        Action::Consistent
    );
    // The no-record + nothing: quiet.
    assert_eq!(
        reconcile(None, &git(None, None, None), None, None),
        Action::Consistent
    );
    // `SIGNOFF-REPAIR.11.61`: the no-record + the repository's channel alone is
    // quiet too, and on purpose. `refs/rb/effective` is the REPOSITORY's, so
    // for an id nobody recorded it holds another publication's commit whenever
    // the repository has one; alerting on it would alarm on every repository in
    // use. The id's own refs are the evidence (row 6 above), and a channel
    // moved out of band is judged at the repository's head (`.11.56`).
    assert_eq!(
        reconcile(None, &git(None, Some(8), None), None, None),
        Action::Consistent
    );
}

/// `SIGNOFF-REPAIR.11.56` — the effective CHANNEL is judged for the head of a
/// repository's chain, and only there. The head's channel must hold what it
/// set; a superseded publication (`None`) sees the channel hold a later commit
/// and stays quiet, which is the pair `the_consistent_pairs_are_quiet` names.
#[test]
fn the_heads_channel_is_judged_and_a_superseded_ones_is_not() {
    // The head, channel where it set it: consistent.
    assert_eq!(
        reconcile(
            Some(&DbState::Effective),
            &git(Some(7), Some(8), None),
            Some(&id(7)),
            Some(&id(8))
        ),
        Action::Consistent
    );
    // The head, channel moved: freeze.
    assert_eq!(
        reconcile(
            Some(&DbState::Effective),
            &git(Some(7), Some(9), None),
            Some(&id(7)),
            Some(&id(8))
        ),
        Action::FreezeAndRepair
    );
    // The head, channel missing: freeze.
    assert_eq!(
        reconcile(
            Some(&DbState::Effective),
            &git(Some(7), None, None),
            Some(&id(7)),
            Some(&id(8))
        ),
        Action::FreezeAndRepair
    );
    // A superseded publication, channel on a later commit: quiet.
    assert_eq!(
        reconcile(
            Some(&DbState::Effective),
            &git(Some(7), Some(9), None),
            Some(&id(7)),
            None
        ),
        Action::Consistent
    );
    // The channel never rescues a moved IMMUTABLE ref.
    assert_eq!(
        reconcile(
            Some(&DbState::Effective),
            &git(Some(6), Some(8), None),
            Some(&id(7)),
            Some(&id(8))
        ),
        Action::FreezeAndRepair
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
        let first = reconcile(db.as_ref(), git, expected.as_ref(), None);
        let second = reconcile(db.as_ref(), git, expected.as_ref(), None);
        assert_eq!(first, second, "the action repeats");
    }
}
