//! The federation trust-agreement proof (`PHASE-8.1.2`, ADR-026, migration
//! 0048): the NAMED tenant-to-tenant pairing is the single capability
//! source. Measured:
//!   - WITHOUT an agreement, a cross-tenant reader sees the NETWORK view
//!     (the pseudonym class — the tenant fields ABSENT);
//!   - a ONE-SIDED proposal widens nothing;
//!   - the BOTH-SIDES accepted agreement widens the visibility to the
//!     TENANT view (the agreed scope — exactly what the agreement names);
//!   - a revocation falls back to the network view;
//!   - a THIRD tenant never inherits the agreement (no transitive
//!     default);
//!   - accepting without a proposal is the typed 409.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

// The guard primitive itself, compiled from the exact production source, so the
// ordering control below holds the same key the routes take
// (`SIGNOFF-REPAIR.3.3.4.12`).
#[allow(dead_code)]
#[path = "../src/authority/transaction.rs"]
mod tenant_transaction;

use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::Duration;

use reasonbraid_core::TenantId;
use tenant_transaction::{transact, GuardError, GuardMode};
use tokio::sync::oneshot;
use tokio::time::timeout;

use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static FEDERATION_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    FEDERATION_LOCK
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
    // Preserve the original scope plus its explicit FK dependencies; retain the CA.
    pg_cleanup::delete_tables(
        &pool,
        &[
            "federation_agreements",
            "quota_events",
            "usage_quotas",
            "outbox_delivery",
            "outbox",
            "node_events",
            "node_inbox",
            "budget_reservations",
            "budget_ceilings",
            "administrative_effects",
            "authorization_records",
            "authority_grants",
            "enrollments",
            "enrollment_boundaries",
            "profile_versions",
            "agent_profiles",
            "runs",
            "incarnations",
            "recruitment_offers",
            "agent_roles",
            "human_principals",
            "idempotency",
            "event_log",
            "aggregate_state",
            "cross_domain_receipts",
            "tenant_bootstrap_requests",
            "node_certificates",
            "node_keys",
            "node_leases",
            "nodes",
            "hosts",
            "mcp_listen_state",
            "node_enrollment_tokens",
            "recruitment_panels",
            "recruitment_responses",
            "recruitment_calls",
            "spend_breakers",
            "tenants",
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
    (status, response.json().await.expect("post json"))
}

async fn put(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    body: &Value,
) -> (u16, Value) {
    let response = client
        .put(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(body)
        .send()
        .await
        .expect("put request");
    let status = response.status().as_u16();
    let text = response.text().await.expect("put body");
    let body = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
    (status, body)
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

/// A profile with the explicit per-field policy (the profiles suite's
/// shape): the capabilities stay tenant-scoped, the interests reach the
/// network — the agreement's widening is the observable difference.
fn visibility_profile() -> Value {
    json!({
        "display_label": "visible everywhere",
        "purpose": "probe the per-reader filtering",
        "conversation_modes": ["architecture_deliberation"],
        "capabilities": [{
            "taxonomy_id": "code_review",
            "confidence": "self_asserted",
        }],
        "interests": ["parser trivia"],
        "languages": ["en"],
        "structured_output_formats": ["json"],
        "scopes": ["repo:example/parser"],
        "confidentiality_classes": ["internal"],
        "cost_latency_class": "cheap",
        "resource_ceilings": { "calls": 100 },
        "visibility": {
            "display_label": "public",
            "purpose": "network",
            "conversation_modes": "tenant",
            "capabilities": "tenant",
            "interests": "network",
            "languages": "network",
            "structured_output_formats": "tenant",
            "scopes": "tenant",
            "confidentiality_classes": "self_only",
            "availability": "tenant",
            "resolver_tool_capabilities": "tenant",
            "cost_latency_class": "tenant",
            "resource_ceilings": "self_only",
            "grants_by_reference": "self_only",
        },
    })
}

#[tokio::test]
async fn the_agreement_widens_the_visibility_and_never_transitively() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // The three tenants (A, B, C) + the role whose profile A hosts.
    let (status, human_a) =
        enroll(&client, &base, json!({ "kind": "human", "name": "fed-a" })).await;
    assert_eq!(status, 200, "A enrolls: {human_a}");
    let a_admin = human_a["principal_id"].as_str().unwrap().to_string();
    let tenant_a = human_a["tenant_id"].as_str().unwrap().to_string();
    let (status, human_b) =
        enroll(&client, &base, json!({ "kind": "human", "name": "fed-b" })).await;
    assert_eq!(status, 200, "B enrolls: {human_b}");
    let b_admin = human_b["principal_id"].as_str().unwrap().to_string();
    let tenant_b = human_b["tenant_id"].as_str().unwrap().to_string();
    let (status, human_c) =
        enroll(&client, &base, json!({ "kind": "human", "name": "fed-c" })).await;
    assert_eq!(status, 200, "C enrolls: {human_c}");
    let c_admin = human_c["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "fed-role", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let (status, written) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &visibility_profile(),
    )
    .await;
    assert_eq!(status, 200, "the role writes the profile: {written}");

    async fn read(
        client: &reqwest::Client,
        base: &str,
        role_id: &str,
        principal: &str,
    ) -> (u16, Value) {
        get(client, base, &format!("/v1/profiles/{role_id}"), principal).await
    }

    // 1. WITHOUT an agreement: the cross-tenant reader sees the NETWORK
    //    view (the tenant fields ABSENT).
    let (status, network) = read(&client, &base, &role_id, &b_admin).await;
    assert_eq!(status, 200, "B reads: {network}");
    assert_eq!(network["visibility"], json!("network"));
    assert!(
        !network["profile"]
            .as_object()
            .unwrap()
            .contains_key("capabilities"),
        "the network view absents the tenant fields: {network}"
    );

    // 2. The ONE-SIDED proposal (A proposes toward B) widens NOTHING.
    let (status, proposed) = post(
        &client,
        &base,
        "/v1/federation-agreements",
        &a_admin,
        &json!({
            "tenant_id": tenant_a,
            "remote_tenant_id": tenant_b,
            "directory_visibility": true,
            "recruitment": false,
        }),
    )
    .await;
    assert_eq!(status, 200, "A proposes: {proposed}");
    let (_, still_network) = read(&client, &base, &role_id, &b_admin).await;
    assert_eq!(still_network["visibility"], json!("network"));
    assert_eq!(
        network["profile"], still_network["profile"],
        "the one-sided proposal widens nothing"
    );

    // 3. Accepting WITHOUT a proposal is the typed 409.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/federation-agreements/accept",
        &b_admin,
        &json!({ "tenant_id": tenant_b, "remote_tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(
        status, 409,
        "the accept without a proposal refuses: {refused}"
    );

    // 4. The BOTH-SIDES pairing: B proposes back + both accept → the
    //    EFFECTIVE agreement widens B's read to the TENANT view.
    let (status, proposed) = post(
        &client,
        &base,
        "/v1/federation-agreements",
        &b_admin,
        &json!({
            "tenant_id": tenant_b,
            "remote_tenant_id": tenant_a,
            "directory_visibility": true,
            "recruitment": false,
        }),
    )
    .await;
    assert_eq!(status, 200, "B proposes back: {proposed}");
    let (status, accepted) = post(
        &client,
        &base,
        "/v1/federation-agreements/accept",
        &a_admin,
        &json!({ "tenant_id": tenant_a, "remote_tenant_id": tenant_b }),
    )
    .await;
    assert_eq!(status, 200, "A accepts: {accepted}");
    let (status, accepted) = post(
        &client,
        &base,
        "/v1/federation-agreements/accept",
        &b_admin,
        &json!({ "tenant_id": tenant_b, "remote_tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "B accepts: {accepted}");
    let (status, tenant_view) = read(&client, &base, &role_id, &b_admin).await;
    assert_eq!(status, 200, "B re-reads: {tenant_view}");
    assert_eq!(tenant_view["visibility"], json!("tenant"));
    assert!(
        tenant_view["profile"]
            .as_object()
            .unwrap()
            .contains_key("capabilities"),
        "the agreement widens to the tenant view: {tenant_view}"
    );

    // 5. The revocation falls back (B revokes its direction).
    let (status, revoked) = post(
        &client,
        &base,
        "/v1/federation-agreements/revoke",
        &b_admin,
        &json!({ "tenant_id": tenant_b, "remote_tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "B revokes: {revoked}");
    let (_, fallen_back) = read(&client, &base, &role_id, &b_admin).await;
    assert_eq!(fallen_back["visibility"], json!("network"));

    // 6. The THIRD tenant never inherits (the transitive-default refusal).
    let (_, stranger_view) = read(&client, &base, &role_id, &c_admin).await;
    assert_eq!(
        stranger_view["visibility"],
        json!("network"),
        "no agreement, no widening — C sees the pseudonym only"
    );
}

// ── Guarded direction administration (`SIGNOFF-REPAIR.3.3.4.12`) ──────────────
//
// Each verb admitted through its own already-committed transaction and then
// mutated ON THE POOL, with nothing recording what the request finally did.

/// Every direction verb records its outcome and carries its receipt, and the
/// three idle states that used to be indistinguishable are distinguished in the
/// record while the wire is unchanged.
#[tokio::test]
async fn each_direction_verb_records_what_it_did() {
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, a) = enroll(&client, &base, json!({ "kind": "human", "name": "dir-a" })).await;
    assert_eq!(status, 200, "A enrolls: {a}");
    let a_admin = a["principal_id"].as_str().unwrap().to_string();
    let tenant_a = a["tenant_id"].as_str().unwrap().to_string();
    let (status, b) = enroll(&client, &base, json!({ "kind": "human", "name": "dir-b" })).await;
    assert_eq!(status, 200, "B enrolls: {b}");
    let tenant_b = b["tenant_id"].as_str().unwrap().to_string();

    let call = |path: &'static str, principal: String, body: Value| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let response = client
                .post(format!("{base}{path}"))
                .header(PRINCIPAL_HEADER, principal)
                .json(&body)
                .send()
                .await
                .expect("request");
            let status = response.status().as_u16();
            let receipt = response
                .headers()
                .get("x-reasonbraid-authorization")
                .map(|v| v.to_str().unwrap().to_string());
            let body: Value = response.json().await.expect("json");
            (status, receipt, body)
        }
    };
    let recorded = |receipt: String| {
        let pool = pool.clone();
        async move {
            sqlx::query_as::<_, (Value, Value)>(
                "SELECT operation, outcome FROM administrative_effects WHERE record_id = $1",
            )
            .bind(receipt)
            .fetch_one(&pool)
            .await
            .expect("the effect record exists")
        }
    };

    let terms = json!({
        "tenant_id": tenant_a,
        "remote_tenant_id": tenant_b,
        "directory_visibility": true,
        "recruitment": true,
    });

    // PROPOSE — applied.
    let (status, receipt, body) =
        call("/v1/federation-agreements", a_admin.clone(), terms.clone()).await;
    assert_eq!(status, 200, "the proposal: {body}");
    assert_eq!(body["status"], json!("proposed"), "{body}");
    let agreement_id = body["agreement_id"].as_str().unwrap().to_string();
    let (operation, outcome) = recorded(receipt.expect("the proposal carries its receipt")).await;
    assert_eq!(
        operation["kind"],
        json!("federation_direction_propose"),
        "{operation}"
    );
    assert_eq!(
        operation["remote_tenant_id"],
        json!(tenant_b),
        "{operation}"
    );
    assert_eq!(outcome["kind"], json!("applied"), "{outcome}");

    // RE-PROPOSE on identical terms — the same answer on the wire, `no_op` in
    // the record. Re-proposing DIFFERENT terms would apply, and would reset an
    // accepted direction to proposed; that behaviour is preserved, not changed.
    let (status, receipt, body) =
        call("/v1/federation-agreements", a_admin.clone(), terms.clone()).await;
    assert_eq!(status, 200, "the re-proposal answers identically: {body}");
    assert_eq!(body["agreement_id"], json!(agreement_id), "{body}");
    assert_eq!(body["status"], json!("proposed"), "{body}");
    let (_, outcome) = recorded(receipt.expect("the re-proposal carries its receipt")).await;
    assert_eq!(
        outcome["kind"],
        json!("no_op"),
        "an unchanged re-proposal records a no-op: {outcome}"
    );

    // ACCEPT — applied, with its cross-domain receipt in the same transaction.
    let action = json!({ "tenant_id": tenant_a, "remote_tenant_id": tenant_b });
    let (status, receipt, body) = call(
        "/v1/federation-agreements/accept",
        a_admin.clone(),
        action.clone(),
    )
    .await;
    assert_eq!(status, 200, "the acceptance: {body}");
    assert_eq!(body["status"], json!("accepted"), "{body}");
    let (operation, outcome) = recorded(receipt.expect("the acceptance carries its receipt")).await;
    assert_eq!(
        operation["kind"],
        json!("federation_direction_accept"),
        "{operation}"
    );
    assert_eq!(outcome["kind"], json!("applied"), "{outcome}");
    let receipts: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM cross_domain_receipts WHERE tenant_id = $1 AND kind = 'agreement'",
    )
    .bind(&tenant_a)
    .fetch_one(&pool)
    .await
    .expect("count receipts");
    assert_eq!(receipts, 1, "the acceptance wrote its cross-domain receipt");

    // ACCEPT AGAIN — the SAME 409 on the wire, `no_op` in the record.
    let (status, receipt, body) = call(
        "/v1/federation-agreements/accept",
        a_admin.clone(),
        action.clone(),
    )
    .await;
    assert_eq!(status, 409, "the repeat acceptance: {body}");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("no PROPOSED agreement"),
        "the message is unchanged: {body}"
    );
    let (_, outcome) = recorded(receipt.expect("the repeat carries its receipt")).await;
    assert_eq!(
        outcome["kind"],
        json!("no_op"),
        "an already-accepted direction is already satisfied: {outcome}"
    );

    // REVOKE — applied.
    let (status, receipt, body) = call(
        "/v1/federation-agreements/revoke",
        a_admin.clone(),
        action.clone(),
    )
    .await;
    assert_eq!(status, 200, "the revocation: {body}");
    assert_eq!(body["revoked"], json!(1), "{body}");
    let (operation, outcome) = recorded(receipt.expect("the revocation carries its receipt")).await;
    assert_eq!(
        operation["kind"],
        json!("federation_direction_revoke"),
        "{operation}"
    );
    assert_eq!(outcome["kind"], json!("applied"), "{outcome}");

    // REVOKE AGAIN — the SAME `revoked: 0` on the wire, `no_op` in the record.
    let (status, receipt, body) = call(
        "/v1/federation-agreements/revoke",
        a_admin.clone(),
        action.clone(),
    )
    .await;
    assert_eq!(status, 200, "the repeat revocation: {body}");
    assert_eq!(body["revoked"], json!(0), "{body}");
    let (_, outcome) = recorded(receipt.expect("the repeat carries its receipt")).await;
    assert_eq!(outcome["kind"], json!("no_op"), "{outcome}");

    // ACCEPT after a revocation — the same 409, but `refused` rather than
    // `no_op`: there is no proposed direction here at all.
    let (status, receipt, body) =
        call("/v1/federation-agreements/accept", a_admin.clone(), action).await;
    assert_eq!(status, 409, "accepting a revoked direction: {body}");
    let (_, outcome) = recorded(receipt.expect("the refusal carries its receipt")).await;
    assert_eq!(outcome["kind"], json!("refused"), "{outcome}");
    assert_eq!(outcome["code"], json!("invalid_transition"), "{outcome}");
}

/// 🔴 Proposing to a tenant that does not exist was a RAISED foreign-key
/// violation and a `500`. It is a typed `404` now, recorded — the refusal a
/// constraint used to deliver is a value the transaction can commit beside.
#[tokio::test]
async fn proposing_to_an_unknown_tenant_refuses_in_the_record_rather_than_raising() {
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, a) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "dir-unknown" }),
    )
    .await;
    assert_eq!(status, 200, "A enrolls: {a}");
    let a_admin = a["principal_id"].as_str().unwrap().to_string();
    let tenant_a = a["tenant_id"].as_str().unwrap().to_string();
    let absent = "ten_01a00000-0000-7000-8000-0000000009f9";

    let response = client
        .post(format!("{base}/v1/federation-agreements"))
        .header(PRINCIPAL_HEADER, &a_admin)
        .json(&json!({
            "tenant_id": tenant_a,
            "remote_tenant_id": absent,
            "directory_visibility": true,
            "recruitment": true,
        }))
        .send()
        .await
        .expect("propose");
    let status = response.status().as_u16();
    let receipt = response
        .headers()
        .get("x-reasonbraid-authorization")
        .map(|v| v.to_str().unwrap().to_string());
    let body: Value = response.json().await.expect("json");

    assert_eq!(
        status, 404,
        "an absent counterparty is a typed refusal, not a storage failure: {body}"
    );
    assert_eq!(body["code"], json!("not_found"), "{body}");
    let (operation, outcome): (Value, Value) = sqlx::query_as(
        "SELECT operation, outcome FROM administrative_effects WHERE record_id = $1",
    )
    .bind(receipt.expect("the refusal carries its receipt"))
    .fetch_one(&pool)
    .await
    .expect("the refusal's effect record exists");
    assert_eq!(
        operation["kind"],
        json!("federation_direction_propose"),
        "{operation}"
    );
    assert_eq!(outcome["kind"], json!("refused"), "{outcome}");
    assert_eq!(outcome["code"], json!("not_found"), "{outcome}");

    let rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM federation_agreements WHERE tenant_id = $1")
            .bind(&tenant_a)
            .fetch_one(&pool)
            .await
            .expect("count");
    assert_eq!(rows, 0, "the refusal recorded no direction");
}

