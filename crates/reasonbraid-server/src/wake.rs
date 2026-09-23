//! The §11.5 wake evaluator (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2`): ONE reading
//! of a profile's `availability` block for every surface that decides whether
//! a role may be woken (the delivery boundary, `node_channel::replay`) or may
//! wake itself (`POST /v1/threads/auto`), and the FORMAT the profile write
//! refuses to violate.
//!
//! Two of the block's three fields were declared and read nowhere:
//! `wake_policy` and `operating_hours` accepted any text and gated nothing
//! (DOC-0135; DOC-0136 stored `"never"` on the running server and the role
//! still initiated). A field a profile can declare has a format the write
//! refuses to violate and a meaning one evaluator applies, or it is not a
//! field. This module is both halves, so the two cannot drift apart.
//!
//! The formats:
//! - `concurrency`: an integer that is zero or more. Zero is the drain switch
//!   (`.4.2.10`): the role receives no new work and initiates none.
//! - `wake_policy`: `"auto"` (what an absent field means) or `"manual_only"`.
//!   A `manual_only` role is woken by no delivery and initiates nothing on its
//!   own; it participates through an already-running client (§11.6's MCP
//!   active-client path), so autonomy never reaches it.
//! - `operating_hours`: `"HH:MM-HH:MM"` in UTC, 24-hour. The window may wrap
//!   midnight (`"22:00-06:00"`); the start is inclusive and the end exclusive;
//!   equal ends are refused because they could mean an empty window or the
//!   whole day. An absent field means always.
//!
//! ⛔ FAIL-CLOSED: a STORED value the evaluator cannot read holds the role and
//! the hold names the field. The write refuses the format now, so only a row
//! written before this module existed, or edited by hand, can be unreadable —
//! and the repair is to write the profile again.
//!
//! ⚠️ A stored NEGATIVE concurrency still delivers: the drain switch tests for
//! exactly zero (`.4.2.10`'s measured shape) and the write now refuses the
//! value, so it can only be a legacy row. Documented rather than reinterpreted.

use chrono::{DateTime, NaiveTime, Utc};

use crate::profiles::Availability;

pub const WAKE_POLICY_AUTO: &str = "auto";
pub const WAKE_POLICY_MANUAL_ONLY: &str = "manual_only";

/// How a role is woken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakePolicy {
    /// Delivered work wakes the node, and the role may initiate under its grant.
    Auto,
    /// Neither: the role acts only through a client that is already running.
    ManualOnly,
}

impl WakePolicy {
    pub fn parse(raw: &str) -> Result<Self, FormatError> {
        match raw {
            WAKE_POLICY_AUTO => Ok(Self::Auto),
            WAKE_POLICY_MANUAL_ONLY => Ok(Self::ManualOnly),
            other => Err(FormatError::WakePolicy(other.to_string())),
        }
    }
}

/// A daily UTC window, start inclusive and end exclusive, wrapping midnight
/// when the end is before the start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperatingHours {
    start: NaiveTime,
    end: NaiveTime,
}

impl OperatingHours {
    pub fn parse(raw: &str) -> Result<Self, FormatError> {
        let refuse = |why: &'static str| FormatError::OperatingHours {
            value: raw.to_string(),
            why,
        };
        let (start, end) = raw
            .split_once('-')
            .ok_or_else(|| refuse("expected `HH:MM-HH:MM`"))?;
        // Exactly five characters, so `9:00` and `09:00:00` are refused rather
        // than read as something they might not mean.
        let clock = |s: &str| {
            (s.len() == 5)
                .then(|| NaiveTime::parse_from_str(s, "%H:%M").ok())
                .flatten()
        };
        let start = clock(start).ok_or_else(|| refuse("the start is not `HH:MM`"))?;
        let end = clock(end).ok_or_else(|| refuse("the end is not `HH:MM`"))?;
        if start == end {
            return Err(refuse(
                "the start and the end are equal, which could mean an empty window or the whole day",
            ));
        }
        Ok(Self { start, end })
    }

    /// Whether `at` (a UTC time of day) falls inside the window.
    pub fn admits(&self, at: NaiveTime) -> bool {
        if self.start < self.end {
            self.start <= at && at < self.end
        } else {
            at >= self.start || at < self.end
        }
    }
}

/// A declared value the evaluator cannot give a meaning to. The write refuses
/// it; a stored one holds the role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    Concurrency(i64),
    WakePolicy(String),
    OperatingHours {
        value: String,
        why: &'static str,
    },
    /// The stored block itself does not parse as an availability block — a
    /// hand-edited row, since the write only ever stores the typed struct.
    Block(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::Concurrency(n) => write!(
                f,
                "availability.concurrency `{n}` is negative — a capacity is zero or more"
            ),
            FormatError::WakePolicy(value) => write!(
                f,
                "availability.wake_policy `{value}` is neither `{WAKE_POLICY_AUTO}` nor `{WAKE_POLICY_MANUAL_ONLY}`"
            ),
            FormatError::OperatingHours { value, why } => write!(
                f,
                "availability.operating_hours `{value}` is not `HH:MM-HH:MM` (UTC): {why}"
            ),
            FormatError::Block(detail) => {
                write!(f, "the stored availability block does not parse: {detail}")
            }
        }
    }
}

