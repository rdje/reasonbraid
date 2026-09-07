//! The node channel, client side (`PHASE-0.3.2`; authenticated by `PHASE-1.2.2`,
//! certificate-proofed by `PHASE-2.1.2.2`):
//! the node's OUTBOUND connection to the control plane (`ROADMAP.md` §4/§9.3 —
//! nodes initiate connections; ordinary deployments expose no inbound ports).
//!
//! The node speaks HTTP/1 JSON over the loopback development profile with the
//! certificate-proof authentication layer (`.1.2.2`, ADR-007): the handshake
//! proves possession of the workload certificate's private key (an ECDSA
//! signature over the canonical channel fields), the server chains the leaf to
//! its CA, checks the node-id → fingerprint binding, and answers with a lease +
//! fencing token; every later message (events, ack, poll, heartbeat) carries
//! that token. The mTLS streaming transport (§9.3: HTTP/2 or gRPC) is a later
//! hardening; the protocol semantics here — cursor-based replay, original-id
//! re-emission, reconciliation directives, lease/fencing — are transport-neutral.
//!
//! The wire types mirror `reasonbraid-server`'s channel DTOs (`src/node_channel.rs`)
//! one-for-one. The duplication is deliberate: a shared wire crate
//! (`reasonbraid-protocol`) is a later roadmap step, and until then each side keeps its
//! own `deny_unknown_fields` contract so a forged or stale field is rejected at the
//! boundary, on both sides.

use std::fmt;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The node channel's wire protocol version (must equal the server's). Version 3
/// (`.1.2.2`) is the certificate-proof contract.
pub const CHANNEL_VERSION: u32 = 4;

/// The exact fields the handshake proof covers, in canonical (serde field)
/// order — the mirrored [`ProofCoverage`] shape the server verifies. Both sides
/// serialize THIS struct to JSON; the node signs it with its workload
/// certificate's private key.
#[derive(Debug, Serialize)]
pub struct ProofCoverage<'a> {
    pub channel_version: u32,
    pub node_id: &'a str,
    pub last_acked_cursor: i64,
    pub pending_operations: &'a [String],
    pub ambiguous_attempts: &'a [AmbiguousAttempt],
}

/// Compute the handshake proof: a hex-encoded ECDSA P-256 signature over the
/// canonical coverage JSON, made with the workload certificate's private key.
/// PUBLIC so server-side integration tests can compute proofs with the same
/// canonicalization the real node uses — a cross-side mismatch surfaces as a
/// test failure, not a silent drift.
pub fn compute_cert_proof(
    key: &rcgen::KeyPair,
    channel_version: u32,
    node_id: &str,
    last_acked_cursor: i64,
    pending_operations: &[String],
    ambiguous_attempts: &[AmbiguousAttempt],
) -> String {
    use rcgen::SigningKey;
    let coverage = ProofCoverage {
        channel_version,
        node_id,
        last_acked_cursor,
        pending_operations,
        ambiguous_attempts,
    };
    let canonical = serde_json::to_vec(&coverage).expect("the coverage shape is encodable");
    to_hex(&key.sign(&canonical).expect("the workload key signs"))
}

/// The rotate exchange: the node presents its CURRENT certificate (the proof
/// covers the rotate coverage) and receives a fresh key + certificate.
#[derive(Debug, Serialize)]
pub struct RotateCoverage<'a> {
    pub channel_version: u32,
    pub node_id: &'a str,
    pub cert_der: &'a str,
}

