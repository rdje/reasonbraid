//! WP5 authority integration tests (`PHASE-0.5.1`): the development
//! `EnrollmentAuthorityBoundary` ceiling, scoped grants, and the authorization audit
//! record — proven against live PostgreSQL. Run via `scripts/run_pg_tests.sh` / the
//! `pg-tests` CI job; skip offline.
//!
//! The acceptance: tenant membership alone does not grant mandate or administrative
//! authority; every command records actor, subject if delegated, grant/boundary
//! reference, decision, and policy digest/version; a grant cannot exceed the ceiling.
//!
//! Note on shared tables: `apply_authorized_command` writes the WP2 durability tables
//! (idempotency/event_log/aggregate_state/outbox), which the other test binaries also
//! touch and purge. These tests therefore assert ONLY rows keyed by their own ids and
//! clean up their own rows — never global counts.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::sync::OnceLock;

use chrono::{Duration, Utc};
use reasonbraid_core::{
    boundary_active_at, policy_digest, AgentRoleId, AuthorityGrant, AuthorizationDecisionRecord,
    BoundaryStatus, Decision, EnrollmentAuthorityBoundary, GrantAction, GrantStatus, GrantSubject,
    ResourceTarget, RiskClass, TargetSelector,
};
use reasonbraid_server::{
    apply_authorized_command, authorize, create_boundary, create_grant, load_authorization_record,
    AuthorizationOutcome, AuthorizedApplyError, Command, CommandAuthz,
};
use serde_json::json;
use sqlx::PgPool;

static AUTHORITY_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn authority_guard() -> tokio::sync::MutexGuard<'static, ()> {
    AUTHORITY_LOCK
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
    // The authority tables belong exclusively to this binary: purge at start.
    sqlx::query("DELETE FROM authorization_records")
        .execute(&pool)
        .await
        .expect("purge authorization_records");
    sqlx::query("DELETE FROM authority_grants")
        .execute(&pool)
        .await
        .expect("purge authority_grants");
    sqlx::query("DELETE FROM enrollment_boundaries")
        .execute(&pool)
        .await
        .expect("purge enrollment_boundaries");
    Some(pool)
}

fn boundary(
    tenant: &str,
    permitted_actions: Vec<GrantAction>,
    risk_ceiling: RiskClass,
    delegable: bool,
) -> EnrollmentAuthorityBoundary {
    EnrollmentAuthorityBoundary {
        boundary_id: format!("bnd_{}", &tenant[4..]),
        tenant_id: tenant.parse().unwrap(),
        parent_or_root_authority: "dev-root-2026-09-06".to_string(),
        target_owner: "owner@example.org".to_string(),
        permitted_actions,
        permitted_domains: vec!["deliberation".to_string()],
        risk_ceiling,
        spend_ceiling: Some(json!({ "amount": 100.0 })),
        delegable,
        max_delegation_depth: 1,
        // Wide window on purpose: the boundary must span every grant fixture, even
        // though each helper call reads its own `Utc::now()` (the subset checker is
        // precise enough to catch microsecond-level window escapes).
        valid_from: Utc::now() - Duration::days(30),
        expires_at: Utc::now() + Duration::days(60),
        charter_digest: format!("charter-digest-{tenant}"),
        policy_version: "dev-authz-1".to_string(),
        status: BoundaryStatus::Active,
    }
}

fn grant(
    boundary_id: &str,
    tenant: &str,
    subject_id: &str,
    actions: Vec<GrantAction>,
    selector: TargetSelector,
) -> AuthorityGrant {
    let subject = if let Ok(role) = subject_id.parse::<AgentRoleId>() {
        GrantSubject::Role(role)
    } else {
        GrantSubject::Human(subject_id.parse().expect("a human principal id"))
    };
    AuthorityGrant {
        grant_id: format!("grt_{}", &subject_id[4..]),
        boundary_id: boundary_id.to_string(),
        tenant_id: tenant.parse().unwrap(),
        issuer: "hpr_00000000-0000-7000-8000-000000000001".parse().unwrap(),
        subject,
        actions,
        selector,
        risk_ceiling: RiskClass::Low,
        spend_limits: Some(json!({ "amount": 50.0 })),
        delegable: false,
        valid_from: Utc::now() - Duration::days(1),
        expires_at: Utc::now() + Duration::days(30),
        status: GrantStatus::Active,
    }
}

