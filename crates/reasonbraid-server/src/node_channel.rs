//! The node channel, server side (`PHASE-0.3.2`; authenticated by `PHASE-1.2.2`).
//!
//! The control plane owns desired commands and delivery attempts (`ROADMAP.md` §17.1).
//! This module is the server half of the node channel (§9.3's node-channel profile,
//! minimal: HTTP/1 JSON over the loopback dev profile — the authenticated streaming
//! profile arrives with ADR-006's formal record; `.1.2.2` adds the Phase 1
//! authentication layer on top of the transport):
//!
//! - [`NodeChannelState`] — the durable state behind the channel: a per-node inbox with
//!   a monotonic cursor and acknowledgement state (`node_inbox`), deduplicated
//!   receipts of node-emitted events (`node_events`, keyed on the node-assigned id),
//!   the lease/presence store (`node_leases`, `migrations/0009_node_leases.sql`), and
//!   the enrollment identity (`node_keys`).
//! - [`node_router`] — the HTTP surface: `handshake` (the authenticated reconnect
//!   exchange), `events` (node results with original ids), `ack` (cursor
//!   acknowledgement), `poll` (the live delivery path after reconciliation),
//!   `heartbeat` (lease renewal), `presence` (observable online/offline state), and
//!   `enroll` (the `.1.2.1` one-time-token enrollment).
//!
//! # Authentication (`.1.2.2`, backlog 13)
//!
//! The channel is authenticated end to end, on the `.6.1` dev stance (the server IS
//! the dev trust store):
//!
//! - the handshake carries an **HMAC-SHA256 key-proof** over the channel fields with
//!   the node's dev secret (its `node_keys` row from `.1.2.1`); a handshake without a
//!   valid proof is refused `401 unauthorized` before any ledger fact is read;
//! - a successful handshake issues a **lease** with a fresh **fencing token** — the
//!   only token that renews the lease (`heartbeat`) or guards `events`/`ack`/`poll`;
//!   a later handshake rotates the token and fences the old one (stale traffic gets
//!   `401`, never silently accepted);
//! - expiry is a database fact (`lease_expires_at`, 60 s dev TTL) — written AND
//!   compared on the database's own clock (`.4.2.3.1`), so the TTL a node gets is
//!   the TTL the presence surface publishes: a heartbeat only renews a LIVE lease;
//!   once expired, presence shows `offline` and channel traffic is refused until the
//!   node re-proves its key with a new handshake.
//!
//! # Replay and trust
//!
//! Replay is computed from the cursor the node REPORTS: the node is authoritative for
//! what it durably holds (`§17.4` step 2), the server replays everything after it, and
//! the node's journal deduplicates by command id — so a redelivery after a crash
//! between journaling and acknowledgement never doubles anything. A node that reports
//! a cursor AHEAD of this server's ledger is refused: its journal saw commands this
//! server cannot reproduce (a `journal_lost`-class anomaly, `§11.4`).
//!
//! # Reconciliation
//!
//! The handshake turns the node's pending/ambiguous journal entries into two concrete
//! server facts: a `Directive` per ambiguous attempt (adjudicated when the server holds
//! a receipt for the operation's event, otherwise the ambiguity stays bounded and
//! visible), and a `KnownEvent` per pending operation the server already holds (so a
//! lost event acknowledgement can be recovered).

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres};

/// The node channel's wire protocol version. Both sides must agree; a mismatch is a
/// `protocol_incompatible` error, never a silent downgrade. Version 2 (`.1.2.2`) adds
/// the authenticated contract: the handshake key-proof, the fencing token on
/// `events`/`ack`/`poll`, and the `heartbeat`/`presence` endpoints.
pub const CHANNEL_VERSION: u32 = 5;

/// The dev-profile lease TTL: a heartbeat renews a LIVE lease by this much. 60 s
/// gives the demo's 15 s heartbeat cadence a 4× margin; a process that stops
/// heartbeating is visibly `offline` within a minute.
pub const LEASE_TTL: ChronoDuration = ChronoDuration::seconds(60);

/// `LEASE_TTL` as the `double precision` seconds `make_interval(secs => …)`
/// takes, so the two lease writers bind the constant instead of spelling `60`
/// into their SQL — a second copy of a number nothing derives is how the
/// published TTL and the granted one come apart.
fn lease_ttl_seconds() -> f64 {
    LEASE_TTL.num_seconds() as f64
}

/// A node-channel identity is the dev node-id space: a `nod_…` node id OR the
/// `rol_…` agent-role wire id the dev wiring collapses node==role onto (one node,
/// one role — `docs/book/src/two-host-demo.md`). The `.1.2.1` surfaces accepted only
/// `NodeId`; the authenticated handshake looks up the `node_keys` row for whatever id
/// the node reports, so the space must accept both.
pub fn is_valid_node_identity(id: &str) -> bool {
    id.parse::<reasonbraid_core::NodeId>().is_ok()
        || id.parse::<reasonbraid_core::AgentRoleId>().is_ok()
}

// ── Wire contract ──────────────────────────────────────────────────────────────

/// The exact fields the handshake key-proof covers, in canonical (serde field)
/// order. Both sides serialize THIS shape to JSON and HMAC it — the mirrored
/// struct is the canonicalization contract. `key_proof` itself is never inside it.
#[derive(Debug, Serialize)]
struct ProofCoverage<'a> {
    channel_version: u32,
    node_id: &'a str,
    last_acked_cursor: i64,
    pending_operations: &'a [String],
    ambiguous_attempts: &'a [AmbiguousAttempt],
    /// `SIGNOFF-REPAIR.4.2.2`: the one field that makes a captured request
    /// unrepeatable. It is INSIDE the signed coverage, so a replayer cannot
    /// change it without invalidating the signature.
    nonce: &'a str,
}

/// The reconnect exchange (`§17.4` steps 2–3): the node reports its durable resume
/// facts and proves possession of its workload certificate's key (`.1.2.2`); the
/// server replies with the replay, the reconciliation guidance, and a fresh lease
/// (fencing token + expiry).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HandshakeRequest {
    pub channel_version: u32,
    pub node_id: String,
    /// Highest command cursor the node has durably recorded.
    pub last_acked_cursor: i64,
    /// Local operations that have not reached a terminal state.
    pub pending_operations: Vec<String>,
    /// Attempts the node's recovery classified `outcome_unknown`.
    pub ambiguous_attempts: Vec<AmbiguousAttempt>,
    /// The node's workload certificate (hex DER) — the leaf the `.1.2.1`
    /// enrollment issued (or `.1.2.2` rotation refreshed).
    pub cert_der: String,
    /// ECDSA P-256 signature over the canonical coverage (see
    /// [`ProofCoverage`]), hex-encoded, made with the certificate's private
    /// key. A handshake without a valid proof is refused `401 unauthorized`
    /// before any ledger fact is read.
    pub proof_signature: String,
    /// A fresh value the node chooses for THIS request
    /// (`SIGNOFF-REPAIR.4.2.2`). It rides the signed coverage and is consumed
    /// once, so a captured request is refused the second time it is presented.
    pub nonce: String,
}

/// The rotate exchange (`.1.2.2`): the node presents its CURRENT certificate
/// and receives a FRESH key + certificate for the same node id.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RotateRequest {
    pub channel_version: u32,
    pub node_id: String,
    pub cert_der: String,
    pub proof_signature: String,
    /// `SIGNOFF-REPAIR.4.2.2` — see [`HandshakeRequest::nonce`].
    pub nonce: String,
}

/// The exact fields the rotate proof covers, in canonical (serde field) order.
#[derive(Debug, Serialize)]
pub struct RotateCoverage<'a> {
    pub channel_version: u32,
    pub node_id: &'a str,
    pub cert_der: &'a str,
    /// `SIGNOFF-REPAIR.4.2.2`. Without it this coverage is entirely STATIC for a
    /// given node and certificate, and each replay answers with a new private key.
    pub nonce: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotateResponse {
    pub node_id: String,
    pub cert_der: String,
    pub key_der: String,
    pub cert_fingerprint: String,
    pub cert_expires_at: DateTime<Utc>,
}

/// One ambiguous attempt the node reports for reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AmbiguousAttempt {
    pub attempt_id: String,
    pub operation_id: String,
}

/// One inbox row replayed to the node, in cursor order. The decision fields
/// (`.1.5.2`, ADR-008) carry the ADMISSION decision the node caches: a row
/// enqueued before migration 0013 has none — the node treats that as
/// fail-closed (no cached decision, no dispatch).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReplayCommand {
    pub cursor: i64,
    pub command_id: String,
    pub tenant_id: String,
    pub thread_id: String,
    pub payload: Value,
    /// The admitting authorization record id.
    pub authz_ref: Option<String>,
    /// The policy digest the record bound.
    pub policy_digest: Option<String>,
    /// The decision time — the node-side freshness TTL runs from it.
    pub decided_at: Option<DateTime<Utc>>,
    /// The tenant's revocation epoch AT DECISION TIME (a later bump invalidates).
    pub revocation_epoch: Option<i64>,
}

/// Reconciliation guidance for one of the node's ambiguous attempts (`§11.4`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Directive {
    /// The server holds a receipt for this operation's event: the node's ambiguity is
    /// over — mark the attempt `reconciled` with the receipt as the evidence.
    Adjudicated {
        attempt_id: String,
        terminal: String,
        evidence: String,
    },
    /// The server holds no receipt and cannot decide: the attempt stays
    /// `outcome_unknown` until a provider proof or an operator adjudicates it.
    NeedsAdjudication { attempt_id: String, reason: String },
}

/// A server-held receipt for one of the node's pending operations — lets the node mark
/// its locally-emitted event acknowledged after a crash that lost the acknowledgement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct KnownEvent {
    pub operation_id: String,
    pub event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HandshakeResponse {
    pub channel_version: u32,
    /// Highest cursor in this node's inbox ledger.
    pub current_cursor: i64,
    /// Inbox rows with `cursor > reported last_acked_cursor`, in cursor order.
    pub replay: Vec<ReplayCommand>,
    /// Adjudication for each ambiguous attempt the node reported.
    pub directives: Vec<Directive>,
    /// Server receipts covering the node's pending operations.
    pub known_events: Vec<KnownEvent>,
    /// The lease this handshake issued: the fresh fencing token (the only token
    /// that renews the lease or guards `events`/`ack`/`poll`) and its expiry.
    pub fencing_token: String,
    pub lease_expires_at: DateTime<Utc>,
    /// The lease EPOCH this handshake's token was issued under (`.2.2`): every
    /// fenced write carries it, so a stale session's writes and renewals are
    /// refused even when they race a newer handshake.
    pub lease_epoch: i64,
    /// The tenant's CURRENT revocation epoch (`.1.5.2`, ADR-008): the node
    /// stores it and evaluates every cached admission decision against it at
    /// the dispatch boundary.
    pub revocation_epoch: i64,
    /// The server's own clock at the moment it answered
    /// (`SIGNOFF-REPAIR.3.4.3.1.2`). The node measures its offset against this
    /// and evaluates the server instants it was sent in the server's terms.
    pub server_time: DateTime<Utc>,
}

/// A node-emitted event (a result, with its ORIGINAL id — `§17.4` step 5).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventSubmission {
    pub channel_version: u32,
    pub node_id: String,
    pub event_id: String,
    pub operation_id: String,
    pub payload: Value,
    /// The fencing token this node's latest handshake issued.
    pub fencing_token: String,
    /// The lease epoch the fencing token was issued under (`.2.2`): a write
    /// from a fenced epoch is refused even if its token check raced past a
    /// newer handshake.
    pub lease_epoch: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventReceipt {
    pub channel_version: u32,
    /// `false` when the server already holds this event id (a redelivery).
    pub accepted: bool,
}

