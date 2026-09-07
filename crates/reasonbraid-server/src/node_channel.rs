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
//! - expiry is a database fact (`lease_expires_at`, 60 s dev TTL): a heartbeat only
//!   renews a LIVE lease; once expired, presence shows `offline` and channel traffic
//!   is refused until the node re-proves its key with a new handshake.
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
    http::StatusCode,
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
}

/// The exact fields the rotate proof covers, in canonical (serde field) order.
#[derive(Debug, Serialize)]
pub struct RotateCoverage<'a> {
    pub channel_version: u32,
    pub node_id: &'a str,
    pub cert_der: &'a str,
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
    pub async fn revocation_epoch(&self, node_id: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT t.revocation_epoch FROM tenants t JOIN nodes n ON n.tenant_id = t.tenant_id \
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
        };
        // The canonical form is the mirrored struct's JSON: field order and
        // encoding are fixed by the struct shape on both sides.
        let canonical = serde_json::to_vec(&coverage)
            .map_err(|e| ApiError::bad_request(format!("unencodable handshake fields: {e}")))?;
        let spki = crate::ca::extract_point(&cert_der).map_err(|_| ApiError::proof_refused())?;
        crate::ca::verify_signature(&spki, &canonical, &signature)
            .map_err(|_| ApiError::proof_refused())
    }

    /// The rotate proof check (the same ladder over the rotate coverage).
    pub async fn verify_rotate_proof(&self, req: &RotateRequest) -> Result<(), ApiError> {
        let cert_der = decode_hex(&req.cert_der).ok_or_else(ApiError::proof_refused)?;
        let signature = decode_hex(&req.proof_signature).ok_or_else(ApiError::proof_refused)?;
        crate::ca::verify_leaf_chain(&self.ca, &cert_der).map_err(|_| ApiError::proof_refused())?;
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
        let coverage = RotateCoverage {
            channel_version: req.channel_version,
            node_id: &req.node_id,
            cert_der: &req.cert_der,
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
    pub async fn verify_fencing(
        &self,
        node_id: &str,
        token: &str,
        lease_epoch: i64,
    ) -> Result<(), ApiError> {
        let expires: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT lease_expires_at FROM node_leases \
             WHERE node_id = $1 AND fencing_token = $2 AND lease_epoch = $3",
        )
        .bind(node_id)
        .bind(token)
        .bind(lease_epoch)
        .fetch_optional(&self.pool)
        .await?;
        match expires {
            None => {
                crate::telemetry::metrics().incr("lease_refusals");
                Err(ApiError::fencing_refused())
            }
            Some(expires) if expires <= Utc::now() => {
                crate::telemetry::metrics().incr("lease_refusals");
                Err(ApiError::lease_expired())
            }
            Some(_) => Ok(()),
        }
    }

    /// The check-vs-commit re-verification (`.2.2`): the same fencing facts,
    /// checked INSIDE the caller's transaction with the lease row locked — a
    /// handshake that rotates the lease between an admission check and the
    /// apply is observed here (the row no longer matches), so a fenced session
    /// can never ride a stale admission into the write.
    pub async fn verify_fencing_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        node_id: &str,
        token: &str,
        lease_epoch: i64,
    ) -> Result<(), ApiError> {
        let expires: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT lease_expires_at FROM node_leases \
             WHERE node_id = $1 AND fencing_token = $2 AND lease_epoch = $3 \
             FOR UPDATE",
        )
        .bind(node_id)
        .bind(token)
        .bind(lease_epoch)
        .fetch_optional(&mut **tx)
        .await?;
        match expires {
            None => {
                crate::telemetry::metrics().incr("lease_refusals");
                Err(ApiError::fencing_refused())
            }
            Some(expires) if expires <= Utc::now() => {
                crate::telemetry::metrics().incr("lease_refusals");
                Err(ApiError::lease_expired())
            }
            Some(_) => Ok(()),
        }
    }

    /// Issue (or rotate) the node's lease: a FRESH fencing token every handshake,
    /// expiry `LEASE_TTL` out, and a bumped lease EPOCH (`.2.2`). The old token
    /// is fenced by the rotation — a stale process's heartbeats/events stop being
    /// accepted the moment a newer handshake lands, and its epoch-bound renewals
    /// match no row. The token is generated IN PostgreSQL (`gen_random_uuid()`)
    /// in the same statement that writes the row — 128 bits of server entropy,
    /// never client-chosen.
    pub async fn issue_lease(
        &self,
        node_id: &str,
        now: DateTime<Utc>,
    ) -> Result<(String, i64, DateTime<Utc>), sqlx::Error> {
        let expires = now + LEASE_TTL;
        let (token, epoch): (String, i64) = sqlx::query_as(
            "INSERT INTO node_leases (node_id, fencing_token, lease_expires_at, last_seen_at, \
                                      issued_at, lease_epoch) \
             VALUES ($1, 'fnc_' || gen_random_uuid()::text, $2, $3, $3, 1) \
             ON CONFLICT (node_id) DO UPDATE SET \
               fencing_token = EXCLUDED.fencing_token, \
               lease_expires_at = EXCLUDED.lease_expires_at, \
               last_seen_at = EXCLUDED.last_seen_at, \
               issued_at = EXCLUDED.issued_at, \
               lease_epoch = node_leases.lease_epoch + 1 \
             RETURNING fencing_token, lease_epoch",
        )
        .bind(node_id)
        .bind(expires)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        Ok((token, epoch, expires))
    }

    /// Renew a LIVE lease (the caller already verified the fencing token): push
    /// `lease_expires_at` and `last_seen_at` forward. The epoch the caller saw
    /// rides the WHERE (`.2.2`): a renewal whose handshake was superseded
    /// between the check and this write matches no row and is refused — a
    /// stale heartbeat can never extend the session that fenced it.
    pub async fn renew_lease(
        &self,
        node_id: &str,
        lease_epoch: i64,
        now: DateTime<Utc>,
    ) -> Result<DateTime<Utc>, sqlx::Error> {
        let expires = now + LEASE_TTL;
        let renewed: Option<DateTime<Utc>> = sqlx::query_scalar(
            "UPDATE node_leases SET lease_expires_at = $3, last_seen_at = $2 \
             WHERE node_id = $1 AND lease_epoch = $4 \
             RETURNING lease_expires_at",
        )
        .bind(node_id)
        .bind(now)
        .bind(expires)
        .bind(lease_epoch)
        .fetch_optional(&self.pool)
        .await?;
        match renewed {
            Some(expiry) => Ok(expiry),
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    /// One node's observable presence (`node_presence`, migration 0009). `None`
    /// when the node is not enrolled.
    pub async fn presence(&self, node_id: &str) -> Result<Option<PresenceResponse>, sqlx::Error> {
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
             FROM node_presence WHERE node_id = $1",
        )
        .bind(node_id)
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
    let revocation_epoch = state.revocation_epoch(&req.node_id).await?;

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
    state.verify_rotate_proof(&req).await?;

    // The durable host claim (the SAN the leaf carries) comes from the node's
    // host row — rotation keeps the same identity binding.
    let host_claim: Option<String> = sqlx::query_scalar(
        "SELECT h.name FROM nodes n JOIN hosts h ON n.host_id = h.host_id \
         WHERE n.node_id = $1",
    )
    .bind(&req.node_id)
    .fetch_optional(&state.pool)
    .await?;
    let Some(host_claim) = host_claim else {
        return Err(ApiError::unknown_node(&req.node_id));
    };

    let (cert_der, key_der) = crate::ca::issue_node_leaf(&state.ca, &req.node_id, &host_claim);
    let cert_fingerprint = crate::ca::cert_fingerprint(&cert_der);
    let now = Utc::now();
    let cert_expires_at = now + ChronoDuration::seconds(crate::ca::LEAF_TTL_SECS);
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
    .execute(&state.pool)
    .await?;

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
        if let Err(e) =
            crate::api::apply_node_result_in_tx(&mut *tx, &req.node_id, &req.payload).await
        {
            crate::telemetry::metrics().incr("results_rejected");
            crate::log_event!("thread_result_rejected", "node_id" => &req.node_id, "reason" => e.to_string());
        }
    }
    tx.commit().await?;
    Ok(Json(EventReceipt {
        channel_version: CHANNEL_VERSION,
        accepted,
    }))
}

