//! Separate site authority, atomic registry/audit writes and ordered revocation.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Duration as StdDuration;

use chrono::{Duration, Utc};
use reasonbraid_core::{AgentRoleId, GrantSubject, HumanPrincipalId};
use reasonbraid_server::site_authority::{
    self as site, Action, Disable, Error, Reason, RegistryCommand, RegistryName, Scope,
};
use serde_json::{json, Value};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
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
    sqlx::raw_sql(
        "DROP TRIGGER IF EXISTS site_test_audit_gate ON public.site_audit; \
         DROP FUNCTION IF EXISTS public.site_test_audit_gate(); \
         DELETE FROM public.site_audit; DELETE FROM public.site_grants; DELETE FROM public.site_boundaries; \
         DELETE FROM public.region_pairs WHERE from_region LIKE 'site-test-%' OR to_region LIKE 'site-test-%'; \
         DELETE FROM public.site_regions WHERE region_id LIKE 'site-test-%'; \
         DELETE FROM public.adapter_allowlist WHERE adapter_id LIKE 'site-test-%'",
    ).execute(&pool).await.unwrap();
    Some(pool)
}

fn subject() -> GrantSubject {
    GrantSubject::Human(HumanPrincipalId::new())
}
fn name(value: &str) -> RegistryName {
    RegistryName::new(value).unwrap()
}
fn reason(value: &str) -> Reason {
    Reason::new(value).unwrap()
}
fn scope(actions: &[Action]) -> Scope {
    Scope {
        actions: actions.to_vec(),
        valid_from: Utc::now() - Duration::hours(2),
        expires_at: Utc::now() + Duration::hours(4),
    }
}
fn all_actions() -> Vec<Action> {
    vec![
        Action::RegistryInspect,
        Action::AdapterAllow,
        Action::AdapterRevoke,
        Action::RegionDeclare,
        Action::RegionPair,
        Action::RegionUnpair,
    ]
}
fn declare(value: &str) -> RegistryCommand {
    RegistryCommand::DeclareRegion {
        region: name(value),
        reason: reason("site service proof"),
    }
}

async fn provision(pool: &PgPool, subject: &GrantSubject, scope: &Scope) -> (String, String) {
    let boundary = site::issue_boundary(pool, scope, &reason("operator boundary"))
        .await
        .unwrap();
    let boundary_id = boundary.result["boundary_id"].as_str().unwrap().to_owned();
    let grant = site::issue_grant(
        pool,
        &boundary_id,
        subject,
        scope,
        &reason("operator grant"),
    )
    .await
    .unwrap();
    assert_eq!(boundary.result["issued_by"], "postgres");
    assert_eq!(grant.result["issued_by"], "postgres");
    let kind = match subject {
        GrantSubject::Human(_) => "human",
        GrantSubject::Role(_) => "role",
    };
    let subject_json = json!({"kind": kind, "id": subject.id_string()});
    assert_eq!(grant.result["subject"], subject_json);
    let decoded: GrantSubject = serde_json::from_value(grant.result["subject"].clone()).unwrap();
    assert_eq!(&decoded, subject);
    assert_eq!(serde_json::to_value(&decoded).unwrap(), subject_json);
    assert_eq!(
        audit(pool, &grant.audit_id).await["target"]["subject"],
        subject_json
    );
    (
        boundary_id,
        grant.result["grant_id"].as_str().unwrap().to_owned(),
    )
}

