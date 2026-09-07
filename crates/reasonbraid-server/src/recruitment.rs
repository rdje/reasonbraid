//! The open-call artifact + the typed recruitment responses (`PHASE-3.4.2`,
//! backlog 29/30): the §10.5 call spec rides the SAME thread invitation
//! machinery (ADR-015's baseline) — the panel snapshot feeds the thread's
//! participant flow, never a parallel system. The response vocabulary is the
//! §10.5 set, typed at the boundary.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

/// The §10.5 response vocabulary (typed, tagged on the wire).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecruitmentResponse {
    Join,
    Observe,
    Decline {
        #[serde(default)]
        reason: Option<String>,
    },
    Defer {
        until: DateTime<Utc>,
    },
    ConditionalJoin {
        requirements: Value,
    },
    Recommend {
        capability_or_visible_role: String,
    },
    RequestContext {
        fields: Vec<String>,
    },
    Recuse {
        reason_class: String,
    },
}

impl RecruitmentResponse {
    pub fn kind(&self) -> &'static str {
        match self {
            RecruitmentResponse::Join => "join",
            RecruitmentResponse::Observe => "observe",
            RecruitmentResponse::Decline { .. } => "decline",
            RecruitmentResponse::Defer { .. } => "defer",
            RecruitmentResponse::ConditionalJoin { .. } => "conditional_join",
            RecruitmentResponse::Recommend { .. } => "recommend",
            RecruitmentResponse::RequestContext { .. } => "request_context",
            RecruitmentResponse::Recuse { .. } => "recuse",
        }
    }
}

/// The dev-scale storm controls (`.4.3`): the per-tenant + per-initiator
/// open-call fan-out caps — the §10.7 "per-tenant, initiator … fan-out
/// limits" at the dev profile's scale. The storm-grade circuit breakers +
/// the quiet hours + the depth/cycle machinery are the named deferrals (the
/// triggers ride the leaf's Done line).
pub const MAX_OPEN_CALLS_PER_TENANT: i64 = 8;
pub const MAX_OPEN_CALLS_PER_INITIATOR: i64 = 4;

/// The initiator's current open calls (the fan-out cap's count).
pub async fn open_calls_by(
    pool: &sqlx::PgPool,
    tenant_id: Option<&str>,
    initiator: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let mut query = String::from("SELECT count(*) FROM recruitment_calls WHERE status = 'open'");
    if tenant_id.is_some() {
        query.push_str(" AND tenant_id = $1");
    }
    if initiator.is_some() {
        query.push_str(if tenant_id.is_some() {
            " AND initiator = $2"
        } else {
            " AND initiator = $1"
        });
    }
    let mut q = sqlx::query_scalar(&query);
    if let Some(t) = tenant_id {
        q = q.bind(t);
    }
    if let Some(i) = initiator {
        q = q.bind(i);
    }
    q.fetch_one(pool).await
}

/// The call spec as stored (the expression rides as the `.3` typed shape).
#[derive(Debug, Clone, Serialize)]
pub struct CallRow {
    pub call_id: String,
    pub tenant_id: String,
    pub thread_id: String,
    pub initiator: String,
    pub expression: Value,
    pub min_participants: i32,
    pub max_participants: i32,
    pub recommendations_allowed: bool,
    pub advertises_at: DateTime<Utc>,
    pub join_deadline: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: String,
}

/// The `open_call` inputs: the call's identity, its spec expression, and its
/// participation/deadline policy — one typed argument instead of ten.
pub struct OpenCallParams<'a> {
    pub tenant_id: &'a str,
    pub thread_id: &'a str,
    pub initiator: &'a str,
    pub expression: &'a Value,
    pub min_participants: i32,
    pub max_participants: i32,
    pub recommendations_allowed: bool,
    pub join_deadline: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Open a call: the spec's expression is stored verbatim (the server resolves
/// it at every response).
pub async fn open_call(
    pool: &sqlx::PgPool,
    params: OpenCallParams<'_>,
) -> Result<String, sqlx::Error> {
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO recruitment_calls \
         (call_id, tenant_id, thread_id, initiator, expression, min_participants, \
          max_participants, recommendations_allowed, advertises_at, join_deadline, expires_at, status) \
         VALUES ('cal_' || gen_random_uuid()::text, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'open') \
         RETURNING call_id",
    )
    .bind(params.tenant_id)
    .bind(params.thread_id)
    .bind(params.initiator)
    .bind(params.expression)
    .bind(params.min_participants)
    .bind(params.max_participants)
    .bind(params.recommendations_allowed)
    .bind(now)
    .bind(params.join_deadline)
    .bind(params.expires_at)
    .fetch_one(pool)
    .await
    .map(|row: sqlx::postgres::PgRow| row.get::<String, _>(0))
}

