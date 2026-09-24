//! The workload-identity leaf's shared timing (ADR-007).
//!
//! One constant both sides of the node channel must agree on
//! (`SIGNOFF-REPAIR.4.1.7`): the node rotates its leaf when this little validity
//! remains, and the server refuses to issue a leaf that would be inside the
//! window at birth, since such a leaf is due for rotation the instant it exists
//! and every handshake would rotate again. It lived in the node alone, where the
//! server could not see it.

/// Rotate the leaf once this many seconds or fewer of its validity remain.
/// Half of the 10-minute leaf ADR-007 specifies.
pub const LEAF_ROTATE_REMAINING_SECS: i64 = 300;
