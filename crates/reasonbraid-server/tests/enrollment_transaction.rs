//! Complete development enrollment: transaction ordering, replay and rollback.
//! Every database and server fixture belongs to the supervised local runner.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::sync::OnceLock;
use std::time::Duration;

use chrono::Utc;
use reasonbraid_core::{
    BoundaryStatus, EnrollmentAuthorityBoundary, GrantAction, RiskClass, TenantId,
};
use reasonbraid_server::{api_router, create_boundary, PRINCIPAL_HEADER};
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

fn post(
    f: &Fixture,
    path: &str,
    body: Value,
    principal: Option<String>,
) -> JoinHandle<Result<(u16, Value), String>> {
    let client = f.client.clone();
    let url = format!("{}{path}", f.base);
    tokio::spawn(async move {
        let mut request = client.post(url).json(&body);
        if let Some(principal) = principal {
            request = request.header(PRINCIPAL_HEADER, principal);
        }
        let response = request.send().await.map_err(|error| error.to_string())?;
        let status = response.status().as_u16();
        Ok((
            status,
            response.json().await.map_err(|error| error.to_string())?,
        ))
    })
}

async fn response(job: JoinHandle<Result<(u16, Value), String>>) -> (u16, Value) {
    joined(job).await.unwrap().unwrap()
}

async fn admin(f: &Fixture) -> Value {
    let result=response(post(f,"/v1/enrollments",json!({"kind":"human","name":format!("owned-{}",TenantId::new()),"actions":["ignored-human-action"]}),None)).await;
    assert_eq!(result.0, 200, "{result:?}");
    result.1
}

fn role(a: &Value, name: &str) -> Value {
    json!({"tenant_id":a["tenant_id"],"kind":"role","name":name})
}

fn revoke(f: &Fixture, a: &Value) -> JoinHandle<Result<(u16, Value), String>> {
    post(
        f,
        &format!(
            "/v1/admin/boundaries/{}/revoke",
            a["boundary_id"].as_str().unwrap()
        ),
        json!({"tenant_id":a["tenant_id"],"reason":"owned enrollment ordering"}),
        Some(a["principal_id"].as_str().unwrap().into()),
    )
}

fn tenant(a: &Value) -> TenantId {
    a["tenant_id"].as_str().unwrap().parse().unwrap()
}

const ENROLLMENT_TABLES: [&str; 9] = [
    "tenants",
    "enrollment_boundaries",
    "authority_grants",
    "human_principals",
    "agent_roles",
    "usage_quotas",
    "enrollments",
    "tenant_authority_guards",
    "tenant_bootstrap_requests",
];

async fn snapshot(pool: &PgPool) -> Vec<Value> {
    let mut rows = Vec::new();
    for table in ENROLLMENT_TABLES {
        rows.push(sqlx::query_scalar(&format!("SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM {table} t"))
            .fetch_one(pool).await.unwrap());
    }
    rows
}

fn changed_tables(before: &[Value], after: &[Value]) -> Vec<String> {
    assert_eq!(before.len(), ENROLLMENT_TABLES.len());
    assert_eq!(after.len(), ENROLLMENT_TABLES.len());
    ENROLLMENT_TABLES
        .iter()
        .zip(before.iter().zip(after))
        .filter(|(_, (before, after))| before != after)
        .map(|(table, (before, after))| {
            format!(
                "{table}: {} -> {} rows",
                before.as_array().unwrap().len(),
                after.as_array().unwrap().len()
            )
        })
        .collect()
}

async fn role_count(pool: &PgPool, a: &Value, name: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM agent_roles WHERE tenant_id = $1 AND name = $2")
        .bind(a["tenant_id"].as_str().unwrap())
        .bind(name)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn hold_issuance(pool: &PgPool) -> (Transaction<'static, Postgres>, i32) {
    let mut held = pool.begin().await.unwrap();
    sqlx::query("SELECT pg_advisory_xact_lock(76312592)")
        .execute(&mut *held)
        .await
        .unwrap();
    let pid = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *held)
        .await
        .unwrap();
    sqlx::raw_sql("CREATE FUNCTION rb_test_enrollment_hold() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock(76312592); RETURN NEW; END $$; CREATE TRIGGER rb_test_enrollment_hold BEFORE INSERT ON authority_grants FOR EACH ROW EXECUTE FUNCTION rb_test_enrollment_hold()")
        .execute(pool).await.unwrap();
    (held, pid)
}