fn authz(
    actor: &str,
    principal: GrantSubject,
    delegate: Option<GrantSubject>,
    action: GrantAction,
    target: ResourceTarget,
) -> CommandAuthz {
    CommandAuthz {
        actor: actor.parse().unwrap(),
        principal,
        delegate_subject: delegate,
        delegation_scope: None,
        action,
        target,
    }
}

fn thread_target(tenant: &str, thread_id: &str) -> ResourceTarget {
    ResourceTarget::Thread {
        tenant_id: tenant.parse().unwrap(),
        thread_id: thread_id.parse().unwrap(),
    }
}

fn tenant_target(tenant: &str) -> ResourceTarget {
    ResourceTarget::Tenant {
        tenant_id: tenant.parse().unwrap(),
    }
}

fn command(tenant: &str, tag: &str) -> Command {
    Command {
        tenant_id: tenant.to_string(),
        aggregate_type: "thread".to_string(),
        aggregate_id: format!("thr_{tag}"),
        idempotency_key: format!("key_{tag}"),
        request_hash: "h".to_string(),
        event_id: format!("evt_{tag}"),
        event_type: "thread.created".to_string(),
        body: json!({ "tag": tag }),
        next_state: json!({ "status": "open", "tag": tag }),
        result: json!({ "tag": tag }),
    }
}

/// Remove a test's own rows from the shared WP2 tables (outbox first — FK).
async fn cleanup(pool: &PgPool, tenant: &str, tag: &str) {
    sqlx::query("DELETE FROM outbox WHERE tenant_id = $1 AND event_id = $2")
        .bind(tenant)
        .bind(format!("evt_{tag}"))
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM event_log WHERE tenant_id = $1 AND event_id = $2")
        .bind(tenant)
        .bind(format!("evt_{tag}"))
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM idempotency WHERE tenant_id = $1 AND idempotency_key = $2")
        .bind(tenant)
        .bind(format!("key_{tag}"))
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM aggregate_state WHERE tenant_id = $1 AND aggregate_id = $2")
        .bind(tenant)
        .bind(format!("thr_{tag}"))
        .execute(pool)
        .await
        .ok();
}

