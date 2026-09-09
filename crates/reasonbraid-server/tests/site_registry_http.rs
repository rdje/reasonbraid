//! Live HTTP proof of distinct site authority and atomic registry outcomes.
#[path = "support/mod.rs"]
mod pg_test_support;
#[path = "support/site.rs"]
mod site_fixture;

use std::sync::OnceLock;
use std::time::Duration as StdDuration;

use chrono::{Duration, Utc};
use reasonbraid_core::{AgentRoleId, GrantSubject, HumanPrincipalId};
use reasonbraid_server::site_authority::{self as site, Action, Disable, Reason, Scope};
use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}
async fn pool() -> Option<PgPool> {
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
    sqlx::raw_sql("DROP TRIGGER IF EXISTS site_http_gate ON public.site_audit; DROP FUNCTION IF EXISTS public.site_http_gate(); DELETE FROM public.site_audit; DELETE FROM public.site_grants; DELETE FROM public.site_boundaries; DELETE FROM public.region_pairs WHERE from_region LIKE 'site-http-%' OR to_region LIKE 'site-http-%'; DELETE FROM public.site_regions WHERE region_id LIKE 'site-http-%'; DELETE FROM public.adapter_allowlist WHERE adapter_id LIKE 'site-http-%'")
        .execute(&pool).await.unwrap();
    Some(pool)
}