async fn audit(pool: &PgPool, id: &str) -> Value {
    sqlx::query_scalar("SELECT to_jsonb(a) FROM public.site_audit a WHERE audit_id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn refused(pool: &PgPool, result: Result<site::Receipt, Error>, expected: &str) -> Value {
    let Error::Refused { reason, audit_id } = result.unwrap_err() else {
        panic!("expected audited refusal")
    };
    assert_eq!(reason, expected);
    let record = audit(pool, &audit_id).await;
    assert_eq!(record["decision"], "denied");
    assert_eq!(record["outcome"], "denied");
    record
}

async fn registry_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.site_regions WHERE region_id LIKE 'site-test-%'",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

#[test]
fn malformed_commands_and_scopes_do_not_become_site_capabilities() {
    for value in ["", "  ", "line\nfeed"] {
        assert!(RegistryName::new(value).is_err());
        assert!(Reason::new(value).is_err());
    }
    assert!(RegistryName::new("x".repeat(257)).is_err());
    assert!(Reason::new("x".repeat(1025)).is_err());
    assert_eq!(
        name("vendor/adapter:v1 – régional").as_str(),
        "vendor/adapter:v1 – régional"
    );
    assert!(serde_json::from_value::<Action>(json!("tenant_admin")).is_err());
}

#[tokio::test]
async fn explicit_operator_service_preserves_effects_noops_reads_and_audit() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let actor = subject();
    let (boundary, grant) = provision(&pool, &actor, &scope(&all_actions())).await;
    let command = RegistryCommand::AllowAdapter {
        adapter_id: name("site-test-adapter"),
        reason: reason("first admission"),
    };
    let first = site::execute(&pool, &actor, &command).await.unwrap();
    let record = audit(&pool, &first.audit_id).await;
    assert_eq!(record["actor_kind"], "human");
    assert_eq!(record["actor"], actor.id_string());
    assert_eq!(record["action"], "adapter_allow");
    assert_eq!(record["target"], json!({"adapter_id": "site-test-adapter"}));
    assert_eq!(record["grant_id"], grant);
    assert_eq!(record["boundary_id"], boundary);
    assert_eq!(record["outcome"], "applied");
    assert_eq!(record["requested_reason"], "first admission");
    assert!(record["decided_at"].is_string());
    let again = site::execute(
        &pool,
        &actor,
        &RegistryCommand::AllowAdapter {
            adapter_id: name("site-test-adapter"),
            reason: reason("second attempt"),
        },
    )
    .await
    .unwrap();
    assert_eq!(audit(&pool, &again.audit_id).await["outcome"], "noop");
    assert_eq!(
        audit(&pool, &again.audit_id).await["requested_reason"],
        "second attempt"
    );
    let list = site::execute(&pool, &actor, &RegistryCommand::ListAdapters)
        .await
        .unwrap();
    let adapter = list.result["adapters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["adapter_id"] == "site-test-adapter")
        .unwrap();
    assert_eq!(adapter["reason"], "first admission");
    assert_eq!(adapter["added_by"], actor.id_string());
    assert_eq!(audit(&pool, &list.audit_id).await["outcome"], "inspected");

    for expected in ["applied", "noop"] {
        let revoked = site::execute(
            &pool,
            &actor,
            &RegistryCommand::RevokeAdapter {
                adapter_id: name("site-test-adapter"),
                reason: reason("retirement"),
            },
        )
        .await
        .unwrap();
        assert_eq!(audit(&pool, &revoked.audit_id).await["outcome"], expected);
    }
    for region in ["site-test-west", "site-test-east"] {
        for expected in ["applied", "noop"] {
            let declared = site::execute(&pool, &actor, &declare(region))
                .await
                .unwrap();
            assert_eq!(audit(&pool, &declared.audit_id).await["outcome"], expected);
        }
    }
    let pair = RegistryCommand::PairRegions {
        from: name("site-test-west"),
        to: name("site-test-east"),
        reason: reason("pair approved"),
    };
    for expected in ["applied", "noop"] {
        let paired = site::execute(&pool, &actor, &pair).await.unwrap();
        assert_eq!(audit(&pool, &paired.audit_id).await["outcome"], expected);
    }
    let listed = site::execute(&pool, &actor, &RegistryCommand::ListRegions)
        .await
        .unwrap();
    assert!(listed.result["pairs"]
        .as_array()
        .unwrap()
        .contains(&json!({"from": "site-test-west", "to": "site-test-east"})));
    for expected in ["applied", "noop"] {
        let unpaired = site::execute(
            &pool,
            &actor,
            &RegistryCommand::UnpairRegions {
                from: name("site-test-west"),
                to: name("site-test-east"),
                reason: reason("unpair approved"),
            },
        )
        .await
        .unwrap();
        assert_eq!(audit(&pool, &unpaired.audit_id).await["outcome"], expected);
    }
    let before = registry_count(&pool).await;
    let bad_pair = RegistryCommand::PairRegions {
        from: name("site-test-west"),
        to: name("site-test-missing"),
        reason: reason("invalid route"),
    };
    let denial = refused(
        &pool,
        site::execute(&pool, &actor, &bad_pair).await,
        "undeclared_region",
    )
    .await;
    assert_eq!(denial["grant_id"], grant);
    assert_eq!(registry_count(&pool).await, before);
}

#[tokio::test]
async fn role_subjects_and_invalid_issuance_have_explicit_contracts() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let actor = GrantSubject::Role(AgentRoleId::new());
    let valid = scope(&[Action::RegionDeclare]);
    for invalid in [
        Scope {
            actions: vec![],
            ..valid.clone()
        },
        Scope {
            expires_at: valid.valid_from,
            ..valid.clone()
        },
    ] {
        assert!(matches!(
            site::issue_boundary(&pool, &invalid, &reason("invalid scope")).await,
            Err(Error::InvalidInput(_))
        ));
    }
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.site_boundaries")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    provision(&pool, &actor, &valid).await;
    let receipt = site::execute(&pool, &actor, &declare("site-test-role"))
        .await
        .unwrap();
    let record = audit(&pool, &receipt.audit_id).await;
    assert_eq!(record["actor_kind"], "role");
    assert_eq!(record["actor"], actor.id_string());
    refused(
        &pool,
        site::execute(&pool, &subject(), &declare("site-test-unrelated-human")).await,
        "site_authority_required",
    )
    .await;
    assert_eq!(registry_count(&pool).await, 1);
}