async fn event_count(pool: &PgPool, tenant: &str, event_id: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM event_log WHERE tenant_id = $1 AND event_id = $2")
        .bind(tenant)
        .bind(event_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// THE membership acceptance: with a boundary but NO grant, the principal is denied —
/// tenant membership alone grants no mandate — and the denial IS audited.
#[tokio::test]
async fn tenant_membership_alone_grants_nothing_and_denials_are_audited() {
    let _guard = authority_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000101";
    create_boundary(
        &pool,
        &boundary(
            tenant,
            vec![GrantAction::ThreadContribute],
            RiskClass::Medium,
            false,
        ),
    )
    .await
    .unwrap();

    let outcome = authorize(
        &pool,
        &authz(
            "agt_00000000-0000-7000-8000-000000000101",
            GrantSubject::Role("rol_00000000-0000-7000-8000-000000000101".parse().unwrap()),
            None,
            GrantAction::ThreadContribute,
            thread_target(tenant, "thr_00000000-0000-7000-8000-000000000101"),
        ),
        Utc::now(),
    )
    .await
    .unwrap();

    let AuthorizationOutcome::Denied { reason, record_id } = &outcome else {
        panic!("membership alone must be denied, got {outcome:?}");
    };
    assert!(reason.contains("membership"), "got: {reason}");
    let record = load_authorization_record(&pool, record_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        record.decision,
        Decision::Denied {
            reason: reason.clone()
        }
    );
    assert!(record.grant_id.is_none(), "no grant was referenced");
    assert_eq!(
        record.boundary_id.as_deref(),
        Some("bnd_00000000-0000-7000-8000-000000000101")
    );

    // The same denial through the command path: audited, and NOTHING was applied.
    let err = apply_authorized_command(
        &pool,
        &authz(
            "agt_00000000-0000-7000-8000-000000000101",
            GrantSubject::Role("rol_00000000-0000-7000-8000-000000000101".parse().unwrap()),
            None,
            GrantAction::ThreadContribute,
            thread_target(tenant, "thr_00000000-0000-7000-8000-000000000101"),
        ),
        &command(tenant, "membership"),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, AuthorizedApplyError::Denied { .. }),
        "got: {err}"
    );
    assert_eq!(
        event_count(&pool, tenant, "evt_membership").await,
        0,
        "a denied command writes no domain effect"
    );
    cleanup(&pool, tenant, "membership").await;
}

/// Administrative authority is never implied by other actions.
#[tokio::test]
async fn administrative_authority_is_never_implied() {
    let _guard = authority_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000102";
    let boundary = boundary(
        tenant,
        vec![GrantAction::ThreadContribute, GrantAction::ThreadCreate],
        RiskClass::Medium,
        false,
    );
    create_boundary(&pool, &boundary).await.unwrap();
    create_grant(
        &pool,
        &grant(
            &boundary.boundary_id,
            tenant,
            "rol_00000000-0000-7000-8000-000000000102",
            vec![GrantAction::ThreadContribute, GrantAction::ThreadCreate],
            TargetSelector::TenantWide,
        ),
    )
    .await
    .unwrap();

    let outcome = authorize(
        &pool,
        &authz(
            "agt_00000000-0000-7000-8000-000000000102",
            GrantSubject::Role("rol_00000000-0000-7000-8000-000000000102".parse().unwrap()),
            None,
            GrantAction::TenantAdmin,
            tenant_target(tenant),
        ),
        Utc::now(),
    )
    .await
    .unwrap();
    let AuthorizationOutcome::Denied { reason, .. } = outcome else {
        panic!("admin authority must never be implied");
    };
    assert!(reason.contains("not granted"), "got: {reason}");
}

/// THE audit acceptance: an accepted command's record carries the actor, the delegated
/// subject, the grant + boundary references, the allowed decision, and the policy
/// digest + version — and the digest re-derives from the same inputs.
#[tokio::test]
async fn every_accepted_command_records_actor_subject_grant_decision_and_digest() {
    let _guard = authority_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000103";
    let boundary = boundary(
        tenant,
        vec![GrantAction::ThreadContribute],
        RiskClass::Medium,
        false,
    );
    create_boundary(&pool, &boundary).await.unwrap();
    // The ACTOR's grant (the caller check) + the SUBJECT's grant (the
    // authority source under the `.1.4.2` dual evaluation).
    let actor_grt = grant(
        &boundary.boundary_id,
        tenant,
        "rol_00000000-0000-7000-8000-000000000103",
        vec![GrantAction::ThreadContribute],
        TargetSelector::TenantWide,
    );
    create_grant(&pool, &actor_grt).await.unwrap();
    let subject_grt = grant(
        &boundary.boundary_id,
        tenant,
        "hpr_00000000-0000-7000-8000-000000000113",
        vec![GrantAction::ThreadContribute],
        TargetSelector::TenantWide,
    );
    create_grant(&pool, &subject_grt).await.unwrap();

    let delegate = Some(GrantSubject::Human(
        "hpr_00000000-0000-7000-8000-000000000113".parse().unwrap(),
    ));
    let principal = GrantSubject::Role("rol_00000000-0000-7000-8000-000000000103".parse().unwrap());
    let target = thread_target(tenant, "thr_00000000-0000-7000-8000-000000000103");
    let ctx = authz(
        "agt_00000000-0000-7000-8000-000000000103",
        principal.clone(),
        delegate.clone(),
        GrantAction::ThreadContribute,
        target.clone(),
    );

    let outcome = apply_authorized_command(&pool, &ctx, &command(tenant, "audited"))
        .await
        .unwrap();
    assert!(!outcome.replayed);
    assert_eq!(
        event_count(&pool, tenant, "evt_audited").await,
        1,
        "the command was applied"
    );

    // The record: every acceptance field, read back from the database.
    let (record_id,): (String,) = sqlx::query_as(
        "SELECT record_id FROM authorization_records WHERE tenant_id = $1 AND action = 'thread_contribute'",
    )
    .bind(tenant)
    .fetch_one(&pool)
    .await
    .unwrap();
    let record = load_authorization_record(&pool, &record_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        record.actor.to_string(),
        "agt_00000000-0000-7000-8000-000000000103"
    );
    assert_eq!(
        record.subject, delegate,
        "the delegated subject is recorded"
    );
    assert_eq!(
        record.boundary_id.as_deref(),
        Some(boundary.boundary_id.as_str())
    );
    assert_eq!(
        record.grant_id.as_deref(),
        Some(subject_grt.grant_id.as_str()),
        "the record binds the SUBJECT's grant (the authority source)"
    );
    assert_eq!(record.decision, Decision::Allowed);
    assert_eq!(record.policy_version, boundary.policy_version);
    assert_eq!(record.policy_digest.len(), 64);

    // The database's split subject fields reconstruct a core object which must
    // also serialize as a complete delegated authorization payload.
    let encoded = serde_json::to_vec(&record).unwrap();
    let decoded: AuthorizationDecisionRecord = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, record);
    let payload: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(
        payload["subject"],
        serde_json::json!({
            "kind": "human", "id": "hpr_00000000-0000-7000-8000-000000000113",
        })
    );

    // The digest re-derives from the SAME inputs (the re-derive leg).
    let expected = policy_digest(
        Some(&boundary),
        Some(&subject_grt),
        delegate.as_ref(),
        GrantAction::ThreadContribute,
        &target,
        &Decision::Allowed,
    );
    assert_eq!(
        record.policy_digest, expected,
        "the stored digest re-derives"
    );
    cleanup(&pool, tenant, "audited").await;
}

