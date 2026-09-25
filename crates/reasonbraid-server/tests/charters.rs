//! The governance charter's decision rules (`SIGNOFF-REPAIR.11.4.7.2.1.2.1`;
//! ROADMAP §4.1 + §13.3) against a live store.
//!
//! The unit tests in `charters.rs` bound the VOCABULARY and the digest; these
//! bound what only a database can answer — the boundary→charter resolution,
//! the fail-closed answer for a boundary nothing resolves, and the control the
//! leaf's acceptance names: a rule allowed WHEN A DECISION WAS TAKEN stays
//! readable after the charter changes.
//!
//! Run with `scripts/run_pg_tests.sh charters`. Without an owned disposable
//! PostgreSQL these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::sync::OnceLock;

use reasonbraid_core::{GrantSubject, HumanPrincipalId};
use reasonbraid_server::charters::{self, CharterError, CharterInput, DecisionRule};
use reasonbraid_server::site_authority::Reason;
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
        "DELETE FROM public.governance_charters WHERE tenant_id LIKE 'charter-test-%'; \
         DELETE FROM public.enrollment_boundaries WHERE tenant_id LIKE 'charter-test-%'",
    )
    .execute(&pool)
    .await
    .unwrap();
    Some(pool)
}

fn input(tenant: &str, rules: &[&str], thresholds: &[(&str, f64)]) -> CharterInput {
    CharterInput {
        tenant_id: tenant.to_owned(),
        allowed_decision_rules: rules.iter().map(|r| (*r).to_owned()).collect(),
        approval_thresholds: thresholds
            .iter()
            .map(|(k, v)| ((*k).to_owned(), *v))
            .collect(),
        charter_digest: None,
        reason: Reason::new("a charter control").unwrap(),
    }
}

/// Bind a tenant's ACTIVE enrollment boundary to a charter digest.
async fn boundary(pool: &PgPool, tenant: &str, charter_digest: &str) {
    sqlx::query(
        "INSERT INTO enrollment_boundaries \
         (boundary_id, tenant_id, parent_or_root_authority, target_owner, permitted_actions, \
          permitted_domains, risk_ceiling, spend_ceiling, delegable, max_delegation_depth, \
          valid_from, expires_at, charter_digest, policy_version, status) \
         VALUES ($1,$2,'root','owner','[\"tenant_admin\"]'::jsonb,'[]'::jsonb,'low',NULL,false,0, \
                 now() - interval '1 day', now() + interval '1 day', $3, 'v1', 'active')",
    )
    .bind(format!("bnd_{}", uuid::Uuid::now_v7()))
    .bind(tenant)
    .bind(charter_digest)
    .execute(pool)
    .await
    .expect("seed the boundary");
}

async fn retire_boundaries(pool: &PgPool, tenant: &str) {
    sqlx::query("UPDATE enrollment_boundaries SET status = 'revoked' WHERE tenant_id = $1")
        .bind(tenant)
        .execute(pool)
        .await
        .expect("retire the boundary");
}

#[allow(dead_code)]
fn subject() -> GrantSubject {
    GrantSubject::Human(HumanPrincipalId::new())
}

#[tokio::test]
async fn a_registered_charter_is_readable_by_the_digest_the_server_derived() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "charter-test-read";
    let stored = charters::register(&pool, &input(tenant, &["consensus", "unanimity"], &[]))
        .await
        .expect("the charter registers");
    assert!(stored.charter_digest.starts_with("sha256:"));
    let read = charters::load(&pool, &stored.charter_digest)
        .await
        .expect("the charter loads");
    assert_eq!(read.tenant_id, tenant);
    // Submitted as `["consensus", "unanimity"]` and stored SORTED, because the
    // digest is taken over the sorted set — so two submissions differing only
    // in order are one charter rather than two rows.
    assert_eq!(
        read.allowed_decision_rules,
        vec![DecisionRule::Unanimity, DecisionRule::Consensus],
        "the stored set is the sorted set the digest was taken over"
    );
    assert!(read.approval_thresholds.is_empty());
}