async fn tenant_admin(pool: &PgPool) -> GrantSubject {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let router = reasonbraid_server::api_router(pool.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let response = reqwest::Client::new()
        .post(format!("http://{address}/v1/enrollments"))
        .json(&json!({"kind": "human", "name": format!("site-test-{}", uuid::Uuid::now_v7())}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let body: Value = response.json().await.unwrap();
    server.abort();
    assert!(server.await.unwrap_err().is_cancelled());
    GrantSubject::Human(body["principal_id"].as_str().unwrap().parse().unwrap())
}

#[tokio::test]
async fn tenant_enrollment_and_read_capabilities_confer_no_site_write_rights() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    for _ in 0..2 {
        let actor = tenant_admin(&pool).await;
        let record = refused(
            &pool,
            site::execute(&pool, &actor, &declare("site-test-tenant-denied")).await,
            "site_authority_required",
        )
        .await;
        assert_eq!(record["actor"], actor.id_string());
        assert!(record["grant_id"].is_null());
        assert_eq!(record["evaluation"], json!([]));
    }
    let grants: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.site_grants")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(grants, 0, "enrollment cannot create site grants");
    assert_eq!(registry_count(&pool).await, 0);
    let reader = subject();
    provision(&pool, &reader, &scope(&[Action::RegistryInspect])).await;
    site::execute(&pool, &reader, &RegistryCommand::ListRegions)
        .await
        .unwrap();
    refused(
        &pool,
        site::execute(&pool, &reader, &declare("site-test-reader-denied")).await,
        "site_authority_required",
    )
    .await;
    assert_eq!(registry_count(&pool).await, 0);
}

#[tokio::test]
async fn actual_parent_binding_and_every_usable_grant_are_checked() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let actor = subject();
    let scope = scope(&[Action::RegionDeclare]);
    let (old_boundary, old_grant) = provision(&pool, &actor, &scope).await;
    site::disable_boundary(
        &pool,
        &old_boundary,
        Disable::Revoke,
        &reason("old parent retired"),
    )
    .await
    .unwrap();
    let replacement = site::issue_boundary(&pool, &scope, &reason("separate new parent"))
        .await
        .unwrap();
    let new_boundary = replacement.result["boundary_id"].as_str().unwrap();
    refused(
        &pool,
        site::execute(&pool, &actor, &declare("site-test-old-parent")).await,
        "site_authority_required",
    )
    .await;
    assert_eq!(
        registry_count(&pool).await,
        0,
        "a different live boundary cannot rearm the old grant"
    );
    assert!(
        sqlx::query("UPDATE public.site_grants SET boundary_id = $2 WHERE grant_id = $1")
            .bind(&old_grant)
            .bind(new_boundary)
            .execute(&pool)
            .await
            .is_err()
    );
    assert!(sqlx::query(
        "UPDATE public.site_boundaries SET status = 'active' WHERE boundary_id = $1"
    )
    .bind(&old_boundary)
    .execute(&pool)
    .await
    .is_err());

    // A genuine new grant succeeds despite the older, revoked-parent candidate.
    let usable = site::issue_grant(
        &pool,
        new_boundary,
        &actor,
        &scope,
        &reason("explicit replacement grant"),
    )
    .await
    .unwrap();
    let applied = site::execute(&pool, &actor, &declare("site-test-new-parent"))
        .await
        .unwrap();
    assert_eq!(
        audit(&pool, &applied.audit_id).await["grant_id"],
        usable.result["grant_id"]
    );

    // A later, narrower grant must not mask the already usable one either.
    let read_scope = Scope {
        actions: vec![Action::RegistryInspect],
        ..scope.clone()
    };
    let read_boundary = site::issue_boundary(&pool, &read_scope, &reason("read ceiling"))
        .await
        .unwrap();
    site::issue_grant(
        &pool,
        read_boundary.result["boundary_id"].as_str().unwrap(),
        &actor,
        &read_scope,
        &reason("newer read grant"),
    )
    .await
    .unwrap();
    site::execute(&pool, &actor, &declare("site-test-not-shadowed"))
        .await
        .unwrap();

    let wider = Scope {
        actions: vec![Action::AdapterAllow],
        ..scope.clone()
    };
    refused(
        &pool,
        site::issue_grant(
            &pool,
            new_boundary,
            &actor,
            &wider,
            &reason("invalid widening"),
        )
        .await,
        "grant_exceeds_boundary",
    )
    .await;
    let longer = Scope {
        expires_at: scope.expires_at + Duration::hours(1),
        ..scope.clone()
    };
    refused(
        &pool,
        site::issue_grant(
            &pool,
            new_boundary,
            &actor,
            &longer,
            &reason("invalid lifetime"),
        )
        .await,
        "grant_exceeds_boundary",
    )
    .await;
}

#[tokio::test]
async fn suspended_revoked_future_and_expired_authority_refuses_effects() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    for boundary_case in [false, true] {
        for operation in [Disable::Suspend, Disable::Revoke] {
            let actor = subject();
            let (boundary, grant) =
                provision(&pool, &actor, &scope(&[Action::RegionDeclare])).await;
            if boundary_case {
                site::disable_boundary(&pool, &boundary, operation, &reason("disable parent"))
                    .await
                    .unwrap();
            } else {
                site::disable_grant(&pool, &grant, operation, &reason("disable grant"))
                    .await
                    .unwrap();
            }
            refused(
                &pool,
                site::execute(&pool, &actor, &declare("site-test-disabled")).await,
                "site_authority_required",
            )
            .await;
        }
    }
    for boundary_case in [false, true] {
        for future in [false, true] {
            let actor = subject();
            let wide = scope(&[Action::RegionDeclare]);
            let (boundary, grant) = provision(&pool, &actor, &wide).await;
            // Represent naturally aged/future records with explicit fixture
            // INSERTs, not a production clock override or an immutable-row update.
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
            site::disable_grant(
                &pool,
                &grant,
                Disable::Revoke,
                &reason("replace fixture source"),
            )
            .await
            .unwrap();
            let fixture_boundary = if boundary_case {
                let id = format!("sbd_{}", uuid::Uuid::now_v7());
                sqlx::query("INSERT INTO public.site_boundaries (boundary_id, issued_by, actions, valid_from, expires_at, status, reason) VALUES ($1,'fixture',ARRAY['region_declare'],$2,$3,'active','aged/future fixture')")
                    .bind(&id).bind(from).bind(until).execute(&pool).await.unwrap();
                id
            } else {
                boundary
            };
            sqlx::query("INSERT INTO public.site_grants (grant_id,boundary_id,issued_by,subject_kind,subject_id,actions,valid_from,expires_at,status,reason) VALUES ($1,$2,'fixture','human',$3,ARRAY['region_declare'],$4,$5,'active','aged/future fixture')")
                .bind(format!("sgr_{}", uuid::Uuid::now_v7())).bind(fixture_boundary).bind(actor.id_string()).bind(from).bind(until)
                .execute(&pool).await.unwrap();
            refused(
                &pool,
                site::execute(&pool, &actor, &declare("site-test-outside-window")).await,
                "site_authority_required",
            )
            .await;
        }
    }
    assert_eq!(registry_count(&pool).await, 0);
}

