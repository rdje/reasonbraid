//! The MCP write-half's qualified gate proof (`PHASE-8.3.5.1`, ADR-024
//! §9.6, the `.1.3.2` `principal`-scope re-open): the write seam adds ONLY
//! the enrollment binding + the per-principal quota; the effects ride the
//! SAME handlers (the thread-command pipeline / the call-respond core / the
//! lifecycle registration). Measured:
//!   - the gate refuses the unenrolled + the foreign-tenant principal;
//!   - the gate counts the admitted calls (the use rows) and refuses the
//!     unconfigured fail-closed + the exhausted with the RECORDED denial;
//!   - the granted `respond` lands (the effect + the audited allowance +
//!     the quota use); the ungranted role is refused by the handler's OWN
//!     authz (the typed `handler:unauthorized`, never the ambient path);
//!   - the `join_call` decline + the policy proposal ride the same handlers.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

#[path = "support/site.rs"]
mod site_fixture;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_core::{ClientContext, CommandEnvelope, GrantSubject, RequestId, PROTOCOL_VERSION};
use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static WRITE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    WRITE_LOCK
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
    pg_cleanup::delete_tables(
        &pool,
        &[
            // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act,
            // so this suite now writes the site trail. Ahead of the policy tables
            // because the audit row outlives the act it records.
            "site_audit",
            "policy_reviews",
            "policy_outcomes",
            "policy_corrections",
            "policy_drift",
            "deployment_assignments",
            "deployment_targets",
            "policy_publications",
            "policy_projections",
            "policy_approvals",
            "policy_decisions",
            "policy_proposals",
            "policy_versions",
            "routing_recommendations",
            "routing_resolutions",
            "evaluation_runs",
            "evaluation_corpora",
            "profile_versions",
            "agent_profiles",
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
    envelope: &CommandEnvelope,
) -> (u16, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(envelope)
        .send()
        .await
        .expect("command request");
    let status = response.status().as_u16();
    let text = response.text().await.expect("command body");
    let body = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
    (status, body)
}

async fn post(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    body: &Value,
) -> (u16, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(body)
        .send()
        .await
        .expect("post request");
    let status = response.status().as_u16();
    let text = response.text().await.expect("post body");
    let body = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
    (status, body)
}

fn role_subject(role_id: &str) -> GrantSubject {
    role_id
        .parse::<reasonbraid_core::AgentRoleId>()
        .map(GrantSubject::Role)
        .expect("the role id parses")
}

/// The bootstrap pair used by every leg: the tenant + the role.
async fn bootstrap(
    client: &reqwest::Client,
    base: &str,
    role_name: &str,
    actions: Value,
) -> (String, String, String) {
    let (status, human) = enroll(
        client,
        base,
        json!({ "kind": "human", "name": "mcpw-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        client,
        base,
        json!({ "kind": "role", "name": role_name, "tenant_id": tenant, "actions": actions }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    (human_id, tenant, role_id)
}

async fn principal_uses(pool: &PgPool, tenant: &str, role_id: &str, kind: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM quota_events q \
         JOIN usage_quotas u ON u.quota_id = q.quota_id \
         WHERE u.tenant_id = $1 AND u.scope_kind = 'principal' AND u.scope_id = $2 \
         AND q.kind = $3",
    )
    .bind(tenant)
    .bind(role_id)
    .bind(kind)
    .fetch_one(pool)
    .await
    .expect("the use count")
}

/// 1. The enrollment binding: the unenrolled + the foreign tenant refuse;
///    the enrolled principal (the enroll created its quota row) passes.
#[tokio::test]
async fn the_gate_binds_the_enrollment() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let (_, tenant, role_id) = bootstrap(&client, &base, "mcpw-bound", json!([])).await;

    // The unenrolled: a well-formed id with no identity row.
    let ghost = GrantSubject::Human(
        "hpr_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee"
            .parse()
            .expect("the ghost id parses"),
    );
    let err = reasonbraid_server::mcp_write_internal::gate(&pool, &tenant, &ghost)
        .await
        .expect_err("the unenrolled refuses");
    assert_eq!(err.family, "enrollment", "the unenrolled: {err:?}");

    // The foreign tenant: the role's recorded tenant vs a second tenant.
    let (status, bob) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "mcpw-bob" }),
    )
    .await;
    assert_eq!(status, 200, "bob: {bob}");
    let other_tenant = bob["tenant_id"].as_str().unwrap().to_string();
    let subject = role_subject(&role_id);
    let err = reasonbraid_server::mcp_write_internal::gate(&pool, &other_tenant, &subject)
        .await
        .expect_err("the foreign tenant refuses");
    assert_eq!(err.family, "enrollment", "the foreign tenant: {err:?}");

    // The enrolled + the matching tenant passes.
    reasonbraid_server::mcp_write_internal::gate(&pool, &tenant, &subject)
        .await
        .expect("the enrolled principal passes");
}

