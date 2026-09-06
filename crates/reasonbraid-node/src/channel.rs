//! The WP3 node channel, client side (`PHASE-0.3.2`): the node's OUTBOUND connection
//! to the control plane (`ROADMAP.md` §4/§9.3 — nodes initiate connections; ordinary
//! deployments expose no inbound ports).
//!
//! Phase 0 speaks HTTP/1 JSON over the loopback development profile. The authenticated
//! streaming profile (§9.3: HTTP/2 or gRPC, mTLS workload identity) arrives with WP5
//! identity and ADR-006's formal record; the protocol semantics here — cursor-based
//! replay, original-id re-emission, reconciliation directives — are transport-neutral.
//!
//! The wire types mirror `reasonbraid-server`'s channel DTOs (`src/node_channel.rs`)
//! one-for-one. The duplication is deliberate: a shared wire crate
//! (`reasonbraid-protocol`) is a later roadmap step, and until then each side keeps its
//! own `deny_unknown_fields` contract so a forged or stale field is rejected at the
//! boundary, on both sides.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The node channel's wire protocol version (must equal the server's).
pub const CHANNEL_VERSION: u32 = 1;

/// The reconnect exchange (`ROADMAP.md` §17.4 step 2): the node reports its durable
/// resume facts — last acknowledged server cursor, pending local operation ids, and
/// the attempts its recovery classified `outcome_unknown`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HandshakeRequest {
    pub channel_version: u32,
    pub node_id: String,
    pub last_acked_cursor: i64,
    pub pending_operations: Vec<String>,
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
    /// The server holds a receipt for this operation's event: mark the attempt
    /// `reconciled` with the receipt as the evidence.
    Adjudicated {
        attempt_id: String,
        terminal: String,
        evidence: String,
    },
    /// The server holds no receipt and cannot decide: the attempt stays
    /// `outcome_unknown` until a provider proof or an operator adjudicates it.
    NeedsAdjudication { attempt_id: String, reason: String },
}

/// A server-held receipt for one of the node's pending operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct KnownEvent {
    pub operation_id: String,
    pub event_id: String,
}

/// The server's answer to the handshake: the replay tail plus reconciliation guidance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HandshakeResponse {
    pub channel_version: u32,
    pub current_cursor: i64,
    pub replay: Vec<ReplayCommand>,
    pub directives: Vec<Directive>,
    pub known_events: Vec<KnownEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventReceipt {
    pub channel_version: u32,
    /// `false` when the server already holds this event id (a redelivery).
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AckResponse {
    pub channel_version: u32,
    pub acknowledged: i64,
}

/// The live delivery tail (`GET /v1/nodes/poll`) for a schedulable node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PollResponse {
    pub channel_version: u32,
    pub current_cursor: i64,
    pub commands: Vec<ReplayCommand>,
}

/// A typed channel error.
#[derive(Debug)]
pub enum ChannelError {
    /// The transport failed (connection refused, timeout, body truncated, …).
    Http(reqwest::Error),
    /// The server answered with a non-success status and a machine-readable error body.
    Server {
        status: u16,
        code: String,
        message: String,
    },
    /// The server answered 2xx but the body was not the expected shape.
    Malformed(String),
}

impl fmt::Display for ChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelError::Http(e) => write!(f, "node channel transport error: {e}"),
            ChannelError::Server {
                status,
                code,
                message,
            } => write!(f, "node channel server error ({status} {code}): {message}"),
            ChannelError::Malformed(detail) => {
                write!(f, "node channel returned a malformed response: {detail}")
            }
        }
    }
}