struct Server {
    base: String,
    client: reqwest::Client,
    handle: tokio::task::JoinHandle<()>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl Server {
    async fn start(pool: &PgPool) -> Self {
        let router = api_router(pool.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        Self {
            base,
            client: reqwest::Client::builder()
                .no_proxy()
                .timeout(StdDuration::from_secs(10))
                .build()
                .unwrap(),
            handle,
        }
    }
    async fn call(&self, actor: &str, path: &str, body: Option<Value>) -> Reply {
        call(&self.client, &self.base, actor, path, body).await
    }
    async fn human(&self) -> Value {
        let response = self
            .call(
                "",
                "/v1/enrollments",
                Some(json!({"kind":"human", "name":format!("site-http-{}", uuid::Uuid::now_v7())})),
            )
            .await;
        assert_eq!(response.status, 200, "{}", response.body);
        response.body
    }
    async fn stop(mut self) {
        self.handle.abort();
        // Await by reference because Drop also covers a panicking control.
        assert!((&mut self.handle)
            .await
            .as_ref()
            .is_err_and(|error| error.is_cancelled()));
    }
}

#[derive(Debug)]
struct Reply {
    status: u16,
    body: Value,
    audit: Option<String>,
}
async fn reply(response: reqwest::Response) -> Reply {
    let status = response.status().as_u16();
    let audit = response
        .headers()
        .get("x-reasonbraid-site-audit")
        .map(|value| value.to_str().unwrap().to_owned());
    let text = response.text().await.unwrap();
    Reply {
        status,
        body: serde_json::from_str(&text).unwrap_or_else(|_| json!({"raw":text})),
        audit,
    }
}
async fn call(
    client: &reqwest::Client,
    base: &str,
    actor: &str,
    path: &str,
    body: Option<Value>,
) -> Reply {
    let url = format!("{base}{path}");
    let mut request = match body {
        Some(body) => client.post(url).json(&body),
        None => client.get(url),
    };
    if !actor.is_empty() {
        request = request.header(PRINCIPAL_HEADER, actor);
    }
    reply(request.send().await.unwrap()).await
}
fn reason() -> Reason {
    Reason::new("HTTP authority control").unwrap()
}
fn subject() -> String {
    HumanPrincipalId::new().to_string()
}
async fn audit(pool: &PgPool, response: &Reply, expected: &str) -> Value {
    let id = if response.status == 200 {
        response.audit.as_deref().unwrap()
    } else {
        response.body["audit_id"].as_str().unwrap()
    };
    let record: Value =
        sqlx::query_scalar("SELECT to_jsonb(a) FROM public.site_audit a WHERE audit_id=$1")
            .bind(id)
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(record["outcome"], expected);
    record
}
async fn snapshot(pool: &PgPool) -> Value {
    sqlx::query_scalar("SELECT jsonb_build_object('adapters',(SELECT COALESCE(jsonb_agg(to_jsonb(a) ORDER BY adapter_id),'[]') FROM public.adapter_allowlist a),'regions',(SELECT COALESCE(jsonb_agg(to_jsonb(r) ORDER BY region_id),'[]') FROM public.site_regions r),'pairs',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY from_region,to_region),'[]') FROM public.region_pairs p))")
        .fetch_one(pool).await.unwrap()
}
fn requests() -> Vec<(&'static str, Option<Value>)> {
    vec![
        ("/v1/admin/adapters", None),
        ("/v1/admin/regions", None),
        (
            "/v1/admin/adapters",
            Some(json!({"adapter_id":"site-http-protected","reason":"attempt allow"})),
        ),
        (
            "/v1/admin/adapters/site-http-protected/revoke",
            Some(json!({"reason":"attempt revoke"})),
        ),
        (
            "/v1/admin/regions",
            Some(json!({"region":"site-http-protected","reason":"attempt declare"})),
        ),
        (
            "/v1/admin/regions/site-http-protected/pair/dev-local",
            Some(json!({"reason":"attempt pair"})),
        ),
        (
            "/v1/admin/regions/site-http-protected/unpair/dev-local",
            Some(json!({"reason":"attempt unpair"})),
        ),
    ]
}
async fn denied_all(server: &Server, pool: &PgPool, actor: &str) {
    let before = snapshot(pool).await;
    for (path, body) in requests() {
        let response = server.call(actor, path, body).await;
        assert_eq!(response.status, 403, "{path}: {response:?}");
        assert_eq!(response.body["code"], "site_authority_required");
        assert_eq!(audit(pool, &response, "denied").await["actor"], actor);
    }
    assert_eq!(
        snapshot(pool).await,
        before,
        "every refusal leaves registry state unchanged"
    );
}

#[tokio::test]
async fn every_verb_preserves_its_body_and_commits_the_attributable_outcome() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = Server::start(&pool).await;
    let human = server.human().await;
    let actor = human["principal_id"].as_str().unwrap();
    let (boundary, grant) = site_fixture::provision(&pool, actor, site_fixture::ALL).await;
    let operations = [
        (
            "/v1/admin/adapters",
            json!({"adapter_id":"site-http-adapter","reason":"first reason"}),
            json!({"adapter_id":"site-http-adapter","allowed":true}),
        ),
        (
            "/v1/admin/regions",
            json!({"region":"site-http-region","reason":"qualified region"}),
            json!({"region":"site-http-region","declared":true}),
        ),
        (
            "/v1/admin/regions/dev-local/pair/site-http-region",
            json!({"reason":"explicit pair"}),
            json!({"from":"dev-local","to":"site-http-region","paired":true}),
        ),
        (
            "/v1/admin/regions/dev-local/unpair/site-http-region",
            json!({"reason":"remove pair"}),
            json!({"from":"dev-local","to":"site-http-region","unpaired":true}),
        ),
    ];
    for (path, body, expected) in operations {
        for outcome in ["applied", "noop"] {
            let mut body = body.clone();
            if outcome == "noop" {
                body["reason"] = json!("retry reason");
            }
            let response = server.call(actor, path, Some(body.clone())).await;
            assert_eq!(response.status, 200, "{response:?}");
            assert_eq!(response.body, expected);
            let row = audit(&pool, &response, outcome).await;
            assert_eq!(row["actor"], actor);
            assert_eq!(row["actor_kind"], "human");
            assert_eq!(row["grant_id"], grant);
            assert_eq!(row["boundary_id"], boundary);
            assert_eq!(row["requested_reason"], body["reason"]);
        }
    }
    let adapters = server.call(actor, "/v1/admin/adapters", None).await;
    assert_eq!(adapters.status, 200);
    audit(&pool, &adapters, "inspected").await;
    let row = adapters.body["adapters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["adapter_id"] == "site-http-adapter")
        .unwrap();
    assert_eq!(row["reason"], "first reason");
    assert_eq!(row["added_by"], actor);
    let regions = server.call(actor, "/v1/admin/regions", None).await;
    assert_eq!(regions.status, 200);
    audit(&pool, &regions, "inspected").await;
    assert!(regions.body["regions"]
        .as_array()
        .unwrap()
        .contains(&json!("site-http-region")));
    assert!(regions.body["pairs"].is_array());
    for outcome in ["applied", "noop"] {
        let response = server
            .call(
                actor,
                "/v1/admin/adapters/site-http-adapter/revoke",
                Some(json!({"reason":"retired"})),
            )
            .await;
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body,
            json!({"adapter_id":"site-http-adapter","revoked":true})
        );
        audit(&pool, &response, outcome).await;
    }
    server.stop().await;
}

#[tokio::test]
async fn two_tenant_admins_cannot_escalate_even_after_boundary_freeze_or_enrollment() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = Server::start(&pool).await;
    let first = server.human().await;
    let second = server.human().await;
    assert_ne!(first["tenant_id"], second["tenant_id"]);
    for human in [first, second] {
        let actor = human["principal_id"].as_str().unwrap();
        denied_all(&server, &pool, actor).await;
        let boundary: String =
            sqlx::query_scalar("SELECT boundary_id FROM enrollment_boundaries WHERE tenant_id=$1")
                .bind(human["tenant_id"].as_str().unwrap())
                .fetch_one(&pool)
                .await
                .unwrap();
        let frozen = server
            .call(
                actor,
                &format!("/v1/admin/boundaries/{boundary}/revoke"),
                Some(json!({"tenant_id":human["tenant_id"],"reason":"freeze tenant"})),
            )
            .await;
        assert_eq!(frozen.status, 200, "{frozen:?}");
        denied_all(&server, &pool, actor).await;
    }
    let human = server.human().await;
    let forged=server.call("","/v1/enrollments",Some(json!({"kind":"role","name":"forged operator","tenant_id":human["tenant_id"],"actions":["region_declare"]}))).await;
    assert!((400..500).contains(&forged.status), "{forged:?}");
    let counts:(i64,i64)=sqlx::query_as("SELECT (SELECT COUNT(*) FROM public.site_grants),(SELECT COUNT(*) FROM public.site_boundaries)").fetch_one(&pool).await.unwrap();
    assert_eq!(counts, (0, 0), "enrollment cannot mint site authority");
    server.stop().await;
}

