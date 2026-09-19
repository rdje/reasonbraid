//! Migration upgrade test (`PHASE-2.4.2`; ROADMAP §17.6): the
//! upgrade-an-EXISTING-database path — every other suite migrates a FRESH
//! database, this one applies all but the LAST migration, seeds the
//! pre-upgrade data in the PRE-UPGRADE SCHEMA'S OWN SHAPE (`.1.3.2`: the new
//! app's enroll writes the NEW schema's quota tables, so the seed can no
//! longer ride the API — the pre-upgrade database holds the OLD app's
//! writes), applies the rest, and asserts the data + the behavior survive.
//! Skips offline (no DATABASE_URL).

#[path = "support/mod.rs"]
mod pg_test_support;

use std::sync::OnceLock;

use serde_json::{json, Value};
use sqlx::migrate::Migrator;

static UPGRADE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

/// Recreate `public` the way this suite needs — and restore the two properties
/// a MANUALLY created schema silently loses.
///
/// A fresh database ships `public` owned by `pg_database_owner` with
/// `GRANT USAGE ... TO PUBLIC`; `CREATE SCHEMA public` gives it to the creating
/// role with no PUBLIC grant at all. Every later suite in this shared cluster
/// then runs in a database where a non-owner role cannot even resolve a
/// qualified name, which is how this fixture silently broke `site_authority`
/// twenty suites later. A fixture that mutates database-wide privilege state
/// owns restoring it, exactly as the cleanup plans own their rows.
async fn recreate_public_schema(pool: &sqlx::PgPool) {
    sqlx::raw_sql(
        "DROP SCHEMA public CASCADE; \
         CREATE SCHEMA public; \
         ALTER SCHEMA public OWNER TO pg_database_owner; \
         GRANT USAGE ON SCHEMA public TO PUBLIC",
    )
    .execute(pool)
    .await
    .unwrap();
}