impl std::error::Error for ChannelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ChannelError::Http(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for ChannelError {
    fn from(e: reqwest::Error) -> Self {
        ChannelError::Http(e)
    }
}

/// The node's outbound connection to one control-plane channel endpoint.
#[derive(Debug, Clone)]
pub struct NodeChannel {
    base_url: String,
    node_id: String,
    client: reqwest::Client,
}

impl NodeChannel {
    /// Build the client for `base_url` (e.g. `http://127.0.0.1:8080`) acting as `node_id`.
    pub fn new(base_url: impl Into<String>, node_id: String) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            node_id,
            client: reqwest::Client::new(),
        }
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    fn parse_error(status: u16, body: &[u8]) -> ChannelError {
        #[derive(Deserialize)]
        struct ErrorBody {
            code: String,
            message: String,
        }
        match serde_json::from_slice::<ErrorBody>(body) {
            Ok(e) => ChannelError::Server {
                status,
                code: e.code,
                message: e.message,
            },
            Err(_) => ChannelError::Server {
                status,
                code: "unknown".to_string(),
                message: String::from_utf8_lossy(body).into_owned(),
            },
        }
    }

    async fn parse<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, ChannelError> {
        let status = response.status().as_u16();
        let body = response.bytes().await?;
        if !(200..300).contains(&status) {
            return Err(Self::parse_error(status, &body));
        }
        serde_json::from_slice(&body).map_err(|e| ChannelError::Malformed(e.to_string()))
    }

    /// The reconnect exchange: report the node's durable resume facts and receive the
    /// replay tail plus reconciliation guidance.
    pub async fn handshake(
        &self,
        req: &HandshakeRequest,
    ) -> Result<HandshakeResponse, ChannelError> {
        let response = self
            .client
            .post(format!("{}/v1/nodes/handshake", self.base_url))
            .json(req)
            .send()
            .await?;
        self.parse(response).await
    }

    /// Submit one node-emitted event with its ORIGINAL id (redelivery-safe).
    pub async fn send_event(
        &self,
        event_id: &str,
        operation_id: &str,
        payload: &Value,
    ) -> Result<EventReceipt, ChannelError> {
        let response = self
            .client
            .post(format!("{}/v1/nodes/events", self.base_url))
            .json(&serde_json::json!({
                "channel_version": CHANNEL_VERSION,
                "node_id": self.node_id,
                "event_id": event_id,
                "operation_id": operation_id,
                "payload": payload,
            }))
            .send()
            .await?;
        self.parse(response).await
    }

    /// Acknowledge that the node durably holds commands up to `ack_cursor`.
    pub async fn acknowledge(&self, ack_cursor: i64) -> Result<AckResponse, ChannelError> {
        let response = self
            .client
            .post(format!("{}/v1/nodes/ack", self.base_url))
            .json(&serde_json::json!({
                "channel_version": CHANNEL_VERSION,
                "node_id": self.node_id,
                "ack_cursor": ack_cursor,
            }))
            .send()
            .await?;
        self.parse(response).await
    }

    /// The live delivery tail after `after_cursor` (the schedulable node's poll path).
    pub async fn poll(&self, after_cursor: i64) -> Result<PollResponse, ChannelError> {
        let response = self
            .client
            .get(format!("{}/v1/nodes/poll", self.base_url))
            .query(&[
                ("node_id", self.node_id.as_str()),
                ("after_cursor", &after_cursor.to_string()),
            ])
            .send()
            .await?;
        self.parse(response).await
    }

    /// Consume a one-time enrollment token (`PHASE-1.2.1`): register this node +
    /// its dev signing secret with the control plane. The token IS the credential —
    /// this call carries no principal header. Returns the enrollment response JSON.
    pub async fn enroll(
        &self,
        token_id: &str,
        node_id: &str,
        host_claim: &str,
        nonce: &str,
        key_secret: &str,
    ) -> Result<serde_json::Value, ChannelError> {
        let response = self
            .client
            .post(format!("{}/v1/nodes/enroll", self.base_url))
            .json(&serde_json::json!({
                "token_id": token_id,
                "node_id": node_id,
                "host_claim": host_claim,
                "nonce": nonce,
                "key_secret": key_secret,
            }))
            .send()
            .await?;
        self.parse(response).await
    }
}
