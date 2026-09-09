//! Keyed development bootstrap recovery against the supervised local database.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::sync::OnceLock;
use std::time::Duration;

use reasonbraid_core::{RequestId, TenantId};
use reasonbraid_server::api_router;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio::sync::{Mutex, MutexGuard};
use tokio::task::JoinHandle;
use tokio::time::timeout;

static LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct Fixture {
    pool: PgPool,
    client: reqwest::Client,
    url: String,
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
    let url = format!("http://{}/v1/enrollments", listener.local_addr().unwrap());
    let router = api_router(pool.clone());
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    Some(Fixture {
        pool,
        // This fixture serves plaintext loopback only. Native certificate and
        // ambient proxy discovery add unrelated host dependencies before tests.
        client: reqwest::Client::builder()
            .no_proxy()
            .tls_built_in_root_certs(false)
            .build()
            .unwrap(),
        url,
        server,
        _guard: guard,
    })
}

fn request() -> Value {
    json!({"kind":"human", "name":format!("bootstrap-{}", RequestId::new()),
        "bootstrap_request_id":RequestId::new()})
}

fn post(f: &Fixture, body: Value) -> JoinHandle<(u16, Value)> {
    let client = f.client.clone();
    let url = f.url.clone();
    tokio::spawn(async move {
        let response = client.post(url).json(&body).send().await.unwrap();
        let status = response.status().as_u16();
        let body = response.text().await.unwrap();
        (
            status,
            serde_json::from_str(&body).unwrap_or(Value::String(body)),
        )
    })
}

async fn joined<T>(mut job: JoinHandle<T>) -> T {
    match timeout(Duration::from_secs(20), &mut job).await {
        Ok(result) => result.unwrap(),
        Err(_) => {
            job.abort();
            let _ = job.await;
            panic!("owned bootstrap request exceeded its bounded completion window");
        }
    }
}

// The baseline used the eight pre-feature tables. Qualification includes the
// outcome table so rollback/replay checks cover the complete durable result.
async fn snapshot(pool: &PgPool) -> Vec<Value> {
    let mut result = Vec::new();
    for table in [
        "tenants",
        "enrollment_boundaries",
        "authority_grants",
        "human_principals",
        "agent_roles",
        "usage_quotas",
        "enrollments",
        "tenant_authority_guards",
        "tenant_bootstrap_requests",
    ] {
        result.push(sqlx::query_scalar(&format!("SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM {table} t"))
            .fetch_one(pool).await.unwrap());
    }
    result
}

#[tokio::test]
async fn keyed_replay_returns_the_original_complete_outcome_and_conflicts_on_name() {
    let Some(f) = fixture().await else { return };
    let body = request();
    let first = joined(post(&f, body.clone())).await;
    eprintln!("keyed bootstrap first response: {first:?}");
    assert_eq!(first.0, 200, "{first:?}");
    assert_eq!(
        first.1["bootstrap_request_id"],
        body["bootstrap_request_id"]
    );
    assert_eq!(first.1["replayed"], false);
    assert!(first.1["boundary_id"].is_string());
    assert!(first.1["grant_id"].is_string());
    let before = snapshot(&f.pool).await;
    let mut replay_body = body.clone();
    replay_body["actions"] = json!(["human-actions-remain-ignored"]);
    let replay = joined(post(&f, replay_body)).await;
    let mut expected = first.1.clone();
    expected["replayed"] = json!(true);
    assert_eq!(replay, (200, expected));
    let mut conflicting = body;
    conflicting["name"] = json!("different bound name");
    let conflict = joined(post(&f, conflicting)).await;
    assert_eq!(conflict.0, 409, "{conflict:?}");
    assert_eq!(conflict.1["code"], "idempotency_conflict");
    assert_eq!(before, snapshot(&f.pool).await);
}

