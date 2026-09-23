//! §4.2's `conditions[]` (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.3`): the two halves
//! every member of the vocabulary needs — what it may say at issuance, and
//! whether it holds at an admission. The vocabulary itself is the core type
//! (`reasonbraid_core::GrantCondition`); this module is where a member earns
//! its place, because a member without an evaluator is not a member.
//!
//! # Adding a member
//!
//! 1. a typed variant on the core enum, `deny_unknown_fields`, snake-case kind;
//! 2. an arm in [`issuance_violations`] refusing every declaration it cannot
//!    give a meaning to;
//! 3. an arm in [`holds`] evaluating it over facts the admission already holds
//!    (`at` is the admission's database time; the target and the actor are on
//!    the `CommandAuthz`) — a member that needs a fact the admission does not
//!    hold waits until the admission holds it;
//! 4. a control that issues it, is refused by it, and is admitted under it;
//! 5. the book's grant-fields paragraph.

use chrono::{DateTime, Utc};
use reasonbraid_core::{AuthorityGrant, BoundaryViolation, GrantCondition};

/// What a grant's `conditions` may say: an empty list binds nothing (omit the
/// field instead), and every member is well-formed by its own rule.
pub(crate) fn issuance_violations(grant: &AuthorityGrant) -> Vec<BoundaryViolation> {
    let Some(conditions) = &grant.conditions else {
        return Vec::new();
    };
    let mut violations = Vec::new();
    if conditions.is_empty() {
        violations.push(BoundaryViolation {
            field: "grant.conditions",
            detail: "an empty condition list binds nothing — omit the field".into(),
        });
    }
    for condition in conditions {
        match condition {
            GrantCondition::WithinHours { window } => {
                if let Err(error) = crate::wake::OperatingHours::parse(window) {
                    violations.push(BoundaryViolation {
                        field: "grant.conditions.within_hours",
                        detail: format!("`{window}` is not a window: {error}"),
                    });
                }
            }
        }
    }
    violations
}

/// Whether one condition holds at the admission. `Err` carries the denial's
/// reason, which names the condition and the fact that failed it.
pub(crate) fn holds(condition: &GrantCondition, at: DateTime<Utc>) -> Result<(), String> {
    match condition {
        GrantCondition::WithinHours { window } => {
            // A stored window the parser cannot read denies, fail-closed: the
            // issuance refused it, so this is a hand-edited row.
            let hours = crate::wake::OperatingHours::parse(window).map_err(|error| {
                format!("the grant's within_hours window is unreadable: {error}")
            })?;
            if hours.admits(at.time()) {
                Ok(())
            } else {
                Err(format!(
                    "the grant's within_hours condition does not hold: {} UTC is outside `{window}`",
                    at.format("%H:%M")
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn grant_with(conditions: Option<Vec<GrantCondition>>) -> AuthorityGrant {
        AuthorityGrant {
            grant_id: "grt_conditions".into(),
            boundary_id: "bnd_conditions".into(),
            tenant_id: "ten_00000000-0000-7000-8000-000000000001".parse().unwrap(),
            issuer: "hpr_00000000-0000-7000-8000-000000000001".parse().unwrap(),
            subject: reasonbraid_core::GrantSubject::Role(
                "rol_00000000-0000-7000-8000-000000000001".parse().unwrap(),
            ),
            actions: vec![reasonbraid_core::GrantAction::ThreadContribute],
            selector: reasonbraid_core::TargetSelector::TenantWide,
            risk_ceiling: reasonbraid_core::RiskClass::Low,
            spend_limits: None,
            auto_bounds: None,
            decision_rule_constraints: None,
            conditions,
            delegable: false,
            valid_from: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
            expires_at: Utc.with_ymd_and_hms(2027, 9, 1, 0, 0, 0).unwrap(),
            status: reasonbraid_core::GrantStatus::Active,
        }
    }

    /// `SIGNOFF-REPAIR.11.4.7.2.1.5.4.3` — issuance refuses what it cannot
    /// give a meaning to, and nothing else.
    #[test]
    fn issuance_refuses_an_empty_list_and_a_malformed_window() {
        assert!(issuance_violations(&grant_with(None)).is_empty());
        assert_eq!(issuance_violations(&grant_with(Some(vec![]))).len(), 1);
        let bad = grant_with(Some(vec![GrantCondition::WithinHours {
            window: "9-17".into(),
        }]));
        let violations = issuance_violations(&bad);
        assert_eq!(violations.len(), 1, "{violations:?}");
        assert!(violations[0].detail.contains("not a window"));
        let good = grant_with(Some(vec![GrantCondition::WithinHours {
            window: "09:00-17:00".into(),
        }]));
        assert!(issuance_violations(&good).is_empty());
    }

    /// The window is inclusive at its start, exclusive at its end, and wraps
    /// midnight — the profile's `operating_hours` semantics, by the same parser.
    #[test]
    fn within_hours_holds_inside_the_window_and_denies_outside_it() {
        let day = |h, m| Utc.with_ymd_and_hms(2026, 9, 23, h, m, 0).unwrap();
        let office = GrantCondition::WithinHours {
            window: "09:00-17:00".into(),
        };
        assert!(holds(&office, day(9, 0)).is_ok());
        assert!(holds(&office, day(16, 59)).is_ok());
        let denied = holds(&office, day(17, 0)).unwrap_err();
        assert!(
            denied.contains("17:00 UTC is outside `09:00-17:00`"),
            "{denied}"
        );
        assert!(holds(&office, day(3, 0)).is_err());
        let night = GrantCondition::WithinHours {
            window: "22:00-06:00".into(),
        };
        assert!(holds(&night, day(23, 30)).is_ok());
        assert!(holds(&night, day(5, 59)).is_ok());
        assert!(holds(&night, day(12, 0)).is_err());
        // A stored window nobody could have issued denies, fail-closed.
        let broken = GrantCondition::WithinHours {
            window: "sometimes".into(),
        };
        assert!(holds(&broken, day(12, 0))
            .unwrap_err()
            .contains("unreadable"));
    }
}