async fn role_pool(role: &str) -> PgPool {
    let url = std::env::var("DATABASE_URL").unwrap();
    let owner = pg_test_support::Ownership::read(
        &pg_test_support::repository_root().unwrap(),
        &url,
        &std::env::var("RB_TEST_CLUSTER").unwrap(),
        &std::env::var("RB_TEST_OWNER").unwrap(),
        &std::env::var("RB_TEST_DATABASE").unwrap(),
    )
    .unwrap();
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(StdDuration::from_secs(5))
        .after_connect(move |connection, _| {
            let owner = owner.clone();
            Box::pin(async move { owner.verify(connection).await })
        })
        .connect_with(PgConnectOptions::from_str(&url).unwrap().username(role))
        .await
        .unwrap()
}

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
    .expect("the intended database lock wait is observed");
}

const WAIT_GUARD: &str = "SELECT COUNT(*) FROM pg_stat_activity WHERE datname = current_database() AND state = 'active' AND wait_event_type = 'Lock' AND query LIKE 'SELECT guard_id FROM public.site_authority_guard%'";
const WAIT_AUDIT: &str = "SELECT COUNT(*) FROM pg_stat_activity WHERE datname = current_database() AND state = 'active' AND wait_event = 'advisory' AND query LIKE 'INSERT INTO public.site_audit%'";