#[tokio::test]
async fn role_actions_are_independent_and_a_live_candidate_survives_a_disabled_sibling() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = Server::start(&pool).await;
    let role = AgentRoleId::new().to_string();
    site_fixture::provision(&pool, &role, &[Action::RegionDeclare]).await;
    let inspect = server.call(&role, "/v1/admin/regions", None).await;
    assert_eq!(inspect.status, 403);
    audit(&pool, &inspect, "denied").await;
    let (_, sibling) = site_fixture::provision(&pool, &role, &[Action::RegionDeclare]).await;
    site::disable_grant(&pool, &sibling, Disable::Revoke, &reason())
        .await
        .unwrap();
    let write = server
        .call(
            &role,
            "/v1/admin/regions",
            Some(json!({"region":"site-http-role","reason":"role action"})),
        )
        .await;
    assert_eq!(write.status, 200, "{write:?}");
    assert_eq!(audit(&pool, &write, "applied").await["actor_kind"], "role");
    let reader = subject();
    site_fixture::provision(&pool, &reader, &[Action::RegistryInspect]).await;
    for (path, body) in requests() {
        let read = body.is_none();
        let before = snapshot(&pool).await;
        let response = server.call(&reader, path, body).await;
        assert_eq!(
            response.status,
            if read { 200 } else { 403 },
            "{response:?}"
        );
        audit(&pool, &response, if read { "inspected" } else { "denied" }).await;
        assert_eq!(snapshot(&pool).await, before);
    }
    server.stop().await;
}

