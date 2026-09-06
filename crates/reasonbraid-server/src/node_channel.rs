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
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres};

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
        let mut conn = self.pool.acquire().await?;
        enqueue_in_tx(
            &mut *conn, node_id, command_id, tenant_id, thread_id, payload,
        )
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
}

// ── Transactional bodies (`PHASE-0.6.2`) ────────────────────────────────────────

/// The transactional body of [`NodeChannelState::enqueue`]: the `.6.1` command
/// transaction enqueues an invite/challenge work item in the SAME transaction as
/// the thread event that produced it — an invitation exists iff its inbox row does.
pub(crate) async fn enqueue_in_tx<'e, E>(
    mut tx: E,
    node_id: &str,
    command_id: &str,
    tenant_id: &str,
    thread_id: &str,
    payload: &Value,
) -> Result<i64, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
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

/// The node channel router: `/v1/nodes/handshake` (reconnect exchange), `/v1/nodes/events`
/// (node results), `/v1/nodes/ack` (cursor acknowledgement), `/v1/nodes/poll` (live tail),
/// `/v1/nodes/enroll` (the `.1.2.1` one-time-token enrollment — the token IS the
/// credential, so this surface carries no principal header).
pub fn node_router(pool: PgPool) -> Router {
    let state = Arc::new(NodeChannelState::new(pool));
    Router::new()
        .route("/v1/nodes/handshake", post(handshake))
        .route("/v1/nodes/events", post(events))
        .route("/v1/nodes/ack", post(ack))
        .route("/v1/nodes/poll", get(poll))
        .route("/v1/nodes/enroll", post(enroll))
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
    // ONE transaction (`PHASE-0.6.2`): the receipt and — when the payload is a
    // thread work result — the domain application (claim → authorize → validate →
    // apply + reservation settlement) commit together. A duplicate event inserts
    // no receipt and applies no domain effect (one effect, ever); a rejected
    // application still commits the receipt (the node DID emit this event) with
    // the rejection stored as the command's idempotent result.
    let mut tx = state.pool.begin().await?;
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
            eprintln!(
                "node channel: thread result from {} was rejected: {e}",
                req.node_id
            );
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

// ── Node enrollment (PHASE-1.2.1; backlog 11) ───────────────────────────────────

/// The `POST /v1/nodes/enroll` body: the one-time token (issued by an authorized
/// human at `/v1/nodes/enroll-tokens`), the node's id, the host claim the token was
/// bound to, the token nonce, and the node's dev signing secret (the server IS the
/// dev trust store — the `.6.1` stance; the HMAC key-proof rides `.1.2.2`'s handshake).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeEnrollRequest {
    pub token_id: String,
    pub node_id: String,
    pub host_claim: String,
    pub nonce: String,
    pub key_secret: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeEnrollResponse {
    pub node_id: String,
    pub host_id: String,
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
    if req.node_id.parse::<reasonbraid_core::NodeId>().is_err() {
        return Err(ApiError::bad_request(format!(
            "node_id `{}` is not a valid node identifier",
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
    let node_insert =
        sqlx::query("INSERT INTO nodes (node_id, host_id, tenant_id) VALUES ($1, $2, $3)")
            .bind(&req.node_id)
            .bind(&host_id)
            .bind(&tenant_id)
            .execute(&mut *tx)
            .await;
    match node_insert {
        Ok(_) => {}
        Err(e)
            if e.as_database_error()
                .is_some_and(|d| d.is_unique_violation()) =>
        {
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
        Err(e) => return Err(e.into()),
    }

    sqlx::query("INSERT INTO node_keys (node_id, key_fingerprint, key_secret) VALUES ($1, $2, $3)")
        .bind(&req.node_id)
        .bind(&fingerprint)
        .bind(&req.key_secret)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE node_enrollment_tokens SET used_at = $2 WHERE token_id = $1")
        .bind(&req.token_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO node_enroll_audit (record_id, tenant_id, node_id, token_id, decision, reason) \
         VALUES ('naux_' || gen_random_uuid()::text, $1, $2, $3, 'enrolled', NULL)",
    )
    .bind(&tenant_id)
    .bind(&req.node_id)
    .bind(&req.token_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(NodeEnrollResponse {
        node_id: req.node_id,
        host_id,
    }))
}