/// The node acknowledges that it durably holds commands up to `ack_cursor`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AckRequest {
    pub channel_version: u32,
    pub node_id: String,
    pub ack_cursor: i64,
    /// The fencing token this node's latest handshake issued.
    pub fencing_token: String,
    /// The lease epoch the fencing token was issued under (`.2.2`).
    pub lease_epoch: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AckResponse {
    pub channel_version: u32,
    /// Inbox rows this acknowledgement marked (idempotent: 0 on a re-ack).
    pub acknowledged: i64,
}

/// The live delivery tail request (`.1.2.2`: `poll` is a POST — the fencing token
/// is a credential and never rides a query string).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollRequest {
    pub channel_version: u32,
    pub node_id: String,
    pub after_cursor: i64,
    /// The fencing token this node's latest handshake issued.
    pub fencing_token: String,
    /// The lease epoch the fencing token was issued under (`.2.2`).
    pub lease_epoch: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PollResponse {
    pub channel_version: u32,
    pub current_cursor: i64,
    pub commands: Vec<ReplayCommand>,
    /// The tenant's CURRENT revocation epoch (`.1.5.2`, ADR-008) — the node's
    /// freshness reference for every cached admission decision.
    pub revocation_epoch: i64,
    /// The server's own clock at the moment it answered
    /// (`SIGNOFF-REPAIR.3.4.3.1.2`). The node measures its offset against this
    /// and evaluates the server instants it was sent in the server's terms,
    /// instead of comparing them to its own clock and calling the difference
    /// staleness.
    pub server_time: DateTime<Utc>,
}

/// The lease renewal (`backlog 13`): a heartbeat extends a LIVE lease — the
/// fencing token must be the one the latest handshake issued. An expired or
/// fenced lease is refused; only a fresh handshake (a new key-proof) restores it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HeartbeatRequest {
    pub channel_version: u32,
    pub node_id: String,
    pub fencing_token: String,
    /// The lease epoch the fencing token was issued under (`.2.2`): the
    /// renewal only lands if the row STILL carries that epoch — a heartbeat
    /// racing a newer handshake loses instead of extending the new session.
    pub lease_epoch: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HeartbeatResponse {
    pub channel_version: u32,
    /// Echoed: the fencing token the renewal was granted to (unchanged — the
    /// handshake is the only rotation point).
    pub fencing_token: String,
    pub lease_expires_at: DateTime<Utc>,
}

/// One node's observable presence (`GET /v1/nodes/presence`): `online` is DERIVED
/// from the lease expiry clock, never a stored flag, so a crashed process cannot
/// leave a stale `online` row behind. The fencing token is never exposed here.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PresenceResponse {
    pub node_id: String,
    pub online: bool,
    /// The node.s workload certificate was revoked (`.1.3.1`): presence
    /// reads `suspended` whatever the lease says — a revoked node cannot
    /// re-handshake.
    pub suspended: bool,
    /// The derived six-state presence (`.3.2.1`): `available` | `offline` |
    /// `suspended` | `draining` | `unknown` (`busy` waits for the `.4`
    /// capacity accounting).
    pub state: String,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub lease_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct PresenceParams {
    pub node_id: String,
}

// ── API error ──────────────────────────────────────────────────────────────────

/// A channel API error: a stable machine-readable code (the §9.8 names) and a safe
/// message. No secrets, no policy internals, no cross-tenant existence leaks.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    fn protocol_incompatible(got: u32) -> Self {
        ApiError {
            status: StatusCode::BAD_REQUEST,
            code: "protocol_incompatible",
            message: format!(
                "channel protocol version {got} is not supported (expected {CHANNEL_VERSION})"
            ),
        }
    }

    fn cursor_ahead(reported: i64, server: i64) -> Self {
        ApiError {
            status: StatusCode::CONFLICT,
            code: "version_conflict",
            message: format!(
                "reported cursor {reported} is ahead of this server's ledger ({server}): \
                 the node's journal saw commands this server cannot reproduce"
            ),
        }
    }

    fn bad_request(message: String) -> Self {
        ApiError {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_command",
            message,
        }
    }

    /// A handshake whose key-proof did not verify (or whose node has no key — the
    /// same refusal: no existence leak). `401 unauthorized`, like the `.1.2.1`
    /// enrollment refusals: the node-channel surface's bad-credential status.
    fn proof_refused() -> Self {
        ApiError {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: "the handshake certificate proof was refused".to_string(),
        }
    }

    /// Channel traffic presented a fencing token that is not the latest lease's —
    /// either never issued, fenced by a newer handshake, or for another node.
    fn fencing_refused() -> Self {
        ApiError {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: "the fencing token was refused — re-handshake".to_string(),
        }
    }

    /// The lease is no longer live: presence is `offline` and only a fresh
    /// handshake (a new key-proof) restores the channel.
    fn lease_expired() -> Self {
        ApiError {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: "the lease has expired — re-handshake".to_string(),
        }
    }

    /// The node holds no usable workload certificate — revoked (`.1.3`) or
    /// lapsed — so its session may not be EXTENDED (`SIGNOFF-REPAIR.4.1.3`).
    /// Distinct from `fencing_refused`: nothing is wrong with the node's token,
    /// and re-handshaking will not help until it holds a usable credential.
    fn credential_refused() -> Self {
        ApiError {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: "the node's workload certificate is no longer usable — \
                      the lease may not be renewed"
                .to_string(),
        }
    }

    /// `SIGNOFF-REPAIR.4.1.6`: the host claim cannot be a certificate SAN. The
    /// claim itself is echoed by the caller who sent it, so naming it leaks
    /// nothing, and the node needs it to know which field to correct.
    fn host_claim_refused(refusal: &crate::ca::HostClaimRefused) -> Self {
        ApiError {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_command",
            message: format!("{refusal}"),
        }
    }

    /// No `x-reasonbraid-principal` header, or one that is not a typed principal
    /// id. `SIGNOFF-REPAIR.3.5.5` — the presence read had no caller at all.
    fn unauthenticated() -> Self {
        ApiError {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthenticated",
            message: format!(
                "missing or malformed `{}` header (dev profile: hpr_… | rol_…)",
                crate::api::PRINCIPAL_HEADER
            ),
        }
    }

    /// A well-formed principal that belongs to no tenant reads nothing. Distinct
    /// from `unauthenticated`: the caller identified itself and the answer is
    /// still no.
    fn unenrolled() -> Self {
        ApiError {
            status: StatusCode::FORBIDDEN,
            code: "unauthorized",
            message: "an unenrolled principal reads no presence".to_string(),
        }
    }

    fn unknown_node(node_id: &str) -> Self {
        ApiError {
            status: StatusCode::NOT_FOUND,
            code: "unknown_node",
            message: format!("no enrolled node `{node_id}`"),
        }
    }

    fn internal() -> Self {
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "dependency_unavailable",
            message: "internal server error".to_string(),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({ "code": self.code, "message": self.message })),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        // The driver error detail never crosses the wire; it belongs in the server log
        // (eprintln until `tracing` lands with the observability work).
        eprintln!("node channel: database error: {e}");
        ApiError::internal()
    }
}

// ── Durable state ──────────────────────────────────────────────────────────────

/// The durable server-side state behind the channel (`node_inbox` + `node_events`,
/// `migrations/0003_node_inbox.sql`).
#[derive(Debug, Clone)]
pub struct NodeChannelState {
    pool: PgPool,
    /// The workload-identity CA (ADR-007): the enrollment path signs the node's
    /// short-lived leaf with it; the `.1.2.2` handshake chains to it.
    ca: Arc<crate::ca::ServerCa>,
}

impl NodeChannelState {
    pub fn new(pool: PgPool, ca: Arc<crate::ca::ServerCa>) -> Self {
        Self { pool, ca }
    }

