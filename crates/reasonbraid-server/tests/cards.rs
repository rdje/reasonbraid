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

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

// The guard primitive itself, compiled from the exact production source, so the
// ordering control below holds the same key the import takes
// (`SIGNOFF-REPAIR.3.3.4.12.1`).
#[allow(dead_code)]
#[path = "../src/authority/transaction.rs"]
mod tenant_transaction;

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
            "administrative_effects",
            "node_enrollment_tokens",
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
            "tenant_bootstrap_requests",
            "node_certificates",
            "node_keys",
            "node_leases",
            "nodes",
            "hosts",
            "mcp_listen_state",
            "recruitment_panels",
            "recruitment_responses",
            "recruitment_calls",
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

    // A failed grant INSERT is a storage failure, with no imported identity,
    // grant, quota, enrollment, profile or cross-domain receipt left behind.
    let mut before = Vec::new();
    let tables = [
        "authority_grants",
        "agent_roles",
        "usage_quotas",
        "enrollments",
        "agent_profiles",
        "profile_versions",
        "cross_domain_receipts",
    ];
    for table in tables {
        before.push(sqlx::query_scalar::<_, Value>(&format!(
            "SELECT coalesce(jsonb_agg(row ORDER BY row::text), '[]'::jsonb) FROM (SELECT to_jsonb(t) AS row FROM {table} t) s"))
            .fetch_one(&pool).await.unwrap());
    }
    sqlx::query("CREATE FUNCTION rb_test_refuse_import_grant() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned import grant fault'; END $$")
        .execute(&pool).await.unwrap();
    sqlx::query("CREATE TRIGGER rb_test_import_grant_fault BEFORE INSERT ON authority_grants FOR EACH ROW EXECUTE FUNCTION rb_test_refuse_import_grant()")
        .execute(&pool).await.unwrap();
    let outcome = client
        .post(format!("{base}/v1/profiles/cards/import"))
        .header(PRINCIPAL_HEADER, &b_admin)
        .json(&import_body(card.clone(), &digest))
        .send()
        .await;
    sqlx::query("DROP TRIGGER rb_test_import_grant_fault ON authority_grants")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DROP FUNCTION rb_test_refuse_import_grant()")
        .execute(&pool)
        .await
        .unwrap();
    let mut after = Vec::new();
    for table in tables {
        after.push(sqlx::query_scalar::<_, Value>(&format!(
            "SELECT coalesce(jsonb_agg(row ORDER BY row::text), '[]'::jsonb) FROM (SELECT to_jsonb(t) AS row FROM {table} t) s"))
            .fetch_one(&pool).await.unwrap());
    }
    assert_eq!(before, after, "grant failure must leave no partial import");
    let response = outcome.expect("storage faults produce HTTP responses");
    let status = response.status().as_u16();
    let body: Value = response.json().await.unwrap();
    assert_eq!(
        status, 500,
        "storage failure is not a policy refusal: {body}"
    );
    assert_eq!(
        body,
        json!({"code":"dependency_unavailable", "message":"internal server error"})
    );

    // A structurally valid administrator may still request an import whose
    // default role actions exceed the importing boundary. Preserve that 400.
    let admin_grant = human_b["grant_id"].as_str().unwrap();
    let parent_actions: Value = sqlx::query_scalar(
        "SELECT permitted_actions FROM enrollment_boundaries WHERE tenant_id = $1",
    )
    .bind(&tenant_b)
    .fetch_one(&pool)
    .await
    .unwrap();
    let admin_actions: Value =
        sqlx::query_scalar("SELECT actions FROM authority_grants WHERE grant_id = $1")
            .bind(admin_grant)
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query("UPDATE enrollment_boundaries SET permitted_actions = $2 WHERE tenant_id = $1")
        .bind(&tenant_b)
        .bind(json!(["tenant_admin"]))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE authority_grants SET actions = $2 WHERE grant_id = $1")
        .bind(admin_grant)
        .bind(json!(["tenant_admin"]))
        .execute(&pool)
        .await
        .unwrap();
    let outcome = client
        .post(format!("{base}/v1/profiles/cards/import"))
        .header(PRINCIPAL_HEADER, &b_admin)
        .json(&import_body(card.clone(), &digest))
        .send()
        .await;
    sqlx::query("UPDATE enrollment_boundaries SET permitted_actions = $2 WHERE tenant_id = $1")
        .bind(&tenant_b)
        .bind(parent_actions)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE authority_grants SET actions = $2 WHERE grant_id = $1")
        .bind(admin_grant)
        .bind(admin_actions)
        .execute(&pool)
        .await
        .unwrap();
    let mut after = Vec::new();
    for table in tables {
        after.push(sqlx::query_scalar::<_, Value>(&format!(
            "SELECT coalesce(jsonb_agg(row ORDER BY row::text), '[]'::jsonb) FROM (SELECT to_jsonb(t) AS row FROM {table} t) s"))
            .fetch_one(&pool).await.unwrap());
    }
    assert_eq!(
        before, after,
        "structural refusal must leave no partial import"
    );
    let response = outcome.unwrap();
    let status = response.status().as_u16();
    let body: Value = response.json().await.unwrap();
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["code"], "invalid_command");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .starts_with("the imported role's grant exceeds the importing boundary: "),
        "{body}"
    );

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