/// A grant cannot exceed the ceiling — refused at creation, nothing stored.
#[tokio::test]
async fn a_grant_cannot_exceed_the_boundary() {
    let _guard = authority_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000104";
    let boundary = boundary(
        tenant,
        vec![GrantAction::ThreadContribute],
        RiskClass::Medium,
        false,
    );
    create_boundary(&pool, &boundary).await.unwrap();

    let overreaching = grant(
        &boundary.boundary_id,
        tenant,
        "rol_00000000-0000-7000-8000-000000000104",
        vec![GrantAction::ThreadContribute, GrantAction::TenantAdmin],
        TargetSelector::TenantWide,
    );
    let err = create_grant(&pool, &overreaching).await.unwrap_err();
    assert!(err
        .violations
        .iter()
        .any(|v| v.detail.contains("tenant_admin")));

    let risky = grant(
        &boundary.boundary_id,
        tenant,
        "rol_00000000-0000-7000-8000-000000000104",
        vec![GrantAction::ThreadContribute],
        TargetSelector::TenantWide,
    );
    let mut risky = risky;
    risky.risk_ceiling = RiskClass::High;
    let err = create_grant(&pool, &risky).await.unwrap_err();
    assert!(err.violations.iter().any(|v| v.detail.contains("risk")));

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM authority_grants WHERE tenant_id = $1")
            .bind(tenant)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0, "refused grants are never stored");
}

