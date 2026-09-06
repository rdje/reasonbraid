//! reasonbraid-server — the control-plane crate (`KICKOFF.md` §3).
//!
//! WP2 (`.2.1`) lands the atomic command transaction here; `.2.2` lands the leased outbox
//! worker with fencing and kill-point tests on top of it.
//!
//! - [`tx::apply_command`] writes current `aggregate_state`, the ordered `event_log`, the
//!   `idempotency` result, and an `outbox` item in one PostgreSQL transaction — a successful
//!   response corresponds to committed durable state, a resubmission with the same key+hash
//!   returns the original result, a different hash is a conflict, and a transport redelivery
//!   produces exactly one domain effect.
//! - [`outbox`] is the leased worker over that outbox: [`outbox::claim_ready`] leases ready
//!   items with a per-claim fencing token and expiry, [`outbox::deliver`] writes the deduped
//!   delivery effect, and [`outbox::complete`] acknowledges — a stale worker whose lease was
//!   superseded by a newer fencing value can never commit.
//!
//! The schema lives at the repository-root `migrations/` (as `KICKOFF.md` §3 sketches);
//! it is applied by the integration tests via [`sqlx::migrate!`] and by
//! `scripts/run_pg_tests.sh` / CI. The HTTP/SSE command surface and the node channel are
//! later leaves (`.2.2` completes WP2; WP3/WP6 follow).

mod outbox;
mod tx;

pub use outbox::{
    claim_ready, complete, deliver, ClaimedOutboxItem, CompleteOutcome, DeliverOutcome,
};
pub use tx::{apply_command, ApplyError, Command, CommandOutcome};
