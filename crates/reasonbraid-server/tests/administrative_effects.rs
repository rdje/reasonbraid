//! Final administrative effect evidence, against a live PostgreSQL
//! (`SIGNOFF-REPAIR.3.3.4.7.2`).
//!
//! `.7.1` defined the representation; this qualifies its durability. The
//! properties under test are the ones the decision record specified and that a
//! unit test cannot reach, because each is a statement about a TRANSACTION:
//!
//! - the evidence and the mutation commit together, so an operator reading
//!   `applied` is reading something that actually happened;
//! - an evidence failure rolls the protected write BACK, rather than leaving a
//!   mutation nobody can explain;
//! - a committed `no_op` or `refused` leaves protected state AND the tenant's
//!   revocation epoch unchanged, so a recorded refusal is not a quiet mutation;
//! - malformed stored evidence is a storage failure, never a guessed outcome;
//! - an absent record reads as absent, not as a successful operation.
//!
//! No production route writes an effect record yet — `.8` is the first — so these
//! controls drive the production writer and reader directly, on their own guarded
//! transactions, exactly as `.3.3.4.2` qualified the tenant guard before any
//! application used it.
#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

// The PRODUCTION guard module, compiled into this test so a control takes the
// same lock the server takes rather than a copy of its SQL.
#[allow(dead_code)]
#[path = "../src/authority/transaction.rs"]
mod tenant_transaction;

use chrono::{DateTime, TimeZone, Utc};
use reasonbraid_core::{
    AdministrativeEffectRecord, AdministrativeOperation, AdministrativeOutcome,
    AdministrativeReason, AdministrativeRefusal, AdministrativeTargetId, AuthorizationRecordId,
    TenantId,
};
use reasonbraid_server::{load_tenant_administrative_effect, record_administrative_effect_in_tx};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use tenant_transaction::{acquire_in_tx, GuardMode};

async fn pool() -> Option<PgPool> {
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    pg_cleanup::delete_tables(
        &pool,
        &[
            "outbox_delivery",
            "outbox",
            "node_events",
            "node_inbox",
            "budget_reservations",
            "budget_ceilings",
            "spend_breakers",
            "administrative_effects",
            "authorization_records",
            "authority_grants",
            "enrollments",
            "enrollment_boundaries",
            "node_enroll_audit",
            "node_keys",
            "node_certificates",
            "server_ca",
            "node_leases",
            "node_enrollment_tokens",
            "runs",
            "incarnations",
            "nodes",
            "hosts",
            "profile_versions",
            "agent_profiles",
            "recruitment_panels",
            "recruitment_responses",
            "recruitment_offers",
            "recruitment_calls",
            "agent_roles",
            "human_principals",
            "claim_assessments",
            "derivations",
            "evidence_snapshots",
            "resource_references",
            "quota_events",
            "usage_quotas",
            "federation_agreements",
            "cross_domain_receipts",
            "mcp_listen_state",
            "tenant_bootstrap_requests",
            "tenants",
            "idempotency",
            "event_log",
            "aggregate_state",
        ],
    )
    .await
    .expect("purge checked fixture plan");
    Some(pool)
}

fn at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 12, 12, 0, 0).unwrap()
}

fn reason(value: &str) -> AdministrativeReason {
    AdministrativeReason::new(value).expect("the fixture reason is within bounds")
}

fn target(value: &str) -> AdministrativeTargetId {
    AdministrativeTargetId::new(value).expect("the fixture target id is within bounds")
}

/// A tenant with a revocation epoch and one active grant to revoke — the
/// smallest protected state whose change is observable.
struct Fixture {
    tenant: TenantId,
    grant_id: String,
}

async fn fixture(pool: &PgPool) -> Fixture {
    let tenant = TenantId::new();
    sqlx::query("INSERT INTO tenants (tenant_id, revocation_epoch) VALUES ($1, 0)")
        .bind(tenant.to_string())
        .execute(pool)
        .await
        .expect("seed the tenant");
    let boundary_id = format!("bnd_{tenant}");
    sqlx::query(
        "INSERT INTO enrollment_boundaries \
         (boundary_id, tenant_id, parent_or_root_authority, target_owner, permitted_actions, \
          permitted_domains, risk_ceiling, spend_ceiling, delegable, max_delegation_depth, \
          valid_from, expires_at, charter_digest, policy_version, status) \
         VALUES ($1,$2,'root','owner','[\"tenant_admin\"]'::jsonb,'[]'::jsonb,'low',NULL,false,0, \
                 now() - interval '1 day', now() + interval '1 day', 'digest', 'v1', 'active')",
    )
    .bind(&boundary_id)
    .bind(tenant.to_string())
    .execute(pool)
    .await
    .expect("seed the boundary");
    let grant_id = format!("grt_{}", uuid::Uuid::now_v7());
    sqlx::query(
        "INSERT INTO authority_grants \
         (grant_id, boundary_id, tenant_id, issuer, subject_kind, subject_id, actions, selector, \
          risk_ceiling, spend_limits, delegable, valid_from, expires_at, status) \
         VALUES ($1,$2,$3,'hpr_00000000-0000-7000-8000-000000000001','human', \
                 'hpr_00000000-0000-7000-8000-000000000001','[\"tenant_admin\"]'::jsonb, \
                 '{\"kind\":\"tenant_wide\"}'::jsonb,'low',NULL,false, \
                 now() - interval '1 day', now() + interval '1 day','active')",
    )
    .bind(&grant_id)
    .bind(&boundary_id)
    .bind(tenant.to_string())
    .execute(pool)
    .await
    .expect("seed the grant");
    Fixture { tenant, grant_id }
}