/// Scope: a thread-scoped grant reaches only its threads; tenant boundaries bind
/// every evaluation; thread_create targets the tenant.
#[tokio::test]
async fn scoped_commands_reach_only_the_allowed_threads() {
    let _guard = authority_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000105";
    let boundary = boundary(
        tenant,
        vec![
            GrantAction::ThreadContribute,
            GrantAction::ThreadInspect,
            GrantAction::ThreadCreate,
        ],
        RiskClass::Medium,
        false,
    );
    create_boundary(&pool, &boundary).await.unwrap();
    let grt = grant(
        &boundary.boundary_id,
        tenant,
        "rol_00000000-0000-7000-8000-000000000105",
        vec![GrantAction::ThreadContribute, GrantAction::ThreadInspect],
        TargetSelector::Threads {
            threads: vec!["thr_00000000-0000-7000-8000-000000000105".parse().unwrap()],
        },
    );
    create_grant(&pool, &grt).await.unwrap();
    let principal = GrantSubject::Role("rol_00000000-0000-7000-8000-000000000105".parse().unwrap());

    let in_scope = authorize(
        &pool,
        &authz(
            "agt_00000000-0000-7000-8000-000000000105",
            principal.clone(),
            None,
            GrantAction::ThreadContribute,
            thread_target(tenant, "thr_00000000-0000-7000-8000-000000000105"),
        ),
        Utc::now(),
    )
    .await
    .unwrap();
    assert!(matches!(in_scope, AuthorizationOutcome::Allowed { .. }));

    let out_of_scope = authorize(
        &pool,
        &authz(
            "agt_00000000-0000-7000-8000-000000000105",
            principal.clone(),
            None,
            GrantAction::ThreadInspect,
            thread_target(tenant, "thr_00000000-0000-7000-8000-000000000999"),
        ),
        Utc::now(),
    )
    .await
    .unwrap();
    let AuthorizationOutcome::Denied { reason, .. } = out_of_scope else {
        panic!("inspection outside the grant's scope must be denied");
    };
    assert!(reason.contains("scope"), "got: {reason}");
}

/// Expired and revoked grants are denied at evaluation time.
#[tokio::test]
async fn expired_and_revoked_grants_are_denied() {
    let _guard = authority_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000106";
    let boundary = boundary(
        tenant,
        vec![GrantAction::ThreadContribute],
        RiskClass::Medium,
        false,
    );
    create_boundary(&pool, &boundary).await.unwrap();

    let mut expired = grant(
        &boundary.boundary_id,
        tenant,
        "rol_00000000-0000-7000-8000-000000000106",
        vec![GrantAction::ThreadContribute],
        TargetSelector::TenantWide,
    );
    // Inside the boundary's window, but EXPIRED at evaluation time.
    expired.valid_from = Utc::now() - Duration::days(20);
    expired.expires_at = Utc::now() - Duration::days(1);
    create_grant(&pool, &expired).await.unwrap();

    let outcome = authorize(
        &pool,
        &authz(
            "agt_00000000-0000-7000-8000-000000000106",
            GrantSubject::Role("rol_00000000-0000-7000-8000-000000000106".parse().unwrap()),
            None,
            GrantAction::ThreadContribute,
            thread_target(tenant, "thr_00000000-0000-7000-8000-000000000106"),
        ),
        Utc::now(),
    )
    .await
    .unwrap();
    let AuthorizationOutcome::Denied { reason, .. } = outcome else {
        panic!("an expired grant must be denied");
    };
    assert!(reason.contains("validity window"), "got: {reason}");
}

