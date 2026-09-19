//! The development budget model (`PHASE-0.5.2`; `ROADMAP.md` §14): multi-dimensional
//! ceilings, reservations against them, and the invariant that **no provider dispatch
//! begins without an applicable reservation** (§14.3 step 4, §14.6).
//!
//! # Dimensions are unknown-or-measured, never zero
//!
//! A dimension that a ceiling does not meter is unconstrained (the reservation's own
//! dimension governs); a dimension that is tracked is a hard number. `None` means
//! "not tracked here", never "free" and never "zero" (§14.1).
//!
//! # The arithmetic is total and fallible
//!
//! [`BudgetDimensions::covers`], [`BudgetDimensions::add`], and
//! [`BudgetDimensions::subtract`] are the only operations the engine uses; the server
//! and the node each run them against their own ledgers (the acceptance: denials
//! cross both boundaries). Underflow and non-coverage are typed errors, never
//! saturating arithmetic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The dev-profile budget dimensions (§14.1's subset): calls, tokens in/out, and
/// wall-clock. `None` = not tracked (no constraint from THIS side).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetDimensions {
    #[serde(default)]
    pub calls: Option<u64>,
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    #[serde(default)]
    pub wall_clock_seconds: Option<u64>,
}

impl BudgetDimensions {
    /// A reservation must at least cover ONE dispatch.
    pub fn covers_a_dispatch(&self) -> bool {
        self.calls.unwrap_or(0) >= 1
    }

    /// Does `self` cover `requested`? FAIL CLOSED: every dimension `requested` tracks
    /// must be tracked AND covered by `self` — a ceiling that does not meter a
    /// dimension cannot vouch for it (§14.6: never silently exceed the hard ceiling).
    pub fn covers(&self, requested: &BudgetDimensions) -> bool {
        let dim = |mine: Option<u64>, asked: Option<u64>| match asked {
            None => true,
            Some(asked) => mine.is_some_and(|mine| mine >= asked),
        };
        dim(self.calls, requested.calls)
            && dim(self.input_tokens, requested.input_tokens)
            && dim(self.output_tokens, requested.output_tokens)
            && dim(self.wall_clock_seconds, requested.wall_clock_seconds)
    }

    /// The held-sum of two dimension sets (ledger arithmetic).
    pub fn add(&self, other: &BudgetDimensions) -> BudgetDimensions {
        let sum = |a: Option<u64>, b: Option<u64>| match (a, b) {
            (Some(a), Some(b)) => Some(a + b),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        };
        BudgetDimensions {
            calls: sum(self.calls, other.calls),
            input_tokens: sum(self.input_tokens, other.input_tokens),
            output_tokens: sum(self.output_tokens, other.output_tokens),
            wall_clock_seconds: sum(self.wall_clock_seconds, other.wall_clock_seconds),
        }
    }

    /// Subtract `used` from `self`; any dimension that would underflow is a typed
    /// error (never saturating — an over-release is a bug, not a rounding).
    pub fn subtract(&self, used: &BudgetDimensions) -> Result<BudgetDimensions, BudgetError> {
        fn sub(
            mine: Option<u64>,
            used: Option<u64>,
            dimension: &'static str,
        ) -> Result<Option<u64>, BudgetError> {
            match (mine, used) {
                (_, None) => Ok(mine),
                (Some(m), Some(u)) if m >= u => Ok(Some(m - u)),
                (Some(m), Some(u)) => Err(BudgetError::Underflow {
                    dimension,
                    held: m,
                    used: u,
                }),
                (None, Some(u)) => Err(BudgetError::Underflow {
                    dimension,
                    held: 0,
                    used: u,
                }),
            }
        }
        Ok(BudgetDimensions {
            calls: sub(self.calls, used.calls, "calls")?,
            input_tokens: sub(self.input_tokens, used.input_tokens, "input_tokens")?,
            output_tokens: sub(self.output_tokens, used.output_tokens, "output_tokens")?,
            wall_clock_seconds: sub(
                self.wall_clock_seconds,
                used.wall_clock_seconds,
                "wall_clock_seconds",
            )?,
        })
    }