    /// Append a command to a node's inbox ledger; returns its assigned per-node cursor
    /// (monotonic, starting at 1). Concurrent enqueues to the SAME node collide on the
    /// `(node_id, cursor)` primary key — the dev profile is single-writer; the collision
    /// is a conflict error, never a silent overwrite.
    ///
    /// Plain channel traffic carries NO admission decision (the decision columns are
    /// NULL — a node would refuse to dispatch it, fail-closed); dispatched WORK items
    /// ride [`enqueue_in_tx`] with the full `.1.5.2` metadata.
    pub async fn enqueue(
        &self,
        node_id: &str,
        command_id: &str,
        tenant_id: &str,
        thread_id: &str,
        payload: &Value,
    ) -> Result<i64, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query_scalar(
            "INSERT INTO node_inbox (node_id, cursor, command_id, tenant_id, thread_id, payload) \
             VALUES ($1, \
                     (SELECT COALESCE(MAX(cursor), 0) + 1 FROM node_inbox WHERE node_id = $1), \
                     $2, $3, $4, $5) \
             RETURNING cursor",
        )
        .bind(node_id)
        .bind(command_id)
        .bind(tenant_id)
        .bind(thread_id)
        .bind(payload)
        .fetch_one(&mut *conn)
        .await
    }

    /// The inbox row for one command, if this node's ledger holds it — the `.6.2`
    /// lookup that turns a node-emitted `work_result` back into its tenant/thread
    /// scope and work kind.
    pub async fn load_command(
        &self,
        node_id: &str,
        command_id: &str,
    ) -> Result<Option<(String, String, Value)>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        load_command_in_tx(&mut *conn, node_id, command_id).await
    }

    /// The highest cursor in a node's inbox ledger (0 when the node has none).
    pub async fn current_cursor(&self, node_id: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COALESCE(MAX(cursor), 0) FROM node_inbox WHERE node_id = $1")
            .bind(node_id)
            .fetch_one(&self.pool)
            .await
    }

    /// The tenant's CURRENT revocation epoch for this node (`.1.5.2`, ADR-008):
    /// the freshness reference the node evaluates every cached admission
    /// decision against. Rides the node's enrollment tenant (the `nodes` row —
    /// a fenced node is always enrolled, so the row exists).
    /// The tenant's current revocation epoch AND the server's own clock, read in
    /// the SAME query (`SIGNOFF-REPAIR.3.4.3.1.2`).
    ///
    /// The clock rides this query rather than its own so a poll — which happens
    /// every few seconds — gains no extra round trip. It is the DATABASE clock
    /// deliberately: `decided_at`, the one server instant the node actually
    /// reads, is `clock_timestamp()` sampled inside the authorizing
    /// transaction, so a node correcting against this value corrects against
    /// the clock that produced the instant it is comparing.
    pub async fn epoch_and_server_time(
        &self,
        node_id: &str,
    ) -> Result<(i64, DateTime<Utc>), sqlx::Error> {
        sqlx::query_as(
            "SELECT t.revocation_epoch, clock_timestamp() \
             FROM tenants t JOIN nodes n ON n.tenant_id = t.tenant_id \
             WHERE n.node_id = $1",
        )
        .bind(node_id)
        .fetch_one(&self.pool)
        .await
    }

    /// The inbox rows after `after_cursor`, in cursor order — the replay for a node
    /// that reports holding up to `after_cursor`. Quarantined rows are ALWAYS
    /// filtered (`.1.2.3`): a quarantined command is never re-delivered, whatever
    /// cursor the node reports (a node that never saw it simply has a hole in its
    /// ledger — cursor acknowledgement still marks it terminal).
    ///
    /// # Delivery requires a usable credential (`SIGNOFF-REPAIR.4.1.3.1`)
    ///
    /// `.4.1.3` bounded a revoked node's session to the remaining lease but left
    /// it receiving NEW work for that tail: the delivery read the inbox and
    /// nothing about whether the node was still trusted. A revocation the
    /// operator has just applied must not be followed by the server handing that
    /// node more work.
    ///
    /// So delivery asks the question `renew_lease` asks, asked the same way:
    /// does this node hold a certificate that is neither revoked nor expired?
    /// While it does not, the tail is EMPTY — and the rows are withheld, never
    /// dropped. That distinction is the whole reason the check sits on this rung
    /// rather than on the enqueue: the replacement ritual re-enrolls the SAME
    /// node id with a fresh certificate, and the withheld tail then replays to
    /// it (`node_replacement`'s durability leg). A filter at the dispatch would
    /// have destroyed that work instead.
    ///
    /// This is a filter, not a refusal, and that is `.1.3.1`'s decision held
    /// intact: `poll` still admits on its pure fencing check, `ack` and `events`
    /// are untouched, so the session in flight still finishes and reports the
    /// work it already holds. It simply gets no more. The same predicate rides
    /// the handshake's replay, where it is vacuous — `verify_cert_proof` has
    /// already required a usable row for this node id before the tail is read.
    pub async fn replay(
        &self,
        node_id: &str,
        after_cursor: i64,
    ) -> Result<Vec<ReplayCommand>, sqlx::Error> {
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                String,
                String,
                String,
                Value,
                Option<String>,
                Option<String>,
                Option<DateTime<Utc>>,
                Option<i64>,
            ),
        >(
            "SELECT cursor, command_id, tenant_id, thread_id, payload, authz_ref, \
                    policy_digest, decided_at, revocation_epoch \
             FROM node_inbox \
             WHERE node_id = $1 AND cursor > $2 AND quarantined_at IS NULL \
               AND EXISTS (SELECT 1 FROM node_certificates c \
                           WHERE c.node_id = $1 \
                             AND c.revoked_at IS NULL AND c.expires_at > now()) \
               AND NOT EXISTS ( \
                   SELECT 1 FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                   WHERE v.role_id = $1 AND v.version = p.current_version \
                     AND (v.profile->'availability'->>'concurrency')::bigint = 0) \
             ORDER BY cursor",
        )
        .bind(node_id)
        .bind(after_cursor)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    cursor,
                    command_id,
                    tenant_id,
                    thread_id,
                    payload,
                    authz_ref,
                    policy_digest,
                    decided_at,
                    revocation_epoch,
                )| {
                    ReplayCommand {
                        cursor,
                        command_id,
                        tenant_id,
                        thread_id,
                        payload,
                        authz_ref,
                        policy_digest,
                        decided_at,
                        revocation_epoch,
                    }
                },
            )
            .collect())
    }

    /// `current_cursor`, inside the caller's transaction (`SIGNOFF-REPAIR.4.2.4`),
    /// so the bound an acknowledgement is checked against and the rows it marks
    /// come from ONE snapshot.
    pub async fn current_cursor_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        node_id: &str,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COALESCE(MAX(cursor), 0) FROM node_inbox WHERE node_id = $1")
            .bind(node_id)
            .fetch_one(&mut **tx)
            .await
    }

    /// `acknowledge`, inside the caller's transaction (`SIGNOFF-REPAIR.4.2.4`):
    /// the same statement, committing with the fencing re-verification that
    /// admitted it rather than on its own connection.
    pub async fn acknowledge_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        node_id: &str,
        ack_cursor: i64,
        now: DateTime<Utc>,
    ) -> Result<i64, sqlx::Error> {
        Ok(sqlx::query(
            "UPDATE node_inbox SET acknowledged_at = $3 \
             WHERE node_id = $1 AND cursor <= $2 AND acknowledged_at IS NULL",
        )
        .bind(node_id)
        .bind(ack_cursor)
        .bind(now)
        .execute(&mut **tx)
        .await?
        .rows_affected() as i64)
    }

    /// Mark every inbox row up to `ack_cursor` acknowledged (idempotent). Returns the
    /// number of rows this call marked.
    pub async fn acknowledge(
        &self,
        node_id: &str,
        ack_cursor: i64,
        now: DateTime<Utc>,
    ) -> Result<i64, sqlx::Error> {
        Ok(sqlx::query(
            "UPDATE node_inbox SET acknowledged_at = $3 \
             WHERE node_id = $1 AND cursor <= $2 AND acknowledged_at IS NULL",
        )
        .bind(node_id)
        .bind(ack_cursor)
        .bind(now)
        .execute(&self.pool)
        .await?
        .rows_affected() as i64)
    }

    /// Record a node event receipt. The event_id primary key is the dedupe key, so a
    /// re-emitted event (original id, `§17.4` step 5) returns `accepted = false` and
    /// writes nothing — one receipt per event, ever.
    pub async fn record_event(
        &self,
        node_id: &str,
        event_id: &str,
        operation_id: &str,
        payload: &Value,
        now: DateTime<Utc>,
    ) -> Result<bool, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        record_event_in_tx(&mut *conn, node_id, event_id, operation_id, payload, now).await
    }

    /// The event id the server holds for an operation, if any — the reconciliation
    /// lookup the handshake performs.
    pub async fn event_id_for_operation(
        &self,
        operation_id: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT event_id FROM node_events WHERE operation_id = $1 \
             ORDER BY received_at LIMIT 1",
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Consume a proof nonce, in ONE statement (`SIGNOFF-REPAIR.4.2.2`).
    ///
    /// `ON CONFLICT DO NOTHING RETURNING` is the whole decision: the first
    /// presentation inserts and returns a row, every later presentation of the
    /// same bytes returns none. Nothing is read and then written, so there is no
    /// window between deciding and recording — the shape `.4.1.1`'s issuance
    /// uses for the same reason, and the reason a raised unique violation is not
    /// used instead (`docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md`:
    /// a constraint error aborts the transaction, and this runs inside one).
    ///
    /// Retention is pruned here rather than swept: a replay is useless once the
    /// certificate it presents has expired, and the certificate's own TTL is ten
    /// minutes, so an hour is a wide margin. The delete is scoped to this node,
    /// so it touches a handful of rows on a path that runs once per reconnect or
    /// rotation — never per poll.
    ///
    /// Takes the caller's executor rather than the pool, so the rotation can
    /// consume inside the transaction that already holds its node row — a
    /// rotation that then fails burns no nonce, and the consume cannot contend
    /// with the lock its own transaction is holding.
    async fn consume_proof_nonce<'e, E>(
        &self,
        mut executor: E,
        node_id: &str,
        nonce: &str,
    ) -> Result<(), ApiError>
    where
        E: std::ops::DerefMut,
        for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
    {
        if nonce.is_empty() || nonce.len() > 200 {
            return Err(ApiError::proof_refused());
        }
        sqlx::query(
            "DELETE FROM node_proof_nonces \
             WHERE node_id = $1 AND seen_at < now() - interval '1 hour'",
        )
        .bind(node_id)
        .execute(&mut *executor)
        .await?;
        let consumed: Option<String> = sqlx::query_scalar(
            "INSERT INTO node_proof_nonces (nonce, node_id) VALUES ($1, $2) \
             ON CONFLICT DO NOTHING RETURNING nonce",
        )
        .bind(nonce)
        .bind(node_id)
        .fetch_optional(&mut *executor)
        .await?;
        match consumed {
            Some(_) => Ok(()),
            None => {
                crate::telemetry::metrics().incr("handshake_refusals");
                Err(ApiError::proof_refused())
            }
        }
    }

    /// Verify the handshake's certificate proof: the presented leaf must chain
    /// to this deployment's CA within its validity window (webpki), its
    /// fingerprint must be registered for THIS node and neither revoked nor
    /// expired (the row check — `.1.3` writes `revoked_at`), and the signature
    /// must verify over the canonical channel fields against the leaf's key.
    /// Everything runs BEFORE any ledger fact (cursor, replay, receipts) is
    /// read; a node with no certificate and a node with a bad proof fail
    /// IDENTICALLY (no existence leak).
    pub async fn verify_cert_proof(&self, req: &HandshakeRequest) -> Result<(), ApiError> {
        let refused = || {
            crate::telemetry::metrics().incr("handshake_refusals");
            ApiError::proof_refused()
        };
        let cert_der = decode_hex(&req.cert_der).ok_or_else(refused)?;
        let signature = decode_hex(&req.proof_signature).ok_or_else(refused)?;
        crate::ca::verify_leaf_chain(&self.ca, &cert_der).map_err(|_| refused())?;
        let fingerprint = crate::ca::cert_fingerprint(&cert_der);
        let row: Option<(String, bool)> = sqlx::query_as(
            "SELECT node_id, (revoked_at IS NOT NULL OR expires_at <= now()) \
             FROM node_certificates WHERE cert_fingerprint = $1",
        )
        .bind(&fingerprint)
        .fetch_optional(&self.pool)
        .await?;
        let Some((node_id, unusable)) = row else {
            return Err(ApiError::proof_refused());
        };
        if node_id != req.node_id || unusable {
            return Err(ApiError::proof_refused());
        }
        let coverage = ProofCoverage {
            channel_version: req.channel_version,
            node_id: &req.node_id,
            last_acked_cursor: req.last_acked_cursor,
            pending_operations: &req.pending_operations,
            ambiguous_attempts: &req.ambiguous_attempts,
            nonce: &req.nonce,
        };
        // The canonical form is the mirrored struct's JSON: field order and
        // encoding are fixed by the struct shape on both sides.
        let canonical = serde_json::to_vec(&coverage)
            .map_err(|e| ApiError::bad_request(format!("unencodable handshake fields: {e}")))?;
        let spki = crate::ca::extract_point(&cert_der).map_err(|_| ApiError::proof_refused())?;
        crate::ca::verify_signature(&spki, &canonical, &signature)
            .map_err(|_| ApiError::proof_refused())
    }

    /// The rotate proof check (the same ladder over the rotate coverage), run
    /// INSIDE the caller's transaction.
    ///
    /// # Why this is not a pool read (`SIGNOFF-REPAIR.4.2.1`)
    ///
    /// The rotation's DECISION (this certificate is live) and its EFFECT (a new
    /// active certificate) used to be separate statements on the pool, so a
    /// revocation landing between them left the node holding a usable
    /// certificate it had just been denied. Measured: with the rotation stalled
    /// after its proof check, a complete revocation ran, and the node finished
    /// with one live certificate.
    ///
    /// The caller now locks the node row first — the same row, in the same mode,
    /// that the revocation locks before it changes anything — so the two
    /// serialize. A rotation that wins the row commits a certificate the
    /// revocation's later `UPDATE` then sees and revokes; one that loses it
    /// re-reads the certificate HERE, after the revocation committed, and is
    /// refused. This is `verify_fencing_in_tx`'s shape applied to the same
    /// check-vs-commit window (`.2.2`).
    pub async fn verify_rotate_proof_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        req: &RotateRequest,
    ) -> Result<(), ApiError> {
        let cert_der = decode_hex(&req.cert_der).ok_or_else(ApiError::proof_refused)?;
        let signature = decode_hex(&req.proof_signature).ok_or_else(ApiError::proof_refused)?;
        crate::ca::verify_leaf_chain(&self.ca, &cert_der).map_err(|_| ApiError::proof_refused())?;
        let fingerprint = crate::ca::cert_fingerprint(&cert_der);
        let row: Option<(String, bool)> = sqlx::query_as(
            "SELECT node_id, (revoked_at IS NOT NULL OR expires_at <= now()) \
             FROM node_certificates WHERE cert_fingerprint = $1",
        )
        .bind(&fingerprint)
        .fetch_optional(&mut **tx)
        .await?;
        let Some((node_id, unusable)) = row else {
            return Err(ApiError::proof_refused());
        };
        if node_id != req.node_id || unusable {
            return Err(ApiError::proof_refused());
        }
        let coverage = RotateCoverage {
            channel_version: req.channel_version,
            node_id: &req.node_id,
            cert_der: &req.cert_der,
            nonce: &req.nonce,
        };
        let canonical = serde_json::to_vec(&coverage)
            .map_err(|e| ApiError::bad_request(format!("unencodable rotate fields: {e}")))?;
        let spki = crate::ca::extract_point(&cert_der).map_err(|_| ApiError::proof_refused())?;
        crate::ca::verify_signature(&spki, &canonical, &signature)
            .map_err(|_| ApiError::proof_refused())
    }

    /// Verify a fencing token against the node's lease: the token must be the
    /// latest handshake's AND the lease must still be live. A missing/mismatched
    /// token and an expired lease are refused differently (the node can tell a
    /// fenced credential from a lapsed one), but neither reads any ledger fact.
    ///
    /// The liveness comparison is the DATABASE's (`SIGNOFF-REPAIR.4.2.3.1`): the
    /// column is written by the database, `node_presence.online` is derived from
    /// it on that clock, and this statement is its own transaction, so `now()`
    /// IS the database clock here. Comparing a returned instant against the
    /// process's `Utc::now()` — which is what this did — would have made the
    /// admission check and the published presence fact disagree by exactly the
    /// process↔database skew.
    pub async fn verify_fencing(
        &self,
        node_id: &str,
        token: &str,
        lease_epoch: i64,
    ) -> Result<(), ApiError> {
        let live: Option<bool> = sqlx::query_scalar(
            "SELECT lease_expires_at > now() FROM node_leases \
             WHERE node_id = $1 AND fencing_token = $2 AND lease_epoch = $3",
        )
        .bind(node_id)
        .bind(token)
        .bind(lease_epoch)
        .fetch_optional(&self.pool)
        .await?;
        match live {
            None => {
                crate::telemetry::metrics().incr("lease_refusals");
                Err(ApiError::fencing_refused())
            }
            Some(false) => {
                crate::telemetry::metrics().incr("lease_refusals");
                Err(ApiError::lease_expired())
            }
            Some(true) => Ok(()),
        }
    }

    /// The check-vs-commit re-verification (`.2.2`): the same fencing facts,
    /// checked INSIDE the caller's transaction with the lease row locked — a
    /// handshake that rotates the lease between an admission check and the
    /// apply is observed here (the row no longer matches), so a fenced session
    /// can never ride a stale admission into the write.
    ///
    /// ⛔ **`clock_timestamp()`, NOT `now()`** (`SIGNOFF-REPAIR.4.2.3.1`). The
    /// sibling above may use `now()` because its statement IS its transaction.
    /// Here the caller's transaction is already open and may have waited in it:
    /// `events` takes the tenant guard's shared mode first, which blocks behind
    /// a revocation holding the exclusive mode for as long as that takes. `now()`
    /// is `transaction_timestamp()` — the instant that transaction BEGAN — so
    /// reading it here would admit a lease that lapsed during the wait, which is
    /// `.4.2.3`'s revival defect re-entering through the clock instead of
    /// through the write. `clock_timestamp()` advances inside the transaction
    /// and is evaluated after the row lock is taken.
    pub async fn verify_fencing_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        node_id: &str,
        token: &str,
        lease_epoch: i64,
    ) -> Result<(), ApiError> {
        let live: Option<bool> = sqlx::query_scalar(
            "SELECT lease_expires_at > clock_timestamp() FROM node_leases \
             WHERE node_id = $1 AND fencing_token = $2 AND lease_epoch = $3 \
             FOR UPDATE",
        )
        .bind(node_id)
        .bind(token)
        .bind(lease_epoch)
        .fetch_optional(&mut **tx)
        .await?;
        match live {
            None => {
                crate::telemetry::metrics().incr("lease_refusals");
                Err(ApiError::fencing_refused())
            }
            Some(false) => {
                crate::telemetry::metrics().incr("lease_refusals");
                Err(ApiError::lease_expired())
            }
            Some(true) => Ok(()),
        }
    }

    /// Issue (or rotate) the node's lease: a FRESH fencing token every handshake,
    /// expiry `LEASE_TTL` out, and a bumped lease EPOCH (`.2.2`). The old token
    /// is fenced by the rotation — a stale process's heartbeats/events stop being
    /// accepted the moment a newer handshake lands, and its epoch-bound renewals
    /// match no row. The token is generated IN PostgreSQL (`gen_random_uuid()`)
    /// in the same statement that writes the row — 128 bits of server entropy,
    /// never client-chosen.
    ///
    /// # The expiry is the DATABASE's (`SIGNOFF-REPAIR.4.2.3.1`)
    ///
    /// `lease_expires_at` is computed `now() + LEASE_TTL` in the statement, not
    /// `Utc::now() + LEASE_TTL` in this process, and the statement RETURNS what
    /// it wrote. Every reader of the column is a SQL comparison against the
    /// database clock — `node_presence.online`, the renewal's own `WHERE`, both
    /// admission checks — so producing it here in process terms made the lease's
    /// real duration `LEASE_TTL` plus the process↔database skew, and the
    /// published 60 s a nominal figure no reader actually got.
    ///
    /// ⚠️ `now` is still this server's OWN observation instant and is still
    /// written to `last_seen_at`/`issued_at`, which are records rather than
    /// predicates — nothing in `crates` or `migrations` compares them. Keeping
    /// them here is also what leaves a fixture able to drive a process clock
    /// that disagrees with the database's on a single host, which is the only
    /// way the property above can be falsified at all.
    pub async fn issue_lease(
        &self,
        node_id: &str,
        now: DateTime<Utc>,
    ) -> Result<(String, i64, DateTime<Utc>), sqlx::Error> {
        let (token, epoch, expires): (String, i64, DateTime<Utc>) = sqlx::query_as(
            "INSERT INTO node_leases (node_id, fencing_token, lease_expires_at, last_seen_at, \
                                      issued_at, lease_epoch) \
             VALUES ($1, 'fnc_' || gen_random_uuid()::text, \
                     now() + make_interval(secs => $3), $2, $2, 1) \
             ON CONFLICT (node_id) DO UPDATE SET \
               fencing_token = EXCLUDED.fencing_token, \
               lease_expires_at = EXCLUDED.lease_expires_at, \
               last_seen_at = EXCLUDED.last_seen_at, \
               issued_at = EXCLUDED.issued_at, \
               lease_epoch = node_leases.lease_epoch + 1 \
             RETURNING fencing_token, lease_epoch, lease_expires_at",
        )
        .bind(node_id)
        .bind(now)
        .bind(lease_ttl_seconds())
        .fetch_one(&self.pool)
        .await?;
        Ok((token, epoch, expires))
    }

    /// Renew a LIVE lease (the caller already verified the fencing token): push
    /// `lease_expires_at` and `last_seen_at` forward. The epoch the caller saw
    /// rides the WHERE (`.2.2`): a renewal whose handshake was superseded
    /// between the check and this write matches no row and is refused — a
    /// stale heartbeat can never extend the session that fenced it.
    ///
    /// # A renewal also requires a usable credential (`SIGNOFF-REPAIR.4.1.3`)
    ///
    /// `.1.3.1` decided that revoking a node gates its RE-ENTRY and does not cut
    /// the running session. Measured, re-entry was never required of a node that
    /// simply never stopped: `heartbeat` read no ledger fact, so a revoked node
    /// renewed its lease every `LEASE_TTL` for ever, never needed the handshake
    /// that would have refused it, and kept polling, acking and emitting events.
    /// Revocation of a running node was permanently ineffective.
    ///
    /// So the credential check goes HERE and deliberately not in `verify_fencing`.
    /// Poll, ack and events keep their pure fencing check, which is exactly the
    /// bounded tail `.1.3.1` chose: the session in flight is not cut, it simply
    /// stops being extended, and it lapses within `LEASE_TTL`. Folding the
    /// condition into the UPDATE rather than checking first leaves no window
    /// between the decision and the write.
    ///
    /// The certificate's liveness is read on the DATABASE clock — the column is
    /// written by the database, and `verify_cert_proof` asks the same question
    /// the same way (`.3.4.3`'s lesson about comparing instants across clocks).
    ///
    /// # A renewal also requires a lease that is still LIVE (`SIGNOFF-REPAIR.4.2.3`)
    ///
    /// The same rule, applied to the last condition the pre-check still owned
    /// alone. `verify_fencing` refuses an expired lease, but it is a separate
    /// statement: a renewal whose check passed and whose write then waited —
    /// behind a row lock, a saturated pool or the scheduler — used to write a
    /// fresh expiry onto a lease that had already lapsed, reviving a node whose
    /// session was over. Reproduced by driving the real route with the write
    /// stalled at the lease row, which is `.4.2.1`'s shape applied to the gap
    /// between these two statements.
    ///
    /// The expiry is compared on the DATABASE clock, like the certificate
    /// predicate beside it. That is deliberate on two counts: one statement must
    /// not mix two clocks, and `node_presence` — the product's published answer
    /// to "is this node online?" — is `lease_expires_at > now()` on the same
    /// clock. The write and the fact the operator reads agree by construction,
    /// so a renewal can never succeed for a node the API simultaneously reports
    /// `offline`. ⚠️ When this was written `verify_fencing` still compared
    /// against the process's `Utc::now()`; `.4.2.3.1` moved it, so the column
    /// now has ONE clock rather than agreeing readers and a disagreeing one.
    ///
    /// # The granted expiry is the DATABASE's too (`SIGNOFF-REPAIR.4.2.3.1`)
    ///
    /// The last cross-clock step this statement had. `lease_expires_at` used to
    /// be written `Utc::now() + LEASE_TTL` while every condition beside it —
    /// the lease's own liveness, the certificate's — was compared against
    /// `now()`, so a heartbeat from a process whose clock ran ahead extended the
    /// session by `LEASE_TTL` plus the skew. It is now `now() + LEASE_TTL`, and
    /// the `now()` that grants the new expiry is the SAME instant as the `now()`
    /// that checked the old one: `now()` is `transaction_timestamp()` and this
    /// statement is its own transaction, so the check and the grant share one
    /// reading of one clock rather than merely agreeing about which clock.
    pub async fn renew_lease(
        &self,
        node_id: &str,
        lease_epoch: i64,
        now: DateTime<Utc>,
    ) -> Result<DateTime<Utc>, sqlx::Error> {
        let renewed: Option<DateTime<Utc>> = sqlx::query_scalar(
            "UPDATE node_leases \
             SET lease_expires_at = now() + make_interval(secs => $3), last_seen_at = $2 \
             WHERE node_id = $1 AND lease_epoch = $4 \
               AND node_leases.lease_expires_at > now() \
               AND EXISTS (SELECT 1 FROM node_certificates c \
                           WHERE c.node_id = node_leases.node_id \
                             AND c.revoked_at IS NULL AND c.expires_at > now()) \
             RETURNING lease_expires_at",
        )
        .bind(node_id)
        .bind(now)
        .bind(lease_ttl_seconds())
        .bind(lease_epoch)
        .fetch_optional(&self.pool)
        .await?;
        match renewed {
            Some(expiry) => Ok(expiry),
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    /// Why a renewal matched no row. The DECISION was already made by the
    /// UPDATE — this only chooses the wording, so a node can tell a fenced
    /// session (re-handshake and continue) from a withdrawn credential
    /// (re-handshaking will not help). A read that itself fails falls back to
    /// the fencing wording: the refusal is right either way, only the
    /// explanation would be a guess.
    ///
    /// `.4.2.3` added a THIRD way to match no row — a lease that lapsed before
    /// the write landed — and it is named here so the wire answer does not
    /// depend on the timing: a heartbeat arriving just after the expiry is
    /// refused by `verify_fencing` as `lease_expired`, and one whose write is
    /// overtaken by the expiry must say the same thing rather than blame a
    /// token that is perfectly good. Both facts are read in ONE statement, so
    /// the two halves of the answer come from one snapshot.
    ///
    /// Precedence follows the node's remedy, most severe first: a withdrawn
    /// credential cannot be fixed by re-handshaking, while a lapsed lease and a
    /// fenced epoch both can.
    async fn classify_renewal_refusal(&self, node_id: &str, lease_epoch: i64) -> ApiError {
        let facts: Result<(bool, bool), sqlx::Error> = sqlx::query_as(
            "SELECT EXISTS (SELECT 1 FROM node_certificates \
                            WHERE node_id = $1 AND revoked_at IS NULL \
                              AND expires_at > now()) AS usable, \
                    EXISTS (SELECT 1 FROM node_leases \
                            WHERE node_id = $1 AND lease_epoch = $2 \
                              AND lease_expires_at <= now()) AS lapsed",
        )
        .bind(node_id)
        .bind(lease_epoch)
        .fetch_one(&self.pool)
        .await;
        match facts {
            Ok((false, _)) => {
                crate::telemetry::metrics().incr("lease_refusals");
                ApiError::credential_refused()
            }
            Ok((true, true)) => {
                crate::telemetry::metrics().incr("lease_refusals");
                ApiError::lease_expired()
            }
            _ => ApiError::fencing_refused(),
        }
    }

    /// One node's observable presence (`node_presence`, migration 0009). `None`
    /// when the node is not enrolled.
    /// `SIGNOFF-REPAIR.3.5.5`: the tenant is a REQUIRED argument, not an option.
    /// Making it a parameter rather than a filter applied by the caller means a
    /// future caller cannot forget it — the type system asks the question.
    pub async fn presence(
        &self,
        node_id: &str,
        tenant_id: &str,
    ) -> Result<Option<PresenceResponse>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct PresenceRow {
            online: bool,
            suspended: bool,
            last_seen_at: Option<DateTime<Utc>>,
            lease_expires_at: Option<DateTime<Utc>>,
            concurrency: Option<i64>,
        }
        let row: Option<PresenceRow> = sqlx::query_as(
            "SELECT online, suspended, last_seen_at, lease_expires_at, \
                    (SELECT (v.profile->'availability'->>'concurrency')::bigint \
                     FROM profile_versions v JOIN agent_profiles p ON p.role_id = v.role_id \
                     WHERE v.role_id = $1 AND v.version = p.current_version) AS concurrency \
             FROM node_presence WHERE node_id = $1 AND tenant_id = $2",
        )
        .bind(node_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| PresenceResponse {
            node_id: node_id.to_string(),
            online: r.online,
            suspended: r.suspended,
            last_seen_at: r.last_seen_at,
            lease_expires_at: r.lease_expires_at,
            state: crate::presence::presence_state(true, r.suspended, r.online, r.concurrency)
                .as_str()
                .to_string(),
        }))
    }
}

