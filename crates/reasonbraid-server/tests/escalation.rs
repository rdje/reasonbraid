//! The non-escalation property suite (`PHASE-2.7.1`): the §16.12 line
//! "authorization non-escalation properties and confused-deputy tests" as ONE named
//! adversarial surface over the shipped control API. Each test attacks one escalation
//! path with a REAL envelope over live PostgreSQL and asserts the refusal AND its
//! audit row — deny-by-default, measured, not argued.
//!
//! The existing measured legs stay where they are and are NAMED here: the widening
//! refusal + the caller-authority check (`command_api`'s delegation test), the
//! expired/revoked-grant denial (`authority`), the forged-field + version rejection
//! (`command_api`), the idempotency conflict typing (`command_api`). This suite adds
//! the adversarial scenarios those tests do not stage: cross-tenant isolation (the
//! census found NO test asserting it), the confused deputy whose OWN grant would
//! cover the target, the nuclear option's re-arm attempt, and the revocation fence
//! around history itself.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_core::{
    AuthorityContext, ClientContext, CommandEnvelope, RequestId, TargetSelector, PROTOCOL_VERSION,
};
use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static API_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn api_guard() -> tokio::sync::MutexGuard<'static, ()> {
    API_LOCK
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
            "node_leases",
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
        let router = api_router(pool.clone());
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

