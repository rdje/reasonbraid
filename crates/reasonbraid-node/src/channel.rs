//! The node channel, client side (`PHASE-0.3.2`; authenticated by `PHASE-1.2.2`):
//! the node's OUTBOUND connection to the control plane (`ROADMAP.md` §4/§9.3 —
//! nodes initiate connections; ordinary deployments expose no inbound ports).
//!
//! Phase 1 speaks HTTP/1 JSON over the loopback development profile with the
//! `.1.2.2` authentication layer: the handshake proves possession of the node's
//! dev secret (HMAC-SHA256 over the channel fields), the server answers with a
//! lease + fencing token, and every later message (events, ack, poll, heartbeat)
//! carries that token. The authenticated streaming profile (§9.3: HTTP/2 or gRPC,
//! mTLS workload identity) arrives with ADR-006/ADR-007's formal records; the
//! protocol semantics here — cursor-based replay, original-id re-emission,
//! reconciliation directives, lease/fencing — are transport-neutral.
//!
//! The wire types mirror `reasonbraid-server`'s channel DTOs (`src/node_channel.rs`)
//! one-for-one. The duplication is deliberate: a shared wire crate
//! (`reasonbraid-protocol`) is a later roadmap step, and until then each side keeps its
//! own `deny_unknown_fields` contract so a forged or stale field is rejected at the
//! boundary, on both sides.

use std::fmt;
use std::sync::{Arc, Mutex};

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// The node channel's wire protocol version (must equal the server's). Version 2
/// (`.1.2.2`) is the authenticated contract.
pub const CHANNEL_VERSION: u32 = 2;

/// The exact fields the handshake key-proof covers, in canonical (serde field)
/// order — the mirrored [`ProofCoverage`] shape the server verifies. Both sides
/// serialize THIS struct to JSON and HMAC it with the node's dev secret.
#[derive(Debug, Serialize)]
pub struct ProofCoverage<'a> {
    pub channel_version: u32,
    pub node_id: &'a str,
    pub last_acked_cursor: i64,
    pub pending_operations: &'a [String],
    pub ambiguous_attempts: &'a [AmbiguousAttempt],
}

/// Compute the handshake key-proof: lowercase-hex HMAC-SHA256 over the canonical
/// coverage JSON, keyed with the node's dev secret. PUBLIC so server-side
/// integration tests compute proofs with the same canonicalization the real node
/// uses — a cross-side mismatch surfaces as a test failure, not a silent drift.
pub fn compute_key_proof(
    channel_version: u32,
    node_id: &str,
    last_acked_cursor: i64,
    pending_operations: &[String],
    ambiguous_attempts: &[AmbiguousAttempt],
    key_secret: &str,
) -> String {
    let coverage = ProofCoverage {
        channel_version,
        node_id,
        last_acked_cursor,
        pending_operations,
        ambiguous_attempts,
    };
    let canonical = serde_json::to_vec(&coverage).expect("the coverage shape is encodable");
    let mut mac =
        HmacSha256::new_from_slice(key_secret.as_bytes()).expect("any key length is valid");
    mac.update(&canonical);
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The reconnect exchange (`ROADMAP.md` §17.4 step 2): the node reports its durable
/// resume facts — last acknowledged server cursor, pending local operation ids, the
/// attempts its recovery classified `outcome_unknown` — and proves possession of
/// its dev key; the server answers with the replay, the reconciliation guidance,
/// and a fresh lease.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HandshakeRequest {
    pub channel_version: u32,
    pub node_id: String,
    pub last_acked_cursor: i64,
    pub pending_operations: Vec<String>,
    pub ambiguous_attempts: Vec<AmbiguousAttempt>,
    /// HMAC-SHA256 over the other fields (see [`ProofCoverage`]), hex-encoded,
    /// keyed with the node's dev secret.
    pub key_proof: String,
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

/// The server's answer to the handshake: the replay tail plus reconciliation
/// guidance plus the fresh lease (fencing token + expiry).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HandshakeResponse {
    pub channel_version: u32,
    pub current_cursor: i64,
    pub replay: Vec<ReplayCommand>,
    pub directives: Vec<Directive>,
    pub known_events: Vec<KnownEvent>,
    pub fencing_token: String,
    pub lease_expires_at: chrono::DateTime<chrono::Utc>,
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

/// The live delivery tail (`POST /v1/nodes/poll`) for a schedulable node. A POST:
/// the fencing token is a credential and never rides a query string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PollResponse {
    pub channel_version: u32,
    pub current_cursor: i64,
    pub commands: Vec<ReplayCommand>,
}