#[tokio::test]
async fn issuance_uses_database_identity_and_rechecks_membership_after_waiting() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    // Generated ASCII role names are local to this owned disposable cluster.
    let outsider = format!("site_test_{}", uuid::Uuid::now_v7().simple());
    let member = format!("site_test_{}", uuid::Uuid::now_v7().simple());
    sqlx::raw_sql(&format!("CREATE ROLE {outsider} LOGIN; CREATE ROLE {member} LOGIN; GRANT pg_read_all_settings TO {outsider}, {member}; DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'reasonbraid_site_operator') THEN CREATE ROLE reasonbraid_site_operator NOLOGIN; END IF; END; $$; GRANT reasonbraid_site_operator TO {member}; GRANT SELECT,INSERT,UPDATE ON public.site_authority_guard,public.site_boundaries,public.site_grants,public.site_audit TO {member}"))
        .execute(&pool).await.unwrap();
    let outsider_pool = role_pool(&outsider).await;
    let scope = scope(&[Action::RegionDeclare]);
    assert!(matches!(
        site::issue_boundary(&outsider_pool, &scope, &reason("unprivileged attempt")).await,
        Err(Error::OperatorRequired)
    ));
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.site_audit")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        count, 0,
        "no audit privilege means no attempted audit write"
    );
    sqlx::query(&format!("GRANT INSERT ON public.site_audit TO {outsider}"))
        .execute(&pool)
        .await
        .unwrap();
    let denied = refused(
        &pool,
        site::issue_boundary(
            &outsider_pool,
            &scope,
            &reason("auditable unprivileged attempt"),
        )
        .await,
        "operator_required",
    )
    .await;
    assert_eq!(denied["actor_kind"], "database");
    assert_eq!(denied["actor"], outsider);
    let member_pool = role_pool(&member).await;
    let issued = site::issue_boundary(
        &member_pool,
        &scope,
        &reason("explicit database membership"),
    )
    .await
    .unwrap();
    assert_eq!(issued.result["issued_by"], member);

    let mut blocker = pool.begin().await.unwrap();
    sqlx::query("SELECT guard_id FROM public.site_authority_guard FOR UPDATE")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    let task_pool = member_pool.clone();
    let pending = tokio::spawn(async move {
        site::issue_boundary(
            &task_pool,
            &scope,
            &reason("membership removed while queued"),
        )
        .await
    });
    wait_for(&pool, WAIT_GUARD).await;
    sqlx::query(&format!("REVOKE reasonbraid_site_operator FROM {member}"))
        .execute(&pool)
        .await
        .unwrap();
    let membership: bool =
        sqlx::query_scalar("SELECT pg_has_role($1::NAME,'reasonbraid_site_operator','MEMBER')")
            .bind(&member)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        !membership,
        "revocation is visible before releasing the issuance guard"
    );
    blocker.rollback().await.unwrap();
    refused(&pool, pending.await.unwrap(), "operator_required").await;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.site_boundaries")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        count, 1,
        "queued issuance must recheck the deployment permission"
    );
    outsider_pool.close().await;
    member_pool.close().await;
    sqlx::raw_sql(&format!(
        "DROP OWNED BY {outsider}, {member}; DROP ROLE {outsider}; DROP ROLE {member}"
    ))
    .execute(&pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn audit_failure_rolls_back_the_registry_effect() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let actor = subject();
    provision(&pool, &actor, &scope(&[Action::RegionDeclare])).await;
    sqlx::raw_sql("CREATE FUNCTION public.site_test_audit_gate() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.action = 'region_declare' THEN RAISE EXCEPTION 'injected audit failure'; END IF; RETURN NEW; END; $$; CREATE TRIGGER site_test_audit_gate BEFORE INSERT ON public.site_audit FOR EACH ROW EXECUTE FUNCTION public.site_test_audit_gate()")
        .execute(&pool).await.unwrap();
    assert!(matches!(
        site::execute(&pool, &actor, &declare("site-test-audit-rollback")).await,
        Err(Error::Sql(_))
    ));
    assert_eq!(registry_count(&pool).await, 0);
    let attempts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.site_audit WHERE action = 'region_declare'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(attempts, 0, "a failed audit cannot leave an allowed result");
    sqlx::raw_sql("DROP TRIGGER site_test_audit_gate ON public.site_audit; DROP FUNCTION public.site_test_audit_gate()")
        .execute(&pool).await.unwrap();
    site::execute(&pool, &actor, &declare("site-test-audit-rollback"))
        .await
        .unwrap();
    assert_eq!(registry_count(&pool).await, 1);
}