async fn restore_issuance(pool: &PgPool) {
    sqlx::raw_sql("DROP TRIGGER rb_test_enrollment_hold ON authority_grants; DROP FUNCTION rb_test_enrollment_hold()")
        .execute(pool).await.unwrap();
}

#[tokio::test]
async fn enrollment_takes_the_guard_before_replay_and_preserves_replay_validation_order() {
    let Some(f) = fixture().await else { return };
    let a = admin(&f).await;
    let created = response(post(
        &f,
        "/v1/enrollments",
        role(&a, "guarded-replay"),
        None,
    ))
    .await;
    assert_eq!(created.0, 200);
    let before = snapshot(&f.pool).await;
    let (held, pid) = hold_guard(&f.pool, tenant(&a), false).await;
    let mut request = role(&a, "guarded-replay");
    request["actions"] = json!(["invalid-but-replayed"]);
    let replay = post(&f, "/v1/enrollments", request, None);
    let observed = waiter(&f.pool, pid, "tenant_authority_guards", &replay).await;
    // Another namespace progresses while the original tenant is held.
    let unrelated = admin(&f).await;
    held.commit().await.unwrap();
    let replay = response(replay).await;
    assert!(
        observed.is_some(),
        "replay bypassed the tenant guard: {replay:?}"
    );
    assert_ne!(a["tenant_id"], unrelated["tenant_id"]);
    assert_eq!(
        replay,
        (
            200,
            json!({"tenant_id":a["tenant_id"],"principal_id":created.1["principal_id"],"kind":"role","name":"guarded-replay","replayed":true})
        )
    );
    assert_eq!(role_count(&f.pool, &a, "guarded-replay").await, 1);
    // Exclude the intentional unrelated bootstrap, then compare this tenant's rows.
    let after = snapshot(&f.pool).await;
    for (before, after) in before.iter().zip(after.iter()) {
        let owned = |rows: &Value| {
            rows.as_array()
                .unwrap()
                .iter()
                .filter(|row| row["tenant_id"] == a["tenant_id"])
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(owned(before), owned(after));
    }
}

#[tokio::test]
async fn enrollment_first_holds_its_guard_through_identity_and_commit_before_revocation() {
    let Some(f) = fixture().await else { return };
    let a = admin(&f).await;
    let (held, pid) = hold_issuance(&f.pool).await;
    let enrollment = post(&f, "/v1/enrollments", role(&a, "before-revocation"), None);
    let issuer = waiter(&f.pool, pid, "authority_grants", &enrollment).await;
    let revocation = revoke(&f, &a);
    let revoker = if let Some(issuer) = issuer {
        waiter(&f.pool, issuer, "tenant_authority_guards", &revocation).await
    } else {
        None
    };
    let before: String =
        sqlx::query_scalar("SELECT status FROM enrollment_boundaries WHERE boundary_id = $1")
            .bind(a["boundary_id"].as_str().unwrap())
            .fetch_one(&f.pool)
            .await
            .unwrap();
    held.commit().await.unwrap();
    let enrolled = joined(enrollment).await;
    let revoked = joined(revocation).await;
    restore_issuance(&f.pool).await;
    let enrolled = enrolled.unwrap().unwrap();
    let revoked = revoked.unwrap().unwrap();
    eprintln!("enrollment-first: issuer={issuer:?}, revoker={revoker:?}, parent={before}, enrollment={enrolled:?}, revocation={revoked:?}");
    assert!(
        issuer.is_some() && revoker.is_some(),
        "both actual waits must be observed"
    );
    assert_eq!(before, "active");
    assert_eq!(enrolled.0, 200);
    assert_eq!(revoked.0, 200);
    assert_eq!(role_count(&f.pool, &a, "before-revocation").await, 1);
    let next = response(post(
        &f,
        "/v1/enrollments",
        role(&a, "after-revocation"),
        None,
    ))
    .await;
    assert_eq!(next.0, 400);
    assert_eq!(role_count(&f.pool, &a, "after-revocation").await, 0);
}

#[tokio::test]
async fn revocation_first_fences_enrollment_after_an_observed_guard_wait() {
    let Some(f) = fixture().await else { return };
    let a = admin(&f).await;
    let mut held = f.pool.begin().await.unwrap();
    sqlx::query("SELECT boundary_id FROM enrollment_boundaries WHERE boundary_id = $1 FOR UPDATE")
        .bind(a["boundary_id"].as_str().unwrap())
        .execute(&mut *held)
        .await
        .unwrap();
    let pid = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *held)
        .await
        .unwrap();
    let revocation = revoke(&f, &a);
    let revoker = waiter(&f.pool, pid, "enrollment_boundaries", &revocation).await;
    let enrollment = post(&f, "/v1/enrollments", role(&a, "revocation-first"), None);
    let issuer = if let Some(revoker) = revoker {
        waiter(&f.pool, revoker, "tenant_authority_guards", &enrollment).await
    } else {
        None
    };
    held.commit().await.unwrap();
    let revoked = joined(revocation).await;
    let enrolled = joined(enrollment).await;
    let revoked = revoked.unwrap().unwrap();
    let enrolled = enrolled.unwrap().unwrap();
    assert!(
        revoker.is_some() && issuer.is_some(),
        "revoker={revoker:?}, issuer={issuer:?}"
    );
    assert_eq!(revoked.0, 200);
    assert_eq!(enrolled.0, 400, "{enrolled:?}");
    assert_eq!(role_count(&f.pool, &a, "revocation-first").await, 0);
}