#[tokio::test]
async fn registering_identical_content_twice_is_one_row() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "charter-test-idempotent";
    let first = charters::register(&pool, &input(tenant, &["consensus"], &[]))
        .await
        .unwrap();
    let second = charters::register(&pool, &input(tenant, &["consensus"], &[]))
        .await
        .unwrap();
    assert_eq!(first.charter_digest, second.charter_digest);
    let rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM governance_charters WHERE tenant_id = $1")
            .bind(tenant)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(rows, 1, "the row's identity IS its content");
}

#[tokio::test]
async fn an_asserted_digest_that_is_not_the_servers_is_refused() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "charter-test-asserted";
    let mut body = input(tenant, &["consensus"], &[]);
    body.charter_digest = Some(format!("sha256:{}", "a".repeat(64)));
    let err = charters::register(&pool, &body).await.unwrap_err();
    let CharterError::DigestMismatch { asserted, derived } = err else {
        panic!("expected a digest mismatch, got {err:?}")
    };
    assert_ne!(asserted, derived);
    let rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM governance_charters WHERE tenant_id = $1")
            .bind(tenant)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(rows, 0, "a refused registration writes nothing");
}

#[tokio::test]
async fn the_tenants_rule_resolves_through_its_active_boundary() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "charter-test-resolve";
    let stored = charters::register(
        &pool,
        &input(
            tenant,
            &["majority_of_electorate", "consensus"],
            &[("majority_of_electorate", 0.67)],
        ),
    )
    .await
    .unwrap();
    boundary(&pool, tenant, &stored.charter_digest).await;

    let (rule, threshold) = charters::allows(&pool, tenant, "majority_of_electorate")
        .await
        .expect("the rule is allowed");
    assert_eq!(rule, DecisionRule::MajorityOfElectorate);
    assert_eq!(threshold, Some(0.67), "the bar rides the answer");

    let (rule, threshold) = charters::allows(&pool, tenant, "consensus").await.unwrap();
    assert_eq!(rule, DecisionRule::Consensus);
    assert_eq!(threshold, None, "only the majority family carries a bar");
}

#[tokio::test]
async fn a_rule_the_charter_does_not_allow_is_refused_and_names_only_itself() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "charter-test-refused";
    let stored = charters::register(&pool, &input(tenant, &["consensus"], &[]))
        .await
        .unwrap();
    boundary(&pool, tenant, &stored.charter_digest).await;

    let err = charters::allows(&pool, tenant, "unanimity")
        .await
        .unwrap_err();
    assert_eq!(err, CharterError::NotAllowed("unanimity".into()));
    let rendered = err.to_string();
    assert!(rendered.contains("unanimity"));
    assert!(
        !rendered.contains("consensus"),
        "the refusal must not enumerate the charter — that answers a question \
         the caller did not ask: {rendered}"
    );
}

#[tokio::test]
async fn a_boundary_whose_charter_resolves_to_nothing_fails_closed() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "charter-test-legacy";
    // Every boundary issued before `migrations/0084` carries a LABEL, not a
    // digest — `dev-charter-000`, `fixture-charter`. The honest answer is that
    // the question cannot be answered, never that the rule is allowed.
    boundary(&pool, tenant, "dev-charter-000").await;
    let err = charters::allows(&pool, tenant, "consensus")
        .await
        .unwrap_err();
    let CharterError::BoundaryCharterUnknown { tenant_id, digest } = err else {
        panic!("a boundary naming an unresolvable charter must FAIL CLOSED, got {err:?}")
    };
    assert_eq!(tenant_id, tenant);
    assert_eq!(digest, "dev-charter-000");
}

#[tokio::test]
async fn a_tenant_with_no_active_boundary_has_no_charter() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "charter-test-unbounded";
    let stored = charters::register(&pool, &input(tenant, &["consensus"], &[]))
        .await
        .unwrap();
    boundary(&pool, tenant, &stored.charter_digest).await;
    retire_boundaries(&pool, tenant).await;
    assert_eq!(
        charters::allows(&pool, tenant, "consensus")
            .await
            .unwrap_err(),
        CharterError::NoBoundary(tenant.into()),
        "a registered charter nobody's boundary names governs nothing"
    );
}

