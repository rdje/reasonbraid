//! The quota/abuse machinery (`PHASE-7.1.3.2`, ADR-034, §16.11): the per-key
//! WINDOWED ceilings — the tenant, the principal, the resolver, the
//! destination — riding the Phase-2 budget pattern: the check runs in the
//! caller's transaction (the use/denial events commit WITH the guarded
//! action), and a refusal is a RECORDED event, never silent.
//!
//! The shipped binding: the per-tenant INVITE bound (the invitation-storm
//! surface). The check is FAIL-CLOSED — a scope with no configured quota is
//! the typed `quota_unconfigured` refusal (the bound must exist before the
//! surface is usable). The migration 0047 backfill gives every existing
//! tenant the dev default; new tenants get it from the enroll path.

use chrono::{DateTime, Utc};

/// The scope kinds (ADR-034's vocabulary).
pub const SCOPE_TENANT: &str = "tenant";
pub const SCOPE_PRINCIPAL: &str = "principal";
pub const SCOPE_RESOLVER: &str = "resolver";
pub const SCOPE_DESTINATION: &str = "destination";

/// The complete vocabulary (the check validates its argument against it).
pub const SCOPE_KINDS: [&str; 4] = [
    SCOPE_TENANT,
    SCOPE_PRINCIPAL,
    SCOPE_RESOLVER,
    SCOPE_DESTINATION,
];

/// The dev-profile default bound (the invite-storm quota): 1000 per hour.
pub const DEV_DEFAULT_INVITE_CEILING: i64 = 1000;
pub const DEV_DEFAULT_WINDOW_SECS: i64 = 3600;

/// The dev-profile default bound for the MCP write gate's per-principal
/// scope (`.3.5.1`): 1000 write calls per hour — the call-volume bound,
/// generous for the demo, still a bound.
pub const DEV_DEFAULT_PRINCIPAL_CEILING: i64 = 1000;

/// The dev-profile default bound for the two ACQUISITION scopes
/// (`SIGNOFF-REPAIR.11.14.3.14`): 1000 acquisitions per hour, per tenant, per
/// resolver and per destination. The same shape as the two above, and the same
/// honesty — ⛔ this is not a measured production figure, and `.11.6` forbids
/// proposing one before the population is measured. What this leaf decided is
/// the SHAPE of the bound, not its number.
pub const DEV_DEFAULT_ACQUISITION_CEILING: i64 = 1000;

/// The wildcard scope id: a tenant's DEFAULT bound for a scope whose member
/// space the server does not control (`SIGNOFF-REPAIR.11.14.3.14`).
///
/// ⭐ The two WIRED scopes are seeded per member because their members are
/// created by a path the server controls — a tenant by the enroll transaction, a
/// principal by enrolment and card import. The two unwired ones are not: the
/// resolver space grows at runtime through `POST /v1/resolvers`, and the
/// destination space is the open internet. A fail-closed bound over a space you
/// cannot enumerate in advance is not a bound, it is an outage.
///
/// ⛔ The fail-closed CONTRACT is unchanged: a scope with neither a specific row
/// nor this default row is still the typed `quota_unconfigured` refusal. What
/// changes is that the default is a ROW rather than an absence.
pub const SCOPE_DEFAULT_ID: &str = "*";

/// A quota refusal — always a recorded fact in `quota_events` before the
/// `Exceeded`/`Unconfigured` error is returned.
#[derive(Debug)]
pub enum QuotaError {
    /// The scope has NO configured quota — the typed fail-closed refusal.
    Unconfigured {
        scope_kind: String,
        scope_id: String,
    },
    /// The window's recorded uses reached the ceiling (the denial row is
    /// already written).
    Exceeded { ceiling: i64, window_seconds: i64 },
    /// The storage layer failed — NOT a quota verdict (the caller maps this
    /// to its own internal failure).
    Storage(sqlx::Error),
}

impl std::fmt::Display for QuotaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotaError::Unconfigured {
                scope_kind,
                scope_id,
            } => write!(
                f,
                "no quota is configured for the {scope_kind} scope `{scope_id}` — the surface is refused until a bound exists"
            ),
            QuotaError::Exceeded {
                ceiling,
                window_seconds,
            } => write!(
                f,
                "the quota is exhausted: {ceiling} uses within {window_seconds}s — the denial is recorded"
            ),
            QuotaError::Storage(e) => write!(f, "quota storage failure: {e}"),
        }
    }
}

impl std::error::Error for QuotaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            QuotaError::Storage(e) => Some(e),
            _ => None,
        }
    }
}

