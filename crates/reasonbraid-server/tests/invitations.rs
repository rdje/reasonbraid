//! Integration tests for the explicit-participants contract (`PHASE-1.3.1`;
//! backlog 16 + 15): the invitation lifecycle — invite records a PENDING offer
//! (typed optional expiry), accept/decline/remove verbs, DERIVED expiry, the
//! invitation as the acceptance capability — and the dispatch move (work rides
//! the ACCEPT transaction, never the invite). Race tests prove exactly one
//! winner under concurrent transitions (the aggregate head lock serializes).
//! Like the other PostgreSQL suites, these skip without `DATABASE_URL`.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use reasonbraid_core::{CommandEnvelope, RequestId, PROTOCOL_VERSION};
use reasonbraid_server::{api_router, ca::ensure_server_ca, node_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static INVITE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    INVITE_LOCK
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
    for table in [
        "outbox_delivery",
        "outbox",
        "node_events",
        "node_inbox",
        "node_leases",
        "budget_reservations",
        "budget_ceilings",
        "spend_breakers",
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
        "profile_versions",
        "agent_profiles",
        "recruitment_panels",
        "recruitment_responses",
        "recruitment_offers",
        "recruitment_calls",
        "agent_roles",
        "human_principals",
        "resource_references",
        "quota_events",
        "usage_quotas",
        "federation_agreements",
        "cross_domain_receipts",
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

/// A full bootstrap: the admin human, an invited role, and an open thread.
/// Returns (tenant, human, role, thread).
async fn bootstrap(
    client: &reqwest::Client,
    base: &str,
    role_name: &str,
) -> (String, String, String, String) {
    let (status, human) = enroll(
        client,
        base,
        json!({ "kind": "human", "name": "organizer" }),
    )
    .await;
    assert_eq!(status, 200, "bootstrap human: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let (status, role) = enroll(
        client,
        base,
        json!({ "kind": "role", "name": role_name, "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "bootstrap role: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let (status, created) = command(
        client,
        base,
        "/v1/threads",
        &human_id,
        &envelope(
            "thread.create",
            &format!("key-create-{role_name}"),
            json!({
                "tenant_id": tenant,
                "subject": "invitation lifecycle",
                "objective": "prove the explicit-participants contract",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "thread creates: {created}");
    let thread = created["thread_id"].as_str().unwrap().to_string();
    (tenant, human_id, role_id, thread)
}

/// The invite's argument bundle (the `.1.1.3` CreateProfileArgs pattern — a
/// struct instead of a seven-plus argument list).
struct InviteSpec<'a> {
    thread: &'a str,
    human: &'a str,
    tenant: &'a str,
    role: &'a str,
    key: &'a str,
    ttl: Option<i64>,
}

async fn invite(client: &reqwest::Client, base: &str, spec: &InviteSpec<'_>) -> (u16, Value) {
    let mut body = json!({ "tenant_id": spec.tenant, "agent_role": spec.role });
    if let Some(ttl) = spec.ttl {
        body["expires_in_seconds"] = json!(ttl);
    }
    command(
        client,
        base,
        &format!("/v1/threads/{}/commands", spec.thread),
        spec.human,
        &envelope("thread.invite", spec.key, body),
    )
    .await
}

async fn accept(
    client: &reqwest::Client,
    base: &str,
    thread: &str,
    role: &str,
    tenant: &str,
    key: &str,
) -> (u16, Value) {
    command(
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
    .await
}

async fn thread_state(
    client: &reqwest::Client,
    base: &str,
    thread: &str,
    tenant: &str,
    human: &str,
) -> Value {
    let response = client
        .get(format!("{base}/v1/threads/{thread}?tenant_id={tenant}"))
        .header(PRINCIPAL_HEADER, human)
        .send()
        .await
        .expect("state request");
    assert_eq!(response.status().as_u16(), 200);
    response.json().await.expect("state json")
}

/// THE `.1.3.1` lifecycle: invite → PENDING (the role may NOT act); accept →
/// `accepted` + the event; the admin removal revokes. Typed refusals along the
/// way.
#[tokio::test]
async fn invitation_lifecycle_accept_acts_and_remove_revokes() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, human, role, thread) = bootstrap(&client, &base, "reviewer").await;

    // Invite: a PENDING offer with its recorded facts.
    let (status, invited) = invite(
        &client,
        &base,
        &InviteSpec {
            thread: &thread,
            human: &human,
            tenant: &tenant,
            role: &role,
            key: "k-invite",
            ttl: None,
        },
    )
    .await;
    assert_eq!(status, 200, "invite: {invited}");
    let state = thread_state(&client, &base, &thread, &tenant, &human).await;
    assert_eq!(state["state"]["participants"][&role], json!("invited"));
    assert!(state["state"]["invitations"][&role]["invited_at"].is_string());
    assert_eq!(
        state["state"]["invitations"][&role]["expires_at"],
        json!(null)
    );

    // An invited role may NOT act: typed `invalid_transition` naming the accept.
    let (status, premature) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role,
        &envelope(
            "thread.contribute",
            "k-premature",
            json!({ "tenant_id": tenant, "content": "jumping the gun" }),
        ),
    )
    .await;
    assert_eq!(status, 409, "premature contribution: {premature}");
    assert_eq!(premature["code"], json!("invalid_transition"));
    assert!(
        premature["message"]
            .as_str()
            .unwrap()
            .contains("has not accepted"),
        "got: {premature}"
    );

    // The explicit accept: the event + the state.
    let (status, accepted) = accept(&client, &base, &thread, &role, &tenant, "k-accept").await;
    assert_eq!(status, 200, "accept: {accepted}");
    assert_eq!(accepted["event_type"], json!("thread.invitation_accepted"));
    let state = thread_state(&client, &base, &thread, &tenant, &human).await;
    assert_eq!(state["state"]["participants"][&role], json!("accepted"));

    // Accepted roles act.
    let (status, contributed) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role,
        &envelope(
            "thread.contribute",
            "k-contribute",
            json!({ "tenant_id": tenant, "content": "now I may speak" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "contribute after accept: {contributed}");

    // Removal: a tenant_admin revokes; the revoked role may no longer act; a
    // non-admin cannot remove; removing a stranger is a typed refusal.
    let (status, removed) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.remove_participant",
            "k-remove",
            json!({ "tenant_id": tenant, "participant": role }),
        ),
    )
    .await;
    assert_eq!(status, 200, "remove: {removed}");
    assert_eq!(removed["event_type"], json!("thread.participant_removed"));
    let state = thread_state(&client, &base, &thread, &tenant, &human).await;
    assert_eq!(state["state"]["participants"][&role], json!("revoked"));

    let (status, after_revoke) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role,
        &envelope(
            "thread.contribute",
            "k-after-revoke",
            json!({ "tenant_id": tenant, "content": "revoked voices are silent" }),
        ),
    )
    .await;
    assert_eq!(status, 403, "revoked role refused: {after_revoke}");

    let (status, stranger) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.remove_participant",
            "k-remove-stranger",
            json!({
                "tenant_id": tenant,
                "participant": "rol_00000000-0000-7000-8000-000000000099",
            }),
        ),
    )
    .await;
    assert_eq!(status, 400, "removing a stranger: {stranger}");

    let (status, non_admin) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role,
        &envelope(
            "thread.remove_participant",
            "k-remove-by-role",
            json!({ "tenant_id": tenant, "participant": human }),
        ),
    )
    .await;
    assert_eq!(status, 403, "a role cannot remove: {non_admin}");
}

/// Decline, derived expiry, and re-invitation: a declined role may be re-invited;
/// a pending role may not be double-invited; an expired invitation refuses both
/// accept and decline (and reads as `expired` in the derived view).
#[tokio::test]
async fn decline_expiry_and_reinvitation() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, human, role, thread) = bootstrap(&client, &base, "decliner").await;

    // A pending role cannot be double-invited.
    let (status, _) = invite(
        &client,
        &base,
        &InviteSpec {
            thread: &thread,
            human: &human,
            tenant: &tenant,
            role: &role,
            key: "k-invite-1",
            ttl: None,
        },
    )
    .await;
    assert_eq!(status, 200);
    let (status, double) = invite(
        &client,
        &base,
        &InviteSpec {
            thread: &thread,
            human: &human,
            tenant: &tenant,
            role: &role,
            key: "k-invite-2",
            ttl: None,
        },
    )
    .await;
    assert_eq!(status, 400, "double invite: {double}");

    // Decline → typed event + state; accept after decline refuses.
    let (status, declined) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role,
        &envelope(
            "thread.decline_invitation",
            "k-decline",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "decline: {declined}");
    assert_eq!(declined["event_type"], json!("thread.invitation_declined"));
    let state = thread_state(&client, &base, &thread, &tenant, &human).await;
    assert_eq!(state["state"]["participants"][&role], json!("declined"));

    let (status, late_accept) =
        accept(&client, &base, &thread, &role, &tenant, "k-late-accept").await;
    assert_eq!(status, 400, "accept after decline: {late_accept}");

    // Re-invitation after a terminal state: allowed — a fresh pending offer.
    let (status, _) = invite(
        &client,
        &base,
        &InviteSpec {
            thread: &thread,
            human: &human,
            tenant: &tenant,
            role: &role,
            key: "k-reinvite",
            ttl: None,
        },
    )
    .await;
    assert_eq!(status, 200, "re-invite after decline");
    let (status, _) = accept(&client, &base, &thread, &role, &tenant, "k-re-accept").await;
    assert_eq!(status, 200, "the fresh offer accepts");

    // Expiry: an already-expired invitation refuses accept AND decline, and the
    // DERIVED inspection view reads `expired` (the stored offer keeps its facts).
    let (status, lapsed_role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "lapsed", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enroll lapsed role: {lapsed_role}");
    let expired_role = lapsed_role["principal_id"].as_str().unwrap().to_string();
    let (status, _) = invite(
        &client,
        &base,
        &InviteSpec {
            thread: &thread,
            human: &human,
            tenant: &tenant,
            role: &expired_role,
            key: "k-expiring",
            ttl: Some(-1),
        },
    )
    .await;
    assert_eq!(status, 200);
    let state = thread_state(&client, &base, &thread, &tenant, &human).await;
    assert_eq!(
        state["state"]["participants"][&expired_role],
        json!("expired"),
        "the derived view reads an expired offer as expired"
    );
    let (status, expired_accept) = accept(
        &client,
        &base,
        &thread,
        &expired_role,
        &tenant,
        "k-expired-accept",
    )
    .await;
    assert_eq!(status, 409, "accept after expiry: {expired_accept}");
    assert!(expired_accept["message"]
        .as_str()
        .unwrap()
        .contains("expired"));
    let (status, expired_decline) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &expired_role,
        &envelope(
            "thread.decline_invitation",
            "k-expired-decline",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 409, "decline after expiry: {expired_decline}");
}

