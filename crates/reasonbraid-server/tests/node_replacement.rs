//! The node-replacement drill (`PHASE-2.7.2`): the total-machine-loss path of the
//! node lost/replaced runbook, measured over live PostgreSQL. The scenario: a node
//! DISPATCHES work (the response is lost — the attempt lands its honest terminal
//! `outcome_unknown` in the journal), then the machine dies and its journal is
//! DESTROYED. The ritual: the operator revokes the dead credential (the old cert is
//! fenced, the tenant's revocation epoch bumps), issues a fresh token, and the
//! REPLACEMENT enrolls (a new cert + a new incarnation; the old certs stay revoked).
//!
//! The measured recovery chain: the fresh journal reconciles and the inbox tail
//! replays; the re-delivered work carries the decision the LOST incarnation cached
//! (epoch 0), so the replacement's dispatch refuses FAIL-CLOSED against the current
//! epoch (1) — the no-false-safe-retry fence, measured, never argued — the row
//! dead-letters (auto-quarantine), the operator's `POST /v1/nodes/replay` refreshes
//! the decision, and the replacement completes the work: exactly one contribution.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job. Without
//! `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use reasonbraid_core::{CommandEnvelope, RequestId, PROTOCOL_VERSION};
use reasonbraid_server::{
    api_router, ca::ensure_server_ca, node_router, CHANNEL_VERSION, PRINCIPAL_HEADER,
};
use serde_json::{json, Value};
use sqlx::PgPool;

/// The dev signing secret every node in this suite enrolls with (the dev
/// trust-store stance; the `.1.2.2` handshake proves possession of it).
const DEV_SECRET: &str = "dev-secret";
const DEV_SECRET_TWO: &str = "dev-secret-replacement";

static DRILL_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    DRILL_LOCK
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
    // Purge in FK order: this suite exclusively owns these tables for its duration.
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
            "evidence_citations",
            "claim_assessments",
            "derivations",
            "evidence_snapshots",
            "reference_registrations",
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

fn envelope(operation: &str, key: &str, body: Value) -> CommandEnvelope {
    CommandEnvelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        operation: operation.to_string(),
        request_id: RequestId::new(),
        idempotency_key: key.to_string(),
        expected_aggregate_version: None,
        body,
        authority_context: None,
        client_context: Default::default(),
    }
}

async fn enroll(client: &reqwest::Client, base: &str, body: Value) -> (u16, Value) {
    let response = client
        .post(format!("{base}/v1/enrollments"))
        .json(&body)
        .send()
        .await
        .expect("enroll request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("enroll json"))
}

async fn command(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    env: &CommandEnvelope,
) -> (u16, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(env)
        .send()
        .await
        .expect("command request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("command json"))
}

async fn get(client: &reqwest::Client, base: &str, path: &str, principal: &str) -> (u16, Value) {
    let response = client
        .get(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .send()
        .await
        .expect("get request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("get json"))
}

fn from_hex(hex: &str) -> Option<Vec<u8>> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

fn journal_path(name: &str) -> std::path::PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target")
        });
    let unique = uuid::Uuid::now_v7();
    let dir = base
        .join("node-replacement")
        .join(format!("{name}-{unique}"));
    std::fs::create_dir_all(dir.parent().expect("the fixture parent")).unwrap();
    // Exclusive: an existing directory belongs to another fixture or an
    // earlier run, and must never be adopted.
    std::fs::DirBuilder::new()
        .create(&dir)
        .expect("the fixture directory is new");
    dir
}

/// Enroll a node through the PUBLIC surface (`.1.2.1`).
async fn enroll_node(
    client: &reqwest::Client,
    base: &str,
    human: &str,
    tenant: &str,
    node_id: &str,
    secret: &str,
) -> (String, String) {
    let response = client
        .post(format!("{base}/v1/nodes/enroll-tokens"))
        .header(PRINCIPAL_HEADER, human)
        .json(&json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "host_claim": "seed-host",
        }))
        .send()
        .await
        .expect("issue-token request");
    assert_eq!(response.status().as_u16(), 200, "the token issues");
    let issued: Value = response.json().await.expect("issue-token json");

    let response = client
        .post(format!("{base}/v1/nodes/enroll"))
        .json(&json!({
            "token_id": issued["token_id"],
            "node_id": node_id,
            "host_claim": "seed-host",
            "nonce": issued["nonce"],
            "key_secret": secret,
        }))
        .send()
        .await
        .expect("enroll request");
    assert_eq!(response.status().as_u16(), 200, "the node enrolls");
    let body: Value = response.json().await.expect("enroll json");
    (
        body["cert_der"].as_str().unwrap().to_string(),
        body["key_der"].as_str().unwrap().to_string(),
    )
}

