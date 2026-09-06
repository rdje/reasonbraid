//! Minimal orthogonal lifecycles (`ROADMAP.md` §8.4; `KICKOFF.md` §3 WP1).
//!
//! Phase 0 keeps only the state subsets the first slice needs — the full §8.4 sets are
//! larger and land later. Each aggregate here is a small state machine whose only
//! operation is `apply`: it accepts a named transition and returns the next state, or a
//! [`TransitionError`] for an invalid move. Transitions are total, deterministic, and
//! fallible — they never panic, and they never reach a state through an undocumented
//! back-edge (`ROADMAP.md` §8.4: "back-edges occur through explicit new revisions or
//! actions, not by overwriting history").

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// An invalid state transition.
///
/// Carries the aggregate name, the source state, and the rejected transition so a
/// diagnostic can say exactly what was refused. `PartialEq`/`Eq` make the rejection
/// itself deterministic and testable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionError {
    pub aggregate: &'static str,
    pub from: &'static str,
    pub event: &'static str,
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid {} transition: `{}` -({})-> rejected",
            self.aggregate, self.from, self.event
        )
    }
}

impl std::error::Error for TransitionError {}

// ── Thread ────────────────────────────────────────────────────────────────────

/// Minimal thread lifecycle (`KICKOFF.md` §3 WP1): `open → closing → closed`,
/// with `cancelled` reachable from `open` or `closing`. `closed` and `cancelled` are
/// terminal. The full §8.4 set (`draft`/`paused`/`expired`) is deferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThreadState {
    Open,
    Closing,
    Closed,
    Cancelled,
}

impl ThreadState {
    /// The wire/record name (snake_case, matches the serde serialization).
    pub fn as_str(self) -> &'static str {
        match self {
            ThreadState::Open => "open",
            ThreadState::Closing => "closing",
            ThreadState::Closed => "closed",
            ThreadState::Cancelled => "cancelled",
        }
    }

    /// Apply a thread transition; invalid moves return [`TransitionError`].
    pub fn apply(self, event: ThreadTransition) -> Result<ThreadState, TransitionError> {
        use ThreadState::*;
        use ThreadTransition::*;
        let next = match (self, event) {
            (Open, BeginClose) => Closing,
            (Closing, FinalizeClose) => Closed,
            (Open, Cancel) | (Closing, Cancel) => Cancelled,
            _ => {
                return Err(TransitionError {
                    aggregate: "Thread",
                    from: self.as_str(),
                    event: event.as_str(),
                })
            }
        };
        Ok(next)
    }
}

/// Commands that drive a thread's lifecycle. Transient; not yet a wire type (the
/// operation/event catalogue is backlog 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThreadTransition {
    BeginClose,
    FinalizeClose,
    Cancel,
}

impl ThreadTransition {
    pub fn as_str(self) -> &'static str {
        match self {
            ThreadTransition::BeginClose => "begin_close",
            ThreadTransition::FinalizeClose => "finalize_close",
            ThreadTransition::Cancel => "cancel",
        }
    }
}

// ── Participation ─────────────────────────────────────────────────────────────

/// Minimal participation lifecycle (`KICKOFF.md` §3 WP1): `invited → accepted` /
/// `invited → declined` / `invited → expired`, and `accepted → left`. `declined`,
/// `expired`, and `left` are terminal. The full §8.4 set (`eligible`/`advertised`/
/// `joined`/`observing`/`deferred`/`revoked`) is deferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParticipationState {
    Invited,
    Accepted,
    Declined,
    Expired,
    Left,
}

