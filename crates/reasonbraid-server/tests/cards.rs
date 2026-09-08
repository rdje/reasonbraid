//! The portable agent-card proof (`PHASE-8.1.3`, ADR-026/027): the
//! digest-pinned export + the four-rung import ladder. Measured:
//!   - the export mints the card with the `sha256:<hex>` digest;
//!   - the import WITHOUT the agreement refuses (the allowlist rung);
//!   - the import with the EFFECTIVE recruitment agreement lands the
//!     local role + the imported profile;
//!   - a TAMPERED card refuses at the digest rung (the re-derivation);
//!   - an unsupported schema refuses at the compatibility rung;
//!   - the card confers NOTHING beyond the local default grant (the
//!     imported role's grant is the boundary-checked default).
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static CARDS_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    CARDS_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the card proof"
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
        "federation_agreements",
        "cross_domain_receipts",
        "quota_events",
        "usage_quotas",
        "outbox_delivery",
        "outbox",
        "node_events",
        "node_inbox",
        "budget_reservations",
        "budget_ceilings",
        "spend_breakers",
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
    let text = response.text().await.expect("post body");
    let body = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
    (status, body)
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
    let text = response.text().await.expect("get body");
    let body = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
    (status, body)
}

fn sample_profile() -> Value {
    json!({
        "display_label": "card-role",
        "purpose": "the imported role",
        "conversation_modes": ["architecture_deliberation"],
        "capabilities": [{ "taxonomy_id": "code_review", "confidence": "self_asserted" }],
        "interests": ["parser trivia"],
        "languages": ["en"],
        "structured_output_formats": ["json"],
        "scopes": ["repo:example/parser"],
        "confidentiality_classes": ["internal"],
        "cost_latency_class": "cheap",
        "resource_ceilings": { "calls": 100 },
    })
}

