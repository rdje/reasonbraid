//! Integration tests for durable inbox hardening (`PHASE-1.2.3`; backlog 14's
//! remainder): a quarantine status (with reason) that the replay/poll paths skip,
//! and a retention window cleaned by an explicit, MEASURED operator action.
//!
//! The operator surface is the control API (`tenant_admin`-audited, like token
//! issuance); the node surface only behaves differently (the quarantined row
//! never re-delivers). Filtered delivery by eligibility stays with Phase 3's
//! directory. Like the other PostgreSQL suites, these skip without
//! `DATABASE_URL`.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

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
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    // This suite exclusively owns these tables for its duration (FK order).
    pg_cleanup::delete_tables(
        &pool,
        &[
            "outbox_delivery",
            "outbox",
            "node_events",
            "node_inbox",
            "node_leases",
            "budget_reservations",
            "budget_ceilings",
            "spend_breakers",
            "administrative_effects",
            "node_enrollment_tokens",
            "authorization_records",
            "authority_grants",
            "enrollments",
            "enrollment_boundaries",
            "node_enroll_audit",
            "node_keys",
            "node_certificates",
            "server_ca",
            "runs",
            "incarnations",
            "node_proof_nonces",
            "nodes",
            "hosts",
            "profile_versions",
            "agent_profiles",
            "recruitment_panels",
            "recruitment_responses",
            "recruitment_offers",
            "recruitment_calls",
            "agent_roles",
            "human_principals",
            "claim_assessments",
            "derivations",
            "evidence_snapshots",
            "resource_references",
            "quota_events",
            "usage_quotas",
            "federation_agreements",
            "cross_domain_receipts",
            "mcp_listen_state",
            "tenant_bootstrap_requests",
            "tenants",
            "idempotency",
            "event_log",
            "aggregate_state",
        ],
    )
    .await
    .expect("purge checked fixture plan");
    // ⚠️ A leaked deliberate fault fails every later test in this suite for a
    // reason unrelated to them; healing it here makes a leak cost one test.
    sqlx::query(
        "ALTER TABLE administrative_effects \
         DROP CONSTRAINT IF EXISTS signoff_repair_3_3_4_10_3_evidence_fault",
    )
    .execute(&pool)
    .await
    .expect("clear any leaked evidence fault");
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
/// ⚠️ The tenant is a PARAMETER since `SIGNOFF-REPAIR.3.3.4.10.3`, and that is
/// the correction, not a tidy-up. These fixtures used to seed the node into a
/// fixed seed tenant and then administer it as an administrator of a DIFFERENT,
/// freshly bootstrapped tenant — and the routes accepted it, because none of the
/// three inbox verbs used the `tenant_id` column the table has carried since
/// migration 0003. A probe measured quarantine and replay answering 200 on a
/// foreign row and prune reporting `{"deleted":2,"before":2,"after":0}` while
/// destroying another tenant's inbox. The verbs are now tenant-bound, so a
/// fixture that wants to administer a node must own it.
async fn seed_node_in(pool: &PgPool, node_id: &str, tenant: &str) -> (String, String) {
    let host_id = format!("hst_seed_{}", &node_id[4..]);
    let host_name = format!("seed-{node_id}");
    sqlx::query("INSERT INTO hosts (host_id, tenant_id, name) VALUES ($1, $2, $3)")
        .bind(&host_id)
        .bind(tenant)
        .bind(&host_name)
        .execute(pool)
        .await
        .expect("seed host");
    sqlx::query("INSERT INTO nodes (node_id, host_id, tenant_id) VALUES ($1, $2, $3)")
        .bind(node_id)
        .bind(&host_id)
        .bind(tenant)
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
    let leaf = reasonbraid_server::ca::issue_node_leaf(&ca, node_id, &host_name);
    let (cert_der, key_der) = (leaf.cert_der, leaf.key_der);
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

async fn enqueue_in(
    state: &NodeChannelState,
    node_id: &str,
    command_id: &str,
    tenant: &str,
) -> i64 {
    state
        .enqueue(
            node_id,
            command_id,
            tenant,
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
    let nonce = reasonbraid_node::fresh_proof_nonce();
    let proof =
        reasonbraid_node::compute_cert_proof(&key, CHANNEL_VERSION, node_id, 0, &[], &[], &nonce);
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
            "nonce": nonce,
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
    // The administrator is bootstrapped FIRST so the node and its inbox rows can
    // be seeded into the tenant that administers them. Before
    // `SIGNOFF-REPAIR.3.3.4.10.3` the order did not matter, because the verbs
    // ignored the tenant entirely.
    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let (cert_hex, key_hex) = seed_node_in(&pool, node_id, &tenant).await;
    for i in 1..=3 {
        enqueue_in(&state, node_id, &format!("cmd_quar_{i}"), &tenant).await;
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
    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let _ = seed_node_in(&pool, node_id, &tenant).await;
    enqueue_in(&state, node_id, "cmd_typed", &tenant).await;

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
    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let _ = seed_node_in(&pool, node_id, &tenant).await;
    for i in 1..=5 {
        enqueue_in(&state, node_id, &format!("cmd_prune_{i}"), &tenant).await;
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

// ── Inbox administration as one guarded transaction (`SIGNOFF-REPAIR.3.3.4.10.3`) ──
//
// The THIRD and last of this parent's children. The repair these controls cover
// is a cross-tenant one: `node_inbox` has carried a `tenant_id` column since
// migration 0003 and none of the three verbs used it. Measured on the superseded
// routes, with an administrator of tenant A acting on tenant B's node:
//
//   quarantine a foreign row -> 200
//   replay a foreign row     -> 200
//   prune a foreign inbox    -> 200  {"deleted":2,"before":2,"after":0}
//
// The prune DESTROYED both of the other tenant's rows. The three fixtures above
// depended on that defect — they seeded into one tenant and administered from
// another — and are corrected rather than deleted.
//
// ⚠️ The guard mode does not change here (shared before, shared after), so no
// lock-holding fixture can discriminate this repair. What discriminates is the
// TENANT BINDING and atomicity, which is what these controls assert.

use reasonbraid_core::{
    AdministrativeOperation, AdministrativeOutcome, AdministrativeRefusal, AdministrativeTargetId,
    TenantId,
};
use reasonbraid_server::load_tenant_administrative_effect;

/// Any of the three inbox POSTs, returning status, the receipt and the body.
async fn inbox_post(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    body: Value,
) -> (u16, Option<String>, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(&body)
        .send()
        .await
        .expect("inbox request");
    let status = response.status().as_u16();
    let receipt = response
        .headers()
        .get("x-reasonbraid-authorization")
        .map(|v| v.to_str().expect("an ASCII receipt").to_owned());
    let text = response.text().await.unwrap_or_default();
    let body = serde_json::from_str(&text).unwrap_or(Value::Null);
    (status, receipt, body)
}

async fn inbox_effect(
    pool: &PgPool,
    tenant: &str,
    receipt: Option<String>,
) -> Option<reasonbraid_core::AdministrativeEffectRecord> {
    let tenant: TenantId = tenant.parse().expect("a tenant id");
    load_tenant_administrative_effect(
        pool,
        tenant,
        receipt
            .expect("every admitted answer carries its receipt")
            .parse()
            .expect("the receipt is a record id"),
    )
    .await
    .expect("read the effect back")
}

async fn inbox_rows(pool: &PgPool, node_id: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM node_inbox WHERE node_id = $1")
        .bind(node_id)
        .fetch_one(pool)
        .await
        .expect("count inbox rows")
}

async fn quarantined_at(pool: &PgPool, command_id: &str) -> Option<chrono::DateTime<Utc>> {
    sqlx::query_scalar("SELECT quarantined_at FROM node_inbox WHERE command_id = $1")
        .bind(command_id)
        .fetch_one(pool)
        .await
        .expect("read the quarantine state")
}

/// ⛔ THE control this child exists for. An administrator of one tenant must not
/// be able to read, move or destroy another tenant's inbox — and before this
/// repair it could do all three, with a 200 each time.
#[tokio::test]
async fn a_foreign_administrator_cannot_quarantine_replay_or_prune_another_tenants_inbox() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // The victim: its own administrator, its own node, its own inbox rows.
    let (victim_tenant, _victim_admin) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000211";
    let _ = seed_node_in(&pool, node_id, &victim_tenant).await;
    for i in 1..=2 {
        enqueue_in(&state, node_id, &format!("cmd_victim_{i}"), &victim_tenant).await;
    }
    // Both rows delivered long ago, so an unbound prune would delete them.
    sqlx::query(
        "UPDATE node_inbox SET acknowledged_at = now() - interval '10 days' WHERE node_id = $1",
    )
    .bind(node_id)
    .execute(&pool)
    .await
    .expect("age the rows");
    let before = inbox_rows(&pool, node_id).await;
    assert_eq!(before, 2);

    // The outsider: a legitimate administrator of a DIFFERENT tenant.
    let (outsider_tenant, outsider) = bootstrap_admin(&client, &base).await;
    assert_ne!(outsider_tenant, victim_tenant);

    let (status, receipt, body) = inbox_post(
        &client,
        &base,
        "/v1/nodes/quarantine",
        &outsider,
        json!({ "tenant_id": outsider_tenant, "node_id": node_id,
                "command_id": "cmd_victim_1", "reason": "probing" }),
    )
    .await;
    assert_eq!(status, 400, "a foreign quarantine is refused: {body}");
    assert_eq!(
        quarantined_at(&pool, "cmd_victim_1").await,
        None,
        "the foreign row is not quarantined"
    );
    let effect = inbox_effect(&pool, &outsider_tenant, receipt)
        .await
        .expect("the refusal is recorded in the CALLER's tenant");
    let AdministrativeOutcome::Refused { code, .. } = &effect.outcome else {
        panic!("expected a recorded refusal, got {:?}", effect.outcome)
    };
    assert_eq!(*code, AdministrativeRefusal::InvalidCommand);

    let (status, receipt, body) = inbox_post(
        &client,
        &base,
        "/v1/nodes/replay",
        &outsider,
        json!({ "tenant_id": outsider_tenant, "node_id": node_id,
                "command_id": "cmd_victim_1" }),
    )
    .await;
    assert_eq!(status, 404, "a foreign replay is refused: {body}");
    let effect = inbox_effect(&pool, &outsider_tenant, receipt)
        .await
        .expect("recorded");
    let AdministrativeOutcome::Refused { code, .. } = &effect.outcome else {
        panic!("expected a recorded refusal, got {:?}", effect.outcome)
    };
    assert_eq!(*code, AdministrativeRefusal::NotFound);

    // ⛔ The destructive one. It answers 200 because zeros are an honest answer
    // for an inbox the caller has none of — and it must delete NOTHING.
    let (status, receipt, body) = inbox_post(
        &client,
        &base,
        "/v1/nodes/inbox/prune",
        &outsider,
        json!({ "tenant_id": outsider_tenant, "node_id": node_id, "min_age_seconds": 0 }),
    )
    .await;
    assert_eq!(
        status, 200,
        "a foreign prune reports its own empty inbox: {body}"
    );
    assert_eq!(
        body["deleted"],
        json!(0),
        "a foreign prune deletes nothing — it used to report deleting the victim's rows"
    );
    assert_eq!(
        body["before"],
        json!(0),
        "the counts describe the caller's own inbox, not the node's other tenant"
    );
    assert_eq!(
        inbox_rows(&pool, node_id).await,
        before,
        "the victim's rows survive"
    );
    let effect = inbox_effect(&pool, &outsider_tenant, receipt)
        .await
        .expect("recorded");
    assert!(
        !effect.outcome.changed_protected_state(),
        "a prune that deleted nothing records no protected change"
    );

    // Nothing at all was recorded in the victim's tenant.
    let victim_admissions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM authorization_records WHERE tenant_id = $1")
            .bind(&victim_tenant)
            .fetch_one(&pool)
            .await
            .expect("count victim admissions");
    assert_eq!(
        victim_admissions, 0,
        "a foreign attempt leaves no trace in the victim's tenant"
    );
}

#[tokio::test]
async fn the_three_inbox_verbs_record_what_they_did() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000212";
    let _ = seed_node_in(&pool, node_id, &tenant).await;
    for i in 1..=2 {
        enqueue_in(&state, node_id, &format!("cmd_rec_{i}"), &tenant).await;
    }
    let target = AdministrativeTargetId::new(node_id).unwrap();

    // Quarantine: `applied`, with the reason persisted alongside the row's copy.
    let (status, receipt, body) = inbox_post(
        &client,
        &base,
        "/v1/nodes/quarantine",
        &alice,
        json!({ "tenant_id": tenant, "node_id": node_id,
                "command_id": "cmd_rec_1", "reason": "poison payload" }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let effect = inbox_effect(&pool, &tenant, receipt)
        .await
        .expect("recorded");
    assert_eq!(
        effect.operation,
        AdministrativeOperation::NodeCommandQuarantine {
            node_id: target.clone(),
            command_id: AdministrativeTargetId::new("cmd_rec_1").unwrap(),
        }
    );
    assert!(effect.outcome.changed_protected_state());
    assert_eq!(
        effect.submitted_reason.as_ref().map(|r| r.as_str()),
        Some("poison payload")
    );
    assert_eq!(
        body["quarantined_at"].as_str().unwrap(),
        effect.effected_at.to_rfc3339(),
        "the response and the record share the transaction's own instant"
    );

    // A repeat is already satisfied: the same 409, recorded `no_op`.
    let (status, receipt, _) = inbox_post(
        &client,
        &base,
        "/v1/nodes/quarantine",
        &alice,
        json!({ "tenant_id": tenant, "node_id": node_id,
                "command_id": "cmd_rec_1", "reason": "again" }),
    )
    .await;
    assert_eq!(status, 409);
    let effect = inbox_effect(&pool, &tenant, receipt)
        .await
        .expect("recorded");
    assert!(matches!(effect.outcome, AdministrativeOutcome::NoOp { .. }));

    // Replay: `applied`, and the quarantine is reversed.
    let (status, receipt, body) = inbox_post(
        &client,
        &base,
        "/v1/nodes/replay",
        &alice,
        json!({ "tenant_id": tenant, "node_id": node_id, "command_id": "cmd_rec_1" }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(quarantined_at(&pool, "cmd_rec_1").await, None);
    let effect = inbox_effect(&pool, &tenant, receipt)
        .await
        .expect("recorded");
    assert_eq!(
        effect.operation,
        AdministrativeOperation::NodeCommandReplay {
            node_id: target.clone(),
            command_id: AdministrativeTargetId::new("cmd_rec_1").unwrap(),
        }
    );
    assert!(effect.outcome.changed_protected_state());
    assert_eq!(
        effect.submitted_reason, None,
        "replay takes no caller reason, so none is invented"
    );

    // Replaying a live command is refused, and recorded as such.
    let (status, receipt, _) = inbox_post(
        &client,
        &base,
        "/v1/nodes/replay",
        &alice,
        json!({ "tenant_id": tenant, "node_id": node_id, "command_id": "cmd_rec_2" }),
    )
    .await;
    assert_eq!(status, 409);
    let effect = inbox_effect(&pool, &tenant, receipt)
        .await
        .expect("recorded");
    let AdministrativeOutcome::Refused { code, .. } = &effect.outcome else {
        panic!("expected a recorded refusal, got {:?}", effect.outcome)
    };
    assert_eq!(*code, AdministrativeRefusal::InvalidTransition);

    // Prune with nothing old enough: 200 with its measured counts, `no_op`.
    let (status, receipt, body) = inbox_post(
        &client,
        &base,
        "/v1/nodes/inbox/prune",
        &alice,
        json!({ "tenant_id": tenant, "node_id": node_id, "min_age_seconds": 604800 }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["deleted"], json!(0));
    assert_eq!(body["before"], json!(2));
    assert_eq!(body["after"], json!(2));
    let effect = inbox_effect(&pool, &tenant, receipt)
        .await
        .expect("recorded");
    assert_eq!(
        effect.operation,
        AdministrativeOperation::NodeInboxPrune {
            node_id: target.clone()
        }
    );
    assert!(matches!(effect.outcome, AdministrativeOutcome::NoOp { .. }));

    // Prune that removes something: `applied`, with the counts as the receipt.
    state.acknowledge(node_id, 3, Utc::now()).await.unwrap();
    sqlx::query(
        "UPDATE node_inbox SET acknowledged_at = now() - interval '10 days' WHERE node_id = $1",
    )
    .bind(node_id)
    .execute(&pool)
    .await
    .expect("age the rows");
    let (status, receipt, body) = inbox_post(
        &client,
        &base,
        "/v1/nodes/inbox/prune",
        &alice,
        json!({ "tenant_id": tenant, "node_id": node_id, "min_age_seconds": 0 }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert!(body["deleted"].as_i64().unwrap() >= 1);
    let effect = inbox_effect(&pool, &tenant, receipt)
        .await
        .expect("recorded");
    assert!(effect.outcome.changed_protected_state());
}

#[tokio::test]
async fn the_quarantine_reason_bounds_are_a_wire_contract_and_refuse_before_any_effect() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000213";
    let _ = seed_node_in(&pool, node_id, &tenant).await;
    enqueue_in(&state, node_id, "cmd_reason", &tenant).await;

    // ⚠️ The documented wire change: blankness was already refused; the byte
    // ceiling and the control-character rule are new.
    for (label, reason) in [
        ("blank", "   ".to_owned()),
        ("a control character", "poison\npayload".to_owned()),
        ("one byte over the ceiling", "x".repeat(1025)),
    ] {
        let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM administrative_effects")
            .fetch_one(&pool)
            .await
            .expect("count effects");
        let (status, receipt, body) = inbox_post(
            &client,
            &base,
            "/v1/nodes/quarantine",
            &alice,
            json!({ "tenant_id": tenant, "node_id": node_id,
                    "command_id": "cmd_reason", "reason": reason }),
        )
        .await;
        assert_eq!(status, 400, "{label} is refused: {body}");
        let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM administrative_effects")
            .fetch_one(&pool)
            .await
            .expect("count effects");
        assert_eq!(
            after, before,
            "{label} never became an operation, so it records no effect"
        );
        assert!(inbox_effect(&pool, &tenant, receipt).await.is_none());
        assert_eq!(quarantined_at(&pool, "cmd_reason").await, None);
    }

    // Exactly at the ceiling succeeds.
    let (status, _, body) = inbox_post(
        &client,
        &base,
        "/v1/nodes/quarantine",
        &alice,
        json!({ "tenant_id": tenant, "node_id": node_id,
                "command_id": "cmd_reason", "reason": "x".repeat(1024) }),
    )
    .await;
    assert_eq!(status, 200, "a reason at the ceiling is usable: {body}");
}

#[tokio::test]
async fn an_evidence_failure_rolls_a_prune_back_and_the_route_recovers() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000214";
    let _ = seed_node_in(&pool, node_id, &tenant).await;
    for i in 1..=2 {
        enqueue_in(&state, node_id, &format!("cmd_roll_{i}"), &tenant).await;
    }
    state.acknowledge(node_id, 2, Utc::now()).await.unwrap();
    sqlx::query(
        "UPDATE node_inbox SET acknowledged_at = now() - interval '10 days' WHERE node_id = $1",
    )
    .bind(node_id)
    .execute(&pool)
    .await
    .expect("age the rows");

    // ⛔ The guard mode did not change in this repair, so no lock fixture can
    // discriminate it. THIS is the discriminating control: a prune is the most
    // destructive of the three, and its deletion must not survive evidence it
    // could not write. NOT VALID applies to new rows only.
    sqlx::query(
        "ALTER TABLE administrative_effects \
         ADD CONSTRAINT signoff_repair_3_3_4_10_3_evidence_fault CHECK (false) NOT VALID",
    )
    .execute(&pool)
    .await
    .expect("install the evidence fault");

    // Observe, REMOVE THE FAULT, then assert.
    let (status, receipt, body) = inbox_post(
        &client,
        &base,
        "/v1/nodes/inbox/prune",
        &alice,
        json!({ "tenant_id": tenant, "node_id": node_id, "min_age_seconds": 0 }),
    )
    .await;
    let rows_after_fault = inbox_rows(&pool, node_id).await;

    sqlx::query(
        "ALTER TABLE administrative_effects \
         DROP CONSTRAINT signoff_repair_3_3_4_10_3_evidence_fault",
    )
    .execute(&pool)
    .await
    .expect("remove the evidence fault");

    assert_eq!(
        status, 500,
        "an evidence failure is not reported as success: {body}"
    );
    assert!(
        receipt.is_none(),
        "a transaction that did not commit advertises no receipt"
    );
    assert_eq!(
        rows_after_fault, 2,
        "the deletion did not survive its evidence"
    );

    // Exact recovery.
    let (status, receipt, body) = inbox_post(
        &client,
        &base,
        "/v1/nodes/inbox/prune",
        &alice,
        json!({ "tenant_id": tenant, "node_id": node_id, "min_age_seconds": 0 }),
    )
    .await;
    assert_eq!(status, 200, "the route recovers: {body}");
    assert_eq!(body["deleted"], json!(2));
    assert_eq!(inbox_rows(&pool, node_id).await, 0);
    let effect = inbox_effect(&pool, &tenant, receipt)
        .await
        .expect("recorded");
    assert!(effect.outcome.changed_protected_state());
}
