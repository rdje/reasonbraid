//! Storage for final administrative effect evidence (`SIGNOFF-REPAIR.3.3.4.7.2`).
//!
//! The representation is `reasonbraid_core`'s; this module is the two ends of its
//! durability: a writer that runs on the caller's ALREADY-GUARDED transaction, so
//! the evidence and the mutation it describes share one commit, and an exact
//! own-tenant reader that filters before it decodes.
//!
//! Corruption must not become a guessed outcome. A stored row this build cannot
//! read is a storage error, exactly as it is for authorization evidence — the
//! reader never invents `applied`, and never downgrades an outcome it does not
//! recognise into one it does.

use chrono::{DateTime, Utc};
use reasonbraid_core::{
    AdministrativeEffectRecord, AdministrativeOperation, AdministrativeOutcome,
    AdministrativeReason, AuthorizationRecordId, TenantId,
};
use serde_json::Value;
use sqlx::{PgConnection, PgPool};

#[derive(sqlx::FromRow)]
struct EffectRow {
    record_id: String,
    tenant_id: String,
    operation: Value,
    submitted_reason: Option<String>,
    outcome: Value,
    effected_at: DateTime<Utc>,
}

const SELECT_EFFECT: &str = "SELECT record_id, tenant_id, operation, submitted_reason, outcome, \
                             effected_at FROM administrative_effects";

fn malformed() -> sqlx::Error {
    sqlx::Error::Protocol("stored administrative effect record is malformed".into())
}

impl TryFrom<EffectRow> for AdministrativeEffectRecord {
    type Error = sqlx::Error;

    fn try_from(row: EffectRow) -> Result<Self, Self::Error> {
        // Each field is decoded through the core codec, so the bounds, the
        // object-only rule and the closed vocabularies are the same on the way
        // out as on the way in. The database CHECK constrains only the `kind`
        // discriminant; everything below it is the codec's to enforce.
        let submitted_reason = row
            .submitted_reason
            .map(AdministrativeReason::new)
            .transpose()
            .map_err(|_| malformed())?;
        Ok(Self {
            record_id: row.record_id.parse().map_err(|_| malformed())?,
            tenant_id: row.tenant_id.parse().map_err(|_| malformed())?,
            operation: serde_json::from_value::<AdministrativeOperation>(row.operation)
                .map_err(|_| malformed())?,
            submitted_reason,
            outcome: serde_json::from_value::<AdministrativeOutcome>(row.outcome)
                .map_err(|_| malformed())?,
            effected_at: row.effected_at,
        })
    }
}

/// Record one admitted request's final local outcome on the caller's OWN
/// transaction.
///
/// ⛔ The caller supplies a connection that is already inside the transaction
/// performing the mutation and already holds that tenant's authority guard. Both
/// halves matter: sharing the transaction is what makes the evidence and the
/// mutation one commit — an evidence failure aborts the protected write rather
/// than leaving it unexplained — and the guard is what orders the whole thing
/// against an authority change. This function takes neither on the caller's
/// behalf, because taking a guard here would be a second acquisition on a
/// connection that already has one.
///
/// The admission named by `record.record_id` must already be committed or being
/// committed in this same transaction; the composite foreign key refuses an
/// effect that cites another tenant's admission, so a writer cannot bind
/// evidence across a tenant boundary even by mistake.
pub async fn record_administrative_effect_in_tx(
    tx: &mut PgConnection,
    record: &AdministrativeEffectRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO administrative_effects \
         (record_id, tenant_id, operation, submitted_reason, outcome, effected_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(record.record_id.to_string())
    .bind(record.tenant_id.to_string())
    .bind(sqlx::types::Json(&record.operation))
    .bind(record.submitted_reason.as_ref().map(|r| r.as_str()))
    .bind(sqlx::types::Json(&record.outcome))
    .bind(record.effected_at)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// The exact own-tenant effect lookup.
///
/// Filter by tenant BEFORE decoding, so a foreign tenant's malformed evidence is
/// indistinguishable from an absent record — the same rule the authorization
/// record lookup follows, and for the same reason: a decode error that only
/// foreign rows can produce is an existence oracle.
///
/// `Ok(None)` means **no outcome was ever recorded** for that admission. It does
/// not mean the operation did nothing, and it does not mean the operation
/// succeeded. Every request admitted before this table existed reads that way,
/// and so does every route that has not yet been migrated onto the writer.
///
/// This is a storage API, not a caller-authorization boundary: the caller admits
/// first and is responsible for proving the reader may see this tenant.
pub async fn load_tenant_administrative_effect(
    pool: &PgPool,
    tenant_id: TenantId,
    record_id: AuthorizationRecordId,
) -> Result<Option<AdministrativeEffectRecord>, sqlx::Error> {
    let row: Option<EffectRow> = sqlx::query_as(&format!(
        "{SELECT_EFFECT} WHERE tenant_id = $1 AND record_id = $2"
    ))
    .bind(tenant_id.to_string())
    .bind(record_id.to_string())
    .fetch_optional(pool)
    .await?;
    row.map(AdministrativeEffectRecord::try_from).transpose()
}
