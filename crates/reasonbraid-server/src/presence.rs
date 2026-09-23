//! The presence state machine (`PHASE-3.2.1`, backlog 27): the seven §10.2
//! states as a DETERMINISTIC derivation — presence READS, it never writes
//! (presence does not change enrollment). The inputs: the enrollment fact,
//! the suspension flag (the 0017 view), the lease clock, the profile's
//! declared concurrency, and the wake evaluator's verdict on the profile's
//! availability block at this instant (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.1`).

use crate::wake::Hold;

/// The seven §10.2 presence states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceState {
    /// A live lease and the profile's availability admits work.
    Available,
    /// At the capacity the profile declared: the node holds as many commands
    /// as it said it can take (`SIGNOFF-REPAIR.11.24.1.2`).
    Busy,
    /// The profile declared no capacity (`concurrency = 0`): the role
    /// accepts no new work while enrolled and reachable.
    Draining,
    /// Enrolled, leased, and deliberately not woken
    /// (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.1`): the profile's wake policy is
    /// `manual_only`, the instant is outside its operating hours, or the
    /// stored availability block cannot be read (fail-closed). The wake
    /// evaluator hands such a role nothing, and until this state it read
    /// `available` while receiving nothing — the honest-state defect
    /// `.11.24.1.2` repaired for `busy`, in a new place. The reason rides
    /// beside the state as `hold`.
    Held,
    /// Enrolled, but the lease expired — the node is KNOWN, just quiet.
    Offline,
    /// The workload certificate is revoked (the 0017 view) — whatever the
    /// lease says, a revoked node cannot re-handshake.
    Suspended,
    /// No enrollment row exists for the id.
    Unknown,
}

impl PresenceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PresenceState::Available => "available",
            PresenceState::Busy => "busy",
            PresenceState::Draining => "draining",
            PresenceState::Held => "held",
            PresenceState::Offline => "offline",
            PresenceState::Suspended => "suspended",
            PresenceState::Unknown => "unknown",
        }
    }
}

/// The deterministic derivation. The precedence answers the §10.2 honesty
/// order: an UNKNOWN id is never fabricated into an offline node; a SUSPENDED
/// node reads suspended whatever the lease says (the 0017 rule); an expired
/// lease reads OFFLINE; a live lease with zero declared concurrency reads
/// DRAINING; a live lease the wake evaluator HOLDS by policy — `manual_only`,
/// off-hours, or an unreadable block — reads HELD; a node holding as much work
/// as it declared reads BUSY; otherwise AVAILABLE.
///
/// # Where `held` sits (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.1`)
///
/// Below `draining` and above `busy`, by the same durability argument: a
/// policy hold is a DECLARATION about this role (its wake policy, its hours),
/// like zero capacity and unlike the measurement of what it holds right now.
/// `draining` outranks it because a role that declared no capacity is winding
/// down whatever its hours say; `busy` is outranked because a held role is
/// handed nothing, so *at capacity* would promise capacity that is not coming.
///
/// # Where `busy` sits, and why that is an argument rather than an order of
/// implementation (`SIGNOFF-REPAIR.11.24.1.2`)
///
/// `draining` is *declared no capacity* and `busy` is *at declared capacity*,
/// and a node with `concurrency = 0` satisfies both readings of **cannot take
/// work**. It must resolve to exactly one, and deciding by whichever arm
/// happens to be written first is how a precedence chain acquires a wrong
/// answer nobody can see.
///
/// ⭐ **`draining` outranks `busy`, because the chain above it already sorts by
/// how DURABLE the fact is** — no enrolment row, then a revoked certificate,
/// then a lapsed lease, then a declaration, then a measurement of this moment.
/// A profile declaring zero capacity is not transiently full; it is
/// deliberately winding down, and `busy` would tell a caller to wait for
/// capacity that is not coming back on its own.
///
/// ⛔ A consequence worth stating: with `draining` above it, the `busy` arm
/// never sees `concurrency = 0`, so *at capacity* cannot be satisfied by a
/// node that declared none.
///
/// `in_flight` is the number of commands the node HOLDS and has not finished —
/// `node_inbox_state.delivery_state = 'transport_received'`, whose exclusions
/// come from that ladder's own precedence rather than from a list. The
/// derivation and that definition are argued in `migrations/0079`.
pub fn presence_state(
    enrolled: bool,
    suspended: bool,
    lease_live: bool,
    concurrency: Option<i64>,
    in_flight: i64,
    hold: Option<&Hold>,
) -> PresenceState {
    if !enrolled {
        return PresenceState::Unknown;
    }
    if suspended {
        return PresenceState::Suspended;
    }
    if !lease_live {
        return PresenceState::Offline;
    }
    if concurrency == Some(0) || matches!(hold, Some(Hold::Draining)) {
        return PresenceState::Draining;
    }
    if hold.is_some() {
        return PresenceState::Held;
    }
    // ⛔ An UNDECLARED concurrency is not a capacity of zero and not a capacity
    // of one: it is no declaration, and a node that never said what it can take
    // cannot be reported at a limit it never gave. `None` therefore never reads
    // `busy` — the honest answer for it is `available`, exactly as before this
    // arm existed.
    if let Some(declared) = concurrency {
        if in_flight >= declared {
            return PresenceState::Busy;
        }
    }
    PresenceState::Available
}