#[tokio::test]
async fn enrollment_checks_database_time_after_waiting_past_parent_expiry() {
    let Some(f) = fixture().await else { return };
    let a = admin(&f).await;
    sqlx::query("UPDATE enrollment_boundaries SET expires_at = clock_timestamp() + interval '750 milliseconds' WHERE boundary_id = $1")
        .bind(a["boundary_id"].as_str().unwrap()).execute(&f.pool).await.unwrap();
    let (held, pid) = hold_guard(&f.pool, tenant(&a), false).await;
    let before = snapshot(&f.pool).await;
    let job = post(&f, "/v1/enrollments", role(&a, "expired-in-queue"), None);
    let observed = waiter(&f.pool, pid, "tenant_authority_guards", &job).await;
    sqlx::query("SELECT pg_sleep(greatest(0,extract(epoch FROM (expires_at - clock_timestamp()))) + 0.02) FROM enrollment_boundaries WHERE boundary_id = $1")
        .bind(a["boundary_id"].as_str().unwrap()).execute(&f.pool).await.unwrap();
    held.commit().await.unwrap();
    let result = response(job).await;
    let after = snapshot(&f.pool).await;
    eprintln!(
        "queued expiry: result={result:?}, changed={:?}",
        changed_tables(&before, &after)
    );
    assert!(observed.is_some());
    assert_eq!(result.0, 400, "{result:?}");
    assert!(
        result.1["message"].as_str().unwrap().contains("not live"),
        "{result:?}"
    );
    assert_eq!(
        before, after,
        "queued expiry must leave every enrollment table unchanged"
    );
}

