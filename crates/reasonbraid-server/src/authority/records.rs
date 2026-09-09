//! Strict interpretation of stored authorization evidence. Corruption must not
//! become a guessed decision, a discarded subject or a panic during inspection.
use chrono::{DateTime, Utc};
use reasonbraid_core::{
    actor_handle_for_subject, AuthorizationDecisionRecord, AuthorizationEvaluation,
    AuthorizationRecordId, Decision, GrantAction, ResourceTarget, TargetSelector, TenantId,
    ThreadId,
};
use serde_json::Value;
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
struct RecordRow {
    record_id: String,
    tenant_id: String,
    actor: String,
    subject_kind: Option<String>,
    subject_id: Option<String>,
    boundary_id: Option<String>,
    grant_id: Option<String>,
    action: String,
    target_kind: String,
    target_tenant: String,
    target_thread: Option<String>,
    decision: String,
    reason: Option<String>,
    policy_digest: String,
    policy_version: String,
    decided_at: DateTime<Utc>,
    evaluation: Value,
}

const SELECT_RECORD: &str =
    "SELECT record_id, tenant_id, actor, subject_kind, subject_id, boundary_id, grant_id, \
     action, target_kind, target_tenant, target_thread, decision, reason, policy_digest, \
     policy_version, decided_at, evaluation FROM authorization_records";

fn malformed() -> sqlx::Error {
    sqlx::Error::Protocol("stored authorization record is malformed".into())
}

impl TryFrom<RecordRow> for AuthorizationDecisionRecord {
    type Error = sqlx::Error;

    fn try_from(row: RecordRow) -> Result<Self, Self::Error> {
        let decision = match (row.decision.as_str(), row.reason) {
            ("allowed", None) => Decision::Allowed,
            ("denied", Some(reason)) => Decision::Denied { reason },
            _ => return Err(malformed()),
        };
        let tenant_id: TenantId = row.tenant_id.parse().map_err(|_| malformed())?;
        let target_tenant: TenantId = row.target_tenant.parse().map_err(|_| malformed())?;
        if tenant_id != target_tenant {
            return Err(malformed());
        }
        let target = match (row.target_kind.as_str(), row.target_thread) {
            ("tenant", None) => ResourceTarget::Tenant { tenant_id },
            ("thread", Some(thread_id)) => ResourceTarget::Thread {
                tenant_id,
                thread_id: thread_id.parse().map_err(|_| malformed())?,
            },
            _ => return Err(malformed()),
        };
        let subject = match (row.subject_kind.as_deref(), row.subject_id.as_deref()) {
            (None, None) => None,
            (Some(kind), Some(id)) => {
                Some(super::subject_from_parts(kind, id).ok_or_else(malformed)?)
            }
            _ => return Err(malformed()),
        };
        let record = Self {
            record_id: row.record_id.parse().map_err(|_| malformed())?,
            tenant_id,
            actor: row.actor.parse().map_err(|_| malformed())?,
            subject,
            boundary_id: row.boundary_id,
            grant_id: row.grant_id,
            action: row.action.parse().map_err(|_| malformed())?,
            target,
            decision,
            evaluation: serde_json::from_value(row.evaluation).map_err(|_| malformed())?,
            policy_digest: row.policy_digest,
            policy_version: row.policy_version,
            decided_at: row.decided_at,
        };
        if let AuthorizationEvaluation::TenantAdminInspection {
            principal,
            boundary_status,
            grant_selector,
            ..
        } = &record.evaluation
        {
            if record.action != GrantAction::TenantAdmin
                || !matches!(record.target, ResourceTarget::Tenant { .. })
                || record.subject.is_some()
                || record.actor != actor_handle_for_subject(principal)
                || record.boundary_id.is_some() != boundary_status.is_some()
                || record.grant_id.is_some() != grant_selector.is_some()
                || record.boundary_id.is_some() != record.grant_id.is_some()
                || (record.decision == Decision::Allowed
                    && !matches!(grant_selector, Some(TargetSelector::TenantWide)))
            {
                return Err(malformed());
            }
        }
        Ok(record)
    }
}

/// Assemble a stored record with explicit provenance; malformed evidence is an
/// error. This storage API is not itself a caller-authorization boundary.
pub async fn load_authorization_record(
    pool: &PgPool,
    record_id: &str,
) -> Result<Option<AuthorizationDecisionRecord>, sqlx::Error> {
    let row: Option<RecordRow> = sqlx::query_as(&format!("{SELECT_RECORD} WHERE record_id = $1"))
        .bind(record_id)
        .fetch_optional(pool)
        .await?;
    row.map(AuthorizationDecisionRecord::try_from).transpose()
}

/// Exact tenant-scoped evidence lookup. Filter before decoding, so foreign
/// malformed evidence cannot be distinguished from an absent record. The caller
/// must separately commit the named inspection admission before using this API.
pub(crate) async fn load_tenant_authorization_record(
    pool: &PgPool,
    tenant_id: TenantId,
    record_id: AuthorizationRecordId,
) -> Result<Option<AuthorizationDecisionRecord>, sqlx::Error> {
    let row: Option<RecordRow> = sqlx::query_as(&format!(
        "{SELECT_RECORD} WHERE tenant_id = $1 AND record_id = $2"
    ))
    .bind(tenant_id.to_string())
    .bind(record_id.to_string())
    .fetch_optional(pool)
    .await?;
    row.map(AuthorizationDecisionRecord::try_from).transpose()
}

/// Existing thread-audit inventory, using the same evidence decoder as exact
/// lookup. Caller authorization belongs to the existing HTTP inspection gate.
pub(crate) async fn load_thread_authorization_records(
    pool: &PgPool,
    tenant_id: TenantId,
    thread_id: ThreadId,
) -> Result<Vec<AuthorizationDecisionRecord>, sqlx::Error> {
    let rows: Vec<RecordRow> = sqlx::query_as(&format!(
        "{SELECT_RECORD} WHERE tenant_id = $1 AND target_kind = 'thread' \
         AND target_thread = $2 ORDER BY decided_at, record_id COLLATE \"C\""
    ))
    .bind(tenant_id.to_string())
    .bind(thread_id.to_string())
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(AuthorizationDecisionRecord::try_from)
        .collect()
}