/// One committed admission, so an effect has something real to be tied to.
async fn admission(pool: &PgPool, tenant: TenantId) -> AuthorizationRecordId {
    let record_id = AuthorizationRecordId::new();
    sqlx::query(
        "INSERT INTO authorization_records \
         (record_id, tenant_id, actor, action, target_kind, target_tenant, decision, \
          policy_digest, policy_version, decided_at, evaluation) \
         VALUES ($1,$2,'agt_00000000-0000-7000-8000-000000000001','tenant_admin','tenant',$2, \
                 'allowed','digest','v1', now(), '{\"kind\":\"boundary_checked\"}'::jsonb)",
    )
    .bind(record_id.to_string())
    .bind(tenant.to_string())
    .execute(pool)
    .await
    .expect("seed the admission");
    record_id
}

async fn grant_status(pool: &PgPool, grant_id: &str) -> String {
    sqlx::query_scalar("SELECT status FROM authority_grants WHERE grant_id = $1")
        .bind(grant_id)
        .fetch_one(pool)
        .await
        .expect("read the grant status")
}

async fn epoch(pool: &PgPool, tenant: TenantId) -> i64 {
    sqlx::query_scalar("SELECT revocation_epoch FROM tenants WHERE tenant_id = $1")
        .bind(tenant.to_string())
        .fetch_one(pool)
        .await
        .expect("read the revocation epoch")
}

async fn effect_rows(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM administrative_effects")
        .fetch_one(pool)
        .await
        .expect("count the effect rows")
}

fn effect(
    record_id: AuthorizationRecordId,
    tenant: TenantId,
    grant_id: &str,
    outcome: AdministrativeOutcome,
) -> AdministrativeEffectRecord {
    AdministrativeEffectRecord {
        record_id,
        tenant_id: tenant,
        operation: AdministrativeOperation::GrantRevoke {
            grant_id: target(grant_id),
        },
        submitted_reason: Some(reason("role retired")),
        outcome,
        effected_at: at(),
    }
}

#[tokio::test]
async fn the_effect_and_the_mutation_commit_together() {
    let Some(pool) = pool().await else { return };
    let f = fixture(&pool).await;
    let record_id = admission(&pool, f.tenant).await;

    let mut tx = pool.begin().await.expect("begin");
    acquire_in_tx(&mut tx, f.tenant, GuardMode::Exclusive)
        .await
        .expect("the exclusive tenant guard a revocation holds");
    sqlx::query(
        "UPDATE authority_grants SET status = 'revoked' WHERE grant_id = $1 AND tenant_id = $2",
    )
    .bind(&f.grant_id)
    .bind(f.tenant.to_string())
    .execute(&mut *tx)
    .await
    .expect("the protected mutation");
    sqlx::query("UPDATE tenants SET revocation_epoch = revocation_epoch + 1 WHERE tenant_id = $1")
        .bind(f.tenant.to_string())
        .execute(&mut *tx)
        .await
        .expect("the epoch bump that rides the revocation");
    let record = effect(
        record_id,
        f.tenant,
        &f.grant_id,
        AdministrativeOutcome::Applied {},
    );
    record_administrative_effect_in_tx(&mut tx, &record)
        .await
        .expect("record the final effect on the SAME transaction");
    // Nothing is visible to another connection until this commit.
    assert_eq!(grant_status(&pool, &f.grant_id).await, "active");
    assert_eq!(effect_rows(&pool).await, 0);
    tx.commit().await.expect("commit");

    assert_eq!(grant_status(&pool, &f.grant_id).await, "revoked");
    assert_eq!(epoch(&pool, f.tenant).await, 1);
    let stored = load_tenant_administrative_effect(&pool, f.tenant, record_id)
        .await
        .expect("read the effect back")
        .expect("the effect committed with its mutation");
    assert_eq!(stored, record);
    assert!(stored.outcome.changed_protected_state());
}