#[tokio::test]
async fn concurrent_same_name_enrollment_has_one_new_identity_and_one_honest_replay() {
    let Some(f) = fixture().await else { return };
    let a = admin(&f).await;
    let (held, pid) = hold_issuance(&f.pool).await;
    let first = post(&f, "/v1/enrollments", role(&a, "same-name"), None);
    let issuer = waiter(&f.pool, pid, "authority_grants", &first).await;
    let second = post(&f, "/v1/enrollments", role(&a, "same-name"), None);
    let queued = if let Some(issuer) = issuer {
        waiter(&f.pool, issuer, "tenant_authority_guards", &second).await
    } else {
        None
    };
    held.commit().await.unwrap();
    let first = joined(first).await;
    let second = joined(second).await;
    restore_issuance(&f.pool).await;
    let first = first.unwrap().unwrap();
    let second = second.unwrap().unwrap();
    eprintln!(
        "same-name: issuer={issuer:?}, queued={queued:?}, first={first:?}, second={second:?}"
    );
    assert!(issuer.is_some() && queued.is_some());
    assert_eq!(first.0, 200);
    assert_eq!(second.0, 200);
    assert_eq!(first.1["replayed"], false);
    assert_eq!(second.1["replayed"], true);
    assert_eq!(first.1["principal_id"], second.1["principal_id"]);
    assert!(first.1.get("grant_id").is_some());
    assert!(second.1.get("grant_id").is_none());
    assert_eq!(role_count(&f.pool, &a, "same-name").await, 1);
    let counts: (i64,i64,i64)=sqlx::query_as("SELECT (SELECT count(*) FROM authority_grants WHERE tenant_id = $1), (SELECT count(*) FROM enrollments WHERE tenant_id = $1), (SELECT count(*) FROM usage_quotas WHERE tenant_id = $1)")
        .bind(a["tenant_id"].as_str().unwrap()).fetch_one(&f.pool).await.unwrap();
    // 5 quota rows, and they are NAMED rather than counted: one tenant-scoped
    // invite ceiling, the two acquisition defaults `.11.14.3.14` added, and one
    // write ceiling per principal — two principals here.
    assert_eq!(
        counts,
        (2, 2, 5),
        "quota scopes: {:?}",
        quota_scopes(&f.pool, a["tenant_id"].as_str().unwrap()).await
    );
    let tenant_id = a["tenant_id"].as_str().unwrap().to_string();
    let scopes = quota_scopes(&f.pool, &tenant_id).await;
    let kinds: Vec<&str> = scopes.iter().map(|(kind, _)| kind.as_str()).collect();
    assert_eq!(
        kinds,
        vec![
            "destination",
            "principal",
            "principal",
            "resolver",
            "tenant"
        ],
        "the five rows are one per acquisition scope, one per principal, one tenant: {scopes:?}"
    );
    for (kind, id) in &scopes {
        match kind.as_str() {
            "destination" | "resolver" => {
                assert_eq!(id, "*", "an acquisition default is the `*` row")
            }
            "tenant" => assert_eq!(id, &tenant_id),
            "principal" => assert!(id.starts_with("rol_") || id.starts_with("hpr_"), "{id}"),
            other => panic!("unexpected quota scope `{other}`"),
        }
    }
}

/// The quota SCOPES a tenant carries, as `(scope_kind, scope_id)` pairs.
///
/// ⛔ Asserting a COUNT alone is what let `SIGNOFF-REPAIR.11.14.3.14` move this
/// number without anyone noticing: it added the two acquisition defaults, the
/// expectations here still said the old total, and nothing ran this suite until
/// the remote did (`SIGNOFF-REPAIR.11.28`). A count has no producer; these
/// pairs do, and a future row that changes the total names itself here.
async fn quota_scopes(pool: &PgPool, tenant: &str) -> Vec<(String, String)> {
    let mut rows: Vec<(String, String)> =
        sqlx::query_as("SELECT scope_kind, scope_id FROM usage_quotas WHERE tenant_id = $1")
            .bind(tenant)
            .fetch_all(pool)
            .await
            .expect("read the tenant's quota scopes");
    rows.sort();
    rows
}

#[tokio::test]
async fn later_enrollment_storage_faults_roll_back_every_row_and_recover() {
    let Some(f) = fixture().await else { return };
    let a = admin(&f).await;
    let mut outcomes = Vec::new();
    for bootstrap in [false, true] {
        for stage in ["identity", "principal_quota", "enrollment", "commit"] {
            let name = format!("{bootstrap}-{stage}");
            let request = if bootstrap {
                json!({"kind":"human","name":name})
            } else {
                role(&a, &name)
            };
            let (table, when, condition) = match stage {
                "identity" => (
                    if bootstrap {
                        "human_principals"
                    } else {
                        "agent_roles"
                    },
                    "BEFORE",
                    "",
                ),
                "principal_quota" => (
                    "usage_quotas",
                    "BEFORE",
                    "WHEN (NEW.scope_kind = 'principal')",
                ),
                "enrollment" => ("enrollments", "BEFORE", ""),
                "commit" => ("enrollments", "AFTER", ""),
                _ => unreachable!(),
            };
            let constraint = if stage == "commit" { "CONSTRAINT " } else { "" };
            let deferred = if stage == "commit" {
                "DEFERRABLE INITIALLY DEFERRED"
            } else {
                ""
            };
            let before = snapshot(&f.pool).await;
            sqlx::raw_sql(&format!("CREATE FUNCTION rb_test_enrollment_fault() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned late enrollment fault'; END $$; CREATE {constraint}TRIGGER rb_test_enrollment_fault {when} INSERT ON {table} {deferred} FOR EACH ROW {condition} EXECUTE FUNCTION rb_test_enrollment_fault()"))
                .execute(&f.pool).await.unwrap();
            let result = joined(post(&f, "/v1/enrollments", request.clone(), None)).await;
            sqlx::raw_sql(&format!("DROP TRIGGER rb_test_enrollment_fault ON {table}; DROP FUNCTION rb_test_enrollment_fault()"))
                .execute(&f.pool).await.unwrap();
            let after = snapshot(&f.pool).await;
            let result = result.unwrap().unwrap();
            let recovery = response(post(&f, "/v1/enrollments", request, None)).await;
            let expected = if stage == "commit" {
                json!({"code":"commit_outcome_unconfirmed","message":"transaction outcome is unconfirmed; inspect the target before retrying"})
            } else {
                json!({"code":"dependency_unavailable","message":"internal server error"})
            };
            let correct = result.0 == 500
                && result.1 == expected
                && before == after
                && recovery.0 == 200
                && recovery.1["replayed"] == false;
            outcomes.push((
                bootstrap,
                stage,
                result,
                before == after,
                recovery.0,
                correct,
                changed_tables(&before, &after),
            ));
        }
    }
    eprintln!("late enrollment faults: {outcomes:?}");
    assert!(outcomes.iter().all(|case| case.5), "{outcomes:?}");
}