async fn revoke(pool: &PgPool, boundary: bool, id: &str) -> Result<site::Receipt, Error> {
    if boundary {
        site::disable_boundary(pool, id, Disable::Revoke, &reason("race revocation")).await
    } else {
        site::disable_grant(pool, id, Disable::Revoke, &reason("race revocation")).await
    }
}

#[tokio::test]
async fn revocation_and_registry_mutation_serialize_in_both_commit_orders() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    for boundary_case in [false, true] {
        for mutation_first in [false, true] {
            let actor = subject();
            let (boundary, grant) =
                provision(&pool, &actor, &scope(&[Action::RegionDeclare])).await;
            let id = if boundary_case { boundary } else { grant };
            let region = format!("site-test-race-{boundary_case}-{mutation_first}");
            let paused_action = if mutation_first {
                "region_declare"
            } else if boundary_case {
                "boundary_revoke"
            } else {
                "grant_revoke"
            };
            sqlx::raw_sql(&format!("CREATE FUNCTION public.site_test_audit_gate() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.action = '{paused_action}' AND NEW.outcome = 'applied' THEN PERFORM pg_advisory_xact_lock(8246271); END IF; RETURN NEW; END; $$; CREATE TRIGGER site_test_audit_gate BEFORE INSERT ON public.site_audit FOR EACH ROW EXECUTE FUNCTION public.site_test_audit_gate()"))
                .execute(&pool).await.unwrap();
            let mut blocker = pool.begin().await.unwrap();
            sqlx::query("SELECT pg_advisory_xact_lock(8246271)")
                .execute(&mut *blocker)
                .await
                .unwrap();
            let (first_pool, first_actor, first_id, first_region) =
                (pool.clone(), actor.clone(), id.clone(), region.clone());
            let first = tokio::spawn(async move {
                if mutation_first {
                    site::execute(&first_pool, &first_actor, &declare(&first_region)).await
                } else {
                    revoke(&first_pool, boundary_case, &first_id).await
                }
            });
            wait_for(&pool, WAIT_AUDIT).await;
            let (second_pool, second_actor, second_id, second_region) =
                (pool.clone(), actor, id, region.clone());
            let second = tokio::spawn(async move {
                if mutation_first {
                    revoke(&second_pool, boundary_case, &second_id).await
                } else {
                    site::execute(&second_pool, &second_actor, &declare(&second_region)).await
                }
            });
            wait_for(&pool, WAIT_GUARD).await;
            blocker.rollback().await.unwrap();
            let first = first.await.unwrap().unwrap();
            let first_record = audit(&pool, &first.audit_id).await;
            assert_eq!(first_record["outcome"], "applied");
            if mutation_first {
                let second = second.await.unwrap().unwrap();
                assert_eq!(audit(&pool, &second.audit_id).await["outcome"], "applied");
            } else {
                refused(&pool, second.await.unwrap(), "site_authority_required").await;
            }
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM public.site_regions WHERE region_id = $1)",
            )
            .bind(&region)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(
                exists, mutation_first,
                "only a mutation ordered before revocation survives"
            );
            sqlx::raw_sql("DROP TRIGGER site_test_audit_gate ON public.site_audit; DROP FUNCTION public.site_test_audit_gate()")
                .execute(&pool).await.unwrap();
        }
    }
}