/// Record one typed response (one per respondent — the second overwrites is a
/// conflict, never a silent merge).
pub async fn record_response(
    pool: &sqlx::PgPool,
    call_id: &str,
    respondent: &str,
    response: &RecruitmentResponse,
) -> Result<(), sqlx::Error> {
    let payload = match response {
        RecruitmentResponse::Decline { reason } => {
            serde_json::json!({ "reason": reason })
        }
        RecruitmentResponse::Defer { until } => {
            serde_json::json!({ "until": until.to_rfc3339() })
        }
        RecruitmentResponse::ConditionalJoin { requirements } => {
            serde_json::json!({ "requirements": requirements })
        }
        RecruitmentResponse::Recommend {
            capability_or_visible_role,
        } => {
            serde_json::json!({ "target": capability_or_visible_role })
        }
        RecruitmentResponse::RequestContext { fields } => {
            serde_json::json!({ "fields": fields })
        }
        RecruitmentResponse::Recuse { reason_class } => {
            serde_json::json!({ "reason_class": reason_class })
        }
        _ => serde_json::json!({}),
    };
    sqlx::query(
        "INSERT INTO recruitment_responses (response_id, call_id, respondent, response_kind, payload) \
         VALUES ('rsp_' || gen_random_uuid()::text, $1, $2, $3, $4) \
         ON CONFLICT (call_id, respondent) DO UPDATE SET response_kind = $3, payload = $4",
    )
    .bind(call_id)
    .bind(respondent)
    .bind(response.kind())
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(())
}

/// The responses so far (the panel snapshot's raw material).
pub async fn responses(
    pool: &sqlx::PgPool,
    call_id: &str,
) -> Result<Vec<(String, String, Value, DateTime<Utc>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT respondent, response_kind, payload, created_at \
         FROM recruitment_responses WHERE call_id = $1 ORDER BY created_at, respondent",
    )
    .bind(call_id)
    .fetch_all(pool)
    .await
}

/// The durable `recruitment_calls` row shape (the query's tuple type).
type CallTuple = (
    String,
    String,
    String,
    String,
    Value,
    i32,
    i32,
    bool,
    DateTime<Utc>,
    DateTime<Utc>,
    DateTime<Utc>,
    String,
);

/// The call row (the response gate needs the expression + the deadlines).
pub async fn call(pool: &sqlx::PgPool, call_id: &str) -> Result<Option<CallRow>, sqlx::Error> {
    let row: Option<CallTuple> = sqlx::query_as(
        "SELECT call_id, tenant_id, thread_id, initiator, expression, min_participants, \
                max_participants, recommendations_allowed, advertises_at, join_deadline, expires_at, status \
         FROM recruitment_calls WHERE call_id = $1",
    )
    .bind(call_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(
        |(
            call_id,
            tenant_id,
            thread_id,
            initiator,
            expression,
            min_participants,
            max_participants,
            recommendations_allowed,
            advertises_at,
            join_deadline,
            expires_at,
            status,
        )| CallRow {
            call_id,
            tenant_id,
            thread_id,
            initiator,
            expression,
            min_participants,
            max_participants,
            recommendations_allowed,
            advertises_at,
            join_deadline,
            expires_at,
            status,
        },
    ))
}

/// Snapshot the panel at the close: the JOINERS (ranked by the default
/// preferences, capped at the max) + the per-panelist selection explanation
/// (the stage-1 reasons + the stage-2 features — the visibility-safe strings).
pub async fn snapshot_panel(
    pool: &sqlx::PgPool,
    call_id: &str,
    ranked: &[crate::matching::RankedCandidate],
    dependence_indicators: &[crate::dependence::DependenceIndicator],
) -> Result<(), sqlx::Error> {
    let panel: Vec<Value> = ranked
        .iter()
        .map(|r| serde_json::json!(r.role_id))
        .collect();
    // The selection explanation: the per-panelist reasons + features AND the
    // panel's dependence indicators (`.6.2`) — the diversity facts beside the
    // ranking facts.
    let explanation = serde_json::json!({
        "per_panelist": ranked
            .iter()
            .map(|r| {
                serde_json::json!({
                    "role_id": r.role_id,
                    "stage1_reasons": r.stage1_reasons,
                    "features": r.features,
                    "total": r.total,
                })
            })
            .collect::<Vec<_>>(),
        "dependence_indicators": dependence_indicators,
    });
    sqlx::query("INSERT INTO recruitment_panels (call_id, panel, explanation) VALUES ($1, $2, $3)")
        .bind(call_id)
        .bind(serde_json::json!(panel))
        .bind(explanation)
        .execute(pool)
        .await?;
    sqlx::query("UPDATE recruitment_calls SET status = 'closed' WHERE call_id = $1")
        .bind(call_id)
        .execute(pool)
        .await?;
    Ok(())
}