impl std::error::Error for FormatError {}

/// The format half: refuse a block the evaluator could not read. The profile
/// write calls this so an unreadable value never reaches the store.
pub fn validate(availability: &Availability) -> Result<(), FormatError> {
    if let Some(n) = availability.concurrency {
        if n < 0 {
            return Err(FormatError::Concurrency(n));
        }
    }
    if let Some(policy) = &availability.wake_policy {
        WakePolicy::parse(policy)?;
    }
    if let Some(hours) = &availability.operating_hours {
        OperatingHours::parse(hours)?;
    }
    Ok(())
}

/// Why a role may NOT be woken now. The order is the durability of the fact:
/// the drain switch, then the policy declaration, then the declaration read
/// against the clock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hold {
    /// `concurrency: 0`.
    Draining,
    /// `wake_policy: "manual_only"`.
    ManualOnly,
    /// The instant is outside `operating_hours`.
    OffHours { window: String },
    /// A stored value the evaluator cannot read — fail-closed.
    Unreadable(FormatError),
}

impl Hold {
    /// The hold's wire name (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.1`): what a
    /// presence read shows beside `held`, so a client that reads only `state`
    /// sees the honest state and one that reads `hold` sees why.
    pub fn wire_name(&self) -> &'static str {
        match self {
            Hold::Draining => "draining",
            Hold::ManualOnly => "manual_only",
            Hold::OffHours { .. } => "off_hours",
            Hold::Unreadable(_) => "unreadable",
        }
    }
}

impl std::fmt::Display for Hold {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hold::Draining => write!(f, "the role's declared concurrency is zero"),
            Hold::ManualOnly => write!(
                f,
                "the role's wake policy is `{WAKE_POLICY_MANUAL_ONLY}` — it is woken by no delivery and initiates nothing on its own"
            ),
            Hold::OffHours { window } => write!(
                f,
                "the instant is outside the role's operating hours `{window}` (UTC)"
            ),
            Hold::Unreadable(error) => write!(
                f,
                "the stored availability cannot be evaluated and holds the role until the profile is written again: {error}"
            ),
        }
    }
}

/// The meaning half: the one evaluation every wake decision makes. `None`
/// admits. No block at all admits — the common case for a plain node, which
/// declares nothing (`.4.2.10`).
pub fn hold(availability: Option<&Availability>, at: DateTime<Utc>) -> Option<Hold> {
    let availability = availability?;
    if availability.concurrency == Some(0) {
        return Some(Hold::Draining);
    }
    if let Some(raw) = &availability.wake_policy {
        match WakePolicy::parse(raw) {
            Ok(WakePolicy::ManualOnly) => return Some(Hold::ManualOnly),
            Ok(WakePolicy::Auto) => {}
            Err(error) => return Some(Hold::Unreadable(error)),
        }
    }
    if let Some(raw) = &availability.operating_hours {
        match OperatingHours::parse(raw) {
            Ok(window) => {
                if !window.admits(at.time()) {
                    return Some(Hold::OffHours {
                        window: raw.clone(),
                    });
                }
            }
            Err(error) => return Some(Hold::Unreadable(error)),
        }
    }
    None
}