#[tokio::test]
async fn an_evidence_failure_rolls_the_protected_write_back() {
    let Some(pool) = pool().await else { return };
    let f = fixture(&pool).await;
    let record_id = admission(&pool, f.tenant).await;
    // The same admission already carries an effect, so the second insert
    // violates the primary key — an evidence failure arriving AFTER the
    // protected write, which is the ordering that matters.
    let mut seed = pool.begin().await.expect("begin");
    record_administrative_effect_in_tx(
        &mut seed,
        &effect(
            record_id,
            f.tenant,
            &f.grant_id,
            AdministrativeOutcome::NoOp {
                detail: reason("the grant was already revoked"),
            },
        ),
    )
    .await
    .expect("seed one effect");
    seed.commit().await.expect("commit the seed");

    let mut tx = pool.begin().await.expect("begin");
    acquire_in_tx(&mut tx, f.tenant, GuardMode::Exclusive)
        .await
        .expect("guard");
    sqlx::query(
        "UPDATE authority_grants SET status = 'revoked' WHERE grant_id = $1 AND tenant_id = $2",
    )
    .bind(&f.grant_id)
    .bind(f.tenant.to_string())
    .execute(&mut *tx)
    .await
    .expect("the protected mutation lands first");
    let failure = record_administrative_effect_in_tx(
        &mut tx,
        &effect(
            record_id,
            f.tenant,
            &f.grant_id,
            AdministrativeOutcome::Applied {},
        ),
    )
    .await
    .expect_err("the duplicate evidence insert fails");
    assert!(
        failure
            .as_database_error()
            .is_some_and(|e| e.is_unique_violation()),
        "expected a unique violation, got {failure:?}"
    );
    tx.rollback()
        .await
        .expect("the aborted transaction rolls back");

    // The protected write did NOT survive its evidence.
    assert_eq!(grant_status(&pool, &f.grant_id).await, "active");
    assert_eq!(epoch(&pool, f.tenant).await, 0);
    assert_eq!(effect_rows(&pool).await, 1);
    let stored = load_tenant_administrative_effect(&pool, f.tenant, record_id)
        .await
        .expect("read back")
        .expect("the seeded effect survives");
    assert!(matches!(stored.outcome, AdministrativeOutcome::NoOp { .. }));
}

#[tokio::test]
async fn a_committed_no_op_and_refusal_change_no_protected_state_or_epoch() {
    let Some(pool) = pool().await else { return };
    let f = fixture(&pool).await;

    for outcome in [
        AdministrativeOutcome::NoOp {
            detail: reason("the grant was already revoked"),
        },
        AdministrativeOutcome::Refused {
            code: AdministrativeRefusal::InvalidTransition,
            detail: reason("the grant is not in a revocable state"),
        },
    ] {
        let record_id = admission(&pool, f.tenant).await;
        let mut tx = pool.begin().await.expect("begin");
        acquire_in_tx(&mut tx, f.tenant, GuardMode::Exclusive)
            .await
            .expect("guard");
        let record = effect(record_id, f.tenant, &f.grant_id, outcome);
        record_administrative_effect_in_tx(&mut tx, &record)
            .await
            .expect("a refusal commits its own evidence");
        tx.commit().await.expect("commit");

        let stored = load_tenant_administrative_effect(&pool, f.tenant, record_id)
            .await
            .expect("read back")
            .expect("the refusal is durable");
        assert_eq!(stored, record);
        assert!(
            !stored.outcome.changed_protected_state(),
            "only `applied` may assert a protected change"
        );
        assert_eq!(grant_status(&pool, &f.grant_id).await, "active");
        assert_eq!(epoch(&pool, f.tenant).await, 0);
    }
}

#[tokio::test]
async fn an_absent_effect_reads_as_absent_rather_than_as_a_successful_operation() {
    let Some(pool) = pool().await else { return };
    let f = fixture(&pool).await;
    // An admission with no effect: every request admitted before this table
    // existed reads exactly this way, and so does every unmigrated route.
    let record_id = admission(&pool, f.tenant).await;
    assert!(
        load_tenant_administrative_effect(&pool, f.tenant, record_id)
            .await
            .expect("an absent effect is not an error")
            .is_none()
    );
    // An admission id that does not exist at all reads the same.
    assert!(
        load_tenant_administrative_effect(&pool, f.tenant, AuthorizationRecordId::new())
            .await
            .expect("an unknown admission is not an error")
            .is_none()
    );
}

#[tokio::test]
async fn a_foreign_tenant_sees_neither_the_effect_nor_its_malformed_evidence() {
    let Some(pool) = pool().await else { return };
    let mine = fixture(&pool).await;
    let theirs = fixture(&pool).await;
    let record_id = admission(&pool, mine.tenant).await;
    let mut tx = pool.begin().await.expect("begin");
    record_administrative_effect_in_tx(
        &mut tx,
        &effect(
            record_id,
            mine.tenant,
            &mine.grant_id,
            AdministrativeOutcome::Applied {},
        ),
    )
    .await
    .expect("record");
    tx.commit().await.expect("commit");

    assert!(
        load_tenant_administrative_effect(&pool, mine.tenant, record_id)
            .await
            .expect("the owner reads it")
            .is_some()
    );
    // Filtered BEFORE decoding, so a foreign row is absent rather than an error
    // an outsider could use as an existence oracle.
    assert!(
        load_tenant_administrative_effect(&pool, theirs.tenant, record_id)
            .await
            .expect("a foreign lookup is not an error")
            .is_none()
    );

    // The binding is structural, not merely a convention the writer follows.
    // Both ways of breaking it are refused by the composite foreign key: an
    // effect recorded under one tenant while citing ANOTHER tenant's admission,
    // and an effect citing no admission at all.
    let mine_second = admission(&pool, mine.tenant).await;
    for (label, cited, owner) in [
        (
            "an effect citing another tenant's admission",
            mine_second,
            theirs.tenant,
        ),
        (
            "an effect citing no admission at all",
            AuthorizationRecordId::new(),
            theirs.tenant,
        ),
    ] {
        let crossed = sqlx::query(
            "INSERT INTO administrative_effects \
             (record_id, tenant_id, operation, submitted_reason, outcome, effected_at) \
             VALUES ($1, $2, '{\"kind\":\"breaker_reset\"}'::jsonb, NULL, \
                     '{\"kind\":\"applied\"}'::jsonb, now())",
        )
        .bind(cited.to_string())
        .bind(owner.to_string())
        .execute(&pool)
        .await
        .unwrap_err();
        assert!(
            crossed
                .as_database_error()
                .is_some_and(|e| e.is_foreign_key_violation()),
            "{label} was admitted: {crossed:?}"
        );
    }
}

