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
pub const CHANNEL_VERSION: u32 = 5;

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
    /// `SIGNOFF-REPAIR.4.2.2`: fresh per request, and inside the signature, so a
    /// captured handshake cannot be replayed to take this node's lease.
    pub nonce: &'a str,
}

/// A fresh nonce for one proof (`SIGNOFF-REPAIR.4.2.2`). Generated per REQUEST,
/// not per session: a node whose handshake response was lost retries with a new
/// value, so the retry is admitted and only a byte-identical replay is refused.
pub fn fresh_proof_nonce() -> String {
    // v7 like the rest of this project's ids. Uniqueness is what the defence
    // needs, not unpredictability: a nonce is consumed only AFTER the signature
    // verifies, so no unauthenticated caller can burn one a node was about to use.
    uuid::Uuid::now_v7().to_string()
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
    nonce: &str,
) -> String {
    use rcgen::SigningKey;
    let coverage = ProofCoverage {
        channel_version,
        node_id,
        last_acked_cursor,
        pending_operations,
        ambiguous_attempts,
        nonce,
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
    /// `SIGNOFF-REPAIR.4.2.2`: fresh per request, so a captured rotate cannot be
    /// replayed for another private key.
    pub nonce: &'a str,
}

/// Compute the ROTATE proof: a hex-encoded ECDSA P-256 signature over the
/// canonical rotate coverage, made with the current certificate's private key.
///
/// PUBLIC for the same reason `compute_cert_proof` is — so server-side
/// integration tests compute proofs with the same canonicalization the real node
/// uses, and a cross-side mismatch surfaces as a test failure rather than silent
/// drift. `SIGNOFF-REPAIR.4.2.2` needs it to replay a captured rotate BYTE FOR
/// BYTE, which is the only way to measure what replaying one actually does.
pub fn compute_rotate_proof(
    key: &rcgen::KeyPair,
    channel_version: u32,
    node_id: &str,
    cert_hex: &str,
    nonce: &str,
) -> String {
    use rcgen::SigningKey;
    let coverage = RotateCoverage {
        channel_version,
        node_id,
        cert_der: cert_hex,
        nonce,
    };
    let canonical = serde_json::to_vec(&coverage).expect("the rotate coverage is encodable");
    to_hex(&key.sign(&canonical).expect("the workload key signs"))
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
    /// `SIGNOFF-REPAIR.4.2.2`: filled in by the channel, like `cert_der` and
    /// `proof_signature` — a caller building this request leaves it empty.
    #[serde(default)]
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
    /// The lease EPOCH this handshake's token was issued under (`.2.2`).
    pub lease_epoch: i64,
    /// The tenant's CURRENT revocation epoch (`.1.5.2`, ADR-008).
    pub revocation_epoch: i64,
    /// The server's own clock when it answered (`SIGNOFF-REPAIR.3.4.3.1.2`).
    pub server_time: chrono::DateTime<chrono::Utc>,
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
    /// The server's own clock when it answered (`SIGNOFF-REPAIR.3.4.3.1.2`).
    pub server_time: chrono::DateTime<chrono::Utc>,
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
    /// The lease epoch the current token was issued under (`.2.2`) — sent with
    /// every fenced write so a stale session can never ride a rotated lease.
    lease_epoch: Arc<Mutex<Option<i64>>>,
    /// How far the SERVER's clock is ahead of this node's, in milliseconds,
    /// from the latest handshake (`SIGNOFF-REPAIR.3.4.3.1.3`). The channel keeps
    /// its own copy because the rotation decision runs INSIDE `handshake`,
    /// before the journal is reached. `0` until the first handshake answers,
    /// which is what this node did before the offset existed.
    clock_offset_ms: Arc<Mutex<i64>>,
    client: reqwest::Client,
}

/// A workload leaf stays valid 10 minutes (ADR-007); rotate when less than
/// half of that remains.
const ROTATE_REMAINING_SECS: i64 = 300;

/// Is the leaf inside its rotation window? A pure decision over two instants,
/// extracted so a control can DRIVE the clock (`SIGNOFF-REPAIR.3.4.3.1.3`).
///
/// ⚠️ `server_now` must be in the SERVER's terms. `not_after` is signed into the
/// certificate from the server's clock, so comparing it to this node's own
/// makes the difference between two clocks read as remaining validity — in the
/// direction that matters: a node BEHIND by more than `ROTATE_REMAINING_SECS`
/// computes a remaining life that never falls to the threshold until after the
/// certificate has actually expired, so it never rotates in time.
///
/// ⚠️ The OPPOSITE direction to the dispatch gate (`.3.4.3.1.2`), where the
/// danger is a node AHEAD. A reader carrying that intuition here gets it
/// backwards, which is why the two live in different leaves.
fn rotation_due(not_after: i64, server_now: i64) -> bool {
    not_after - server_now <= ROTATE_REMAINING_SECS
}

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
            lease_epoch: Arc::new(Mutex::new(None)),
            clock_offset_ms: Arc::new(Mutex::new(0)),
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
            lease_epoch: Arc::new(Mutex::new(None)),
            clock_offset_ms: Arc::new(Mutex::new(0)),
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
        let offset_ms = *self
            .clock_offset_ms
            .lock()
            .expect("the clock-offset lock is not poisoned");
        // In the SERVER's terms — `not_after` is the server's instant, not ours.
        let server_now = chrono::Utc::now().timestamp() + offset_ms / 1_000;
        rotation_due(x509.validity().not_after.timestamp(), server_now)
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

    /// The lease epoch from the latest successful handshake (the same guard as
    /// the token — an unauthenticated channel has neither).
    fn current_lease_epoch(&self) -> Result<i64, ChannelError> {
        self.lease_epoch
            .lock()
            .expect("the lease-epoch lock is not poisoned")
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
        let nonce = fresh_proof_nonce();
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
                    &nonce,
                ),
            )
        };
        let mut with_proof = req.clone();
        with_proof.cert_der = cert_hex;
        with_proof.proof_signature = proof;
        with_proof.nonce = nonce;
        let sent = chrono::Utc::now();
        let response = self
            .client
            .post(format!("{}/v1/nodes/handshake", self.base_url))
            .json(&with_proof)
            .send()
            .await?;
        let parsed: HandshakeResponse = self.parse(response).await?;
        {
            let mut token = self
                .fencing_token
                .lock()
                .expect("the fencing-token lock is not poisoned");
            let mut epoch = self
                .lease_epoch
                .lock()
                .expect("the lease-epoch lock is not poisoned");
            *token = Some(parsed.fencing_token.clone());
            *epoch = Some(parsed.lease_epoch);
        }
        // The server stated its clock (`SIGNOFF-REPAIR.3.4.3.1.2`); the channel
        // keeps its own copy so the NEXT handshake's rotation check — which runs
        // before the journal is reached — is evaluated in the server's terms.
        {
            let mut offset = self
                .clock_offset_ms
                .lock()
                .expect("the clock-offset lock is not poisoned");
            let midpoint = sent + (chrono::Utc::now() - sent) / 2;
            *offset = (parsed.server_time - midpoint).num_milliseconds();
        }
        Ok(parsed)
    }

    /// Rotate the workload certificate: prove possession of the CURRENT leaf
    /// over the rotate coverage and receive a FRESH key + certificate for the
    /// same node id. The caller installs the returned pair (the running
    /// session's fencing token stays valid — rotation is additive).
    pub async fn rotate(&self) -> Result<(Vec<u8>, rcgen::KeyPair), ChannelError> {
        use rcgen::SigningKey;
        let nonce = fresh_proof_nonce();
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
                nonce: &nonce,
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
                "nonce": nonce,
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
                "lease_epoch": self.current_lease_epoch()?,
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
                "lease_epoch": self.current_lease_epoch()?,
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
                "lease_epoch": self.current_lease_epoch()?,
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
                "lease_epoch": self.current_lease_epoch()?,
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
        self.enroll_with_facts(
            token_id, node_id, host_claim, nonce, key_secret, None, None, None, None,
        )
        .await
    }

    /// Enrollment with the §8.1 incarnation facts (`.1.6.1`): the node declares
    /// the provider/model/harness/config it KNOWS at start; the server writes
    /// them to the `incarnations` row when the node id is the role wire id it
    /// serves (a plain `nod_…` node records no incarnation).
    #[allow(clippy::too_many_arguments)]
    pub async fn enroll_with_facts(
        &self,
        token_id: &str,
        node_id: &str,
        host_claim: &str,
        nonce: &str,
        key_secret: &str,
        provider: Option<&str>,
        model: Option<&str>,
        harness: Option<&str>,
        config: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value, ChannelError> {
        let mut body = serde_json::json!({
            "token_id": token_id,
            "node_id": node_id,
            "host_claim": host_claim,
            "nonce": nonce,
            "key_secret": key_secret,
        });
        if let Some(v) = provider {
            body["provider"] = serde_json::json!(v);
        }
        if let Some(v) = model {
            body["model"] = serde_json::json!(v);
        }
        if let Some(v) = harness {
            body["harness"] = serde_json::json!(v);
        }
        if let Some(v) = config {
            body["config"] = v.clone();
        }
        let response = self
            .client
            .post(format!("{}/v1/nodes/enroll", self.base_url))
            .json(&body)
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

#[cfg(test)]
mod rotation_clock {
    use super::{rotation_due, ROTATE_REMAINING_SECS};

    /// The workload leaf's life (ADR-007), so the arithmetic below reads
    /// against a real certificate rather than invented numbers.
    const LEAF_TTL_SECS: i64 = 600;

    /// `SIGNOFF-REPAIR.3.4.3.1.3` — REPRODUCED, with the clock driven.
    ///
    /// `.3.4.3.1` opened this as arithmetic on a source reading and forbade
    /// writing it up as a field failure until a control drove a skewed clock at
    /// it. This is that control. A certificate is issued at server-time 0 and
    /// expires at 600; rotation should fire from server-time 300 onward.
    #[test]
    fn a_node_clock_behind_the_server_rotates_only_after_the_certificate_expired() {
        let issued = 1_000_000i64;
        let not_after = issued + LEAF_TTL_SECS;
        let skew = 600i64; // this node is 600 s BEHIND the server

        // THE DEFECT, uncorrected: the node compares against its own clock,
        // which reads `skew` seconds earlier than the server's.
        let uncorrected = |real_server_time: i64| rotation_due(not_after, real_server_time - skew);
        // At the moment rotation is due, the node does not rotate.
        assert!(
            !uncorrected(not_after - ROTATE_REMAINING_SECS),
            "uncorrected: the node does not rotate when it should"
        );
        // It still does not rotate one second before the certificate expires.
        assert!(
            !uncorrected(not_after - 1),
            "uncorrected: the node still does not rotate with 1 s of validity left"
        );
        // It only decides to rotate once the certificate has ALREADY expired —
        // by `skew - ROTATE_REMAINING_SECS` seconds, here a full 300 s of dead
        // channel before it even tries.
        assert!(
            !uncorrected(not_after + (skew - ROTATE_REMAINING_SECS) - 1),
            "uncorrected: still not rotating, and the certificate expired \
             {} s ago",
            skew - ROTATE_REMAINING_SECS - 1
        );
        assert!(
            uncorrected(not_after + (skew - ROTATE_REMAINING_SECS)),
            "uncorrected: rotation finally fires — after expiry"
        );

        // THE REPAIR: the same node, evaluating in the server's terms.
        let corrected = |real_server_time: i64| rotation_due(not_after, real_server_time);
        assert!(
            !corrected(not_after - ROTATE_REMAINING_SECS - 1),
            "corrected: not yet due one second before the window opens"
        );
        assert!(
            corrected(not_after - ROTATE_REMAINING_SECS),
            "corrected: rotation fires exactly when the window opens, with \
             {ROTATE_REMAINING_SECS} s of validity still in hand"
        );
    }

    /// The other direction, asserted so the correction is a SHIFT rather than a
    /// blanket "always rotate": a node whose clock runs AHEAD rotates early
    /// without the correction, and on time with it. Early is harmless, which is
    /// why this direction was never the defect — but it must not stay wrong.
    #[test]
    fn a_node_clock_ahead_of_the_server_stops_rotating_early() {
        let not_after = 1_000_600i64;
        let skew = 600i64; // AHEAD: the node's clock reads later than the server's
        let long_before_due = not_after - ROTATE_REMAINING_SECS - 400;

        assert!(
            rotation_due(not_after, long_before_due + skew),
            "uncorrected: an early rotation, 400 s before the window opens"
        );
        assert!(
            !rotation_due(not_after, long_before_due),
            "corrected: not due yet, because it genuinely is not"
        );
    }

    /// The WIRING, not just the decision: `cert_expires_soon()` must actually
    /// apply the stored offset. Without this the pure-function controls above
    /// would still pass while the real check compared against the local clock.
    #[test]
    fn cert_expires_soon_applies_the_stored_offset() {
        use rcgen::{CertificateParams, KeyPair};

        // A real leaf whose validity ends 400 s from now — OUTSIDE the 300 s
        // rotation window, so an uncorrected node says "not yet".
        let key = KeyPair::generate().expect("key");
        let mut params = CertificateParams::new(vec!["host-a".to_string()]).expect("params");
        let now = time::OffsetDateTime::from_unix_timestamp(chrono::Utc::now().timestamp())
            .expect("representable");
        params.not_before = now - time::Duration::seconds(200);
        params.not_after = now + time::Duration::seconds(400);
        let cert = params.self_signed(&key).expect("self-sign");

        let channel = super::NodeChannel::new(
            "http://127.0.0.1:1",
            "nod_00000000-0000-7000-8000-000000000001".to_string(),
            cert.der().to_vec(),
            key,
        );
        assert!(
            !channel.cert_expires_soon(),
            "400 s of validity left and no skew: not due"
        );

        // Now say the SERVER's clock reads 200 s later than ours — this node is
        // behind. In the server's terms only 200 s of validity remain, which is
        // inside the window, so rotation IS due.
        *channel
            .clock_offset_ms
            .lock()
            .expect("the clock-offset lock is not poisoned") = 200_000;
        assert!(
            channel.cert_expires_soon(),
            "the stored offset must reach the rotation decision"
        );
    }

    /// With no skew the decision is unchanged, so the correction cannot be
    /// satisfied by moving the boundary.
    #[test]
    fn the_window_boundary_is_unchanged_without_skew() {
        let not_after = 1_000_600i64;
        assert!(!rotation_due(
            not_after,
            not_after - ROTATE_REMAINING_SECS - 1
        ));
        assert!(rotation_due(not_after, not_after - ROTATE_REMAINING_SECS));
        assert!(rotation_due(not_after, not_after));
        assert!(rotation_due(not_after, not_after + 1));
    }
}