/// ⭐ **The acceptance's own control.** Changing the allowed set must not
/// rewrite what a past decision was taken under.
#[tokio::test]
async fn changing_the_set_leaves_the_earlier_charter_exactly_as_it_was() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "charter-test-versioned";

    // Monday: the tenant may decide by consensus. A decision recorded now
    // carries THIS digest, because `reasonbraid_core::authority`'s decision
    // digest folds the boundary's `charter_digest` into every record.
    let monday = charters::register(&pool, &input(tenant, &["consensus"], &[]))
        .await
        .unwrap();
    boundary(&pool, tenant, &monday.charter_digest).await;
    let taken_under = monday.charter_digest.clone();
    assert!(charters::allows(&pool, tenant, "consensus").await.is_ok());

    // Tuesday: consensus is withdrawn and unanimity allowed instead.
    let tuesday = charters::register(&pool, &input(tenant, &["unanimity"], &[]))
        .await
        .unwrap();
    assert_ne!(tuesday.charter_digest, taken_under);
    retire_boundaries(&pool, tenant).await;
    boundary(&pool, tenant, &tuesday.charter_digest).await;

    // The tenant may no longer decide by consensus …
    assert_eq!(
        charters::allows(&pool, tenant, "consensus")
            .await
            .unwrap_err(),
        CharterError::NotAllowed("consensus".into())
    );
    // … and Monday's decision remains interpretable, because the charter it
    // was taken under is a different row that nothing overwrote.
    let past = charters::load(&pool, &taken_under)
        .await
        .expect("the charter a past decision names is still addressable");
    assert_eq!(past.allowed_decision_rules, vec![DecisionRule::Consensus]);
    assert_eq!(
        past.charter_digest, taken_under,
        "the historical charter still hashes to what the decision record carries"
    );
}

/// `SIGNOFF-REPAIR.9.1.1` — the charter SITE ACT admits an operator holding
/// `charter_register`, end to end. Before migration `0109` no boundary could
/// hold that action, so the act, which `POST /v1/governance-charters` calls,
/// refused every caller; this suite had only ever registered through
/// `charters::register`, underneath the gate.
#[tokio::test]
async fn an_operator_holding_charter_register_registers_through_the_site_act() {
    use chrono::{Duration, Utc};
    use reasonbraid_server::site_authority::{self as site, Action, Scope};
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let operator = GrantSubject::Human(HumanPrincipalId::new());
    let scope = Scope {
        actions: vec![Action::CharterRegister],
        valid_from: Utc::now() - Duration::hours(2),
        expires_at: Utc::now() + Duration::hours(4),
    };
    let reason = Reason::new("charter operator").unwrap();
    let boundary = site::issue_boundary(&pool, &scope, &reason)
        .await
        .expect("a boundary can hold charter_register");
    let boundary_id = boundary.result["boundary_id"].as_str().unwrap().to_owned();
    site::issue_grant(&pool, &boundary_id, &operator, &scope, &reason)
        .await
        .expect("a grant can hold charter_register");

    let receipt = site::charters::register_charter(
        &pool,
        &operator,
        &input("charter-test-site-act", &["consensus"], &[]),
    )
    .await
    .expect("the site act admits the operator");
    assert_eq!(
        receipt.result["tenant_id"],
        serde_json::json!("charter-test-site-act"),
        "the charter was registered: {:?}",
        receipt.result
    );

    // The matched pair: a principal holding no charter_register is refused.
    let stranger = GrantSubject::Human(HumanPrincipalId::new());
    assert!(
        site::charters::register_charter(
            &pool,
            &stranger,
            &input("charter-test-site-act-stranger", &["consensus"], &[]),
        )
        .await
        .is_err(),
        "only a holder registers a charter"
    );
}