/// The wake evaluator's verdict on a STORED availability block
/// (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.1`), for the presence reads that hold
/// the block as JSON rather than as the typed profile: absent → no hold; a
/// block that is not the typed struct → held, fail-closed, exactly as the
/// delivery boundary treats it (`node_channel::replay`); otherwise the
/// evaluator's own answer at `at`.
pub fn hold_from_stored(
    block: Option<&serde_json::Value>,
    at: chrono::DateTime<chrono::Utc>,
) -> Option<Hold> {
    let block = block?;
    if block.is_null() {
        return None;
    }
    match serde_json::from_value::<crate::profiles::Availability>(block.clone()) {
        Ok(availability) => crate::wake::hold(Some(&availability), at),
        Err(error) => Some(Hold::Unreadable(crate::wake::FormatError::Block(
            error.to_string(),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.1` — §10.2's seventh state: a role
    /// the wake evaluator holds by policy reads `held`, below `draining` and
    /// above `busy`. Before this state such a role read `available` while the
    /// replay handed it nothing.
    #[test]
    fn a_policy_hold_reads_held_below_draining_and_above_busy() {
        let manual = Hold::ManualOnly;
        let off = Hold::OffHours {
            window: "22:00-06:00".into(),
        };
        assert_eq!(
            presence_state(true, false, true, Some(2), 0, Some(&manual)),
            PresenceState::Held
        );
        assert_eq!(
            presence_state(true, false, true, None, 0, Some(&off)),
            PresenceState::Held
        );
        // Above busy: a held role at declared capacity is held, not busy.
        assert_eq!(
            presence_state(true, false, true, Some(2), 2, Some(&manual)),
            PresenceState::Held
        );
        // Below draining: zero capacity outranks the policy hold, and the
        // evaluator's own draining verdict is the same fact.
        assert_eq!(
            presence_state(true, false, true, Some(0), 0, Some(&manual)),
            PresenceState::Draining
        );
        assert_eq!(
            presence_state(true, false, true, Some(2), 0, Some(&Hold::Draining)),
            PresenceState::Draining
        );
        // Below offline and suspended, like everything after the lease.
        assert_eq!(
            presence_state(true, false, false, Some(2), 0, Some(&manual)),
            PresenceState::Offline
        );
        assert_eq!(
            presence_state(true, true, true, Some(2), 0, Some(&manual)),
            PresenceState::Suspended
        );
        // No hold: unchanged.
        assert_eq!(
            presence_state(true, false, true, Some(2), 0, None),
            PresenceState::Available
        );
    }

    /// `SIGNOFF-REPAIR.11.24.1.2` — §10.2's sixth state, and the boundary it
    /// turns on.
    ///
    /// 🔴 **THE DEFECT.** `Busy` was declared, rendered as `"busy"`, and
    /// constructed nowhere, so a client branching on §10.2's vocabulary had a
    /// value that could never arrive. ⛔ Deleting it was not available: §10.2
    /// says *a role can be `available`, `busy`, `draining`, `offline`,
    /// `suspended`, or `unknown`*, so removing the variant would leave §10.2
    /// conformance to make a vocabulary honest.
    ///
    /// ⭐ The arm that matters is the boundary. A control that only checked a
    /// node far over capacity would pass against `in_flight > declared`,
    /// `>= declared + 1`, or any other off-by-one — so all three of
    /// `declared - 1`, `declared` and `declared + 1` are driven.
    #[test]
    fn a_node_at_its_declared_capacity_is_busy_and_below_it_is_available() {
        // One under: work is still accepted.
        assert_eq!(
            presence_state(true, false, true, Some(2), 1, None),
            PresenceState::Available
        );
        // AT the declared capacity is already busy — "at capacity", not past it.
        assert_eq!(
            presence_state(true, false, true, Some(2), 2, None),
            PresenceState::Busy
        );
        // Over, which a re-delivery or a raised ceiling can produce.
        assert_eq!(
            presence_state(true, false, true, Some(2), 3, None),
            PresenceState::Busy
        );
        // ⛔ The negative arm the acceptance demands: `busy` is not simply
        // reported once a concurrency is declared.
        assert_eq!(
            presence_state(true, false, true, Some(2), 0, None),
            PresenceState::Available
        );
    }

    /// ⛔ An UNDECLARED concurrency never reads `busy`, however much the node
    /// holds: a node that never said what it can take cannot be reported at a
    /// limit it never gave.
    #[test]
    fn an_undeclared_concurrency_is_never_at_capacity() {
        for in_flight in [0, 1, 99] {
            assert_eq!(
                presence_state(true, false, true, None, in_flight, None),
                PresenceState::Available,
                "no declaration, so no capacity to be at ({in_flight} in flight)"
            );
        }
    }

    /// `draining` OUTRANKS `busy`, and the acceptance required that to be
    /// decided by argument before any code — see `presence_state`'s header.
    ///
    /// A node declaring `concurrency = 0` satisfies both readings of *cannot
    /// take work*. The chain sorts by how DURABLE the fact is, and a
    /// declaration outranks a measurement of this moment: `busy` would tell a
    /// caller to wait for capacity that is not coming back on its own.
    #[test]
    fn a_node_declaring_no_capacity_is_draining_rather_than_busy() {
        for in_flight in [0, 1, 7] {
            assert_eq!(
                presence_state(true, false, true, Some(0), in_flight, None),
                PresenceState::Draining,
                "a declaration outranks a measurement ({in_flight} in flight)"
            );
        }
    }

    /// And the states above `busy` keep their precedence: holding work does
    /// not make a suspended, offline or unknown node read `busy`.
    #[test]
    fn work_in_flight_does_not_outrank_suspension_offline_or_unknown() {
        assert_eq!(
            presence_state(false, false, true, Some(1), 5, None),
            PresenceState::Unknown
        );
        assert_eq!(
            presence_state(true, true, true, Some(1), 5, None),
            PresenceState::Suspended
        );
        assert_eq!(
            presence_state(true, false, false, Some(1), 5, None),
            PresenceState::Offline
        );
    }

    /// The honesty order: an unknown id is unknown, never fabricated.
    #[test]
    fn an_unenrolled_node_is_unknown_whatever_the_other_inputs_say() {
        assert_eq!(
            presence_state(false, true, true, None, 0, None),
            PresenceState::Unknown
        );
        assert_eq!(
            presence_state(false, false, false, None, 0, None),
            PresenceState::Unknown
        );
    }

    /// The 0017 rule: suspension outranks the lease.
    #[test]
    fn a_suspended_node_reads_suspended_whatever_the_lease_says() {
        assert_eq!(
            presence_state(true, true, true, None, 0, None),
            PresenceState::Suspended
        );
        assert_eq!(
            presence_state(true, true, false, None, 0, None),
            PresenceState::Suspended
        );
    }

    /// The offline-KNOWN state: enrolled + expired lease.
    #[test]
    fn an_enrolled_node_with_an_expired_lease_reads_offline() {
        assert_eq!(
            presence_state(true, false, false, None, 0, None),
            PresenceState::Offline
        );
    }

    /// The profile's declared zero concurrency drains the node.
    #[test]
    fn a_live_lease_with_zero_declared_concurrency_reads_draining() {
        assert_eq!(
            presence_state(true, false, true, Some(0), 0, None),
            PresenceState::Draining
        );
    }

    /// The default: a live lease with capacity admits work.
    #[test]
    fn a_live_lease_with_capacity_reads_available() {
        assert_eq!(
            presence_state(true, false, true, None, 0, None),
            PresenceState::Available
        );
        assert_eq!(
            presence_state(true, false, true, Some(2), 0, None),
            PresenceState::Available
        );
    }
}