/// How many rows a delivery may hand the node NOW
/// (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.2`): the declared capacity minus the
/// commands it already holds (`node_presence.in_flight`, the ladder's
/// `transport_received` rung). `None` is unbounded — no capacity was declared,
/// or a legacy negative one, which is no declaration. `Some(0)` is §10.2's
/// `busy`: the node is at the capacity it declared and is handed nothing until
/// it finishes something. Zero capacity is the drain switch and is decided by
/// [`hold`] before this is asked; it answers `Some(0)` here too, so the two can
/// never disagree.
pub fn delivery_budget(concurrency: Option<i64>, in_flight: i64) -> Option<i64> {
    match concurrency {
        Some(declared) if declared >= 0 => Some((declared - in_flight).max(0)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_delivery_budget_is_capacity_minus_what_the_node_holds() {
        assert_eq!(delivery_budget(None, 5), None, "no declaration is no limit");
        assert_eq!(
            delivery_budget(Some(-1), 0),
            None,
            "a legacy negative is no declaration"
        );
        assert_eq!(
            delivery_budget(Some(0), 0),
            Some(0),
            "the drain switch agrees with hold"
        );
        assert_eq!(delivery_budget(Some(2), 0), Some(2));
        assert_eq!(delivery_budget(Some(2), 1), Some(1));
        assert_eq!(delivery_budget(Some(2), 2), Some(0), "at capacity: busy");
        assert_eq!(
            delivery_budget(Some(2), 3),
            Some(0),
            "over capacity never goes negative"
        );
    }

    fn clock(s: &str) -> NaiveTime {
        NaiveTime::parse_from_str(s, "%H:%M").unwrap()
    }

    fn at(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(&format!("2026-09-23T{s}:00Z"))
            .unwrap()
            .with_timezone(&Utc)
    }

    fn block(concurrency: Option<i64>, policy: Option<&str>, hours: Option<&str>) -> Availability {
        Availability {
            concurrency,
            wake_policy: policy.map(str::to_string),
            operating_hours: hours.map(str::to_string),
        }
    }

    #[test]
    fn the_formats_are_exact() {
        assert_eq!(WakePolicy::parse("auto"), Ok(WakePolicy::Auto));
        assert_eq!(WakePolicy::parse("manual_only"), Ok(WakePolicy::ManualOnly));
        assert_eq!(
            WakePolicy::parse("never"),
            Err(FormatError::WakePolicy("never".into()))
        );
        assert!(OperatingHours::parse("09:00-17:00").is_ok());
        assert!(
            OperatingHours::parse("22:00-06:00").is_ok(),
            "a wrap is a window"
        );
        for bad in [
            "9-17",
            "9:00-17:00",
            "09:00-17:00:00",
            "25:00-01:00",
            "09:00",
            "",
        ] {
            assert!(
                OperatingHours::parse(bad).is_err(),
                "`{bad}` must be refused"
            );
        }
        assert!(
            matches!(
                OperatingHours::parse("09:00-09:00"),
                Err(FormatError::OperatingHours { why, .. }) if why.contains("equal")
            ),
            "equal ends are ambiguous"
        );
    }

    #[test]
    fn a_window_is_start_inclusive_end_exclusive_and_may_wrap() {
        let day = OperatingHours::parse("09:00-17:00").unwrap();
        assert!(day.admits(clock("09:00")));
        assert!(day.admits(clock("12:00")));
        assert!(day.admits(clock("16:59")));
        assert!(!day.admits(clock("17:00")));
        assert!(!day.admits(clock("08:59")));
        let night = OperatingHours::parse("22:00-06:00").unwrap();
        assert!(night.admits(clock("22:00")));
        assert!(night.admits(clock("23:30")));
        assert!(night.admits(clock("00:00")));
        assert!(night.admits(clock("05:59")));
        assert!(!night.admits(clock("06:00")));
        assert!(!night.admits(clock("12:00")));
    }

    #[test]
    fn validate_refuses_exactly_what_the_evaluator_could_not_read() {
        assert_eq!(
            validate(&block(Some(2), Some("auto"), Some("22:00-06:00"))),
            Ok(())
        );
        assert_eq!(validate(&block(None, None, None)), Ok(()));
        assert_eq!(
            validate(&block(Some(0), None, None)),
            Ok(()),
            "zero is the drain switch"
        );
        assert_eq!(
            validate(&block(Some(-1), None, None)),
            Err(FormatError::Concurrency(-1))
        );
        assert!(matches!(
            validate(&block(None, Some("never"), None)),
            Err(FormatError::WakePolicy(_))
        ));
        assert!(matches!(
            validate(&block(None, None, Some("9-17"))),
            Err(FormatError::OperatingHours { .. })
        ));
    }

    #[test]
    fn the_hold_is_ordered_by_the_durability_of_the_fact() {
        let noon = at("12:00");
        assert_eq!(hold(None, noon), None, "no block admits");
        assert_eq!(hold(Some(&block(None, None, None)), noon), None);
        assert_eq!(
            hold(Some(&block(Some(-1), None, None)), noon),
            None,
            "legacy negative delivers"
        );
        assert_eq!(
            hold(Some(&block(Some(0), Some("manual_only"), None)), noon),
            Some(Hold::Draining),
            "the drain switch outranks the policy"
        );
        assert_eq!(
            hold(
                Some(&block(Some(2), Some("manual_only"), Some("09:00-17:00"))),
                noon
            ),
            Some(Hold::ManualOnly),
            "the policy outranks the clock"
        );
        assert_eq!(
            hold(
                Some(&block(Some(2), Some("auto"), Some("09:00-17:00"))),
                noon
            ),
            None
        );
        assert_eq!(
            hold(
                Some(&block(Some(2), Some("auto"), Some("09:00-17:00"))),
                at("17:00")
            ),
            Some(Hold::OffHours {
                window: "09:00-17:00".into()
            })
        );
        assert_eq!(
            hold(
                Some(&block(Some(2), Some("auto"), Some("22:00-06:00"))),
                at("23:30")
            ),
            None,
            "inside a wrapping window"
        );
        assert!(matches!(
            hold(Some(&block(Some(2), Some("never"), None)), noon),
            Some(Hold::Unreadable(FormatError::WakePolicy(_)))
        ));
        assert!(matches!(
            hold(Some(&block(Some(2), Some("auto"), Some("never"))), noon),
            Some(Hold::Unreadable(FormatError::OperatingHours { .. }))
        ));
    }
}