/// Decode a lowercase hex string to bytes (`None` on odd length or a non-hex
/// digit). Used for the handshake proof; NOT a general-purpose codec.
fn decode_hex(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    hex.as_bytes()
        .chunks(2)
        .map(|pair| {
            let hi = (pair[0] as char).to_digit(16)?;
            let lo = (pair[1] as char).to_digit(16)?;
            Some(((hi << 4) | lo) as u8)
        })
        .collect()
}

// ── Transactional bodies (`PHASE-0.6.2`) ────────────────────────────────────────

/// The transactional body of [`NodeChannelState::enqueue`]: the `.6.1` command
/// transaction enqueues an invite/challenge work item in the SAME transaction as
/// the thread event that produced it — an invitation exists iff its inbox row does.
/// The decision metadata (`.1.5.2`, ADR-008) rides the row: the admitting
/// authorization record, the policy digest, the decision time (the node-side
/// freshness TTL runs from it), and the tenant's epoch AT DECISION TIME.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn enqueue_in_tx<'e, E>(
    mut tx: E,
    node_id: &str,
    command_id: &str,
    tenant_id: &str,
    thread_id: &str,
    payload: &Value,
    authz_ref: &str,
    policy_digest: &str,
    decided_at: DateTime<Utc>,
    revocation_epoch: i64,
) -> Result<i64, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    sqlx::query_scalar(
        "INSERT INTO node_inbox (node_id, cursor, command_id, tenant_id, thread_id, payload, \
                                 authz_ref, policy_digest, decided_at, revocation_epoch) \
         VALUES ($1, \
                 (SELECT COALESCE(MAX(cursor), 0) + 1 FROM node_inbox WHERE node_id = $1), \
                 $2, $3, $4, $5, $6, $7, $8, $9) \
         RETURNING cursor",
    )
    .bind(node_id)
    .bind(command_id)
    .bind(tenant_id)
    .bind(thread_id)
    .bind(payload)
    .bind(authz_ref)
    .bind(policy_digest)
    .bind(decided_at)
    .bind(revocation_epoch)
    .fetch_one(&mut *tx)
    .await
}

