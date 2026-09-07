//! The retry policy (`.2.3`; `ROADMAP.md` §14.6, §1104–1108): a PURE decision
//! over the previous attempt's facts. ReasonBraid never silently retries — a
//! re-dispatch happens only where the provider can provably NOT have run
//! (the dispatch boundary was never crossed) or where an explicit
//! possible-duplicate authorization covers the risk.

/// The dev-profile bound on re-dispatches of a retryable class. Exhaustion is
/// a terminal refusal — an item that keeps failing is visible, never retried
/// forever.
pub const MAX_DISPATCH_ATTEMPTS: usize = 3;

/// The re-dispatch verdict for one work item, from its journal facts and its
/// delivery payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryVerdict {
    /// Dispatch (or re-dispatch) is safe under the policy.
    Retry,
    /// The item must NOT be re-dispatched. The reason carries the §9.8
    /// `retry_requires_authorization` code where that is the refusal.
    Refuse { reason: &'static str },
}

/// The pure retry decision (§14.6 classes):
///
/// - `None`/`prepared` — the dispatch boundary was never crossed: the provider
///   was provably never contacted, so re-dispatch cannot duplicate (the
///   existing skip rule).
/// - `failed_before_dispatch` WITHOUT a reservation — the SERVER denied the
///   budget at dispatch time; re-dispatching cannot change that refusal, so
///   the refusal is terminal (the demo's contract: exactly one
///   `failed_before_dispatch` per budget denial).
/// - `failed_before_dispatch` WITH a reservation — the refusal happened on the
///   node side (adapter/cache) before any provider contact: bounded retry,
///   exhausted at [`MAX_DISPATCH_ATTEMPTS`].
/// - `outcome_unknown` — the provider MAY have run (the billable call was
///   accepted): retry ONLY with an explicit possible-duplicate authorization
///   (`allow_possible_duplicate` on the delivery), bounded; without it the
///   refusal names §9.8's `retry_requires_authorization`.
/// - every other status (`completed`, `failed_known`, `reconciled`, …) is
///   terminal: never retried.
pub fn retry_decision(
    latest_status: Option<&str>,
    attempt_count: usize,
    reservation_present: bool,
    duplicate_authorized: bool,
) -> RetryVerdict {
    match latest_status {
        None | Some("prepared") => RetryVerdict::Retry,
        Some("failed_before_dispatch") => {
            if !reservation_present {
                return RetryVerdict::Refuse {
                    reason: "the server denied the reservation — a retry cannot change it",
                };
            }
            if attempt_count >= MAX_DISPATCH_ATTEMPTS {
                return RetryVerdict::Refuse {
                    reason: "the bounded retry budget is exhausted",
                };
            }
            RetryVerdict::Retry
        }
        Some("outcome_unknown") => {
            if !duplicate_authorized {
                return RetryVerdict::Refuse {
                    reason: "retry_requires_authorization",
                };
            }
            if attempt_count >= MAX_DISPATCH_ATTEMPTS {
                return RetryVerdict::Refuse {
                    reason: "the bounded retry budget is exhausted",
                };
            }
            RetryVerdict::Retry
        }
        // A dispatched attempt is in flight (crash between dispatch and
        // completion): proof or adjudication owns it, never a silent retry.
        Some("dispatched") => RetryVerdict::Refuse {
            reason: "the attempt is dispatched — proof or adjudication owns it",
        },
        Some(_) => RetryVerdict::Refuse {
            reason: "the attempt is terminal",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dispatch_boundary_never_crossed_retries_unconditionally() {
        assert_eq!(retry_decision(None, 0, true, false), RetryVerdict::Retry);
        assert_eq!(
            retry_decision(Some("prepared"), 0, false, false),
            RetryVerdict::Retry
        );
    }

    #[test]
    fn a_budget_denied_item_is_terminal_whatever_the_attempt_count() {
        // The server refused to reserve: retrying can never change that.
        for attempts in [0usize, 1, 2, MAX_DISPATCH_ATTEMPTS, 99] {
            assert_eq!(
                retry_decision(Some("failed_before_dispatch"), attempts, false, false),
                RetryVerdict::Refuse {
                    reason: "the server denied the reservation — a retry cannot change it"
                },
                "attempt {attempts}"
            );
        }
    }

    #[test]
    fn a_reserved_pre_dispatch_refusal_retries_bounded() {
        assert_eq!(
            retry_decision(Some("failed_before_dispatch"), 1, true, false),
            RetryVerdict::Retry
        );
        assert_eq!(
            retry_decision(
                Some("failed_before_dispatch"),
                MAX_DISPATCH_ATTEMPTS,
                true,
                false
            ),
            RetryVerdict::Refuse {
                reason: "the bounded retry budget is exhausted"
            }
        );
    }

    #[test]
    fn an_ambiguous_outcome_needs_the_explicit_authorization() {
        assert_eq!(
            retry_decision(Some("outcome_unknown"), 1, true, false),
            RetryVerdict::Refuse {
                reason: "retry_requires_authorization"
            }
        );
        assert_eq!(
            retry_decision(Some("outcome_unknown"), 1, true, true),
            RetryVerdict::Retry
        );
        assert_eq!(
            retry_decision(Some("outcome_unknown"), MAX_DISPATCH_ATTEMPTS, true, true),
            RetryVerdict::Refuse {
                reason: "the bounded retry budget is exhausted"
            }
        );
    }

    #[test]
    fn terminal_states_are_never_retried() {
        for status in ["completed", "failed_known", "reconciled"] {
            assert_eq!(
                retry_decision(Some(status), 1, true, true),
                RetryVerdict::Refuse {
                    reason: "the attempt is terminal"
                },
                "status {status}"
            );
        }
        assert_eq!(
            retry_decision(Some("dispatched"), 1, true, true),
            RetryVerdict::Refuse {
                reason: "the attempt is dispatched — proof or adjudication owns it"
            }
        );
    }
}
