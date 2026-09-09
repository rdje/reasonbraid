//! Standalone authority writer integration: real guard waits, issuance-time
//! liveness, revocation order, checked status storage and transaction outcomes.
//! Every fixture uses the supervised runner's disposable database ownership.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::sync::OnceLock;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reasonbraid_core::{
    AgentRoleId, AuthorityGrant, BoundaryStatus, EnrollmentAuthorityBoundary, GrantAction,
    GrantStatus, GrantSubject, HumanPrincipalId, RiskClass, TargetSelector, TenantId,
};
use reasonbraid_server::{
    api_router, create_boundary, create_grant, AuthorityTransactionError, GrantCreateError,
    PRINCIPAL_HEADER,
};
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::{Mutex, MutexGuard};
use tokio::task::JoinHandle;
use tokio::time::timeout;

static LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct Fixture {
    pool: PgPool,
    client: reqwest::Client,
    base: String,
    server: JoinHandle<()>,
    _guard: MutexGuard<'static, ()>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn fixture() -> Option<Fixture> {
    let guard = LOCK.get_or_init(|| Mutex::new(())).lock().await;
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let router = api_router(pool.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    Some(Fixture {
        pool,
        client: reqwest::Client::new(),
        base,
        server,
        _guard: guard,
    })
}

fn parent(tenant: TenantId) -> EnrollmentAuthorityBoundary {
    EnrollmentAuthorityBoundary {
        boundary_id: format!("bnd_{tenant}"),
        tenant_id: tenant,
        parent_or_root_authority: "owned-issuance-fixture".into(),
        target_owner: "fixture".into(),
        permitted_actions: vec![GrantAction::ThreadContribute],
        permitted_domains: vec!["deliberation".into()],
        risk_ceiling: RiskClass::Low,
        spend_ceiling: None,
        delegable: false,
        max_delegation_depth: 1,
        valid_from: Utc::now() - chrono::Duration::days(1),
        expires_at: Utc::now() + chrono::Duration::days(2),
        charter_digest: "owned-issuance-charter".into(),
        policy_version: "owned-issuance-1".into(),
        status: BoundaryStatus::Active,
    }
}

async fn candidate(pool: &PgPool, tenant: TenantId, boundary: &str) -> AuthorityGrant {
    let (valid_from, expires_at): (DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
        "SELECT valid_from, expires_at FROM enrollment_boundaries WHERE boundary_id = $1",
    )
    .bind(boundary)
    .fetch_one(pool)
    .await
    .unwrap();
    let role = AgentRoleId::new();
    AuthorityGrant {
        grant_id: format!("grt_{role}"),
        boundary_id: boundary.into(),
        tenant_id: tenant,
        issuer: HumanPrincipalId::new(),
        subject: GrantSubject::Role(role),
        actions: vec![GrantAction::ThreadContribute],
        selector: TargetSelector::TenantWide,
        risk_ceiling: RiskClass::Low,
        spend_limits: None,
        delegable: false,
        valid_from,
        expires_at,
        status: GrantStatus::Active,
    }
}

struct Admin {
    tenant: TenantId,
    principal: String,
    boundary: String,
}

async fn admin(f: &Fixture) -> Admin {
    let response = f
        .client
        .post(format!("{}/v1/enrollments", f.base))
        .json(&json!({"kind":"human", "name":format!("issuer-{}", TenantId::new())}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let body: Value = response.json().await.unwrap();
    Admin {
        tenant: body["tenant_id"].as_str().unwrap().parse().unwrap(),
        principal: body["principal_id"].as_str().unwrap().into(),
        boundary: body["boundary_id"].as_str().unwrap().into(),
    }
}

async fn hold_guard(
    pool: &PgPool,
    tenant: TenantId,
    shared: bool,
) -> (Transaction<'static, Postgres>, i32) {
    sqlx::query(
        "INSERT INTO tenant_authority_guards (tenant_id) VALUES ($1) ON CONFLICT DO NOTHING",
    )
    .bind(tenant.to_string())
    .execute(pool)
    .await
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let query = if shared {
        "SELECT tenant_id FROM tenant_authority_guards WHERE tenant_id = $1 FOR SHARE"
    } else {
        "SELECT tenant_id FROM tenant_authority_guards WHERE tenant_id = $1 FOR NO KEY UPDATE"
    };
    sqlx::query(query)
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let pid = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    (tx, pid)
}

/// A finished job or timeout is not evidence of a lock. Return the actual waiter
/// PID only when PostgreSQL reports this query waiting on this holder.
async fn waiter<T>(pool: &PgPool, holder: i32, query: &str, job: &JoinHandle<T>) -> Option<i32> {
    timeout(Duration::from_secs(4), async {
        loop {
            if job.is_finished() { return None; }
            let pid: Option<i32> = sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname = current_database() AND wait_event_type = 'Lock' AND query LIKE $1 AND $2 = ANY(pg_blocking_pids(pid)) ORDER BY pid LIMIT 1")
                .bind(format!("%{query}%")).bind(holder).fetch_optional(pool).await.unwrap();
            if pid.is_some() { return pid; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.unwrap_or(None)
}

fn issue(pool: &PgPool, grant: &AuthorityGrant) -> JoinHandle<Result<(), String>> {
    let pool = pool.clone();
    let grant = grant.clone();
    tokio::spawn(async move {
        create_grant(&pool, &grant)
            .await
            .map_err(|error| format!("{error:?}"))
    })
}

fn revoke(
    f: &Fixture,
    a: &Admin,
    kind: &str,
    target: &str,
) -> JoinHandle<reqwest::Result<reqwest::Response>> {
    let client = f.client.clone();
    let url = format!("{}/v1/admin/{kind}/{target}/revoke", f.base);
    let principal = a.principal.clone();
    let body = json!({"tenant_id":a.tenant, "reason":"owned issuance control"});
    tokio::spawn(async move {
        client
            .post(url)
            .header(PRINCIPAL_HEADER, principal)
            .json(&body)
            .send()
            .await
    })
}

// Keep the handle on timeout so a failed control cannot detach a pending writer.
async fn joined<T>(mut job: JoinHandle<T>) -> Result<T, String> {
    match timeout(Duration::from_secs(4), &mut job).await {
        Ok(result) => result.map_err(|error| error.to_string()),
        Err(_) => {
            job.abort();
            let _ = job.await;
            Err("owned writer did not finish after its blocker was released".into())
        }
    }
}

async fn read_response(
    outcome: Result<reqwest::Result<reqwest::Response>, String>,
) -> (u16, Value) {
    let response = outcome.unwrap().unwrap();
    (response.status().as_u16(), response.json().await.unwrap())
}

async fn response(job: JoinHandle<reqwest::Result<reqwest::Response>>) -> (u16, Value) {
    read_response(joined(job).await).await
}

async fn grant_count(pool: &PgPool, grant: &AuthorityGrant) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM authority_grants WHERE grant_id = $1")
        .bind(&grant.grant_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn standalone_boundary_writer_waits_for_shared_guard_and_other_tenants_progress() {
    let Some(f) = fixture().await else { return };
    let tenant = TenantId::new();
    let boundary = parent(tenant);
    let (held, pid) = hold_guard(&f.pool, tenant, true).await;
    let pool = f.pool.clone();
    let value = boundary.clone();
    let job = tokio::spawn(async move {
        create_boundary(&pool, &value)
            .await
            .map_err(|e| e.to_string())
    });
    let observed = waiter(&f.pool, pid, "tenant_authority_guards", &job).await;
    let unrelated = parent(TenantId::new());
    let progress = timeout(Duration::from_secs(3), create_boundary(&f.pool, &unrelated)).await;
    held.commit().await.unwrap();
    let result = timeout(Duration::from_secs(4), job).await.unwrap().unwrap();
    let identities: i64 =
        sqlx::query_scalar("SELECT count(*) FROM tenants WHERE tenant_id = $1 OR tenant_id = $2")
            .bind(tenant.to_string())
            .bind(unrelated.tenant_id.to_string())
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert!(
        progress.unwrap().is_ok(),
        "other tenant progresses independently"
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(identities, 0, "anchors must not create identities");
    assert!(
        observed.is_some(),
        "standalone boundary creation bypassed the existing shared tenant guard"
    );
}

#[tokio::test]
async fn active_boundary_lookup_waits_for_exclusive_guard() {
    let Some(f) = fixture().await else { return };
    let a = admin(&f).await;
    let (mut held, pid) = hold_guard(&f.pool, a.tenant, false).await;
    let client = f.client.clone();
    let url = format!("{}/v1/enrollments", f.base);
    let body = json!({"kind":"role", "name":"guarded-lookup", "tenant_id":a.tenant});
    let job = tokio::spawn(async move { client.post(url).json(&body).send().await });
    let observed = waiter(&f.pool, pid, "tenant_authority_guards", &job).await;
    sqlx::query("UPDATE enrollment_boundaries SET status = 'revoked' WHERE boundary_id = $1")
        .bind(&a.boundary)
        .execute(&mut *held)
        .await
        .unwrap();
    held.commit().await.unwrap();
    let (status, body) = response(job).await;
    let roles: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_roles WHERE tenant_id = $1")
        .bind(a.tenant.to_string())
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert!(
        observed.is_some(),
        "existing-tenant active-boundary lookup bypassed the guard: {status} {body}"
    );
    assert_eq!(status, 400, "{body}");
    assert_eq!(roles, 0);
}

#[tokio::test]
async fn standalone_grant_issuance_requires_a_currently_live_parent() {
    let Some(f) = fixture().await else { return };
    let mut outcomes = Vec::new();
    for future in [false, true] {
        let mut boundary = parent(TenantId::new());
        if future {
            boundary.valid_from = Utc::now() + chrono::Duration::days(1);
        } else {
            boundary.expires_at = Utc::now() - chrono::Duration::hours(1);
        }
        create_boundary(&f.pool, &boundary).await.unwrap();
        let grant = candidate(&f.pool, boundary.tenant_id, &boundary.boundary_id).await;
        let result = create_grant(&f.pool, &grant).await;
        outcomes.push((future, result.is_err(), grant_count(&f.pool, &grant).await));
    }
    assert!(
        outcomes
            .iter()
            .all(|(_, refused, rows)| *refused && *rows == 0),
        "non-live parent issuance: {outcomes:?}"
    );
    let boundary = parent(TenantId::new());
    create_boundary(&f.pool, &boundary).await.unwrap();
    let mut scheduled = candidate(&f.pool, boundary.tenant_id, &boundary.boundary_id).await;
    scheduled.valid_from = Utc::now() + chrono::Duration::hours(1);
    create_grant(&f.pool, &scheduled)
        .await
        .expect("a future grant under a currently live parent remains supported");
}

#[tokio::test]
async fn queued_grant_issuance_checks_parent_time_after_the_guard_wait() {
    let Some(f) = fixture().await else { return };
    let boundary = parent(TenantId::new());
    create_boundary(&f.pool, &boundary).await.unwrap();
    let (held, pid) = hold_guard(&f.pool, boundary.tenant_id, true).await;
    let expires: DateTime<Utc> = sqlx::query_scalar("UPDATE enrollment_boundaries SET expires_at = clock_timestamp() + interval '750 milliseconds' WHERE boundary_id = $1 RETURNING expires_at")
        .bind(&boundary.boundary_id).fetch_one(&f.pool).await.unwrap();
    let grant = candidate(&f.pool, boundary.tenant_id, &boundary.boundary_id).await;
    let before: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&f.pool)
        .await
        .unwrap();
    let job = issue(&f.pool, &grant);
    let observed = waiter(&f.pool, pid, "tenant_authority_guards", &job).await;
    sqlx::query("SELECT pg_sleep(GREATEST(0, EXTRACT(EPOCH FROM ($1::timestamptz - clock_timestamp()))) + 0.02)")
        .bind(expires).execute(&f.pool).await.unwrap();
    held.commit().await.unwrap();
    let outcome = timeout(Duration::from_secs(4), job).await.unwrap().unwrap();
    let rows = grant_count(&f.pool, &grant).await;
    assert!(before < expires, "fixture enters before parent expiration");
    assert!(
        observed.is_some(),
        "issuance bypassed the guard: {outcome:?}, rows={rows}"
    );
    assert!(
        outcome.is_err(),
        "a parent expiring during the observed wait cannot issue: {outcome:?}"
    );
    assert_eq!(rows, 0);
}

#[tokio::test]
async fn boundary_revocation_ordered_first_refuses_later_issuance() {
    let Some(f) = fixture().await else { return };
    let a = admin(&f).await;
    let grant = candidate(&f.pool, a.tenant, &a.boundary).await;
    let (guard_seed, _) = hold_guard(&f.pool, a.tenant, true).await;
    guard_seed.commit().await.unwrap();
    let mut held = f.pool.begin().await.unwrap();
    sqlx::query("SELECT boundary_id FROM enrollment_boundaries WHERE boundary_id = $1 FOR UPDATE")
        .bind(&a.boundary)
        .execute(&mut *held)
        .await
        .unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *held)
        .await
        .unwrap();
    let revocation = revoke(&f, &a, "boundaries", &a.boundary);
    let revoker = waiter(&f.pool, pid, "enrollment_boundaries", &revocation).await;
    let issuance = issue(&f.pool, &grant);
    let issuing = if let Some(revoker) = revoker {
        waiter(&f.pool, revoker, "tenant_authority_guards", &issuance).await
    } else {
        None
    };
    held.commit().await.unwrap();
    let revoked = response(revocation).await;
    let issued = joined(issuance).await.unwrap();
    let rows = grant_count(&f.pool, &grant).await;
    assert!(
        revoker.is_some(),
        "fixture must observe the actual revocation at its target lock: {revoked:?}"
    );
    assert_eq!(revoked.0, 200, "{revoked:?}");
    assert!(
        issuing.is_some(),
        "issuance did not wait on revocation's tenant guard: {issued:?}, rows={rows}"
    );
    assert!(issued.is_err(), "revocation first must refuse issuance");
    assert_eq!(rows, 0);
}

#[tokio::test]
async fn issuance_ordered_first_finishes_before_boundary_revocation() {
    let Some(f) = fixture().await else { return };
    let a = admin(&f).await;
    let grant = candidate(&f.pool, a.tenant, &a.boundary).await;
    let mut held = f.pool.begin().await.unwrap();
    sqlx::query("SELECT pg_advisory_xact_lock(76312591)")
        .execute(&mut *held)
        .await
        .unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *held)
        .await
        .unwrap();
    sqlx::raw_sql("CREATE FUNCTION rb_test_hold_issuance() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock(76312591); RETURN NEW; END $$; CREATE TRIGGER rb_test_hold_issuance BEFORE INSERT ON authority_grants FOR EACH ROW EXECUTE FUNCTION rb_test_hold_issuance()")
        .execute(&f.pool).await.unwrap();
    let issuance = issue(&f.pool, &grant);
    let issuer = waiter(&f.pool, pid, "authority_grants", &issuance).await;
    let revocation = revoke(&f, &a, "boundaries", &a.boundary);
    let revoker = if let Some(issuer) = issuer {
        waiter(&f.pool, issuer, "tenant_authority_guards", &revocation).await
    } else {
        None
    };
    let status_before_release: String =
        sqlx::query_scalar("SELECT status FROM enrollment_boundaries WHERE boundary_id = $1")
            .bind(&a.boundary)
            .fetch_one(&f.pool)
            .await
            .unwrap();
    held.commit().await.unwrap();
    let issued = joined(issuance).await;
    let revoked = joined(revocation).await;
    sqlx::raw_sql("DROP TRIGGER rb_test_hold_issuance ON authority_grants; DROP FUNCTION rb_test_hold_issuance()")
        .execute(&f.pool).await.unwrap();
    let issued = issued.unwrap();
    let revoked = read_response(revoked).await;
    assert!(
        issuer.is_some(),
        "fixture must observe issuance inside its INSERT"
    );
    assert!(issued.is_ok(), "{issued:?}");
    assert_eq!(revoked.0, 200, "{revoked:?}");
    assert_eq!(grant_count(&f.pool, &grant).await, 1);
    assert!(
        revoker.is_some(),
        "revocation bypassed issuance's guard; parent was {status_before_release}"
    );
    assert_eq!(status_before_release, "active");
}

#[tokio::test]
async fn both_status_services_wait_for_the_tenant_guard() {
    let Some(f) = fixture().await else { return };
    let a = admin(&f).await;
    let grant = candidate(&f.pool, a.tenant, &a.boundary).await;
    create_grant(&f.pool, &grant).await.unwrap();
    let mut outcomes = Vec::new();
    for (kind, target) in [("grants", &grant.grant_id), ("boundaries", &a.boundary)] {
        let before: i64 =
            sqlx::query_scalar("SELECT revocation_epoch FROM tenants WHERE tenant_id = $1")
                .bind(a.tenant.to_string())
                .fetch_one(&f.pool)
                .await
                .unwrap();
        let (held, pid) = hold_guard(&f.pool, a.tenant, true).await;
        let job = revoke(&f, &a, kind, target);
        let observed = waiter(&f.pool, pid, "tenant_authority_guards", &job).await;
        held.commit().await.unwrap();
        let result = response(job).await;
        let after: i64 =
            sqlx::query_scalar("SELECT revocation_epoch FROM tenants WHERE tenant_id = $1")
                .bind(a.tenant.to_string())
                .fetch_one(&f.pool)
                .await
                .unwrap();
        outcomes.push((kind, observed, result, after - before));
    }
    assert!(
        outcomes
            .iter()
            .all(|(_, observed, result, delta)| observed.is_some()
                && result.0 == 200
                && *delta == 1),
        "status guard outcomes: {outcomes:?}"
    );
}

async fn target_snapshot(
    pool: &PgPool,
    table: &str,
    key: &str,
    id: &str,
    tenant: TenantId,
) -> Value {
    sqlx::query_scalar(&format!("SELECT jsonb_build_object('target', to_jsonb(t), 'epoch', (SELECT revocation_epoch FROM tenants WHERE tenant_id = $2)) FROM {table} t WHERE {key} = $1"))
        .bind(id).bind(tenant.to_string()).fetch_one(pool).await.unwrap()
}

#[tokio::test]
async fn malformed_target_status_is_preserved_and_cannot_advance_the_epoch() {
    let Some(f) = fixture().await else { return };
    let mut outcomes = Vec::new();
    for boundary_target in [false, true] {
        let a = admin(&f).await;
        let (table, key, kind, target, original) = if boundary_target {
            let mut boundary = parent(a.tenant);
            boundary.boundary_id.push_str("_suspended");
            boundary.status = BoundaryStatus::Suspended;
            create_boundary(&f.pool, &boundary).await.unwrap();
            (
                "enrollment_boundaries",
                "boundary_id",
                "boundaries",
                boundary.boundary_id,
                "suspended",
            )
        } else {
            let grant = candidate(&f.pool, a.tenant, &a.boundary).await;
            create_grant(&f.pool, &grant).await.unwrap();
            (
                "authority_grants",
                "grant_id",
                "grants",
                grant.grant_id,
                "active",
            )
        };
        sqlx::query(&format!(
            "UPDATE {table} SET status = 'owned_malformed_status' WHERE {key} = $1"
        ))
        .bind(&target)
        .execute(&f.pool)
        .await
        .unwrap();
        let before = target_snapshot(&f.pool, table, key, &target, a.tenant).await;
        let job = revoke(&f, &a, kind, &target);
        let result = joined(job).await;
        let after = target_snapshot(&f.pool, table, key, &target, a.tenant).await;
        sqlx::query(&format!("UPDATE {table} SET status = $2 WHERE {key} = $1"))
            .bind(&target)
            .bind(original)
            .execute(&f.pool)
            .await
            .unwrap();
        let result = read_response(result).await;
        let recovery = response(revoke(&f, &a, kind, &target)).await;
        outcomes.push((kind, result, before == after, recovery.0));
    }
    eprintln!("malformed target outcomes: {outcomes:?}");
    assert!(
        outcomes
            .iter()
            .all(|(_, result, unchanged, recovery)| result.0 == 500
                && result.1
                    == json!({"code":"dependency_unavailable", "message":"internal server error"})
                && *unchanged
                && *recovery == 200),
        "{outcomes:?}"
    );
}

#[tokio::test]
async fn deferred_boundary_commit_failure_keeps_uncertainty_and_rolls_back() {
    let Some(f) = fixture().await else { return };
    let boundary = parent(TenantId::new());
    sqlx::raw_sql("CREATE FUNCTION rb_test_late_boundary_fault() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned late boundary fault'; END $$; CREATE CONSTRAINT TRIGGER rb_test_late_boundary_fault AFTER INSERT ON enrollment_boundaries DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION rb_test_late_boundary_fault()")
        .execute(&f.pool).await.unwrap();
    let result = create_boundary(&f.pool, &boundary).await;
    sqlx::raw_sql("DROP TRIGGER rb_test_late_boundary_fault ON enrollment_boundaries; DROP FUNCTION rb_test_late_boundary_fault()")
        .execute(&f.pool).await.unwrap();
    let counts: (i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM enrollment_boundaries WHERE boundary_id = $1), (SELECT count(*) FROM tenant_authority_guards WHERE tenant_id = $2)")
        .bind(&boundary.boundary_id).bind(boundary.tenant_id.to_string()).fetch_one(&f.pool).await.unwrap();
    create_boundary(&f.pool, &boundary)
        .await
        .expect("successful recovery after the deferred fault is removed");
    let error = result.unwrap_err();
    assert_eq!(
        counts,
        (0, 0),
        "failed commit preserves no provisional boundary or anchor"
    );
    assert!(
        error.to_string().contains("commit outcome unconfirmed"),
        "commit failure must retain its phase: {error:?}"
    );
}

#[tokio::test]
async fn grant_creation_preserves_deferred_commit_failure_as_a_transaction_error() {
    let Some(f) = fixture().await else { return };
    let boundary = parent(TenantId::new());
    create_boundary(&f.pool, &boundary).await.unwrap();
    let grant = candidate(&f.pool, boundary.tenant_id, &boundary.boundary_id).await;
    sqlx::raw_sql("CREATE FUNCTION rb_test_late_grant_fault() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned late grant fault'; END $$; CREATE CONSTRAINT TRIGGER rb_test_late_grant_fault AFTER INSERT ON authority_grants DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION rb_test_late_grant_fault()")
        .execute(&f.pool).await.unwrap();
    let result = create_grant(&f.pool, &grant).await;
    sqlx::raw_sql("DROP TRIGGER rb_test_late_grant_fault ON authority_grants; DROP FUNCTION rb_test_late_grant_fault()")
        .execute(&f.pool).await.unwrap();
    let rows = grant_count(&f.pool, &grant).await;
    create_grant(&f.pool, &grant)
        .await
        .expect("successful grant recovery after removal of the deferred fault");
    let error = result.unwrap_err();
    assert_eq!(rows, 0);
    assert!(
        matches!(&error, GrantCreateError::Transaction(AuthorityTransactionError::Commit(sqlx::Error::Database(source))) if source.code().as_deref()==Some("P0001")),
        "commit errors retain both phase and original SQL cause: {error:?}"
    );
    let source = std::error::Error::source(&error).unwrap();
    assert!(source.downcast_ref::<AuthorityTransactionError>().is_some());
    assert!(std::error::Error::source(source)
        .unwrap()
        .downcast_ref::<sqlx::Error>()
        .is_some());
}

#[tokio::test]
async fn foreign_parent_refusal_does_not_decode_or_lock_foreign_policy() {
    let Some(f) = fixture().await else { return };
    let foreign = parent(TenantId::new());
    create_boundary(&f.pool, &foreign).await.unwrap();
    let grant = candidate(&f.pool, TenantId::new(), &foreign.boundary_id).await;
    let (held, _) = hold_guard(&f.pool, foreign.tenant_id, false).await;
    sqlx::query("UPDATE enrollment_boundaries SET permitted_actions = $2 WHERE boundary_id = $1")
        .bind(&foreign.boundary_id)
        .bind(json!(["owned_unknown_action"]))
        .execute(&f.pool)
        .await
        .unwrap();
    let before: Value = sqlx::query_scalar(
        "SELECT to_jsonb(b) FROM enrollment_boundaries b WHERE boundary_id = $1",
    )
    .bind(&foreign.boundary_id)
    .fetch_one(&f.pool)
    .await
    .unwrap();
    let result = timeout(Duration::from_secs(3), create_grant(&f.pool, &grant)).await;
    let after: Value = sqlx::query_scalar(
        "SELECT to_jsonb(b) FROM enrollment_boundaries b WHERE boundary_id = $1",
    )
    .bind(&foreign.boundary_id)
    .fetch_one(&f.pool)
    .await
    .unwrap();
    sqlx::query("UPDATE enrollment_boundaries SET permitted_actions = $2 WHERE boundary_id = $1")
        .bind(&foreign.boundary_id)
        .bind(serde_json::to_value(&foreign.permitted_actions).unwrap())
        .execute(&f.pool)
        .await
        .unwrap();
    held.commit().await.unwrap();
    let error = result
        .expect("a binding refusal must not wait on the foreign guard")
        .unwrap_err();
    assert_eq!(before, after, "foreign policy remains untouched");
    assert_eq!(grant_count(&f.pool, &grant).await, 0);
    assert!(
        matches!(&error, GrantCreateError::Refused(refusal) if refusal.violations.iter().any(|v|v.field=="grant.tenant_id")),
        "foreign policy must not be decoded for issuance: {error:?}"
    );
}

#[tokio::test]
async fn http_status_commit_failures_preserve_uncertainty_and_recover() {
    let Some(f) = fixture().await else { return };
    let mut outcomes = Vec::new();
    for boundary_target in [false, true] {
        let a = admin(&f).await;
        let (table, key, kind, target) = if boundary_target {
            (
                "enrollment_boundaries",
                "boundary_id",
                "boundaries",
                a.boundary.clone(),
            )
        } else {
            let grant = candidate(&f.pool, a.tenant, &a.boundary).await;
            create_grant(&f.pool, &grant).await.unwrap();
            ("authority_grants", "grant_id", "grants", grant.grant_id)
        };
        let before = target_snapshot(&f.pool, table, key, &target, a.tenant).await;
        sqlx::raw_sql(&format!("CREATE FUNCTION rb_test_late_status_fault() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned late status fault'; END $$; CREATE CONSTRAINT TRIGGER rb_test_late_status_fault AFTER UPDATE ON {table} DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION rb_test_late_status_fault()"))
            .execute(&f.pool).await.unwrap();
        let result = joined(revoke(&f, &a, kind, &target)).await;
        sqlx::raw_sql(&format!("DROP TRIGGER rb_test_late_status_fault ON {table}; DROP FUNCTION rb_test_late_status_fault()"))
            .execute(&f.pool).await.unwrap();
        let after = target_snapshot(&f.pool, table, key, &target, a.tenant).await;
        let result = read_response(result).await;
        let recovery = response(revoke(&f, &a, kind, &target)).await;
        outcomes.push((kind, result, before == after, recovery.0));
    }
    assert!(outcomes.iter().all(|(_, result, unchanged, recovery)| result.0==500
        && result.1==json!({"code":"commit_outcome_unconfirmed", "message":"transaction outcome is unconfirmed; inspect the target before retrying"})
        && *unchanged && *recovery==200), "HTTP commit uncertainty and controlled rollback/recovery: {outcomes:?}");
}

#[derive(Clone, Copy, Debug)]
enum RefusalKind {
    Missing,
    Structural,
    Time,
}

impl RefusalKind {
    fn matches(self, result: &Result<(), GrantCreateError>) -> bool {
        matches!(
            (self, result),
            (Self::Missing, Err(GrantCreateError::MissingBoundary { .. }))
                | (Self::Structural, Err(GrantCreateError::Refused(_)))
                | (Self::Time, Err(GrantCreateError::BoundaryNotLive { .. }))
        )
    }
}

async fn refused_candidate(pool: &PgPool, kind: RefusalKind) -> AuthorityGrant {
    let mut boundary = parent(TenantId::new());
    if matches!(kind, RefusalKind::Time) {
        boundary.expires_at = Utc::now() - chrono::Duration::hours(1);
    }
    create_boundary(pool, &boundary).await.unwrap();
    let mut grant = candidate(pool, boundary.tenant_id, &boundary.boundary_id).await;
    match kind {
        RefusalKind::Missing => grant.boundary_id.push_str("_missing"),
        RefusalKind::Structural => grant.actions = vec![GrantAction::ThreadCreate],
        RefusalKind::Time => {}
    }
    grant
}

async fn anchor_count(pool: &PgPool, tenant: TenantId) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM tenant_authority_guards WHERE tenant_id = $1")
        .bind(tenant.to_string())
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Model an authority namespace whose coordination row has not been created yet.
/// Only the unique fixture tenant is touched, before any concurrent operation.
async fn remove_fixture_anchor(pool: &PgPool, tenant: TenantId) {
    let deleted = sqlx::query("DELETE FROM tenant_authority_guards WHERE tenant_id = $1")
        .bind(tenant.to_string())
        .execute(pool)
        .await
        .unwrap();
    assert_eq!(deleted.rows_affected(), 1);
}

async fn recover_refused_candidate(pool: &PgPool, tenant: TenantId) {
    sqlx::query("UPDATE enrollment_boundaries SET expires_at = clock_timestamp() + interval '2 days' WHERE tenant_id = $1")
        .bind(tenant.to_string()).execute(pool).await.unwrap();
    let grant = candidate(pool, tenant, &format!("bnd_{tenant}")).await;
    create_grant(pool, &grant).await.unwrap();
    assert_eq!(grant_count(pool, &grant).await, 1);
    assert_eq!(anchor_count(pool, tenant).await, 1);
}

#[tokio::test]
async fn grant_refusals_roll_back_new_anchors_and_preserve_existing_anchors() {
    let Some(f) = fixture().await else { return };
    let mut outcomes = Vec::new();
    for kind in [
        RefusalKind::Missing,
        RefusalKind::Structural,
        RefusalKind::Time,
    ] {
        for existing in [false, true] {
            let grant = refused_candidate(&f.pool, kind).await;
            if !existing {
                remove_fixture_anchor(&f.pool, grant.tenant_id).await;
            }
            let result = create_grant(&f.pool, &grant).await;
            let anchors = anchor_count(&f.pool, grant.tenant_id).await;
            let grants = grant_count(&f.pool, &grant).await;
            let correct = kind.matches(&result) && anchors == i64::from(existing) && grants == 0;
            outcomes.push((
                kind,
                existing,
                format!("{result:?}"),
                anchors,
                grants,
                correct,
            ));
            recover_refused_candidate(&f.pool, grant.tenant_id).await;
        }
    }
    eprintln!("grant refusal anchor outcomes: {outcomes:?}");
    assert!(outcomes.iter().all(|outcome| outcome.5), "{outcomes:?}");
}

#[tokio::test]
async fn grant_refusals_do_not_reach_deferred_anchor_commit_faults() {
    let Some(f) = fixture().await else { return };
    let mut outcomes = Vec::new();
    for kind in [
        RefusalKind::Missing,
        RefusalKind::Structural,
        RefusalKind::Time,
    ] {
        let grant = refused_candidate(&f.pool, kind).await;
        remove_fixture_anchor(&f.pool, grant.tenant_id).await;
        sqlx::raw_sql("CREATE FUNCTION rb_test_refusal_anchor_fault() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned refusal anchor commit fault'; END $$; CREATE CONSTRAINT TRIGGER rb_test_refusal_anchor_fault AFTER INSERT ON tenant_authority_guards DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION rb_test_refusal_anchor_fault()")
            .execute(&f.pool).await.unwrap();
        let result = create_grant(&f.pool, &grant).await;
        sqlx::raw_sql("DROP TRIGGER rb_test_refusal_anchor_fault ON tenant_authority_guards; DROP FUNCTION rb_test_refusal_anchor_fault()")
            .execute(&f.pool).await.unwrap();
        let anchors = anchor_count(&f.pool, grant.tenant_id).await;
        let grants = grant_count(&f.pool, &grant).await;
        let correct = kind.matches(&result) && anchors == 0 && grants == 0;
        outcomes.push((kind, format!("{result:?}"), anchors, grants, correct));
        recover_refused_candidate(&f.pool, grant.tenant_id).await;
    }
    eprintln!("grant refusal deferred fault outcomes: {outcomes:?}");
    assert!(outcomes.iter().all(|outcome| outcome.4), "{outcomes:?}");
}
