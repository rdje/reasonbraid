//! The quota/abuse proof (`PHASE-7.1.3.2`, ADR-034 §16.11, migration 0047):
//! the per-tenant INVITE bound — the invitation-storm surface. Measured:
//!   - the enroll path creates the tenant's default quota (the bound exists
//!     WITH the tenant);
//!   - invites under the ceiling pass, each recording a `use` event;
//!   - the ceiling-th crossing invite is the typed `quota_exceeded` refusal
//!     (429) AND the denial row is RECORDED (never silent);
//!   - the window slide releases the bound (the old uses age out);
//!   - a tenant with NO quota row is the typed `quota_unconfigured`
//!     refusal (fail-closed — the surface refuses until a bound exists).
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_core::{ClientContext, CommandEnvelope, RequestId, PROTOCOL_VERSION};
use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static QUOTA_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    QUOTA_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the quota proof"
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
    for table in [
        "quota_events",
        "usage_quotas",
        "outbox_delivery",
        "outbox",
        "budget_reservations",
        "budget_ceilings",
        "authorization_records",
        "authority_grants",
        "enrollments",
        "enrollment_boundaries",
        "profile_versions",
        "agent_profiles",
        "agent_roles",
        "human_principals",
        "idempotency",
        "event_log",
        "aggregate_state",
        "tenants",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&pool)
            .await
            .expect("purge table");
    }
    Some(pool)
}

struct TestServer {
    addr: SocketAddr,
    _handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn start(pool: &PgPool) -> Self {
        let router = api_router(pool.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().unwrap();
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
        client_context: ClientContext::default(),
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

#[tokio::test]
async fn the_invite_quota_bounds_the_storm_and_records_every_refusal() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // 1. The bootstrap human creates the tenant — the enroll path creates the
    //    default invite quota IN the same transaction (a tenant exists with
    //    its bounds).
    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "quo-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant = human["tenant_id"].as_str().unwrap().to_string();

    let default: Option<(String, i64, i64)> = sqlx::query_as(
        "SELECT scope_id, ceiling, window_seconds FROM usage_quotas \
         WHERE tenant_id = $1 AND scope_kind = 'tenant'",
    )
    .bind(&tenant)
    .fetch_optional(&pool)
    .await
    .expect("read the default quota");
    let Some((scope_id, ceiling, window)) = default else {
        panic!("the enroll path must create the tenant's default quota");
    };
    assert_eq!(scope_id, tenant, "the default quota scopes the tenant");
    assert_eq!(ceiling, 1000, "the dev default ceiling");
    assert_eq!(window, 3600, "the dev default window");

    // The measured bound: shrink the ceiling to 2.
    sqlx::query(
        "UPDATE usage_quotas SET ceiling = 2 WHERE tenant_id = $1 AND scope_kind = 'tenant'",
    )
    .bind(&tenant)
    .execute(&pool)
    .await
    .expect("shrink the ceiling");

    // 2. A thread + three roles (the invite targets).
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &envelope(
            "thread.create",
            "q-create",
            json!({
                "tenant_id": tenant,
                "subject": "the storm",
                "objective": "bound the invitations",
                "budget": { "calls": 3 },
                "participant_rules": { "allow_explicit_invites": true, "allow_join_requests": false },
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the thread creates: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let mut roles = Vec::new();
    for name in ["quo-role-1", "quo-role-2", "quo-role-3", "quo-role-4"] {
        let (status, role) = enroll(
            &client,
            &base,
            json!({ "kind": "role", "name": name, "tenant_id": tenant }),
        )
        .await;
        assert_eq!(status, 200, "the role enrolls: {role}");
        roles.push(role["principal_id"].as_str().unwrap().to_string());
    }

    let invite = |key: &str, role: &str| {
        envelope(
            "thread.invite",
            key,
            json!({ "tenant_id": tenant, "agent_role": role }),
        )
    };
    let path = format!("/v1/threads/{thread_id}/commands");

    // 3. Two invites pass (the uses record); the THIRD crosses the ceiling —
    //    the typed 429 + the recorded denial row (never silent).
    for (key, role) in [("q-i1", &roles[0]), ("q-i2", &roles[1])] {
        let (status, invited) = command(&client, &base, &path, &human_id, &invite(key, role)).await;
        assert_eq!(
            status, 200,
            "the invite under the ceiling passes: {invited}"
        );
    }

    let (status, refused) =
        command(&client, &base, &path, &human_id, &invite("q-i3", &roles[2])).await;
    assert_eq!(
        status, 429,
        "the ceiling-th crossing invite refuses: {refused}"
    );
    assert_eq!(refused["code"], json!("quota_exceeded"));

    let uses: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM quota_events e JOIN usage_quotas q ON q.quota_id = e.quota_id \
         WHERE q.tenant_id = $1 AND e.kind = 'use'",
    )
    .bind(&tenant)
    .fetch_one(&pool)
    .await
    .expect("count uses");
    assert_eq!(uses, 2, "each accepted invite recorded exactly one use");
    let denials: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM quota_events e JOIN usage_quotas q ON q.quota_id = e.quota_id \
         WHERE q.tenant_id = $1 AND e.kind = 'denial'",
    )
    .bind(&tenant)
    .fetch_one(&pool)
    .await
    .expect("count denials");
    assert_eq!(denials, 1, "the refusal is a RECORDED denial, never silent");

    // 4. The window slide: the uses age out — the bound releases.
    sqlx::query(
        "UPDATE quota_events SET at = now() - interval '2 hours' \
         WHERE quota_id IN (SELECT quota_id FROM usage_quotas WHERE tenant_id = $1)",
    )
    .bind(&tenant)
    .execute(&pool)
    .await
    .expect("age the events out");
    let (status, invited) =
        command(&client, &base, &path, &human_id, &invite("q-i4", &roles[2])).await;
    assert_eq!(status, 200, "the slid window releases the bound: {invited}");

    // 5. The fail-closed stance: a tenant with NO quota row is the typed
    //    `quota_unconfigured` refusal — the surface refuses until a bound
    //    exists.
    let quota_id = format!("quo_{tenant}_invites");
    sqlx::query("DELETE FROM quota_events WHERE quota_id = $1")
        .bind(&quota_id)
        .execute(&pool)
        .await
        .expect("remove the quota events");
    sqlx::query("DELETE FROM usage_quotas WHERE tenant_id = $1")
        .bind(&tenant)
        .execute(&pool)
        .await
        .expect("remove the quota");
    let (status, refused) =
        command(&client, &base, &path, &human_id, &invite("q-i5", &roles[3])).await;
    assert_eq!(status, 503, "the unconfigured scope refuses: {refused}");
    assert_eq!(refused["code"], json!("quota_unconfigured"));
    let uses: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM quota_events WHERE quota_id = $1 AND kind = 'use'",
    )
    .bind(&quota_id)
    .fetch_one(&pool)
    .await
    .expect("count uses");
    assert_eq!(
        uses, 0,
        "the refused unconfigured invite recorded NO use (nothing new after the removal)"
    );
}
