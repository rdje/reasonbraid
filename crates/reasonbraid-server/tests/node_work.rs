//! WP6 node-wiring integration tests (`PHASE-0.6.2`): the two-host demo's contract
//! over a live PostgreSQL — an accepted invite dispatches a work item (with a
//! reservation against the thread ceiling) into the invited role's inbox; a
//! node-emitted `work_result` folds into the thread through the SAME
//! claim → authorize → validate → apply flow a CLI command rides; duplicate
//! transport produces one domain effect; a challenge dispatches revise work; a
//! budget denial enqueues the work item WITHOUT a reservation (the node's budget
//! gate refuses the dispatch); and a result arriving after close is stored as the
//! work command's idempotent rejection.
//!
//! All channel traffic uses the PUBLIC surface (`handshake` reads the inbox —
//! inspection never needs database surgery); direct SQL is assertion-only.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job. Without
//! `DATABASE_URL` these skip, so `make check` stays green offline.

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_core::{CommandEnvelope, RequestId, PROTOCOL_VERSION};
use reasonbraid_server::{api_router, node_router, CHANNEL_VERSION, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

/// The wiring tests own the thread/channel/authority/budget tables: tests never
/// run concurrently against the same PG database.
static WIRING_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    WIRING_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the real node-wiring proof"
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
    // Purge in FK order: this suite exclusively owns these tables for its duration.
    for table in [
        "outbox_delivery",
        "outbox",
        "node_events",
        "node_inbox",
        "budget_reservations",
        "budget_ceilings",
        "authorization_records",
        "authority_grants",
        "enrollments",
        "enrollment_boundaries",
        "idempotency",
        "event_log",
        "aggregate_state",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&pool)
            .await
            .expect("purge table");
    }
    Some(pool)
}

/// The running server half: control API and node channel on one listener.
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
        let router = api_router(pool.clone()).merge(node_router(pool.clone()));
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

/// The public channel path: the node reports holding nothing and receives its
/// inbox replay.
async fn handshake(client: &reqwest::Client, base: &str, node_id: &str) -> Value {
    let response = client
        .post(format!("{base}/v1/nodes/handshake"))
        .json(&json!({
            "channel_version": CHANNEL_VERSION,
            "node_id": node_id,
            "last_acked_cursor": 0,
            "pending_operations": [],
            "ambiguous_attempts": [],
        }))
        .send()
        .await
        .expect("handshake request");
    assert_eq!(response.status().as_u16(), 200, "handshake succeeds");
    response.json().await.expect("handshake json")
}

async fn submit_event(
    client: &reqwest::Client,
    base: &str,
    node_id: &str,
    event_id: &str,
    operation_id: &str,
    payload: &Value,
) -> (u16, Value) {
    let response = client
        .post(format!("{base}/v1/nodes/events"))
        .json(&json!({
            "channel_version": CHANNEL_VERSION,
            "node_id": node_id,
            "event_id": event_id,
            "operation_id": operation_id,
            "payload": payload,
        }))
        .send()
        .await
        .expect("event request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("event json"))
}

/// The events of one thread, in order (the API inspection surface).
async fn thread_events(
    client: &reqwest::Client,
    base: &str,
    thread_id: &str,
    tenant: &str,
    principal: &str,
) -> Vec<Value> {
    let (status, body) = get(
        client,
        base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant}"),
        principal,
    )
    .await;
    assert_eq!(status, 200, "events view succeeds");
    body["events"].as_array().expect("events array").clone()
}