/// The transactional body of [`NodeChannelState::record_event`].
pub(crate) async fn record_event_in_tx<'e, E>(
    mut tx: E,
    node_id: &str,
    event_id: &str,
    operation_id: &str,
    payload: &Value,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    Ok(sqlx::query(
        "INSERT INTO node_events (event_id, node_id, operation_id, payload, received_at) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (event_id) DO NOTHING",
    )
    .bind(event_id)
    .bind(node_id)
    .bind(operation_id)
    .bind(payload)
    .bind(now)
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1)
}

/// The transactional body of [`NodeChannelState::load_command`].
pub(crate) async fn load_command_in_tx<'e, E>(
    mut tx: E,
    node_id: &str,
    command_id: &str,
) -> Result<Option<(String, String, Value)>, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    sqlx::query_as::<_, (String, String, Value)>(
        "SELECT tenant_id, thread_id, payload FROM node_inbox \
         WHERE node_id = $1 AND command_id = $2",
    )
    .bind(node_id)
    .bind(command_id)
    .fetch_optional(&mut *tx)
    .await
}

/// The same lookup, bound to the tenant whose authority guard the caller holds
/// (`SIGNOFF-REPAIR.3.3.4.5`). The fold's effect SQL names the guarded tenant, so
/// a row that changed between the pre-lock resolution and this guarded read
/// matches nothing instead of being folded under a guard that does not cover it.
pub(crate) async fn load_command_for_tenant_in_tx<'e, E>(
    mut tx: E,
    node_id: &str,
    command_id: &str,
    tenant_id: &str,
) -> Result<Option<(String, String, Value)>, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    sqlx::query_as::<_, (String, String, Value)>(
        "SELECT tenant_id, thread_id, payload FROM node_inbox \
         WHERE node_id = $1 AND command_id = $2 AND tenant_id = $3",
    )
    .bind(node_id)
    .bind(command_id)
    .bind(tenant_id)
    .fetch_optional(&mut *tx)
    .await
}

/// The tenant whose authority guard must be held before a node event's effect is
/// applied — resolved BEFORE any lock is taken, so the guard can be acquired
/// first and the node's lease row second (`SIGNOFF-REPAIR.3.3.4.5`).
///
/// `None` means the event has no tenant-bound effect: an ordinary channel
/// receipt, or a command this node's ledger does not hold. Those record their
/// receipt and change nothing else, so there is nothing to order.
///
/// A stored tenant that is not a tenant id is refused rather than folded: an
/// effect no guard can cover must not be applied unordered.
async fn result_tenant_in_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    node_id: &str,
    payload: &Value,
) -> Result<Option<reasonbraid_core::TenantId>, ApiError> {
    let Some((_, command_id)) = crate::api::node_result_command(payload) else {
        return Ok(None);
    };
    let stored: Option<String> = sqlx::query_scalar(
        "SELECT tenant_id FROM node_inbox WHERE node_id = $1 AND command_id = $2",
    )
    .bind(node_id)
    .bind(command_id)
    .fetch_optional(&mut **tx)
    .await?;
    let Some(stored) = stored else {
        return Ok(None);
    };
    stored.parse().map(Some).map_err(|_| {
        eprintln!(
            "node channel: node {node_id} command {command_id} stores a tenant id that does not parse"
        );
        ApiError::internal()
    })
}

// ── HTTP surface ───────────────────────────────────────────────────────────────

/// The node channel router: `/v1/nodes/handshake` (the authenticated reconnect
/// exchange), `/v1/nodes/events` (node results), `/v1/nodes/ack` (cursor
/// acknowledgement), `/v1/nodes/poll` (live tail), `/v1/nodes/heartbeat` (lease
/// renewal), `/v1/nodes/presence` (observable online/offline state), and
/// `/v1/nodes/enroll` (the `.1.2.1` one-time-token enrollment — the token IS the
/// credential there, so that surface carries no principal header).
pub fn node_router(pool: PgPool, ca: Arc<crate::ca::ServerCa>) -> Router {
    let state = Arc::new(NodeChannelState::new(pool, ca));
    Router::new()
        .route("/v1/nodes/handshake", post(handshake))
        .route("/v1/nodes/events", post(events))
        .route("/v1/nodes/ack", post(ack))
        .route("/v1/nodes/poll", post(poll))
        .route("/v1/nodes/heartbeat", post(heartbeat))
        .route("/v1/nodes/presence", get(presence))
        .route("/v1/nodes/enroll", post(enroll))
        .route("/v1/nodes/rotate", post(rotate))
        .with_state(state)
}

fn check_version(version: u32) -> Result<(), ApiError> {
    if version == CHANNEL_VERSION {
        Ok(())
    } else {
        Err(ApiError::protocol_incompatible(version))
    }
}

async fn handshake(
    State(state): State<Arc<NodeChannelState>>,
    Json(req): Json<HandshakeRequest>,
) -> Result<Json<HandshakeResponse>, ApiError> {
    check_version(req.channel_version)?;
    // Authentication FIRST: a handshake without a valid certificate proof is
    // refused before any ledger fact (cursor, replay, receipts) is read.
    state.verify_cert_proof(&req).await?;
    // Then the nonce, and the ORDER matters (`SIGNOFF-REPAIR.4.2.2`): consuming
    // it only after the signature verifies means an unauthenticated caller
    // cannot burn nonces, so forged traffic can never deny a legitimate node a
    // value it was about to use.
    let mut conn = state.pool.acquire().await?;
    state
        .consume_proof_nonce(&mut *conn, &req.node_id, &req.nonce)
        .await?;
    drop(conn);

    let current = state.current_cursor(&req.node_id).await?;
    if req.last_acked_cursor > current {
        return Err(ApiError::cursor_ahead(req.last_acked_cursor, current));
    }
    let replay = state.replay(&req.node_id, req.last_acked_cursor).await?;

    let mut directives = Vec::with_capacity(req.ambiguous_attempts.len());
    for attempt in &req.ambiguous_attempts {
        match state.event_id_for_operation(&attempt.operation_id).await? {
            Some(event_id) => directives.push(Directive::Adjudicated {
                attempt_id: attempt.attempt_id.clone(),
                terminal: "reconciled".to_string(),
                evidence: format!(
                    "server holds event {event_id} for operation {}",
                    attempt.operation_id
                ),
            }),
            None => directives.push(Directive::NeedsAdjudication {
                attempt_id: attempt.attempt_id.clone(),
                reason: format!(
                    "no server receipt for operation {} — provider proof or operator \
                     adjudication required",
                    attempt.operation_id
                ),
            }),
        }
    }

    let mut known_events = Vec::new();
    for operation_id in &req.pending_operations {
        if let Some(event_id) = state.event_id_for_operation(operation_id).await? {
            known_events.push(KnownEvent {
                operation_id: operation_id.clone(),
                event_id,
            });
        }
    }

    // The authenticated reconnect issued a fresh lease: a NEW fencing token and
    // a bumped epoch, so a stale process fenced by this rotation is refused from
    // here on — its token, its epoch-bound renewals, and its writes all fail.
    let (fencing_token, lease_epoch, lease_expires_at) =
        state.issue_lease(&req.node_id, Utc::now()).await?;
    let (revocation_epoch, server_time) = state.epoch_and_server_time(&req.node_id).await?;

    Ok(Json(HandshakeResponse {
        channel_version: CHANNEL_VERSION,
        current_cursor: current,
        replay,
        directives,
        known_events,
        fencing_token,
        lease_expires_at,
        revocation_epoch,
        lease_epoch,
        server_time,
    }))
}

