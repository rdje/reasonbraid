//! Integration tests for durable inbox hardening (`PHASE-1.2.3`; backlog 14's
//! remainder): a quarantine status (with reason) that the replay/poll paths skip,
//! and a retention window cleaned by an explicit, MEASURED operator action.
//!
//! The operator surface is the control API (`tenant_admin`-audited, like token
//! issuance); the node surface only behaves differently (the quarantined row
//! never re-delivers). Filtered delivery by eligibility stays with Phase 3's
//! directory. Like the other PostgreSQL suites, these skip without
//! `DATABASE_URL`.

use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use chrono::Utc;
use reasonbraid_server::{
    api_router, ca::ensure_server_ca, node_router, NodeChannelState, CHANNEL_VERSION,
    PRINCIPAL_HEADER,
};
use serde_json::{json, Value};
use sqlx::PgPool;

/// The dev signing secret the seeded nodes share (mirrors `node_channel.rs`).
const DEV_SECRET: &str = "dev-secret";

static INBOX_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    INBOX_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the real PostgreSQL proof"
            );
            return None;
        }
    };
    let pool = PgPool::connect(&url)
        .await
        .expect("connect to DATABASE_URL");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    // This suite exclusively owns these tables for its duration (FK order).
    for table in [
        "outbox_delivery",
        "outbox",
        "node_events",
        "node_inbox",
        "node_leases",
        "budget_reservations",
        "budget_ceilings",
        "authorization_records",
        "authority_grants",
        "enrollments",
        "enrollment_boundaries",
        "node_enroll_audit",
        "node_keys",
        "node_certificates",
        "server_ca",
        "node_enrollment_tokens",
        "runs",
        "incarnations",
        "nodes",
        "hosts",
        "agent_roles",
        "human_principals",
        "tenants",
        "idempotency",
        "event_log",
        "aggregate_state",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&pool)
            .await
            .expect("purge table");
    }
    // The seed tenant the `seed_node` host rows reference.
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name) \
         VALUES ('ten_00000000-0000-7000-8000-000000000000', 'inbox-seed') \
         ON CONFLICT (tenant_id) DO NOTHING",
    )
    .execute(&pool)
    .await
    .expect("seed tenant");
    Some(pool)
}

struct TestServer {
    addr: SocketAddr,
    _handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn start(pool: &PgPool) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().unwrap();
        let ca = Arc::new(ensure_server_ca(pool).await.expect("server CA"));
        let router = api_router(pool.clone()).merge(node_router(pool.clone(), ca));
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.expect("serve");
        });
        Self {
            addr,
            _handle: handle,
        }
    }

    fn base(&self) -> String {
        format!("http://{}", self.addr)
    }
}

/// Bootstrap a human admin and return (tenant, principal id).
async fn bootstrap_admin(client: &reqwest::Client, base: &str) -> (String, String) {
    let response = client
        .post(format!("{base}/v1/enrollments"))
        .json(&json!({ "kind": "human", "name": "alice" }))
        .send()
        .await
        .expect("enroll human");
    assert_eq!(response.status().as_u16(), 200);
    let body: Value = response.json().await.expect("enroll json");
    (
        body["tenant_id"].as_str().unwrap().to_string(),
        body["principal_id"].as_str().unwrap().to_string(),
    )
}