#[tokio::test]
async fn enrollment_policy_errors_roll_back_new_anchors_and_preserve_validation_and_replay() {
    let Some(f) = fixture().await else { return };
    let a = admin(&f).await;
    let r = response(post(&f, "/v1/enrollments", role(&a, "defaults"), None)).await;
    assert_eq!(r.0, 200);
    let actions: Value =
        sqlx::query_scalar("SELECT actions FROM authority_grants WHERE grant_id = $1")
            .bind(r.1["grant_id"].as_str().unwrap())
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!(
        actions,
        json!(["thread_contribute", "thread_invitation_respond"])
    );
    let human_actions: Value =
        sqlx::query_scalar("SELECT actions FROM authority_grants WHERE grant_id = $1")
            .bind(a["grant_id"].as_str().unwrap())
            .fetch_one(&f.pool)
            .await
            .unwrap();
    assert_eq!(human_actions.as_array().unwrap().len(), 9);
    let mut outcomes = Vec::new();
    for condition in [
        "invalid_action",
        "structural",
        "expired",
        "future",
        "frozen",
        "unknown",
    ] {
        let owned = admin(&f).await;
        let mut request = role(&owned, condition);
        let expected = match condition {
            "invalid_action" => {
                request["actions"] = json!(["unknown_action"]);
                "action `unknown_action`"
            }
            "structural" => {
                request["actions"] = json!(["thread_create"]);
                sqlx::query(
                    "UPDATE enrollment_boundaries SET permitted_actions = $2 WHERE tenant_id = $1",
                )
                .bind(owned["tenant_id"].as_str().unwrap())
                .bind(json!(["thread_contribute"]))
                .execute(&f.pool)
                .await
                .unwrap();
                "exceeds its boundary"
            }
            "expired" => {
                sqlx::query("UPDATE enrollment_boundaries SET valid_from = clock_timestamp() - interval '1 day', expires_at = clock_timestamp() - interval '1 second' WHERE tenant_id = $1").bind(owned["tenant_id"].as_str().unwrap()).execute(&f.pool).await.unwrap();
                "not live"
            }
            "future" => {
                sqlx::query("UPDATE enrollment_boundaries SET valid_from = clock_timestamp() + interval '1 day' WHERE tenant_id = $1").bind(owned["tenant_id"].as_str().unwrap()).execute(&f.pool).await.unwrap();
                "not live"
            }
            "frozen" => {
                sqlx::query(
                    "UPDATE enrollment_boundaries SET status = 'revoked' WHERE tenant_id = $1",
                )
                .bind(owned["tenant_id"].as_str().unwrap())
                .execute(&f.pool)
                .await
                .unwrap();
                "no active enrollment boundary"
            }
            "unknown" => {
                request["tenant_id"] = json!(TenantId::new());
                request["actions"] = json!(["unknown_action"]);
                "no active enrollment boundary"
            }
            _ => unreachable!(),
        };
        // An exact fixture namespace before any concurrent operation: model first use.
        sqlx::query("DELETE FROM tenant_authority_guards WHERE tenant_id = $1")
            .bind(owned["tenant_id"].as_str().unwrap())
            .execute(&f.pool)
            .await
            .unwrap();
        let before = snapshot(&f.pool).await;
        let result = response(post(&f, "/v1/enrollments", request, None)).await;
        let after = snapshot(&f.pool).await;
        let correct = result.0 == 400
            && result.1["code"] == "invalid_command"
            && result.1["message"].as_str().unwrap().contains(expected)
            && before == after;
        outcomes.push((
            condition,
            result,
            before == after,
            correct,
            changed_tables(&before, &after),
        ));
    }
    sqlx::query("UPDATE enrollment_boundaries SET status = 'revoked' WHERE tenant_id = $1")
        .bind(a["tenant_id"].as_str().unwrap())
        .execute(&f.pool)
        .await
        .unwrap();
    let before = snapshot(&f.pool).await;
    let mut replay = role(&a, "defaults");
    replay["actions"] = json!(["unknown_action"]);
    let replay = response(post(&f, "/v1/enrollments", replay, None)).await;
    assert_eq!(replay.0, 200);
    assert_eq!(replay.1["principal_id"], r.1["principal_id"]);
    assert_eq!(replay.1["replayed"], true);
    assert_eq!(before, snapshot(&f.pool).await);
    for request in [
        json!({"kind":"role","name":"missing-tenant"}),
        json!({"kind":"invalid","name":"bad-kind"}),
        json!({"kind":"human","name":"bad-tenant","tenant_id":"malformed"}),
    ] {
        let result = response(post(&f, "/v1/enrollments", request, None)).await;
        assert_eq!(result.0, 400);
    }
    eprintln!("enrollment policy outcomes: {outcomes:?}");
    assert!(outcomes.iter().all(|case| case.3), "{outcomes:?}");
}