impl ParticipationState {
    pub fn as_str(self) -> &'static str {
        match self {
            ParticipationState::Invited => "invited",
            ParticipationState::Accepted => "accepted",
            ParticipationState::Declined => "declined",
            ParticipationState::Expired => "expired",
            ParticipationState::Left => "left",
        }
    }

    /// Apply a participation transition; invalid moves return [`TransitionError`].
    pub fn apply(
        self,
        event: ParticipationTransition,
    ) -> Result<ParticipationState, TransitionError> {
        use ParticipationState::*;
        use ParticipationTransition::*;
        let next = match (self, event) {
            (Invited, Accept) => Accepted,
            (Invited, Decline) => Declined,
            (Invited, Expire) => Expired,
            (Accepted, Leave) => Left,
            _ => {
                return Err(TransitionError {
                    aggregate: "Participation",
                    from: self.as_str(),
                    event: event.as_str(),
                })
            }
        };
        Ok(next)
    }
}

/// Commands that drive a participation's lifecycle. Transient; not yet a wire type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParticipationTransition {
    Accept,
    Decline,
    Expire,
    Leave,
}

impl ParticipationTransition {
    pub fn as_str(self) -> &'static str {
        match self {
            ParticipationTransition::Accept => "accept",
            ParticipationTransition::Decline => "decline",
            ParticipationTransition::Expire => "expire",
            ParticipationTransition::Leave => "leave",
        }
    }
}

// ── Provider attempt ──────────────────────────────────────────────────────────

/// Minimal provider-attempt lifecycle (`KICKOFF.md` §3 WP1): `prepared → dispatched`
/// (or `prepared → failed_before_dispatch`), `dispatched → completed` (proven success)
/// or `dispatched → outcome_unknown` (ambiguous), and `outcome_unknown → reconciled`.
/// `completed`, `failed_before_dispatch`, and `reconciled` are terminal.
///
/// A *proven* failure after dispatch (`failed_known` / `cancelled_known` in the full
/// §8.4 set) is deliberately out of Phase 0 scope: the honest minimal answer for an
/// indeterminate attempt is `outcome_unknown → reconciled`, not a guessed failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAttemptState {
    Prepared,
    Dispatched,
    Completed,
    FailedBeforeDispatch,
    OutcomeUnknown,
    Reconciled,
}

impl ProviderAttemptState {
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderAttemptState::Prepared => "prepared",
            ProviderAttemptState::Dispatched => "dispatched",
            ProviderAttemptState::Completed => "completed",
            ProviderAttemptState::FailedBeforeDispatch => "failed_before_dispatch",
            ProviderAttemptState::OutcomeUnknown => "outcome_unknown",
            ProviderAttemptState::Reconciled => "reconciled",
        }
    }

    /// Apply a provider-attempt transition; invalid moves return [`TransitionError`].
    pub fn apply(
        self,
        event: ProviderAttemptTransition,
    ) -> Result<ProviderAttemptState, TransitionError> {
        use ProviderAttemptState::*;
        use ProviderAttemptTransition::*;
        let next = match (self, event) {
            (Prepared, Dispatch) => Dispatched,
            (Prepared, FailBeforeDispatch) => FailedBeforeDispatch,
            (Dispatched, Complete) => Completed,
            (Dispatched, MarkOutcomeUnknown) => OutcomeUnknown,
            (OutcomeUnknown, Reconcile) => Reconciled,
            _ => {
                return Err(TransitionError {
                    aggregate: "ProviderAttempt",
                    from: self.as_str(),
                    event: event.as_str(),
                })
            }
        };
        Ok(next)
    }
}

/// Commands that drive a provider attempt's lifecycle. Transient; not yet a wire type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProviderAttemptTransition {
    Dispatch,
    FailBeforeDispatch,
    Complete,
    MarkOutcomeUnknown,
    Reconcile,
}