    /// The usage dimensions of one dispatched attempt, derived from a normalized
    /// receipt (a call always counts as one call).
    pub fn attempt_usage(
        input_tokens: Option<u64>,
        output_tokens: Option<u64>,
    ) -> BudgetDimensions {
        BudgetDimensions {
            calls: Some(1),
            input_tokens,
            output_tokens,
            wall_clock_seconds: None,
        }
    }
}

/// A typed budget arithmetic/availability error (mapped to §9.8's
/// `budget_unavailable` at the boundaries).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetError {
    /// The ceiling (or local headroom) does not cover the request.
    Unavailable { detail: String },
    /// The reservation reference is not applicable to a dispatch.
    NotApplicable { detail: String },
    /// A settlement/release overruns what was held.
    Underflow {
        dimension: &'static str,
        held: u64,
        used: u64,
    },
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetError::Unavailable { detail } => write!(f, "budget unavailable: {detail}"),
            BudgetError::NotApplicable { detail } => {
                write!(f, "reservation not applicable: {detail}")
            }
            BudgetError::Underflow {
                dimension,
                held,
                used,
            } => write!(
                f,
                "budget underflow: {used} {dimension} used from {held} held"
            ),
        }
    }
}

impl std::error::Error for BudgetError {}

/// The server-issued reservation proof a node verifies BEFORE dispatching (§14.3
/// step 4). Development profile: unsigned (workload identity signs these in WP7);
/// the reference is still load-bearing — without it the supervisor refuses dispatch.
///
/// ⛔ **`expires_at` is the ledger's own instant, not a second clock**
/// (`SIGNOFF-REPAIR.11.24.1.1.2.2`). The server stops counting an active
/// reservation's hold the moment `budget_reservations.expires_at` passes, and it
/// does so deliberately (`expired_reservations_stop_holding`). Until this field
/// existed the reference carried `issued_at` and no window, so the node ran §14.3
/// step 4 — *verify an applicable reservation before dispatch* — against a proof
/// it could not date, and the two halves of the same rule read different clocks:
/// the allowance was re-lent while the work item still claimed it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReservationReference {
    pub reservation_id: String,
    pub dimensions: BudgetDimensions,
    pub issued_at: DateTime<Utc>,
    /// The instant the issuing ledger stops holding this reservation's dimensions.
    pub expires_at: DateTime<Utc>,
}