#[tokio::test]
async fn the_card_import_runs_the_ladder_and_lands_the_local_role() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // The origin tenant A + the role + the profile.
    let (status, human_a) =
        enroll(&client, &base, json!({ "kind": "human", "name": "card-a" })).await;
    assert_eq!(status, 200, "A enrolls: {human_a}");
    let a_admin = human_a["principal_id"].as_str().unwrap().to_string();
    let tenant_a = human_a["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "card-role", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let (status, written) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &sample_profile(),
    )
    .await;
    assert_eq!(status, 200, "the profile writes: {written}");

    // The importer tenant B.
    let (status, human_b) =
        enroll(&client, &base, json!({ "kind": "human", "name": "card-b" })).await;
    assert_eq!(status, 200, "B enrolls: {human_b}");
    let b_admin = human_b["principal_id"].as_str().unwrap().to_string();
    let tenant_b = human_b["tenant_id"].as_str().unwrap().to_string();

    // 1. The export mints the card + the digest.
    let (status, exported) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/card"),
        &role_id,
    )
    .await;
    assert_eq!(status, 200, "the card exports: {exported}");
    let card = exported["card"].clone();
    let digest = exported["digest"].as_str().unwrap().to_string();
    assert!(digest.starts_with("sha256:"), "the ADR-011 digest");
    assert_eq!(card["origin_tenant_id"], json!(tenant_a));

    let import_body = |card: Value, digest: &str| json!({ "tenant_id": tenant_b, "card": card, "digest": digest });

    // 2. WITHOUT the agreement: the allowlist rung refuses.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/profiles/cards/import",
        &b_admin,
        &import_body(card.clone(), &digest),
    )
    .await;
    assert_eq!(status, 403, "the no-agreement import refuses: {refused}");

    // 3. The EFFECTIVE recruitment agreement (both sides).
    for (admin, tenant, remote) in [
        (&a_admin, &tenant_a, &tenant_b),
        (&b_admin, &tenant_b, &tenant_a),
    ] {
        let (status, proposed) = post(
            &client,
            &base,
            "/v1/federation-agreements",
            admin,
            &json!({
                "tenant_id": tenant,
                "remote_tenant_id": remote,
                "directory_visibility": false,
                "recruitment": true,
            }),
        )
        .await;
        assert_eq!(status, 200, "the propose: {proposed}");
    }
    for (admin, tenant, remote) in [
        (&a_admin, &tenant_a, &tenant_b),
        (&b_admin, &tenant_b, &tenant_a),
    ] {
        let (status, accepted) = post(
            &client,
            &base,
            "/v1/federation-agreements/accept",
            admin,
            &json!({ "tenant_id": tenant, "remote_tenant_id": remote }),
        )
        .await;
        assert_eq!(status, 200, "the accept: {accepted}");
    }

    // 4. The tampered card refuses at the digest rung (the re-derivation).
    let mut tampered = card.clone();
    tampered["profile"]["purpose"] = json!("the TAMPERED purpose");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/profiles/cards/import",
        &b_admin,
        &import_body(tampered, &digest),
    )
    .await;
    assert_eq!(status, 400, "the tampered card refuses: {refused}");

    // 5. The unsupported schema refuses at the compatibility rung.
    let mut wrong_schema = card.clone();
    wrong_schema["schema_version"] = json!("agent-card/99");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/profiles/cards/import",
        &b_admin,
        &import_body(wrong_schema, "sha256:00000000"),
    )
    .await;
    assert_eq!(status, 400, "the unknown schema refuses: {refused}");

    // 6. The clean import lands the local role + the imported profile.
    let (status, imported) = post(
        &client,
        &base,
        "/v1/profiles/cards/import",
        &b_admin,
        &import_body(card.clone(), &digest),
    )
    .await;
    assert_eq!(status, 200, "the import lands: {imported}");
    let local_role = imported["role_id"].as_str().unwrap().to_string();
    assert_ne!(local_role, role_id, "the local identity is fresh");
    let (status, local_profile) = get(
        &client,
        &base,
        &format!("/v1/profiles/{local_role}"),
        &local_role,
    )
    .await;
    assert_eq!(status, 200, "the imported profile reads: {local_profile}");
    assert_eq!(
        local_profile["profile"]["purpose"],
        json!("the imported role"),
        "the card's content landed locally"
    );

    // 7. The card conferred NOTHING beyond the local default grant: the
    //    imported role's grant is the boundary-checked default pair.
    let grant_id = format!("grt_{local_role}");
    let grant_actions: i32 = sqlx::query_scalar(
        "SELECT jsonb_array_length(actions) FROM authority_grants WHERE grant_id = $1",
    )
    .bind(&grant_id)
    .fetch_one(&pool)
    .await
    .expect("read the imported grant");
    assert_eq!(
        grant_actions, 2,
        "the card conferred only the default grant pair (the ADR-026 invariant)"
    );

    // 8. The cross-domain receipt (`.1.4`): the remote reference is the
    //    card's digest, the local reference is the fresh role — the
    //    receipt CROSS-REFERENCES, never merges the chains.
    let receipt: Option<(String, String)> = sqlx::query_as(
        "SELECT remote_ref, local_ref FROM cross_domain_receipts \
         WHERE tenant_id = $1 AND kind = 'card_import'",
    )
    .bind(&tenant_b)
    .fetch_optional(&pool)
    .await
    .expect("read the receipt");
    let Some((remote_ref, local_ref)) = receipt else {
        panic!("the import must record its cross-domain receipt");
    };
    assert_eq!(
        remote_ref, digest,
        "the remote reference is the card's digest"
    );
    assert_eq!(
        local_ref, local_role,
        "the local reference is the fresh role"
    );
    let (status, listed) = get(
        &client,
        &base,
        &format!("/v1/audit/receipts?tenant_id={tenant_b}"),
        &b_admin,
    )
    .await;
    assert_eq!(status, 200, "the receipts read: {listed}");
    let listed_kinds: Vec<String> = listed["receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["kind"].as_str().map(|k| k.to_string()))
        .collect();
    assert_eq!(
        listed_kinds
            .iter()
            .filter(|k| k.as_str() == "card_import")
            .count(),
        1,
        "the read surface returns the card-import receipt"
    );
    assert!(
        listed_kinds.iter().any(|k| k.as_str() == "agreement"),
        "the read surface also carries the agreement receipts"
    );
}