/// 2. The quota: the admitted calls count (the use rows); the unconfigured
///    refuses fail-closed; the exhausted commits the RECORDED denial.
#[tokio::test]
async fn the_gate_counts_calls_and_refuses_the_unconfigured_and_the_exhausted() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let (_, tenant, role_id) = bootstrap(&client, &base, "mcpw-quota", json!([])).await;
    let subject = role_subject(&role_id);

    assert_eq!(
        principal_uses(&pool, &tenant, &role_id, "use").await,
        0,
        "the fresh role starts at zero uses"
    );

    reasonbraid_server::mcp_write_internal::gate(&pool, &tenant, &subject)
        .await
        .expect("the first call admits");
    reasonbraid_server::mcp_write_internal::gate(&pool, &tenant, &subject)
        .await
        .expect("the second call admits");
    assert_eq!(
        principal_uses(&pool, &tenant, &role_id, "use").await,
        2,
        "the admitted calls count"
    );

    // The unconfigured: the row removed → the fail-closed refusal. (The
    // quota_events rows reference the quota row — the re-configuration
    // clears them first, exactly like a real binding change.)
    sqlx::query(
        "DELETE FROM quota_events USING usage_quotas \
         WHERE quota_events.quota_id = usage_quotas.quota_id \
         AND usage_quotas.tenant_id = $1 AND usage_quotas.scope_kind = 'principal' \
         AND usage_quotas.scope_id = $2",
    )
    .bind(&tenant)
    .bind(&role_id)
    .execute(&pool)
    .await
    .expect("clear the events");
    sqlx::query(
        "DELETE FROM usage_quotas \
         WHERE tenant_id = $1 AND scope_kind = 'principal' AND scope_id = $2",
    )
    .bind(&tenant)
    .bind(&role_id)
    .execute(&pool)
    .await
    .expect("remove the quota row");
    let err = reasonbraid_server::mcp_write_internal::gate(&pool, &tenant, &subject)
        .await
        .expect_err("the unconfigured refuses");
    assert_eq!(
        err.family, "quota_unconfigured",
        "the unconfigured: {err:?}"
    );

    // The exhausted: a zero ceiling → the first call denies + the denial
    // row COMMITS (a refusal is a recorded fact, never silent).
    sqlx::query(
        "INSERT INTO usage_quotas (quota_id, tenant_id, scope_kind, scope_id, ceiling, window_seconds) \
         VALUES ($1, $2, 'principal', $3, 0, 3600)",
    )
    .bind(format!("quo_{role_id}_writes"))
    .bind(&tenant)
    .bind(&role_id)
    .execute(&pool)
    .await
    .expect("the zero ceiling");
    let err = reasonbraid_server::mcp_write_internal::gate(&pool, &tenant, &subject)
        .await
        .expect_err("the exhausted refuses");
    assert_eq!(err.family, "quota_exceeded", "the exhausted: {err:?}");
    assert_eq!(
        principal_uses(&pool, &tenant, &role_id, "denial").await,
        1,
        "the denial row persists"
    );
}