/// Insert the dev-profile default quotas for a NEW tenant (the enroll path
/// calls this in the tenant-creation transaction).
pub(crate) async fn insert_defaults_in_tx<'e, E>(
    mut tx: E,
    tenant_id: &str,
) -> Result<(), sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    sqlx::query(
        "INSERT INTO usage_quotas (quota_id, tenant_id, scope_kind, scope_id, ceiling, window_seconds) \
         VALUES ($1, $2, $3, $2, $4, $5)",
    )
    .bind(format!("quo_{tenant_id}_invites"))
    .bind(tenant_id)
    .bind(SCOPE_TENANT)
    .bind(DEV_DEFAULT_INVITE_CEILING)
    .bind(DEV_DEFAULT_WINDOW_SECS)
    .execute(&mut *tx)
    .await?;
    // The two acquisition scopes get a DEFAULT row rather than one per member,
    // because the server controls neither member space
    // (`SIGNOFF-REPAIR.11.14.3.14`). `migrations/0068` backfills the same two
    // rows for tenants that existed before this binding.
    for scope_kind in [SCOPE_RESOLVER, SCOPE_DESTINATION] {
        sqlx::query(
            "INSERT INTO usage_quotas (quota_id, tenant_id, scope_kind, scope_id, ceiling, window_seconds) \
             VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (tenant_id, scope_kind, scope_id) DO NOTHING",
        )
        .bind(format!("quo_{tenant_id}_{scope_kind}_default"))
        .bind(tenant_id)
        .bind(scope_kind)
        .bind(SCOPE_DEFAULT_ID)
        .bind(DEV_DEFAULT_ACQUISITION_CEILING)
        .bind(DEV_DEFAULT_WINDOW_SECS)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}

/// Insert the dev-profile default quota for a NEW principal (the
/// principal-creation paths — the enroll + the card-import — call this in
/// their identity-insert transaction; the `.3.5.1` MCP write gate checks
/// this scope fail-closed: an unbound principal is the typed
/// `quota_unconfigured` refusal).
pub(crate) async fn insert_principal_default_in_tx<'e, E>(
    mut tx: E,
    tenant_id: &str,
    principal_id: &str,
) -> Result<(), sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    sqlx::query(
        "INSERT INTO usage_quotas (quota_id, tenant_id, scope_kind, scope_id, ceiling, window_seconds) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(format!("quo_{principal_id}_writes"))
    .bind(tenant_id)
    .bind(SCOPE_PRINCIPAL)
    .bind(principal_id)
    .bind(DEV_DEFAULT_PRINCIPAL_CEILING)
    .bind(DEV_DEFAULT_WINDOW_SECS)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// The check for a scope whose member space the server does not control
/// (`SIGNOFF-REPAIR.11.14.3.14`): the specific row wins, the tenant's
/// [`SCOPE_DEFAULT_ID`] row is the fallback, and the absence of BOTH is the same
/// fail-closed `quota_unconfigured` refusal [`check_in_tx`] already gives.
///
/// Most-specific-wins is one extra existence probe rather than a second
/// counting path, so the window arithmetic, the recorded `use`/`denial` and the
/// transaction semantics below are reached unchanged whichever row applies.
pub(crate) async fn check_open_scope_in_tx<'e, E>(
    mut tx: E,
    tenant_id: &str,
    scope_kind: &str,
    scope_id: &str,
    at: DateTime<Utc>,
) -> Result<(), QuotaError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let specific: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM usage_quotas \
         WHERE tenant_id = $1 AND scope_kind = $2 AND scope_id = $3)",
    )
    .bind(tenant_id)
    .bind(scope_kind)
    .bind(scope_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(QuotaError::Storage)?;
    let effective = if specific { scope_id } else { SCOPE_DEFAULT_ID };
    check_in_tx(&mut *tx, tenant_id, scope_kind, effective, at).await
}

/// The in-transaction check (the budget-machinery pattern): count the
/// window's recorded USES; under the ceiling, record a `use` and pass; at
/// the ceiling, record a `denial` and refuse. Both rows commit with the
/// caller's transaction — a refusal is never silent, and a rolled-back
/// guarded action rolls its use back too (the count = committed actions).
pub(crate) async fn check_in_tx<'e, E>(
    mut tx: E,
    tenant_id: &str,
    scope_kind: &str,
    scope_id: &str,
    at: DateTime<Utc>,
) -> Result<(), QuotaError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    if !SCOPE_KINDS.contains(&scope_kind) {
        return Err(QuotaError::Unconfigured {
            scope_kind: scope_kind.to_string(),
            scope_id: scope_id.to_string(),
        });
    }
    let row: Option<(String, i64, i64)> = sqlx::query_as(
        "SELECT quota_id, ceiling, window_seconds FROM usage_quotas \
         WHERE tenant_id = $1 AND scope_kind = $2 AND scope_id = $3",
    )
    .bind(tenant_id)
    .bind(scope_kind)
    .bind(scope_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(QuotaError::Storage)?;
    let Some((quota_id, ceiling, window_seconds)) = row else {
        return Err(QuotaError::Unconfigured {
            scope_kind: scope_kind.to_string(),
            scope_id: scope_id.to_string(),
        });
    };

    let window_start = at - chrono::Duration::seconds(window_seconds);
    let recent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM quota_events \
         WHERE quota_id = $1 AND kind = 'use' AND at > $2",
    )
    .bind(&quota_id)
    .bind(window_start)
    .fetch_one(&mut *tx)
    .await
    .map_err(QuotaError::Storage)?;

    let kind = if recent >= ceiling { "denial" } else { "use" };
    sqlx::query("INSERT INTO quota_events (quota_id, kind, at) VALUES ($1, $2, $3)")
        .bind(&quota_id)
        .bind(kind)
        .bind(at)
        .execute(&mut *tx)
        .await
        .map_err(QuotaError::Storage)?;

    if kind == "denial" {
        return Err(QuotaError::Exceeded {
            ceiling,
            window_seconds,
        });
    }
    Ok(())
}
