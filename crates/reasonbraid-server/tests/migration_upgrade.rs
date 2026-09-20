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

use chrono::{DateTime, Utc};
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

/// RFC 3339 → the instant the fixture writes and the assertion compares against.
fn instant(text: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text)
        .expect("fixture instant")
        .with_timezone(&Utc)
}

/// One `enrollment_boundaries` row in the PRE-`0077` shape — no `revoked_at`,
/// because the column does not exist yet at the version this seeds into.
async fn seed_legacy_boundary(pool: &sqlx::PgPool, boundary: &str, tenant: &str, status: &str) {
    sqlx::query(
        "INSERT INTO enrollment_boundaries \
         (boundary_id, tenant_id, parent_or_root_authority, target_owner, permitted_actions, \
          permitted_domains, risk_ceiling, delegable, max_delegation_depth, valid_from, \
          expires_at, charter_digest, policy_version, status) \
         VALUES ($1, $2, 'legacy-root', 'legacy-owner', '[\"tenant_admin\"]', '[]', 'low', \
                 false, 0, '2026-01-01T00:00:00Z', '2026-12-31T00:00:00Z', 'legacy-charter', \
                 'legacy', $3)",
    )
    .bind(boundary)
    .bind(tenant)
    .bind(status)
    .execute(pool)
    .await
    .unwrap();
}

/// One `authority_grants` row in the pre-`0077` shape, hung off `boundary`.
async fn seed_legacy_grant(
    pool: &sqlx::PgPool,
    grant: &str,
    boundary: &str,
    tenant: &str,
    status: &str,
) {
    sqlx::query(
        "INSERT INTO authority_grants \
         (grant_id, boundary_id, tenant_id, issuer, subject_kind, subject_id, actions, selector, \
          risk_ceiling, delegable, valid_from, expires_at, status) \
         VALUES ($1, $2, $3, 'hpr_00000000-0000-7000-8000-000000000177', 'role', \
                 'rol_00000000-0000-7000-8000-000000000177', '[\"tenant_admin\"]', \
                 '{\"kind\":\"tenant_wide\"}', 'low', false, '2026-01-01T00:00:00Z', \
                 '2026-12-31T00:00:00Z', $4)",
    )
    .bind(grant)
    .bind(boundary)
    .bind(tenant)
    .bind(status)
    .execute(pool)
    .await
    .unwrap();
}

/// One administrative effect record and the admission it is bound to.
///
/// The admission is not decoration: `administrative_effects` carries
/// `FOREIGN KEY (record_id, tenant_id) REFERENCES authorization_records`, so an
/// effect that cites no admission — or cites one under another tenant — is
/// unrepresentable, and the cross-tenant arm below has to buy its own admission
/// in the other tenant to exist at all.
async fn seed_administrative_effect(
    pool: &sqlx::PgPool,
    record: &str,
    tenant: &str,
    operation: Value,
    outcome: Value,
    effected_at: DateTime<Utc>,
) {
    sqlx::query(
        "INSERT INTO authorization_records \
         (record_id, tenant_id, actor, action, target_kind, target_tenant, decision, reason, \
          policy_digest, policy_version, decided_at) \
         VALUES ($1, $2, 'hpr_00000000-0000-7000-8000-000000000177', 'tenant_admin', 'tenant', \
                 $2, 'allowed', 'historical admission', 'legacy-digest', 'legacy', $3)",
    )
    .bind(record)
    .bind(tenant)
    .bind(effected_at)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO administrative_effects \
         (record_id, tenant_id, operation, submitted_reason, outcome, effected_at) \
         VALUES ($1, $2, $3, 'historical revocation', $4, $5)",
    )
    .bind(record)
    .bind(tenant)
    .bind(operation)
    .bind(outcome)
    .bind(effected_at)
    .execute(pool)
    .await
    .unwrap();
}