#[tokio::test]
async fn inactive_or_out_of_window_grants_and_actual_parents_refuse_every_route() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = Server::start(&pool).await;
    for boundary_case in [false, true] {
        for operation in [Disable::Suspend, Disable::Revoke] {
            let actor = subject();
            let (boundary, grant) = site_fixture::provision(&pool, &actor, site_fixture::ALL).await;
            if boundary_case {
                site::disable_boundary(&pool, &boundary, operation, &reason())
                    .await
                    .unwrap();
            } else {
                site::disable_grant(&pool, &grant, operation, &reason())
                    .await
                    .unwrap();
            }
            denied_all(&server, &pool, &actor).await;
        }
        for future in [false, true] {
            let actor = subject();
            let (parent, grant) = site_fixture::provision(&pool, &actor, site_fixture::ALL).await;
            site::disable_grant(&pool, &grant, Disable::Revoke, &reason())
                .await
                .unwrap();
            let from = if future {
                Utc::now() + Duration::hours(1)
            } else {
                Utc::now() - Duration::hours(1)
            };
            let until = if future {
                Utc::now() + Duration::hours(2)
            } else {
                Utc::now() - Duration::minutes(1)
            };
            let actions: Vec<_> = site_fixture::ALL
                .iter()
                .map(|action| action.as_str())
                .collect();
            // Explicit aged/future storage fixtures avoid production clock injection
            // and leave immutable issuance facts untouched.
            let parent = if boundary_case {
                let id = format!("sbd_{}", uuid::Uuid::now_v7());
                sqlx::query("INSERT INTO public.site_boundaries (boundary_id,issued_by,actions,valid_from,expires_at,status,reason) VALUES ($1,'fixture',$2,$3,$4,'active','aged/future HTTP fixture')").bind(&id).bind(&actions).bind(from).bind(until).execute(&pool).await.unwrap();
                id
            } else {
                parent
            };
            let (child_from, child_until) = if boundary_case {
                (
                    Utc::now() - Duration::hours(2),
                    Utc::now() + Duration::hours(4),
                )
            } else {
                (from, until)
            };
            sqlx::query("INSERT INTO public.site_grants (grant_id,boundary_id,issued_by,subject_kind,subject_id,actions,valid_from,expires_at,status,reason) VALUES ($1,$2,'fixture','human',$3,$4,$5,$6,'active','aged/future HTTP fixture')").bind(format!("sgr_{}",uuid::Uuid::now_v7())).bind(&parent).bind(&actor).bind(&actions).bind(child_from).bind(child_until).execute(&pool).await.unwrap();
            denied_all(&server, &pool, &actor).await;
        }
    }
    server.stop().await;
}