/// Seed an enrolled node (host → node → key → certificate) so the authenticated
/// handshake can read this node.s inbox; the enrollment endpoint.s own semantics
/// are proven by `node_enrollment.rs`. Returns the certificate + key hex.
async fn seed_node(pool: &PgPool, node_id: &str) -> (String, String) {
    let host_id = format!("hst_seed_{}", &node_id[4..]);
    let host_name = format!("seed-{node_id}");
    sqlx::query("INSERT INTO hosts (host_id, tenant_id, name) VALUES ($1, $2, $3)")
        .bind(&host_id)
        .bind("ten_00000000-0000-7000-8000-000000000000")
        .bind(&host_name)
        .execute(pool)
        .await
        .expect("seed host");
    sqlx::query("INSERT INTO nodes (node_id, host_id, tenant_id) VALUES ($1, $2, $3)")
        .bind(node_id)
        .bind(&host_id)
        .bind("ten_00000000-0000-7000-8000-000000000000")
        .execute(pool)
        .await
        .expect("seed node");
    sqlx::query("INSERT INTO node_keys (node_id, key_fingerprint, key_secret) VALUES ($1, $2, $3)")
        .bind(node_id)
        .bind(format!("seed-fingerprint-{node_id}"))
        .bind(DEV_SECRET)
        .execute(pool)
        .await
        .expect("seed key");
    let ca = ensure_server_ca(pool).await.expect("server CA");
    let (cert_der, key_der) = reasonbraid_server::ca::issue_node_leaf(&ca, node_id, &host_name);
    let fingerprint = reasonbraid_server::ca::cert_fingerprint(&cert_der);
    sqlx::query(
        "INSERT INTO node_certificates \
         (cert_fingerprint, node_id, cert_der, key_der, issued_at, expires_at) \
         VALUES ($1, $2, $3, $4, now(), now() + interval '10 minutes')",
    )
    .bind(&fingerprint)
    .bind(node_id)
    .bind(&cert_der)
    .bind(&key_der)
    .execute(pool)
    .await
    .expect("seed certificate");
    (
        reasonbraid_server::ca::to_hex(&cert_der),
        reasonbraid_server::ca::to_hex(&key_der),
    )
}

async fn enqueue(state: &NodeChannelState, node_id: &str, command_id: &str) -> i64 {
    state
        .enqueue(
            node_id,
            command_id,
            "ten_00000000-0000-7000-8000-000000000000",
            "thr_00000000-0000-7000-8000-000000000000",
            &json!({ "operation": "contribute", "command_id": command_id }),
        )
        .await
        .expect("enqueue command")
}

/// The authenticated handshake (raw wire; the proof is computed with the node
/// crate's public canonicalization). Returns the response body.
async fn handshake(
    client: &reqwest::Client,
    base: &str,
    node_id: &str,
    cert_hex: &str,
    key_hex: &str,
) -> Value {
    let key_der = from_hex(key_hex).expect("key hex");
    let der = rustls_pki_types::PrivateKeyDer::try_from(key_der).expect("key DER");
    let key = rcgen::KeyPair::from_der_and_sign_algo(&der, &rcgen::PKCS_ECDSA_P256_SHA256)
        .expect("key parses");
    let proof = reasonbraid_node::compute_cert_proof(&key, CHANNEL_VERSION, node_id, 0, &[], &[]);
    let response = client
        .post(format!("{base}/v1/nodes/handshake"))
        .json(&json!({
            "channel_version": CHANNEL_VERSION,
            "node_id": node_id,
            "last_acked_cursor": 0,
            "pending_operations": [],
            "ambiguous_attempts": [],
            "cert_der": cert_hex,
            "proof_signature": proof,
        }))
        .send()
        .await
        .expect("handshake request");
    assert_eq!(response.status().as_u16(), 200, "handshake succeeds");
    response.json().await.expect("handshake json")
}

