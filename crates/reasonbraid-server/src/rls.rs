//! The RLS tenant-claim helpers (`PHASE-7.1.3.1`,
//! `docs/decisions/2026-09-08_rls-tenant-claim.md`): the transaction-local
//! `app.tenant_id` GUC that the migration 0046 policies gate on.
//!
//! The claim is TRANSACTION-LOCAL by design (`set_config(..., is_local :=
//! true)` — valid only inside a transaction): a pooled connection must never
//! carry a session-level claim past its transaction, or the tenant would leak
//! across requests — the exact bug class the layer exists to stop.
//!
//! Fail-closed: the policies compare `tenant_id` to the claim's `missing_ok`
//! form, which reads NULL when unset — a path that forgets the claim sees
//! nothing. The dev profile's superuser connection bypasses RLS regardless;
//! the claim-setting is harmless there and binds the moment the app role lands.

use sqlx::PgPool;

/// The GUC name the migration 0046 policies gate on.
pub(crate) const TENANT_CLAIM_GUC: &str = "app.tenant_id";

/// Set the transaction-local tenant claim (the FIRST statement of a
/// transaction touching a protected table). Idempotent within the
/// transaction: the write path's claim and apply steps may both call it.
pub(crate) async fn set_tenant_claim<'e, E>(mut tx: E, tenant_id: &str) -> Result<(), sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    sqlx::query("SELECT set_config($1, $2, true)")
        .bind(TENANT_CLAIM_GUC)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;
    Ok(())
}

/// Run a pool-direct tenant-scoped READ on a short transaction with the
/// claim set: BEGIN → set the claim → the closure → COMMIT. The
/// inspection paths that read `aggregate_state`/`event_log` outside the
/// command transaction route through here.
///
/// The boxed-future form lets the closure's future borrow the connection
/// for the transaction's lifetime (the same shape sqlx's own transaction
/// helpers use).
pub(crate) async fn with_tenant_claim<T, F>(
    pool: &PgPool,
    tenant_id: &str,
    f: F,
) -> Result<T, sqlx::Error>
where
    F: for<'c> FnOnce(
        &'c mut sqlx::PgConnection,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<T, sqlx::Error>> + Send + 'c>,
    >,
{
    let mut tx = pool.begin().await?;
    set_tenant_claim(&mut *tx, tenant_id).await?;
    let fut = f(&mut tx);
    let result = fut.await?;
    tx.commit().await?;
    Ok(result)
}