#[tokio::test]
async fn malformed_domain_and_storage_failures_keep_distinct_status_and_audit_semantics() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = Server::start(&pool).await;
    let actor = subject();
    site_fixture::provision(&pool, &actor, site_fixture::ALL).await;
    let before = snapshot(&pool).await;
    let audit_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.site_audit")
        .fetch_one(&pool)
        .await
        .unwrap();
    for (path, body) in [
        ("/v1/admin/adapters", json!({"adapter_id":"site-http-bad"})),
        ("/v1/admin/adapters", json!({"adapter_id":" ","reason":"x"})),
        (
            "/v1/admin/adapters",
            json!({"adapter_id":"x".repeat(257),"reason":"x"}),
        ),
        (
            "/v1/admin/regions",
            json!({"region":"site-http-bad","reason":"x".repeat(1025)}),
        ),
        (
            "/v1/admin/regions",
            json!({"region":"site-http-bad","reason":"line\nfeed"}),
        ),
        (
            "/v1/admin/regions",
            json!({"region":"site-http-bad","reason":" ","extra":"secret-input-marker"}),
        ),
        (
            "/v1/admin/regions",
            json!({"region":"site-http-bad","reason":"x","extra":"secret-input-marker"}),
        ),
        ("/v1/admin/adapters/site-http-bad/revoke", json!({})),
        ("/v1/admin/adapters/%20/revoke", json!({"reason":"x"})),
        (
            "/v1/admin/regions/%20/pair/dev-local",
            json!({"reason":"x"}),
        ),
        (
            "/v1/admin/regions/dev-local/unpair/site-http-bad",
            json!({}),
        ),
    ] {
        let response = server.call(&actor, path, Some(body)).await;
        assert_eq!(response.status, 400, "{response:?}");
        assert_eq!(response.body["code"], "invalid_command");
        assert!(response.body.get("audit_id").is_none());
        assert!(response.audit.is_none());
        assert!(!response.body.to_string().contains("secret-input-marker"));
    }
    let malformed = reply(
        server
            .client
            .post(format!("{}/v1/admin/regions", server.base))
            .header(PRINCIPAL_HEADER, &actor)
            .header("content-type", "application/json")
            .body("{secret-input-marker")
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(malformed.status, 400);
    assert_eq!(malformed.body["code"], "invalid_command");
    let missing_identity = server.call("", "/v1/admin/regions", None).await;
    assert_eq!(missing_identity.status, 401);
    let audit_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.site_audit")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(audit_after, audit_before);
    assert_eq!(snapshot(&pool).await, before);
    let domain = server
        .call(
            &actor,
            "/v1/admin/regions/dev-local/pair/site-http-missing",
            Some(json!({"reason":"domain refusal"})),
        )
        .await;
    assert_eq!(domain.status, 400);
    assert_eq!(domain.body["code"], "undeclared_region");
    audit(&pool, &domain, "denied").await;
    sqlx::raw_sql("CREATE FUNCTION public.site_http_gate() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'secret-database-marker'; END; $$; CREATE TRIGGER site_http_gate BEFORE INSERT ON public.site_audit FOR EACH ROW EXECUTE FUNCTION public.site_http_gate()")
        .execute(&pool).await.unwrap();
    for (path, body) in [
        (
            "/v1/admin/regions",
            Some(json!({"region":"site-http-rollback","reason":"audit rollback"})),
        ),
        (
            "/v1/admin/regions/dev-local/pair/site-http-missing",
            Some(json!({"reason":"failed denial audit"})),
        ),
        ("/v1/admin/adapters", None),
    ] {
        let failed = server.call(&actor, path, body).await;
        assert_eq!(failed.status, 500, "{failed:?}");
        assert_eq!(failed.body["code"], "dependency_unavailable");
        assert!(failed.body.get("audit_id").is_none());
        assert!(failed.audit.is_none());
        assert!(!failed.body.to_string().contains("secret-database-marker"));
    }
    assert_eq!(snapshot(&pool).await, before);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.site_audit")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, audit_before + 1);
    sqlx::raw_sql(
        "DROP TRIGGER site_http_gate ON public.site_audit; DROP FUNCTION public.site_http_gate()",
    )
    .execute(&pool)
    .await
    .unwrap();
    let recovered = server
        .call(
            &actor,
            "/v1/admin/regions",
            Some(json!({"region":"site-http-rollback","reason":"recovery"})),
        )
        .await;
    assert_eq!(recovered.status, 200);
    audit(&pool, &recovered, "applied").await;
    server.stop().await;
}