#[tokio::test]
async fn malformed_stored_evidence_is_a_storage_failure_not_a_guessed_outcome() {
    let Some(pool) = pool().await else { return };
    let f = fixture(&pool).await;

    // Each row passes the column CHECK — its `kind` is in the vocabulary — and
    // fails the codec. That is the interesting case: the database constrains the
    // discriminant, and everything under it is the decoder's to refuse.
    for (label, operation, outcome, reason_text) in [
        (
            "an operation missing its target",
            json!({"kind": "grant_revoke"}),
            json!({"kind": "applied"}),
            None,
        ),
        (
            "an operation carrying a target that cannot belong to it",
            json!({"kind": "breaker_arm", "grant_id": "grt_x"}),
            json!({"kind": "applied"}),
            None,
        ),
        (
            "a refusal with no reason code",
            json!({"kind": "breaker_reset"}),
            json!({"kind": "refused", "detail": "no code"}),
            None,
        ),
        (
            "a refusal naming a code outside the measured set",
            json!({"kind": "breaker_reset"}),
            json!({"kind": "refused", "code": "quota_exceeded", "detail": "x"}),
            None,
        ),
        (
            "a submitted reason that is only whitespace",
            json!({"kind": "breaker_reset"}),
            json!({"kind": "applied"}),
            Some("   "),
        ),
    ] {
        let record_id = admission(&pool, f.tenant).await;
        sqlx::query(
            "INSERT INTO administrative_effects \
             (record_id, tenant_id, operation, submitted_reason, outcome, effected_at) \
             VALUES ($1, $2, $3, $4, $5, now())",
        )
        .bind(record_id.to_string())
        .bind(f.tenant.to_string())
        .bind(sqlx::types::Json(&operation))
        .bind(reason_text)
        .bind(sqlx::types::Json(&outcome))
        .execute(&pool)
        .await
        .unwrap_or_else(|e| panic!("{label} must pass the column CHECK to test the codec: {e}"));

        let error = load_tenant_administrative_effect(&pool, f.tenant, record_id)
            .await
            .expect_err(label);
        assert!(
            matches!(&error, sqlx::Error::Protocol(message)
                if message.contains("stored administrative effect record is malformed")),
            "{label} produced {error:?}"
        );

        sqlx::query("DELETE FROM administrative_effects WHERE record_id = $1")
            .bind(record_id.to_string())
            .execute(&pool)
            .await
            .expect("clear the malformed row");
    }
}

#[tokio::test]
async fn the_column_check_and_the_declared_kind_lists_are_the_same_vocabulary() {
    let Some(pool) = pool().await else { return };
    let f = fixture(&pool).await;

    // Every declared operation kind is accepted by the migration's CHECK...
    for kind in AdministrativeOperation::KINDS {
        let record_id = admission(&pool, f.tenant).await;
        sqlx::query(
            "INSERT INTO administrative_effects \
             (record_id, tenant_id, operation, submitted_reason, outcome, effected_at) \
             VALUES ($1, $2, $3, NULL, '{\"kind\":\"applied\"}'::jsonb, now())",
        )
        .bind(record_id.to_string())
        .bind(f.tenant.to_string())
        .bind(sqlx::types::Json(json!({"kind": kind})))
        .execute(&pool)
        .await
        .unwrap_or_else(|e| {
            panic!("the migration's CHECK rejects the declared operation kind `{kind}`: {e}")
        });
    }
    for kind in AdministrativeOutcome::KINDS {
        let record_id = admission(&pool, f.tenant).await;
        sqlx::query(
            "INSERT INTO administrative_effects \
             (record_id, tenant_id, operation, submitted_reason, outcome, effected_at) \
             VALUES ($1, $2, '{\"kind\":\"breaker_reset\"}'::jsonb, NULL, $3, now())",
        )
        .bind(record_id.to_string())
        .bind(f.tenant.to_string())
        .bind(sqlx::types::Json(json!({"kind": kind})))
        .execute(&pool)
        .await
        .unwrap_or_else(|e| {
            panic!("the migration's CHECK rejects the declared outcome kind `{kind}`: {e}")
        });
    }

    // ...and a kind outside either list is refused by the database, not merely
    // by the decoder. A CHECK that accepted anything would make the list above
    // pass while proving nothing.
    for (column, bad) in [
        ("operation", json!({"kind": "grant_revoke_all"})),
        ("outcome", json!({"kind": "partially_applied"})),
    ] {
        let record_id = admission(&pool, f.tenant).await;
        let (operation, outcome) = if column == "operation" {
            (bad.clone(), json!({"kind": "applied"}))
        } else {
            (json!({"kind": "breaker_reset"}), bad.clone())
        };
        let error = sqlx::query(
            "INSERT INTO administrative_effects \
             (record_id, tenant_id, operation, submitted_reason, outcome, effected_at) \
             VALUES ($1, $2, $3, NULL, $4, now())",
        )
        .bind(record_id.to_string())
        .bind(f.tenant.to_string())
        .bind(sqlx::types::Json(&operation))
        .bind(sqlx::types::Json(&outcome))
        .execute(&pool)
        .await
        .unwrap_err();
        assert!(
            error
                .as_database_error()
                .is_some_and(|e| e.is_check_violation()),
            "the {column} CHECK admitted `{bad}`: {error:?}"
        );
    }

    // A non-object in either column is refused too — `jsonb_typeof` is part of
    // the constraint precisely so a stored array cannot reach the decoder.
    let record_id = admission(&pool, f.tenant).await;
    let error = sqlx::query(
        "INSERT INTO administrative_effects \
         (record_id, tenant_id, operation, submitted_reason, outcome, effected_at) \
         VALUES ($1, $2, '[\"grant_revoke\"]'::jsonb, NULL, '{\"kind\":\"applied\"}'::jsonb, now())",
    )
    .bind(record_id.to_string())
    .bind(f.tenant.to_string())
    .execute(&pool)
    .await
    .unwrap_err();
    assert!(error
        .as_database_error()
        .is_some_and(|e| e.is_check_violation()));
}