impl ReservationReference {
    /// A dispatch may proceed under this reference only if it names a reservation,
    /// covers at least one call, and is still held at `now` — the §14.3 step 4
    /// "applicable reservation" check.
    ///
    /// ⛔ It takes the instant rather than reading a clock, because the caller
    /// already has one and a check that samples its own time cannot be driven to
    /// the boundary by a control.
    pub fn applicable_at(&self, now: DateTime<Utc>) -> Result<(), BudgetError> {
        if self.reservation_id.is_empty() {
            return Err(BudgetError::NotApplicable {
                detail: "the reservation has no id".to_string(),
            });
        }
        if !self.dimensions.covers_a_dispatch() {
            return Err(BudgetError::NotApplicable {
                detail: "the reservation covers no dispatch".to_string(),
            });
        }
        // The boundary is the ledger's: `expires_at > $2` holds, so an instant
        // EQUAL to `expires_at` is already past the window on both sides.
        if now >= self.expires_at {
            return Err(BudgetError::NotApplicable {
                detail: format!(
                    "the reservation's hold lapsed at {} and the ceiling has re-lent it",
                    self.expires_at.to_rfc3339()
                ),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn dims(calls: Option<u64>, input: Option<u64>, output: Option<u64>) -> BudgetDimensions {
        BudgetDimensions {
            calls,
            input_tokens: input,
            output_tokens: output,
            wall_clock_seconds: None,
        }
    }

    /// Coverage: tracked dimensions must fit; untracked ones impose no constraint.
    #[test]
    fn covers_respects_tracked_and_untracked_dimensions() {
        let ceiling = dims(Some(10), Some(1000), None);
        // FAIL CLOSED: a requested dimension the ceiling does not meter is refused.
        assert!(
            !ceiling.covers(&dims(Some(1), Some(500), Some(50))),
            "an unmetered requested dimension must be refused"
        );
        assert!(ceiling.covers(&dims(Some(1), Some(500), None)));
        assert!(
            !ceiling.covers(&dims(Some(11), Some(1), None)),
            "calls exceed"
        );
        assert!(
            !ceiling.covers(&dims(Some(1), Some(1001), None)),
            "tokens exceed"
        );
        // A request with no tracked dims is covered trivially.
        assert!(ceiling.covers(&dims(None, None, None)));
    }

    /// Ledger arithmetic: unknown + measured stays measured (a held unknown is not
    /// zero), and underflow is an error, never saturation.
    #[test]
    fn add_and_subtract_are_total_and_fallible() {
        let a = dims(Some(2), Some(100), None);
        let b = dims(Some(3), None, None);
        assert_eq!(a.add(&b), dims(Some(5), Some(100), None));

        assert_eq!(
            a.subtract(&dims(Some(1), Some(40), None)).unwrap(),
            dims(Some(1), Some(60), None)
        );
        let err = a.subtract(&dims(Some(3), None, None)).unwrap_err();
        assert!(
            matches!(
                err,
                BudgetError::Underflow {
                    dimension: "calls",
                    ..
                }
            ),
            "got: {err}"
        );
        // Subtracting a dimension that was never held fails closed (an over-release
        // is a bug, not a rounding).
        assert!(matches!(
            a.subtract(&dims(None, None, Some(9))),
            Err(BudgetError::Underflow {
                dimension: "output_tokens",
                ..
            })
        ));
    }

    /// The applicability check: a dispatch needs a reservation that covers a call.
    #[test]
    fn a_reservation_must_cover_a_dispatch() {
        let now = chrono::Utc::now();
        let held_until = now + Duration::minutes(10);
        let good = ReservationReference {
            reservation_id: "res_1".to_string(),
            dimensions: dims(Some(1), Some(100), None),
            issued_at: now,
            expires_at: held_until,
        };
        assert!(good.applicable_at(now).is_ok());

        let no_calls = ReservationReference {
            reservation_id: "res_2".to_string(),
            dimensions: dims(None, Some(100), None),
            issued_at: now,
            expires_at: held_until,
        };
        assert!(matches!(
            no_calls.applicable_at(now),
            Err(BudgetError::NotApplicable { .. })
        ));

        let no_id = ReservationReference {
            reservation_id: String::new(),
            dimensions: dims(Some(1), None, None),
            issued_at: now,
            expires_at: held_until,
        };
        assert!(matches!(
            no_id.applicable_at(now),
            Err(BudgetError::NotApplicable { .. })
        ));
    }

    /// `SIGNOFF-REPAIR.11.24.1.1.2.2` — the window is the LEDGER's, so the check
    /// must agree with `expires_at > $2` at the boundary itself, not near it.
    #[test]
    fn a_reservation_is_not_applicable_once_its_hold_has_lapsed() {
        let issued = chrono::Utc::now();
        let expires_at = issued + Duration::minutes(10);
        let reference = ReservationReference {
            reservation_id: "res_window".to_string(),
            dimensions: dims(Some(1), Some(100), None),
            issued_at: issued,
            expires_at,
        };
        assert!(
            reference
                .applicable_at(expires_at - Duration::milliseconds(1))
                .is_ok(),
            "a reservation one millisecond inside its window still holds"
        );
        // The ledger's predicate is `expires_at > now`, so equality is OUTSIDE.
        for (label, at) in [
            ("at the boundary", expires_at),
            ("past the boundary", expires_at + Duration::minutes(1)),
        ] {
            let refusal = reference.applicable_at(at);
            assert!(
                matches!(refusal, Err(BudgetError::NotApplicable { .. })),
                "{label}: a lapsed hold must refuse the dispatch, got {refusal:?}"
            );
            let BudgetError::NotApplicable { detail } = refusal.unwrap_err() else {
                unreachable!("matched above")
            };
            assert!(
                detail.contains("lapsed"),
                "{label}: the refusal must say the hold lapsed, got {detail}"
            );
        }
    }

    /// One attempt's usage: exactly one call plus the measured tokens.
    #[test]
    fn attempt_usage_counts_exactly_one_call() {
        let usage = BudgetDimensions::attempt_usage(Some(10), Some(20));
        assert_eq!(usage.calls, Some(1));
        assert_eq!(usage.input_tokens, Some(10));
        assert_eq!(usage.output_tokens, Some(20));
    }
}