const WAIT_GUARD:&str="SELECT COUNT(*) FROM pg_stat_activity WHERE datname=current_database() AND state='active' AND wait_event_type='Lock' AND query LIKE 'SELECT guard_id FROM public.site_authority_guard%'";
const WAIT_AUDIT:&str="SELECT COUNT(*) FROM pg_stat_activity WHERE datname=current_database() AND state='active' AND wait_event='advisory' AND query LIKE 'INSERT INTO public.site_audit%'";
async fn wait_for(pool: &PgPool, query: &str) {
    tokio::time::timeout(StdDuration::from_secs(3), async {
        loop {
            let count: i64 = sqlx::query_scalar(query).fetch_one(pool).await.unwrap();
            if count > 0 {
                break;
            }
            tokio::time::sleep(StdDuration::from_millis(10)).await;
        }
    })
    .await
    .expect("the intended live database wait is observed");
}
async fn revoke(pool: &PgPool, boundary: bool, id: &str) -> site::Receipt {
    if boundary {
        site::disable_boundary(pool, id, Disable::Revoke, &reason())
            .await
            .unwrap()
    } else {
        site::disable_grant(pool, id, Disable::Revoke, &reason())
            .await
            .unwrap()
    }
}

#[tokio::test]
async fn http_effect_and_operator_revocation_serialize_in_both_orders() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = Server::start(&pool).await;
    for boundary_case in [false, true] {
        for mutation_first in [false, true] {
            let actor = subject();
            let (boundary, grant) =
                site_fixture::provision(&pool, &actor, &[Action::RegionDeclare]).await;
            let id = if boundary_case { boundary } else { grant };
            let region = format!("site-http-race-{boundary_case}-{mutation_first}");
            let paused = if mutation_first {
                "region_declare"
            } else if boundary_case {
                "boundary_revoke"
            } else {
                "grant_revoke"
            };
            sqlx::raw_sql(&format!("CREATE FUNCTION public.site_http_gate() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.action='{paused}' AND NEW.outcome='applied' THEN PERFORM pg_advisory_xact_lock(8246272); END IF; RETURN NEW; END; $$; CREATE TRIGGER site_http_gate BEFORE INSERT ON public.site_audit FOR EACH ROW EXECUTE FUNCTION public.site_http_gate()"))
                .execute(&pool).await.unwrap();
            let mut blocker = pool.begin().await.unwrap();
            sqlx::query("SELECT pg_advisory_xact_lock(8246272)")
                .execute(&mut *blocker)
                .await
                .unwrap();
            let write = || {
                let (client, base, actor, region) = (
                    server.client.clone(),
                    server.base.clone(),
                    actor.clone(),
                    region.clone(),
                );
                tokio::spawn(async move {
                    call(
                        &client,
                        &base,
                        &actor,
                        "/v1/admin/regions",
                        Some(json!({"region":region,"reason":"concurrent HTTP write"})),
                    )
                    .await
                })
            };
            let disable = || {
                let (pool, id) = (pool.clone(), id.clone());
                tokio::spawn(async move { revoke(&pool, boundary_case, &id).await })
            };
            let (write_task, revoke_task) = if mutation_first {
                let write_task = write();
                wait_for(&pool, WAIT_AUDIT).await;
                let revoke_task = disable();
                wait_for(&pool, WAIT_GUARD).await;
                (write_task, revoke_task)
            } else {
                let revoke_task = disable();
                wait_for(&pool, WAIT_AUDIT).await;
                let write_task = write();
                wait_for(&pool, WAIT_GUARD).await;
                (write_task, revoke_task)
            };
            blocker.rollback().await.unwrap();
            let response = write_task.await.unwrap();
            let revoked = revoke_task.await.unwrap();
            assert_eq!(revoked.result["status"], "revoked");
            assert_eq!(
                response.status,
                if mutation_first { 200 } else { 403 },
                "{response:?}"
            );
            audit(
                &pool,
                &response,
                if mutation_first { "applied" } else { "denied" },
            )
            .await;
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM public.site_regions WHERE region_id=$1)",
            )
            .bind(&region)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(exists, mutation_first);
            sqlx::raw_sql("DROP TRIGGER site_http_gate ON public.site_audit; DROP FUNCTION public.site_http_gate()").execute(&pool).await.unwrap();
        }
    }
    server.stop().await;
}