/// The certificate rotation (`.1.2.2`): the node presents its CURRENT
/// certificate (proof over the rotate coverage) and receives a FRESH key +
/// certificate for the same node id. Rotation is ADDITIVE — the old
/// fingerprint stays valid until expiry or revocation (`.1.3`), so a running
/// session is never cut; the fresh identity rides the NEXT handshake.
async fn rotate(
    State(state): State<Arc<NodeChannelState>>,
    Json(req): Json<RotateRequest>,
) -> Result<Json<RotateResponse>, ApiError> {
    check_version(req.channel_version)?;

    // ONE transaction, and the node row is locked FIRST
    // (`SIGNOFF-REPAIR.4.2.1`). The proof check, the host lookup and the insert
    // used to be three statements on the pool, so the rotation's decision and
    // its effect were not atomic with respect to a revocation: a revocation that
    // committed between them left the node holding a certificate it had just
    // been denied — and since `.4.1.3` and `.4.1.3.1` both gate on exactly that
    // certificate, the node's lease renewal and its work delivery came back.
    //
    // The lock is the one the revocation already takes before it changes
    // anything ("tenant-bound selection inside the guard, locked before the
    // change"), so no new lock, no new guard and no new ordering are invented —
    // the two paths simply take the same row in the same mode. A rotation that
    // wins it commits a certificate the revocation's later UPDATE sees and
    // revokes; one that loses it re-reads the certificate after the revocation
    // committed and is refused. Both orders are correct, which is the property
    // the unrepaired shape did not have.
    let mut tx = state.pool.begin().await?;
    let host_claim: Option<String> = sqlx::query_scalar(
        "SELECT h.name FROM nodes n JOIN hosts h ON n.host_id = h.host_id \
         WHERE n.node_id = $1 FOR UPDATE OF n",
    )
    .bind(&req.node_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(host_claim) = host_claim else {
        return Err(ApiError::unknown_node(&req.node_id));
    };
    // AFTER the lock: a rotation that queued behind a revocation reads the
    // certificate as that revocation left it.
    state.verify_rotate_proof_in_tx(&mut tx, &req).await?;
    // And after the signature, so forged traffic cannot burn nonces
    // (`SIGNOFF-REPAIR.4.2.2`). Inside THIS transaction, which already holds the
    // node row: a rotation that fails burns no nonce.
    state
        .consume_proof_nonce(&mut *tx, &req.node_id, &req.nonce)
        .await?;

    // `SIGNOFF-REPAIR.4.1.6`: a typed refusal, not an unwind. Issuance now
    // checks the claim before the token is written, so a stored host name that
    // the library refuses means a row predating that check — the rotation says
    // so instead of dropping the connection.
    let leaf = crate::ca::issue_node_leaf(&state.ca, &req.node_id, &host_claim)
        .map_err(|refusal| ApiError::host_claim_refused(&refusal))?;
    let (cert_der, key_der) = (leaf.cert_der, leaf.key_der);
    let cert_fingerprint = crate::ca::cert_fingerprint(&cert_der);
    let now = Utc::now();
    // `SIGNOFF-REPAIR.3.4.3.1.1`: the certificate's OWN expiry, not a second
    // derivation from a second clock read. A verifier enforces `not_after`, so
    // a stored value computed alongside it is a claim about the certificate
    // that nothing bound to the certificate.
    let cert_expires_at = leaf.not_after;
    sqlx::query(
        "INSERT INTO node_certificates \
         (cert_fingerprint, node_id, cert_der, key_der, issued_at, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&cert_fingerprint)
    .bind(&req.node_id)
    .bind(&cert_der)
    .bind(&key_der)
    .bind(now)
    .bind(cert_expires_at)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(RotateResponse {
        node_id: req.node_id,
        cert_der: crate::ca::to_hex(&cert_der),
        key_der: crate::ca::to_hex(&key_der),
        cert_fingerprint,
        cert_expires_at,
    }))
}

async fn events(
    State(state): State<Arc<NodeChannelState>>,
    Json(req): Json<EventSubmission>,
) -> Result<Json<EventReceipt>, ApiError> {
    check_version(req.channel_version)?;
    state
        .verify_fencing(&req.node_id, &req.fencing_token, req.lease_epoch)
        .await?;
    // ONE transaction (`PHASE-0.6.2`): the receipt and — when the payload is a
    // thread work result — the domain application (claim → authorize → validate →
    // apply + reservation settlement) commit together. A duplicate event inserts
    // no receipt and applies no domain effect (one effect, ever); a rejected
    // application still commits the receipt (the node DID emit this event) with
    // the rejection stored as the command's idempotent result.
    let mut tx = state.pool.begin().await?;
    // The tenant authority guard, FIRST — before the lease row, the receipt, the
    // idempotency claim and every domain lock (`SIGNOFF-REPAIR.3.3.4.5`). The
    // tenant is resolved from the inbox row by a plain read that takes no lock,
    // so the guard is genuinely the first lock this transaction holds and no
    // inversion against the lease is possible. Shared, because node results read
    // authority and must run concurrently with one another; a revocation takes
    // the exclusive mode and therefore fences every result that has not already
    // reached this point.
    //
    // Before this, the path took no guard at all, so a node-emitted result and a
    // revocation had no ordering between them to narrow.
    let guarded_tenant = result_tenant_in_tx(&mut tx, &req.node_id, &req.payload).await?;
    if let Some(tenant) = guarded_tenant {
        crate::authority::transaction::acquire_in_tx(
            &mut tx,
            tenant,
            crate::authority::transaction::GuardMode::Shared,
        )
        .await?;
    }
    // The check-vs-commit window (`.2.2`): the admission check above ran
    // OUTSIDE the transaction — a handshake that lands between then and the
    // apply would have rotated the lease under this write. Re-verify INSIDE,
    // locking the lease row, so the rotation is observed, not lost.
    state
        .verify_fencing_in_tx(&mut tx, &req.node_id, &req.fencing_token, req.lease_epoch)
        .await?;
    let accepted = record_event_in_tx(
        &mut *tx,
        &req.node_id,
        &req.event_id,
        &req.operation_id,
        &req.payload,
        Utc::now(),
    )
    .await?;
    if accepted {
        if let Some(tenant) = guarded_tenant {
            if let Err(e) =
                crate::api::apply_node_result_in_tx(&mut tx, &req.node_id, &req.payload, tenant)
                    .await
            {
                crate::telemetry::metrics().incr("results_rejected");
                crate::log_event!("thread_result_rejected", "node_id" => &req.node_id, "reason" => e.to_string());
            }
        }
    }
    // A REJECTED application is a committed fact: the node did emit this event,
    // and the refusal is stored as the work command's idempotent result. A SQL
    // FAILURE is not — it aborts the transaction, so the COMMIT below would be
    // executed as a ROLLBACK and this handler would answer `accepted: true` for
    // a receipt that does not exist, which the node would never re-emit. Probe
    // the transaction's health first, exactly as the tenant guard's own runner
    // does, so a discarded error cannot become a reported success.
    sqlx::query("SELECT 1").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(EventReceipt {
        channel_version: CHANNEL_VERSION,
        accepted,
    }))
}

/// Mark the node's inbox terminal up to a cursor.
///
/// # One transaction, re-verified inside it (`SIGNOFF-REPAIR.4.2.4`)
///
/// This ran as three separate pool statements: the admission check, the cursor
/// read, and the `UPDATE`. A handshake landing between the first and the third
/// rotated the lease under a write that committed anyway, so a session that had
/// just been fenced could mark the NEW session's delivery acknowledged.
///
/// 🔴 `acknowledged_at` is not merely a record: it is the retention prune's
/// DELETE predicate (`authority/node_admin.rs`). A stale ack makes work the
/// live session is still holding eligible for deletion, and the node's ledger
/// and the server's then disagree permanently — which is exactly what the
/// channel's duplicate-safety contract exists to prevent.
///
/// The fix is `events`' shape, not a second one: re-verify with
/// `verify_fencing_in_tx`, which locks the lease row, and commit the
/// acknowledgement with it. The cursor bound is read in the same transaction so
/// the value checked and the rows marked come from one snapshot.
///
/// ⛔ NO tenant guard, deliberately, and not by omission. `events` takes one
/// because it applies domain effects; an acknowledgement touches only this
/// node's inbox. `.4.1.3.1` decided that `ack` and `events` keep their pure
/// fencing check so a revoked node's session finishes the work it already
/// holds — adding a guard here would order acknowledgements against revocation
/// and cut that tail, reversing a decision this leaf does not own.
async fn ack(
    State(state): State<Arc<NodeChannelState>>,
    Json(req): Json<AckRequest>,
) -> Result<Json<AckResponse>, ApiError> {
    check_version(req.channel_version)?;
    state
        .verify_fencing(&req.node_id, &req.fencing_token, req.lease_epoch)
        .await?;
    let mut tx = state.pool.begin().await?;
    // The check-vs-commit window (`.2.2`): the admission check above ran OUTSIDE
    // the transaction. Re-verify INSIDE it, with the lease row locked, so a
    // handshake that rotates the lease is observed rather than written under.
    state
        .verify_fencing_in_tx(&mut tx, &req.node_id, &req.fencing_token, req.lease_epoch)
        .await?;
    let current = state.current_cursor_in_tx(&mut tx, &req.node_id).await?;
    if req.ack_cursor > current {
        return Err(ApiError::cursor_ahead(req.ack_cursor, current));
    }
    let acknowledged = state
        .acknowledge_in_tx(&mut tx, &req.node_id, req.ack_cursor, Utc::now())
        .await?;
    tx.commit().await?;
    Ok(Json(AckResponse {
        channel_version: CHANNEL_VERSION,
        acknowledged,
    }))
}

async fn poll(
    State(state): State<Arc<NodeChannelState>>,
    Json(req): Json<PollRequest>,
) -> Result<Json<PollResponse>, ApiError> {
    check_version(req.channel_version)?;
    if req.node_id.is_empty() {
        return Err(ApiError::bad_request("node_id is required".to_string()));
    }
    state
        .verify_fencing(&req.node_id, &req.fencing_token, req.lease_epoch)
        .await?;
    let current = state.current_cursor(&req.node_id).await?;
    if req.after_cursor > current {
        return Err(ApiError::cursor_ahead(req.after_cursor, current));
    }
    let commands = state.replay(&req.node_id, req.after_cursor).await?;
    let (revocation_epoch, server_time) = state.epoch_and_server_time(&req.node_id).await?;
    Ok(Json(PollResponse {
        channel_version: CHANNEL_VERSION,
        current_cursor: current,
        commands,
        revocation_epoch,
        server_time,
    }))
}