/// 3. The granted `respond` rides the SAME thread-command pipeline; the
///    ungranted role is refused by the handler's OWN authz — never the
///    ambient path.
///
/// ⚠️ **The heading used to say "the effect + the audited allowance + the quota
/// use", naming three things as one pipeline, and it is true of two**
/// (`SIGNOFF-REPAIR.6.1.4`). The effect and the allowance commit together inside
/// `run_thread_command`; the QUOTA use commits in the gate's own transaction
/// BEFORE the handler is reached, and is spent whatever the handler then decides.
/// The split is deliberate and is asserted on its own by
/// `the_quota_counts_the_admitted_call_and_not_the_effect`. This test still
/// checks the use count, and that is fine — what it may not do is imply the
/// three share a transaction.
#[tokio::test]
async fn the_granted_respond_lands_and_the_ungranted_is_refused_by_the_handler() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let (human_id, tenant, role_id) = bootstrap(
        &client,
        &base,
        "mcpw-writer",
        json!(["thread_contribute", "thread_invitation_respond"]),
    )
    .await;

    // The thread: the human creates; the role joins via the invite/accept.
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &envelope(
            "thread.create",
            "k-create",
            json!({
                "tenant_id": tenant,
                "subject": "the mcp write gate",
                "objective": "prove the same-handler path",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let (status, invited) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &human_id,
        &envelope(
            "thread.invite",
            "k-invite",
            json!({ "tenant_id": tenant, "agent_role": role_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite: {invited}");
    let (status, accepted) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &role_id,
        &envelope(
            "thread.accept_invitation",
            "k-accept",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "accept: {accepted}");

    // The seam respond: the body WITHOUT the tenant (the seam injects it —
    // the dev-profile trust shape; the tool's payload never carries the
    // tenant twice).
    let result = reasonbraid_server::mcp_write_internal::respond(
        &pool,
        &tenant,
        &role_subject(&role_id),
        &thread_id,
        json!({ "content": "the mcp write lands", "kind": "claim" }),
    )
    .await
    .expect("the granted respond lands");
    assert_eq!(result["status"], json!(200), "the respond: {result}");

    // The effect is visible to the reader (the human's events read).
    let events = client
        .get(format!(
            "{base}/v1/threads/{thread_id}/events?tenant_id={tenant}"
        ))
        .header(PRINCIPAL_HEADER, &human_id)
        .send()
        .await
        .expect("the events read");
    assert_eq!(events.status().as_u16(), 200, "the events read");
    let events: Value = events.json().await.expect("the events json");
    let kinds: Vec<String> = events["events"]
        .as_array()
        .unwrap_or_else(|| panic!("the event list: {events}"))
        .iter()
        .map(|e| e["event_type"].as_str().unwrap_or("").to_string())
        .collect();
    assert!(
        kinds.contains(&"thread.contribution_submitted".to_string()),
        "the contribution event lands: {kinds:?}"
    );

    // The audited allowance + the quota use ride the same transaction. The
    // record carries the ACTOR handle (the subject columns are the
    // delegation-only split).
    let actor = reasonbraid_core::actor_handle_for_subject(&role_subject(&role_id)).to_string();
    let allowances: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM authorization_records \
         WHERE actor = $1 AND action = 'thread_contribute' AND decision = 'allowed'",
    )
    .bind(&actor)
    .fetch_one(&pool)
    .await
    .expect("the allowance count");
    assert!(allowances >= 1, "the allowance is audited");
    assert!(
        principal_uses(&pool, &tenant, &role_id, "use").await >= 1,
        "the admitted respond counted — in the GATE's transaction, not this one"
    );

    // The ungranted role (the explicit EMPTY action set — the default set
    // would grant `thread_contribute`): the gate passes (the enrollment +
    // the quota) but the handler's OWN authz refuses.
    let (status, ghost) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "mcpw-ghost", "tenant_id": tenant, "actions": [] }),
    )
    .await;
    assert_eq!(status, 200, "the ghost enrolls: {ghost}");
    let ghost_id = ghost["principal_id"].as_str().unwrap().to_string();
    let err = reasonbraid_server::mcp_write_internal::respond(
        &pool,
        &tenant,
        &role_subject(&ghost_id),
        &thread_id,
        json!({ "content": "the ungranted write", "kind": "claim" }),
    )
    .await
    .expect_err("the ungranted refuses");
    assert_eq!(
        err.family, "handler:unauthorized",
        "the ungranted: {err:?} — message: {}",
        err.message
    );
    let ghost_actor =
        reasonbraid_core::actor_handle_for_subject(&role_subject(&ghost_id)).to_string();
    let denials: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM authorization_records \
         WHERE actor = $1 AND action = 'thread_contribute' AND decision = 'denied'",
    )
    .bind(&ghost_actor)
    .fetch_one(&pool)
    .await
    .expect("the denial count");
    assert_eq!(denials, 1, "the handler's denial is audited");
}