// ── The atomic import (`SIGNOFF-REPAIR.3.3.4.11.3`) ───────────────────────────
//
// The superseded route committed the grant, the `agent_roles` row, the quota
// row, the enrollment row and the cross-domain receipt as one transaction, and
// then wrote the profile ON THE POOL. A failure there answered 500 with an
// imported identity already durable and no profile behind it.

/// 🔴 THE discriminating control: a profile write that cannot succeed must leave
/// NOTHING behind — no role, no grant, no quota row, no enrollment, no receipt,
/// and no admission. Against the superseded route every one of those survived a
/// request that told the caller it had failed.
#[tokio::test]
async fn a_profile_write_failure_leaves_no_imported_identity_behind() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human_a) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "atomic-a" }),
    )
    .await;
    assert_eq!(status, 200, "A enrolls: {human_a}");
    let a_admin = human_a["principal_id"].as_str().unwrap().to_string();
    let tenant_a = human_a["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "atomic-role", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &sample_profile(),
    )
    .await;
    assert_eq!(status, 200, "the profile writes");

    let (status, human_b) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "atomic-b" }),
    )
    .await;
    assert_eq!(status, 200, "B enrolls: {human_b}");
    let b_admin = human_b["principal_id"].as_str().unwrap().to_string();
    let tenant_b = human_b["tenant_id"].as_str().unwrap().to_string();

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

    for (admin, tenant, remote) in [
        (&a_admin, &tenant_a, &tenant_b),
        (&b_admin, &tenant_b, &tenant_a),
    ] {
        let (status, _) = post(
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
        assert_eq!(status, 200, "the propose");
        let (status, _) = post(
            &client,
            &base,
            "/v1/federation-agreements/accept",
            admin,
            &json!({ "tenant_id": tenant, "remote_tenant_id": remote }),
        )
        .await;
        assert_eq!(status, 200, "the accept");
    }

    let count = |sql: &'static str, bind: String| {
        let pool = pool.clone();
        async move {
            sqlx::query_scalar::<_, i64>(sql)
                .bind(bind)
                .fetch_one(&pool)
                .await
                .expect("count")
        }
    };
    let roles_before = count(
        "SELECT count(*) FROM agent_roles WHERE tenant_id = $1",
        tenant_b.clone(),
    )
    .await;
    let grants_before = count(
        "SELECT count(*) FROM authority_grants WHERE tenant_id = $1",
        tenant_b.clone(),
    )
    .await;
    let receipts_before = count(
        "SELECT count(*) FROM cross_domain_receipts WHERE tenant_id = $1",
        tenant_b.clone(),
    )
    .await;
    // Tenant B's own bootstrap already owns quota rows, so this is a
    // before/after comparison rather than a count of zero.
    let quotas_before = count(
        "SELECT count(*) FROM usage_quotas WHERE tenant_id = $1",
        tenant_b.clone(),
    )
    .await;
    // Likewise for admissions: B's own propose and accept each committed one.
    let admissions_before = count(
        "SELECT count(*) FROM authorization_records WHERE tenant_id = $1",
        tenant_b.clone(),
    )
    .await;

    // Make every NEW profile version row fail, touching no existing row: NOT
    // VALID applies to new rows only. The suite is single-threaded, so this is
    // the only writer while it stands.
    sqlx::query(
        "ALTER TABLE profile_versions \
         ADD CONSTRAINT signoff_repair_3_3_4_11_3_profile_fault CHECK (false) NOT VALID",
    )
    .execute(&pool)
    .await
    .expect("install the profile-write fault");

    // ⛔ Ordering: observe everything, REMOVE THE FAULT, and only then assert.
    // An assertion that fires while the fault stands leaks it into every later
    // test in the suite, for a reason that has nothing to do with them.
    let (status, body) = post(
        &client,
        &base,
        "/v1/profiles/cards/import",
        &b_admin,
        &json!({ "tenant_id": tenant_b, "card": card, "digest": digest }),
    )
    .await;
    let roles_after = count(
        "SELECT count(*) FROM agent_roles WHERE tenant_id = $1",
        tenant_b.clone(),
    )
    .await;
    let grants_after = count(
        "SELECT count(*) FROM authority_grants WHERE tenant_id = $1",
        tenant_b.clone(),
    )
    .await;
    let receipts_after = count(
        "SELECT count(*) FROM cross_domain_receipts WHERE tenant_id = $1",
        tenant_b.clone(),
    )
    .await;
    let enrollments_after = count(
        "SELECT count(*) FROM enrollments WHERE tenant_id = $1 AND kind = 'role'",
        tenant_b.clone(),
    )
    .await;
    let quotas_after = count(
        "SELECT count(*) FROM usage_quotas WHERE tenant_id = $1",
        tenant_b.clone(),
    )
    .await;
    let admissions_after = count(
        "SELECT count(*) FROM authorization_records WHERE tenant_id = $1",
        tenant_b.clone(),
    )
    .await;

    sqlx::query(
        "ALTER TABLE profile_versions DROP CONSTRAINT signoff_repair_3_3_4_11_3_profile_fault",
    )
    .execute(&pool)
    .await
    .expect("remove the profile-write fault");

    assert_eq!(
        status, 500,
        "a profile write failure is not reported as success: {body}"
    );
    assert_eq!(
        roles_after, roles_before,
        "the refused import left no imported role"
    );
    assert_eq!(
        grants_after, grants_before,
        "the refused import left no grant"
    );
    assert_eq!(
        receipts_after, receipts_before,
        "the refused import left no cross-domain receipt"
    );
    assert_eq!(
        enrollments_after, 0,
        "the refused import left no role enrollment"
    );
    assert_eq!(
        quotas_after, quotas_before,
        "the refused import left no per-principal quota row"
    );
    assert_eq!(
        admissions_after, admissions_before,
        "the import's own admission rolled back with the writes it permitted"
    );

    // And the route recovers: the same import now succeeds completely.
    let (status, imported) = post(
        &client,
        &base,
        "/v1/profiles/cards/import",
        &b_admin,
        &json!({ "tenant_id": tenant_b, "card": exported["card"].clone(), "digest": exported["digest"].as_str().unwrap() }),
    )
    .await;
    assert_eq!(status, 200, "the import recovers: {imported}");
    let local = imported["role_id"].as_str().unwrap().to_string();
    let (status, profile) = get(&client, &base, &format!("/v1/profiles/{local}"), &b_admin).await;
    assert_eq!(status, 200, "the imported profile is readable: {profile}");
}