/// `SIGNOFF-REPAIR.13.4.3.1` — `migrations/0077` dates an already-revoked row
/// from the act that revoked it, and this control is what drives that backfill.
///
/// ⛔ THE GAP IT CLOSES. Every pg suite applies `0077`, so its two
/// `UPDATE … FROM administrative_effects` statements parse and the upgrade path
/// runs — but until this control existed **nothing asserted they produce an
/// instant, let alone the right one.** A join that matched NOTHING would have
/// been indistinguishable from one that matched correctly, and
/// `migration_upgrade` would have stayed green either way. That is
/// `.7.1.2.2.1`'s shape — *a green gate is not evidence a suite runs* — applied
/// to a DATA MIGRATION rather than to a test suite.
///
/// ⭐ SO EVERY LEG OF THE JOIN IS DRIVEN, not only the one that dates a row.
/// The migration header argues five conditions in prose; each gets a row whose
/// answer is known before the migration runs, and every effect record carries a
/// DISTINCT instant, so a predicate that lost a leg dates the wrong row with the
/// wrong instant rather than merely dating one row too many:
///
///   * `applied`, same tenant, matching kind and id — the one row that must be
///     dated, and dated to the effect record's own `effected_at`. All the
///     fixture instants are in the past, so an implementation that reached for
///     `now()` or for the row's own age fails this arm rather than passing it;
///   * NO effect record at all — `administrative_effects` arrived in `0058`, so
///     a revocation applied before it is undatable and must stay NULL. This is
///     the arm that separates *not recoverable* from *not revoked*, which the
///     migration header is explicit the NULL must mean;
///   * `no_op` — a repeated revocation, whose instant belongs to the first one;
///   * `refused` — no change at all, so there is nothing to date;
///   * an `applied` effect under ANOTHER tenant naming the same id — the
///     `e.tenant_id = g.tenant_id` leg, which the header calls out as the one
///     place this join must not forget;
///   * `status <> 'revoked'` — a live row is not dated even when an applied
///     revocation effect names it.
///
/// The boundary half adds an `applied` effect of the WRONG KIND
/// (`grant_revoke`) naming the boundary's id, which must not date it.
///
/// ⭐ AND WHICH LEG EACH ARM DRIVES IS MEASURED, NOT ASSERTED, because a control
/// that would still pass with a leg deleted is not driving it. Nine mutations of
/// `0077` were run against this control at `SIGNOFF-REPAIR.13.4.3.1`. **Six turn
/// it RED**: deleting both `UPDATE`s; dropping the grant statement's
/// `outcome = 'applied'`, `e.tenant_id = g.tenant_id` and `g.status = 'revoked'`
/// legs; replacing `e.effected_at` with `now()`; and dropping the boundary
/// statement's `operation->>'boundary_id'` leg.
///
/// ⚠️ **Three legs cannot be turned red by any upgrade, and that is a property
/// of the schema rather than a hole in the control.** Dropping either
/// statement's `operation->>'kind'` leg leaves it GREEN, because
/// `AdministrativeOperation` has exactly ONE variant carrying `grant_id` and
/// exactly one carrying `boundary_id` — for any record the encoder can produce,
/// the id-FIELD leg already implies the kind, so the kind predicate is defense
/// in depth against a future variant that adds one of those field names.
/// Dropping `g.revoked_at IS NULL` likewise stays GREEN: `ADD COLUMN` leaves
/// every row NULL, so that leg is an idempotence guard for a second run the
/// migration ledger does not permit, and no first upgrade can observe it.
///
/// ⚠️ WHAT IS NOT CLAIMED: an ordering for two `applied` effects naming one
/// target. The live path records the second revocation of an already-revoked
/// grant as `no_op`, so the schema does not produce that pair; asserting a
/// winner would pin behavior `UPDATE … FROM` does not define.
#[tokio::test]
async fn authority_revoked_at_upgrade_dates_only_what_an_applied_effect_dates() {
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
    assert!(
        migrator.migrations.iter().any(|m| m.version == 77),
        "the migration this control exists to drive must be present"
    );
    through(76).run(&pool).await.unwrap();

    // The column is genuinely absent before the upgrade — otherwise this suite
    // would be seeding into the post-migration schema and measuring nothing.
    for table in ["authority_grants", "enrollment_boundaries"] {
        let present: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM information_schema.columns \
             WHERE table_schema = 'public' AND table_name = $1 AND column_name = 'revoked_at'",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(present, 0, "{table}.revoked_at must arrive with 0077");
    }

    let owner = "ten_00000000-0000-7000-8000-000000000177";
    let other = "ten_00000000-0000-7000-8000-000000000178";
    for tenant in [owner, other] {
        sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
            .bind(tenant)
            .execute(&pool)
            .await
            .unwrap();
    }

    // Distinct instants, all in the past: the assertion names WHICH one landed.
    const GRANT_APPLIED_AT: &str = "2026-03-01T04:05:06Z";
    const GRANT_NO_OP_AT: &str = "2026-03-02T05:06:07Z";
    const GRANT_REFUSED_AT: &str = "2026-03-03T06:07:08Z";
    const GRANT_CROSS_TENANT_AT: &str = "2026-03-04T07:08:09Z";
    const GRANT_ACTIVE_AT: &str = "2026-03-05T08:09:10Z";
    const BOUNDARY_APPLIED_AT: &str = "2026-04-01T09:10:11Z";
    const BOUNDARY_NO_OP_AT: &str = "2026-04-02T10:11:12Z";
    const BOUNDARY_WRONG_KIND_AT: &str = "2026-04-03T11:12:13Z";

    let applied = || json!({"kind": "applied"});
    let no_op = || json!({"kind": "no_op", "detail": "already revoked"});
    let refused =
        || json!({"kind": "refused", "code": "invalid_transition", "detail": "not revocable"});

    // ── The boundaries. `bnd_host` is the live one every grant hangs off; the
    // rest are the boundary half of the measurement. Only one may be `active`:
    // `enrollment_boundaries_tenant_active_idx` is unique per tenant.
    seed_legacy_boundary(&pool, "bnd_applied_effect", owner, "revoked").await;
    seed_legacy_boundary(&pool, "bnd_host", owner, "active").await;
    seed_legacy_boundary(&pool, "bnd_no_effect", owner, "revoked").await;
    seed_legacy_boundary(&pool, "bnd_no_op_effect", owner, "revoked").await;
    seed_legacy_boundary(&pool, "bnd_wrong_kind", owner, "revoked").await;

    // ── The grants.
    for (grant, status) in [
        ("grt_active_with_effect", "active"),
        ("grt_applied_effect", "revoked"),
        ("grt_cross_tenant_effect", "revoked"),
        ("grt_no_effect", "revoked"),
        ("grt_no_op_effect", "revoked"),
        ("grt_refused_effect", "revoked"),
    ] {
        seed_legacy_grant(&pool, grant, "bnd_host", owner, status).await;
    }

    // ── The effect records, one per arm that has one. `grt_no_effect` and
    // `bnd_no_effect` deliberately get none: they are the pre-`0058` past.
    for (record, tenant, operation, outcome, at) in [
        (
            "authz_grant_applied",
            owner,
            json!({"kind": "grant_revoke", "grant_id": "grt_applied_effect"}),
            applied(),
            GRANT_APPLIED_AT,
        ),
        (
            "authz_grant_no_op",
            owner,
            json!({"kind": "grant_revoke", "grant_id": "grt_no_op_effect"}),
            no_op(),
            GRANT_NO_OP_AT,
        ),
        (
            "authz_grant_refused",
            owner,
            json!({"kind": "grant_revoke", "grant_id": "grt_refused_effect"}),
            refused(),
            GRANT_REFUSED_AT,
        ),
        // The admission — and therefore the effect — lives in `other`; the grant
        // it names lives in `owner`. Nothing but the tenant leg refuses this.
        (
            "authz_grant_cross_tenant",
            other,
            json!({"kind": "grant_revoke", "grant_id": "grt_cross_tenant_effect"}),
            applied(),
            GRANT_CROSS_TENANT_AT,
        ),
        (
            "authz_grant_active",
            owner,
            json!({"kind": "grant_revoke", "grant_id": "grt_active_with_effect"}),
            applied(),
            GRANT_ACTIVE_AT,
        ),
        (
            "authz_boundary_applied",
            owner,
            json!({"kind": "boundary_revoke", "boundary_id": "bnd_applied_effect"}),
            applied(),
            BOUNDARY_APPLIED_AT,
        ),
        (
            "authz_boundary_no_op",
            owner,
            json!({"kind": "boundary_revoke", "boundary_id": "bnd_no_op_effect"}),
            no_op(),
            BOUNDARY_NO_OP_AT,
        ),
        // An applied effect of the WRONG KIND naming the boundary's id, in the
        // field a grant revocation uses. What refuses it is measured below
        // rather than asserted: it is the `operation->>'boundary_id'` leg, not
        // the `kind` leg.
        (
            "authz_boundary_wrong_kind",
            owner,
            json!({"kind": "grant_revoke", "grant_id": "bnd_wrong_kind"}),
            applied(),
            BOUNDARY_WRONG_KIND_AT,
        ),
    ] {
        seed_administrative_effect(&pool, record, tenant, operation, outcome, instant(at)).await;
    }

    let grants_before: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(g) FROM authority_grants g ORDER BY grant_id COLLATE \"C\"",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let boundaries_before: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(b) FROM enrollment_boundaries b ORDER BY boundary_id COLLATE \"C\"",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    through(77).run(&pool).await.unwrap();

    // ── WHAT THE BACKFILL DATED ─────────────────────────────────────────────
    let grants: Vec<(String, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT grant_id, revoked_at FROM authority_grants ORDER BY grant_id COLLATE \"C\"",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        grants,
        vec![
            ("grt_active_with_effect".to_string(), None),
            (
                "grt_applied_effect".to_string(),
                Some(instant(GRANT_APPLIED_AT))
            ),
            ("grt_cross_tenant_effect".to_string(), None),
            ("grt_no_effect".to_string(), None),
            ("grt_no_op_effect".to_string(), None),
            ("grt_refused_effect".to_string(), None),
        ],
        "only an applied, same-tenant grant_revoke dates a revoked grant"
    );

    let boundaries: Vec<(String, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT boundary_id, revoked_at FROM enrollment_boundaries \
         ORDER BY boundary_id COLLATE \"C\"",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        boundaries,
        vec![
            (
                "bnd_applied_effect".to_string(),
                Some(instant(BOUNDARY_APPLIED_AT))
            ),
            ("bnd_host".to_string(), None),
            ("bnd_no_effect".to_string(), None),
            ("bnd_no_op_effect".to_string(), None),
            ("bnd_wrong_kind".to_string(), None),
        ],
        "only an applied, same-tenant boundary_revoke dates a revoked boundary"
    );

    // ⛔ The NULLs mean *this instant is not recoverable*, never *this authority
    // is not revoked* — so `status` must be exactly what it was.
    let revoked_grants: i64 =
        sqlx::query_scalar("SELECT count(*) FROM authority_grants WHERE status = 'revoked'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(revoked_grants, 5, "the upgrade answers WHEN, never WHETHER");

    // ⛔ And it INVENTS no history: every pre-existing column of every row is
    // byte-for-byte what it was, and no row appeared or vanished.
    for (table, before, after_sql, key) in [
        (
            "authority_grants",
            grants_before,
            "SELECT to_jsonb(g) FROM authority_grants g ORDER BY grant_id COLLATE \"C\"",
            "grant_id",
        ),
        (
            "enrollment_boundaries",
            boundaries_before,
            "SELECT to_jsonb(b) FROM enrollment_boundaries b ORDER BY boundary_id COLLATE \"C\"",
            "boundary_id",
        ),
    ] {
        let after: Vec<Value> = sqlx::query_scalar(after_sql)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(after.len(), before.len(), "{table} retains every row");
        for (old, mut upgraded) in before.into_iter().zip(after) {
            let id = upgraded[key].clone();
            upgraded.as_object_mut().unwrap().remove("revoked_at");
            assert_eq!(
                upgraded, old,
                "{table} {id}: every historical field survives"
            );
        }
    }
}

/// `SIGNOFF-REPAIR.11.24.1.1.1` — `migrations/0078` adds §10.6's `offered`
/// rung and DELIBERATELY backfills nothing, so this drives the absence.
///
/// ⛔ A refusal to backfill is a decision, and a decision nothing exercises is
/// indistinguishable from an oversight — `.13.4.3.1`'s finding applied to the
/// very next migration. The refusal is derived twice over: a row already at
/// `transport_received` or above OUTRANKS `offered`, so dating it would change
/// nothing observable while recording an offer at an instant (`acknowledged_at`)
/// that is provably later than the real one; and a row still at `queued` is
/// exactly where the answer would matter, with nothing in the schema to derive
/// it from.
///
/// ⭐ The second thing this drives is the view REPLACEMENT itself. `0078` must
/// `DROP` and re-`CREATE` rather than `CREATE OR REPLACE`, because the view
/// selects `i.*` and the new column lands ahead of `delivery_state` in that
/// expansion. Applying the migration over a populated pre-`0078` database is
/// what proves the replacement path works on a real upgrade rather than only on
/// a fresh schema.
#[tokio::test]
async fn node_inbox_offered_upgrade_adds_the_rung_and_invents_no_offer() {
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
    assert!(migrator.migrations.iter().any(|m| m.version == 78));
    through(77).run(&pool).await.unwrap();

    let column: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'node_inbox' \
           AND column_name = 'offered_at'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(column, 0, "node_inbox.offered_at must arrive with 0078");

    let tenant = "ten_00000000-0000-7000-8000-000000000178";
    let node = "nod_00000000-0000-7000-8000-000000000178";
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind(tenant)
        .execute(&pool)
        .await
        .unwrap();
    // A delivered row and an undelivered one, in the pre-0078 shape. The
    // delivered one is the tempting backfill: it was certainly offered, and the
    // only instant available to date it by is the WRONG one.
    for (command, acknowledged) in [("cmd_pre_delivered", true), ("cmd_pre_queued", false)] {
        sqlx::query(
            "INSERT INTO node_inbox \
             (node_id, cursor, command_id, tenant_id, thread_id, payload, acknowledged_at) \
             VALUES ($1, $2, $3, $4, 'thr_00000000-0000-7000-8000-000000000178', \
                     '{\"kind\":\"contribute\"}'::jsonb, \
                     CASE WHEN $5 THEN TIMESTAMPTZ '2026-05-05T05:05:05Z' ELSE NULL END)",
        )
        .bind(node)
        .bind(if acknowledged { 1_i64 } else { 2_i64 })
        .bind(command)
        .bind(tenant)
        .bind(acknowledged)
        .execute(&pool)
        .await
        .unwrap();
    }

    through(78).run(&pool).await.unwrap();

    let states: Vec<(String, Option<DateTime<Utc>>, String)> = sqlx::query_as(
        "SELECT command_id, offered_at, delivery_state FROM node_inbox_state \
         ORDER BY command_id COLLATE \"C\"",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        states,
        vec![
            (
                "cmd_pre_delivered".to_string(),
                None,
                "transport_received".to_string()
            ),
            ("cmd_pre_queued".to_string(), None, "queued".to_string()),
        ],
        "the upgrade adds the rung and dates no pre-existing offer"
    );

    // ⭐ And the rung is REACHABLE after the upgrade, not merely present in a
    // CASE nothing can satisfy: writing the column moves the state.
    sqlx::query("UPDATE node_inbox SET offered_at = now() WHERE command_id = 'cmd_pre_queued'")
        .execute(&pool)
        .await
        .unwrap();
    let state: String = sqlx::query_scalar(
        "SELECT delivery_state FROM node_inbox_state WHERE command_id = 'cmd_pre_queued'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        state, "offered",
        "the new rung is reachable on an upgraded row"
    );
}

/// `SIGNOFF-REPAIR.11.24.1.2` — `migrations/0079` appends `in_flight` to
/// `node_presence`, and the append is what makes `CREATE OR REPLACE VIEW`
/// legal at all.
///
/// ⛔ `migrations/0078` had to DROP and recreate `node_inbox_state`, because
/// that view selects `i.*` and a new table column lands ahead of its derived
/// one — PostgreSQL refuses with `42P16`. `node_presence` names its columns,
/// so the new one appends and a replacement is permitted. That is an assertion
/// about this specific view, and an assertion about a migration is worth what
/// drives it: this applies the migration over a POPULATED pre-`0079` database,
/// which is the only place the replacement path runs.
#[tokio::test]
async fn node_presence_in_flight_upgrade_appends_without_dropping_the_view() {
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
    assert!(migrator.migrations.iter().any(|m| m.version == 79));
    through(78).run(&pool).await.unwrap();

    let column: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'node_presence' \
           AND column_name = 'in_flight'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(column, 0, "node_presence.in_flight must arrive with 0079");

    let tenant = "ten_00000000-0000-7000-8000-000000000179";
    let node = "nod_00000000-0000-7000-8000-000000000179";
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind(tenant)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO hosts (host_id, tenant_id, name) VALUES ('hst_179', $1, 'h')")
        .bind(tenant)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO nodes (node_id, host_id, tenant_id) VALUES ($1, 'hst_179', $2)")
        .bind(node)
        .bind(tenant)
        .execute(&pool)
        .await
        .unwrap();
    // One held command and one that is merely queued, so the upgraded column
    // has to DISCRIMINATE rather than count rows.
    for (cursor, command, held) in [(1_i64, "cmd_179_held", true), (2, "cmd_179_queued", false)] {
        sqlx::query(
            "INSERT INTO node_inbox \
             (node_id, cursor, command_id, tenant_id, thread_id, payload, offered_at, acknowledged_at) \
             VALUES ($1, $2, $3, $4, 'thr_179', '{\"kind\":\"contribute\"}'::jsonb, \
                     CASE WHEN $5 THEN now() ELSE NULL END, \
                     CASE WHEN $5 THEN now() ELSE NULL END)",
        )
        .bind(node)
        .bind(cursor)
        .bind(command)
        .bind(tenant)
        .bind(held)
        .execute(&pool)
        .await
        .unwrap();
    }
    let before: Vec<Value> =
        sqlx::query_scalar("SELECT to_jsonb(p) FROM node_presence p ORDER BY node_id")
            .fetch_all(&pool)
            .await
            .unwrap();

    through(79).run(&pool).await.unwrap();

    let (in_flight, online): (i64, bool) =
        sqlx::query_as("SELECT in_flight, online FROM node_presence WHERE node_id = $1")
            .bind(node)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        in_flight, 1,
        "only the HELD command counts; the queued one is not in flight"
    );
    assert!(!online, "the pre-existing columns still derive as they did");

    // ⭐ The append is an append: every column the view had is unchanged, and
    // the new one is the only addition.
    let after: Vec<Value> =
        sqlx::query_scalar("SELECT to_jsonb(p) FROM node_presence p ORDER BY node_id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(after.len(), before.len(), "the view still has one row");
    for (old, mut upgraded) in before.into_iter().zip(after) {
        let added = upgraded.as_object_mut().unwrap().remove("in_flight");
        assert!(added.is_some(), "in_flight is the column that was added");
        assert_eq!(upgraded, old, "every pre-existing column is unchanged");
    }
}