/// A full bootstrap: human (tenant owner) + one role, and a created thread.
/// Returns (tenant, human, role, thread).
async fn bootstrap(client: &reqwest::Client, base: &str) -> (String, String, String, String) {
    let (status, human) = enroll(
        client,
        base,
        json!({ "kind": "human", "name": "organizer" }),
    )
    .await;
    assert_eq!(status, 200, "human enrolls");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let (status, role) = enroll(
        client,
        base,
        json!({ "kind": "role", "name": "agent-a", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let (status, created) = command(
        client,
        base,
        "/v1/threads",
        &human_id,
        &envelope(
            "thread.create",
            "key-create",
            json!({
                "tenant_id": tenant,
                "subject": "two-host demo",
                "objective": "prove delivery survives crashes",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "thread creates: {created:?}");
    let thread = created["thread_id"].as_str().unwrap().to_string();
    (tenant, human_id, role_id, thread)
}

/// A `work_result` payload as the node emits it (`.6.2` contract).
fn work_result(
    command_id: &str,
    work_kind: &str,
    content: &str,
    reservation_id: &str,
    attempt_id: &str,
) -> Value {
    json!({
        "kind": "work_result",
        "work_kind": work_kind,
        "command_id": command_id,
        "reservation_id": reservation_id,
        "attempt_id": attempt_id,
        "content": content,
        "usage": { "input_tokens": 41, "output_tokens": 17 },
    })
}

/// THE `.6.2` contract: an accepted invite dispatches work into the invited
/// role's inbox in the SAME transaction as the invite event, and the work item
/// carries a reservation against the thread ceiling.
#[tokio::test]
async fn invite_dispatches_work_with_a_reservation() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread) = bootstrap(&client, &server.base()).await;

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-invite",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds");

    // The node reads its inbox through the PUBLIC channel surface.
    let handshake = handshake(&client, &server.base(), &role).await;
    let replay = handshake["replay"].as_array().expect("replay array");
    assert_eq!(
        replay.len(),
        1,
        "the invitation produced exactly one work item"
    );
    let work = &replay[0];
    assert_eq!(work["tenant_id"].as_str().unwrap(), tenant);
    assert_eq!(work["thread_id"].as_str().unwrap(), thread);
    assert!(
        work["command_id"]
            .as_str()
            .unwrap()
            .starts_with("work_evt_"),
        "the work command id correlates with its trigger event"
    );
    let payload = &work["payload"];
    assert_eq!(payload["kind"].as_str().unwrap(), "contribute");
    assert_eq!(payload["agent_role"].as_str().unwrap(), role);
    assert!(
        payload["reservation"]["reservation_id"].as_str().is_some(),
        "the work item carries a reservation"
    );
    assert!(
        payload["reservation"]["dimensions"]["calls"]
            .as_u64()
            .unwrap()
            >= 1,
        "the reservation covers a dispatch"
    );

    let active: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM budget_reservations WHERE status = 'active'")
            .fetch_one(&pool)
            .await
            .expect("count active reservations");
    assert_eq!(active, 1, "exactly one active reservation was issued");
}

/// A node-emitted work result folds into the thread as a contribution — and
/// duplicate transport produces exactly ONE domain effect at both layers (the
/// receipt dedupe and the idempotency claim).
#[tokio::test]
async fn node_result_becomes_one_contribution_despite_duplicates() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread) = bootstrap(&client, &server.base()).await;

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-invite",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds");

    let handshake = handshake(&client, &server.base(), &role).await;
    let work = handshake["replay"][0].clone();
    let command_id = work["command_id"].as_str().unwrap().to_string();
    let reservation_id = work["payload"]["reservation"]["reservation_id"]
        .as_str()
        .unwrap()
        .to_string();
    let result = work_result(
        &command_id,
        "contribute",
        "the agent's independent answer",
        &reservation_id,
        "att_00000000-0000-7000-8000-000000000001",
    );

    // First delivery: accepted, and the thread gains exactly one contribution.
    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        "evt_00000000-0000-7000-8000-000000000001",
        "op_00000000-0000-7000-8000-000000000001",
        &result,
    )
    .await;
    assert_eq!(status, 200, "event accepted");
    assert_eq!(receipt["accepted"], json!(true));

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    let contributions: Vec<&Value> = events
        .iter()
        .filter(|e| e["event_type"] == "thread.contribution_submitted")
        .collect();
    assert_eq!(contributions.len(), 1, "exactly one contribution exists");
    assert_eq!(
        contributions[0]["body"]["author"].as_str().unwrap(),
        role,
        "the contribution's author is the node's role"
    );
    assert_eq!(
        contributions[0]["body"]["content"].as_str().unwrap(),
        "the agent's independent answer",
        "the adapter content landed verbatim"
    );

    // The reservation settled with the reported usage.
    let settled: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM budget_reservations WHERE reservation_id = $1 AND status = 'settled'",
    )
    .bind(&reservation_id)
    .fetch_one(&pool)
    .await
    .expect("count settled");
    assert_eq!(settled, 1, "the reservation settled with the result");

    // Duplicate transport #1: the SAME event id → the receipt dedupe says no.
    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        "evt_00000000-0000-7000-8000-000000000001",
        "op_00000000-0000-7000-8000-000000000001",
        &result,
    )
    .await;
    assert_eq!(status, 200, "duplicate event answered");
    assert_eq!(
        receipt["accepted"],
        json!(false),
        "the duplicate was refused"
    );

    // Duplicate transport #2: a NEW event id re-delivering the SAME work result →
    // the idempotency claim replays the original application.
    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        "evt_00000000-0000-7000-8000-000000000002",
        "op_00000000-0000-7000-8000-000000000002",
        &result,
    )
    .await;
    assert_eq!(status, 200, "redelivered work result answered");
    assert_eq!(
        receipt["accepted"],
        json!(true),
        "the new receipt is recorded"
    );

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    let contributions: Vec<&Value> = events
        .iter()
        .filter(|e| e["event_type"] == "thread.contribution_submitted")
        .collect();
    assert_eq!(
        contributions.len(),
        1,
        "the redelivered result produced NO second domain effect"
    );

    // The audit trail records the role's authorized application.
    let (status, audit) = get(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/audit?tenant_id={tenant}"),
        &human,
    )
    .await;
    assert_eq!(status, 200, "audit view succeeds");
    let applied: Vec<&Value> = audit["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["action"] == "thread_contribute" && r["decision"] == "allowed")
        .collect();
    assert!(!applied.is_empty(), "the node's application is audited");
}