async fn ack(
    State(state): State<Arc<NodeChannelState>>,
    Json(req): Json<AckRequest>,
) -> Result<Json<AckResponse>, ApiError> {
    check_version(req.channel_version)?;
    state
        .verify_fencing(&req.node_id, &req.fencing_token, req.lease_epoch)
        .await?;
    let current = state.current_cursor(&req.node_id).await?;
    if req.ack_cursor > current {
        return Err(ApiError::cursor_ahead(req.ack_cursor, current));
    }
    let acknowledged = state
        .acknowledge(&req.node_id, req.ack_cursor, Utc::now())
        .await?;
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
    let revocation_epoch = state.revocation_epoch(&req.node_id).await?;
    Ok(Json(PollResponse {
        channel_version: CHANNEL_VERSION,
        current_cursor: current,
        commands,
        revocation_epoch,
    }))
}

/// Renew a LIVE lease: the fencing token must be the latest handshake's, the
/// epoch must still be current, and the lease must not have expired. Renewal
/// returns the new expiry; the token itself rotates only at the handshake. A
/// renewal whose epoch was superseded between the check and the write matches
/// no row — the stale heartbeat loses the race (`.2.2`), it never extends the
/// session that fenced it.
async fn heartbeat(
    State(state): State<Arc<NodeChannelState>>,
    Json(req): Json<HeartbeatRequest>,
) -> Result<Json<HeartbeatResponse>, ApiError> {
    check_version(req.channel_version)?;
    state
        .verify_fencing(&req.node_id, &req.fencing_token, req.lease_epoch)
        .await?;
    let lease_expires_at = state
        .renew_lease(&req.node_id, req.lease_epoch, Utc::now())
        .await
        .map_err(|_| ApiError::fencing_refused())?;
    Ok(Json(HeartbeatResponse {
        channel_version: CHANNEL_VERSION,
        fencing_token: req.fencing_token,
        lease_expires_at,
    }))
}

