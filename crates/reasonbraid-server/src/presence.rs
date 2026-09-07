//! The presence state machine (`PHASE-3.2.1`, backlog 27): the six §10.2
//! states as a DETERMINISTIC derivation — presence READS, it never writes
//! (presence does not change enrollment). The inputs: the enrollment fact,
//! the suspension flag (the 0017 view), the lease clock, and the profile's
//! declared concurrency.

/// The six §10.2 presence states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceState {
    /// A live lease and the profile's availability admits work.
    Available,
    /// The concurrency accounting says the node is at capacity — the `.4`
    /// capacity-reservations lane owns the input; nothing feeds this arm yet.
    #[allow(dead_code)] // the `.4` lane's named trigger; no input exists today
    Busy,
    /// The profile declared no capacity (`concurrency = 0`): the role
    /// accepts no new work while enrolled and reachable.
    Draining,
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
/// DRAINING; otherwise AVAILABLE. `Busy` waits for the `.4` capacity
/// accounting (no input exists yet — the arm is the named trigger).
pub fn presence_state(
    enrolled: bool,
    suspended: bool,
    lease_live: bool,
    concurrency: Option<i64>,
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
    if concurrency == Some(0) {
        return PresenceState::Draining;
    }
    PresenceState::Available
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The honesty order: an unknown id is unknown, never fabricated.
    #[test]
    fn an_unenrolled_node_is_unknown_whatever_the_other_inputs_say() {
        assert_eq!(
            presence_state(false, true, true, None),
            PresenceState::Unknown
        );
        assert_eq!(
            presence_state(false, false, false, None),
            PresenceState::Unknown
        );
    }

    /// The 0017 rule: suspension outranks the lease.
    #[test]
    fn a_suspended_node_reads_suspended_whatever_the_lease_says() {
        assert_eq!(
            presence_state(true, true, true, None),
            PresenceState::Suspended
        );
        assert_eq!(
            presence_state(true, true, false, None),
            PresenceState::Suspended
        );
    }

    /// The offline-KNOWN state: enrolled + expired lease.
    #[test]
    fn an_enrolled_node_with_an_expired_lease_reads_offline() {
        assert_eq!(
            presence_state(true, false, false, None),
            PresenceState::Offline
        );
    }

    /// The profile's declared zero concurrency drains the node.
    #[test]
    fn a_live_lease_with_zero_declared_concurrency_reads_draining() {
        assert_eq!(
            presence_state(true, false, true, Some(0)),
            PresenceState::Draining
        );
    }

    /// The default: a live lease with capacity admits work.
    #[test]
    fn a_live_lease_with_capacity_reads_available() {
        assert_eq!(
            presence_state(true, false, true, None),
            PresenceState::Available
        );
        assert_eq!(
            presence_state(true, false, true, Some(2)),
            PresenceState::Available
        );
    }
}