/// A challenge of a role's contribution dispatches revise work to that role's
/// node; the node's revision folds in and answers the challenge.
#[tokio::test]
async fn challenge_dispatches_revise_work_and_the_revision_lands() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread) = bootstrap(&client, &server.base()).await;

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-invite",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds");

    let first_view = handshake(&client, &server.base(), &role).await;
    let command_id = first_view["replay"][0]["command_id"]
        .as_str()
        .unwrap()
        .to_string();
    let reservation_id = first_view["replay"][0]["payload"]["reservation"]["reservation_id"]
        .as_str()
        .unwrap()
        .to_string();
    let (status, _) = submit_event(
        &client,
        &server.base(),
        &role,
        "evt_00000000-0000-7000-8000-000000000011",
        "op_00000000-0000-7000-8000-000000000011",
        &work_result(
            &command_id,
            "contribute",
            "answer one",
            &reservation_id,
            "att_00000000-0000-7000-8000-000000000011",
        ),
    )
    .await;
    assert_eq!(status, 200, "contribution lands");

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    let contribution_id = events
        .iter()
        .find(|e| e["event_type"] == "thread.contribution_submitted")
        .expect("the contribution exists")["event_id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.challenge",
            "key-challenge",
            json!({
                "tenant_id": tenant,
                "target_event_id": contribution_id,
                "content": "is this justified?",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "challenge succeeds");

    // The challenge dispatched revise work to the contribution's author (the role).
    let replay_view = handshake(&client, &server.base(), &role).await;
    let replay = replay_view["replay"].as_array().expect("replay array");
    assert_eq!(replay.len(), 2, "invite work + revise work");
    let revise = replay
        .iter()
        .find(|c| c["payload"]["kind"] == "revise")
        .expect("the revise work item exists");
    let challenge_id = thread_events(&client, &server.base(), &thread, &tenant, &human)
        .await
        .iter()
        .find(|e| e["event_type"] == "thread.challenge_posted")
        .expect("the challenge event exists")["event_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        revise["payload"]["target_event_id"].as_str().unwrap(),
        challenge_id,
        "the revise work targets the challenge (the domain's revise target)"
    );

    let (status, _) = submit_event(
        &client,
        &server.base(),
        &role,
        "evt_00000000-0000-7000-8000-000000000012",
        "op_00000000-0000-7000-8000-000000000012",
        &work_result(
            revise["command_id"].as_str().unwrap(),
            "revise",
            "the revision",
            revise["payload"]["reservation"]["reservation_id"]
                .as_str()
                .unwrap(),
            "att_00000000-0000-7000-8000-000000000012",
        ),
    )
    .await;
    assert_eq!(status, 200, "revision lands");

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    assert!(
        events
            .iter()
            .any(|e| e["event_type"] == "thread.revision_submitted"),
        "the revision is in the timeline"
    );
    let (status, body) = get(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}?tenant_id={tenant}"),
        &human,
    )
    .await;
    assert_eq!(status, 200, "state view succeeds");
    assert_eq!(
        body["state"]["open_challenges"].as_u64().unwrap(),
        0,
        "the revision answered the challenge"
    );
    assert_eq!(body["state"]["revisions"].as_u64().unwrap(), 1);
}