/// The coordination namespace must cover identity and standalone authority
/// stores without inventing identity rows or rewriting historical evidence.
#[tokio::test]
async fn tenant_guard_upgrade_preserves_namespaces_and_backfills_exactly_once() {
    let _g = guard().await;
    let Some(pool) = pg_test_support::pool().await else {
        return;
    };
    let migrator =
        Migrator::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"))
            .await
            .unwrap();
    let through = |version| Migrator {
        migrations: std::borrow::Cow::Owned(
            migrator
                .migrations
                .iter()
                .filter(|m| m.version <= version)
                .cloned()
                .collect(),
        ),
        ignore_missing: false,
        no_tx: false,
        locking: true,
    };
    recreate_public_schema(&pool).await;
    through(55).run(&pool).await.unwrap();
    let identity = "ten_00000000-0000-7000-8000-000000000151";
    let boundary = "ten_00000000-0000-7000-8000-000000000152";
    let grant = "ten_00000000-0000-7000-8000-000000000153";
    let evidence = "ten_00000000-0000-7000-8000-000000000154";
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind(identity)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO enrollment_boundaries \
        (boundary_id, tenant_id, parent_or_root_authority, target_owner, permitted_actions, \
         permitted_domains, risk_ceiling, delegable, max_delegation_depth, valid_from, \
         expires_at, charter_digest, policy_version, status) \
        VALUES ('bnd_guard_upgrade', $1, 'legacy-root', 'legacy-owner', '[\"tenant_admin\"]', \
                '[]', 'low', false, 0, '2026-09-01T00:00:00Z', '2026-10-01T00:00:00Z', 'legacy-charter', 'legacy', 'revoked')")
        .bind(boundary).execute(&pool).await.unwrap();
    // This legacy grant's tenant differs from its parent's tenant. The current
    // evaluator refuses that relationship; a coordination migration must still
    // preserve both namespaces and all fields rather than repair or omit evidence.
    sqlx::query("INSERT INTO authority_grants \
        (grant_id, boundary_id, tenant_id, issuer, subject_kind, subject_id, actions, selector, \
         risk_ceiling, delegable, valid_from, expires_at, status) \
        VALUES ('grt_guard_upgrade', 'bnd_guard_upgrade', $1, \
                'hpr_00000000-0000-7000-8000-000000000151', 'role', \
                'rol_00000000-0000-7000-8000-000000000151', '[\"tenant_admin\"]', \
                '{\"kind\":\"tenant_wide\"}', 'low', false, '2026-09-01T00:00:00Z', '2026-10-01T00:00:00Z', 'revoked')")
        .bind(grant).execute(&pool).await.unwrap();
    for (id, tenant) in [
        ("authz_00000000-0000-7000-8000-000000000151", identity),
        ("authz_00000000-0000-7000-8000-000000000154", evidence),
    ] {
        sqlx::query(
            "INSERT INTO authorization_records \
            (record_id, tenant_id, actor, action, target_kind, target_tenant, decision, reason, \
             policy_digest, policy_version, decided_at) \
            VALUES ($1, $2, 'agt_00000000-0000-7000-8000-000000000151', 'tenant_admin', \
                    'tenant', $2, 'denied', 'historical refusal', 'legacy-digest', 'legacy', \
                    '2026-09-09T00:00:00Z')",
        )
        .bind(id)
        .bind(tenant)
        .execute(&pool)
        .await
        .unwrap();
    }
    // NOT a cleanup plan: this snapshots the pre-upgrade schema at version 55,
    // so it must name only tables that exist THERE. `administrative_effects`
    // arrives with 0058 and deliberately has no row to preserve.
    let tables = [
        "tenants",
        "enrollment_boundaries",
        "authority_grants",
        "authorization_records",
    ];
    let mut before = Vec::new();
    for table in tables {
        let rows: Vec<Value> = sqlx::query_scalar(&format!(
            "SELECT to_jsonb(r) FROM {table} r ORDER BY to_jsonb(r)::text COLLATE \"C\""
        ))
        .fetch_all(&pool)
        .await
        .unwrap();
        before.push(rows);
    }
    through(56).run(&pool).await.unwrap();
    let present: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public.tenant_authority_guards')::text")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        present.is_some(),
        "the tenant authority guard migration is present"
    );
    let anchors: Vec<String> = sqlx::query_scalar(
        "SELECT tenant_id FROM tenant_authority_guards ORDER BY tenant_id COLLATE \"C\"",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(anchors, [identity, boundary, grant, evidence]);
    for (table, expected) in tables.into_iter().zip(before) {
        let actual: Vec<Value> = sqlx::query_scalar(&format!(
            "SELECT to_jsonb(r) FROM {table} r ORDER BY to_jsonb(r)::text COLLATE \"C\""
        ))
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            actual, expected,
            "all {table} rows and fields remain unchanged"
        );
    }
    through(56).run(&pool).await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM tenant_authority_guards")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 4, "migration replay does not duplicate anchors");
}

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    UPGRADE_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
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