#[tokio::test]
async fn the_migration_invents_no_history_and_relabels_no_admission() {
    let Some(pool) = pool().await else { return };
    let f = fixture(&pool).await;
    let record_id = admission(&pool, f.tenant).await;

    // Nothing is backfilled: the table is empty after migration even though
    // admissions exist, because an unrecorded outcome is not a recoverable one.
    assert_eq!(effect_rows(&pool).await, 0);

    // The admission's own columns are untouched — the effect is a separate row,
    // not a new field on the record an operator already reads.
    let row = sqlx::query(
        "SELECT decision, evaluation::text AS evaluation FROM authorization_records \
         WHERE record_id = $1",
    )
    .bind(record_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("read the admission");
    assert_eq!(row.get::<String, _>("decision"), "allowed");
    assert_eq!(
        row.get::<String, _>("evaluation"),
        "{\"kind\": \"boundary_checked\"}"
    );
    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT column_name::text FROM information_schema.columns \
         WHERE table_name = 'authorization_records' \
         ORDER BY column_name::text COLLATE \"C\"",
    )
    .fetch_all(&pool)
    .await
    .expect("read the admission columns");
    assert_eq!(
        columns,
        vec![
            "action",
            "actor",
            "boundary_id",
            "decided_at",
            "decision",
            "evaluation",
            "grant_id",
            "policy_digest",
            "policy_version",
            "reason",
            "record_id",
            "subject_id",
            "subject_kind",
            "target_kind",
            "target_tenant",
            "target_thread",
            "tenant_id",
        ],
        "the migration added or removed an authorization_records column"
    );
}

// ── The first producer: grant and boundary revocation (`SIGNOFF-REPAIR.3.3.4.8`) ──
//
// The controls above drive the storage primitive directly. These drive the HTTP
// route that now writes through it, because the properties `.8` claims are about
// the ROUTE: that its admission, its mutation and its evidence share one commit
// under one exclusive guard, where before it ran two guarded transactions with
// nothing ordering them against each other.

use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use std::net::SocketAddr;
use std::time::Duration;
use tenant_transaction::{transact, GuardError};
use tokio::sync::oneshot;
use tokio::time::timeout;

async fn serve(pool: &PgPool) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the control API");
    let addr = listener.local_addr().expect("the listener address");
    let router = api_router(pool.clone());
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });
    addr
}

async fn enroll(client: &reqwest::Client, base: &str, body: Value) -> Value {
    let response = client
        .post(format!("{base}/v1/enrollments"))
        .json(&body)
        .send()
        .await
        .expect("enrollment request");
    assert_eq!(response.status().as_u16(), 200, "enrollment must succeed");
    response.json().await.expect("enrollment json")
}

/// The revoke POST, returning status, the `x-reasonbraid-authorization` receipt
/// and the body. Every answer carries the receipt — that is the contract.
async fn revoke(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    tenant: &str,
    reason: &str,
) -> (u16, Option<String>, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(&json!({ "tenant_id": tenant, "reason": reason }))
        .send()
        .await
        .expect("revoke request");
    let status = response.status().as_u16();
    let receipt = response
        .headers()
        .get("x-reasonbraid-authorization")
        .map(|value| value.to_str().expect("an ASCII receipt").to_owned());
    (status, receipt, response.json().await.expect("revoke json"))
}

async fn admission_count(pool: &PgPool, tenant: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM authorization_records WHERE tenant_id = $1")
        .bind(tenant)
        .fetch_one(pool)
        .await
        .expect("count admissions")
}

/// A tenant with an administrating human and one role grant to revoke.
struct Enrolled {
    tenant: String,
    admin: String,
    admin_grant: String,
    role_grant: String,
    boundary: String,
}

async fn enrolled(client: &reqwest::Client, base: &str, name: &str) -> Enrolled {
    let human = enroll(client, base, json!({ "kind": "human", "name": name })).await;
    let tenant = human["tenant_id"].as_str().unwrap().to_owned();
    let role = enroll(
        client,
        base,
        json!({ "kind": "role", "name": format!("{name}-role"), "tenant_id": tenant }),
    )
    .await;
    Enrolled {
        tenant,
        admin: human["principal_id"].as_str().unwrap().to_owned(),
        admin_grant: human["grant_id"].as_str().unwrap().to_owned(),
        role_grant: role["grant_id"].as_str().unwrap().to_owned(),
        boundary: human["boundary_id"].as_str().unwrap().to_owned(),
    }
}