/// Hold one tenant's authority guard until released, so a request that must
/// queue behind it can be observed queuing.
struct Holder {
    release: oneshot::Sender<()>,
    job: tokio::task::JoinHandle<()>,
}

async fn hold(pool: &PgPool, tenant: TenantId, mode: GuardMode) -> Holder {
    let pool = pool.clone();
    let (entered_tx, entered) = oneshot::channel();
    let (release, release_rx) = oneshot::channel();
    let job = tokio::spawn(async move {
        transact(&pool, &[(tenant, mode)], move |_| {
            Box::pin(async move {
                let _ = entered_tx.send(());
                let _ = release_rx.await;
                Ok::<(), GuardError>(())
            })
        })
        .await
        .expect("the holder's guarded transaction completes");
    });
    timeout(Duration::from_secs(5), entered)
        .await
        .expect("the holder acquires its guard")
        .expect("the holder reports entry");
    Holder { release, job }
}

impl Holder {
    async fn release(self) {
        let _ = self.release.send(());
        timeout(Duration::from_secs(5), self.job)
            .await
            .expect("the holder finishes")
            .expect("the holder's task joins");
    }
}

/// 🔴 THE ordering control: authority that ends while a direction revocation
/// queues must be seen as ended.
///
/// ⚠️ The holder is SHARED, and the choice is derived rather than copied. This
/// repair moves the operation from a shared-guard admission plus an unguarded
/// pool mutation to ONE exclusive-guard transaction, so a shared holder
/// discriminates it: the repaired verb must wait, and the superseded one did not
/// — its admission took the shared mode, which does not exclude shared, and its
/// mutation took no guard at all (`docs/knowledge/proving-a-race-is-closed.md`).
#[tokio::test]
async fn authority_that_ends_while_a_direction_revocation_waits_refuses_it() {
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, a) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "dir-race-a" }),
    )
    .await;
    assert_eq!(status, 200, "A enrolls: {a}");
    let a_admin = a["principal_id"].as_str().unwrap().to_string();
    let tenant_a = a["tenant_id"].as_str().unwrap().to_string();
    let admin_grant = a["grant_id"].as_str().unwrap().to_string();
    let (status, b) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "dir-race-b" }),
    )
    .await;
    assert_eq!(status, 200, "B enrolls: {b}");
    let tenant_b = b["tenant_id"].as_str().unwrap().to_string();

    // A live direction to revoke.
    let (status, proposed) = post(
        &client,
        &base,
        "/v1/federation-agreements",
        &a_admin,
        &json!({
            "tenant_id": tenant_a,
            "remote_tenant_id": tenant_b,
            "directory_visibility": true,
            "recruitment": true,
        }),
    )
    .await;
    assert_eq!(status, 200, "the proposal: {proposed}");

    let tenant: TenantId = tenant_a.parse().expect("a tenant id");
    let holder = hold(&pool, tenant, GuardMode::Shared).await;
    let (base2, client2, admin2, a2, b2) = (
        base.clone(),
        client.clone(),
        a_admin.clone(),
        tenant_a.clone(),
        tenant_b.clone(),
    );
    let request = tokio::spawn(async move {
        post(
            &client2,
            &base2,
            "/v1/federation-agreements/revoke",
            &admin2,
            &json!({ "tenant_id": a2, "remote_tenant_id": b2 }),
        )
        .await
    });
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(!request.is_finished(), "the revocation is waiting");

    // End the caller's OWN administration underneath the blocked request. The
    // decision time is sampled after the guard wait, so this must be seen.
    sqlx::query("UPDATE authority_grants SET status = 'revoked' WHERE grant_id = $1")
        .bind(&admin_grant)
        .execute(&pool)
        .await
        .expect("end the caller's authority");
    holder.release().await;

    let (status, body) = timeout(Duration::from_secs(10), request)
        .await
        .expect("the revocation completes")
        .expect("the request task joins");
    assert_eq!(
        status, 403,
        "authority that ended during the wait is gone: {body}"
    );

    // And it changed nothing: the direction is still live.
    let status_now: String = sqlx::query_scalar(
        "SELECT status FROM federation_agreements WHERE tenant_id = $1 AND remote_tenant_id = $2",
    )
    .bind(&tenant_a)
    .bind(&tenant_b)
    .fetch_one(&pool)
    .await
    .expect("the direction row");
    assert_eq!(
        status_now, "proposed",
        "a denied revocation leaves the direction alone"
    );
}