/// THE `.1.2.3` quarantine acceptance: a quarantined command is NEVER
/// re-delivered — the handshake replay and the poll tail both skip it, the
/// reason rides the row, and the inspection surface shows the facts.
#[tokio::test]
async fn quarantine_skips_replay_and_poll_and_is_inspectable() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let node_id = "nod_00000000-0000-7000-8000-000000000201";
    let (cert_hex, key_hex) = seed_node(&pool, node_id).await;

    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    for i in 1..=3 {
        enqueue(&state, node_id, &format!("cmd_quar_{i}")).await;
    }

    // The operator quarantines the middle command WITH a reason.
    let response = client
        .post(format!("{base}/v1/nodes/quarantine"))
        .header(PRINCIPAL_HEADER, &alice)
        .json(&json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "command_id": "cmd_quar_2",
            "reason": "poison payload from the adapter",
        }))
        .send()
        .await
        .expect("quarantine request");
    assert_eq!(response.status().as_u16(), 200);
    let quarantined: Value = response.json().await.unwrap();
    assert_eq!(quarantined["command_id"], json!("cmd_quar_2"));
    assert!(quarantined["quarantined_at"].is_string());

    // Replay: the quarantined command is skipped, whatever the reported cursor.
    let hs = handshake(&client, &base, node_id, &cert_hex, &key_hex).await;
    assert_eq!(hs["current_cursor"], json!(3));
    let replay_ids: Vec<&str> = hs["replay"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["command_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        replay_ids,
        vec!["cmd_quar_1", "cmd_quar_3"],
        "the quarantined command is never re-delivered"
    );

    // Poll: same filter on the live tail.
    let fencing_token = hs["fencing_token"].as_str().unwrap();
    let lease_epoch = hs["lease_epoch"].as_i64().unwrap();
    let poll = client
        .post(format!("{base}/v1/nodes/poll"))
        .json(&json!({
            "channel_version": CHANNEL_VERSION,
            "node_id": node_id,
            "after_cursor": 0,
            "fencing_token": fencing_token,
            "lease_epoch": lease_epoch,
        }))
        .send()
        .await
        .expect("poll request");
    assert_eq!(poll.status().as_u16(), 200);
    let poll_body: Value = poll.json().await.unwrap();
    let poll_ids: Vec<&str> = poll_body["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["command_id"].as_str().unwrap())
        .collect();
    assert_eq!(poll_ids, vec!["cmd_quar_1", "cmd_quar_3"]);

    // The inspection surface shows the delivery + quarantine facts.
    let inspection = client
        .get(format!("{base}/v1/nodes/inbox"))
        .header(PRINCIPAL_HEADER, &alice)
        .query(&[("tenant_id", tenant.as_str()), ("node_id", node_id)])
        .send()
        .await
        .expect("inspection request");
    assert_eq!(inspection.status().as_u16(), 200);
    let inbox: Value = inspection.json().await.unwrap();
    assert_eq!(inbox["rows"].as_array().unwrap().len(), 3);
    let quarantined_row = inbox["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["command_id"] == "cmd_quar_2")
        .expect("the quarantined row is listed");
    assert!(quarantined_row["quarantined_at"].is_string());
    assert_eq!(
        quarantined_row["quarantine_reason"],
        json!("poison payload from the adapter"),
        "the reason rides the row"
    );
    assert!(
        inbox["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["command_id"] != "cmd_quar_2")
            .all(|r| r["quarantined_at"].is_null()),
        "only the quarantined row carries a quarantine"
    );
}

/// The operator-action refusals are typed: `tenant_admin` only (a role is 403),
/// an unknown command is a 400, a re-quarantine is a 409, an empty reason is a
/// 400, and a negative prune window is a 400. Every refusal leaves the inbox
/// untouched.
#[tokio::test]
async fn quarantine_and_prune_refusals_are_typed() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let node_id = "nod_00000000-0000-7000-8000-000000000202";
    let _ = seed_node(&pool, node_id).await;

    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    enqueue(&state, node_id, "cmd_typed").await;

    // A role (default thread_contribute grant) is NOT tenant_admin.
    let role = client
        .post(format!("{base}/v1/enrollments"))
        .json(&json!({ "kind": "role", "name": "reviewer", "tenant_id": tenant }))
        .send()
        .await
        .expect("enroll role");
    let role_body: Value = role.json().await.unwrap();
    let role_id = role_body["principal_id"].as_str().unwrap().to_string();

    let quarantine = |principal: String, body: Value| {
        let client = client.clone();
        let base = base.clone();
        async move {
            client
                .post(format!("{base}/v1/nodes/quarantine"))
                .header(PRINCIPAL_HEADER, principal)
                .json(&body)
                .send()
                .await
                .unwrap()
        }
    };

    // Role denied (403, audited).
    let denied = quarantine(
        role_id.clone(),
        json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "command_id": "cmd_typed",
            "reason": "because",
        }),
    )
    .await;
    assert_eq!(denied.status().as_u16(), 403);
    let body: Value = denied.json().await.unwrap();
    assert_eq!(body["code"], json!("unauthorized"));

    // Unknown command (400, no row touched).
    let unknown = quarantine(
        alice.clone(),
        json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "command_id": "cmd_no_such",
            "reason": "because",
        }),
    )
    .await;
    assert_eq!(unknown.status().as_u16(), 400);
    let body: Value = unknown.json().await.unwrap();
    assert_eq!(body["code"], json!("invalid_command"));

    // Empty reason (400).
    let empty = quarantine(
        alice.clone(),
        json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "command_id": "cmd_typed",
            "reason": "   ",
        }),
    )
    .await;
    assert_eq!(empty.status().as_u16(), 400);

    // The real quarantine succeeds, and a re-quarantine is a typed 409.
    let ok = quarantine(
        alice.clone(),
        json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "command_id": "cmd_typed",
            "reason": "bad content",
        }),
    )
    .await;
    assert_eq!(ok.status().as_u16(), 200);
    let again = quarantine(
        alice.clone(),
        json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "command_id": "cmd_typed",
            "reason": "another reason",
        }),
    )
    .await;
    assert_eq!(again.status().as_u16(), 409);
    let body: Value = again.json().await.unwrap();
    assert_eq!(body["code"], json!("invalid_transition"));

    // Prune: role denied; negative window refused.
    let prune = |principal: String, body: Value| {
        let client = client.clone();
        let base = base.clone();
        async move {
            client
                .post(format!("{base}/v1/nodes/inbox/prune"))
                .header(PRINCIPAL_HEADER, principal)
                .json(&body)
                .send()
                .await
                .unwrap()
        }
    };
    let denied_prune = prune(
        role_id.clone(),
        json!({ "tenant_id": tenant, "node_id": node_id, "min_age_seconds": 0 }),
    )
    .await;
    assert_eq!(denied_prune.status().as_u16(), 403);
    let negative = prune(
        alice.clone(),
        json!({ "tenant_id": tenant, "node_id": node_id, "min_age_seconds": -1 }),
    )
    .await;
    assert_eq!(negative.status().as_u16(), 400);

    // Nothing was deleted by any refusal.
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM node_inbox WHERE node_id = $1")
        .bind(node_id)
        .fetch_one(&pool)
        .await
        .expect("count rows");
    assert_eq!(count, 1, "every refusal left the inbox untouched");
}