#[tokio::test]
async fn standalone_authority_namespace_cannot_implicitly_create_a_tenant_identity() {
    let Some(f) = fixture().await else { return };
    let b = parent(TenantId::new());
    create_boundary(&f.pool, &b).await.unwrap();
    let before = snapshot(&f.pool).await;
    let request = json!({"tenant_id":b.tenant_id,"kind":"role","name":"no-tenant-identity","actions":["thread_contribute"]});
    let result = response(post(&f, "/v1/enrollments", request.clone(), None)).await;
    assert_eq!(
        result,
        (
            500,
            json!({"code":"dependency_unavailable","message":"internal server error"})
        )
    );
    assert_eq!(before, snapshot(&f.pool).await);
    // Supply the missing prerequisite explicitly in this isolated fixture.
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind(b.tenant_id.to_string())
        .execute(&f.pool)
        .await
        .unwrap();
    let recovery = response(post(&f, "/v1/enrollments", request, None)).await;
    assert_eq!(recovery.0, 200);
    assert_eq!(recovery.1["replayed"], false);
}

/// No-key bootstrap is intentionally a new request on each invocation. This
/// control preserves that compatibility while proving why recovery needs an
/// explicit client-known request identity rather than a name-based retry.
#[tokio::test]
async fn unconfirmed_no_key_bootstrap_can_commit_before_a_distinct_repeated_request() {
    let Some(f) = fixture().await else { return };
    let name = format!("uncertain-bootstrap-{}", TenantId::new());
    let request = json!({"kind":"human", "name":name});
    // Each fixed sleep is below the 10s statement ceiling, but together they
    // exceed the 15s whole-operation limit while COMMIT is already executing.
    sqlx::raw_sql("CREATE FUNCTION rb_test_bootstrap_body_delay() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_sleep(6); RETURN NEW; END $$; CREATE TRIGGER rb_test_bootstrap_body_delay BEFORE INSERT ON enrollments FOR EACH ROW EXECUTE FUNCTION rb_test_bootstrap_body_delay(); CREATE FUNCTION rb_test_bootstrap_commit_delay() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_sleep(9.5); RETURN NEW; END $$; CREATE CONSTRAINT TRIGGER rb_test_bootstrap_commit_delay AFTER INSERT ON enrollments DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION rb_test_bootstrap_commit_delay()")
        .execute(&f.pool).await.unwrap();
    let mut job = post(&f, "/v1/enrollments", request.clone(), None);
    let committing = timeout(Duration::from_secs(10), async {
        loop {
            let pid: Option<i32> = sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname = current_database() AND query = 'COMMIT' AND wait_event = 'PgSleep' ORDER BY pid LIMIT 1")
                .fetch_optional(&f.pool).await.unwrap();
            if pid.is_some() || job.is_finished() { return pid; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.ok().flatten();
    // Keep ownership on timeout; never detach the client request job.
    let result = match timeout(Duration::from_secs(20), &mut job).await {
        Ok(result) => result
            .map_err(|error| error.to_string())
            .and_then(|result| result),
        Err(_) => {
            job.abort();
            let _ = job.await;
            Err("bootstrap did not finish after its bounded commit wait".into())
        }
    };
    let readback = timeout(Duration::from_secs(5), async {
        loop {
            let rows: Vec<(String, String)> = sqlx::query_as("SELECT tenant_id, principal_id FROM enrollments WHERE kind = 'human' AND name = $1 ORDER BY tenant_id")
                .bind(&name).fetch_all(&f.pool).await.unwrap();
            if !rows.is_empty() { return rows; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await;
    // Restore all fixed faults before outcome assertions or the next request.
    sqlx::raw_sql("DROP TRIGGER rb_test_bootstrap_body_delay ON enrollments; DROP FUNCTION rb_test_bootstrap_body_delay(); DROP TRIGGER rb_test_bootstrap_commit_delay ON enrollments; DROP FUNCTION rb_test_bootstrap_commit_delay()")
        .execute(&f.pool).await.unwrap();
    let result = result.unwrap();
    let original = readback.expect("authoritative readback of the original committed bootstrap");
    eprintln!("uncertain no-key bootstrap: COMMIT backend={committing:?}, response={result:?}, original={original:?}");
    assert!(committing.is_some(), "must observe actual COMMIT/PgSleep");
    assert_eq!(
        result,
        (
            500,
            json!({"code":"commit_outcome_unconfirmed","message":"transaction outcome is unconfirmed; inspect the target before retrying"})
        )
    );
    assert_eq!(
        original.len(),
        1,
        "exactly one original committed bootstrap"
    );
    let original_tenant = &original[0].0;
    let original_principal = &original[0].1;
    let repeated = response(post(&f, "/v1/enrollments", request, None)).await;
    assert_eq!(repeated.0, 200, "{repeated:?}");
    assert_eq!(repeated.1["replayed"], false);
    assert_ne!(repeated.1["tenant_id"], *original_tenant);
    assert_ne!(repeated.1["principal_id"], *original_principal);
    let tenants: Vec<String> = sqlx::query_scalar(
        "SELECT tenant_id FROM enrollments WHERE kind = 'human' AND name = $1 ORDER BY tenant_id",
    )
    .bind(&name)
    .fetch_all(&f.pool)
    .await
    .unwrap();
    assert_eq!(tenants.len(), 2);
    for tenant in tenants {
        let counts: (i64,i64,i64,i64,i64,i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM tenants WHERE tenant_id = $1), (SELECT count(*) FROM enrollment_boundaries WHERE tenant_id = $1), (SELECT count(*) FROM authority_grants WHERE tenant_id = $1), (SELECT count(*) FROM human_principals WHERE tenant_id = $1), (SELECT count(*) FROM usage_quotas WHERE tenant_id = $1), (SELECT count(*) FROM enrollments WHERE tenant_id = $1), (SELECT count(*) FROM tenant_authority_guards WHERE tenant_id = $1)")
            .bind(&tenant).fetch_one(&f.pool).await.unwrap();
        // 4 quota rows per tenant, NAMED rather than counted (see `quota_scopes`).
        assert_eq!(
            counts,
            (1, 1, 1, 1, 4, 1, 1),
            "quota scopes: {:?}",
            quota_scopes(&f.pool, &tenant).await
        );
        let scopes = quota_scopes(&f.pool, &tenant).await;
        let kinds: Vec<&str> = scopes.iter().map(|(kind, _)| kind.as_str()).collect();
        assert_eq!(
            kinds,
            vec!["destination", "principal", "resolver", "tenant"],
            "one tenant invite ceiling, the two acquisition defaults, one principal write ceiling: {scopes:?}"
        );
    }
}