/// Hold one tenant guard until released, reporting when it is actually held so a
/// control never races its own fixture. The PRODUCTION runner, so the control
/// takes the same lock a revocation takes.
struct Holder {
    release: oneshot::Sender<()>,
    job: tokio::task::JoinHandle<()>,
}

async fn hold(pool: &PgPool, tenant: TenantId, mode: GuardMode) -> Holder {
    let pool = pool.clone();
    let (entered_tx, entered) = oneshot::channel();
    let (release, release_rx) = oneshot::channel();
    let job = tokio::spawn(async move {
        transact(&pool, &[(tenant, mode)], move |_| {
            Box::pin(async move {
                let _ = entered_tx.send(());
                let _ = release_rx.await;
                Ok::<(), GuardError>(())
            })
        })
        .await
        .expect("the holder's guarded transaction completes");
    });
    timeout(Duration::from_secs(5), entered)
        .await
        .expect("the holder acquires its guard")
        .expect("the holder reports entry");
    Holder { release, job }
}

impl Holder {
    async fn release(self) {
        let _ = self.release.send(());
        timeout(Duration::from_secs(5), self.job)
            .await
            .expect("the holder finishes")
            .expect("the holder's task joins");
    }
}

#[tokio::test]
async fn a_revocation_commits_its_admission_mutation_and_effect_together() {
    let Some(pool) = pool().await else { return };
    let base = format!("http://{}", serve(&pool).await);
    let client = reqwest::Client::new();
    let alice = enrolled(&client, &base, "revocation-applies").await;
    let tenant: TenantId = alice.tenant.parse().unwrap();
    let before = admission_count(&pool, &alice.tenant).await;

    let (status, receipt, body) = revoke(
        &client,
        &base,
        &format!("/v1/admin/grants/{}/revoke", alice.role_grant),
        &alice.admin,
        &alice.tenant,
        "role retired",
    )
    .await;
    assert_eq!(status, 200, "an eligible administrator revokes: {body}");
    let receipt = receipt.expect("every answer carries its admission receipt");
    assert_eq!(body["grant_id"], json!(alice.role_grant));

    assert_eq!(grant_status(&pool, &alice.role_grant).await, "revoked");
    assert_eq!(epoch(&pool, tenant).await, 1);
    assert_eq!(
        admission_count(&pool, &alice.tenant).await,
        before + 1,
        "exactly one admission per request"
    );

    let effect = load_tenant_administrative_effect(&pool, tenant, receipt.parse().unwrap())
        .await
        .expect("read the effect back")
        .expect("the effect committed with its mutation");
    assert!(effect.outcome.changed_protected_state());
    assert_eq!(
        effect.operation,
        AdministrativeOperation::GrantRevoke {
            grant_id: target(&alice.role_grant)
        }
    );
    assert_eq!(
        effect.submitted_reason.as_ref().map(|r| r.as_str()),
        Some("role retired"),
        "the submitted reason is persisted with the mutation, not merely validated"
    );
    // The recorded instant is the transaction's own database time, so it agrees
    // with the response rather than with a clock read afterwards.
    assert_eq!(
        body["revoked_at"].as_str().unwrap(),
        effect.effected_at.to_rfc3339()
    );
}

#[tokio::test]
async fn a_repeated_revocation_records_a_no_op_and_advances_no_epoch() {
    let Some(pool) = pool().await else { return };
    let base = format!("http://{}", serve(&pool).await);
    let client = reqwest::Client::new();
    let alice = enrolled(&client, &base, "revocation-repeats").await;
    let tenant: TenantId = alice.tenant.parse().unwrap();
    let path = format!("/v1/admin/grants/{}/revoke", alice.role_grant);

    let (status, _, _) = revoke(&client, &base, &path, &alice.admin, &alice.tenant, "first").await;
    assert_eq!(status, 200);
    assert_eq!(epoch(&pool, tenant).await, 1);

    let (status, receipt, body) =
        revoke(&client, &base, &path, &alice.admin, &alice.tenant, "again").await;
    assert_eq!(status, 409, "a repeat is refused: {body}");
    assert_eq!(
        epoch(&pool, tenant).await,
        1,
        "a repeated revocation advances no epoch (`SIGNOFF-REPAIR.3.1`)"
    );
    let effect = load_tenant_administrative_effect(
        &pool,
        tenant,
        receipt
            .expect("a refusal carries its receipt")
            .parse()
            .unwrap(),
    )
    .await
    .expect("read back")
    .expect("the no-op is recorded, not merely returned");
    assert!(!effect.outcome.changed_protected_state());
    let AdministrativeOutcome::NoOp { detail } = &effect.outcome else {
        panic!("expected a recorded no-op, got {:?}", effect.outcome)
    };
    assert!(detail.as_str().contains("already revoked"));
}