/// The lease renewal (`POST /v1/nodes/heartbeat`): a heartbeat extends a LIVE
/// lease. A refused heartbeat (fenced or expired) means re-handshake.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HeartbeatResponse {
    pub channel_version: u32,
    pub fencing_token: String,
    pub lease_expires_at: chrono::DateTime<chrono::Utc>,
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
    /// The channel is not authenticated yet: the caller must handshake (and the
    /// node lifecycle makes that impossible outside `reconcile`).
    NotAuthenticated,
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
            ChannelError::NotAuthenticated => {
                write!(f, "the node channel is not authenticated — handshake first")
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
///
/// The client keeps the fencing token from the latest successful handshake in
/// shared state: every clone of a [`NodeChannel`] sees the same lease (the worker
/// polls with it, the heartbeat task renews it, `reconcile` rotates it).
#[derive(Debug, Clone)]
pub struct NodeChannel {
    base_url: String,
    node_id: String,
    key_secret: String,
    fencing_token: Arc<Mutex<Option<String>>>,
    client: reqwest::Client,
}

impl NodeChannel {
    /// Build the client for `base_url` (e.g. `http://127.0.0.1:8080`) acting as
    /// `node_id`, proving possession of the node's dev signing secret
    /// (`--node-secret`, the `.1.2.1` enrollment key).
    pub fn new(base_url: impl Into<String>, node_id: String, key_secret: String) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            node_id,
            key_secret,
            fencing_token: Arc::new(Mutex::new(None)),
            client: reqwest::Client::new(),
        }
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// The fencing token from the latest successful handshake, or a typed error
    /// when the channel has not been authenticated (or was reset).
    fn current_fencing_token(&self) -> Result<String, ChannelError> {
        self.fencing_token
            .lock()
            .expect("the fencing-token lock is not poisoned")
            .clone()
            .ok_or(ChannelError::NotAuthenticated)
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

    /// The authenticated reconnect exchange: report the node's durable resume
    /// facts WITH a key-proof over them, and receive the replay tail, the
    /// reconciliation guidance, and a fresh lease. On success the returned
    /// fencing token becomes the channel's credential for events/ack/poll/
    /// heartbeat — until the next handshake rotates it.
    pub async fn handshake(
        &self,
        req: &HandshakeRequest,
    ) -> Result<HandshakeResponse, ChannelError> {
        let proof = compute_key_proof(
            req.channel_version,
            &req.node_id,
            req.last_acked_cursor,
            &req.pending_operations,
            &req.ambiguous_attempts,
            &self.key_secret,
        );
        let mut with_proof = req.clone();
        with_proof.key_proof = proof;
        let response = self
            .client
            .post(format!("{}/v1/nodes/handshake", self.base_url))
            .json(&with_proof)
            .send()
            .await?;
        let parsed: HandshakeResponse = self.parse(response).await?;
        *self
            .fencing_token
            .lock()
            .expect("the fencing-token lock is not poisoned") = Some(parsed.fencing_token.clone());
        Ok(parsed)
    }

    /// Submit one node-emitted event with its ORIGINAL id (redelivery-safe),
    /// carrying the current fencing token.
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
                "fencing_token": self.current_fencing_token()?,
            }))
            .send()
            .await?;
        self.parse(response).await
    }

    /// Acknowledge that the node durably holds commands up to `ack_cursor`,
    /// carrying the current fencing token.
    pub async fn acknowledge(&self, ack_cursor: i64) -> Result<AckResponse, ChannelError> {
        let response = self
            .client
            .post(format!("{}/v1/nodes/ack", self.base_url))
            .json(&serde_json::json!({
                "channel_version": CHANNEL_VERSION,
                "node_id": self.node_id,
                "ack_cursor": ack_cursor,
                "fencing_token": self.current_fencing_token()?,
            }))
            .send()
            .await?;
        self.parse(response).await
    }

    /// The live delivery tail after `after_cursor` (the schedulable node's poll
    /// path), carrying the current fencing token.
    pub async fn poll(&self, after_cursor: i64) -> Result<PollResponse, ChannelError> {
        let response = self
            .client
            .post(format!("{}/v1/nodes/poll", self.base_url))
            .json(&serde_json::json!({
                "channel_version": CHANNEL_VERSION,
                "node_id": self.node_id,
                "after_cursor": after_cursor,
                "fencing_token": self.current_fencing_token()?,
            }))
            .send()
            .await?;
        self.parse(response).await
    }

    /// Renew the server-side lease: the current fencing token must still be the
    /// live lease's. A refusal (fenced by a newer handshake, or expired) means
    /// the caller should re-handshake — the node lifecycle does exactly that.
    pub async fn heartbeat(&self) -> Result<HeartbeatResponse, ChannelError> {
        let response = self
            .client
            .post(format!("{}/v1/nodes/heartbeat", self.base_url))
            .json(&serde_json::json!({
                "channel_version": CHANNEL_VERSION,
                "node_id": self.node_id,
                "fencing_token": self.current_fencing_token()?,
            }))
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
