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
    AdministrativeReason, AdministrativeTargetId, AuthorizationRecordId, KnownReasonCode, TenantId,
};
use reasonbraid_server::{load_tenant_administrative_effect, record_administrative_effect_in_tx};
use serde_json::json;
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
            code: KnownReasonCode::InvalidTransition,
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
            "a refusal naming a code outside the registry",
            json!({"kind": "breaker_reset"}),
            json!({"kind": "refused", "code": "quota_exhausted", "detail": "x"}),
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
