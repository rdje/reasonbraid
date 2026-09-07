//! reasonbraid-server — the control-plane crate (`KICKOFF.md` §3).
//!
//! WP2 (`.2.1`) lands the atomic command transaction here; `.2.2` lands the leased outbox
//! worker with fencing and kill-point tests on top of it; `.3.2` lands the node channel's
//! server side.
//!
//! - [`tx::apply_command`] writes current `aggregate_state`, the ordered `event_log`, the
//!   `idempotency` result, and an `outbox` item in one PostgreSQL transaction — a successful
//!   response corresponds to committed durable state, a resubmission with the same key+hash
//!   returns the original result, a different hash is a conflict, and a transport redelivery
//!   produces exactly one domain effect. `PHASE-1.1.1` extracts that machinery into the
//!   aggregate/event/outbox library [`agg`] (the single write path; backlog 9, ADR-004) and
//!   re-expresses `tx` as its typed compatibility shim.
//! - [`outbox`] is the leased worker over that outbox: [`outbox::claim_ready`] leases ready
//!   items with a per-claim fencing token and expiry, [`outbox::deliver`] writes the deduped
//!   delivery effect, and [`outbox::complete`] acknowledges — a stale worker whose lease was
//!   superseded by a newer fencing value can never commit.
//! - [`node_channel`] is the server half of the WP3 node channel: the durable per-node inbox
//!   with a monotonic cursor, replay from the cursor the node reports, deduplicated node-event
//!   receipts, and the reconciliation handshake ([`node_channel::node_router`] + the
//!   `migrations/0003_node_inbox.sql` schema).
//!
//! The schema lives at the repository-root `migrations/` (as `KICKOFF.md` §3 sketches);
//! it is applied by the integration tests via [`sqlx::migrate!`] and by
//! `scripts/run_pg_tests.sh` / CI. The HTTP/SSE command surface and the general-purpose
//! API are later leaves (`.3.2` completes WP3; WP5/WP6 follow).

pub mod agg;
mod api;
mod authority;
pub mod broker;
mod budget;
pub mod ca;
mod dependence;
pub mod extraction;
pub mod fetcher;
pub mod git;
mod matching;
pub mod mediated;
mod node_channel;
mod outbox;
mod presence;
mod profiles;
mod recruitment;
mod resolvers;
mod resources;
pub mod ssrf;
mod telemetry;
mod threads;
mod tx;
/// The `.1.6.2` static inspection console (embedded at compile time; no API
/// routes, no write path — the page reads the existing GET surfaces).
pub mod ui;

pub use api::{
    api_router, ApiState, ControlApiError, EnrollRequest, EnrollResponse, PRINCIPAL_HEADER,
};
pub use authority::{
    apply_authorized_command, authorize, create_boundary, create_grant, load_authorization_record,
    AuthorizationOutcome, AuthorizedApplyError, CommandAuthz, GrantRefused,
};
pub use budget::{
    create_ceiling, create_reservation, release_reservation, settle_reservation, Reservation,
    Settlement,
};
pub use node_channel::{
    is_valid_node_identity, node_router, AckRequest, AckResponse, AmbiguousAttempt, ApiError,
    Directive, EventReceipt, EventSubmission, HandshakeRequest, HandshakeResponse,
    HeartbeatRequest, HeartbeatResponse, KnownEvent, NodeChannelState, PollRequest, PollResponse,
    PresenceParams, PresenceResponse, ReplayCommand, CHANNEL_VERSION, LEASE_TTL,
};
pub use outbox::{
    claim_ready, complete, deliver, ClaimedOutboxItem, CompleteOutcome, DeliverOutcome,
};
pub use tx::{apply_command, ApplyError, Command, CommandOutcome};
pub use ui::ui_router;