/// 4. The `join_call` decline + the policy proposal ride the same handlers
///    through the gate (the non-participation kinds skip the eligibility
///    profile machinery; the proposal rides the lifecycle registration).
#[tokio::test]
async fn the_join_call_decline_and_the_proposal_ride_the_same_handlers() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let (human_id, tenant, role_id) =
        bootstrap(&client, &base, "mcpw-caller", json!(["thread_contribute"])).await;

    // The thread (the call + the proposal reference it).
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &envelope(
            "thread.create",
            "k-create",
            json!({
                "tenant_id": tenant,
                "subject": "the mcp write ride",
                "objective": "the same handlers",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    // The call opens (the human's invite authority); the role DECLINES via
    // the seam — the non-participation kind skips the eligibility gate.
    let deadline = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let expiry = (chrono::Utc::now() + chrono::Duration::hours(2)).to_rfc3339();
    let (status, opened) = post(
        &client,
        &base,
        "/v1/calls",
        &human_id,
        &json!({
            "tenant_id": tenant,
            "thread_id": thread_id,
            "expression": {
                "scope": "tenant",
                "capabilities": [],
                "presence_states": ["available"],
            },
            "min_participants": 1,
            "max_participants": 2,
            "join_deadline": deadline,
            "expires_at": expiry,
        }),
    )
    .await;
    assert_eq!(status, 200, "the call opens: {opened}");
    let call_id = opened["call_id"].as_str().unwrap().to_string();

    let result = reasonbraid_server::mcp_write_internal::join_call(
        &pool,
        &tenant,
        &role_subject(&role_id),
        &call_id,
        json!({ "kind": "decline" }),
    )
    .await
    .expect("the decline rides the same handler");
    assert_eq!(
        result["response"],
        json!("decline"),
        "the decline: {result}"
    );

    // The policy: the human registers the document; the role's proposal
    // rides the lifecycle registration through the gate.
    //
    // `SIGNOFF-REPAIR.6.1.5.4`: registering the document is a SITE act and the
    // body carries the reason every site act carries. The role's PROPOSAL below
    // is untouched by that — a proposal is its tenant's, which is the whole of
    // DOC-0071's split, and this fixture exercises both halves in one body.
    let grant_id = format!("grt_{human_id}");
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let (status, registered) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "reason": "the MCP write fixture seeds the governance library",
            "policy_id": "mcp-pol",
            "version": "1.0.0",
            "digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "lifecycle": "draft",
            "title": "the mcp write policy",
            "intent": "the proposal surface",
            "domain": "deliberation",
            "risk_class": "low",
            "owning_authority": grant_id,
            "clauses": [
                { "id": "c1", "statement": "every write rides a local grant" },
            ],
            "applicability": [ { "layer": "organization", "target": "*" } ],
            "exceptions": [],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers: {registered}");

    let result = reasonbraid_server::mcp_write_internal::propose_policy_change(
        &pool,
        &tenant,
        &role_subject(&role_id),
        json!({
            "proposal_id": "prp_mcp_1",
            "policy_id": "mcp-pol",
            "policy_version": "1.0.0",
            "thread_id": thread_id,
        }),
    )
    .await
    .expect("the proposal rides the same handler");
    assert_eq!(
        result["proposal_id"],
        json!("prp_mcp_1"),
        "the proposal: {result}"
    );
}

/// The call-binding control (`SIGNOFF-REPAIR.6.1.2`): a response must be bound
/// to the call's OWN tenant, on every surface that records one.
///
/// ⛔ The seam gates the caller against a tenant the CALLER SUPPLIES and then
/// hands the call id to a core that fetches by that id ALONE — two identifiers,
/// one checked. But the HTTP verb takes no tenant at all and rides the same
/// core, so the binding belongs in the CORE and the control asserts both
/// surfaces. A repair at the seam would leave the HTTP verb wide open and the
/// seam's own test green.
///
/// ⭐ `decline` is a separate leg because it is the one that skips the
/// eligibility gate entirely (`participation` is false), so nothing else in
/// the path even incidentally looks at the respondent.
#[tokio::test]
async fn a_response_is_bound_to_the_calls_own_tenant() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // Tenant A: the outsider role that will try to answer.
    let (_alice_human, tenant_a, outsider_role) = bootstrap(
        &client,
        &base,
        "mcpw-outsider",
        json!(["thread_contribute", "thread_invitation_respond"]),
    )
    .await;

    // Tenant B: its own human, thread and call.
    let (status, bob) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "mcpw-bob" }),
    )
    .await;
    assert_eq!(status, 200, "bob enrolls: {bob}");
    let bob_id = bob["principal_id"].as_str().unwrap().to_string();
    let tenant_b = bob["tenant_id"].as_str().unwrap().to_string();
    assert_ne!(
        tenant_a, tenant_b,
        "the two enrolments mint distinct tenants"
    );

    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &bob_id,
        &envelope(
            "thread.create",
            "mcpw-bind-create",
            json!({
                "tenant_id": tenant_b,
                "subject": "bob's call",
                "objective": "the call-binding control",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "bob creates his thread: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let deadline = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let expiry = (chrono::Utc::now() + chrono::Duration::hours(2)).to_rfc3339();
    let (status, opened) = post(
        &client,
        &base,
        "/v1/calls",
        &bob_id,
        &json!({
            "tenant_id": tenant_b,
            "thread_id": thread_id,
            "expression": {
                "scope": "tenant",
                "capabilities": [],
                "presence_states": ["available"],
            },
            "min_participants": 1,
            "max_participants": 2,
            "join_deadline": deadline,
            "expires_at": expiry,
        }),
    )
    .await;
    assert_eq!(status, 200, "bob's call opens: {opened}");
    let call_id = opened["call_id"].as_str().unwrap().to_string();

    let mut breaches: Vec<String> = Vec::new();

    // The two NON-PARTICIPATION kinds, which are the ones that matter here:
    // `participation` is false for them, so the eligibility gate is skipped
    // entirely and nothing else in the path looks at the respondent.
    //
    // ⛔ `join` is deliberately NOT a leg. Measured: it is refused earlier,
    // by `respondent_candidate` finding no enrolled node for the role — a
    // different mechanism, and one that would make this control pass without
    // the binding ever being checked. A leg that goes green for the wrong
    // reason is worse than no leg.
    let kinds = [
        json!({ "kind": "decline" }),
        json!({ "kind": "recuse", "reason_class": "conflict_of_interest" }),
    ];

    // Leg A — through the SEAM, gated on the outsider's OWN tenant, which is
    // the shape the seam actually permits: the gate passes and the call is
    // fetched by id alone.
    for body in &kinds {
        let kind = body["kind"].as_str().unwrap();
        let outcome = reasonbraid_server::mcp_write_internal::join_call(
            &pool,
            &tenant_a,
            &role_subject(&outsider_role),
            &call_id,
            body.clone(),
        )
        .await;
        if let Ok(value) = outcome {
            breaches.push(format!(
                "A/{kind}: a role in tenant A answered tenant B's call through the seam: {value}"
            ));
        }
    }

    // Leg B — through the HTTP verb, which takes NO tenant at all and rides
    // the same core. A repair placed at the seam would leave this open.
    for body in &kinds {
        let kind = body["kind"].as_str().unwrap();
        let (status, response) = post(
            &client,
            &base,
            &format!("/v1/calls/{call_id}/respond"),
            &outsider_role,
            body,
        )
        .await;
        if status == 200 {
            breaches.push(format!(
                "B/{kind}: a role in tenant A answered tenant B's call over HTTP: {response}"
            ));
        }
    }

    // Leg C — no response row survived either attempt.
    let recorded: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM recruitment_responses WHERE call_id = $1 AND respondent = $2",
    )
    .bind(&call_id)
    .bind(&outsider_role)
    .fetch_one(&pool)
    .await
    .expect("the response count");
    if recorded != 0 {
        breaches.push(format!(
            "C: {recorded} response row(s) from a foreign tenant survived"
        ));
    }

    // Leg D — tenant B's OWN role still answers, so the repair is not a
    // blanket refusal.
    let (status, insider) = enroll(
        &client,
        &base,
        json!({
            "kind": "role",
            "name": "mcpw-insider",
            "tenant_id": tenant_b,
            "actions": ["thread_contribute", "thread_invitation_respond"],
        }),
    )
    .await;
    assert_eq!(status, 200, "the insider role enrolls: {insider}");
    let insider_role = insider["principal_id"].as_str().unwrap().to_string();

    let (status, body) = post(
        &client,
        &base,
        &format!("/v1/calls/{call_id}/respond"),
        &insider_role,
        &json!({ "kind": "decline" }),
    )
    .await;
    if status != 200 {
        breaches.push(format!("D/http: the call's own tenant was refused: {body}"));
    }

    let outcome = reasonbraid_server::mcp_write_internal::join_call(
        &pool,
        &tenant_b,
        &role_subject(&insider_role),
        &call_id,
        json!({ "kind": "recuse", "reason_class": "conflict_of_interest" }),
    )
    .await;
    if let Err(refused) = outcome {
        breaches.push(format!(
            "D/seam: the call's own tenant was refused: {}: {}",
            refused.family, refused.message
        ));
    }

    assert!(
        breaches.is_empty(),
        "{} of the call-binding legs breach:\n{}",
        breaches.len(),
        breaches.join("\n")
    );
}

/// `SIGNOFF-REPAIR.6.1.3` — the write seam refuses a body it cannot index.
///
/// 🔴 `mcp_write::respond` took a `serde_json::Value` it received untyped and
/// wrote `body["tenant_id"] = …` into it. `IndexMut<&str>` for `Value` PANICS on
/// a string, a number, a bool or an array, so a caller that handed the `pub`
/// seam a non-object UNWOUND the task instead of being refused.
///
/// ⚠️ **Bounded honestly, and the bound is why this control calls the function
/// DIRECTLY.** The MCP tool serializes a typed `ContributePayload`, so the panic
/// is not reachable through that surface today. `respond` is `pub` through
/// `mcp_write_internal` and takes a `Value`, which is where the promise is made
/// and where the guard belongs — `SIGNOFF-REPAIR.4.2.7`'s promoted rule, *a
/// signature is a promise the body must keep*, on a third surface.
#[tokio::test]
async fn the_write_seam_refuses_a_body_it_cannot_index() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let (human_id, tenant, role_id) = bootstrap(
        &client,
        &base,
        "mcpw-untyped",
        json!(["thread_contribute", "thread_invitation_respond"]),
    )
    .await;

    // A real thread the role may contribute to, so the gate and the authz both
    // PASS and the body is the only thing under test. A control that refused at
    // the gate would pass against the unrepaired code for the wrong reason.
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &envelope(
            "thread.create",
            "ut-create",
            json!({
                "tenant_id": tenant,
                "subject": "the untyped body",
                "objective": "prove the seam refuses what it cannot index",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let (status, invited) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &human_id,
        &envelope(
            "thread.invite",
            "ut-invite",
            json!({ "tenant_id": tenant, "agent_role": role_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite: {invited}");
    let (status, accepted) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &role_id,
        &envelope(
            "thread.accept_invitation",
            "ut-accept",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "accept: {accepted}");

    // ── THE REFUSALS ─────────────────────────────────────────────────────────
    // Every JSON shape that is not an object. The string, the number, the bool
    // and the array each PANICKED before this leaf; `null` did not, because
    // serde_json silently replaces a Null with an empty object — so it reached
    // the downstream parse and refused there, for a reason that named the
    // missing fields rather than the real problem. All five are refused at the
    // seam now, and `null` is included deliberately: fabricating an object out
    // of a caller's `null` is not a service to anybody.
    for body in [
        json!("a bare string"),
        json!(7),
        json!(true),
        json!(["an", "array"]),
        json!(null),
    ] {
        let shape = match &body {
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Bool(_) => "bool",
            Value::Array(_) => "array",
            Value::Null => "null",
            Value::Object(_) => unreachable!("the object is the accepted shape"),
        };
        let err = reasonbraid_server::mcp_write_internal::respond(
            &pool,
            &tenant,
            &role_subject(&role_id),
            &thread_id,
            body.clone(),
        )
        .await
        .expect_err(&format!("a {shape} body must be REFUSED, never accepted"));
        assert_eq!(
            err.family, "handler:invalid_command",
            "a {shape} body is a typed refusal, not a panic and not an authority \
             failure: {err:?}"
        );
        assert!(
            err.message.contains("JSON object"),
            "the refusal names what was wrong with the body, so a caller can fix \
             it — a {shape} body said: {}",
            err.message
        );
    }

    // ⭐ THE POSITIVE ARM, without which the guard is indistinguishable from
    // refusing everything: the ordinary object body still lands, and the seam
    // still injects the tenant the caller must not send twice.
    let ok = reasonbraid_server::mcp_write_internal::respond(
        &pool,
        &tenant,
        &role_subject(&role_id),
        &thread_id,
        json!({ "content": "the object body still lands", "kind": "claim" }),
    )
    .await
    .expect("the object body is unaffected by the guard");
    assert_eq!(ok["status"], json!(200), "the accepted respond: {ok}");
}

/// `SIGNOFF-REPAIR.6.1.4` — the quota counts the ADMITTED CALL, and the
/// pipeline counts the EFFECT. They are two transactions and this proves it.
///
/// ⛔ **This is not a defect report.** `gate()` commits its quota use in its own
/// transaction before `run_thread_command` applies idempotency, and
/// `mcp_write.rs`'s header has always said so. What was missing is a control:
/// no test observed the two commits separately, so nothing would have noticed if
/// the split had been closed by accident — and test 3's heading named *the
/// effect + the audited allowance + the quota use* as one pipeline, which is
/// true of two of the three. Both directions are asserted here.
///
/// The decision behind it: `docs/decisions/2026-09-19_the-write-quota-counts-calls-not-effects.md`.
#[tokio::test]
async fn the_quota_counts_the_admitted_call_and_not_the_effect() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let (human_id, tenant, role_id) = bootstrap(
        &client,
        &base,
        "mcpw-quota",
        json!(["thread_contribute", "thread_invitation_respond"]),
    )
    .await;

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
                "subject": "the quota split",
                "objective": "prove the call and the effect are counted apart",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let (status, invited) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &human_id,
        &envelope(
            "thread.invite",
            "q-invite",
            json!({ "tenant_id": tenant, "agent_role": role_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite: {invited}");
    let (status, accepted) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &role_id,
        &envelope(
            "thread.accept_invitation",
            "q-accept",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "accept: {accepted}");

    let contributions = |pool: PgPool, thread_id: String, tenant: String| async move {
        let n: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM event_log \
             WHERE tenant_id = $1 AND aggregate_id = $2 \
             AND event_type = 'thread.contribution_submitted'",
        )
        .bind(&tenant)
        .bind(&thread_id)
        .fetch_one(&pool)
        .await
        .expect("the contribution count");
        n
    };

    // ── DIRECTION 1: an ADMITTED call whose handler REFUSES still spends the
    //    quota. The gate's transaction has already committed by the time the
    //    handler decides, and that is the bound's whole purpose: the abuse
    //    surface is the CALL, so a caller whose every call is refused must not
    //    get an unlimited supply of them.
    let (status, ghost) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "mcpw-quota-ghost", "tenant_id": tenant, "actions": [] }),
    )
    .await;
    assert_eq!(status, 200, "the ghost enrolls: {ghost}");
    let ghost_id = ghost["principal_id"].as_str().unwrap().to_string();
    assert_eq!(
        principal_uses(&pool, &tenant, &ghost_id, "use").await,
        0,
        "the ghost has made no call yet"
    );

    let err = reasonbraid_server::mcp_write_internal::respond(
        &pool,
        &tenant,
        &role_subject(&ghost_id),
        &thread_id,
        json!({ "content": "the refused write", "kind": "claim" }),
    )
    .await
    .expect_err("the ungranted role is refused by the handler");
    assert_eq!(
        err.family, "handler:unauthorized",
        "the GATE admitted it and the HANDLER refused it — a quota refusal here \
         would mean this arm never reached the split it is measuring: {err:?}"
    );
    assert_eq!(
        principal_uses(&pool, &tenant, &ghost_id, "use").await,
        1,
        "the admitted call spent quota although its effect never happened"
    );
    assert_eq!(
        principal_uses(&pool, &tenant, &ghost_id, "denial").await,
        0,
        "and it is NOT recorded as a quota denial — the quota admitted this call; \
         something downstream refused it"
    );

    // ── DIRECTION 2: an admitted call whose handler SUCCEEDS, then the same
    //    call REPLAYED. The replay is idempotent — the key is deterministic over
    //    (thread, principal, body) — so the pipeline produces no second effect,
    //    while the quota counts a second call. One caller, two calls, one
    //    contribution: the two commits, observed by their outcomes rather than
    //    by interrupting between them.
    let before = principal_uses(&pool, &tenant, &role_id, "use").await;
    let body = json!({ "content": "the counted write", "kind": "claim" });
    let first = reasonbraid_server::mcp_write_internal::respond(
        &pool,
        &tenant,
        &role_subject(&role_id),
        &thread_id,
        body.clone(),
    )
    .await
    .expect("the granted respond lands");
    assert_eq!(first["status"], json!(200), "the first call: {first}");
    assert_eq!(
        principal_uses(&pool, &tenant, &role_id, "use").await,
        before + 1,
        "the admitted call is counted once"
    );
    let after_first = contributions(pool.clone(), thread_id.clone(), tenant.clone()).await;
    assert_eq!(after_first, 1, "exactly one contribution so far");

    let replay = reasonbraid_server::mcp_write_internal::respond(
        &pool,
        &tenant,
        &role_subject(&role_id),
        &thread_id,
        body.clone(),
    )
    .await
    .expect("the replay returns the original result");
    // ⭐ The replay returns the ORIGINAL EVENT — same `event_id` — and says so
    // in the body. A result that merely looked the same would not distinguish a
    // replay from a second contribution that happened to match; the id does.
    assert_eq!(
        replay["result"]["event_id"], first["result"]["event_id"],
        "the replay returns the ORIGINAL event, not a second one: {replay}"
    );
    assert_eq!(
        replay["result"]["replayed"],
        json!(true),
        "and the pipeline says so rather than leaving it to be inferred: {replay}"
    );
    assert_eq!(
        contributions(pool.clone(), thread_id.clone(), tenant.clone()).await,
        1,
        "the replay produced NO second effect — the idempotency held"
    );
    assert_eq!(
        principal_uses(&pool, &tenant, &role_id, "use").await,
        before + 2,
        "and it spent quota anyway: the quota counts CALLS, the pipeline counts \
         EFFECTS, and they are two transactions"
    );
}