/// Renew a LIVE lease: the fencing token must be the latest handshake's, the
/// epoch must still be current, and the lease must not have expired. Renewal
/// returns the new expiry; the token itself rotates only at the handshake. A
/// renewal whose epoch was superseded between the check and the write matches
/// no row — the stale heartbeat loses the race (`.2.2`), it never extends the
/// session that fenced it.
///
/// Every one of those three conditions rides the WRITE, not this pre-check
/// (`.2.2` for the epoch, `.4.1.3` for the credential, `.4.2.3` for the
/// expiry). `verify_fencing` still runs first, because it is what tells a node
/// a fenced token from a lapsed lease before any write is attempted — but it is
/// an admission check, and nothing downstream depends on it having been true a
/// statement later.
async fn heartbeat(
    State(state): State<Arc<NodeChannelState>>,
    Json(req): Json<HeartbeatRequest>,
) -> Result<Json<HeartbeatResponse>, ApiError> {
    check_version(req.channel_version)?;
    state
        .verify_fencing(&req.node_id, &req.fencing_token, req.lease_epoch)
        .await?;
    let lease_expires_at = match state
        .renew_lease(&req.node_id, req.lease_epoch, Utc::now())
        .await
    {
        Ok(expiry) => expiry,
        Err(_) => {
            return Err(state
                .classify_renewal_refusal(&req.node_id, req.lease_epoch)
                .await)
        }
    };
    Ok(Json(HeartbeatResponse {
        channel_version: CHANNEL_VERSION,
        fencing_token: req.fencing_token,
        lease_expires_at,
    }))
}

/// One node's observable presence. Presence is a read-only observability fact
/// (no token, no key, no fingerprint — never a credential), derived from the
/// lease clock; an unenrolled node is a 404, not a fabricated `offline`.
/// One node's observable state — **bound to the caller's own tenant**
/// (`SIGNOFF-REPAIR.3.5.5`).
///
/// 🔴 This handler used to take no `HeaderMap`, so it never learned who was
/// calling and never asked whether they may. It then selected from
/// `node_presence` — a view that carries `tenant_id` (migration 0017, from
/// `nodes.tenant_id NOT NULL`) — by `node_id` alone. Anyone who could reach the
/// port read any node's presence, and `200` versus `unknown_node` told them
/// which node ids exist. Meanwhile `GET /v1/admin/nodes/presence` gated the SAME
/// view behind a `tenant_admin` grant, and `rb-server.rs` serves both from one
/// process on one port.
///
/// The tenant is DERIVED from the authenticated caller rather than accepted from
/// the query, which is `SIGNOFF-REPAIR.3.5.2.1`'s shape: a principal belongs to
/// exactly one tenant structurally, so there is no second identifier to bind and
/// no wire change. The only caller in this repository — the web console's
/// `api()` helper — already sends the header on every GET, and no node client
/// calls this route.
///
/// ⛔ A node in ANOTHER tenant answers `unknown_node`, exactly as a node that
/// does not exist does. Distinguishing them would keep the existence oracle that
/// §9.8's `scope_hidden` exists to prevent, and this file already takes that
/// stance for the handshake ("or whose node has no key — the same refusal: no
/// existence leak").
async fn presence(
    State(state): State<Arc<NodeChannelState>>,
    headers: HeaderMap,
    Query(params): Query<PresenceParams>,
) -> Result<Json<PresenceResponse>, ApiError> {
    if params.node_id.is_empty() {
        return Err(ApiError::bad_request("node_id is required".to_string()));
    }
    let principal =
        crate::api::resolve_principal(&headers).map_err(|_| ApiError::unauthenticated())?;
    let Some(tenant) = crate::api::reader_tenant(&state.pool, &principal).await? else {
        // An unenrolled principal reads no presence — the same wording the
        // reference, snapshot and assessment reads use for this case.
        return Err(ApiError::unenrolled());
    };
    match state.presence(&params.node_id, &tenant).await? {
        Some(p) => Ok(Json(p)),
        None => Err(ApiError::unknown_node(&params.node_id)),
    }
}

// ── Node enrollment (PHASE-1.2.1; backlog 11) ───────────────────────────────────

/// The `POST /v1/nodes/enroll` body: the one-time token (issued by an authorized
/// principal holding `TenantAdmin` at `/v1/nodes/enroll-tokens` — a human or an
/// agent role; see that route's note, `SIGNOFF-REPAIR.4.1.4`), the node's id,
/// the host claim the token was
/// bound to, the token nonce, and the node's dev signing secret (the server IS the
/// dev trust store — the `.6.1` stance; the secret is the key the `.1.2.2`
/// handshake's HMAC proof rides). The §8.1 incarnation facts (`.1.6.1`) ride the
/// same body: the provider/model/harness/configuration the node KNOWS at start —
/// written to the `incarnations` row when the node id is the role wire id it
/// serves (the dev wiring; a plain `nod_…` node serves no role, so it has no
/// incarnation row).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeEnrollRequest {
    pub token_id: String,
    pub node_id: String,
    pub host_claim: String,
    pub nonce: String,
    pub key_secret: String,
    /// §8.1: the provider backend (e.g. `anthropic`, `openai`, `fake`).
    #[serde(default)]
    pub provider: Option<String>,
    /// §8.1: the model/harness version the node starts with.
    #[serde(default)]
    pub model: Option<String>,
    /// §8.1: the harness (the adapter family — `claude`, `codex`, `fake`, …).
    #[serde(default)]
    pub harness: Option<String>,
    /// §8.1: free-form configuration facts (stored verbatim, never interpreted).
    #[serde(default)]
    pub config: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeEnrollResponse {
    pub node_id: String,
    pub host_id: String,
    /// The issued workload leaf (hex DER): CN `node:<node_id>`, SAN = the
    /// token's host claim, 10-minute validity (`.1.2.1`, ADR-007).
    pub cert_der: String,
    /// The leaf's private key (hex DER) — server-generated, dev-escrowed
    /// (the `.1.2.1` trust-store stance; the Internet profile re-evaluates).
    pub key_der: String,
    /// sha256 over the cert DER — the node-id → current-fingerprint binding
    /// the `.1.2.2` handshake gates on.
    pub cert_fingerprint: String,
    pub cert_expires_at: DateTime<Utc>,
    /// The incarnation row this enrollment wrote (`.1.6.1`) — present when the
    /// node id is the role wire id it serves (a plain `nod_…` node serves no
    /// role and records no incarnation).
    pub incarnation_id: Option<String>,
}

/// A token refusal (unknown / used / expired / bound elsewhere / nonce mismatch) —
/// `unauthorized` on the wire; the refusal is ALWAYS audited (a committed row).
fn enrollment_refused(reason: impl Into<String>) -> ApiError {
    ApiError {
        status: StatusCode::UNAUTHORIZED,
        code: "unauthorized",
        message: format!("the enrollment token was refused: {}", reason.into()),
    }
}

/// Write a refusal audit row (the denial-row pattern from the budget engine) and
/// return its record id; the caller commits and turns it into the typed refusal.
async fn insert_refusal_audit(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    token_tenant: Option<&str>,
    node_id: &str,
    token_id: &str,
    reason: &str,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO node_enroll_audit (record_id, tenant_id, node_id, token_id, decision, reason) \
         VALUES ('naux_' || gen_random_uuid()::text, COALESCE($1, ''), $2, $3, 'refused', $4) \
         RETURNING record_id",
    )
    .bind(token_tenant)
    .bind(node_id)
    .bind(token_id)
    .bind(reason)
    .fetch_one(&mut **tx)
    .await
}

/// One stored enrollment token row (the 0008 table, as amended by 0059/0060).
#[derive(sqlx::FromRow)]
struct EnrollmentTokenRow {
    tenant_id: String,
    node_id: String,
    host_claim: String,
    nonce: String,
    expires_at: DateTime<Utc>,
    used_at: Option<DateTime<Utc>>,
    /// Set when the grant or boundary that issued this token was revoked
    /// (`SIGNOFF-REPAIR.4.1.2`). The decision was made there, under the tenant's
    /// exclusive guard; redemption only reads the column it wrote.
    voided_at: Option<DateTime<Utc>>,
}