#[tokio::test]
async fn a_missing_and_a_foreign_target_are_one_answer_that_records_a_refusal() {
    let Some(pool) = pool().await else { return };
    let base = format!("http://{}", serve(&pool).await);
    let client = reqwest::Client::new();
    let alice = enrolled(&client, &base, "revocation-outsider").await;
    let victim = enrolled(&client, &base, "revocation-victim").await;
    let tenant: TenantId = alice.tenant.parse().unwrap();
    let victim_tenant: TenantId = victim.tenant.parse().unwrap();

    let absent = format!("grt_{}", uuid::Uuid::now_v7());
    let mut answers = Vec::new();
    for target_id in [&absent, &victim.role_grant] {
        let (status, receipt, body) = revoke(
            &client,
            &base,
            &format!("/v1/admin/grants/{target_id}/revoke"),
            &alice.admin,
            &alice.tenant,
            "probing",
        )
        .await;
        assert_eq!(status, 404, "missing and foreign both 404: {body}");
        let effect = load_tenant_administrative_effect(
            &pool,
            tenant,
            receipt.expect("receipt").parse().unwrap(),
        )
        .await
        .expect("read back")
        .expect("an admitted caller's refused operation is recorded");
        let AdministrativeOutcome::Refused { code, .. } = &effect.outcome else {
            panic!("expected a recorded refusal, got {:?}", effect.outcome)
        };
        assert_eq!(*code, AdministrativeRefusal::NotFound);
        answers.push(body["message"].as_str().unwrap().to_owned());
    }
    // The evidence lives in the CALLER's tenant and names only the id the caller
    // supplied, so the two cases stay indistinguishable to it.
    assert_ne!(answers[0], answers[1], "each message names its own id");
    assert_eq!(grant_status(&pool, &victim.role_grant).await, "active");
    assert_eq!(epoch(&pool, victim_tenant).await, 0);
    assert_eq!(
        admission_count(&pool, &victim.tenant).await,
        0,
        "a foreign attempt leaves no trace in the victim's tenant"
    );
}

#[tokio::test]
async fn the_reason_bounds_are_a_wire_contract_and_refuse_before_any_effect() {
    let Some(pool) = pool().await else { return };
    let base = format!("http://{}", serve(&pool).await);
    let client = reqwest::Client::new();
    let alice = enrolled(&client, &base, "revocation-reason").await;
    let tenant: TenantId = alice.tenant.parse().unwrap();
    let path = format!("/v1/admin/grants/{}/revoke", alice.role_grant);

    // ⚠️ The documented wire change: blankness was already refused; the byte
    // ceiling and the control-character rule are new.
    for (label, reason) in [
        ("blank", "   ".to_owned()),
        ("a control character", "role\nretired".to_owned()),
        ("one byte over the ceiling", "x".repeat(1025)),
    ] {
        let before = effect_rows(&pool).await;
        let (status, receipt, body) =
            revoke(&client, &base, &path, &alice.admin, &alice.tenant, &reason).await;
        assert_eq!(status, 400, "{label} is refused: {body}");
        assert_eq!(
            effect_rows(&pool).await,
            before,
            "{label} never became an operation, so it records no effect"
        );
        // The admission still commits: an admitted caller who sent something
        // malformed is a fact worth keeping, and that is the established order.
        let receipt = receipt.expect("a malformed request still names its admission");
        assert!(
            load_tenant_administrative_effect(&pool, tenant, receipt.parse().unwrap())
                .await
                .expect("read back")
                .is_none()
        );
        assert_eq!(grant_status(&pool, &alice.role_grant).await, "active");
    }

    // Exactly at the ceiling succeeds, so the bound is checked at its edge.
    let (status, _, body) = revoke(
        &client,
        &base,
        &path,
        &alice.admin,
        &alice.tenant,
        &"x".repeat(1024),
    )
    .await;
    assert_eq!(
        status, 200,
        "a reason exactly at the ceiling is usable: {body}"
    );
    assert_eq!(epoch(&pool, tenant).await, 1);
}

#[tokio::test]
async fn a_caller_without_administration_is_denied_and_records_no_effect() {
    let Some(pool) = pool().await else { return };
    let base = format!("http://{}", serve(&pool).await);
    let client = reqwest::Client::new();
    let alice = enrolled(&client, &base, "revocation-denied").await;
    let tenant: TenantId = alice.tenant.parse().unwrap();
    let role = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "ordinary", "tenant_id": alice.tenant }),
    )
    .await;
    let before = effect_rows(&pool).await;

    let (status, receipt, body) = revoke(
        &client,
        &base,
        &format!("/v1/admin/grants/{}/revoke", alice.role_grant),
        role["principal_id"].as_str().unwrap(),
        &alice.tenant,
        "not mine to revoke",
    )
    .await;
    assert_eq!(status, 403, "a role without tenant_admin is denied: {body}");
    assert_eq!(
        effect_rows(&pool).await,
        before,
        "a denial never became an operation; the admission record already says denied"
    );
    assert!(load_tenant_administrative_effect(
        &pool,
        tenant,
        receipt.expect("a denial names its record").parse().unwrap()
    )
    .await
    .expect("read back")
    .is_none());
    assert_eq!(grant_status(&pool, &alice.role_grant).await, "active");
    assert_eq!(epoch(&pool, tenant).await, 0);
}