#[tokio::test]
async fn invalid_key_or_mode_is_a_semantic_bad_request_without_effects() {
    let Some(f) = fixture().await else { return };
    let before = snapshot(&f.pool).await;
    let mut cases = Vec::new();
    for key in [
        "".to_string(),
        "req_bad".into(),
        TenantId::new().to_string(),
        "req_00000000-0000-4000-8000-000000000001".into(),
        "req_00000000-0000-7000-8000-00000000000A".into(),
        "req_00000000000070008000000000000001".into(),
        "x".repeat(4096),
    ] {
        let mut body = request();
        body["bootstrap_request_id"] = json!(key);
        cases.push(body);
    }
    let mut role = request();
    role["kind"] = json!("role");
    role["tenant_id"] = json!(TenantId::new());
    cases.push(role);
    let mut existing = request();
    existing["tenant_id"] = json!(TenantId::new());
    cases.push(existing);
    let mut results = Vec::new();
    for body in cases {
        results.push(joined(post(&f, body)).await);
    }
    eprintln!("invalid bootstrap key/mode responses: {results:?}");
    assert_eq!(before, snapshot(&f.pool).await);
    assert!(results
        .iter()
        .all(|(status, body)| *status == 400 && body["code"] == "invalid_command"));
}

fn assert_one_bootstrap(before: &[Value], after: &[Value]) {
    let growth: Vec<usize> = before
        .iter()
        .zip(after)
        .map(|(before, after)| after.as_array().unwrap().len() - before.as_array().unwrap().len())
        .collect();
    assert_eq!(
        growth,
        [1, 1, 1, 1, 0, 2, 1, 1, 1],
        "no losing provisional row or guard may survive"
    );
}