/// Consume a one-time enrollment token: validate it, get-or-create the host, land
/// the node + its dev key in the 0007/0008 identity tables, and mark the token
/// used — ONE transaction. A refused attempt commits only its audit row.
async fn enroll(
    State(state): State<Arc<NodeChannelState>>,
    Json(req): Json<NodeEnrollRequest>,
) -> Result<Json<NodeEnrollResponse>, ApiError> {
    if !is_valid_node_identity(&req.node_id) {
        return Err(ApiError::bad_request(format!(
            "node_id `{}` is not a valid node identity (a `nod_…` node id or the `rol_…` \
             role wire id the dev profile serves)",
            req.node_id
        )));
    }
    let fingerprint = Sha256::digest(req.key_secret.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();

    let mut tx = state.pool.begin().await?;
    let now = Utc::now();

    // The token row is the serialization point: one token, one use, ever.
    let token: Option<EnrollmentTokenRow> = sqlx::query_as(
        "SELECT tenant_id, node_id, host_claim, nonce, expires_at, used_at, voided_at \
         FROM node_enrollment_tokens WHERE token_id = $1 FOR UPDATE",
    )
    .bind(&req.token_id)
    .fetch_optional(&mut *tx)
    .await?;

    let token_tenant: Option<String> = token.as_ref().map(|t| t.tenant_id.clone());

    // The token validation is one pure decision; the refusal audit row commits
    // with the error (the denial-row pattern — a refusal is an audited event).
    let refusal: Option<&'static str> = match &token {
        None => Some("the token is unknown"),
        Some(t) => {
            if t.used_at.is_some() {
                Some("the token was already used")
            } else if t.voided_at.is_some() {
                // A DISTINCT reason, and the distinction is the operator's:
                // waiting does not help and neither does re-issuing under the
                // same authority. `FOR UPDATE` above is what orders this read
                // against the revocation that wrote the column
                // (`SIGNOFF-REPAIR.4.1.2`).
                Some("the authority that issued this token has been revoked")
            } else if t.expires_at <= now {
                Some("the token has expired")
            } else if t.node_id != req.node_id {
                Some("the token is bound to a different node id")
            } else if t.host_claim != req.host_claim {
                Some("the token is bound to a different host claim")
            } else if t.nonce != req.nonce {
                Some("the nonce does not match the issued token")
            } else {
                None
            }
        }
    };
    if let Some(reason) = refusal {
        let record = insert_refusal_audit(
            &mut tx,
            token_tenant.as_deref(),
            &req.node_id,
            &req.token_id,
            reason,
        )
        .await?;
        tx.commit().await?;
        return Err(enrollment_refused(format!("{reason} (audit {record})")));
    }
    // Safe: `refusal` is `None` only when the token row exists and passed.
    let token = token.expect("refusal none implies the token exists");
    let tenant_id = token.tenant_id;

    // Get-or-create the host row (the 0008 partial unique index keys it).
    sqlx::query(
        "INSERT INTO hosts (host_id, tenant_id, name) \
         VALUES ('hst_' || gen_random_uuid()::text, $1, $2) \
         ON CONFLICT (tenant_id, name) WHERE name IS NOT NULL DO NOTHING",
    )
    .bind(&tenant_id)
    .bind(&req.host_claim)
    .execute(&mut *tx)
    .await?;
    let host_id: String =
        sqlx::query_scalar("SELECT host_id FROM hosts WHERE tenant_id = $1 AND name = $2")
            .bind(&tenant_id)
            .bind(&req.host_claim)
            .fetch_one(&mut *tx)
            .await?;

    // The node + its dev key + the token consumption + the audit row: all or none.
    // A REPLACEMENT enroll (`.7.2` — the node-lost ritual) rides the same path:
    // the node row exists but every certificate is revoked (the operator declared
    // the loss), so the enrollment proceeds as the replacement — a fresh cert +
    // key + incarnation land below, the old certs stay revoked (fenced). The
    // state check runs BEFORE any insert (a unique-violation probe would abort
    // the transaction).
    // `SIGNOFF-REPAIR.3.5.1`: read the node's OWNER, not "does it exist in MY
    // tenant". `nodes.node_id` is a GLOBAL primary key, so a node identity
    // belongs to at most one tenant ever — and the tenant-scoped `EXISTS` this
    // replaces reported false for a node owned by someone else, sending the
    // insert below into that primary key. The comment above is right that a
    // unique violation aborts the transaction; the check simply asked the wrong
    // question, and the answer was `500 dependency_unavailable`.
    let (active_certs, total_certs, owner): (i64, i64, Option<String>) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM node_certificates WHERE node_id = $1 AND revoked_at IS NULL), \
                (SELECT count(*) FROM node_certificates WHERE node_id = $1), \
                (SELECT tenant_id FROM nodes WHERE node_id = $1)",
    )
    .bind(&req.node_id)
    .fetch_one(&mut *tx)
    .await?;

    // ⛔ A node owned by ANOTHER tenant is refused before anything else, and in
    // particular before the replacement branch. That branch swaps `node_keys`
    // and issues a fresh workload certificate for the node id WITHOUT a tenant
    // check, so widening the existence test without this arm would have handed
    // a foreign tenant the node's key the moment its owner revoked its
    // certificates — a takeover, strictly worse than the 500 being repaired.
    //
    // The refusal reuses the same-tenant duplicate's wording deliberately: the
    // caller supplied the node id, and learns exactly what they would have
    // learned about a node of their own. Nothing names the owner.
    if let Some(other) = owner.as_deref() {
        if other != tenant_id {
            let record = insert_refusal_audit(
                &mut tx,
                token_tenant.as_deref(),
                &req.node_id,
                &req.token_id,
                "the node is already enrolled",
            )
            .await?;
            tx.commit().await?;
            return Err(enrollment_refused(format!(
                "the node is already enrolled (audit {record})"
            )));
        }
    }

    let replacement = if owner.is_some() {
        // The node row already exists: a replacement enroll requires the
        // operator's revocation first (every certificate revoked) — otherwise it
        // is a duplicate enrollment and refuses (the audit row commits with the
        // error).
        if active_certs == 0 && total_certs > 0 {
            true
        } else {
            let record = insert_refusal_audit(
                &mut tx,
                token_tenant.as_deref(),
                &req.node_id,
                &req.token_id,
                "the node is already enrolled",
            )
            .await?;
            tx.commit().await?;
            return Err(enrollment_refused(format!(
                "the node is already enrolled (audit {record})"
            )));
        }
    } else {
        sqlx::query("INSERT INTO nodes (node_id, host_id, tenant_id) VALUES ($1, $2, $3)")
            .bind(&req.node_id)
            .bind(&host_id)
            .bind(&tenant_id)
            .execute(&mut *tx)
            .await?;
        false
    };

    // `SIGNOFF-REPAIR.4.1.5`: a replacement MOVES the node to its new host.
    //
    // 🔴 The row kept the old `host_id` while the response echoed the new one,
    // and `rotate` reads the certificate's host from THIS row
    // (`SELECT h.name FROM nodes n JOIN hosts h ON n.host_id = h.host_id`). So
    // the enrollment certificate named the machine the node actually runs on and
    // the FIRST automatic rotation — which happens within half a leaf lifetime —
    // silently reverted the SAN to the machine it was replaced from. Measured:
    // `nodes.host_id after the replacement names: host-a`.
    //
    // The `hosts` row for the new claim already exists at this point: the
    // get-or-create above ran for `req.host_claim` and `host_id` holds its id.
    if replacement {
        sqlx::query("UPDATE nodes SET host_id = $2 WHERE node_id = $1")
            .bind(&req.node_id)
            .bind(&host_id)
            .execute(&mut *tx)
            .await?;
    }

    // `SIGNOFF-REPAIR.4.1.5`: a replacement ENDS the replaced machine's session.
    //
    // 🔴 Leaving the lease alone was not merely untidy, it partially undid
    // `.4.1.3`. That leaf stopped a revoked node renewing by requiring a usable
    // certificate in `renew_lease`'s WHERE — and a replacement issues a fresh
    // certificate for the SAME node id, so the predicate becomes true again and
    // the REPLACED process, still holding its old token and epoch, resumes
    // renewing a lease it should never have had back. Measured: its heartbeat
    // answered `200` after the replacement.
    //
    // The epoch is BUMPED rather than the row deleted. `.2.2`'s fencing
    // vocabulary is the epoch, so moving it is how this codebase says "that
    // session is over"; and deleting would let `issue_lease`'s INSERT path reset
    // the epoch to 1, breaking the per-node monotonicity `.4.2.3` relies on when
    // it argues that `(node_id, lease_epoch)` determines the fencing token. The
    // expiry is set too, so presence reads `offline` immediately rather than
    // waiting out the remaining TTL.
    //
    // The instant is the DATABASE's, and `clock_timestamp()` rather than `now()`
    // (`SIGNOFF-REPAIR.4.2.3.1`): this runs inside the enrolment transaction, so
    // `now()` would be that transaction's START and would record the session as
    // having ended before the work that ended it. ⛔ The third writer of this
    // column, and `.4.2.3`'s census — which found two — predates it: `.4.1.5`
    // (REPAIR-0167) added this statement afterwards.
    //
    // ⚠️ Unlike the other two writers this site takes no clock ARGUMENT — it
    // sampled `Utc::now()` inside the handler — so there is no seam through
    // which a fixture on one host could separate the two clocks, and this change
    // is correct by the rule rather than by measurement. What it used to cost is
    // stated rather than covered: with the process clock ahead by `S`, presence
    // reported a replaced node `online` for `S` after its replacement. The epoch
    // bump is what FENCES the old session either way.
    if replacement {
        sqlx::query(
            "UPDATE node_leases \
             SET lease_epoch = lease_epoch + 1, lease_expires_at = clock_timestamp() \
             WHERE node_id = $1",
        )
        .bind(&req.node_id)
        .execute(&mut *tx)
        .await?;
    }

    // The dev key: a fresh enrollment inserts the row; a replacement SWAPS the
    // secret (the old one died with the machine — one dev secret per node).
    if replacement {
        sqlx::query(
            "UPDATE node_keys SET key_fingerprint = $1, key_secret = $2 WHERE node_id = $3",
        )
        .bind(&fingerprint)
        .bind(&req.key_secret)
        .bind(&req.node_id)
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO node_keys (node_id, key_fingerprint, key_secret) VALUES ($1, $2, $3)",
        )
        .bind(&req.node_id)
        .bind(&fingerprint)
        .bind(&req.key_secret)
        .execute(&mut *tx)
        .await?;
    }

    // The workload certificate (`.1.2.1`, ADR-007): issue the short-lived leaf,
    // persist it, and return it (with the dev-escrowed key) to the node.
    // `SIGNOFF-REPAIR.4.1.6`: a typed refusal, not an unwind. The transaction
    // rolls back with the `?`, exactly as the panic's unwind did — what changes
    // is that the caller receives an answer, and the token is not left
    // outstanding-and-unredeemable for the rest of its lifetime.
    let leaf = crate::ca::issue_node_leaf(&state.ca, &req.node_id, &req.host_claim)
        .map_err(|refusal| ApiError::host_claim_refused(&refusal))?;
    let (cert_der, key_der) = (leaf.cert_der, leaf.key_der);
    let cert_fingerprint = crate::ca::cert_fingerprint(&cert_der);
    // `SIGNOFF-REPAIR.3.4.3.1.1`: the certificate's own expiry. This site's `now`
    // is sampled before the enrollment transaction's database work, so the old
    // `now + LEAF_TTL_SECS` under-reported the certificate's real validity by
    // however long that work took.
    let cert_expires_at = leaf.not_after;
    sqlx::query(
        "INSERT INTO node_certificates \
         (cert_fingerprint, node_id, cert_der, key_der, issued_at, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&cert_fingerprint)
    .bind(&req.node_id)
    .bind(&cert_der)
    .bind(&key_der)
    .bind(now)
    .bind(cert_expires_at)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE node_enrollment_tokens SET used_at = $2 WHERE token_id = $1")
        .bind(&req.token_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;

    let decision = if replacement { "replaced" } else { "enrolled" };
    sqlx::query(
        "INSERT INTO node_enroll_audit (record_id, tenant_id, node_id, token_id, decision, reason) \
         VALUES ('naux_' || gen_random_uuid()::text, $1, $2, $3, $4, NULL)",
    )
    .bind(&tenant_id)
    .bind(&req.node_id)
    .bind(&req.token_id)
    .bind(decision)
    .execute(&mut *tx)
    .await?;

    // The incarnation writer (`.1.6.1`; deferral #4's first half): when the node
    // id is the agent ROLE wire id it serves (the dev wiring — the work path),
    // record the §8.1 facts the node declared. A plain `nod_…` node serves no
    // role, so it records no incarnation (the hierarchy's role_id is NOT NULL).
    // ⛔ CORRECTION (`SIGNOFF-REPAIR.4.1.5`): "re-enrollment cannot duplicate —
    // the `nodes` primary key refuses a second enroll for the id" is FALSE, and
    // it is the sentence that hid this. A REPLACEMENT is precisely a second
    // enroll for an existing id; the primary key never sees it, because the
    // replacement branch updates rather than inserts. Measured: a replaced role
    // held `2 open of 2` incarnations, each reading as current.
    //
    // So a replacement CLOSES the previous incarnation first. §8.1 history is
    // kept, never overwritten — two rows remain, and exactly one is open.
    //
    // ⚠️ The consequence was narrower than "the wrong incarnation is selected",
    // and that was measured too: both selectors (`api.rs:4300`, `api.rs:6266`)
    // order by `valid_from DESC LIMIT 1`, so each already picked the newest. The
    // defect is in the LEDGER — an incarnation that never ends cannot be
    // attributed against, which is what §8.1 exists for, and the operator
    // listing at `api.rs:5479` showed a node with two live identities.
    let incarnation_id: Option<String> = if req
        .node_id
        .parse::<reasonbraid_core::AgentRoleId>()
        .is_ok()
    {
        let id = reasonbraid_core::AgentIncarnationId::new().to_string();
        if replacement {
            sqlx::query(
                "UPDATE incarnations SET valid_to = $2 WHERE role_id = $1 AND valid_to IS NULL",
            )
            .bind(&req.node_id)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        }
        sqlx::query(
                "INSERT INTO incarnations \
                 (incarnation_id, role_id, tenant_id, provider, model, harness, config, valid_from) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(&id)
            .bind(&req.node_id)
            .bind(&tenant_id)
            .bind(&req.provider)
            .bind(&req.model)
            .bind(&req.harness)
            .bind(&req.config)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        Some(id)
    } else {
        None
    };

    tx.commit().await?;
    Ok(Json(NodeEnrollResponse {
        node_id: req.node_id,
        host_id,
        cert_der: crate::ca::to_hex(&cert_der),
        key_der: crate::ca::to_hex(&key_der),
        cert_fingerprint,
        cert_expires_at,
        incarnation_id,
    }))
}