/// One node's observable presence. Presence is a read-only observability fact
/// (no token, no key, no fingerprint — never a credential), derived from the
/// lease clock; an unenrolled node is a 404, not a fabricated `offline`.
async fn presence(
    State(state): State<Arc<NodeChannelState>>,
    Query(params): Query<PresenceParams>,
) -> Result<Json<PresenceResponse>, ApiError> {
    if params.node_id.is_empty() {
        return Err(ApiError::bad_request("node_id is required".to_string()));
    }
    match state.presence(&params.node_id).await? {
        Some(p) => Ok(Json(p)),
        None => Err(ApiError::unknown_node(&params.node_id)),
    }
}

// ── Node enrollment (PHASE-1.2.1; backlog 11) ───────────────────────────────────

/// The `POST /v1/nodes/enroll` body: the one-time token (issued by an authorized
/// human at `/v1/nodes/enroll-tokens`), the node's id, the host claim the token was
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

/// One stored enrollment token row (the 0008 table).
#[derive(sqlx::FromRow)]
struct EnrollmentTokenRow {
    tenant_id: String,
    node_id: String,
    host_claim: String,
    nonce: String,
    expires_at: DateTime<Utc>,
    used_at: Option<DateTime<Utc>>,
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
        "SELECT tenant_id, node_id, host_claim, nonce, expires_at, used_at \
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
    let (active_certs, total_certs, node_exists): (i64, i64, bool) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM node_certificates WHERE node_id = $1 AND revoked_at IS NULL), \
                (SELECT count(*) FROM node_certificates WHERE node_id = $1), \
                EXISTS(SELECT 1 FROM nodes WHERE node_id = $1 AND tenant_id = $2)",
    )
    .bind(&req.node_id)
    .bind(&tenant_id)
    .fetch_one(&mut *tx)
    .await?;

    let replacement = if node_exists {
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
    let (cert_der, key_der) = crate::ca::issue_node_leaf(&state.ca, &req.node_id, &req.host_claim);
    let cert_fingerprint = crate::ca::cert_fingerprint(&cert_der);
    let cert_expires_at = now + ChronoDuration::seconds(crate::ca::LEAF_TTL_SECS);
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
    // Re-enrollment cannot duplicate: the `nodes` primary key refuses a second
    // enroll for the id, and rotation never touches this table — one incarnation
    // per (role, enroll), with `valid_from` = now and no `valid_to` yet.
    let incarnation_id: Option<String> = if req
        .node_id
        .parse::<reasonbraid_core::AgentRoleId>()
        .is_ok()
    {
        let id = reasonbraid_core::AgentIncarnationId::new().to_string();
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