/// The reconnect exchange (`ROADMAP.md` §17.4 step 2): the node reports its durable
/// resume facts — last acknowledged server cursor, pending local operation ids, the
/// attempts its recovery classified `outcome_unknown` — and proves possession of
/// its workload certificate's private key; the server answers with the replay, the
/// reconciliation guidance, and a fresh lease.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HandshakeRequest {
    pub channel_version: u32,
    pub node_id: String,
    pub last_acked_cursor: i64,
    pub pending_operations: Vec<String>,
    pub ambiguous_attempts: Vec<AmbiguousAttempt>,
    /// The node's workload certificate (hex DER) — the leaf the `.1.2.1`
    /// enrollment issued (or `.1.2.2` rotation refreshed).
    pub cert_der: String,
    /// ECDSA P-256 signature over the canonical coverage (see
    /// [`ProofCoverage`]), hex-encoded, made with the certificate's private key.
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RotateResponse {
    pub node_id: String,
    pub cert_der: String,
    pub key_der: String,
    pub cert_fingerprint: String,
    pub cert_expires_at: chrono::DateTime<chrono::Utc>,
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
    /// The admitting authorization record id (`.1.5.2`, ADR-008) — `None` for
    /// rows enqueued before migration 0013 or plain channel traffic.
    pub authz_ref: Option<String>,
    /// The policy digest the record bound.
    pub policy_digest: Option<String>,
    /// The decision time — the freshness TTL runs from it.
    pub decided_at: Option<chrono::DateTime<chrono::Utc>>,
    /// The tenant's revocation epoch AT DECISION TIME.
    pub revocation_epoch: Option<i64>,
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
    /// The tenant's CURRENT revocation epoch (`.1.5.2`, ADR-008).
    pub revocation_epoch: i64,
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
    /// The tenant's CURRENT revocation epoch (`.1.5.2`, ADR-008) — the node
    /// stores it and evaluates every cached admission decision against it at
    /// the dispatch boundary.
    pub revocation_epoch: i64,
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
    /// The channel has no workload identity: enroll first (the one-time token
    /// is the credential) and install the issued certificate.
    NotEnrolled,
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
            ChannelError::NotEnrolled => {
                write!(f, "the node has no workload certificate — enroll first")
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
/// polls with it, the heartbeat task renews it, `reconcile` rotates it). The
/// workload identity (certificate + key) is also shared: a rotation installs
/// the fresh pair and the next handshake signs with it.
#[derive(Debug, Clone)]
pub struct NodeChannel {
    base_url: String,
    node_id: String,
    identity: WorkloadIdentity,
    fencing_token: Arc<Mutex<Option<String>>>,
    client: reqwest::Client,
}

/// A workload leaf stays valid 10 minutes (ADR-007); rotate when less than
/// half of that remains.
const ROTATE_REMAINING_SECS: i64 = 300;

/// The installed workload identity: the certificate (DER) + its key. Shared so
/// a rotation installs the fresh pair and the next handshake signs with it.
type WorkloadIdentity = Arc<Mutex<Option<(Vec<u8>, rcgen::KeyPair)>>>;

impl NodeChannel {
    /// Build the client for `base_url` (e.g. `http://127.0.0.1:8080`) acting as
    /// `node_id`, proving possession of the workload certificate's key (the
    /// `.1.2.1` enrollment's leaf, or a `.1.2.2` rotation's refresh).
    pub fn new(
        base_url: impl Into<String>,
        node_id: String,
        cert_der: Vec<u8>,
        key: rcgen::KeyPair,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            node_id,
            identity: Arc::new(Mutex::new(Some((cert_der, key)))),
            fencing_token: Arc::new(Mutex::new(None)),
            client: reqwest::Client::new(),
        }
    }

    /// Build a client WITHOUT a workload identity: only the enrollment call is
    /// possible (the one-time token is its credential). The caller installs the
    /// identity from the enroll response before any handshake.
    pub fn for_enrollment(base_url: impl Into<String>, node_id: String) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            node_id,
            identity: Arc::new(Mutex::new(None)),
            fencing_token: Arc::new(Mutex::new(None)),
            client: reqwest::Client::new(),
        }
    }

    /// Install a fresh workload identity (the enrollment response's or a
    /// rotation's certificate + key).
    pub fn install_identity(&self, cert_der: Vec<u8>, key: rcgen::KeyPair) {
        *self
            .identity
            .lock()
            .expect("the identity lock is not poisoned") = Some((cert_der, key));
    }

    /// Does the current leaf have ≤ half its lifetime left? (The rotate
    /// trigger — a leaf with no identity reports `false`: enrollment installs
    /// one before any handshake.)
    pub fn cert_expires_soon(&self) -> bool {
        let identity = self
            .identity
            .lock()
            .expect("the identity lock is not poisoned");
        let Some((cert_der, _)) = identity.as_ref() else {
            return false;
        };
        let Ok((_, x509)) = x509_parser::parse_x509_certificate(cert_der) else {
            return false;
        };
        let not_after = x509.validity().not_after.timestamp();
        let now = chrono::Utc::now().timestamp();
        not_after - now <= ROTATE_REMAINING_SECS
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
    /// facts WITH a certificate proof over them, and receive the replay tail,
    /// the reconciliation guidance, and a fresh lease. On success the returned
    /// fencing token becomes the channel's credential for events/ack/poll/
    /// heartbeat — until the next handshake rotates it. If the workload leaf
    /// is within its rotation window, the client rotates FIRST (the fresh
    /// identity signs this handshake).
    pub async fn handshake(
        &self,
        req: &HandshakeRequest,
    ) -> Result<HandshakeResponse, ChannelError> {
        if self.cert_expires_soon() {
            let (cert_der, key) = self.rotate().await?;
            self.install_identity(cert_der, key);
        }
        let (cert_hex, proof) = {
            let identity = self
                .identity
                .lock()
                .expect("the identity lock is not poisoned");
            let Some((cert_der, key)) = identity.as_ref() else {
                return Err(ChannelError::NotEnrolled);
            };
            (
                to_hex(cert_der),
                compute_cert_proof(
                    key,
                    req.channel_version,
                    &req.node_id,
                    req.last_acked_cursor,
                    &req.pending_operations,
                    &req.ambiguous_attempts,
                ),
            )
        };
        let mut with_proof = req.clone();
        with_proof.cert_der = cert_hex;
        with_proof.proof_signature = proof;
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

    /// Rotate the workload certificate: prove possession of the CURRENT leaf
    /// over the rotate coverage and receive a FRESH key + certificate for the
    /// same node id. The caller installs the returned pair (the running
    /// session's fencing token stays valid — rotation is additive).
    pub async fn rotate(&self) -> Result<(Vec<u8>, rcgen::KeyPair), ChannelError> {
        use rcgen::SigningKey;
        let (cert_hex, proof) = {
            let identity = self
                .identity
                .lock()
                .expect("the identity lock is not poisoned");
            let Some((cert_der, key)) = identity.as_ref() else {
                return Err(ChannelError::NotEnrolled);
            };
            let cert_hex = to_hex(cert_der);
            let coverage = RotateCoverage {
                channel_version: CHANNEL_VERSION,
                node_id: &self.node_id,
                cert_der: &cert_hex,
            };
            let canonical =
                serde_json::to_vec(&coverage).expect("the rotate coverage is encodable");
            (
                cert_hex,
                to_hex(&key.sign(&canonical).expect("the workload key signs")),
            )
        };
        let response = self
            .client
            .post(format!("{}/v1/nodes/rotate", self.base_url))
            .json(&serde_json::json!({
                "channel_version": CHANNEL_VERSION,
                "node_id": self.node_id,
                "cert_der": cert_hex,
                "proof_signature": proof,
            }))
            .send()
            .await?;
        let parsed: RotateResponse = self.parse(response).await?;
        let cert_der = from_hex(&parsed.cert_der).map_err(ChannelError::Malformed)?;
        let key = from_hex(&parsed.key_der).map_err(ChannelError::Malformed)?;
        let key = rustls_pki_types::PrivateKeyDer::try_from(key)
            .map_err(|e| ChannelError::Malformed(e.to_string()))?;
        let key = rcgen::KeyPair::from_der_and_sign_algo(&key, &rcgen::PKCS_ECDSA_P256_SHA256)
            .map_err(|e| ChannelError::Malformed(e.to_string()))?;
        Ok((cert_der, key))
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

/// Lowercase hex (the wire ships DER as hex — the codebase hand-rolls hex).
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn from_hex(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("odd-length hex".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}
