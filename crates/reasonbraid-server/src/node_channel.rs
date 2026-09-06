//! The WP3 node channel, server side (`PHASE-0.3.2`).
//!
//! The control plane owns desired commands and delivery attempts (`ROADMAP.md` §17.1).
//! This module is the server half of the Phase 0 node channel (`§9.3`'s node-channel
//! profile, minimal: HTTP/1 JSON over the loopback dev profile — the authenticated
//! streaming profile arrives with WP5 identity and ADR-006's formal record):
//!
//! - [`NodeChannelState`] — the durable state behind the channel: a per-node inbox with
//!   a monotonic cursor and acknowledgement state (`node_inbox`), and deduplicated
//!   receipts of node-emitted events (`node_events`, keyed on the node-assigned id).
//! - [`node_router`] — the HTTP surface: `handshake` (the reconnect exchange),
//!   `events` (node results with original ids), `ack` (cursor acknowledgement), and
//!   `poll` (the live delivery path after reconciliation).
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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

/// The node channel's wire protocol version. Both sides must agree; a mismatch is a
/// `protocol_incompatible` error, never a silent downgrade.
pub const CHANNEL_VERSION: u32 = 1;

// ── Wire contract ──────────────────────────────────────────────────────────────

/// The reconnect exchange (`§17.4` steps 2–3): the node reports its durable resume
/// facts, the server replies with the replay and the reconciliation guidance.
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
}

/// One ambiguous attempt the node reports for reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AmbiguousAttempt {
    pub attempt_id: String,
    pub operation_id: String,
}

/// One inbox row replayed to the node, in cursor order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReplayCommand {
    pub cursor: i64,
    pub command_id: String,
    pub tenant_id: String,
    pub thread_id: String,
    pub payload: Value,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AckResponse {
    pub channel_version: u32,
    /// Inbox rows this acknowledgement marked (idempotent: 0 on a re-ack).
    pub acknowledged: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PollParams {
    pub node_id: String,
    pub after_cursor: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PollResponse {
    pub channel_version: u32,
    pub current_cursor: i64,
    pub commands: Vec<ReplayCommand>,
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
}

impl NodeChannelState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Append a command to a node's inbox ledger; returns its assigned per-node cursor
    /// (monotonic, starting at 1). Concurrent enqueues to the SAME node collide on the
    /// `(node_id, cursor)` primary key — the dev profile is single-writer; the collision
    /// is a conflict error, never a silent overwrite.
    pub async fn enqueue(
        &self,
        node_id: &str,
        command_id: &str,
        tenant_id: &str,
        thread_id: &str,
        payload: &Value,
    ) -> Result<i64, sqlx::Error> {
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
        .fetch_one(&self.pool)
        .await
    }

    /// The highest cursor in a node's inbox ledger (0 when the node has none).
    pub async fn current_cursor(&self, node_id: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COALESCE(MAX(cursor), 0) FROM node_inbox WHERE node_id = $1")
            .bind(node_id)
            .fetch_one(&self.pool)
            .await
    }

    /// The inbox rows after `after_cursor`, in cursor order — the replay for a node
    /// that reports holding up to `after_cursor`.
    pub async fn replay(
        &self,
        node_id: &str,
        after_cursor: i64,
    ) -> Result<Vec<ReplayCommand>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, String, String, String, Value)>(
            "SELECT cursor, command_id, tenant_id, thread_id, payload FROM node_inbox \
             WHERE node_id = $1 AND cursor > $2 ORDER BY cursor",
        )
        .bind(node_id)
        .bind(after_cursor)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(cursor, command_id, tenant_id, thread_id, payload)| ReplayCommand {
                    cursor,
                    command_id,
                    tenant_id,
                    thread_id,
                    payload,
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
        .execute(&self.pool)
        .await?
        .rows_affected()
            == 1)
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
}

// ── HTTP surface ───────────────────────────────────────────────────────────────

/// The node channel router: `/v1/nodes/handshake` (reconnect exchange), `/v1/nodes/events`
/// (node results), `/v1/nodes/ack` (cursor acknowledgement), `/v1/nodes/poll` (live tail).
pub fn node_router(pool: PgPool) -> Router {
    let state = Arc::new(NodeChannelState::new(pool));
    Router::new()
        .route("/v1/nodes/handshake", post(handshake))
        .route("/v1/nodes/events", post(events))
        .route("/v1/nodes/ack", post(ack))
        .route("/v1/nodes/poll", get(poll))
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

    Ok(Json(HandshakeResponse {
        channel_version: CHANNEL_VERSION,
        current_cursor: current,
        replay,
        directives,
        known_events,
    }))
}

async fn events(
    State(state): State<Arc<NodeChannelState>>,
    Json(req): Json<EventSubmission>,
) -> Result<Json<EventReceipt>, ApiError> {
    check_version(req.channel_version)?;
    let accepted = state
        .record_event(
            &req.node_id,
            &req.event_id,
            &req.operation_id,
            &req.payload,
            Utc::now(),
        )
        .await?;
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
    Query(params): Query<PollParams>,
) -> Result<Json<PollResponse>, ApiError> {
    if params.node_id.is_empty() {
        return Err(ApiError::bad_request("node_id is required".to_string()));
    }
    let current = state.current_cursor(&params.node_id).await?;
    if params.after_cursor > current {
        return Err(ApiError::cursor_ahead(params.after_cursor, current));
    }
    let commands = state.replay(&params.node_id, params.after_cursor).await?;
    Ok(Json(PollResponse {
        channel_version: CHANNEL_VERSION,
        current_cursor: current,
        commands,
    }))
}