#[tokio::test]
async fn validity_is_evaluated_after_the_guard_wait_using_database_time() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let actor = subject();
    let parent_scope = scope(&[Action::RegionDeclare]);
    let parent = site::issue_boundary(&pool, &parent_scope, &reason("clock control parent"))
        .await
        .unwrap();
    let database_now: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let expiry = database_now + Duration::seconds(3);
    let grant_scope = Scope {
        expires_at: expiry,
        ..parent_scope
    };
    site::issue_grant(
        &pool,
        parent.result["boundary_id"].as_str().unwrap(),
        &actor,
        &grant_scope,
        &reason("short-lived queued grant"),
    )
    .await
    .unwrap();
    let mut blocker = pool.begin().await.unwrap();
    sqlx::query("SELECT guard_id FROM public.site_authority_guard FOR UPDATE")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    let task_pool = pool.clone();
    let request = tokio::spawn(async move {
        site::execute(&task_pool, &actor, &declare("site-test-expired-in-queue")).await
    });
    wait_for(&pool, WAIT_GUARD).await;
    let started: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT xact_start FROM pg_stat_activity WHERE datname = current_database() AND wait_event_type = 'Lock' AND query LIKE 'SELECT guard_id FROM public.site_authority_guard%' LIMIT 1")
        .fetch_one(&pool).await.unwrap();
    assert!(
        started < expiry,
        "the command began while the grant was still live"
    );
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
    refused(&pool, request.await.unwrap(), "site_authority_required").await;
    assert_eq!(registry_count(&pool).await, 0);
}