/// Each rung's refusal is RECORDED, and the allowlist rung is the one post-
/// admission refusal in the fourteen administrative operations that answers
/// `unauthorized` (`SIGNOFF-REPAIR.3.3.4.7.4`).
#[tokio::test]
async fn each_import_rung_records_what_it_refused() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human_a) =
        enroll(&client, &base, json!({ "kind": "human", "name": "rung-a" })).await;
    assert_eq!(status, 200, "A enrolls: {human_a}");
    let tenant_a = human_a["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "rung-role", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &sample_profile(),
    )
    .await;
    assert_eq!(status, 200, "the profile writes");
    let (status, human_b) =
        enroll(&client, &base, json!({ "kind": "human", "name": "rung-b" })).await;
    assert_eq!(status, 200, "B enrolls: {human_b}");
    let b_admin = human_b["principal_id"].as_str().unwrap().to_string();
    let tenant_b = human_b["tenant_id"].as_str().unwrap().to_string();
    let (status, exported) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/card"),
        &role_id,
    )
    .await;
    assert_eq!(status, 200, "the card exports");
    let card = exported["card"].clone();
    let digest = exported["digest"].as_str().unwrap().to_string();

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
    let import = |body: Value, principal: String| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/profiles/cards/import"))
                .header(PRINCIPAL_HEADER, principal)
                .json(&body)
                .send()
                .await
                .expect("import request");
            let status = response.status().as_u16();
            let receipt = response
                .headers()
                .get("x-reasonbraid-authorization")
                .map(|v| v.to_str().unwrap().to_string());
            let body: Value = response.json().await.expect("import json");
            (status, receipt, body)
        }
    };

    // The ALLOWLIST rung, with no agreement in place. 403 with body code
    // `unauthorized`, and the record must say the same word.
    let (status, receipt, body) = import(
        json!({ "tenant_id": tenant_b, "card": card.clone(), "digest": digest }),
        b_admin.clone(),
    )
    .await;
    assert_eq!(status, 403, "the no-agreement import refuses: {body}");
    assert_eq!(body["code"], json!("unauthorized"), "{body}");
    let (operation, outcome) = recorded(receipt.expect("the refusal carries its receipt")).await;
    assert_eq!(
        operation["kind"],
        json!("profile_card_import"),
        "{operation}"
    );
    assert_eq!(
        operation["card_digest"],
        json!(exported["digest"].as_str().unwrap()),
        "the target is the card's own digest: {operation}"
    );
    assert_eq!(outcome["kind"], json!("refused"), "{outcome}");
    assert_eq!(
        outcome["code"],
        json!("unauthorized"),
        "the record says exactly what the response said: {outcome}"
    );

    // The DIGEST rung, with a tampered card. The presented digest no longer
    // re-derives, and the record's target is the digest of the card that was
    // actually submitted — which still exists, and is not the presented string.
    let mut tampered = card.clone();
    tampered["profile"]["purpose"] = json!("tampered after the export");
    let (status, receipt, body) = import(
        json!({ "tenant_id": tenant_b, "card": tampered.clone(), "digest": exported["digest"].as_str().unwrap() }),
        b_admin.clone(),
    )
    .await;
    assert_eq!(status, 400, "the tampered card refuses: {body}");
    assert_eq!(body["code"], json!("invalid_command"), "{body}");
    let (operation, outcome) = recorded(receipt.expect("the refusal carries its receipt")).await;
    assert_eq!(outcome["kind"], json!("refused"), "{outcome}");
    assert_eq!(outcome["code"], json!("invalid_command"), "{outcome}");
    assert_ne!(
        operation["card_digest"],
        json!(exported["digest"].as_str().unwrap()),
        "the target names the card submitted, not the digest claimed for it: {operation}"
    );
}

