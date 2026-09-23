//! A stub control plane a test node can REALLY reconcile against
//! (`SIGNOFF-REPAIR.4.4.4.2.1`).
//!
//! Every other node-crate control points its node at `http://127.0.0.1:1`, so no
//! test node could ever become `Schedulable`, and nothing a schedulable node does
//! — delivering a result, re-emitting a pending one — was visible below the live
//! server suites. This stub answers the three writes a reconcile and a delivery
//! make (`handshake`, `events`, `ack`) with the channel's OWN response types, so
//! the node's client parses exactly what it parses in production, and it records
//! every event it receives for the control to inspect. `poll` answers an empty
//! tail, so a control can drive `Worker::tick` (`SIGNOFF-REPAIR.4.4.5.2`).
//!
//! ⚠️ It is a stub, not a server. It verifies no certificate proof, keeps no
//! inbox, replays nothing and folds nothing; a control that needs any of those
//! belongs in the live suites. What it does check is the one credential a
//! delivery carries: an event or an acknowledgement under a fencing token the
//! stub never issued is refused, so a node cannot reach it without a handshake.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use reasonbraid_node::{
    AckResponse, EventReceipt, HandshakeResponse, Node, PollResponse, CHANNEL_VERSION,
};
use serde_json::Value;

/// The fencing token every handshake issues.
const STUB_TOKEN: &str = "stub-fencing-token";

/// One event the stub received, as the node sent it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReceivedEvent {
    pub event_id: String,
    pub operation_id: String,
    pub payload: Value,
}

#[derive(Default)]
struct Recorded {
    /// The epoch map every handshake and poll answers with. A response's map is
    /// the COMPLETE set of tenants the node may act for, and the node replaces
    /// its own with it, so a control that ticks must name its tenants here.
    revocation_epochs: BTreeMap<String, i64>,
    handshakes: u32,
    events: Vec<ReceivedEvent>,
}

/// A running stub; it stops when dropped.
pub struct StubControlPlane {
    base_url: String,
    recorded: Arc<Mutex<Recorded>>,
    server: tokio::task::JoinHandle<()>,
}

impl StubControlPlane {
    /// Bind a loopback port and serve until dropped, answering an EMPTY epoch map.
    pub async fn start() -> Self {
        Self::start_with_epochs(BTreeMap::new()).await
    }

    /// [`StubControlPlane::start`], answering `revocation_epochs` on every
    /// handshake and poll.
    pub async fn start_with_epochs(revocation_epochs: BTreeMap<String, i64>) -> Self {
        let recorded = Arc::new(Mutex::new(Recorded {
            revocation_epochs,
            ..Recorded::default()
        }));
        let app = Router::new()
            .route("/v1/nodes/handshake", post(handshake))
            .route("/v1/nodes/events", post(events))
            .route("/v1/nodes/ack", post(ack))
            .route("/v1/nodes/poll", post(poll))
            .with_state(recorded.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind a loopback port");
        let base_url = format!("http://{}", listener.local_addr().expect("bound address"));
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("the stub serves");
        });
        Self {
            base_url,
            recorded,
            server,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// How many handshakes the stub has answered.
    pub fn handshakes(&self) -> u32 {
        self.recorded.lock().expect("not poisoned").handshakes
    }

    /// Every event received, in arrival order.
    pub fn events(&self) -> Vec<ReceivedEvent> {
        self.recorded.lock().expect("not poisoned").events.clone()
    }
}

/// A node journaling at `journal`, pointed at `stub` and RECONCILED against it,
/// so it is `Schedulable` and may dispatch (`SIGNOFF-REPAIR.4.4.4.2.2`: an
/// unschedulable node spends nothing). The reconcile replaces the journal's
/// epoch map with the stub's EMPTY one, so a control sets its epochs and server
/// time AFTER this returns.
pub async fn reconciled_node(stub: &StubControlPlane, journal: &Path) -> Node {
    let key = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).expect("keypair");
    let node = Node::open(
        journal,
        stub.base_url(),
        "nod_00000000-0000-7000-8000-000000000001".to_string(),
        vec![0x00, 0x01, 0x02],
        key,
    )
    .await
    .expect("open node");
    node.reconcile().await.expect("reconcile against the stub");
    node
}