#[tokio::test]
async fn a_revocation_waits_for_the_exclusive_guard_and_then_applies() {
    let Some(pool) = pool().await else { return };
    let base = format!("http://{}", serve(&pool).await);
    let client = reqwest::Client::new();
    let alice = enrolled(&client, &base, "revocation-waits").await;
    let tenant: TenantId = alice.tenant.parse().unwrap();

    let holder = hold(&pool, tenant, GuardMode::Exclusive).await;
    let path = format!("/v1/admin/grants/{}/revoke", alice.role_grant);
    let (base2, client2, admin, tenant_key) = (
        base.clone(),
        client.clone(),
        alice.admin.clone(),
        alice.tenant.clone(),
    );
    let request = tokio::spawn(async move {
        revoke(
            &client2,
            &base2,
            &path,
            &admin,
            &tenant_key,
            "after the wait",
        )
        .await
    });

    // While the guard is held the revocation cannot have applied. Before
    // `.3.3.4.8` the mutation ran in its OWN transaction after a shared-guard
    // admission, so it was not fenced by this at all.
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(grant_status(&pool, &alice.role_grant).await, "active");
    assert_eq!(epoch(&pool, tenant).await, 0);
    assert!(!request.is_finished(), "the revocation is waiting");

    holder.release().await;
    let (status, receipt, body) = timeout(Duration::from_secs(10), request)
        .await
        .expect("the revocation completes after the guard is released")
        .expect("the request task joins");
    assert_eq!(status, 200, "it then applies: {body}");
    assert_eq!(grant_status(&pool, &alice.role_grant).await, "revoked");
    assert_eq!(epoch(&pool, tenant).await, 1);
    assert!(receipt.is_some());
}

#[tokio::test]
async fn authority_that_ends_while_a_revocation_waits_refuses_it() {
    let Some(pool) = pool().await else { return };
    let base = format!("http://{}", serve(&pool).await);
    let client = reqwest::Client::new();
    let alice = enrolled(&client, &base, "revocation-queued").await;
    let tenant: TenantId = alice.tenant.parse().unwrap();

    let holder = hold(&pool, tenant, GuardMode::Exclusive).await;
    let path = format!("/v1/admin/grants/{}/revoke", alice.role_grant);
    let (base2, client2, admin, tenant_key) = (
        base.clone(),
        client.clone(),
        alice.admin.clone(),
        alice.tenant.clone(),
    );
    let request = tokio::spawn(async move {
        revoke(&client2, &base2, &path, &admin, &tenant_key, "queued").await
    });
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(!request.is_finished(), "the revocation is waiting");

    // End the caller's OWN administration underneath the blocked request. The
    // decision time is sampled after the guard wait, so this must be seen.
    sqlx::query("UPDATE authority_grants SET status = 'revoked' WHERE grant_id = $1")
        .bind(&alice.admin_grant)
        .execute(&pool)
        .await
        .expect("end the caller's authority");
    holder.release().await;

    let (status, receipt, body) = timeout(Duration::from_secs(10), request)
        .await
        .expect("the revocation completes")
        .expect("the request task joins");
    assert_eq!(
        status, 403,
        "authority that ended during the wait is gone: {body}"
    );
    assert_eq!(
        grant_status(&pool, &alice.role_grant).await,
        "active",
        "a denied revocation changes no protected target"
    );
    // The epoch is still 0, and that is the point: the raw UPDATE above ends the
    // caller's authority WITHOUT the application's epoch bump, so any advance
    // here would have had to come from the blocked request itself. It did not.
    assert_eq!(
        epoch(&pool, tenant).await,
        0,
        "a denied revocation advances no revocation epoch"
    );
    assert!(
        load_tenant_administrative_effect(
            &pool,
            tenant,
            receipt.expect("receipt").parse().unwrap()
        )
        .await
        .expect("read back")
        .is_none(),
        "a denial records no effect"
    );
}

#[tokio::test]
async fn a_boundary_revocation_takes_the_same_shape_and_fences_the_next_request() {
    let Some(pool) = pool().await else { return };
    let base = format!("http://{}", serve(&pool).await);
    let client = reqwest::Client::new();
    let alice = enrolled(&client, &base, "revocation-boundary").await;
    let tenant: TenantId = alice.tenant.parse().unwrap();
    let path = format!("/v1/admin/boundaries/{}/revoke", alice.boundary);

    let (status, receipt, body) = revoke(
        &client,
        &base,
        &path,
        &alice.admin,
        &alice.tenant,
        "tenant frozen",
    )
    .await;
    assert_eq!(status, 200, "the boundary revokes: {body}");
    let effect = load_tenant_administrative_effect(
        &pool,
        tenant,
        receipt.expect("receipt").parse().unwrap(),
    )
    .await
    .expect("read back")
    .expect("the boundary effect committed");
    assert_eq!(
        effect.operation,
        AdministrativeOperation::BoundaryRevoke {
            boundary_id: target(&alice.boundary)
        }
    );
    assert!(effect.outcome.changed_protected_state());
    assert_eq!(epoch(&pool, tenant).await, 1);

    // A frozen boundary fences the next administrative write, and the repeat is
    // the recorded no-op rather than a second epoch bump.
    let (status, _, body) =
        revoke(&client, &base, &path, &alice.admin, &alice.tenant, "again").await;
    assert_eq!(
        status, 403,
        "the frozen boundary now refuses its own admin: {body}"
    );
    assert_eq!(epoch(&pool, tenant).await, 1);
}
