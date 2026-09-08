//! The cross-domain audit receipts (`PHASE-8.1.4`, ADR-026): the
//! cross-domain actions record a RECEIPT — the remote domain's own
//! digest-pinned reference + the local record it attached to. The
//! receipts CROSS-REFERENCE, never merge: the remote reference is
//! verifiable against the REMOTE domain's records; the local chain stays
//! the local truth (the ADR-022 groundwork's federation form).

use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// The receipt kinds.
pub const KIND_CARD_IMPORT: &str = "card_import";
pub const KIND_AGREEMENT: &str = "agreement";

/// Record one receipt IN the caller's transaction — the cross-domain
/// action and its receipt commit together (the audit trail rides the
/// action, never a follow-up).
pub(crate) async fn record_in_tx<'e, E>(
    mut tx: E,
    tenant_id: &str,
    remote_tenant_id: &str,
    kind: &str,
    remote_ref: &str,
    local_ref: &str,
) -> Result<(), sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    sqlx::query(
        "INSERT INTO cross_domain_receipts \
         (receipt_id, tenant_id, remote_tenant_id, kind, remote_ref, local_ref) \
         VALUES ('xrec_' || gen_random_uuid()::text, $1, $2, $3, $4, $5)",
    )
    .bind(tenant_id)
    .bind(remote_tenant_id)
    .bind(kind)
    .bind(remote_ref)
    .bind(local_ref)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// The stored receipt (the read surface).
#[derive(Debug, Clone, serde::Serialize)]
pub struct StoredReceipt {
    pub receipt_id: String,
    pub tenant_id: String,
    pub remote_tenant_id: String,
    pub kind: String,
    pub remote_ref: String,
    pub local_ref: String,
    pub created_at: DateTime<Utc>,
}

/// The tenant's receipts (oldest first — the read surface).
pub async fn list(pool: &PgPool, tenant_id: &str) -> Result<Vec<StoredReceipt>, sqlx::Error> {
    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            String,
            String,
            String,
            DateTime<Utc>,
        ),
    >(
        "SELECT receipt_id, tenant_id, remote_tenant_id, kind, remote_ref, local_ref, created_at \
         FROM cross_domain_receipts WHERE tenant_id = $1 ORDER BY created_at",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(receipt_id, tenant_id, remote_tenant_id, kind, remote_ref, local_ref, created_at)| {
                StoredReceipt {
                    receipt_id,
                    tenant_id,
                    remote_tenant_id,
                    kind,
                    remote_ref,
                    local_ref,
                    created_at,
                }
            },
        )
        .collect())
}