impl Drop for StubControlPlane {
    fn drop(&mut self) {
        self.server.abort();
    }
}

/// A handshake with nothing to replay: no commands, no directives, no known
/// events, and the configured epoch map (EMPTY unless the control named one; a
/// control that never ticks sets its epochs on the journal AFTER reconciling,
/// which is when the gate reads them).
async fn handshake(State(recorded): State<Arc<Mutex<Recorded>>>) -> Json<HandshakeResponse> {
    let revocation_epochs = {
        let mut recorded = recorded.lock().expect("not poisoned");
        recorded.handshakes += 1;
        recorded.revocation_epochs.clone()
    };
    let now = chrono::Utc::now();
    Json(HandshakeResponse {
        channel_version: CHANNEL_VERSION,
        current_cursor: 0,
        replay: Vec::new(),
        directives: Vec::new(),
        known_events: Vec::new(),
        fencing_token: STUB_TOKEN.to_string(),
        lease_expires_at: now + chrono::Duration::hours(1),
        lease_epoch: 1,
        revocation_epochs,
        server_time: now,
        offers_pending: 0,
    })
}

/// An empty delivery tail under the configured epoch map.
async fn poll(
    State(recorded): State<Arc<Mutex<Recorded>>>,
    Json(body): Json<Value>,
) -> Result<Json<PollResponse>, StatusCode> {
    if !issued(&body) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let revocation_epochs = recorded
        .lock()
        .expect("not poisoned")
        .revocation_epochs
        .clone();
    Ok(Json(PollResponse {
        channel_version: CHANNEL_VERSION,
        current_cursor: 0,
        commands: Vec::new(),
        revocation_epochs,
        server_time: chrono::Utc::now(),
    }))
}

fn issued(body: &Value) -> bool {
    body["fencing_token"] == STUB_TOKEN
}

async fn events(
    State(recorded): State<Arc<Mutex<Recorded>>>,
    Json(body): Json<Value>,
) -> Result<Json<EventReceipt>, StatusCode> {
    if !issued(&body) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let event = ReceivedEvent {
        event_id: body["event_id"].as_str().unwrap_or_default().to_string(),
        operation_id: body["operation_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        payload: body["payload"].clone(),
    };
    let mut recorded = recorded.lock().expect("not poisoned");
    let accepted = !recorded.events.iter().any(|e| e.event_id == event.event_id);
    recorded.events.push(event);
    Ok(Json(EventReceipt {
        channel_version: CHANNEL_VERSION,
        accepted,
        refused: None,
    }))
}

async fn ack(Json(body): Json<Value>) -> Result<Json<AckResponse>, StatusCode> {
    if !issued(&body) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(AckResponse {
        channel_version: CHANNEL_VERSION,
        acknowledged: body["ack_cursor"].as_i64().unwrap_or_default(),
    }))
}

/// A server that ACCEPTS every connection and never answers a byte
/// (`SIGNOFF-REPAIR.4.4.5.1`): the failure a timeout exists for. A refused
/// connection fails fast by itself; an accepted one that goes silent waits as
/// long as the client lets it. The accepted sockets are held open until the
/// server is dropped, so the silence is the server's and not a closed socket.
pub struct StalledServer {
    base_url: String,
    server: tokio::task::JoinHandle<()>,
}

impl StalledServer {
    pub async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind a loopback port");
        let base_url = format!("http://{}", listener.local_addr().expect("bound address"));
        let server = tokio::spawn(async move {
            let mut held = Vec::new();
            while let Ok((socket, _)) = listener.accept().await {
                held.push(socket);
            }
        });
        Self { base_url, server }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Drop for StalledServer {
    fn drop(&mut self) {
        self.server.abort();
    }
}