/// THE `.1.2.3` retention acceptance: pruning deletes ONLY delivered rows older
/// than the window, and the response is a measured before/after — an explicit
/// operator action, never a background sweep.
#[tokio::test]
async fn prune_deletes_only_delivered_rows_older_than_the_window() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let node_id = "nod_00000000-0000-7000-8000-000000000203";
    let _ = seed_node(&pool, node_id).await;

    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    for i in 1..=5 {
        enqueue(&state, node_id, &format!("cmd_prune_{i}")).await;
    }
    // The node acknowledged rows 1..=3 (delivered); 4 and 5 are still pending.
    state.acknowledge(node_id, 3, Utc::now()).await.unwrap();
    // Fast-forward rows 1 and 2 ten days; row 3's acknowledgement is fresh.
    sqlx::query(
        "UPDATE node_inbox SET acknowledged_at = now() - interval '10 days' \
         WHERE node_id = $1 AND cursor <= 2",
    )
    .bind(node_id)
    .execute(&pool)
    .await
    .expect("backdate the old acknowledgements");

    let prune = client
        .post(format!("{base}/v1/nodes/inbox/prune"))
        .header(PRINCIPAL_HEADER, &alice)
        .json(&json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "min_age_seconds": 7 * 24 * 3600,
        }))
        .send()
        .await
        .expect("prune request");
    assert_eq!(prune.status().as_u16(), 200);
    let measured: Value = prune.json().await.unwrap();
    assert_eq!(measured["before"], json!(5));
    assert_eq!(measured["deleted"], json!(2));
    assert_eq!(measured["after"], json!(3));
    assert!(measured["cutoff_at"].is_string());

    // Only the two old DELIVERED rows are gone; the fresh delivery and the two
    // undelivered rows survive.
    let survivors: Vec<String> =
        sqlx::query_scalar("SELECT command_id FROM node_inbox WHERE node_id = $1 ORDER BY cursor")
            .bind(node_id)
            .fetch_all(&pool)
            .await
            .expect("survivors");
    assert_eq!(survivors, vec!["cmd_prune_3", "cmd_prune_4", "cmd_prune_5"]);

    // A second prune (nothing new is old enough) measures a no-op.
    let again = client
        .post(format!("{base}/v1/nodes/inbox/prune"))
        .header(PRINCIPAL_HEADER, &alice)
        .json(&json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "min_age_seconds": 7 * 24 * 3600,
        }))
        .send()
        .await
        .expect("second prune");
    let measured: Value = again.json().await.unwrap();
    assert_eq!(measured["before"], json!(3));
    assert_eq!(measured["deleted"], json!(0));
    assert_eq!(measured["after"], json!(3));
}

/// Lowercase-hex decode (the wire ships DER as hex).
fn from_hex(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("odd-length hex".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}