/// A ceiling that cannot cover the work reservation denies the dispatch AT THE
/// INVITE — the work item is still enqueued (without a reservation), so the
/// node's budget gate refuses `failed_before_dispatch` instead of calling a
/// provider.
#[tokio::test]
async fn budget_denial_enqueues_work_without_a_reservation() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, _thread) = bootstrap(&client, &server.base()).await;

    let (status, created) = command(
        &client,
        &server.base(),
        "/v1/threads",
        &human,
        &envelope(
            "thread.create",
            "key-create-tight",
            json!({
                "tenant_id": tenant,
                "subject": "tight budget",
                "objective": "deny the dispatch",
                "budget": { "calls": 0 },
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the tight thread creates");
    let thread = created["thread_id"].as_str().unwrap().to_string();

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-invite-tight",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(
        status, 200,
        "the invite itself succeeds (dispatch is denied, not the invite)"
    );

    let handshake = handshake(&client, &server.base(), &role).await;
    let work = &handshake["replay"][0];
    assert_eq!(work["payload"]["kind"].as_str().unwrap(), "contribute");
    assert!(
        work["payload"]["reservation"].is_null(),
        "no reservation: the ceiling refused"
    );
    assert!(
        work["payload"]["reservation_reason"]
            .as_str()
            .unwrap()
            .contains("budget"),
        "the denial reason rides with the work item"
    );

    let denied: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM budget_reservations WHERE status = 'denied'")
            .fetch_one(&pool)
            .await
            .expect("count denials");
    assert_eq!(denied, 1, "the denial is itself a recorded row");
}

/// A work result arriving after the thread closed is stored as the work
/// command's idempotent REJECTION — the receipt still stands (the node did emit
/// it), and a re-delivery reproduces the rejection without a domain effect.
#[tokio::test]
async fn result_after_close_is_stored_as_a_rejection() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread) = bootstrap(&client, &server.base()).await;

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-invite",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds");
    let handshake = handshake(&client, &server.base(), &role).await;
    let command_id = handshake["replay"][0]["command_id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.close",
            "key-close",
            json!({ "tenant_id": tenant, "reason": "time" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "thread closes");

    let result = work_result(
        &command_id,
        "contribute",
        "too late",
        "",
        "att_00000000-0000-7000-8000-000000000021",
    );
    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        "evt_00000000-0000-7000-8000-000000000021",
        "op_00000000-0000-7000-8000-000000000021",
        &result,
    )
    .await;
    assert_eq!(status, 200, "the receipt stands");
    assert_eq!(receipt["accepted"], json!(true));

    // The rejection is stored: a re-delivery (new event id, same work) replays it.
    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        "evt_00000000-0000-7000-8000-000000000022",
        "op_00000000-0000-7000-8000-000000000022",
        &result,
    )
    .await;
    assert_eq!(status, 200, "redelivery answered");
    assert_eq!(receipt["accepted"], json!(true));

    let stored: (Value,) = sqlx::query_as(
        "SELECT response_result FROM idempotency WHERE tenant_id = $1 AND idempotency_key = $2",
    )
    .bind(&tenant)
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("the stored result");
    assert_eq!(stored.0["ok"], json!(false), "the rejection was stored");
    assert_eq!(
        stored.0["error"]["code"].as_str().unwrap(),
        "invalid_transition",
        "the stored code names the transition refusal"
    );

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    assert!(
        !events
            .iter()
            .any(|e| e["event_type"] == "thread.contribution_submitted"),
        "no contribution entered the closed thread"
    );
    // Closure preserved the timeline as-is: the close event is last.
    assert_eq!(
        events.last().unwrap()["event_type"].as_str().unwrap(),
        "thread.closed"
    );
}

/// Ordinary channel events (WP3 traffic without a work payload) stay receipts:
/// no domain effect, no idempotency claim.
#[tokio::test]
async fn ordinary_channel_events_stay_receipts_only() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread) = bootstrap(&client, &server.base()).await;

    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        "evt_00000000-0000-7000-8000-000000000031",
        "op_00000000-0000-7000-8000-000000000031",
        &json!({ "kind": "some_other_signal", "value": 1 }),
    )
    .await;
    assert_eq!(status, 200, "the event is accepted");
    assert_eq!(receipt["accepted"], json!(true));

    let claims: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM idempotency WHERE tenant_id = $1")
        .bind(&tenant)
        .fetch_one(&pool)
        .await
        .expect("count claims");
    assert_eq!(claims, 1, "only the thread-create command claimed a key");
    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    assert_eq!(events.len(), 1, "the thread timeline is untouched");
}