/// 🔴 A card whose display label is already taken in the importing tenant is a
/// typed refusal that RECORDS itself, not a raised constraint and a `500`
/// (`SIGNOFF-REPAIR.3.3.4.11.5`). A repeated import of the same card is the
/// commonest way to reach it; a card from a different origin sharing a label is
/// another.
///
/// ⛔ This asserts the label is taken and that the refusal is recorded. It does
/// NOT assert what a repeated import ought to do — the ordinary enrollment route
/// answers a name collision with a replay, and whether a card import should too
/// is card replay semantics, owned by `SIGNOFF-REPAIR.5.3`.
#[tokio::test]
async fn a_taken_display_label_refuses_in_the_record_rather_than_raising() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human_a) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "label-a" }),
    )
    .await;
    assert_eq!(status, 200, "A enrolls: {human_a}");
    let a_admin = human_a["principal_id"].as_str().unwrap().to_string();
    let tenant_a = human_a["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "label-role", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &sample_profile(),
    )
    .await;
    assert_eq!(status, 200, "the profile writes");

    let (status, human_b) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "label-b" }),
    )
    .await;
    assert_eq!(status, 200, "B enrolls: {human_b}");
    let b_admin = human_b["principal_id"].as_str().unwrap().to_string();
    let tenant_b = human_b["tenant_id"].as_str().unwrap().to_string();
    let (status, exported) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/card"),
        &role_id,
    )
    .await;
    assert_eq!(status, 200, "the card exports");
    for (admin, tenant, remote) in [
        (&a_admin, &tenant_a, &tenant_b),
        (&b_admin, &tenant_b, &tenant_a),
    ] {
        let (status, _) = post(
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
        assert_eq!(status, 200, "the propose");
        let (status, _) = post(
            &client,
            &base,
            "/v1/federation-agreements/accept",
            admin,
            &json!({ "tenant_id": tenant, "remote_tenant_id": remote }),
        )
        .await;
        assert_eq!(status, 200, "the accept");
    }

    let body = json!({
        "tenant_id": tenant_b,
        "card": exported["card"].clone(),
        "digest": exported["digest"].as_str().unwrap(),
    });
    let (status, first) = post(&client, &base, "/v1/profiles/cards/import", &b_admin, &body).await;
    assert_eq!(status, 200, "the first import lands: {first}");

    let roles_before: i64 =
        sqlx::query_scalar("SELECT count(*) FROM agent_roles WHERE tenant_id = $1")
            .bind(&tenant_b)
            .fetch_one(&pool)
            .await
            .expect("count roles");

    let response = client
        .post(format!("{base}/v1/profiles/cards/import"))
        .header(PRINCIPAL_HEADER, &b_admin)
        .json(&body)
        .send()
        .await
        .expect("second import");
    let status = response.status().as_u16();
    let receipt = response
        .headers()
        .get("x-reasonbraid-authorization")
        .map(|v| v.to_str().unwrap().to_string());
    let refused: Value = response.json().await.expect("second import json");

    assert_eq!(
        status, 400,
        "the taken label is a typed refusal, not a storage failure: {refused}"
    );
    assert_eq!(refused["code"], json!("invalid_command"), "{refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("already holds an identity labelled"),
        "{refused}"
    );

    // The refusal is RECORDED — the thing a raised constraint made impossible.
    let receipt = receipt.expect("the refusal carries its receipt");
    let (operation, outcome): (Value, Value) = sqlx::query_as(
        "SELECT operation, outcome FROM administrative_effects WHERE record_id = $1",
    )
    .bind(&receipt)
    .fetch_one(&pool)
    .await
    .expect("the refusal's effect record exists");
    assert_eq!(
        operation["kind"],
        json!("profile_card_import"),
        "{operation}"
    );
    assert_eq!(outcome["kind"], json!("refused"), "{outcome}");
    assert_eq!(outcome["code"], json!("invalid_command"), "{outcome}");
    assert!(
        outcome["detail"]
            .as_str()
            .unwrap()
            .contains("already holds an identity labelled"),
        "the record says what the response said: {outcome}"
    );

    // And it wrote nothing: the refused import added no second role.
    let roles_after: i64 =
        sqlx::query_scalar("SELECT count(*) FROM agent_roles WHERE tenant_id = $1")
            .bind(&tenant_b)
            .fetch_one(&pool)
            .await
            .expect("count roles");
    assert_eq!(
        roles_after, roles_before,
        "the refused import created no second role"
    );
}

