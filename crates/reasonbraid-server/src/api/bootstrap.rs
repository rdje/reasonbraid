//! Recovery of a client-known request whose server-generated tenant is unknown.
//! Only routing metadata crosses an undeclared tenant scope. A redirect aborts
//! the provisional transaction before the recorded tenant is guarded and read.

use std::time::Duration;

use reasonbraid_core::{HumanPrincipalId, RequestId, TenantId};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::time::Instant;

use super::{authority, enroll_in_guard, ControlApiError, EnrollRequest, EnrollResponse};
use super::{GuardMode, Json, StatusCode, TenantTransaction};
use authority::{AuthorityTransactionError as GuardError, Limits};

const TOTAL: Duration = Duration::from_secs(15);

fn canonical_v7(id: &uuid::Uuid) -> bool {
    id.get_version_num() == 7 && id.get_variant() == uuid::Variant::RFC4122
}

pub(super) fn validate_key(raw: &str, req: &EnrollRequest) -> Result<RequestId, ControlApiError> {
    if req.kind != "human" || req.tenant_id.is_some() {
        return Err(ControlApiError::invalid_command(
            "bootstrap_request_id is valid only for a new-human bootstrap without tenant_id",
        ));
    }
    let key = (raw.len() == 40)
        .then(|| raw.parse::<RequestId>().ok())
        .flatten()
        .filter(|key| key.to_string() == raw && canonical_v7(key.as_uuid()));
    key.ok_or_else(|| {
        ControlApiError::invalid_command("bootstrap_request_id must be a canonical req_ UUIDv7")
    })
}

fn corrupt() -> ControlApiError {
    // Do not reflect stored payloads, foreign identities or caller names.
    ControlApiError::internal_with_log("invalid bootstrap request/outcome binding".into())
}

#[derive(Debug)]
enum AttemptError {
    Redirect(TenantId),
    Api(ControlApiError),
}

impl From<ControlApiError> for AttemptError {
    fn from(error: ControlApiError) -> Self {
        Self::Api(error)
    }
}

impl From<GuardError> for AttemptError {
    fn from(error: GuardError) -> Self {
        Self::Api(error.into())
    }
}

impl From<sqlx::Error> for AttemptError {
    fn from(error: sqlx::Error) -> Self {
        Self::Api(error.into())
    }
}

pub(super) async fn enroll(
    pool: &PgPool,
    req: EnrollRequest,
    key: RequestId,
    candidate: TenantId,
) -> Result<Json<EnrollResponse>, ControlApiError> {
    let deadline = Instant::now() + TOTAL;
    let mut tenant = candidate;
    for redirected in [false, true] {
        let req = req.clone();
        // Floor to the owner's whole-millisecond limits. Each transaction owns
        // its timeout and COMMIT phase; an outer timeout would erase that phase.
        let remaining = deadline.saturating_duration_since(Instant::now());
        let remaining = Duration::from_millis(remaining.as_millis() as u64);
        if remaining.is_zero() {
            return Err(GuardError::Deadline.into());
        }
        let limits = Limits::shortened(
            remaining.min(Duration::from_secs(5)),
            remaining.min(Duration::from_secs(10)),
            remaining,
        )?;
        let result = authority::transact_with_error(
            pool,
            &[(tenant, GuardMode::Exclusive)],
            limits,
            move |tx| Box::pin(attempt(tx, req, key, tenant, redirected)),
        )
        .await;
        match result {
            Ok(response) => return Ok(response),
            Err(AttemptError::Api(error)) => return Err(error),
            Err(AttemptError::Redirect(recorded)) if !redirected => tenant = recorded,
            Err(AttemptError::Redirect(_)) => return Err(corrupt()),
        }
    }
    Err(corrupt())
}

/// The only unguarded foreign fact permitted is the immutable routing tenant.
async fn routing(
    tx: &mut TenantTransaction<'_>,
    key: RequestId,
    tenant: TenantId,
) -> Result<Option<TenantId>, AttemptError> {
    let raw: Option<String> =
        sqlx::query_scalar("SELECT tenant_id FROM tenant_bootstrap_requests WHERE request_id = $1")
            .bind(key.to_string())
            .fetch_optional(tx.connection(tenant, GuardMode::Exclusive)?)
            .await?;
    raw.map(|raw| {
        raw.parse::<TenantId>()
            .ok()
            .filter(|id| id.to_string() == raw && canonical_v7(id.as_uuid()))
            .ok_or_else(|| corrupt().into())
    })
    .transpose()
}

