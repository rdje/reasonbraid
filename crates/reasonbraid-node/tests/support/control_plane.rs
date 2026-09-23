//! A stub control plane a test node can REALLY reconcile against
//! (`SIGNOFF-REPAIR.4.4.4.2.1`).
//!
//! Every other node-crate control points its node at `http://127.0.0.1:1`, so no
//! test node could ever become `Schedulable`, and nothing a schedulable node does
//! — delivering a result, re-emitting a pending one — was visible below the live
//! server suites. This stub answers the three writes a reconcile and a delivery
//! make (`handshake`, `events`, `ack`) with the channel's OWN response types, so
//! the node's client parses exactly what it parses in production, and it records
//! every event it receives for the control to inspect.
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
use reasonbraid_node::{AckResponse, EventReceipt, HandshakeResponse, Node, CHANNEL_VERSION};
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
    /// Bind a loopback port and serve until dropped.
    pub async fn start() -> Self {
        let recorded = Arc::new(Mutex::new(Recorded::default()));
        let app = Router::new()
            .route("/v1/nodes/handshake", post(handshake))
            .route("/v1/nodes/events", post(events))
            .route("/v1/nodes/ack", post(ack))
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
/// events, and an EMPTY epoch map — a control sets the epochs it needs on the
/// journal AFTER reconciling, which is when the gate reads them.
async fn handshake(State(recorded): State<Arc<Mutex<Recorded>>>) -> Json<HandshakeResponse> {
    recorded.lock().expect("not poisoned").handshakes += 1;
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
        revocation_epochs: BTreeMap::new(),
        server_time: now,
        offers_pending: 0,
    })
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