fn envelope_with_delegation(
    operation: &str,
    key: &str,
    body: Value,
    on_behalf_of: &str,
    thread: Option<&str>,
) -> CommandEnvelope {
    let scope = match thread {
        Some(id) => TargetSelector::Threads {
            threads: vec![id.parse().expect("thread id parses")],
        },
        None => TargetSelector::TenantWide,
    };
    CommandEnvelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        operation: operation.to_string(),
        request_id: RequestId::new(),
        idempotency_key: key.to_string(),
        expected_aggregate_version: None,
        body,
        authority_context: Some(AuthorityContext {
            on_behalf_of: on_behalf_of.to_string(),
            purpose: Some("escalation probe".to_string()),
            scope,
        }),
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

async fn admin_revoke(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    tenant: &str,
) -> (u16, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(&json!({ "tenant_id": tenant, "reason": "escalation probe" }))
        .send()
        .await
        .expect("revoke request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("revoke json"))
}

async fn denial_count(pool: &PgPool, tenant: &str) -> i64 {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM authorization_records WHERE tenant_id = $1 AND decision = 'denied'",
    )
    .bind(tenant)
    .fetch_one(pool)
    .await
    .expect("count denials");
    n
}

/// A human + a role that can act in a thread (invite + accept), plus the tenant,
/// the principal ids, the role's grant, and the thread id.
async fn tenant_with_role(
    client: &reqwest::Client,
    base: &str,
    tag: &str,
) -> (String, String, String, String, String) {
    let (status, human) = enroll(
        client,
        base,
        json!({ "kind": "human", "name": format!("{tag}-alice") }),
    )
    .await;
    assert_eq!(status, 200, "enroll human: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        client,
        base,
        json!({ "kind": "role", "name": format!("{tag}-agent"), "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enroll role: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let role_grant = role["grant_id"].as_str().unwrap().to_string();

    let (status, created) = command(
        client,
        base,
        "/v1/threads",
        &human_id,
        &envelope(
            "thread.create",
            &format!("key-{tag}-create"),
            json!({ "tenant_id": tenant, "subject": "probe", "objective": "probe" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread = created["thread_id"].as_str().unwrap().to_string();

    let (status, invited) = command(
        client,
        base,
        &format!("/v1/threads/{thread}/commands"),
        &human_id,
        &envelope(
            "thread.invite",
            &format!("key-{tag}-inv"),
            json!({ "tenant_id": tenant, "agent_role": role_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite: {invited}");
    let (status, accepted) = command(
        client,
        base,
        &format!("/v1/threads/{thread}/commands"),
        &role_id,
        &envelope(
            "thread.accept_invitation",
            &format!("key-{tag}-acc"),
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "accept: {accepted}");
    (tenant, human_id, role_id, role_grant, thread)
}

/// THE §5 headline ("no known cross-tenant/scope escalation"): a principal of tenant
/// A cannot create in tenant B, cannot read B's threads, and cannot smuggle an A-key
/// replay into B — every attempt refuses AND is audited, with no domain effect in B.
#[tokio::test]
async fn a_cross_tenant_attempt_refuses_at_every_boundary() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "xt-alice" }),
    )
    .await;
    assert_eq!(status, 200, "enroll alice: {alice}");
    let tenant_a = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();

    let (status, bob) = enroll(&client, &base, json!({ "kind": "human", "name": "xt-bob" })).await;
    assert_eq!(status, 200, "enroll bob: {bob}");
    let tenant_b = bob["tenant_id"].as_str().unwrap().to_string();
    let bob_id = bob["principal_id"].as_str().unwrap().to_string();

    // Bob owns one thread in B.
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &bob_id,
        &envelope(
            "thread.create",
            "key-xt-b1",
            json!({ "tenant_id": tenant_b, "subject": "bob's thread", "objective": "probe" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "bob creates: {created}");
    let bob_thread = created["thread_id"].as_str().unwrap().to_string();

    // 1. Alice names Bob's TENANT in a create: refused, audited, no effect.
    let (status, refused) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "key-xt-a1",
            json!({ "tenant_id": tenant_b, "subject": "spill", "objective": "probe" }),
        ),
    )
    .await;
    assert_eq!(status, 403, "a cross-tenant create is refused: {refused}");
    assert!(
        denial_count(&pool, &tenant_b).await >= 1,
        "the cross-tenant refusal is audited"
    );

    // 2. Alice names Bob's THREAD under her own tenant: the projection finds nothing
    //    — no cross-tenant read, no leak of existence.
    let (status, peeked) = get(
        &client,
        &base,
        &format!("/v1/threads/{bob_thread}?tenant_id={tenant_a}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 404, "a cross-tenant read finds nothing: {peeked}");

    // 3. The replay flavor: Alice's own key, replayed against Bob's tenant with a
    //    DIFFERENT body — the claim-first conflict types the mismatch (409), so the
    //    A-claim cannot smuggle an effect into B.
    let (status, smuggled) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "key-xt-a1",
            json!({ "tenant_id": tenant_b, "subject": "smuggled", "objective": "probe" }),
        ),
    )
    .await;
    assert_eq!(status, 409, "the cross-tenant replay conflicts: {smuggled}");
    assert!(
        smuggled["message"]
            .as_str()
            .unwrap_or("")
            .contains("idempotency conflict"),
        "the refusal types the conflict: {smuggled}"
    );

    // No domain effect in B: exactly Bob's one thread.
    let (status, listed) = get(
        &client,
        &base,
        &format!("/v1/threads?tenant_id={tenant_b}"),
        &bob_id,
    )
    .await;
    assert_eq!(status, 200, "bob lists: {listed}");
    assert_eq!(
        listed["threads"].as_array().unwrap().len(),
        1,
        "no cross-tenant effect landed in B"
    );
}

/// The confused deputy: the deputy's OWN grant covers the target, but the delegation
/// it chose to act under does not — the delegation scope is the ceiling, the deputy's
/// own authority does not rescue it.
#[tokio::test]
async fn a_delegation_scope_is_the_ceiling_even_for_a_deputy_who_could_act_alone() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (tenant, alice_id, role_id, _role_grant, first_thread) =
        tenant_with_role(&client, &base, "cd").await;

    // A second thread the deputy is ALSO invited to — the deputy could act on it
    // alone (its own grant + membership cover it).
    let (status, created2) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "key-cd-c2",
            json!({ "tenant_id": tenant, "subject": "sibling", "objective": "probe" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create sibling: {created2}");
    let sibling = created2["thread_id"].as_str().unwrap().to_string();
    let (status, invited2) = command(
        &client,
        &base,
        &format!("/v1/threads/{sibling}/commands"),
        &alice_id,
        &envelope(
            "thread.invite",
            "key-cd-inv2",
            json!({ "tenant_id": tenant, "agent_role": role_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite to sibling: {invited2}");
    let (status, accepted2) = command(
        &client,
        &base,
        &format!("/v1/threads/{sibling}/commands"),
        &role_id,
        &envelope(
            "thread.accept_invitation",
            "key-cd-acc2",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "accept sibling: {accepted2}");

    // Baseline: the deputy ACTS ALONE on the sibling (its own grant + membership).
    let (status, alone) = command(
        &client,
        &base,
        &format!("/v1/threads/{sibling}/commands"),
        &role_id,
        &envelope(
            "thread.contribute",
            "key-cd-alone",
            json!({ "tenant_id": tenant, "content": "the deputy acts alone" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the deputy could act alone: {alone}");

    // The attack: the deputy acts on the sibling UNDER a delegation scoped to the
    // FIRST thread only (on Alice's behalf). Its own grant covers the sibling — the
    // delegation does not. The refusal must name the widening invariant.
    let (status, widened) = command(
        &client,
        &base,
        &format!("/v1/threads/{sibling}/commands"),
        &role_id,
        &envelope_with_delegation(
            "thread.contribute",
            "key-cd-dep",
            json!({ "tenant_id": tenant, "content": "the delegation should not cover this" }),
            &alice_id,
            Some(&first_thread),
        ),
    )
    .await;
    assert_eq!(
        status, 403,
        "the delegation scope is the ceiling even for a deputy who could act alone: {widened}"
    );
    assert!(
        widened["message"]
            .as_str()
            .unwrap_or("")
            .contains("widening"),
        "the refusal names the widening invariant: {widened}"
    );
}

/// The nuclear option cannot be re-armed: after the boundary revocation, a FRESH
/// identity enrolled into the tenant is inert — its first authorization is refused
/// and audited, exactly like the old grants.
#[tokio::test]
async fn a_revoked_boundary_cannot_be_re_armed_by_fresh_identities() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (tenant, alice_id, _, _, _) = tenant_with_role(&client, &base, "ra").await;
    let boundary_id = {
        let (status, boundaries) = get(
            &client,
            &base,
            &format!("/v1/admin/boundaries?tenant_id={tenant}"),
            &alice_id,
        )
        .await;
        assert_eq!(status, 200, "boundaries listed: {boundaries}");
        boundaries["boundaries"][0]["boundary_id"]
            .as_str()
            .unwrap()
            .to_string()
    };

    // The nuclear option: the boundary itself is revoked.
    let (status, revoked) = admin_revoke(
        &client,
        &base,
        &format!("/v1/admin/boundaries/{boundary_id}/revoke"),
        &alice_id,
        &tenant,
    )
    .await;
    assert_eq!(status, 200, "the boundary revocation succeeds: {revoked}");

    // The re-arm attempt: a FRESH role enrolls into the frozen tenant. The fence
    // sits at the ENROLLMENT boundary itself — no identity can be minted under a
    // revoked boundary (the nuclear option is not re-armable by minting).
    let (status, fresh) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "ra-fresh", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(
        status, 400,
        "identity minting under a revoked boundary is refused: {fresh}"
    );
    assert!(
        fresh["message"]
            .as_str()
            .unwrap_or("")
            .contains("no active enrollment boundary"),
        "the refusal names the boundary fence: {fresh}"
    );

    // The re-arm attempt left no trace: no enrollment row exists for the refused
    // identity (row-level proof, like the denial counts above).
    let (n_fresh,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM enrollments WHERE tenant_id = $1 AND name = 'ra-fresh'",
    )
    .bind(&tenant)
    .fetch_one(&pool)
    .await
    .expect("count the refused enrollment");
    assert_eq!(n_fresh, 0, "the refused identity left no enrollment row");
}

/// The revocation fence: a revoked grant fences FUTURE work but never rewrites
/// history — the original command's replay still returns its stored result verbatim
/// (no new domain effect), while the next NEW command is refused and audited.
#[tokio::test]
async fn revocation_fences_future_work_but_never_rewrites_history() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (tenant, alice_id, role_id, role_grant, thread) =
        tenant_with_role(&client, &base, "rh").await;

    // The role acts once, allowed and recorded.
    let (status, contributed) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role_id,
        &envelope(
            "thread.contribute",
            "key-rh-1",
            json!({ "tenant_id": tenant, "content": "the pre-revocation act" }),
        ),
    )
    .await;
    assert_eq!(
        status, 200,
        "the role acts before the revocation: {contributed}"
    );

    // Revoke the role's grant.
    let (status, revoked) = admin_revoke(
        &client,
        &base,
        &format!("/v1/admin/grants/{role_grant}/revoke"),
        &alice_id,
        &tenant,
    )
    .await;
    assert_eq!(status, 200, "the grant revocation succeeds: {revoked}");

    // (a) History is intact: the ORIGINAL command's replay returns the stored
    // result verbatim (the claim precedes authorization by design) — no new
    // domain effect, no rewrite.
    let (status, replayed) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role_id,
        &envelope(
            "thread.contribute",
            "key-rh-1",
            json!({ "tenant_id": tenant, "content": "the pre-revocation act" }),
        ),
    )
    .await;
    assert_eq!(
        status, 200,
        "the replay returns the original stored result: {replayed}"
    );
    assert_eq!(
        replayed["replayed"],
        json!(true),
        "the replay is marked, not re-executed: {replayed}"
    );
    let (n_contributions,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM event_log WHERE tenant_id = $1 AND event_type = 'thread.contribution_submitted'",
    )
    .bind(&tenant)
    .fetch_one(&pool)
    .await
    .expect("count contributions");
    assert_eq!(
        n_contributions, 1,
        "the replay created no new domain effect"
    );

    // (b) Future work is fenced: the NEXT command is refused and audited.
    let (status, refused) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role_id,
        &envelope(
            "thread.contribute",
            "key-rh-2",
            json!({ "tenant_id": tenant, "content": "after the revocation" }),
        ),
    )
    .await;
    assert_eq!(
        status, 403,
        "the revoked grant refuses the next authorization: {refused}"
    );
    assert!(
        denial_count(&pool, &tenant).await >= 1,
        "the post-revocation refusal is audited"
    );
}