/// The authenticated handshake; `expect_ok` false expects the proof refusal (the
/// revoked-credential fence).
async fn handshake(
    client: &reqwest::Client,
    base: &str,
    node_id: &str,
    cert_hex: &str,
    key_hex: &str,
    expect_ok: bool,
) -> (u16, Value) {
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
    let status = response.status().as_u16();
    let body: Value = response.json().await.expect("handshake json");
    if expect_ok {
        assert_eq!(status, 200, "handshake succeeds: {body}");
    } else {
        assert_ne!(status, 200, "the revoked credential is fenced: {body}");
    }
    (status, body)
}

async fn accept_invitation(
    client: &reqwest::Client,
    base: &str,
    role: &str,
    thread: &str,
    tenant: &str,
    key: &str,
) {
    let (status, accepted) = command(
        client,
        base,
        &format!("/v1/threads/{thread}/commands"),
        role,
        &envelope(
            "thread.accept_invitation",
            key,
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the invited role accepts: {accepted}");
}

/// THE `.7.2` drill, one continuous scenario: dispatch → total loss → revoke →
/// replacement enroll → inbox replay → exactly one fold.
#[tokio::test]
async fn the_replacement_ritual_recovers_a_lost_node() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // Bootstrap: human + role (node==role) + thread + invite + accept → the work
    // item dispatches into the role's inbox.
    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "organizer" }),
    )
    .await;
    assert_eq!(status, 200, "human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "agent-a", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let (cert_hex, key_hex) =
        enroll_node(&client, &base, &human_id, &tenant, &role_id, DEV_SECRET).await;

    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &envelope(
            "thread.create",
            "key-create",
            json!({ "tenant_id": tenant, "subject": "replacement drill", "objective": "prove the ritual" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "thread creates: {created}");
    let thread = created["thread_id"].as_str().unwrap().to_string();
    let (status, _) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &human_id,
        &envelope(
            "thread.invite",
            "key-invite",
            json!({ "tenant_id": tenant, "agent_role": role_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds");
    accept_invitation(&client, &base, &role_id, &thread, &tenant, "key-accept").await;

    // Phase 1: the ORIGINAL node reconciles, receives the work, and DISPATCHES —
    // the response is LOST (the stream ends without a terminal), so the attempt
    // lands its HONEST terminal `outcome_unknown` in the node's journal. Then the
    // machine dies and the journal is DESTROYED (the total-machine-loss path).
    let journal_one = journal_path("drill-lost");
    {
        let key_der = from_hex(&key_hex).expect("key hex");
        let der = rustls_pki_types::PrivateKeyDer::try_from(key_der).expect("key DER");
        let key = rcgen::KeyPair::from_der_and_sign_algo(&der, &rcgen::PKCS_ECDSA_P256_SHA256)
            .expect("key parses");
        let node = reasonbraid_node::Node::open(
            journal_one.join("node.db"),
            base.clone(),
            role_id.clone(),
            from_hex(&cert_hex).expect("cert hex"),
            key,
        )
        .await
        .expect("open the lost node");
        node.reconcile().await.expect("reconcile the lost node");
        let work = node.journal().work_items().await.expect("work items");
        assert_eq!(work.len(), 1, "the work item delivered to the lost node");
        let worker = reasonbraid_node::Worker::new(
            node.clone(),
            reasonbraid_adapter::FakeAdapter::new(
                vec![reasonbraid_adapter::ScriptStep::LoseResponse],
                reasonbraid_adapter::StatusLookupSpec::Unsupported,
                reasonbraid_adapter::AdapterCapabilities {
                    streaming: true,
                    cancellation: reasonbraid_adapter::CancellationStrength::BestEffort,
                    provider_idempotency: false,
                    status_lookup: false,
                    tool_support: false,
                    policy_injection: reasonbraid_adapter::PolicyInjectionMode::None,
                },
            ),
            reasonbraid_node::LocalBudget::new(reasonbraid_core::BudgetDimensions {
                calls: Some(100),
                input_tokens: Some(100_000),
                output_tokens: Some(100_000),
                wall_clock_seconds: Some(10_000),
            }),
            std::time::Duration::from_millis(50),
        );
        // One tick: the dispatch crosses the boundary, the response is lost, and
        // the supervisor surfaces the ambiguity as the honest `OutcomeUnknown`
        // error — never a silent retry, never a fabricated terminal.
        match worker.tick().await {
            Err(reasonbraid_node::WorkerError::Supervision(
                reasonbraid_node::SupervisorError::OutcomeUnknown { .. },
            )) => {}
            other => panic!("the lost response must surface the honest ambiguity, got {other:?}"),
        }
        let refreshed = node.journal().work_items().await.expect("work items");
        let operation_id = refreshed[0]
            .operation_id
            .as_deref()
            .expect("the command has a local operation");
        let attempts = node
            .journal()
            .attempts_for_operation(operation_id)
            .await
            .expect("the attempt record");
        assert_eq!(attempts.len(), 1, "the dispatch journaled one attempt");
        assert_eq!(
            attempts[0].status, "outcome_unknown",
            "the lost response lands the honest terminal in the journal"
        );
    }
    // The total machine loss: the journal directory is destroyed — the
    // `outcome_unknown` fact dies with the machine.
    std::fs::remove_dir_all(&journal_one).expect("the machine burned down");

    // Phase 2: the ritual. The operator revokes the dead credential — the old
    // cert's handshake is fenced (the proof refusal), and the epoch bumped.
    let (status, revoked) = admin_revoke(
        &client,
        &base,
        "/v1/nodes/revoke",
        &human_id,
        &tenant,
        &role_id,
    )
    .await;
    assert_eq!(status, 200, "the revocation declares the loss: {revoked}");
    let (_fence_status, fenced) =
        handshake(&client, &base, &role_id, &cert_hex, &key_hex, false).await;
    assert!(
        fenced["message"].as_str().unwrap_or("").contains("proof")
            || fenced["code"].as_str().is_some(),
        "the old credential refuses with a typed proof error: {fenced}"
    );

    // The REPLACEMENT enroll (the `.7.2` machinery): the node row exists but every
    // cert is revoked, so the fresh token + secret enroll a NEW incarnation.
    let (cert_hex2, key_hex2) =
        enroll_node(&client, &base, &human_id, &tenant, &role_id, DEV_SECRET_TWO).await;
    let (n_incarnations,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM incarnations WHERE role_id = $1")
            .bind(&role_id)
            .fetch_one(&pool)
            .await
            .expect("count incarnations");
    assert_eq!(n_incarnations, 2, "the replacement is a NEW incarnation");
    let (decision,): (String,) = sqlx::query_as(
        "SELECT decision FROM node_enroll_audit WHERE node_id = $1 ORDER BY decision DESC LIMIT 1",
    )
    .bind(&role_id)
    .fetch_one(&pool)
    .await
    .expect("the audit decision");
    assert_eq!(decision, "replaced", "the replacement is audited as such");

    // The presence derivation (0017): the replacement is NOT suspended (the old
    // certs stay revoked — still fenced — but an active cert serves the new one).
    let (status, presence) = get(
        &client,
        &base,
        &format!("/v1/nodes/presence?tenant_id={tenant}&node_id={role_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "presence reads: {presence}");
    assert_eq!(
        presence["suspended"],
        json!(false),
        "the replacement reads NOT suspended: {presence}"
    );

    // Phase 3: the replacement node opens a FRESH journal and reconciles — the
    // inbox tail replays (cursor 0 → full tail) and the work re-delivers.
    let journal_two = journal_path("drill-replacement");
    {
        let key_der = from_hex(&key_hex2).expect("key hex");
        let der = rustls_pki_types::PrivateKeyDer::try_from(key_der).expect("key DER");
        let key = rcgen::KeyPair::from_der_and_sign_algo(&der, &rcgen::PKCS_ECDSA_P256_SHA256)
            .expect("key parses");
        let node = reasonbraid_node::Node::open(
            journal_two.join("node.db"),
            base.clone(),
            role_id.clone(),
            from_hex(&cert_hex2).expect("cert hex"),
            key,
        )
        .await
        .expect("open the replacement node");
        node.reconcile().await.expect("reconcile the replacement");
        let work = node.journal().work_items().await.expect("work items");
        assert_eq!(
            work.len(),
            1,
            "the inbox tail replays to the fresh journal (the durability leg)"
        );

        // THE no-false-safe-retry fence, measured: the revocation bumped the
        // tenant's epoch, and the re-delivery carries the decision the LOST
        // incarnation cached (epoch 0) — so the replacement's dispatch refuses
        // fail-closed (a stale decision, never a silent re-dispatch of the lost
        // node's in-flight work), the bounded retry budget exhausts, and the row
        // dead-letters (the server's auto-quarantine).
        let work_command_id = work[0].command_id.clone();
        let worker = reasonbraid_node::Worker::new(
            node.clone(),
            reasonbraid_adapter::FakeAdapter::new(
                vec![reasonbraid_adapter::ScriptStep::FailBeforeDispatch {
                    reason: "provider unavailable".to_string(),
                }],
                reasonbraid_adapter::StatusLookupSpec::Unsupported,
                reasonbraid_adapter::AdapterCapabilities {
                    streaming: false,
                    cancellation: reasonbraid_adapter::CancellationStrength::BestEffort,
                    provider_idempotency: false,
                    status_lookup: false,
                    tool_support: false,
                    policy_injection: reasonbraid_adapter::PolicyInjectionMode::None,
                },
            ),
            reasonbraid_node::LocalBudget::new(reasonbraid_core::BudgetDimensions {
                calls: Some(100),
                input_tokens: Some(100_000),
                output_tokens: Some(100_000),
                wall_clock_seconds: Some(10_000),
            }),
            std::time::Duration::from_millis(50),
        );
        for _ in 0..4 {
            let _ = worker.tick().await; // the refusals + the terminal report
        }
        let quarantine: (Option<chrono::DateTime<chrono::Utc>>, Option<String>) = sqlx::query_as(
            "SELECT quarantined_at, quarantine_reason FROM node_inbox \
             WHERE node_id = $1 AND command_id = $2",
        )
        .bind(&role_id)
        .bind(&work_command_id)
        .fetch_one(&pool)
        .await
        .expect("the row");
        assert!(
            quarantine.0.is_some(),
            "the stale-decision re-delivery dead-lettered (auto-quarantine)"
        );
        assert!(
            quarantine
                .1
                .as_deref()
                .is_some_and(|r| r.contains("bounded retry budget is exhausted")),
            "the quarantine carries the terminal reason: {:?}",
            quarantine.1
        );

        // THE operator's explicit recovery: `POST /v1/nodes/replay` clears the
        // quarantine, refreshes the admission decision against the current epoch,
        // and re-sequences the row — then the replacement's worker completes it.
        let response = client
            .post(format!("{base}/v1/nodes/replay"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "tenant_id": tenant,
                "node_id": role_id,
                "command_id": work_command_id,
            }))
            .send()
            .await
            .expect("replay request");
        assert_eq!(
            response.status().as_u16(),
            200,
            "the operator replay succeeds"
        );

        let completing_worker = reasonbraid_node::Worker::new(
            node.clone(),
            reasonbraid_adapter::FakeAdapter::new(
                vec![
                    reasonbraid_adapter::ScriptStep::EmitChunk {
                        chunk: "recovered".to_string(),
                    },
                    reasonbraid_adapter::ScriptStep::Complete {
                        usage: Some(
                            json!({ "input_tokens": 1, "output_tokens": 1, "exact": true }),
                        ),
                    },
                ],
                reasonbraid_adapter::StatusLookupSpec::Unsupported,
                reasonbraid_adapter::AdapterCapabilities {
                    streaming: true,
                    cancellation: reasonbraid_adapter::CancellationStrength::BestEffort,
                    provider_idempotency: false,
                    status_lookup: false,
                    tool_support: false,
                    policy_injection: reasonbraid_adapter::PolicyInjectionMode::None,
                },
            ),
            reasonbraid_node::LocalBudget::new(reasonbraid_core::BudgetDimensions {
                calls: Some(100),
                input_tokens: Some(100_000),
                output_tokens: Some(100_000),
                wall_clock_seconds: Some(10_000),
            }),
            std::time::Duration::from_millis(50),
        );
        for _ in 0..4 {
            let _ = completing_worker.tick().await;
        }
    }

    // Exactly ONE contribution folded (the re-delivery produced no duplicate).
    let (n_contributions,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM event_log WHERE tenant_id = $1 AND event_type = 'thread.contribution_submitted'",
    )
    .bind(&tenant)
    .fetch_one(&pool)
    .await
    .expect("count contributions");
    assert_eq!(
        n_contributions, 1,
        "the recovery folded exactly one contribution"
    );
}

/// The admin node-revocation verb (`.1.3.1`).
async fn admin_revoke(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    tenant: &str,
    node_id: &str,
) -> (u16, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(&json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "reason": "the machine burned down",
        }))
        .send()
        .await
        .expect("revoke request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("revoke json"))
}

/// Enrol a node onto a NAMED host (the shared helper hardcodes `seed-host`).
/// Returns the enrollment response, because `.4.1.5` asserts against the
/// `host_id` it echoes as well as against the row it wrote.
async fn enroll_node_on_host(
    client: &reqwest::Client,
    base: &str,
    human: &str,
    tenant: &str,
    node_id: &str,
    host_claim: &str,
    secret: &str,
) -> Value {
    let response = client
        .post(format!("{base}/v1/nodes/enroll-tokens"))
        .header(PRINCIPAL_HEADER, human)
        .json(&json!({ "tenant_id": tenant, "node_id": node_id, "host_claim": host_claim }))
        .send()
        .await
        .expect("issue-token request");
    assert_eq!(response.status().as_u16(), 200, "the token issues");
    let issued: Value = response.json().await.expect("issue-token json");

    let response = client
        .post(format!("{base}/v1/nodes/enroll"))
        .json(&json!({
            "token_id": issued["token_id"],
            "node_id": node_id,
            "host_claim": host_claim,
            "nonce": issued["nonce"],
            "key_secret": secret,
        }))
        .send()
        .await
        .expect("enroll request");
    assert_eq!(
        response.status().as_u16(),
        200,
        "the node enrols onto {host_claim}"
    );
    response.json().await.expect("enroll json")
}

/// `SIGNOFF-REPAIR.4.1.5` — what a replacement enrollment ENDS.
///
/// Three clauses of one source-census record (`census-2.md:68`), driven together
/// because they are one question. `the_replacement_ritual_recovers_a_lost_node`
/// proves the ritual's happy path — the work reaches the new machine — and
/// asserts none of these:
///
///  1. the replacement kept the OLD `nodes.host_id`, and `rotate` reads the
///     certificate's host from THAT row, so the first automatic rotation
///     reverted the SAN to the machine the node no longer runs on;
///  2. the previous incarnation's `valid_to` was never closed, so a replaced
///     role had two rows each reading as current;
///  3. the old lease was left untouched — and because the replacement issues a
///     FRESH certificate for the same node id, the credential predicate
///     `.4.1.3` put into `renew_lease` became true again, handing the REPLACED
///     process its session back.
///
/// ⚠️ Clause 3 is bounded and the bound is what makes it reachable: the old
/// lease must still be live, i.e. the replacement happens within `LEASE_TTL` of
/// the revocation. That is the ordinary operational case — an operator revokes
/// a dead machine and enrols its replacement in the same minute.
///
/// ⛔ It does NOT conflict with `.4.1.3.1`, which preserves the revoked node's
/// WITHHELD WORK so it replays to the replacement. That decision is about inbox
/// rows; this is about the lease. The work still reaches the new machine —
/// asserted below, so the two properties are held together rather than traded.
#[tokio::test]
async fn a_replacement_ends_the_old_machines_session_host_and_incarnation() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "operator" }),
    )
    .await;
    assert_eq!(status, 200, "human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "agent-replaced", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls: {role}");
    let node_id = role["principal_id"].as_str().unwrap().to_string();

    // ── The ORIGINAL machine, on host-a, with a live session ─────────────────
    let first = enroll_node_on_host(
        &client, &base, &human_id, &tenant, &node_id, "host-a", DEV_SECRET,
    )
    .await;
    let (cert_a, key_a) = (
        first["cert_der"].as_str().unwrap().to_string(),
        first["key_der"].as_str().unwrap().to_string(),
    );
    let (_, session) = handshake(&client, &base, &node_id, &cert_a, &key_a, true).await;
    let token_a = session["fencing_token"].as_str().unwrap().to_string();
    let epoch_a = session["lease_epoch"].as_i64().unwrap();

    let heartbeat = async |token: &str, epoch: i64| -> u16 {
        client
            .post(format!("{base}/v1/nodes/heartbeat"))
            .json(&json!({
                "channel_version": CHANNEL_VERSION,
                "node_id": node_id,
                "fencing_token": token,
                "lease_epoch": epoch,
            }))
            .send()
            .await
            .expect("heartbeat")
            .status()
            .as_u16()
    };
    assert_eq!(
        heartbeat(&token_a, epoch_a).await,
        200,
        "sanity: the original machine's session is live before anything happens"
    );

    // ── The machine is lost: the operator revokes, then enrols the replacement
    //    onto a DIFFERENT host, inside the lease window. ────────────────────
    let (status, revoked) = admin_revoke(
        &client,
        &base,
        "/v1/nodes/revoke",
        &human_id,
        &tenant,
        &node_id,
    )
    .await;
    assert_eq!(status, 200, "the node is revoked: {revoked}");

    let second = enroll_node_on_host(
        &client,
        &base,
        &human_id,
        &tenant,
        &node_id,
        "host-b",
        "dev-secret-two",
    )
    .await;

    // ── CLAUSE 3: the replaced machine's session is OVER ─────────────────────
    let after = heartbeat(&token_a, epoch_a).await;
    println!("  .4.1.5 the replaced machine's heartbeat after the replacement: {after}");
    assert_ne!(
        after, 200,
        "the REPLACED machine must not renew its lease. Its certificate was \
         revoked — but the replacement issues a fresh one for the same node id, \
         so `renew_lease`'s credential predicate becomes true again and hands \
         the old process its session back unless the replacement ends the lease"
    );

    // ── CLAUSE 1: the host the node now runs on ──────────────────────────────
    assert_eq!(
        second["host_id"].as_str().map(|s| !s.is_empty()),
        Some(true),
        "the response echoes a host id"
    );
    let (row_host,): (String,) = sqlx::query_as(
        "SELECT h.name FROM nodes n JOIN hosts h ON h.host_id = n.host_id WHERE n.node_id = $1",
    )
    .bind(&node_id)
    .fetch_one(&pool)
    .await
    .expect("read the node's host");
    println!("  .4.1.5 nodes.host_id after the replacement names: {row_host}");
    assert_eq!(
        row_host, "host-b",
        "the replacement moved the node to host-b, and the row must say so — \
         `rotate` reads the certificate's host from THIS row, so a stale value \
         reverts the SAN to the machine the node no longer runs on"
    );

    // And the consequence itself, driven rather than inferred: a rotation
    // issues a certificate naming the CURRENT host.
    let cert_b = second["cert_der"].as_str().unwrap().to_string();
    let key_b = second["key_der"].as_str().unwrap().to_string();
    let key_der = from_hex(&key_b).expect("key hex");
    let der = rustls_pki_types::PrivateKeyDer::try_from(key_der).expect("key DER");
    let key = rcgen::KeyPair::from_der_and_sign_algo(&der, &rcgen::PKCS_ECDSA_P256_SHA256)
        .expect("key parses");
    let nonce = reasonbraid_node::fresh_proof_nonce();
    let proof =
        reasonbraid_node::compute_rotate_proof(&key, CHANNEL_VERSION, &node_id, &cert_b, &nonce);
    let rotated: Value = client
        .post(format!("{base}/v1/nodes/rotate"))
        .json(&json!({
            "channel_version": CHANNEL_VERSION,
            "node_id": node_id,
            "cert_der": cert_b,
            "proof_signature": proof,
            "nonce": nonce,
        }))
        .send()
        .await
        .expect("rotate request")
        .json()
        .await
        .expect("rotate json");
    let rotated_der = from_hex(rotated["cert_der"].as_str().expect("a rotated certificate"))
        .expect("rotated cert hex");
    let (_, parsed) = x509_parser::parse_x509_certificate(&rotated_der).expect("the leaf parses");
    let sans = parsed
        .subject_alternative_name()
        .ok()
        .flatten()
        .map(|san| format!("{:?}", san.value.general_names))
        .unwrap_or_default();
    println!("  .4.1.5 the rotated certificate's SAN: {sans}");
    assert!(
        sans.contains("host-b"),
        "the rotated certificate must name the host the node RUNS on: {sans}"
    );
    assert!(
        !sans.contains("host-a"),
        "and must not revert to the machine it was replaced from: {sans}"
    );

    // ── CLAUSE 2: exactly one incarnation reads as current ───────────────────
    let (open, total): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM incarnations WHERE role_id = $1 AND valid_to IS NULL), \
                (SELECT count(*) FROM incarnations WHERE role_id = $1)",
    )
    .bind(&node_id)
    .fetch_one(&pool)
    .await
    .expect("count the incarnations");
    println!("  .4.1.5 incarnations: {open} open of {total}");
    assert_eq!(
        total, 2,
        "the replacement records its own incarnation — §8.1 history is kept, \
         never overwritten"
    );
    assert_eq!(
        open, 1,
        "but exactly ONE reads as current: an incarnation that never ends \
         cannot be attributed against, which is what §8.1 exists for"
    );

    // ── `.4.1.3.1` HELD: the replacement's own session still works ───────────
    let (_, fresh) = handshake(&client, &base, &node_id, &cert_b, &key_b, true).await;
    assert!(
        fresh["fencing_token"].as_str().is_some(),
        "the REPLACEMENT handshakes and takes its own lease — the two properties \
         are held together, not traded: {fresh}"
    );
}