async fn waiter<T>(f: &Fixture, holder: i32, query: &str, job: &JoinHandle<T>) -> Option<i32> {
    timeout(Duration::from_secs(4), async {
        loop {
            if job.is_finished() { return None; }
            let pid: Option<i32> = sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname = current_database() AND wait_event_type = 'Lock' AND query LIKE $1 AND $2 = ANY(pg_blocking_pids(pid)) ORDER BY pid LIMIT 1")
                .bind(format!("%{query}%")).bind(holder).fetch_optional(&f.pool).await.unwrap();
            if pid.is_some() { return pid; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.ok().flatten()
}

#[tokio::test]
async fn colliding_requests_commit_one_result_or_take_over_a_rolled_back_winner() {
    let Some(f) = fixture().await else { return };
    for (rollback_first, conflict) in [(false, false), (false, true), (true, false)] {
        let before = snapshot(&f.pool).await;
        let mut gate = f.pool.begin().await.unwrap();
        sqlx::query("SELECT pg_advisory_xact_lock(76432791)")
            .execute(&mut *gate)
            .await
            .unwrap();
        let holder: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *gate)
            .await
            .unwrap();
        sqlx::raw_sql("CREATE SEQUENCE rb_test_bootstrap_attempt; CREATE FUNCTION rb_test_bootstrap_collision() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF nextval('rb_test_bootstrap_attempt') = 1 THEN PERFORM pg_advisory_xact_lock(76432791); IF TG_ARGV[0] = 'rollback' THEN RAISE EXCEPTION 'owned bootstrap rollback'; END IF; END IF; RETURN NEW; END $$")
            .execute(&f.pool).await.unwrap();
        let action = if rollback_first { "rollback" } else { "commit" };
        sqlx::raw_sql(&format!("CREATE TRIGGER rb_test_bootstrap_collision AFTER INSERT ON tenant_bootstrap_requests FOR EACH ROW EXECUTE FUNCTION rb_test_bootstrap_collision('{action}')"))
            .execute(&f.pool).await.unwrap();
        let body = request();
        let first = post(&f, body.clone());
        let winner = waiter(&f, holder, "INSERT INTO tenant_bootstrap_requests", &first).await;
        let mut other = body.clone();
        if conflict {
            other["name"] = json!("a different concurrent request");
        }
        let second = post(&f, other);
        let loser = if let Some(winner) = winner {
            waiter(&f, winner, "INSERT INTO tenant_bootstrap_requests", &second).await
        } else {
            None
        };
        gate.commit().await.unwrap();
        let first = joined(first).await;
        let second = joined(second).await;
        sqlx::raw_sql("DROP TRIGGER rb_test_bootstrap_collision ON tenant_bootstrap_requests; DROP FUNCTION rb_test_bootstrap_collision(); DROP SEQUENCE rb_test_bootstrap_attempt")
            .execute(&f.pool).await.unwrap();
        eprintln!("collision rollback_first={rollback_first} conflict={conflict}: winner={winner:?} loser={loser:?} responses={first:?}/{second:?}");
        assert!(
            winner.is_some() && loser.is_some(),
            "require the actual unique-key wait dependency"
        );
        assert_eq!(first.0, if rollback_first { 500 } else { 200 });
        if rollback_first {
            assert_eq!(first.1["code"], "dependency_unavailable");
            assert_eq!(second.0, 200, "{second:?}");
            assert_eq!(second.1["replayed"], false);
        } else if conflict {
            assert_eq!(second.0, 409, "{second:?}");
            assert_eq!(second.1["code"], "idempotency_conflict");
        } else {
            let mut expected = first.1;
            expected["replayed"] = json!(true);
            assert_eq!(second, (200, expected));
        }
        assert_one_bootstrap(&before, &snapshot(&f.pool).await);
        assert_eq!(joined(post(&f, body)).await.1["replayed"], true);
    }
}

#[tokio::test]
async fn replay_guards_the_recorded_tenant_before_decoding_and_preserves_frozen_outcomes() {
    let Some(f) = fixture().await else { return };
    let body = request();
    let first = joined(post(&f, body.clone())).await;
    assert_eq!(first.0, 200);
    let tenant = first.1["tenant_id"].as_str().unwrap();
    let key = body["bootstrap_request_id"].as_str().unwrap();
    let mut holder = f.pool.begin().await.unwrap();
    sqlx::query(
        "SELECT tenant_id FROM tenant_authority_guards WHERE tenant_id = $1 FOR NO KEY UPDATE",
    )
    .bind(tenant)
    .execute(&mut *holder)
    .await
    .unwrap();
    let pid = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *holder)
        .await
        .unwrap();
    // Corrupt only this owned fixture. If the full outcome were decoded before
    // the recorded guard, the retry would fail immediately instead of waiting.
    sqlx::query("UPDATE tenant_bootstrap_requests SET outcome = '{}' WHERE request_id = $1")
        .bind(key)
        .execute(&f.pool)
        .await
        .unwrap();
    let before = snapshot(&f.pool).await;
    let retry = post(&f, body.clone());
    let blocked = waiter(&f, pid, "tenant_authority_guards", &retry).await;
    // An unrelated tenant remains usable while this replay is queued.
    let unrelated = joined(post(&f, request())).await;
    holder.rollback().await.unwrap();
    let result = joined(retry).await;
    let after = snapshot(&f.pool).await;
    sqlx::query("UPDATE tenant_bootstrap_requests SET outcome = $2 WHERE request_id = $1")
        .bind(key)
        .bind(&first.1)
        .execute(&f.pool)
        .await
        .unwrap();
    assert!(
        blocked.is_some(),
        "recorded tenant guard must precede decode"
    );
    assert_eq!(unrelated.0, 200);
    assert_eq!(
        result,
        (
            500,
            json!({"code":"dependency_unavailable","message":"internal server error"})
        )
    );
    assert_one_bootstrap(&before, &after);
    sqlx::query("UPDATE enrollment_boundaries SET status = 'revoked', expires_at = clock_timestamp() - interval '1 day' WHERE tenant_id = $1")
        .bind(tenant).execute(&f.pool).await.unwrap();
    sqlx::query("UPDATE authority_grants SET status = 'revoked' WHERE tenant_id = $1")
        .bind(tenant)
        .execute(&f.pool)
        .await
        .unwrap();
    let frozen = snapshot(&f.pool).await;
    let mut expected = first.1;
    expected["replayed"] = json!(true);
    assert_eq!(joined(post(&f, body)).await, (200, expected));
    assert_eq!(frozen, snapshot(&f.pool).await);
}

#[tokio::test]
async fn malformed_or_foreign_outcomes_refuse_without_replacement_bootstrap() {
    let Some(f) = fixture().await else { return };
    let body = request();
    let first = joined(post(&f, body.clone())).await;
    let foreign = joined(post(&f, request())).await;
    assert_eq!((first.0, foreign.0), (200, 200));
    let key = body["bootstrap_request_id"].as_str().unwrap();
    let mut cases = vec![json!({}), foreign.1.clone()];
    for field in [
        "tenant_id",
        "principal_id",
        "kind",
        "name",
        "boundary_id",
        "grant_id",
        "bootstrap_request_id",
        "replayed",
    ] {
        let mut missing = first.1.clone();
        missing.as_object_mut().unwrap().remove(field);
        cases.push(missing);
        let mut null = first.1.clone();
        null[field] = Value::Null;
        cases.push(null);
    }
    for (field, value) in [
        ("tenant_id", json!(TenantId::new())),
        (
            "principal_id",
            json!("hpr_00000000-0000-7000-8000-00000000000A"),
        ),
        ("kind", json!("role")),
        ("name", json!("changed")),
        ("boundary_id", foreign.1["boundary_id"].clone()),
        ("grant_id", foreign.1["grant_id"].clone()),
        ("bootstrap_request_id", json!(RequestId::new())),
        ("replayed", json!(true)),
        ("extra", json!(true)),
    ] {
        let mut changed = first.1.clone();
        changed[field] = value;
        cases.push(changed);
    }
    for (index, outcome) in cases.into_iter().enumerate() {
        sqlx::query("UPDATE tenant_bootstrap_requests SET outcome = $2 WHERE request_id = $1")
            .bind(key)
            .bind(outcome)
            .execute(&f.pool)
            .await
            .unwrap();
        let before = snapshot(&f.pool).await;
        let result = joined(post(&f, body.clone())).await;
        assert_eq!(result.0, 500, "malformed outcome {index}: {result:?}");
        assert_eq!(before, snapshot(&f.pool).await, "malformed outcome {index}");
    }
    sqlx::query("UPDATE tenant_bootstrap_requests SET outcome = $2, outcome_version = 2 WHERE request_id = $1")
        .bind(key).bind(&first.1).execute(&f.pool).await.unwrap();
    let before = snapshot(&f.pool).await;
    assert_eq!(joined(post(&f, body.clone())).await.0, 500);
    assert_eq!(before, snapshot(&f.pool).await);
    sqlx::query("UPDATE tenant_bootstrap_requests SET outcome_version = 1 WHERE request_id = $1")
        .bind(key)
        .execute(&f.pool)
        .await
        .unwrap();
    // A well-shaped outcome still cannot refer to a grant bound to another
    // tenant/subject/parent or an absent identity/enrollment.
    for (table, column, replacement) in [
        (
            "authority_grants",
            "tenant_id",
            foreign.1["tenant_id"].as_str().unwrap(),
        ),
        (
            "authority_grants",
            "subject_id",
            foreign.1["principal_id"].as_str().unwrap(),
        ),
        (
            "authority_grants",
            "boundary_id",
            foreign.1["boundary_id"].as_str().unwrap(),
        ),
        (
            "authority_grants",
            "issuer",
            foreign.1["principal_id"].as_str().unwrap(),
        ),
        ("human_principals", "name", "wrong-name"),
        ("enrollments", "name", "wrong-name"),
    ] {
        let (id_column, id) = if table == "authority_grants" {
            ("grant_id", first.1["grant_id"].as_str().unwrap())
        } else {
            ("principal_id", first.1["principal_id"].as_str().unwrap())
        };
        let original: String = sqlx::query_scalar(&format!(
            "SELECT {column} FROM {table} WHERE {id_column} = $1"
        ))
        .bind(id)
        .fetch_one(&f.pool)
        .await
        .unwrap();
        sqlx::query(&format!(
            "UPDATE {table} SET {column} = $2 WHERE {id_column} = $1"
        ))
        .bind(id)
        .bind(replacement)
        .execute(&f.pool)
        .await
        .unwrap();
        let before = snapshot(&f.pool).await;
        let result = joined(post(&f, body.clone())).await;
        let after = snapshot(&f.pool).await;
        sqlx::query(&format!(
            "UPDATE {table} SET {column} = $2 WHERE {id_column} = $1"
        ))
        .bind(id)
        .bind(original)
        .execute(&f.pool)
        .await
        .unwrap();
        assert_eq!(result.0, 500, "foreign {table}.{column}: {result:?}");
        assert_eq!(before, after);
    }
    // A legacy identity row can satisfy the FK without being a canonical route.
    // Raw owner mutation is a corruption control, never a supported API path.
    let legacy = format!("invalid-route-{}", RequestId::new());
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind(&legacy)
        .execute(&f.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE tenant_bootstrap_requests SET tenant_id = $2 WHERE request_id = $1")
        .bind(key)
        .bind(&legacy)
        .execute(&f.pool)
        .await
        .unwrap();
    let before = snapshot(&f.pool).await;
    let result = joined(post(&f, body.clone())).await;
    let after = snapshot(&f.pool).await;
    sqlx::query("UPDATE tenant_bootstrap_requests SET tenant_id = $2 WHERE request_id = $1")
        .bind(key)
        .bind(first.1["tenant_id"].as_str().unwrap())
        .execute(&f.pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM tenants WHERE tenant_id = $1")
        .bind(&legacy)
        .execute(&f.pool)
        .await
        .unwrap();
    assert_eq!(result.0, 500);
    assert_eq!(before, after);
    let mut expected = first.1;
    expected["replayed"] = json!(true);
    assert_eq!(joined(post(&f, body)).await, (200, expected));
}

#[tokio::test]
async fn receipt_insert_or_deferred_commit_failure_rolls_back_and_key_can_recover() {
    let Some(f) = fixture().await else { return };
    for deferred in [false, true] {
        let before = snapshot(&f.pool).await;
        sqlx::raw_sql("CREATE FUNCTION rb_test_bootstrap_failure() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned outcome failure'; END $$")
            .execute(&f.pool).await.unwrap();
        let trigger = if deferred {
            "CREATE CONSTRAINT TRIGGER rb_test_bootstrap_failure AFTER INSERT ON tenant_bootstrap_requests DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION rb_test_bootstrap_failure()"
        } else {
            "CREATE TRIGGER rb_test_bootstrap_failure BEFORE INSERT ON tenant_bootstrap_requests FOR EACH ROW EXECUTE FUNCTION rb_test_bootstrap_failure()"
        };
        sqlx::raw_sql(trigger).execute(&f.pool).await.unwrap();
        let body = request();
        let result = joined(post(&f, body.clone())).await;
        let after = snapshot(&f.pool).await;
        sqlx::raw_sql("DROP TRIGGER rb_test_bootstrap_failure ON tenant_bootstrap_requests; DROP FUNCTION rb_test_bootstrap_failure()")
            .execute(&f.pool).await.unwrap();
        assert_eq!(result.0, 500);
        assert_eq!(
            result.1["code"],
            if deferred {
                "commit_outcome_unconfirmed"
            } else {
                "dependency_unavailable"
            }
        );
        assert_eq!(
            before, after,
            "every provisional row including the key rolls back"
        );
        let recovered = joined(post(&f, body.clone())).await;
        assert_eq!(recovered.0, 200);
        assert_eq!(recovered.1["replayed"], false);
        assert_one_bootstrap(&before, &snapshot(&f.pool).await);
        assert_eq!(joined(post(&f, body)).await.1["replayed"], true);
    }
}

#[tokio::test]
async fn actual_unconfirmed_commit_is_recovered_by_the_same_key_without_another_tenant() {
    let Some(f) = fixture().await else { return };
    let before = snapshot(&f.pool).await;
    let body = request();
    sqlx::raw_sql("CREATE FUNCTION rb_test_keyed_body_delay() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_sleep(6); RETURN NEW; END $$; CREATE TRIGGER rb_test_keyed_body_delay BEFORE INSERT ON tenant_bootstrap_requests FOR EACH ROW EXECUTE FUNCTION rb_test_keyed_body_delay(); CREATE FUNCTION rb_test_keyed_commit_delay() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_sleep(9.5); RETURN NEW; END $$; CREATE CONSTRAINT TRIGGER rb_test_keyed_commit_delay AFTER INSERT ON tenant_bootstrap_requests DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION rb_test_keyed_commit_delay()")
        .execute(&f.pool).await.unwrap();
    let job = post(&f, body.clone());
    let committing = timeout(Duration::from_secs(10), async {
        loop {
            let pid: Option<i32> = sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname = current_database() AND query = 'COMMIT' AND wait_event = 'PgSleep' ORDER BY pid LIMIT 1")
                .fetch_optional(&f.pool).await.unwrap();
            if pid.is_some() || job.is_finished() { return pid; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.ok().flatten();
    let result = joined(job).await;
    let original = timeout(Duration::from_secs(5), async {
        loop {
            let outcome: Option<Value> = sqlx::query_scalar(
                "SELECT outcome FROM tenant_bootstrap_requests WHERE request_id = $1",
            )
            .bind(body["bootstrap_request_id"].as_str().unwrap())
            .fetch_optional(&f.pool)
            .await
            .unwrap();
            if let Some(outcome) = outcome {
                return outcome;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await;
    sqlx::raw_sql("DROP TRIGGER rb_test_keyed_body_delay ON tenant_bootstrap_requests; DROP FUNCTION rb_test_keyed_body_delay(); DROP TRIGGER rb_test_keyed_commit_delay ON tenant_bootstrap_requests; DROP FUNCTION rb_test_keyed_commit_delay()")
        .execute(&f.pool).await.unwrap();
    eprintln!("keyed actual COMMIT backend={committing:?}, response={result:?}, committed outcome={original:?}");
    assert!(committing.is_some());
    assert_eq!(result.0, 500);
    assert_eq!(result.1["code"], "commit_outcome_unconfirmed");
    let mut expected = original.unwrap();
    expected["replayed"] = json!(true);
    let committed = snapshot(&f.pool).await;
    assert_one_bootstrap(&before, &committed);
    assert_eq!(joined(post(&f, body)).await, (200, expected));
    assert_eq!(committed, snapshot(&f.pool).await);
}

#[tokio::test]
async fn no_key_and_existing_tenant_replay_keep_their_original_wire_contract() {
    let Some(f) = fixture().await else { return };
    let mut body = request();
    body.as_object_mut().unwrap().remove("bootstrap_request_id");
    let first = joined(post(&f, body.clone())).await;
    let second = joined(post(&f, body.clone())).await;
    assert_eq!((first.0, second.0), (200, 200));
    assert_ne!(first.1["tenant_id"], second.1["tenant_id"]);
    assert!(first.1.get("bootstrap_request_id").is_none());
    body["tenant_id"] = first.1["tenant_id"].clone();
    assert_eq!(
        joined(post(&f, body)).await,
        (
            200,
            json!({"tenant_id":first.1["tenant_id"], "principal_id":first.1["principal_id"], "kind":"human", "name":first.1["name"], "replayed":true})
        )
    );
}

#[tokio::test]
async fn a_mapping_removed_during_redirect_refuses_without_fresh_creation() {
    let Some(f) = fixture().await else { return };
    let body = request();
    let original = joined(post(&f, body.clone())).await;
    assert_eq!(original.0, 200);
    let key = body["bootstrap_request_id"].as_str().unwrap();
    let tenant = original.1["tenant_id"].as_str().unwrap();
    let mut holder = f.pool.begin().await.unwrap();
    sqlx::query(
        "SELECT tenant_id FROM tenant_authority_guards WHERE tenant_id = $1 FOR NO KEY UPDATE",
    )
    .bind(tenant)
    .execute(&mut *holder)
    .await
    .unwrap();
    let pid = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *holder)
        .await
        .unwrap();
    let retry = post(&f, body.clone());
    let blocked = waiter(&f, pid, "tenant_authority_guards", &retry).await;
    let receipt: Value = sqlx::query_scalar(
        "DELETE FROM tenant_bootstrap_requests r WHERE request_id = $1 RETURNING to_jsonb(r)",
    )
    .bind(key)
    .fetch_one(&f.pool)
    .await
    .unwrap();
    let before = snapshot(&f.pool).await;
    holder.rollback().await.unwrap();
    let result = joined(retry).await;
    let after = snapshot(&f.pool).await;
    sqlx::query("INSERT INTO tenant_bootstrap_requests SELECT * FROM jsonb_populate_record(NULL::tenant_bootstrap_requests, $1)")
        .bind(receipt).execute(&f.pool).await.unwrap();
    assert!(blocked.is_some());
    assert_eq!(
        result.0, 500,
        "immutable mapping disappearance must be an invariant failure"
    );
    assert_eq!(before, after);
    let mut expected = original.1;
    expected["replayed"] = json!(true);
    assert_eq!(joined(post(&f, body)).await, (200, expected));
}

#[tokio::test]
async fn redirect_uses_the_original_total_deadline_instead_of_starting_another_budget() {
    let Some(f) = fixture().await else { return };
    let before = snapshot(&f.pool).await;
    // Delay the first request in two separate statements, each below the 10s
    // ceiling. Its private fixture row is visible only to that transaction.
    // The same-key winner can finish while the first request is still asleep.
    sqlx::raw_sql("CREATE SEQUENCE rb_test_budget_order; CREATE TABLE rb_test_slow_bootstrap (tenant_id TEXT PRIMARY KEY); CREATE FUNCTION rb_test_budget_tenant() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF nextval('rb_test_budget_order') = 1 THEN INSERT INTO rb_test_slow_bootstrap VALUES (NEW.tenant_id); PERFORM pg_sleep(6); END IF; RETURN NEW; END $$; CREATE TRIGGER rb_test_budget_tenant AFTER INSERT ON tenants FOR EACH ROW EXECUTE FUNCTION rb_test_budget_tenant(); CREATE FUNCTION rb_test_budget_enrollment() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF EXISTS (SELECT 1 FROM rb_test_slow_bootstrap WHERE tenant_id = NEW.tenant_id) THEN PERFORM pg_sleep(6); END IF; RETURN NEW; END $$; CREATE TRIGGER rb_test_budget_enrollment BEFORE INSERT ON enrollments FOR EACH ROW EXECUTE FUNCTION rb_test_budget_enrollment()")
        .execute(&f.pool).await.unwrap();
    let body = request();
    let start = tokio::time::Instant::now();
    let retry = post(&f, body.clone());
    let sleeping = timeout(Duration::from_secs(4), async {
        loop {
            let pid: Option<i32> = sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname = current_database() AND query LIKE 'INSERT INTO tenants%' AND wait_event = 'PgSleep' ORDER BY pid LIMIT 1")
                .fetch_optional(&f.pool).await.unwrap();
            if pid.is_some() || retry.is_finished() { return pid; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.ok().flatten();
    let winner = joined(post(&f, body.clone())).await;
    let mut holder = f.pool.begin().await.unwrap();
    sqlx::query(
        "SELECT tenant_id FROM tenant_authority_guards WHERE tenant_id = $1 FOR NO KEY UPDATE",
    )
    .bind(winner.1["tenant_id"].as_str().unwrap_or_default())
    .execute(&mut *holder)
    .await
    .unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *holder)
        .await
        .unwrap();
    let blocked = timeout(Duration::from_secs(14), async {
        loop {
            let waiting: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_stat_activity WHERE datname = current_database() AND query LIKE '%tenant_authority_guards%' AND wait_event_type = 'Lock' AND $1 = ANY(pg_blocking_pids(pid)))")
                .bind(pid).fetch_one(&f.pool).await.unwrap();
            if waiting || retry.is_finished() { return waiting; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.unwrap_or(false);
    let result = joined(retry).await;
    let elapsed = start.elapsed();
    holder.rollback().await.unwrap();
    let after = snapshot(&f.pool).await;
    let residue: i64 = sqlx::query_scalar("SELECT count(*) FROM rb_test_slow_bootstrap")
        .fetch_one(&f.pool)
        .await
        .unwrap();
    sqlx::raw_sql("DROP TRIGGER rb_test_budget_tenant ON tenants; DROP FUNCTION rb_test_budget_tenant(); DROP TRIGGER rb_test_budget_enrollment ON enrollments; DROP FUNCTION rb_test_budget_enrollment(); DROP TABLE rb_test_slow_bootstrap; DROP SEQUENCE rb_test_budget_order")
        .execute(&f.pool).await.unwrap();
    eprintln!("shared bootstrap budget: sleeping={sleeping:?}, guarded_redirect={blocked}, elapsed={elapsed:?}, result={result:?}");
    assert!(sleeping.is_some() && blocked);
    assert_eq!(winner.0, 200);
    assert_eq!(result.0, 500);
    assert_eq!(
        result.1["code"], "dependency_unavailable",
        "no COMMIT was attempted by the losing request"
    );
    assert!(
        elapsed >= Duration::from_secs(14) && elapsed < Duration::from_millis(16_500),
        "one 15s budget, not 12s plus a new 5s lock ceiling: {elapsed:?}"
    );
    assert_eq!(residue, 0);
    assert_one_bootstrap(&before, &after);
    assert_eq!(joined(post(&f, body)).await.1["replayed"], true);
}

#[tokio::test]
async fn a_client_disconnect_during_commit_recovers_the_original_outcome() {
    let Some(f) = fixture().await else { return };
    let before = snapshot(&f.pool).await;
    // No server deadline failure here: lose the caller while a short COMMIT is
    // genuinely in flight, then recover solely from its pre-known request key.
    sqlx::raw_sql("CREATE FUNCTION rb_test_lost_ack() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_sleep(1); RETURN NEW; END $$; CREATE CONSTRAINT TRIGGER rb_test_lost_ack AFTER INSERT ON tenant_bootstrap_requests DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION rb_test_lost_ack()")
        .execute(&f.pool).await.unwrap();
    let body = request();
    let job = post(&f, body.clone());
    let committing = timeout(Duration::from_secs(4), async {
        loop {
            let pid: Option<i32> = sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname = current_database() AND query = 'COMMIT' AND wait_event = 'PgSleep' ORDER BY pid LIMIT 1")
                .fetch_optional(&f.pool).await.unwrap();
            if pid.is_some() || job.is_finished() { return pid; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.ok().flatten();
    job.abort();
    let cancelled = job.await;
    let original = timeout(Duration::from_secs(4), async {
        loop {
            let outcome: Option<Value> = sqlx::query_scalar(
                "SELECT outcome FROM tenant_bootstrap_requests WHERE request_id = $1",
            )
            .bind(body["bootstrap_request_id"].as_str().unwrap())
            .fetch_optional(&f.pool)
            .await
            .unwrap();
            if let Some(outcome) = outcome {
                return outcome;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await;
    sqlx::raw_sql("DROP TRIGGER rb_test_lost_ack ON tenant_bootstrap_requests; DROP FUNCTION rb_test_lost_ack()")
        .execute(&f.pool).await.unwrap();
    assert!(committing.is_some());
    assert!(cancelled.unwrap_err().is_cancelled());
    let mut expected = original.unwrap();
    expected["replayed"] = json!(true);
    let committed = snapshot(&f.pool).await;
    assert_one_bootstrap(&before, &committed);
    assert_eq!(joined(post(&f, body)).await, (200, expected));
    assert_eq!(committed, snapshot(&f.pool).await);
}