/// THE §17.6 upgrade leg: an existing database upgrades — the seeded rows and
/// the API behavior survive the last migration.
#[tokio::test]
async fn an_existing_database_upgrades_and_its_data_survives() {
    let _g = guard().await;
    let Some(pool) = pg_test_support::pool().await else {
        return;
    };
    let migrator =
        Migrator::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"))
            .await
            .expect("migrator");

    // 1. All but the LAST migration (the pre-upgrade database).
    let all_but_last: Vec<_> = migrator
        .migrations
        .iter()
        .filter(|m| m.version != migrator.migrations.last().unwrap().version)
        .cloned()
        .collect();
    let prefix = Migrator {
        migrations: std::borrow::Cow::Owned(all_but_last),
        ignore_missing: false,
        no_tx: false,
        locking: true,
    };
    // The upgrade path starts from a CLEAN pre-upgrade database.
    recreate_public_schema(&pool).await;
    prefix
        .run(&pool)
        .await
        .expect("apply all but the last migration");

    // 2. The pre-upgrade data, in the PRE-UPGRADE schema's shape (the rows
    //    the OLD app wrote — the tenant + the dev boundary; `.1.3.2`: the
    //    NEW app's enroll depends on the NEW schema's quota tables, so the
    //    seed rides raw SQL, not the API).
    let tenant = "ten_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind(tenant)
        .execute(&pool)
        .await
        .expect("seed the tenant");
    sqlx::query(
        "INSERT INTO enrollment_boundaries \
         (boundary_id, tenant_id, parent_or_root_authority, target_owner, permitted_actions, \
          permitted_domains, risk_ceiling, spend_ceiling, delegable, max_delegation_depth, \
          valid_from, expires_at, charter_digest, policy_version, status) \
         VALUES ($1, $2, 'dev-root', 'dev-operator', $3, $4, 'low', $5, false, 0, \
                 now(), now() + interval '365 days', 'dev-charter-000', 'dev-authz-1', 'active')",
    )
    .bind(format!("bnd_{tenant}"))
    .bind(tenant)
    .bind(json!([
        "thread_create",
        "thread_invite",
        "thread_contribute",
        "thread_inspect",
        "thread_close",
        "thread_cancel",
        "thread_invitation_respond",
        "thread_advance_round",
        "tenant_admin",
    ]))
    .bind(json!(["deliberation"]))
    .bind(json!({ "amount": 1000.0 }))
    .execute(&pool)
    .await
    .expect("seed the boundary");
    let seeded: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE tenant_id = $1")
        .bind(tenant)
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(seeded, 1, "the pre-upgrade data seeded the tenant");

    // 3. THE upgrade: the remaining migration(s) over the existing data.
    migrator
        .run(&pool)
        .await
        .expect("apply the remaining migrations");

    // 4. The data survived — and the upgrade backfilled the tenant's quota
    //    (migration 0047: a tenant exists WITH its bounds).
    let tenants: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE tenant_id = $1")
        .bind(tenant)
        .fetch_one(&pool)
        .await
        .expect("count after upgrade");
    assert_eq!(tenants, 1, "the tenant row survived the upgrade");
    let boundaries: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM enrollment_boundaries WHERE tenant_id = $1")
            .bind(tenant)
            .fetch_one(&pool)
            .await
            .expect("boundaries after upgrade");
    assert_eq!(boundaries, 1, "the boundary row survived the upgrade");
    // The migration-boundary note: the "all but last" prefix MOVES as the
    // migrations land — a backfill that ran as an earlier last-migration
    // (the 0047 quota backfill) is no longer the boundary's concern; the
    // survival assertions above are the boundary-independent truth.

    // 5. The API behavior survives (the post-upgrade surface works over the
    //    upgraded database): the role enroll path still answers.
    let server = {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().unwrap();
        let pool = pool.clone();
        tokio::spawn(async move {
            let router = reasonbraid_server::api_router(pool);
            axum::serve(listener, router).await.expect("serve");
        });
        format!("http://{addr}")
    };
    let client = reqwest::Client::new();
    let (status, role) = enroll(
        &client,
        &server,
        json!({ "kind": "role", "name": "upgrade-role", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls after the upgrade: {role}");
    assert!(role["principal_id"].as_str().unwrap().starts_with("rol_"));
}

/// Pin the provenance transition itself: future migrations must not silently
/// move this test's historical schema or turn a fresh-schema pass into an upgrade.
#[tokio::test]
async fn evaluation_provenance_upgrade_preserves_legacy_records_without_inference() {
    let _g = guard().await;
    let Some(pool) = pg_test_support::pool().await else {
        return;
    };
    let migrator =
        Migrator::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"))
            .await
            .unwrap();
    recreate_public_schema(&pool).await;
    let through = |version| Migrator {
        migrations: std::borrow::Cow::Owned(
            migrator
                .migrations
                .iter()
                .filter(|m| m.version <= version)
                .cloned()
                .collect(),
        ),
        ignore_missing: false,
        no_tx: false,
        locking: true,
    };
    assert!(migrator.migrations.iter().any(|m| m.version == 55));
    through(54).run(&pool).await.unwrap();
    let tenant = "ten_00000000-0000-7000-8000-000000000141";
    let actor = "agt_00000000-0000-7000-8000-000000000141";
    let old_insert = "INSERT INTO authorization_records \
        (record_id, tenant_id, actor, action, target_kind, target_tenant, decision, reason, \
         policy_digest, policy_version, decided_at) \
        VALUES ($1, $2, $3, 'tenant_admin', 'tenant', $2, $4, $5, \
                'historical-digest', 'historical-policy', '2026-09-09T00:00:00Z')";
    let allowed = "authz_00000000-0000-7000-8000-000000000141";
    let denied = "authz_00000000-0000-7000-8000-000000000142";
    for (id, decision, reason) in [
        (allowed, "allowed", None),
        (denied, "denied", Some("historical refusal")),
    ] {
        sqlx::query(old_insert)
            .bind(id)
            .bind(tenant)
            .bind(actor)
            .bind(decision)
            .bind(reason)
            .execute(&pool)
            .await
            .unwrap();
    }
    let before: Vec<Value> =
        sqlx::query_scalar("SELECT to_jsonb(r) FROM authorization_records r ORDER BY record_id")
            .fetch_all(&pool)
            .await
            .unwrap();
    through(55).run(&pool).await.unwrap();
    let after: Vec<Value> =
        sqlx::query_scalar("SELECT to_jsonb(r) FROM authorization_records r ORDER BY record_id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(before.len(), 2);
    assert_eq!(
        after.len(),
        before.len(),
        "upgrade retains the exact row count"
    );
    for (old, mut upgraded) in before.into_iter().zip(after) {
        assert_eq!(
            upgraded.as_object_mut().unwrap().remove("evaluation"),
            Some(json!({"kind":"legacy_unspecified"}))
        );
        assert_eq!(upgraded, old, "every historical field survives unchanged");
    }
    for id in [allowed, denied] {
        let record = reasonbraid_server::load_authorization_record(&pool, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            record.evaluation,
            reasonbraid_core::AuthorizationEvaluation::LegacyUnspecified {}
        );
    }
    let old_writer = "authz_00000000-0000-7000-8000-000000000143";
    sqlx::query(old_insert)
        .bind(old_writer)
        .bind(tenant)
        .bind(actor)
        .bind("denied")
        .bind("old writer after upgrade")
        .execute(&pool)
        .await
        .unwrap();
    let record = reasonbraid_server::load_authorization_record(&pool, old_writer)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        record.evaluation,
        reasonbraid_core::AuthorizationEvaluation::LegacyUnspecified {}
    );
    for invalid in [json!(null), json!([]), json!({}), json!({"kind":"unknown"})] {
        let error =
            sqlx::query("UPDATE authorization_records SET evaluation = $1 WHERE record_id = $2")
                .bind(invalid)
                .bind(old_writer)
                .execute(&pool)
                .await
                .unwrap_err();
        assert_eq!(
            error.as_database_error().and_then(|e| e.code()).as_deref(),
            Some("23514")
        );
    }
    // The database constrains the discriminator; typed readback also constrains
    // the complete shape. Unknown fields may not be silently erased.
    sqlx::query("UPDATE authorization_records SET evaluation = $1 WHERE record_id = $2")
        .bind(json!({"kind":"boundary_checked","fabricated":true}))
        .bind(old_writer)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        reasonbraid_server::load_authorization_record(&pool, old_writer).await,
        Err(sqlx::Error::Protocol(_))
    ));
    sqlx::query("UPDATE authorization_records SET evaluation = DEFAULT WHERE record_id = $1")
        .bind(old_writer)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        reasonbraid_server::load_authorization_record(&pool, old_writer)
            .await
            .unwrap()
            .unwrap(),
        record
    );
}