async fn attempt(
    tx: &mut TenantTransaction<'_>,
    req: EnrollRequest,
    key: RequestId,
    tenant: TenantId,
    redirected: bool,
) -> Result<Json<EnrollResponse>, AttemptError> {
    if redirected {
        // Never fall back to creation if the supposedly immutable mapping is
        // missing or has moved. Read the full outcome only in this tenant scope.
        return replay(tx, key, tenant, &req.name).await.map_err(Into::into);
    }
    if let Some(recorded) = routing(tx, key, tenant).await? {
        if recorded != tenant {
            return Err(AttemptError::Redirect(recorded));
        }
        return replay(tx, key, tenant, &req.name).await.map_err(Into::into);
    }

    let name = req.name.clone();
    let Json(mut response) = enroll_in_guard(tx, req, "human", tenant).await?;
    response.bootstrap_request_id = Some(key.to_string());
    let outcome = serde_json::to_value(&response).map_err(|_| corrupt())?;
    // Validate the creation codec too, before storing or acknowledging it.
    decode(outcome.clone(), key, tenant, &name)?;
    let inserted = sqlx::query(
        "INSERT INTO tenant_bootstrap_requests \
         (request_id, tenant_id, request_name, outcome_version, outcome) \
         VALUES ($1, $2, $3, 1, $4) ON CONFLICT (request_id) DO NOTHING",
    )
    .bind(key.to_string())
    .bind(tenant.to_string())
    .bind(name)
    .bind(outcome)
    .execute(tx.connection(tenant, GuardMode::Exclusive)?)
    .await?
    .rows_affected();
    if inserted == 1 {
        return Ok(Json(response));
    }
    // ON CONFLICT can wait for a winner invisible to its statement snapshot.
    // READ COMMITTED gives this new routing statement the committed mapping.
    let recorded = routing(tx, key, tenant).await?.ok_or_else(corrupt)?;
    if recorded == tenant {
        return Err(corrupt().into());
    }
    Err(AttemptError::Redirect(recorded))
}

/// Version one stores every successful keyed field, with no optional/defaulted
/// IDs and no positional sequence form. It describes historical creation, not
/// the current liveness or permissions of its grant.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Outcome {
    tenant_id: String,
    principal_id: String,
    kind: String,
    name: String,
    boundary_id: String,
    grant_id: String,
    bootstrap_request_id: String,
    replayed: bool,
}

fn decode(
    value: serde_json::Value,
    key: RequestId,
    tenant: TenantId,
    name: &str,
) -> Result<Outcome, ControlApiError> {
    if !value.is_object() {
        return Err(corrupt());
    }
    let outcome: Outcome = serde_json::from_value(value).map_err(|_| corrupt())?;
    let human: HumanPrincipalId = outcome.principal_id.parse().map_err(|_| corrupt())?;
    if outcome.tenant_id != tenant.to_string()
        || outcome.bootstrap_request_id != key.to_string()
        || outcome.principal_id != human.to_string()
        || !canonical_v7(human.as_uuid())
        || outcome.kind != "human"
        || outcome.name != name
        || outcome.boundary_id != format!("bnd_{tenant}")
        || outcome.grant_id != format!("grt_{human}")
        || outcome.replayed
    {
        return Err(corrupt());
    }
    Ok(outcome)
}

async fn replay(
    tx: &mut TenantTransaction<'_>,
    key: RequestId,
    tenant: TenantId,
    name: &str,
) -> Result<Json<EnrollResponse>, ControlApiError> {
    let (bound_name, version, value): (String, i16, serde_json::Value) = sqlx::query_as(
        "SELECT request_name, outcome_version, outcome FROM tenant_bootstrap_requests \
         WHERE request_id = $1 AND tenant_id = $2",
    )
    .bind(key.to_string())
    .bind(tenant.to_string())
    .fetch_optional(tx.connection(tenant, GuardMode::Exclusive)?)
    .await?
    .ok_or_else(corrupt)?;
    if version != 1 {
        return Err(corrupt());
    }
    let outcome = decode(value, key, tenant, &bound_name)?;
    // Check immutable identity/parent bindings, without policy decoding or
    // status/time checks: a frozen grant must not erase its creation receipt.
    let bound: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM tenants t \
         JOIN human_principals h ON h.tenant_id = t.tenant_id \
         JOIN enrollments e ON e.tenant_id = t.tenant_id AND e.principal_id = h.principal_id \
         JOIN authority_grants g ON g.tenant_id = t.tenant_id AND g.subject_id = h.principal_id \
         JOIN enrollment_boundaries b ON b.tenant_id = t.tenant_id AND b.boundary_id = g.boundary_id \
         WHERE t.tenant_id = $1 AND h.principal_id = $2 AND h.name COLLATE \"C\" = $3 \
         AND e.kind = 'human' AND e.name COLLATE \"C\" = $3 AND g.grant_id = $4 \
         AND g.subject_kind = 'human' AND g.issuer = h.principal_id AND b.boundary_id = $5)",
    )
    .bind(tenant.to_string())
    .bind(&outcome.principal_id)
    .bind(&bound_name)
    .bind(&outcome.grant_id)
    .bind(&outcome.boundary_id)
    .fetch_one(tx.connection(tenant, GuardMode::Exclusive)?)
    .await?;
    if !bound {
        return Err(corrupt());
    }
    if bound_name != name {
        return Err(ControlApiError {
            status: StatusCode::CONFLICT,
            code: "idempotency_conflict",
            message: "bootstrap_request_id is already bound to a different request".into(),
        });
    }
    Ok(Json(EnrollResponse {
        tenant_id: outcome.tenant_id,
        principal_id: outcome.principal_id,
        kind: outcome.kind,
        name: outcome.name,
        boundary_id: Some(outcome.boundary_id),
        grant_id: Some(outcome.grant_id),
        bootstrap_request_id: Some(outcome.bootstrap_request_id),
        replayed: true,
    }))
}