#[tokio::test]
async fn http_grant_expiry_is_rechecked_after_the_guard_wait() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = Server::start(&pool).await;
    let actor = subject();
    let scope = Scope {
        actions: vec![Action::RegionDeclare],
        valid_from: Utc::now() - Duration::hours(1),
        expires_at: Utc::now() + Duration::hours(1),
    };
    let parent = site::issue_boundary(&pool, &scope, &reason())
        .await
        .unwrap();
    let now: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let expiry = now + Duration::seconds(3);
    let scope = Scope {
        expires_at: expiry,
        ..scope
    };
    site::issue_grant(
        &pool,
        parent.result["boundary_id"].as_str().unwrap(),
        &GrantSubject::Human(actor.parse().unwrap()),
        &scope,
        &reason(),
    )
    .await
    .unwrap();
    let mut blocker = pool.begin().await.unwrap();
    sqlx::query("SELECT guard_id FROM public.site_authority_guard FOR UPDATE")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    let (client, base) = (server.client.clone(), server.base.clone());
    let write = tokio::spawn(async move {
        call(
            &client,
            &base,
            &actor,
            "/v1/admin/regions",
            Some(json!({"region":"site-http-expired-in-queue","reason":"queued expiry"})),
        )
        .await
    });
    wait_for(&pool, WAIT_GUARD).await;
    let started:chrono::DateTime<Utc>=sqlx::query_scalar("SELECT xact_start FROM pg_stat_activity WHERE datname=current_database() AND wait_event_type='Lock' AND query LIKE 'SELECT guard_id FROM public.site_authority_guard%' LIMIT 1").fetch_one(&pool).await.unwrap();
    assert!(started < expiry);
    loop {
        let now: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .unwrap();
        if now >= expiry {
            break;
        }
        tokio::time::sleep(StdDuration::from_millis(20)).await;
    }
    blocker.rollback().await.unwrap();
    let response = write.await.unwrap();
    assert_eq!(response.status, 403);
    audit(&pool, &response, "denied").await;
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.site_regions WHERE region_id='site-http-expired-in-queue'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 0);
    server.stop().await;
}

#[tokio::test]
async fn wire_extraction_errors_have_safe_typed_responses_without_a_site_audit() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = Server::start(&pool).await;
    let actor = subject();
    site_fixture::provision(&pool, &actor, site_fixture::ALL).await;
    let before = snapshot(&pool).await;
    let count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.site_audit")
        .fetch_one(&pool)
        .await
        .unwrap();
    let cases = [
        ("/v1/admin/regions", None, "{}".to_owned(), 415),
        (
            "/v1/admin/regions",
            Some("application/json"),
            "x".repeat(2 * 1024 * 1024 + 1),
            413,
        ),
        (
            "/v1/admin/adapters/%FF/revoke",
            Some("application/json"),
            "{\"reason\":\"wire control\"}".to_owned(),
            400,
        ),
        (
            "/v1/admin/regions/%FF/pair/dev-local",
            Some("application/json"),
            "{\"reason\":\"wire control\"}".to_owned(),
            400,
        ),
        (
            "/v1/admin/regions/dev-local/unpair/%FF",
            Some("application/json"),
            "{\"reason\":\"wire control\"}".to_owned(),
            400,
        ),
    ];
    for (path, content_type, body, status) in cases {
        let mut request = server
            .client
            .post(format!("{}{path}", server.base))
            .header(PRINCIPAL_HEADER, &actor)
            .body(body);
        if let Some(content_type) = content_type {
            request = request.header("Content-Type", content_type);
        }
        let response = reply(request.send().await.unwrap()).await;
        assert_eq!(response.status, status, "{path}: {response:?}");
        assert_eq!(
            response.body["code"], "invalid_command",
            "{path}: {response:?}"
        );
        assert!(response.body.get("audit_id").is_none());
        assert!(response.audit.is_none());
    }
    assert_eq!(snapshot(&pool).await, before);
    let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.site_audit")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_before, count_after);
    server.stop().await;
}
