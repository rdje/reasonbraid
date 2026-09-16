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
pub mod browse;
mod budget;
pub mod ca;
pub mod cards;
pub mod claims;
pub mod corrections;
mod dependence;
pub mod deployments;
pub mod derivations;
pub mod evaluation;
pub mod extraction;
pub mod extraction_input;
pub mod federation;
pub mod fetcher;
pub mod git;
pub mod lifecycle;
mod matching;
mod mcp_listen;
/// The test seam for the listen-state machinery (the `.3.4` suite).
pub mod mcp_listen_internal {
    pub use crate::mcp_listen::{
        listen_state as state, record_delivery_in_tx as record, resume_plan, ResumePlan,
    };
}
mod mcp_read;
/// The seam for the MCP read half (the `.3.5.2` tools consume this — the same
/// pattern as `mcp_write_internal`). Added by `SIGNOFF-REPAIR.6.1.1`, which
/// found the read tools running a private re-implementation of an
/// authorization their own module header said they shared.
pub mod mcp_read_internal {
    pub use crate::mcp_read::{inbox, policy_bundle, thread, ReadRefused};
}
mod mcp_write;
/// The seam for the MCP write gate (the `.3.5.2` tools + the live suite
/// consume this — the same pattern as `mcp_listen_internal`).
pub mod mcp_write_internal {
    pub use crate::mcp_write::{gate, join_call, propose_policy_change, respond, WriteRefused};
}
mod regions;
pub mod site_authority;
/// The seam for the regional-routing decision (the `.5.2` suite + the
/// `.5.3` store-and-forward consume this).
pub mod regions_internal {
    pub use crate::regions::{route, RegionRefusal, RouteError};
}
pub mod mediated;
pub mod mtls;
mod node_channel;
mod outbox;
pub mod policy;
mod presence;
mod profiles;
pub mod project_storage;
pub mod projections;
pub mod publications;
pub mod publisher;
mod quota;
mod receipts;
pub mod reconciler;
mod recruitment;
mod resolvers;
mod resources;
pub mod reviews;
mod rls;
pub mod routing;
pub mod secret_store;

/// How far a bind address reaches, as a word an operator can read in one line
/// of a log (`SIGNOFF-REPAIR.11.12`).
///
/// ⛔ This is a REPORT, not a gate. `docs/book/src/deployment.md` documents
/// `rb-server --host 0.0.0.0` as the supported **trusted LAN** profile, so a
/// refusal here would break a shipped, documented deployment. What was wrong is
/// that the startup line said "(Phase 0 dev profile)" for EVERY bind, so a log
/// could not tell a loopback boot from one reachable by every host on the
/// network. The decision and where the limit is published:
/// `docs/decisions/2026-09-16_rb-server-bind-exposure.md`.
pub fn bind_exposure(addr: &std::net::SocketAddr) -> &'static str {
    let ip = addr.ip();
    if ip.is_loopback() {
        "loopback"
    } else if ip.is_unspecified() {
        "every interface"
    } else {
        "one named interface"
    }
}

#[cfg(test)]
mod bind_exposure_tests {
    use super::bind_exposure;
    use std::net::SocketAddr;

    #[test]
    fn the_startup_line_can_tell_a_loopback_boot_from_a_reachable_one() {
        for (raw, expected) in [
            ("127.0.0.1:4310", "loopback"),
            ("[::1]:4310", "loopback"),
            ("0.0.0.0:4310", "every interface"),
            ("[::]:4310", "every interface"),
            ("192.168.1.10:4310", "one named interface"),
            ("[2001:db8::1]:4310", "one named interface"),
        ] {
            let addr: SocketAddr = raw.parse().expect("the fixture address parses");
            assert_eq!(bind_exposure(&addr), expected, "{raw}");
        }
    }
}
pub mod snapshots;
pub mod ssrf;
mod telemetry;
mod threads;
mod tx;
/// The `.1.6.2` static inspection console (embedded at compile time; no API
/// routes, no write path — the page reads the existing GET surfaces).
pub mod ui;
pub mod workflows;

pub use api::{
    api_router, api_router_gated, api_router_with_acquisition, api_router_with_publication_root,
    r5r3rx_enabled, ApiState, ControlApiError, EnrollRequest, EnrollResponse, PRINCIPAL_HEADER,
};
pub use authority::{
    apply_authorized_command, authorize, create_boundary, create_grant, load_authorization_record,
    load_tenant_administrative_effect, record_administrative_effect_in_tx,
    AuthorityTransactionError, AuthorizationOutcome, AuthorizedApplyError, CommandAuthz,
    Delegation, GrantCreateError, GrantRefused,
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
pub use resolvers::sync_gated_entries;
pub use tx::{apply_command, ApplyError, Command, CommandOutcome};
pub use ui::ui_router;