/// Request recovery is additive: old tenants/enrollments retain their exact
/// identity, and migration never guesses a request identity from a name.
#[tokio::test]
async fn bootstrap_request_upgrade_preserves_legacy_rows_and_enforces_binding_constraints() {
    let _g = guard().await;
    let Some(pool) = pg_test_support::pool().await else {
        return;
    };
    let migrator =
        Migrator::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"))
            .await
            .unwrap();
    let prefix = Migrator {
        migrations: std::borrow::Cow::Owned(
            migrator
                .migrations
                .iter()
                .filter(|m| m.version <= 56)
                .cloned()
                .collect(),
        ),
        ignore_missing: false,
        no_tx: false,
        locking: true,
    };
    recreate_public_schema(&pool).await;
    prefix.run(&pool).await.unwrap();
    let tenant = reasonbraid_core::TenantId::new().to_string();
    sqlx::query("INSERT INTO tenants (tenant_id, name) VALUES ($1, 'legacy bootstrap tenant')")
        .bind(&tenant)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO enrollments (principal_id, tenant_id, kind, name) VALUES ('legacy-human', $1, 'human', 'legacy bootstrap name')")
        .bind(&tenant).execute(&pool).await.unwrap();
    let before: Value = sqlx::query_scalar("SELECT jsonb_build_array((SELECT jsonb_agg(to_jsonb(t)) FROM tenants t), (SELECT jsonb_agg(to_jsonb(e)) FROM enrollments e))")
        .fetch_one(&pool).await.unwrap();
    migrator.run(&pool).await.unwrap();
    let after: Value = sqlx::query_scalar("SELECT jsonb_build_array((SELECT jsonb_agg(to_jsonb(t)) FROM tenants t), (SELECT jsonb_agg(to_jsonb(e)) FROM enrollments e))")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(before, after);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM tenant_bootstrap_requests")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "no invented legacy request receipt");
    let key = reasonbraid_core::RequestId::new().to_string();
    for (request, target, version, outcome, code) in [
        (
            "req_bad".to_string(),
            tenant.clone(),
            1_i16,
            json!({}),
            "23514",
        ),
        (
            "req_00000000-0000-7000-8000-00000000000A".into(),
            tenant.clone(),
            1,
            json!({}),
            "23514",
        ),
        (
            key.clone(),
            reasonbraid_core::TenantId::new().to_string(),
            1,
            json!({}),
            "23503",
        ),
        (key.clone(), tenant.clone(), 0, json!({}), "23514"),
        (key.clone(), tenant.clone(), 1, json!([]), "23514"),
        (key.clone(), tenant.clone(), 1, Value::Null, "23514"),
    ] {
        let error = sqlx::query("INSERT INTO tenant_bootstrap_requests (request_id, tenant_id, request_name, outcome_version, outcome) VALUES ($1, $2, 'legacy bootstrap name', $3, $4)")
            .bind(request).bind(target).bind(version).bind(outcome).execute(&pool).await.unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some(code)
        );
    }
    sqlx::query("INSERT INTO tenant_bootstrap_requests (request_id, tenant_id, request_name, outcome_version, outcome) VALUES ($1, $2, 'schema-only fixture', 1, '{}')")
        .bind(&key).bind(&tenant).execute(&pool).await.unwrap();
    for request in [key.clone(), reasonbraid_core::RequestId::new().to_string()] {
        let error = sqlx::query("INSERT INTO tenant_bootstrap_requests (request_id, tenant_id, request_name, outcome_version, outcome) VALUES ($1, $2, 'schema-only fixture', 1, '{}')")
            .bind(request).bind(&tenant).execute(&pool).await.unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("23505")
        );
    }
    let receipt: Value = sqlx::query_scalar(
        "SELECT to_jsonb(r) FROM tenant_bootstrap_requests r WHERE request_id = $1",
    )
    .bind(&key)
    .fetch_one(&pool)
    .await
    .unwrap();
    migrator.run(&pool).await.unwrap();
    let replay: Value = sqlx::query_scalar(
        "SELECT to_jsonb(r) FROM tenant_bootstrap_requests r WHERE request_id = $1",
    )
    .bind(&key)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(receipt, replay);
    let error = sqlx::query("DELETE FROM tenants WHERE tenant_id = $1")
        .bind(&tenant)
        .execute(&pool)
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("23503")
    );
}