/// THE backlog-16 race test: concurrent accept and decline (distinct
/// idempotency keys) both consume the SAME pending invitation — exactly ONE
/// wins; the other is a typed refusal; the timeline holds exactly one
/// transition event. (Accept vs REMOVE is deliberately NOT the race: the
/// suite's first shape proved both succeed — the serialized order
/// accept-then-revoke is a legitimate sequence, and the final snapshot is
/// `revoked` either way. The aggregate head lock serializes; the DOMAIN decides
/// which transitions conflict.) The aggregate head lock is the serialization
/// point.
#[tokio::test]
async fn concurrent_accept_and_decline_have_exactly_one_winner() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, human, role, thread) = bootstrap(&client, &base, "racer").await;

    let (status, _) = invite(
        &client,
        &base,
        &InviteSpec {
            thread: &thread,
            human: &human,
            tenant: &tenant,
            role: &role,
            key: "k-race-invite",
            ttl: None,
        },
    )
    .await;
    assert_eq!(status, 200);

    // Fire accept and decline (both as the invited role) concurrently.
    let accept_fut = async {
        let (status, body) = accept(&client, &base, &thread, &role, &tenant, "k-race-accept").await;
        (status, body)
    };
    let decline_fut = async {
        let (status, body) = command(
            &client,
            &base,
            &format!("/v1/threads/{thread}/commands"),
            &role,
            &envelope(
                "thread.decline_invitation",
                "k-race-decline",
                json!({ "tenant_id": tenant }),
            ),
        )
        .await;
        (status, body)
    };
    let ((accept_status, accept_body), (decline_status, decline_body)) =
        tokio::join!(accept_fut, decline_fut);

    // Exactly one winner.
    let accept_ok = accept_status == 200;
    let decline_ok = decline_status == 200;
    assert_ne!(
        accept_ok, decline_ok,
        "exactly one transition wins (accept={accept_status}: {accept_body}; \
         decline={decline_status}: {decline_body})"
    );

    // The timeline holds exactly one of the two transition events.
    let response = client
        .get(format!(
            "{base}/v1/threads/{thread}/events?tenant_id={tenant}"
        ))
        .header(PRINCIPAL_HEADER, &human)
        .send()
        .await
        .expect("events request");
    let events: Value = response.json().await.unwrap();
    let transitions: Vec<&str> = events["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["event_type"].as_str().unwrap())
        .filter(|t| *t == "thread.invitation_accepted" || *t == "thread.invitation_declined")
        .collect();
    assert_eq!(
        transitions.len(),
        1,
        "exactly one transition event exists: {transitions:?}"
    );

    // The final snapshot matches the winner.
    let state = thread_state(&client, &base, &thread, &tenant, &human).await;
    let expected = if accept_ok { "accepted" } else { "declined" };
    assert_eq!(
        state["state"]["participants"][&role],
        json!(expected),
        "the snapshot matches the winning transition"
    );

    // The acceptance capability is spent either way: a late accept refuses.
    let (status, late) = accept(&client, &base, &thread, &role, &tenant, "k-race-late").await;
    assert_eq!(status, 400, "the spent invitation refuses: {late}");
}