/// The export gate the chapter documents, which nothing covered until now: only
/// the `full` class — the role itself or its tenant administrator — mints the
/// portable card, because the card carries the UNFILTERED profile. A same-tenant
/// sibling and a stranger both refuse, and the refusal is the same for both.
#[tokio::test]
async fn only_the_full_class_exports_the_portable_card() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, owner) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "export-owner" }),
    )
    .await;
    assert_eq!(status, 200, "the owner enrolls: {owner}");
    let owner_id = owner["principal_id"].as_str().unwrap().to_string();
    let tenant = owner["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "export-role", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let (status, sibling) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "export-sibling", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "the sibling enrolls: {sibling}");
    let sibling_id = sibling["principal_id"].as_str().unwrap().to_string();
    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "export-stranger" }),
    )
    .await;
    assert_eq!(status, 200, "the stranger enrolls: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &sample_profile(),
    )
    .await;
    assert_eq!(status, 200, "the profile writes");

    let card = format!("/v1/profiles/{role_id}/card");
    // The two `full` readers export.
    for (who, principal) in [
        ("the role itself", &role_id),
        ("its tenant admin", &owner_id),
    ] {
        let (status, body) = get(&client, &base, &card, principal).await;
        assert_eq!(status, 200, "{who} exports: {body}");
        assert!(body["digest"].as_str().unwrap().starts_with("sha256:"));
    }
    // A same-tenant sibling reads the TENANT view of the profile but exports
    // nothing — the card is not a filtered form.
    let (status, refused) = get(&client, &base, &card, &sibling_id).await;
    assert_eq!(status, 403, "a tenant sibling cannot export: {refused}");
    let (status, profile) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &sibling_id,
    )
    .await;
    assert_eq!(
        status, 200,
        "though it reads the filtered profile: {profile}"
    );
    assert_eq!(profile["visibility"], json!("tenant"), "{profile}");
    // A stranger in another tenant refuses identically.
    let (status, refused_stranger) = get(&client, &base, &card, &stranger_id).await;
    assert_eq!(status, 403, "a stranger cannot export: {refused_stranger}");
    assert_eq!(
        refused["message"], refused_stranger["message"],
        "the two refusals are the same answer"
    );
}