/// `SIGNOFF-REPAIR.6.1.5.2` — `migrations/0073` derives a stored lifecycle row's
/// tenant by LINEAGE, and leaves NULL everything lineage cannot reach.
///
/// The leaf's acceptance asks for the backfill's coverage as a MEASURED count
/// rather than an assertion, so this control seeds three chains whose answers
/// are known in advance and publishes what the migration attributed:
///
///   * attributable — a proposal on a thread that exists under exactly one
///     tenant, and the decision, approval, publication, drift, correction,
///     outcome and review descended from it;
///   * orphaned — the same chain on a `thread_id` present in no aggregate, so
///     nothing downstream can be attributed either;
///   * ambiguous — a `thread_id` present under TWO tenants. ⛔ `aggregate_state`
///     is keyed `(tenant_id, aggregate_id)`, so an aggregate id is unique per
///     tenant and not globally; the migration's `HAVING count(*) = 1` is what
///     makes this a derivation rather than a coin toss, and this arm is the only
///     thing that proves the clause is load-bearing.
///
/// ⛔ `policy_projections` is seeded and attributed by NOTHING, deliberately: a
/// projection has no ancestor, so its tenant is its author's, and an authorship
/// the old schema never recorded cannot be recovered from it.
#[tokio::test]
async fn policy_lifecycle_tenant_upgrade_derives_by_lineage_and_leaves_the_rest_null() {
    let _g = guard().await;
    let Some(pool) = pg_test_support::pool().await else {
        return;
    };
    let migrator =
        Migrator::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"))
            .await
            .unwrap();
    recreate_public_schema(&pool).await;
    let through = |version| Migrator {
        migrations: std::borrow::Cow::Owned(
            migrator
                .migrations
                .iter()
                .filter(|m| m.version <= version)
                .cloned()
                .collect(),
        ),
        ignore_missing: false,
        no_tx: false,
        locking: true,
    };
    assert!(migrator.migrations.iter().any(|m| m.version == 73));
    through(72).run(&pool).await.unwrap();

    let owner = "ten_00000000-0000-7000-8000-000000000173";
    let other = "ten_00000000-0000-7000-8000-000000000174";
    for tenant in [owner, other] {
        sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
            .bind(tenant)
            .execute(&pool)
            .await
            .unwrap();
    }
    // `thr-one` exists under exactly one tenant; `thr-both` under two.
    for (tenant, thread) in [(owner, "thr-one"), (owner, "thr-both"), (other, "thr-both")] {
        sqlx::query(
            "INSERT INTO aggregate_state \
             (tenant_id, aggregate_id, aggregate_type, aggregate_version, state) \
             VALUES ($1, $2, 'thread', 1, '{}'::jsonb)",
        )
        .bind(tenant)
        .bind(thread)
        .execute(&pool)
        .await
        .unwrap();
    }

    // `thr-gone` is in no aggregate at all — the orphaned chain's root.
    for (proposal, thread) in [
        ("prop-owned", "thr-one"),
        ("prop-orphan", "thr-gone"),
        ("prop-ambiguous", "thr-both"),
    ] {
        sqlx::query(
            "INSERT INTO policy_proposals \
             (proposal_id, policy_id, policy_version, thread_id, status) \
             VALUES ($1, 'pol', '1.0.0', $2, 'approved')",
        )
        .bind(proposal)
        .bind(thread)
        .execute(&pool)
        .await
        .unwrap();
    }
    // The owned and the orphaned chains carry one row per lifecycle table; the
    // ambiguous one stops at its proposal, so the two refusal reasons stay
    // distinguishable in the counts below.
    for (suffix, proposal) in [("owned", "prop-owned"), ("orphan", "prop-orphan")] {
        let decision = format!("dec-{suffix}");
        let approval = format!("app-{suffix}");
        let publication = format!("pub-{suffix}");
        sqlx::query(
            "INSERT INTO policy_decisions \
             (decision_id, proposal_id, rule, electorate, verdict_event_id) \
             VALUES ($1, $2, 'majority', '{\"participants\":[\"p\"]}'::jsonb, 'evt')",
        )
        .bind(&decision)
        .bind(proposal)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO policy_approvals \
             (approval_id, proposal_id, decision_id, approver, grant_id, quorum) \
             VALUES ($1, $2, $3, 'p', 'grt_p', '{\"participants\":[\"p\"]}'::jsonb)",
        )
        .bind(&approval)
        .bind(proposal)
        .bind(&decision)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO policy_publications \
             (publication_id, proposal_id, decision_id, approval_id, projection_id, state, \
              manifest_digest) VALUES ($1, $2, $3, $4, 'proj', 'effective', 'deadbeef')",
        )
        .bind(&publication)
        .bind(proposal)
        .bind(&decision)
        .bind(&approval)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO policy_drift \
             (drift_id, target_id, publication_id, category, desired_digest) \
             VALUES ($1, 'tgt', $2, 'pending_rollout', 'deadbeef')",
        )
        .bind(format!("drift-{suffix}"))
        .bind(&publication)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO policy_corrections \
             (correction_id, publication_id, operation, authority_grant, reason) \
             VALUES ($1, $2, 'retraction', 'grt_p', 'historical')",
        )
        .bind(format!("corr-{suffix}"))
        .bind(&publication)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO policy_outcomes (outcome_id, publication_id, kind, note) \
             VALUES ($1, $2, 'incident', 'historical')",
        )
        .bind(format!("out-{suffix}"))
        .bind(&publication)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO policy_reviews (review_id, publication_id, trigger, status) \
             VALUES ($1, $2, 'drift', 'due')",
        )
        .bind(format!("rev-{suffix}"))
        .bind(&publication)
        .execute(&pool)
        .await
        .unwrap();
    }
    for projection in ["proj-one", "proj-two"] {
        sqlx::query(
            "INSERT INTO policy_projections \
             (projection_id, target, digest, bytes, unrepresentable) \
             VALUES ($1, 'generic', 'deadbeef', 'bytes', '[]'::jsonb)",
        )
        .bind(projection)
        .execute(&pool)
        .await
        .unwrap();
    }

    let before: Vec<Value> =
        sqlx::query_scalar("SELECT to_jsonb(p) FROM policy_proposals p ORDER BY proposal_id")
            .fetch_all(&pool)
            .await
            .unwrap();

    through(73).run(&pool).await.unwrap();

    // ── THE COVERAGE, MEASURED ────────────────────────────────────────────────
    let mut coverage = Vec::new();
    for table in [
        "policy_proposals",
        "policy_decisions",
        "policy_approvals",
        "policy_projections",
        "policy_publications",
        "policy_drift",
        "policy_corrections",
        "policy_outcomes",
        "policy_reviews",
    ] {
        let (total, attributed): (i64, i64) =
            sqlx::query_as(&format!("SELECT count(*), count(tenant_id) FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap();
        coverage.push((table, total, attributed));
    }
    let published = coverage
        .iter()
        .map(|(t, total, got)| format!("{t} {got}/{total}"))
        .collect::<Vec<_>>()
        .join(", ");
    assert_eq!(
        coverage,
        vec![
            ("policy_proposals", 3, 1),
            ("policy_decisions", 2, 1),
            ("policy_approvals", 2, 1),
            // ⛔ 0 of 2, and it is the decision rather than a shortfall: nothing
            // in the schema can attribute a projection to an author.
            ("policy_projections", 2, 0),
            ("policy_publications", 2, 1),
            ("policy_drift", 2, 1),
            ("policy_corrections", 2, 1),
            ("policy_outcomes", 2, 1),
            ("policy_reviews", 2, 1),
        ],
        "backfill coverage: {published}"
    );

    // The one attributed row per table is the one descended from `thr-one`, and
    // the refusals are the orphan and the ambiguous thread — not an arbitrary row.
    let attributed: Vec<String> =
        sqlx::query_scalar("SELECT proposal_id FROM policy_proposals WHERE tenant_id = $1")
            .bind(owner)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(attributed, vec!["prop-owned".to_string()]);
    let unattributed: Vec<String> = sqlx::query_scalar(
        "SELECT proposal_id FROM policy_proposals WHERE tenant_id IS NULL ORDER BY proposal_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        unattributed,
        vec!["prop-ambiguous".to_string(), "prop-orphan".to_string()],
        "a thread under two tenants derives nothing, exactly as a vanished one does"
    );

    // ⛔ The upgrade INVENTS no history: every pre-existing column is untouched
    // and the row count is unchanged.
    let after: Vec<Value> =
        sqlx::query_scalar("SELECT to_jsonb(p) FROM policy_proposals p ORDER BY proposal_id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(after.len(), before.len(), "the upgrade retains every row");
    for (old, mut upgraded) in before.into_iter().zip(after) {
        upgraded.as_object_mut().unwrap().remove("tenant_id");
        assert_eq!(upgraded, old, "every historical field survives unchanged");
    }
}