impl ProviderAttemptTransition {
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderAttemptTransition::Dispatch => "dispatch",
            ProviderAttemptTransition::FailBeforeDispatch => "fail_before_dispatch",
            ProviderAttemptTransition::Complete => "complete",
            ProviderAttemptTransition::MarkOutcomeUnknown => "mark_outcome_unknown",
            ProviderAttemptTransition::Reconcile => "reconcile",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The single source of truth for each state machine is an explicit (state, event,
    // target) table. The test asserts BOTH that every listed edge resolves to its target
    // AND that every unlisted (state, event) pair in the full product is rejected — so
    // coverage is exhaustive and rejection is deterministic, not a side effect.

    const THREAD_STATES: [ThreadState; 4] = [
        ThreadState::Open,
        ThreadState::Closing,
        ThreadState::Closed,
        ThreadState::Cancelled,
    ];
    const THREAD_EVENTS: [ThreadTransition; 3] = [
        ThreadTransition::BeginClose,
        ThreadTransition::FinalizeClose,
        ThreadTransition::Cancel,
    ];
    const THREAD_VALID: &[(ThreadState, ThreadTransition, ThreadState)] = &[
        (
            ThreadState::Open,
            ThreadTransition::BeginClose,
            ThreadState::Closing,
        ),
        (
            ThreadState::Closing,
            ThreadTransition::FinalizeClose,
            ThreadState::Closed,
        ),
        (
            ThreadState::Open,
            ThreadTransition::Cancel,
            ThreadState::Cancelled,
        ),
        (
            ThreadState::Closing,
            ThreadTransition::Cancel,
            ThreadState::Cancelled,
        ),
    ];

    const PARTICIPATION_STATES: [ParticipationState; 5] = [
        ParticipationState::Invited,
        ParticipationState::Accepted,
        ParticipationState::Declined,
        ParticipationState::Expired,
        ParticipationState::Left,
    ];
    const PARTICIPATION_EVENTS: [ParticipationTransition; 4] = [
        ParticipationTransition::Accept,
        ParticipationTransition::Decline,
        ParticipationTransition::Expire,
        ParticipationTransition::Leave,
    ];
    const PARTICIPATION_VALID: &[(
        ParticipationState,
        ParticipationTransition,
        ParticipationState,
    )] = &[
        (
            ParticipationState::Invited,
            ParticipationTransition::Accept,
            ParticipationState::Accepted,
        ),
        (
            ParticipationState::Invited,
            ParticipationTransition::Decline,
            ParticipationState::Declined,
        ),
        (
            ParticipationState::Invited,
            ParticipationTransition::Expire,
            ParticipationState::Expired,
        ),
        (
            ParticipationState::Accepted,
            ParticipationTransition::Leave,
            ParticipationState::Left,
        ),
    ];

    const ATTEMPT_STATES: [ProviderAttemptState; 6] = [
        ProviderAttemptState::Prepared,
        ProviderAttemptState::Dispatched,
        ProviderAttemptState::Completed,
        ProviderAttemptState::FailedBeforeDispatch,
        ProviderAttemptState::OutcomeUnknown,
        ProviderAttemptState::Reconciled,
    ];
    const ATTEMPT_EVENTS: [ProviderAttemptTransition; 5] = [
        ProviderAttemptTransition::Dispatch,
        ProviderAttemptTransition::FailBeforeDispatch,
        ProviderAttemptTransition::Complete,
        ProviderAttemptTransition::MarkOutcomeUnknown,
        ProviderAttemptTransition::Reconcile,
    ];
    const ATTEMPT_VALID: &[(
        ProviderAttemptState,
        ProviderAttemptTransition,
        ProviderAttemptState,
    )] = &[
        (
            ProviderAttemptState::Prepared,
            ProviderAttemptTransition::Dispatch,
            ProviderAttemptState::Dispatched,
        ),
        (
            ProviderAttemptState::Prepared,
            ProviderAttemptTransition::FailBeforeDispatch,
            ProviderAttemptState::FailedBeforeDispatch,
        ),
        (
            ProviderAttemptState::Dispatched,
            ProviderAttemptTransition::Complete,
            ProviderAttemptState::Completed,
        ),
        (
            ProviderAttemptState::Dispatched,
            ProviderAttemptTransition::MarkOutcomeUnknown,
            ProviderAttemptState::OutcomeUnknown,
        ),
        (
            ProviderAttemptState::OutcomeUnknown,
            ProviderAttemptTransition::Reconcile,
            ProviderAttemptState::Reconciled,
        ),
    ];

    #[test]
    fn thread_transitions_match_table_exhaustively() {
        for &(s, e, want) in THREAD_VALID {
            assert_eq!(s.apply(e), Ok(want), "{s:?} + {e:?}");
        }
        for s in THREAD_STATES {
            for e in THREAD_EVENTS {
                let is_valid = THREAD_VALID.iter().any(|&(vs, ve, _)| vs == s && ve == e);
                if !is_valid {
                    assert!(s.apply(e).is_err(), "{s:?} + {e:?} must be rejected");
                }
            }
        }
    }

    #[test]
    fn participation_transitions_match_table_exhaustively() {
        for &(s, e, want) in PARTICIPATION_VALID {
            assert_eq!(s.apply(e), Ok(want), "{s:?} + {e:?}");
        }
        for s in PARTICIPATION_STATES {
            for e in PARTICIPATION_EVENTS {
                let is_valid = PARTICIPATION_VALID
                    .iter()
                    .any(|&(vs, ve, _)| vs == s && ve == e);
                if !is_valid {
                    assert!(s.apply(e).is_err(), "{s:?} + {e:?} must be rejected");
                }
            }
        }
    }

    #[test]
    fn provider_attempt_transitions_match_table_exhaustively() {
        for &(s, e, want) in ATTEMPT_VALID {
            assert_eq!(s.apply(e), Ok(want), "{s:?} + {e:?}");
        }
        for s in ATTEMPT_STATES {
            for e in ATTEMPT_EVENTS {
                let is_valid = ATTEMPT_VALID.iter().any(|&(vs, ve, _)| vs == s && ve == e);
                if !is_valid {
                    assert!(s.apply(e).is_err(), "{s:?} + {e:?} must be rejected");
                }
            }
        }
    }

    #[test]
    fn terminal_states_reject_every_event() {
        use ThreadState::*;
        use ThreadTransition::*;
        for s in [Closed, Cancelled] {
            for e in [BeginClose, FinalizeClose, Cancel] {
                assert!(s.apply(e).is_err(), "terminal {s:?} must reject {e:?}");
            }
        }
    }

    #[test]
    fn rejections_are_deterministic_and_self_describing() {
        let a = ThreadState::Closed.apply(ThreadTransition::BeginClose);
        let b = ThreadState::Closed.apply(ThreadTransition::BeginClose);
        assert_eq!(a, b);
        let err = a.unwrap_err();
        assert_eq!(err.aggregate, "Thread");
        assert_eq!(err.from, "closed");
        assert_eq!(err.event, "begin_close");
        assert!(err.to_string().contains("closed"));
        assert!(err.to_string().contains("begin_close"));
    }

    #[test]
    fn states_serialize_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&ThreadState::Open).unwrap(),
            "\"open\""
        );
        assert_eq!(
            serde_json::to_string(&ProviderAttemptState::OutcomeUnknown).unwrap(),
            "\"outcome_unknown\""
        );
        let back: ParticipationState = serde_json::from_str("\"accepted\"").unwrap();
        assert_eq!(back, ParticipationState::Accepted);
    }

    #[test]
    fn state_kinds_are_distinct_types() {
        let kinds = [
            std::any::TypeId::of::<ThreadState>(),
            std::any::TypeId::of::<ParticipationState>(),
            std::any::TypeId::of::<ProviderAttemptState>(),
        ];
        for i in 0..kinds.len() {
            for j in (i + 1)..kinds.len() {
                assert_ne!(kinds[i], kinds[j], "state enums {i} and {j} collapsed");
            }
        }
    }
}