/// 🔴 THE `.3.3.4.12.1` control: an import must be ordered against the ORIGIN
/// tenant's own authority operations, not only the importing tenant's.
///
/// `.11.3` predeclared this limit and `.12` closed half of it: with the
/// direction verbs taking their own tenant's EXCLUSIVE guard, an import was
/// fenced against the IMPORTING tenant revoking its direction and still not
/// against the ORIGIN tenant revoking its side. The import now declares both
/// keys in one sorted set, so a holder of the ORIGIN tenant's guard fences it.
///
/// ⚠️ The holder is EXCLUSIVE on the ORIGIN key, which is precisely what a
/// direction revocation takes there. Against the superseded shape this control
/// does not fail slowly — it does not block at all, because that transaction
/// never declared the origin's key.
#[tokio::test]
async fn an_import_is_fenced_by_the_origin_tenants_own_guard() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, a) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "fence-a" }),
    )
    .await;
    assert_eq!(status, 200, "A enrolls: {a}");
    let a_admin = a["principal_id"].as_str().unwrap().to_string();
    let tenant_a = a["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "fence-role", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &sample_profile(),
    )
    .await;
    assert_eq!(status, 200, "the profile writes");
    let (status, b) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "fence-b" }),
    )
    .await;
    assert_eq!(status, 200, "B enrolls: {b}");
    let b_admin = b["principal_id"].as_str().unwrap().to_string();
    let tenant_b = b["tenant_id"].as_str().unwrap().to_string();
    let (status, exported) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/card"),
        &role_id,
    )
    .await;
    assert_eq!(status, 200, "the card exports");
    for (admin, tenant, remote) in [
        (&a_admin, &tenant_a, &tenant_b),
        (&b_admin, &tenant_b, &tenant_a),
    ] {
        let (status, _) = post(
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
        assert_eq!(status, 200, "the propose");
        let (status, _) = post(
            &client,
            &base,
            "/v1/federation-agreements/accept",
            admin,
            &json!({ "tenant_id": tenant, "remote_tenant_id": remote }),
        )
        .await;
        assert_eq!(status, 200, "the accept");
    }

    // Hold the ORIGIN tenant's key exclusively — the key its own direction
    // revocation takes. The importing tenant's key is untouched.
    let origin: reasonbraid_core::TenantId = tenant_a.parse().expect("a tenant id");
    let holder = hold(&pool, origin, tenant_transaction::GuardMode::Exclusive).await;

    let (base2, client2, admin2, body2) = (
        base.clone(),
        client.clone(),
        b_admin.clone(),
        json!({
            "tenant_id": tenant_b,
            "card": exported["card"].clone(),
            "digest": exported["digest"].as_str().unwrap(),
        }),
    );
    let request = tokio::spawn(async move {
        post(
            &client2,
            &base2,
            "/v1/profiles/cards/import",
            &admin2,
            &body2,
        )
        .await
    });
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    assert!(
        !request.is_finished(),
        "the import waits on the ORIGIN tenant's guard"
    );
    let roles_mid: i64 =
        sqlx::query_scalar("SELECT count(*) FROM agent_roles WHERE tenant_id = $1")
            .bind(&tenant_b)
            .fetch_one(&pool)
            .await
            .expect("count");
    holder.release().await;

    let (status, body) = tokio::time::timeout(std::time::Duration::from_secs(10), request)
        .await
        .expect("the import completes once the origin's guard is released")
        .expect("the request task joins");
    assert_eq!(roles_mid, 0, "nothing was written while it waited: {body}");
    assert_eq!(status, 200, "and it then succeeds normally: {body}");
}