/// THE `.1.3.2` subscriptions contract: `thread.join` admits a role only when
/// the thread's join door is open; `allow_explicit_invites=false` refuses the
/// invite verb; the joined role acts immediately (no invitation needed); the
/// self-request path is recorded in the event.
#[tokio::test]
async fn join_requests_and_invite_enforcement() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, human, role, thread) = bootstrap(&client, &base, "joiner").await;

    // The default thread's join door is CLOSED (`.1.1.3` stated default).
    let (status, refused) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role,
        &envelope(
            "thread.join",
            "k-join-closed",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 400, "join on a closed-join thread: {refused}");
    assert!(refused["message"]
        .as_str()
        .unwrap()
        .contains("does not allow join requests"));

    // A thread with the join door open admits the role through the self-request
    // path — no invitation.
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &human,
        &envelope(
            "thread.create",
            "key-create-join",
            json!({
                "tenant_id": tenant,
                "subject": "open door",
                "objective": "prove the join path",
                "participant_rules": { "allow_join_requests": true },
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "open-join thread creates: {created}");
    let open_thread = created["thread_id"].as_str().unwrap().to_string();

    let (status, joined) = command(
        &client,
        &base,
        &format!("/v1/threads/{open_thread}/commands"),
        &role,
        &envelope("thread.join", "k-join-open", json!({ "tenant_id": tenant })),
    )
    .await;
    assert_eq!(status, 200, "join: {joined}");
    assert_eq!(joined["event_type"], json!("thread.participant_joined"));
    let state = thread_state(&client, &base, &open_thread, &tenant, &human).await;
    assert_eq!(state["state"]["participants"][&role], json!("accepted"));

    // The joined role acts immediately.
    let (status, contributed) = command(
        &client,
        &base,
        &format!("/v1/threads/{open_thread}/commands"),
        &role,
        &envelope(
            "thread.contribute",
            "k-join-contribute",
            json!({ "tenant_id": tenant, "content": "joined voices speak" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the joined role contributes: {contributed}");

    // A second join is refused (an open membership exists).
    let (status, again) = command(
        &client,
        &base,
        &format!("/v1/threads/{open_thread}/commands"),
        &role,
        &envelope(
            "thread.join",
            "k-join-again",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 400, "double join: {again}");

    // `allow_explicit_invites=false` is ENFORCED: the invite verb refuses.
    let (status, closed_invites) = command(
        &client,
        &base,
        "/v1/threads",
        &human,
        &envelope(
            "thread.create",
            "key-create-no-invites",
            json!({
                "tenant_id": tenant,
                "subject": "join only",
                "objective": "no explicit invites",
                "participant_rules": {
                    "allow_explicit_invites": false,
                    "allow_join_requests": true,
                },
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "join-only thread creates: {closed_invites}");
    let join_only = closed_invites["thread_id"].as_str().unwrap().to_string();

    let (status, denied_invite) = invite(
        &client,
        &base,
        &InviteSpec {
            thread: &join_only,
            human: &human,
            tenant: &tenant,
            role: &role,
            key: "k-invite-closed",
            ttl: None,
        },
    )
    .await;
    assert_eq!(status, 400, "invite on a join-only thread: {denied_invite}");
    assert!(denied_invite["message"]
        .as_str()
        .unwrap()
        .contains("does not allow explicit invitations"));

    // The same thread still admits joins.
    let (status, joined2) = command(
        &client,
        &base,
        &format!("/v1/threads/{join_only}/commands"),
        &role,
        &envelope(
            "thread.join",
            "k-join-join-only",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the join-only thread admits joins: {joined2}");
    let _ = &pool;
}