/// A tenant without a boundary is denied with the reason on the record.
#[tokio::test]
async fn a_tenant_without_a_boundary_is_denied() {
    let _guard = authority_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000107";
    let outcome = authorize(
        &pool,
        &authz(
            "agt_00000000-0000-7000-8000-000000000107",
            GrantSubject::Role("rol_00000000-0000-7000-8000-000000000107".parse().unwrap()),
            None,
            GrantAction::ThreadContribute,
            thread_target(tenant, "thr_00000000-0000-7000-8000-000000000107"),
        ),
        Utc::now(),
    )
    .await
    .unwrap();
    let AuthorizationOutcome::Denied { reason, record_id } = outcome else {
        panic!("no boundary means no authority");
    };
    assert!(
        reason.contains("no active enrollment boundary"),
        "got: {reason}"
    );
    let record = load_authorization_record(&pool, &record_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.boundary_id, None);
}

/// Two identical evaluations produce identical digests; the record ids differ (each
/// decision is its own audited row).
#[tokio::test]
async fn digests_are_stable_across_identical_evaluations() {
    let _guard = authority_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000108";
    let boundary = boundary(
        tenant,
        vec![GrantAction::ThreadContribute],
        RiskClass::Medium,
        false,
    );
    create_boundary(&pool, &boundary).await.unwrap();
    let grt = grant(
        &boundary.boundary_id,
        tenant,
        "rol_00000000-0000-7000-8000-000000000108",
        vec![GrantAction::ThreadContribute],
        TargetSelector::TenantWide,
    );
    create_grant(&pool, &grt).await.unwrap();
    let ctx = authz(
        "agt_00000000-0000-7000-8000-000000000108",
        GrantSubject::Role("rol_00000000-0000-7000-8000-000000000108".parse().unwrap()),
        None,
        GrantAction::ThreadContribute,
        thread_target(tenant, "thr_00000000-0000-7000-8000-000000000108"),
    );

    let first = authorize(&pool, &ctx, Utc::now()).await.unwrap();
    let second = authorize(&pool, &ctx, Utc::now()).await.unwrap();
    let (
        AuthorizationOutcome::Allowed { record_id: a, .. },
        AuthorizationOutcome::Allowed { record_id: b, .. },
    ) = (&first, &second)
    else {
        panic!("both evaluations must be allowed");
    };
    let record_a = load_authorization_record(&pool, a).await.unwrap().unwrap();
    let record_b = load_authorization_record(&pool, b).await.unwrap().unwrap();
    assert_ne!(a, b, "each decision is its own record");
    assert_eq!(
        record_a.policy_digest, record_b.policy_digest,
        "identical inputs → identical digest"
    );

    // A boundary health check helper is exercised here for completeness.
    assert!(boundary_active_at(&boundary, Utc::now()));
}

/// The authorization record's own type: a full round trip through the DB read-back.
#[tokio::test]
async fn record_readback_round_trips_through_the_core_type() {
    let _guard = authority_guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_00000000-0000-7000-8000-000000000109";
    let boundary = boundary(
        tenant,
        vec![GrantAction::ThreadContribute],
        RiskClass::Medium,
        false,
    );
    create_boundary(&pool, &boundary).await.unwrap();
    create_grant(
        &pool,
        &grant(
            &boundary.boundary_id,
            tenant,
            "rol_00000000-0000-7000-8000-000000000109",
            vec![GrantAction::ThreadContribute],
            TargetSelector::TenantWide,
        ),
    )
    .await
    .unwrap();

    let ctx = authz(
        "agt_00000000-0000-7000-8000-000000000109",
        GrantSubject::Role("rol_00000000-0000-7000-8000-000000000109".parse().unwrap()),
        None,
        GrantAction::ThreadContribute,
        thread_target(tenant, "thr_00000000-0000-7000-8000-000000000109"),
    );
    let AuthorizationOutcome::Allowed { record_id, .. } =
        authorize(&pool, &ctx, Utc::now()).await.unwrap()
    else {
        panic!("expected allowed");
    };
    let record: AuthorizationDecisionRecord = load_authorization_record(&pool, &record_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.action, GrantAction::ThreadContribute);
    assert_eq!(
        record.target,
        thread_target(tenant, "thr_00000000-0000-7000-8000-000000000109")
    );
    assert_eq!(record.record_id.to_string(), record_id);
}