/// Hold one tenant's authority guard until released.
struct Holder {
    release: tokio::sync::oneshot::Sender<()>,
    job: tokio::task::JoinHandle<()>,
}

async fn hold(
    pool: &PgPool,
    tenant: reasonbraid_core::TenantId,
    mode: tenant_transaction::GuardMode,
) -> Holder {
    let pool = pool.clone();
    let (entered_tx, entered) = tokio::sync::oneshot::channel();
    let (release, release_rx) = tokio::sync::oneshot::channel();
    let job = tokio::spawn(async move {
        tenant_transaction::transact(&pool, &[(tenant, mode)], move |_| {
            Box::pin(async move {
                let _ = entered_tx.send(());
                let _ = release_rx.await;
                Ok::<(), tenant_transaction::GuardError>(())
            })
        })
        .await
        .expect("the holder's guarded transaction completes");
    });
    tokio::time::timeout(std::time::Duration::from_secs(5), entered)
        .await
        .expect("the holder acquires its guard")
        .expect("the holder reports entry");
    Holder { release, job }
}

impl Holder {
    async fn release(self) {
        let _ = self.release.send(());
        tokio::time::timeout(std::time::Duration::from_secs(5), self.job)
            .await
            .expect("the holder finishes")
            .expect("the holder's task joins");
    }
}
